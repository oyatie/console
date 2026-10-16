//! Mixed-format persistence test, using the existing SET ROLE RLS fixture.
//! This does not claim actual restricted-LOGIN or deployment qualification.
use super::*;
use base64::Engine as _;
use secrecy::ExposeSecret;

const LEGACY_ORG: Uuid = Uuid::from_u128(0x11111111_1111_1111_1111_111111111111);
const LEGACY_ACCOUNT: Uuid = Uuid::from_u128(0x22222222_2222_2222_2222_222222222222);
const SMTP_OLD: &[u8] = b"legacy-smtp-password";
const IMAP_OLD: &[u8] = b"legacy-imap-password";
const SMTP_NEW: &[u8] = b"aes-smtp-after-upgrade";
const IMAP_NEW: &[u8] = b"aes-imap-after-upgrade";

fn frozen_legacy(field: &str) -> SealedCredential {
    // Captured from the actual old product executable before AES changes.
    // Never generate or reseal historical fixtures with the new writer.
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/credential-legacy-v1.json")).unwrap();
    assert_eq!(rows.len(), 2);
    let matching: Vec<_> = rows.iter().filter(|r| r["field"] == field).collect();
    assert_eq!(matching.len(), 1);
    let row = matching[0];
    let decode = |name| {
        base64::engine::general_purpose::STANDARD
            .decode(row[name].as_str().unwrap())
            .unwrap()
    };
    let result = SealedCredential {
        ciphertext: decode("ciphertext"),
        nonce: decode("nonce"),
        dek_wrapped: decode("dek_wrapped"),
        dek_nonce: decode("dek_nonce"),
        key_version: row["key_version"].as_str().unwrap().parse().unwrap(),
    };
    assert_eq!(result.nonce.len(), 24);
    assert_eq!(result.dek_nonce.len(), 24);
    assert_eq!(result.dek_wrapped.len(), 48);
    assert_eq!(result.key_version, 1);
    result
}

fn mail_upsert(
    account: EmailAccountId,
    actor: UserId,
    smtp: Option<SealedCredential>,
    imap: Option<SealedCredential>,
) -> AccountUpsert {
    AccountUpsert {
        id: account,
        actor,
        display_name: "Historical mailbox".into(),
        email_address: "legacy@example.test".into(),
        from_name: Some("Legacy".into()),
        imap_host: "imap.example.test".into(),
        imap_port: 993,
        imap_security: MailSecurity::SslTls,
        imap_username: "legacy".into(),
        smtp_host: "smtp.example.test".into(),
        smtp_port: 587,
        smtp_security: MailSecurity::StartTls,
        smtp_username: "legacy".into(),
        smtp_password: smtp,
        imap_password: imap,
    }
}

fn assert_passwords(stored: &console_comms_application::StoredAccount, smtp: &[u8], imap: &[u8]) {
    // Fresh reader instance also rules out relying on the encrypting object's
    // transient state to choose the format or remember its DEK.
    let reader = EnvelopeCredentialCipher::from_key_bytes(&[7u8; 32]).unwrap();
    for (field, sealed, expected) in [
        ("smtp_password", &stored.smtp_password, smtp),
        ("imap_password", &stored.imap_password, imap),
    ] {
        let clear = reader
            .decrypt(
                sealed,
                Aad {
                    org_id: "11111111-1111-1111-1111-111111111111",
                    account_id: "22222222-2222-2222-2222-222222222222",
                    field,
                },
            )
            .unwrap();
        assert_eq!(clear.expose_secret().as_slice(), expected);
        assert_eq!(
            sealed.key_version, 1,
            "algorithm must not consume shared KEK version"
        );
    }
    let view = serde_json::to_string(&stored.to_view()).unwrap();
    for forbidden in [
        "smtp_password_ct",
        "imap_password_ct",
        "dek_wrapped",
        "legacy-smtp-password",
        "legacy-imap-password",
        "aes-smtp-after-upgrade",
        "aes-imap-after-upgrade",
    ] {
        assert!(
            !view.contains(forbidden),
            "write-only view disclosed credential material"
        );
    }
}

fn assert_aes(sealed: &SealedCredential) {
    assert_eq!(sealed.nonce.len(), 12);
    assert_eq!(sealed.dek_nonce.len(), 12);
    assert_eq!(sealed.dek_wrapped.len(), 49);
    assert_eq!(sealed.dek_wrapped[0], 2);
    assert_eq!(sealed.key_version, 1);
}

async fn raw_bundles(pool: &PgPool, account: EmailAccountId) -> [SealedCredential; 2] {
    // Independent physical-row comparison under the same explicit owner
    // observer used by the original tests. No plaintext columns are added.
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL row_security=off")
        .execute(tx.as_mut())
        .await
        .unwrap();
    type RawCredentialRow = (
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        Vec<u8>,
        i16,
    );
    let row: RawCredentialRow = sqlx::query_as("SELECT smtp_password_ct,smtp_password_nonce,dek_wrapped,dek_nonce,imap_password_ct,imap_password_nonce,imap_dek_wrapped,imap_dek_nonce,key_version FROM public.email_accounts WHERE id=$1")
        .bind(*account.as_uuid()).fetch_one(tx.as_mut()).await.unwrap();
    tx.commit().await.unwrap();
    [
        SealedCredential {
            ciphertext: row.0,
            nonce: row.1,
            dek_wrapped: row.2,
            dek_nonce: row.3,
            key_version: row.8,
        },
        SealedCredential {
            ciphertext: row.4,
            nonce: row.5,
            dek_wrapped: row.6,
            dek_nonce: row.7,
            key_version: row.8,
        },
    ]
}

