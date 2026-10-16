// Included only inside deployment_operator_designation. Additive assurance;
// original twelve tests and v6 contract stay byte-identical.
mod startup_effective_privileges {
    use super::*;

    // Original native15 + users/organizations/groups, supplemented by the
    // owner-only topology/grants and canonical Person/Employee source contracts.
    const BUSINESS_TABLES: &[&str] = &[
        "employees",
        "persons",
        "person_revisions",
        "employee_person_bindings",
        "group_memberships",
        "group_role_grants",
    ];
    const TABLE_RIGHTS: &[&str] = &[
        "SELECT",
        "INSERT",
        "UPDATE",
        "DELETE",
        "TRUNCATE",
        "REFERENCES",
        "TRIGGER",
        "MAINTAIN",
    ];
    const COLUMN_RIGHTS: &[&str] = &["SELECT", "INSERT", "UPDATE", "REFERENCES"];
    // Exact mandatory identities prevent a removed routine or changed owner
    // from silently shrinking the catalog-derived protected routine population.
    const MANDATORY_ROUTINES: &[&str] = &[
        "public.account_context_presence_v1(uuid)",
        "public.account_login_consent_v1(uuid)",
        "public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])",
        "public.account_registration_begin_v1()",
        "public.account_security_lock_exclusive_v1(uuid)",
        "public.account_security_lock_shared_v1(uuid)",
        "public.account_session_logout_v1(uuid,uuid,bigint,interval)",
        "public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)",
        "public.account_session_shared_material_v1(uuid,uuid)",
        "public.account_terms_current_v1()",
        "public.account_terms_registration_head_v1()",
        "public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)",
        "public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)",
        "public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)",
        "public.auth_account_session_shared_material_v1(uuid,uuid)",
        "public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)",
        "public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)",
        "public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)",
        "public.auth_legacy_cold_start_admin_v1()",
        "public.auth_legacy_company_lock_v1(uuid)",
        "public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)",
        "public.auth_legacy_group_passkey_flag_v1(uuid)",
        "public.auth_legacy_purge_company_v1(uuid)",
        "public.auth_legacy_purge_subjects_v1(uuid)",
        "public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)",
        "public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)",
        "public.auth_legacy_self_passkey_count_v1(uuid,uuid)",
        "public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)",
        "public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)",
        "public.auth_legacy_self_passkeys_v1(uuid,uuid)",
        "public.auth_legacy_session_context_v1(uuid,uuid)",
        "public.auth_legacy_user_active_v1(uuid,uuid)",
        "public.auth_legacy_user_has_passkey_v1(uuid,uuid)",
        "public.group_member_org_ids(uuid,uuid)",
        "public.group_role_grants_for_user(uuid)",
        "public.platform_assign_org_to_group(uuid,uuid)",
        "public.platform_attach_group_of_one(uuid)",
        "public.platform_attach_membership(uuid,uuid)",
        "public.platform_create_group(text,text)",
        "public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)",
        "public.platform_create_organization(text,text)",
        "public.platform_force_remove_direct_org_children(uuid)",
        "public.platform_force_remove_organization(uuid)",
        "public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)",
        "public.platform_get_group(uuid)",
        "public.platform_get_organization(uuid)",
        "public.platform_list_group_accounts(uuid)",
        "public.platform_list_groups()",
        "public.platform_list_organizations()",
        "public.platform_mint_group_row(uuid,text,text)",
        "public.platform_mint_missing_group_of_one(uuid)",
        "public.platform_remove_org_from_group(uuid,uuid)",
        "public.platform_remove_organization(uuid)",
        "public.platform_resolve_bootstrap_org(bytea)",
        "public.platform_resolve_credential_org(text)",
        "public.platform_resolve_token_org(bytea)",
        "public.platform_revoke_group_role(uuid,uuid,text)",
        "public.platform_set_organization_status(uuid,text)",
        "public.platform_update_group(uuid,text,text,text)",
    ];

