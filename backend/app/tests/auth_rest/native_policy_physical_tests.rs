// Private additive test source. Include as a child of company_setup:
// mod native_policy_physical_tests {
//     include!("company_setup/native_policy_physical_tests.rs");
// }
// Uses existing finalized ready_designated and actual HTTP Company enrollment.
// No product implementation, installer injection, or business fixture inserts.
use super::*;

const INPUTS: &str = "native_company_policy_inputs_v1";
const RECEIPTS: &str = "native_company_policy_receipts_v1";
const PARTICIPANTS: &[&str] = &[
    "policy_roles",
    "policy_role_revisions",
    "user_role_assignments",
    "policy_assignment_revisions",
    "native_company_catalog_installs",
    "ont_builtin_catalog_installs",
    "ont_object_types",
];

async fn assert_policy_tables(pool: &PgPool) {
    let found: Vec<String> = sqlx::query_scalar(
        "SELECT c.relname::text FROM pg_class c WHERE c.relnamespace='public'::regnamespace AND c.relkind='r' AND c.relname ~ '^native_company_policy_' ORDER BY c.relname",
    ).fetch_all(pool).await.unwrap();
    assert_eq!(
        found,
        vec![INPUTS.to_owned(), RECEIPTS.to_owned()],
        "NATIVE_POLICY_PHYSICAL_V1: required two-relation owner contract is absent"
    );
}

async fn columns(pool: &PgPool, table: &str) -> BTreeMap<String, (String, bool, String)> {
    sqlx::query_as::<_, (String, String, bool, String)>(
        "SELECT a.attname::text,format_type(a.atttypid,a.atttypmod),a.attnotnull,a.attgenerated::text FROM pg_attribute a WHERE a.attrelid=to_regclass('public.'||$1) AND a.attnum>0 AND NOT a.attisdropped ORDER BY a.attnum",
    ).bind(table).fetch_all(pool).await.unwrap().into_iter()
        .map(|(name, ty, required, generated)| (name, (ty, required, generated))).collect()
}

async fn assert_key(pool: &PgPool, table: &str, kind: &str, names: &[&str]) {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*) FROM pg_constraint k
        WHERE k.conrelid=to_regclass('public.'||$1) AND k.contype::text=$2
          AND k.convalidated AND NOT k.condeferrable
          AND ARRAY(SELECT a.attname::text FROM unnest(k.conkey) WITH ORDINALITY q(id,n)
            JOIN pg_attribute a ON a.attrelid=k.conrelid AND a.attnum=q.id ORDER BY q.n)=$3::text[]
          AND EXISTS(SELECT 1 FROM pg_index i WHERE i.indexrelid=k.conindid
            AND i.indisvalid AND i.indisready AND i.indpred IS NULL AND NOT i.indnullsnotdistinct)
    "#,
    )
    .bind(table)
    .bind(kind)
    .bind(names)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        count, 1,
        "missing full NULL-distinct {kind} key on {table}: {names:?}"
    );
}

async fn assert_fk(
    pool: &PgPool,
    table: &str,
    names: &[&str],
    target: &str,
    targets: &[&str],
    must_be_deferred: bool,
) {
    let rows: Vec<(bool, bool, String, String, String, bool)> = sqlx::query_as(
        r#"
        SELECT k.condeferrable,k.condeferred,k.confmatchtype::text,
          k.confupdtype::text,k.confdeltype::text,k.convalidated
        FROM pg_constraint k WHERE k.contype='f'
          AND k.conrelid=to_regclass('public.'||$1) AND k.confrelid=to_regclass('public.'||$3)
          AND ARRAY(SELECT a.attname::text FROM unnest(k.conkey) WITH ORDINALITY q(id,n)
            JOIN pg_attribute a ON a.attrelid=k.conrelid AND a.attnum=q.id ORDER BY q.n)=$2::text[]
          AND ARRAY(SELECT a.attname::text FROM unnest(k.confkey) WITH ORDINALITY q(id,n)
            JOIN pg_attribute a ON a.attrelid=k.confrelid AND a.attnum=q.id ORDER BY q.n)=$4::text[]
    "#,
    )
    .bind(table)
    .bind(names)
    .bind(target)
    .bind(targets)
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(
        rows.len(),
        1,
        "missing exact same-scope FK {table} {names:?}"
    );
    let (deferrable, initially_deferred, match_type, update, delete, valid) = &rows[0];
    assert!(
        *valid
            && match_type == "s"
            && matches!(update.as_str(), "a" | "r")
            && matches!(delete.as_str(), "a" | "r"),
        "unsafe FK semantics on {table}"
    );
    if table == RECEIPTS && target == RECEIPTS {
        assert!(
            update == "r" && delete == "r",
            "predecessor history requires RESTRICT"
        );
    }
    // False leaves deferral unspecified; the contract requires it only for cyclic FKs.
    if must_be_deferred {
        assert!(
            *deferrable && *initially_deferred,
            "effect-cycle FK must be deferred: {table}"
        );
    }
}