async fn reopen(
    owner_pool: &PgPool,
    org: OrgId,
) -> (
    PgPool,
    PgMailStore,
    console_comms_application::StoredAccount,
) {
    let runtime = runtime_role_pool(owner_pool).await;
    let role: (String, bool, bool) = sqlx::query_as(
        "SELECT current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user",
    )
    .fetch_one(&runtime)
    .await
    .unwrap();
    assert_eq!(role, ("console_rt".into(), false, false));
    let store = PgMailStore::new(runtime.clone());
    let stored = CURRENT_ORG
        .scope(org, store.get_account())
        .await
        .unwrap()
        .unwrap();
    (runtime, store, stored)
}

#[sqlx::test(migrations = "../../platform/db/migrations")]
async fn legacy_mailbox_reopens_then_smtp_only_and_imap_only_updates_preserve_other_bundle(
    owner_pool: PgPool,
) {
    let org = OrgId::from_uuid(LEGACY_ORG);
    let account = EmailAccountId::from_uuid(LEGACY_ACCOUNT);
    seed_org(&owner_pool, LEGACY_ORG, "aes-legacy").await;
    let actor = seed_active_user(&owner_pool, LEGACY_ORG).await;
    let legacy_smtp = frozen_legacy("smtp_password");
    let legacy_imap = frozen_legacy("imap_password");
    let runtime = runtime_role_pool(&owner_pool).await;
    let store = PgMailStore::new(runtime.clone());
    let audit = account_config_audit_event(
        actor,
        account,
        TraceContext::generate(),
        OffsetDateTime::now_utc(),
    )
    .unwrap()
    .with_org(org);
    let first = CURRENT_ORG
        .scope(
            org,
            store.upsert_account(
                mail_upsert(
                    account,
                    actor,
                    Some(legacy_smtp.clone()),
                    Some(legacy_imap.clone()),
                ),
                audit,
            ),
        )
        .await
        .unwrap();
    assert_eq!(first.smtp_password, legacy_smtp);
    assert_eq!(first.imap_password, legacy_imap);
    assert_passwords(&first, SMTP_OLD, IMAP_OLD);
    let initial = raw_bundles(&owner_pool, account).await;
    assert_eq!(initial, [legacy_smtp.clone(), legacy_imap.clone()]);
    drop(store);
    runtime.close().await;

    let (runtime, store, reopened) = reopen(&owner_pool, org).await;
    assert_passwords(&reopened, SMTP_OLD, IMAP_OLD);
    assert_eq!(
        raw_bundles(&owner_pool, account).await,
        initial,
        "historical read/reopen silently resealed data"
    );
    let writer = EnvelopeCredentialCipher::from_key_bytes(&[7u8; 32]).unwrap();
    let smtp = seal(&writer, org, account, "smtp_password", SMTP_NEW);
    assert_aes(&smtp);
    let audit = account_config_audit_event(
        actor,
        account,
        TraceContext::generate(),
        OffsetDateTime::now_utc(),
    )
    .unwrap()
    .with_org(org);
    let mixed = CURRENT_ORG
        .scope(
            org,
            store.upsert_account(mail_upsert(account, actor, Some(smtp.clone()), None), audit),
        )
        .await
        .unwrap();
    assert_eq!(mixed.smtp_password, smtp);
    assert_eq!(mixed.imap_password, legacy_imap);
    assert_passwords(&mixed, SMTP_NEW, IMAP_OLD);
    let after_smtp = raw_bundles(&owner_pool, account).await;
    assert_eq!(after_smtp, [smtp.clone(), initial[1].clone()]);
    drop(store);
    runtime.close().await;

    let (runtime, store, reopened) = reopen(&owner_pool, org).await;
    assert_passwords(&reopened, SMTP_NEW, IMAP_OLD);
    assert_eq!(
        raw_bundles(&owner_pool, account).await,
        after_smtp,
        "mixed-format reopen mutated an untouched secret"
    );
    let imap = seal(&writer, org, account, "imap_password", IMAP_NEW);
    assert_aes(&imap);
    let audit = account_config_audit_event(
        actor,
        account,
        TraceContext::generate(),
        OffsetDateTime::now_utc(),
    )
    .unwrap()
    .with_org(org);
    let upgraded = CURRENT_ORG
        .scope(
            org,
            store.upsert_account(mail_upsert(account, actor, None, Some(imap.clone())), audit),
        )
        .await
        .unwrap();
    assert_eq!(upgraded.smtp_password, smtp);
    assert_eq!(upgraded.imap_password, imap);
    assert_passwords(&upgraded, SMTP_NEW, IMAP_NEW);
    let after_imap = raw_bundles(&owner_pool, account).await;
    assert_eq!(after_imap, [after_smtp[0].clone(), imap]);
    drop(store);
    runtime.close().await;

    let (runtime, store, reopened) = reopen(&owner_pool, org).await;
    assert_passwords(&reopened, SMTP_NEW, IMAP_NEW);
    assert_eq!(raw_bundles(&owner_pool, account).await, after_imap);
    assert_eq!(
        audit_count(&owner_pool, "email.account.configure", &account.to_string()).await,
        3
    );
    drop(store);
    runtime.close().await;
}