    fn protected_tables() -> Vec<&'static str> {
        TABLES.iter().chain(BUSINESS_TABLES).copied().collect()
    }

    async fn positive_designation(pool: &PgPool, runtime: &PgPool, account: Uuid) -> Designation {
        let input = designation(pool, account).await;
        let receipt = designate(runtime, &input)
            .await
            .expect("actual startup designation positive control");
        assert!(!receipt.0.is_nil() && receipt.1 == 1 && !receipt.2);
        let stored: (Uuid, Uuid, i64, String) = sqlx::query_as(
            "SELECT h.account_id,r.command_id,h.revision,r.kind FROM public.deployment_operator_head h JOIN public.deployment_operator_receipts r ON r.receipt_id=h.receipt_id WHERE r.receipt_id=$1"
        ).bind(receipt.0).fetch_one(pool).await.unwrap();
        assert_eq!(stored, (account, input.command, 1, "DESIGNATE".into()));
        input
    }

    async fn positive_revocation(pool: &PgPool, runtime: &PgPool, input: &Designation) {
        let command = Uuid::new_v4();
        let receipt = revoke(runtime, input, command, 1, "startup capability acceptance")
            .await
            .expect("actual startup revocation positive control");
        assert!(!receipt.0.is_nil() && receipt.1 == 2 && !receipt.2);
        let stored: (Uuid, Uuid, i64, String) = sqlx::query_as(
            "SELECT h.account_id,r.command_id,h.revision,r.kind FROM public.deployment_operator_head h JOIN public.deployment_operator_receipts r ON r.receipt_id=h.receipt_id WHERE r.receipt_id=$1"
        ).bind(receipt.0).fetch_one(pool).await.unwrap();
        assert_eq!(stored, (input.account, command, 2, "REVOKE".into()));
    }

    async fn actual_denial(runtime: &PgPool, statement: String) {
        let mut tx = runtime.begin().await.unwrap();
        let result = sqlx::query(sqlx::AssertSqlSafe(statement))
            .execute(&mut *tx)
            .await;
        tx.rollback().await.unwrap();
        refused(result, "42501", None);
    }

    #[sqlx::test(migrations = false)]
    async fn startup_login_has_no_effective_protected_table_or_column_privileges(pool: PgPool) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (account, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let tables = protected_tables();
        assert_eq!(tables.len(), 24);
        let untouched = rows(&pool, &tables).await;
        let input = positive_designation(&pool, &runtime, account.account).await;
        let history = operator_rows(&pool).await;
        for table in &tables {
            let qualified = format!("public.{table}");
            let ordinary: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND c.relname=$1 AND c.relkind='r')"
            ).bind(table).fetch_one(&pool).await.unwrap();
            assert!(
                ordinary,
                "PREREQUISITE: expected protected ordinary table {table}"
            );
            let columns: Vec<String> = sqlx::query_scalar(
                "SELECT attname::text FROM pg_catalog.pg_attribute WHERE attrelid=pg_catalog.to_regclass($1) AND attnum>0 AND NOT attisdropped ORDER BY attnum"
            ).bind(&qualified).fetch_all(&pool).await.unwrap();
            assert!(
                !columns.is_empty(),
                "PREREQUISITE: expected columns for {table}"
            );
            // PostgreSQL effective checks include PUBLIC, role inheritance and
            // implicit ownership; direct ACL rows alone are not the oracle.
            let rights: Vec<(String, bool)> = sqlx::query_as(
                "SELECT right_name,pg_catalog.has_table_privilege(current_user,pg_catalog.to_regclass($1),right_name) FROM unnest($2::text[]) AS rights(right_name)"
            ).bind(&qualified).bind(TABLE_RIGHTS).fetch_all(&runtime).await.unwrap();
            assert_eq!(rights.len(), TABLE_RIGHTS.len());
            for (right, permitted) in rights {
                assert!(!permitted, "startup effective table {right} on {table}");
            }
            let rights: Vec<(String, String, bool)> = sqlx::query_as(
                "SELECT a.attname::text,right_name,pg_catalog.has_column_privilege(current_user,a.attrelid,a.attnum,right_name) FROM pg_catalog.pg_attribute a CROSS JOIN unnest($2::text[]) AS rights(right_name) WHERE a.attrelid=pg_catalog.to_regclass($1) AND a.attnum>0 AND NOT a.attisdropped ORDER BY a.attnum,right_name"
            ).bind(&qualified).bind(COLUMN_RIGHTS).fetch_all(&runtime).await.unwrap();
            assert_eq!(rights.len(), columns.len() * COLUMN_RIGHTS.len());
            for (column, right, permitted) in rights {
                assert!(
                    !permitted,
                    "startup effective column {right} on {table}.{column}"
                );
            }
            for column in &columns {
                let quoted = column.replace('"', "\"\"");
                actual_denial(
                    &runtime,
                    format!("SELECT \"{quoted}\" FROM {qualified} WHERE false"),
                )
                .await;
            }
            let first = columns[0].replace('"', "\"\"");
            for statement in [
                format!("INSERT INTO {qualified} DEFAULT VALUES"),
                format!("UPDATE {qualified} SET \"{first}\"=\"{first}\" WHERE false"),
                format!("DELETE FROM {qualified} WHERE false"),
                format!("TRUNCATE {qualified}"),
            ] {
                actual_denial(&runtime, statement).await;
            }
        }
        // REFERENCES/TRIGGER/MAINTAIN are catalog-effective coverage. Do not
        // mislabel lack of CREATE/schema access as proof of those privileges.
        assert!(operator_rows(&pool).await == history && rows(&pool, &tables).await == untouched);
        positive_revocation(&pool, &runtime, &input).await;
        assert!(rows(&pool, &tables).await == untouched);
        runtime.close().await;
    }

    // Every argument is nonnull and explicitly typed, so STRICT short-circuit
    // and overload inference cannot turn a skipped call into a denial witness.
    // Values need type validity, not invented Company authority: anything other
    // than42501 (including domain validation) means the denied owner was reached.
    fn denied_call(signature: &str, account: Uuid) -> String {
        let (name, arguments) = signature.split_once('(').unwrap();
        let arguments = arguments.strip_suffix(')').unwrap();
        let arguments = if arguments.is_empty() {
            Vec::new()
        } else {
            arguments
                .split(',')
                .map(|kind| match kind {
                    "uuid" => format!("'{account}'::uuid"),
                    "text" => "'startup-denial-control'::text".into(),
                    "bigint" => "1::bigint".into(),
                    "bytea" => "decode(repeat('00',32),'hex')::bytea".into(),
                    "text[]" => "ARRAY['MEMBER']::text[]".into(),
                    "bytea[]" => "ARRAY[decode(repeat('00',32),'hex')]::bytea[]".into(),
                    "interval" => "interval '5 minutes'".into(),
                    "timestamp with time zone" => "CURRENT_TIMESTAMP::timestamptz".into(),
                    "character" => "'0'::character".into(),
                    "jsonb" => "'{}'::jsonb".into(),
                    "boolean" => "false::boolean".into(),
                    _ => panic!("PREREQUISITE: unhandled fixed signature argument {kind}"),
                })
                .collect::<Vec<String>>()
        };
        format!("SELECT * FROM {name}({})", arguments.join(","))
    }

    async fn actual_function_denial(runtime: &PgPool, signature: &str, account: Uuid) {
        let mut tx = runtime.begin().await.unwrap();
        let result = sqlx::query(sqlx::AssertSqlSafe(denied_call(signature, account)))
            .execute(&mut *tx)
            .await;
        tx.rollback().await.unwrap();
        let Err(error) = result else {
            panic!("startup unexpectedly executed protected owner")
        };
        let database = error
            .as_database_error()
            .expect("database permission denial, not transport");
        let name = signature
            .strip_prefix("public.")
            .unwrap()
            .split_once('(')
            .unwrap()
            .0;
        assert!(
            database.code().as_deref() == Some("42501")
                && database.message() == format!("permission denied for function {name}"),
            "must be function EXECUTE refusal, not a SECURITY DEFINER body's own42501"
        );
    }

    #[sqlx::test(migrations = false)]
    async fn startup_login_executes_only_exact_designation_owners_in_protected_routines(
        pool: PgPool,
    ) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (account, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let tables = protected_tables();
        let untouched = rows(&pool, &tables).await;
        let input = positive_designation(&pool, &runtime, account.account).await;
        let history = operator_rows(&pool).await;
        for signature in MANDATORY_ROUTINES {
            let shape: Option<(String, bool, bool)> = sqlx::query_as(
                "SELECT p.prokind::text,p.prosecdef,p.prorettype IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) FROM pg_catalog.pg_proc p WHERE p.oid=pg_catalog.to_regprocedure($1)"
            ).bind(signature).fetch_optional(&pool).await.unwrap();
            let (kind, definer, trigger) =
                shape.expect("PREREQUISITE: mandatory exact existing routine");
            assert!(
                kind == "f" && definer && !trigger,
                "PREREQUISITE: mandatory callable SECURITY DEFINER function {signature}"
            );
            let permitted: bool = sqlx::query_scalar(
                "SELECT pg_catalog.has_function_privilege(current_user,pg_catalog.to_regprocedure($1),'EXECUTE')"
            ).bind(signature).fetch_one(&runtime).await.unwrap();
            assert!(!permitted, "startup effective EXECUTE on {signature}");
            actual_function_denial(&runtime, signature, account.account).await;
        }
        // Include *all* overloads, protected-owner functions outside public, and
        // public SECURITY DEFINER routines even when owned by console_app. The
        // only allowlist entries are two exact OIDs resolved from full signatures.
        // Ordinary pg_catalog/PUBLIC builtins are intentionally outside scope.
        let population: Vec<(String, String, bool, bool)> = sqlx::query_as(
            "SELECT p.oid::regprocedure::text,p.prokind::text,pg_catalog.has_function_privilege(current_user,p.oid,'EXECUTE'),p.oid=ANY(ARRAY[pg_catalog.to_regprocedure($1)::oid,pg_catalog.to_regprocedure($2)::oid]) FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace WHERE p.proowner IN (SELECT oid FROM pg_catalog.pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.prosecdef) ORDER BY n.nspname,p.proname,p.oid"
        ).bind(DESIGNATE).bind(REVOKE).fetch_all(&runtime).await.unwrap();
        assert!(
            population.len() >= MANDATORY_ROUTINES.len() + 2,
            "protected routine population cannot shrink"
        );
        let mut allowed = 0;
        for (signature, kind, permitted, allowlisted) in population {
            assert!(
                kind == "f" || kind == "p",
                "PREREQUISITE: classified function/procedure {signature}"
            );
            assert_eq!(
                permitted, allowlisted,
                "startup exact effective EXECUTE boundary for {signature}"
            );
            if allowlisted {
                allowed += 1;
                assert_eq!(kind, "f");
            }
        }
        assert_eq!(allowed, 2);
        assert!(operator_rows(&pool).await == history && rows(&pool, &tables).await == untouched);
        positive_revocation(&pool, &runtime, &input).await;
        assert!(rows(&pool, &tables).await == untouched);
        runtime.close().await;
    }

    async fn oracle_acl_snapshot(pool: &PgPool) -> (String, String, String) {
        sqlx::query_as(
            "SELECT COALESCE(c.relacl::text,'NULL'),COALESCE(a.attacl::text,'NULL'),COALESCE(p.proacl::text,'NULL') FROM pg_catalog.pg_class c JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attname='id' JOIN pg_catalog.pg_proc p ON p.oid='public.account_registration_begin_v1()'::regprocedure WHERE c.oid='public.accounts'::regclass"
        ).fetch_one(pool).await.unwrap()
    }
    const ORACLE_RIGHTS: &str = "SELECT pg_catalog.has_table_privilege('console_auth_startup','public.accounts','SELECT'),pg_catalog.has_column_privilege('console_auth_startup','public.accounts','id','UPDATE'),pg_catalog.has_function_privilege('console_auth_startup','public.account_registration_begin_v1()','EXECUTE')";

    #[sqlx::test(migrations = false)]
    async fn effective_oracles_detect_public_inheritance_without_direct_startup_grants(
        pool: PgPool,
    ) {
        let app = fixture(&pool).await;
        owner_exists(&pool).await;
        let (account, _) = enrolled(&app).await;
        let runtime = startup(&pool).await;
        let tables = protected_tables();
        let untouched = rows(&pool, &tables).await;
        let input = positive_designation(&pool, &runtime, account.account).await;
        let history = operator_rows(&pool).await;
        let before = oracle_acl_snapshot(&pool).await;
        let baseline: (bool, bool, bool) = sqlx::query_as(ORACLE_RIGHTS)
            .fetch_one(&runtime)
            .await
            .unwrap();
        assert_eq!(baseline, (false, false, false));
        let mut tx = pool.begin().await.unwrap();
        // Negative ACL fault injection, never a positive authority fixture.
        // A second LOGIN cannot see uncommitted GRANT. The injected-positive
        // oracle therefore evaluates the named startup role in this owner txn;
        // baseline, after-rollback and all real denials use the actual LOGIN.
        for statement in [
            "GRANT SELECT ON public.accounts TO PUBLIC",
            "GRANT UPDATE(id) ON public.accounts TO PUBLIC",
            "GRANT EXECUTE ON FUNCTION public.account_registration_begin_v1() TO PUBLIC",
        ] {
            sqlx::query(statement).execute(&mut *tx).await.unwrap();
        }
        let injected: (bool, bool, bool) = sqlx::query_as(ORACLE_RIGHTS)
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        let direct: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM (SELECT x.grantee FROM pg_catalog.pg_class c CROSS JOIN LATERAL pg_catalog.aclexplode(c.relacl) x WHERE c.oid='public.accounts'::regclass UNION ALL SELECT x.grantee FROM pg_catalog.pg_attribute a CROSS JOIN LATERAL pg_catalog.aclexplode(a.attacl) x WHERE a.attrelid='public.accounts'::regclass AND a.attname='id' UNION ALL SELECT x.grantee FROM pg_catalog.pg_proc p CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) x WHERE p.oid='public.account_registration_begin_v1()'::regprocedure) grants WHERE grantee=(SELECT oid FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup')"
        ).fetch_one(&mut *tx).await.unwrap();
        tx.rollback().await.unwrap();
        assert_eq!(
            injected,
            (true, true, true),
            "effective oracles must detect inherited PUBLIC rights"
        );
        assert_eq!(
            direct, 0,
            "injected access must not be a direct startup grant"
        );
        assert!(
            oracle_acl_snapshot(&pool).await == before,
            "exact ACL preservation after rollback"
        );
        let after: (bool, bool, bool) = sqlx::query_as(ORACLE_RIGHTS)
            .fetch_one(&runtime)
            .await
            .unwrap();
        assert_eq!(after, (false, false, false));
        actual_denial(
            &runtime,
            "SELECT id FROM public.accounts WHERE false".into(),
        )
        .await;
        actual_denial(
            &runtime,
            "UPDATE public.accounts SET id=id WHERE false".into(),
        )
        .await;
        actual_denial(
            &runtime,
            "SELECT * FROM public.account_registration_begin_v1()".into(),
        )
        .await;
        assert!(operator_rows(&pool).await == history && rows(&pool, &tables).await == untouched);
        positive_revocation(&pool, &runtime, &input).await;
        assert!(rows(&pool, &tables).await == untouched);
        runtime.close().await;
    }
}