async fn assert_guard_coverage(pool: &PgPool, table: &str, immutable: bool) {
    let guards: Vec<(i16, bool, bool)> = sqlx::query_as(
        r#"
        SELECT t.tgtype,t.tgdeferrable,t.tginitdeferred FROM pg_trigger t
        WHERE t.tgrelid=to_regclass('public.'||$1) AND NOT t.tgisinternal
          AND t.tgenabled='A' AND t.tgqual IS NULL AND t.tgnargs=0
          AND cardinality(t.tgattr::smallint[])=0
    "#,
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap();
    // PostgreSQL trigger bits: ROW=1 BEFORE=2 INSERT=4 DELETE=8 UPDATE=16 TRUNCATE=32.
    let before = |event| {
        guards
            .iter()
            .any(|(ty, deferred, _)| *ty & 3 == 3 && *ty & event != 0 && !*deferred)
    };
    assert!(
        before(4),
        "missing unconditional ALWAYS INSERT frame guard on {table}"
    );
    assert!(
        guards.iter().any(|(ty, deferred, initially)| *ty & 3 == 1
            && *ty & 4 != 0
            && *deferred
            && *initially),
        "missing deferred ALWAYS insertion closure on {table}"
    );
    if immutable {
        // Unconditional statement guards can reject these mutations before any row.
        // INSERT above still needs the row frame guard and deferred row closure.
        let before_mutation = |event| {
            guards
                .iter()
                .any(|(ty, deferred, _)| *ty & 2 == 2 && *ty & event != 0 && !*deferred)
        };
        assert!(
            before_mutation(8) && before_mutation(16),
            "immutable UPDATE/DELETE guard absent on {table}"
        );
        assert!(
            guards
                .iter()
                .any(|(ty, deferred, _)| *ty & 3 == 2 && *ty & 32 != 0 && !*deferred),
            "immutable TRUNCATE guard absent on {table}"
        );
    }
}

async fn assert_physical_contract(pool: &PgPool) {
    type PolicyRow = (
        String,
        String,
        bool,
        Vec<i64>,
        Option<String>,
        Option<String>,
    );

    assert_policy_tables(pool).await;
    let input_required = [
        ("actor_account_id", "uuid"),
        ("command_id", "uuid"),
        ("org_id", "uuid"),
        ("operation", "smallint"),
        ("codec_version", "smallint"),
        ("input_bytes", "bytea"),
        ("input_digest", "bytea"),
        ("intake_receipt_id", "uuid"),
        ("accepted_at", "timestamp with time zone"),
        ("execution_not_after", "timestamp with time zone"),
        ("accepting_session_id", "uuid"),
        ("acceptance_xid", "xid8"),
        ("acceptance_backend_pid", "integer"),
    ];
    let terminal_required = [
        ("actor_account_id", "uuid"),
        ("command_id", "uuid"),
        ("org_id", "uuid"),
        ("operation", "smallint"),
        ("codec_version", "smallint"),
        ("intake_receipt_id", "uuid"),
        ("input_digest", "bytea"),
        ("receipt_id", "uuid"),
        ("outcome", "text"),
        ("result_code", "text"),
        ("execution_session_id", "uuid"),
        ("executed_at", "timestamp with time zone"),
        ("effect_xid", "xid8"),
        ("effect_backend_pid", "integer"),
        ("epoch_before", "bigint"),
        ("epoch_after", "bigint"),
        ("catalog_version", "text"),
        ("manifest_digest", "bytea"),
    ];
    let terminal_optional = [
        ("predecessor_receipt_id", "uuid"),
        ("committed_epoch", "bigint"),
        ("installed_object_type_id", "uuid"),
        ("recipient_account_id", "uuid"),
        ("role_id", "uuid"),
        ("role_revision", "bigint"),
        ("assignment_id", "uuid"),
        ("assignment_revision_before", "bigint"),
        ("assignment_revision_after", "bigint"),
        ("assignment_state_after", "text"),
        ("assignment_valid_from", "timestamp with time zone"),
        ("assignment_valid_until", "timestamp with time zone"),
    ];
    for (table, required, optional) in [
        (INPUTS, input_required.as_slice(), &[][..]),
        (
            RECEIPTS,
            terminal_required.as_slice(),
            terminal_optional.as_slice(),
        ),
    ] {
        let actual = columns(pool, table).await;
        let expected: BTreeMap<_, _> = required
            .iter()
            .map(|(n, t)| (n.to_string(), (t.to_string(), true, String::new())))
            .chain(optional.iter().map(|(n, t)| {
                (
                    n.to_string(),
                    (
                        t.to_string(),
                        false,
                        if *n == "committed_epoch" { "s" } else { "" }.to_owned(),
                    ),
                )
            }))
            .collect();
        assert_eq!(actual, expected, "closed physical column contract: {table}");
        let metadata: (String, bool, bool) = sqlx::query_as(
            "SELECT pg_get_userbyid(c.relowner)::text,c.relrowsecurity,c.relforcerowsecurity FROM pg_class c WHERE c.oid=to_regclass('public.'||$1)",
        ).bind(table).fetch_one(pool).await.unwrap();
        assert_eq!(metadata, ("console_account_owner".into(), true, true));
        let policies: Vec<PolicyRow> = sqlx::query_as(
            r#"
            SELECT p.polname::text,p.polcmd::text,p.polpermissive,
              ARRAY(SELECT x::bigint FROM unnest(p.polroles) x),
              pg_get_expr(p.polqual,p.polrelid),pg_get_expr(p.polwithcheck,p.polrelid)
            FROM pg_policy p WHERE p.polrelid=to_regclass('public.'||$1) ORDER BY p.polname
        "#,
        )
        .bind(table)
        .fetch_all(pool)
        .await
        .unwrap();
        let predicate =
            "(org_id = (NULLIF(current_setting('app.current_org'::text, true), ''::text))::uuid)";
        assert_eq!(
            policies,
            vec![(
                "org_isolation".into(),
                "*".into(),
                true,
                vec![0],
                Some(predicate.into()),
                Some(predicate.into())
            )]
        );
        assert_guard_coverage(pool, table, true).await;
        assert_key(pool, table, "p", &["actor_account_id", "command_id"]).await;
    }
    assert_key(pool, INPUTS, "u", &["intake_receipt_id"]).await;
    let exact_input = &[
        "actor_account_id",
        "command_id",
        "org_id",
        "operation",
        "codec_version",
        "intake_receipt_id",
        "input_digest",
    ];
    assert_key(pool, INPUTS, "u", exact_input).await;
    assert_fk(
        pool,
        INPUTS,
        &["actor_account_id"],
        "accounts",
        &["id"],
        false,
    )
    .await;
    assert_fk(pool, INPUTS, &["org_id"], "organizations", &["id"], false).await;
    assert_fk(pool, RECEIPTS, exact_input, INPUTS, exact_input, false).await;
    for key in [
        &["receipt_id"][..],
        &["org_id", "receipt_id"],
        &["org_id", "committed_epoch"],
        &["org_id", "receipt_id", "committed_epoch"],
    ] {
        assert_key(pool, RECEIPTS, "u", key).await;
    }
    assert_fk(
        pool,
        RECEIPTS,
        &["org_id", "predecessor_receipt_id", "epoch_before"],
        RECEIPTS,
        &["org_id", "receipt_id", "committed_epoch"],
        false,
    )
    .await;
    for (names, target, targets) in [
        (
            &["org_id", "recipient_account_id"][..],
            "company_actors",
            &["org_id", "account_id"][..],
        ),
        (
            &["org_id", "installed_object_type_id"][..],
            "ont_object_types",
            &["org_id", "id"][..],
        ),
        (
            &["org_id", "role_id", "role_revision"][..],
            "policy_role_revisions",
            &["org_id", "role_id", "revision"][..],
        ),
        (
            &["org_id", "assignment_id", "assignment_revision_after"][..],
            "policy_assignment_revisions",
            &["org_id", "assignment_id", "revision"][..],
        ),
    ] {
        assert_fk(
            pool,
            RECEIPTS,
            names,
            target,
            targets,
            target == "policy_role_revisions" || target == "policy_assignment_revisions",
        )
        .await;
    }
    let forbidden_session_fks: i64 = sqlx::query_scalar(r#"
        SELECT count(*) FROM pg_constraint k
        WHERE k.contype='f' AND k.conrelid=ANY(ARRAY[to_regclass('public.'||$1),to_regclass('public.'||$2)])
          AND k.confrelid IN (to_regclass('public.auth_refresh_token_families'),to_regclass('public.auth_refresh_tokens'))
    "#).bind(INPUTS).bind(RECEIPTS).fetch_one(pool).await.unwrap();
    assert_eq!(
        forbidden_session_fks, 0,
        "historical sessions cannot depend on permanent credentials"
    );
    let pointer_tables: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT c.relname::text FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid
        WHERE c.relnamespace='public'::regnamespace AND c.relkind IN ('r','p')
          AND a.attname='policy_receipt_id' AND a.attnum>0 AND NOT a.attisdropped ORDER BY c.relname
    "#,
    )
    .fetch_all(pool)
    .await
    .unwrap();
    let mut wanted: Vec<_> = PARTICIPANTS.iter().map(|s| s.to_string()).collect();
    wanted.sort();
    assert_eq!(
        pointer_tables, wanted,
        "exact seven business provenance participants"
    );
    for table in PARTICIPANTS {
        assert_eq!(
            columns(pool, table).await.get("policy_receipt_id"),
            Some(&("uuid".into(), false, String::new()))
        );
        assert_fk(
            pool,
            table,
            &["org_id", "policy_receipt_id"],
            RECEIPTS,
            &["org_id", "receipt_id"],
            false,
        )
        .await;
        assert_guard_coverage(pool, table, false).await;
    }
    for table in PARTICIPANTS
        .iter()
        .copied()
        .chain(std::iter::once("company_authority_heads"))
    {
        assert_fk(
            pool,
            table,
            &[
                "org_id",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "company_enrollment_receipts",
            &["org_id", "account_id", "command_id", "receipt_id"],
            true,
        )
        .await;
    }
    assert_eq!(
        columns(pool, "company_authority_heads")
            .await
            .get("current_policy_receipt_id"),
        Some(&("uuid".into(), false, String::new()))
    );
    assert_guard_coverage(pool, "company_authority_heads", true).await;
    assert_fk(
        pool,
        "company_authority_heads",
        &["org_id", "current_policy_receipt_id", "epoch"],
        RECEIPTS,
        &["org_id", "receipt_id", "committed_epoch"],
        true,
    )
    .await;
}

#[sqlx::test(migrations = false)]
async fn native_policy_two_relation_schema_acl_and_birth_preservation(pool: PgPool) {
    let (app, account, cookies, startup, _) = ready_designated(&pool).await;
    let csrf = proof(&app, &cookies).await;
    let command = Uuid::new_v4();
    let input = enrollment(command, account.account);
    let created = committed(
        &submit(&app, &cookies, &csrf, &input).await,
        StatusCode::CREATED,
        command,
        account.account,
        false,
    );
    durable(&pool, &created, &input, account.account).await;
    let before = all_rows(&pool).await;
    assert_physical_contract(&pool).await;
    for table in [INPUTS, RECEIPTS] {
        assert_eq!(
            before[table], "[]",
            "Company birth cannot silently install business policy"
        );
    }
    for table in PARTICIPANTS {
        let query = format!(
            "SELECT count(*) FROM public.{table} WHERE org_id=$1 AND policy_receipt_id IS NOT NULL"
        );
        let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
            .bind(created.company)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 0, "birth provenance was backfilled on {table}");
    }
    let head: (i64,Option<Uuid>) = sqlx::query_as(
        "SELECT epoch,current_policy_receipt_id FROM public.company_authority_heads WHERE org_id=$1",
    ).bind(created.company).fetch_one(&pool).await.unwrap();
    assert_eq!(head, (1, None));
    let owner: (bool,bool,bool) = sqlx::query_as(
        "SELECT rolcanlogin,rolsuper,rolcreaterole FROM pg_roles WHERE rolname='console_account_owner'",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(owner, (false, false, false));
    for login in [
        TestDatabaseLogin::Business,
        TestDatabaseLogin::Auth,
        TestDatabaseLogin::LeaveCommand,
        TestDatabaseLogin::OntologyCommand,
        TestDatabaseLogin::PlatformForceCommand,
    ] {
        let serving = login_test_pool(&pool, login).await;
        for table in [INPUTS, RECEIPTS] {
            let denied: bool = sqlx::query_scalar(
                "SELECT NOT (has_table_privilege(current_user,$1,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN') OR has_any_column_privilege(current_user,$1,'SELECT,INSERT,UPDATE,REFERENCES'))",
            ).bind(format!("public.{table}")).fetch_one(&serving).await.unwrap();
            assert!(denied, "serving LOGIN has raw policy bytes/effects access");
        }
        for owner in [
            "console_account_owner",
            "console_ontology_writer",
            "console_app",
        ] {
            let member: bool = sqlx::query_scalar("SELECT pg_has_role(current_user,$1,'MEMBER')")
                .bind(owner)
                .fetch_one(&serving)
                .await
                .unwrap();
            assert!(!member, "serving LOGIN can become policy owner");
        }
        serving.close().await;
    }
    let saved = request(
        &app,
        "GET",
        &format!("/api/v2/companies/enrollments/{command}"),
        &cookies,
        None,
        &[],
    )
    .await;
    let reopened = committed(&saved, StatusCode::OK, command, account.account, true);
    assert!(created.company == reopened.company && created.receipt == reopened.receipt);
    initial_ceiling(&app, &cookies, &created).await;
    assert!(
        before == all_rows(&pool).await,
        "schema inspection or birth reopen rewrote historical commands, credentials, custody or audit rows"
    );
    startup.close().await;
}

// These vectors evaluate the installed declarative constraints without inserting
// fabricated business state. They do not replace actual owner-transition tests.
async fn receipt_checks_accept(pool: &PgPool, row: &Value) -> bool {
    let checks: Vec<String> = sqlx::query_scalar(
        "SELECT pg_get_expr(conbin,conrelid) FROM pg_constraint WHERE conrelid=to_regclass('public.native_company_policy_receipts_v1') AND contype='c' AND convalidated ORDER BY conname",
    ).fetch_all(pool).await.unwrap();
    assert!(
        !checks.is_empty(),
        "closed terminal shape requires installed CHECK constraints"
    );
    // Trusted catalog expressions only, and ordinary bound data; no raw input SQL.
    let expression = checks
        .iter()
        .map(|s| format!("({s}) IS NOT FALSE"))
        .collect::<Vec<_>>()
        .join(" AND ");
    let query = format!(
        "SELECT {expression} FROM jsonb_populate_record(NULL::public.native_company_policy_receipts_v1,$1::jsonb) r"
    );
    sqlx::query_scalar(sqlx::AssertSqlSafe(query))
        .bind(row)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn rejection_vector(operation: i16, code: &str) -> Value {
    json!({
        "actor_account_id":"00000000-0000-0000-0000-000000000021",
        "command_id":"00000000-0000-0000-0000-000000000022",
        "org_id":"00000000-0000-0000-0000-000000000023",
        "operation":operation,"codec_version":1,
        "intake_receipt_id":"00000000-0000-0000-0000-000000000024",
        "input_digest":format!("\\x{}","01".repeat(32)),
        "receipt_id":"00000000-0000-0000-0000-000000000025",
        "outcome":"REJECTED","result_code":code,
        "execution_session_id":"00000000-0000-0000-0000-000000000026",
        "executed_at":"2026-09-21T00:00:00Z","effect_xid":"1","effect_backend_pid":1,
        "epoch_before":1,"epoch_after":1,"predecessor_receipt_id":null,"committed_epoch":null,
        "catalog_version":"native-payroll-collection-read-v1",
        "manifest_digest":"\\x07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd",
        "installed_object_type_id":null,"recipient_account_id":null,"role_id":null,"role_revision":null,
        "assignment_id":null,"assignment_revision_before":null,"assignment_revision_after":null,
        "assignment_state_after":null,"assignment_valid_from":null,"assignment_valid_until":null
    })
}

#[sqlx::test(migrations = false)]
async fn native_policy_terminal_four_rejections_no_effect_and_successful_epoch_shape(pool: PgPool) {
    let (_app, _account, _cookies, startup, _) = ready_designated(&pool).await;
    assert_policy_tables(&pool).await;
    let before = all_rows(&pool).await;
    let generator: Option<String> = sqlx::query_scalar(
        r#"
        SELECT pg_get_expr(d.adbin,d.adrelid) FROM pg_attribute a
        JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
        WHERE a.attrelid=to_regclass('public.native_company_policy_receipts_v1')
          AND a.attname='committed_epoch' AND a.attgenerated='s'
    "#,
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    let generator = generator.expect("successful-only epoch must be GENERATED ALWAYS STORED");
    for operation in 1..=3 {
        for code in [
            "intake_expired",
            "revision_conflict",
            "grant_expiry_invalid",
            "recipient_ineligible",
        ] {
            let row = rejection_vector(operation, code);
            assert_eq!(
                receipt_checks_accept(&pool, &row).await,
                operation == 2 || matches!(code, "intake_expired" | "revision_conflict"),
                "operation/result mapping admits unsupported rejection or loses a required result"
            );
            let query = format!(
                "SELECT ({generator})::bigint FROM jsonb_populate_record(NULL::public.native_company_policy_receipts_v1,$1::jsonb) r"
            );
            let epoch: Option<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
                .bind(&row)
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(
                epoch, None,
                "rejected receipt can occupy a successful epoch"
            );
        }
        for invalid in [
            "",
            "installed",
            "granted",
            "revoked",
            "stale_company_epoch",
            "stale_assignment_revision",
            "catalog_already_installed",
            "catalog_conflict",
            "catalog_not_installed",
            "assignment_exists",
            "assignment_not_found",
            "assignment_already_active",
            "assignment_not_active",
            "revision_exhausted",
            "UNKNOWN",
        ] {
            assert!(
                !receipt_checks_accept(&pool, &rejection_vector(operation, invalid)).await,
                "unsupported terminal rejection was admitted: {invalid}"
            );
        }
    }
    for (key, value) in [
        ("epoch_after", json!(2)),
        ("predecessor_receipt_id", json!(Uuid::new_v4())),
        ("installed_object_type_id", json!(Uuid::new_v4())),
        ("recipient_account_id", json!(Uuid::new_v4())),
        ("role_id", json!(Uuid::new_v4())),
        ("role_revision", json!(1)),
        ("assignment_id", json!(Uuid::new_v4())),
        ("assignment_revision_before", json!(1)),
        ("assignment_revision_after", json!(2)),
        ("assignment_state_after", json!("ACTIVE")),
        ("assignment_valid_from", json!("2026-09-21T00:00:00Z")),
        ("assignment_valid_until", json!("2026-09-22T00:00:00Z")),
    ] {
        let mut bad = rejection_vector(2, "revision_conflict");
        bad[key] = value;
        assert!(
            !receipt_checks_accept(&pool, &bad).await,
            "rejected receipt permits effect/head field {key}"
        );
    }
    let mut later = rejection_vector(1, "revision_conflict");
    later["epoch_before"] = json!(i64::MAX);
    later["epoch_after"] = json!(i64::MAX);
    later["predecessor_receipt_id"] = json!(Uuid::new_v4());
    assert!(
        receipt_checks_accept(&pool, &later).await,
        "maximum observed head must reject without bigint overflow"
    );
    later["predecessor_receipt_id"] = Value::Null;
    assert!(
        !receipt_checks_accept(&pool, &later).await,
        "later epoch escaped predecessor binding"
    );
    let mut installed = rejection_vector(1, "installed");
    installed["outcome"] = json!("COMMITTED");
    installed["epoch_after"] = json!(2);
    installed["committed_epoch"] = json!(2);
    installed["installed_object_type_id"] = json!(Uuid::new_v4());
    assert!(
        receipt_checks_accept(&pool, &installed).await,
        "required committed install shape rejected"
    );
    let query = format!(
        "SELECT ({generator})::bigint FROM jsonb_populate_record(NULL::public.native_company_policy_receipts_v1,$1::jsonb) r"
    );
    let epoch: Option<i64> = sqlx::query_scalar(sqlx::AssertSqlSafe(query))
        .bind(installed)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(epoch, Some(2));
    assert!(
        before == all_rows(&pool).await,
        "constraint vectors must never persist business fixtures"
    );
    startup.close().await;
}
