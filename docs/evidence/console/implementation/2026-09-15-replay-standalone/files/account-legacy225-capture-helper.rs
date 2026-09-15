// Evidence-only instrumentation for the unchanged historical225 seed owner.
// This function is not part of the proposed permanent replay fixture.
async fn capture_legacy225_if_requested(pool: &PgPool, fixture: &LegacyFixture) {
    use sha2::Digest as _;
    use std::io::Write as _;
    use std::os::unix::fs::{OpenOptionsExt as _, PermissionsExt as _};

    let Some(directory) = std::env::var_os("CONSOLE_TEST_LEGACY225_CAPTURE_DIR") else {
        return;
    };
    let directory = std::path::PathBuf::from(directory);
    let metadata = std::fs::symlink_metadata(&directory).expect("capture directory must exist");
    assert!(directory.is_absolute() && metadata.is_dir() && !metadata.file_type().is_symlink());
    assert_eq!(
        metadata.permissions().mode() & 0o077,
        0,
        "capture directory must be private"
    );
    let marker: bool = sqlx::query_scalar(
        "SELECT session_user='console_buck_admin' AND current_user=session_user \
         AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1' \
         AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user)",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(
        marker,
        "capture requires the marked disposable administrator"
    );
    let rows: Value = sqlx::query_scalar(r#"WITH fixture_subjects AS (
    SELECT * FROM public.users WHERE id = ANY($1::uuid[])
), fixture_orgs AS (
    SELECT * FROM public.organizations WHERE id IN (SELECT org_id FROM fixture_subjects)
), fixture_ceremonies AS (
    SELECT * FROM public.auth_webauthn_ceremonies WHERE user_id = ANY($1::uuid[])
)
SELECT jsonb_build_object(
    'database', current_database(),
    'server_version_num', current_setting('server_version_num'),
    'migration_version', (SELECT max(version) FROM public._sqlx_migrations WHERE success),
    'migration_checksums', (SELECT jsonb_agg(jsonb_build_array(version, encode(checksum,'hex')) ORDER BY version)
        FROM public._sqlx_migrations WHERE version <= 225),
    'tables', jsonb_build_object(
        'groups', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.groups t WHERE id IN (SELECT group_id FROM fixture_orgs)),
        'organizations', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_orgs t),
        'group_memberships', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.group_id,t.org_id),'[]'::jsonb)
            FROM public.group_memberships t WHERE org_id IN (SELECT id FROM fixture_orgs)),
        'users', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_subjects t),
        'auth_webauthn_credentials', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_webauthn_credentials t WHERE user_id = ANY($1::uuid[])),
        'auth_webauthn_ceremonies', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb) FROM fixture_ceremonies t),
        'auth_webauthn_ceremony_bindings', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.ceremony_id),'[]'::jsonb)
            FROM public.auth_webauthn_ceremony_bindings t WHERE ceremony_id IN (SELECT id FROM fixture_ceremonies)),
        'auth_refresh_token_families', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_refresh_token_families t WHERE user_id = ANY($1::uuid[])),
        'auth_refresh_tokens', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.auth_refresh_tokens t WHERE user_id = ANY($1::uuid[])),
        'audit_events', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY t.id),'[]'::jsonb)
            FROM public.audit_events t WHERE id = ANY($2::uuid[]))
    ),
    'unused_fixture_edges', jsonb_build_object(
        'user_branches', (SELECT count(*) FROM public.user_branches WHERE user_id=ANY($1::uuid[])),
        'group_role_grants', (SELECT count(*) FROM public.group_role_grants WHERE user_id=ANY($1::uuid[]))
    )
)"#)
        .bind(&fixture.subjects).bind(&fixture.audit_ids).fetch_one(pool).await.unwrap();
    assert_eq!(rows["migration_version"], 225);
    assert_eq!(fixture.subjects.len(), 2);
    assert!(
        rows["unused_fixture_edges"] == json!({"user_branches":0,"group_role_grants":0}),
        "new fixture edge requires explicit capture review"
    );
    for (name, expected) in [
        ("groups", 2),
        ("organizations", 2),
        ("group_memberships", 2),
        ("users", 2),
        ("auth_webauthn_credentials", 2),
        ("auth_refresh_token_families", 2),
        ("auth_refresh_tokens", 4),
    ] {
        assert_eq!(
            rows["tables"][name].as_array().unwrap().len(),
            expected,
            "capture roster mismatch"
        );
    }
    assert_eq!(
        rows["tables"]["audit_events"].as_array().unwrap().len(),
        fixture.audit_ids.len()
    );
    assert_eq!(
        rows["migration_checksums"].as_array().unwrap().len(),
        fixture.old_checksums.len()
    );
    for (row, (version, checksum)) in rows["migration_checksums"]
        .as_array()
        .unwrap()
        .iter()
        .zip(&fixture.old_checksums)
    {
        assert!(
            row == &json!([version, hex::encode(checksum)]),
            "capture checksum differs"
        );
    }
    let revoked_hash = sha2::Sha256::digest(fixture.revoked_token.as_bytes()).to_vec();
    let revoked: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM public.auth_refresh_tokens t \
         JOIN public.auth_refresh_token_families f ON f.id=t.family_id \
         WHERE t.token_hash=$1 AND t.user_id=ANY($2::uuid[]) \
         AND t.revoked_at IS NOT NULL AND f.revoked_at IS NOT NULL)",
    )
    .bind(&revoked_hash)
    .bind(&fixture.subjects)
    .fetch_one(pool)
    .await
    .unwrap();
    assert!(
        revoked,
        "raw exported TEST_ONLY token must already be revoked and hash-correlated"
    );
    let descriptor = rows["database"].as_str().unwrap();
    let suffix = descriptor
        .strip_prefix("_sqlx_test_")
        .expect("disposable database name");
    assert!(
        suffix.len() == 52
            && suffix
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    );
    let output = directory.join(format!("{descriptor}.json"));
    let payload = json!({
        "kind":"TEST_ONLY_HISTORICAL225_OWNER_OUTPUT",
        "format_version":1,
        "producer_base_sha":"1bb508a28e43fef188d52f88e11fcb5b0d0c4dbf",
        "subjects":fixture.subjects,
        "audit_ids":fixture.audit_ids,
        "credential_count":fixture.credential_count,
        "material_snapshot":fixture.snapshot,
        "revoked_token":fixture.revoked_token,
        "revoked_token_sha256":hex::encode(revoked_hash),
        "capture":rows
    });
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(output)
        .expect("exclusive private capture file");
    serde_json::to_writer_pretty(&mut file, &payload).expect("write synthetic capture");
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
    // No fixture bytes or raw token enter stdout; original caller continues
    // into actual current migration and all of its existing assertions.
}
