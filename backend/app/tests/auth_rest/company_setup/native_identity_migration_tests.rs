use super::*;

// Include in company_setup alongside other proposed tests. The resource below
// is the complete future atomic schema+guards+ACL+custody successor, NOT the
// shape-only SQL. Missing resource/fixture is a prerequisite failure, not RED.
const NATIVE_IDENTITY_COMPLETE_RESOURCE: &str =
    include_str!("../../../../../ops/postgres-company-native-identity.sql");

async fn genuine_legacy_policy_fixture(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    use console_identity_application::{
        CreatePolicyRoleCommand, PolicyRoleCondition, PolicyRolePermission,
        UpdatePolicyRoleStatusCommand,
    };
    let business = login_test_pool(pool, TestDatabaseLogin::Business).await;
    // Populate only the frozen preextension SQL-owner surfaces. Calling the
    // evolving onboard_tenant Rust API here would reintroduce its obsolete
    // unbound actor input after the guarded mutation cutover.
    let (org, user) = legacy_preextension_company_user(pool, &business).await;
    let role = console_platform_request_context::scope_org(OrgId::from_uuid(org), async {
        let store = console_identity_adapter_postgres::PgOrgStore::new(business.clone());
        let now = OffsetDateTime::now_utc();
        let role = store
            .create_policy_role(CreatePolicyRoleCommand {
                actor: UserId::from_uuid(user),
                role_key: "legacy_migration_role".into(),
                display_name: "기존 검토 역할".into(),
                description: Some("보존되는 이력".into()),
                permissions: vec![PolicyRolePermission {
                    feature_key: "login".into(),
                    permission_level: "allow".into(),
                }],
                conditions: vec![PolicyRoleCondition {
                    condition_key: "team_scope".into(),
                    attribute: "team".into(),
                    operator: "equals".into(),
                    values: vec!["migration-team".into()],
                }],
                trace: TraceContext::generate(),
                occurred_at: now,
            })
            .await
            .expect("actual legacy policy role owner");
        store
            .update_policy_role_status(UpdatePolicyRoleStatusCommand {
                actor: UserId::from_uuid(user),
                role_id: role.id,
                status: "ACTIVE".into(),
                trace: TraceContext::generate(),
                occurred_at: now,
            })
            .await
            .unwrap();
        role.id
    })
    .await;
    legacy_policy_replace(&business, org, user, vec![role]).await;
    business.close().await;
    (org, user, role)
}

// Historical SQL-owner population, not a complete tenant-onboarding journey.
// No current guarded owner, native authority, fixture DML or privilege grant.
async fn legacy_preextension_company_user(observer: &PgPool, business: &PgPool) -> (Uuid, Uuid) {
    let mut tx = business.begin().await.unwrap();
    let login: (String, String, bool, bool) = sqlx::query_as(
        "SELECT session_user::text,current_user::text,rolsuper,rolbypassrls FROM pg_roles WHERE rolname=current_user",
    ).fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(
        login,
        ("console_rt".into(), "console_rt".into(), false, false)
    );
    let historical: bool = sqlx::query_scalar(
        "SELECT to_regclass('public.company_enrollment_effect_bindings') IS NULL AND to_regprocedure('public.company_enrollment_execute_v1(uuid,uuid,uuid,bytea,text,text)') IS NULL AND NOT EXISTS(SELECT 1 FROM pg_attribute WHERE attrelid='public.organizations'::regclass AND attname='origin_account_id' AND NOT attisdropped)",
    ).fetch_one(tx.as_mut()).await.unwrap();
    assert!(
        historical,
        "legacy owner population requires frozen preextension schema"
    );
    let slug = format!("legacy-{}", Uuid::new_v4().simple());
    let org: Uuid = sqlx::query_scalar("SELECT public.platform_create_organization($1,$2)")
        .bind(&slug)
        .bind("기존 회사")
        .fetch_one(tx.as_mut())
        .await
        .unwrap();
    assert!(!org.is_nil());
    let group: Uuid = sqlx::query_scalar("SELECT group_id FROM public.organizations WHERE id=$1 AND slug=$2 AND name='기존 회사' AND status='ACTIVE'")
        .bind(org).bind(&slug).fetch_one(tx.as_mut()).await.unwrap();
    assert!(!group.is_nil());
    let membership: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM public.group_memberships WHERE org_id=$1 AND group_id=$2",
    )
    .bind(org)
    .bind(group)
    .fetch_one(tx.as_mut())
    .await
    .unwrap();
    assert_eq!(membership, 1);
    let user: Option<Uuid> =
        sqlx::query_scalar("SELECT public.platform_create_group_account($1,$2,$3,$4,$5,$6,$7)")
            .bind(group)
            .bind(org)
            .bind("Legacy preservation administrator")
            .bind(Option::<String>::None)
            .bind(vec!["SUPER_ADMIN"])
            .bind("GROUP_ADMIN")
            .bind(Option::<Uuid>::None)
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    let user = user.expect("actual retained group-account owner must create a User");
    assert!(!user.is_nil());
    let actual: (Uuid, Vec<String>, bool) =
        sqlx::query_as("SELECT org_id,roles,is_active FROM public.users WHERE id=$1")
            .bind(user)
            .fetch_one(tx.as_mut())
            .await
            .unwrap();
    assert_eq!(actual, (org, vec!["SUPER_ADMIN".into()], true));
    let grants: i64 = sqlx::query_scalar("SELECT count(*) FROM public.group_role_grants WHERE group_id=$1 AND user_id=$2 AND group_role='GROUP_ADMIN' AND granted_by IS NULL")
        .bind(group).bind(user).fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(grants, 1);
    tx.commit().await.unwrap();

    // Preserve nonempty credential AND genuine owning-operation audit coverage.
    // This retained API calls auth_legacy_bootstrap_issue_v1 and appends its own
    // auth.bootstrap.issue audit in the same transaction. Never fabricate the
    // lost platform.tenant.create audit or write credential/audit rows here.
    let now = OffsetDateTime::now_utc();
    let issue = console_platform_provisioning::BootstrapCredentialStore
        .issue_for_zero_credential_user(
            business,
            user,
            OrgId::from_uuid(org),
            now,
            Duration::minutes(10),
        )
        .await
        .expect("actual retained credential issuance and audit owner");
    let credential_id = issue.credential_id;
    assert_eq!(issue.user_id, user);
    assert_eq!(issue.expires_at, now + Duration::minutes(10));
    drop(issue); // Synthetic OTP is neither logged nor needed by this fixture.

    // Independent existing bootstrap observer only reads persisted results.
    // The full all_rows migration census below preserves these exact bytes.
    let reopened = login_test_pool(observer, TestDatabaseLogin::Business).await;
    let mut tx = reopened.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let reopened_pair: (Uuid,Uuid) = sqlx::query_as(
        "SELECT o.group_id,u.id FROM public.organizations o JOIN public.users u ON u.org_id=o.id WHERE o.id=$1 AND u.id=$2 AND u.is_active AND u.roles=ARRAY['SUPER_ADMIN']::text[]",
    ).bind(org).bind(user).fetch_one(tx.as_mut()).await.unwrap();
    assert_eq!(reopened_pair, (group, user));
    tx.commit().await.unwrap();
    reopened.close().await;
    let credential: (Uuid,Uuid,bool,bool,bool,i32) = sqlx::query_as(
        "SELECT org_id,user_id,issued_at=$3,expires_at=$4,consumed_at IS NULL AND revoked_at IS NULL,octet_length(token_hash) FROM public.auth_bootstrap_credentials WHERE id=$1 AND user_id=$2",
    ).bind(credential_id).bind(user).bind(now).bind(now+Duration::minutes(10))
        .fetch_one(observer).await.unwrap();
    assert_eq!(credential, (org, user, true, true, true, 32));
    let audits: Vec<(Option<Uuid>,String,Value)> = sqlx::query_as(
        "SELECT actor,target_type,after_snap FROM public.audit_events WHERE org_id=$1 AND action='auth.bootstrap.issue' AND target_id=$2",
    ).bind(org).bind(user.to_string()).fetch_all(observer).await.unwrap();
    assert_eq!(audits.len(), 1);
    assert_eq!(audits[0].0, None);
    assert_eq!(audits[0].1, "auth_bootstrap_credential");
    assert_eq!(audits[0].2["user_id"], json!(user));
    assert_eq!(
        audits[0]
            .2
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["user_id", "expires_at"])
    );
    (org, user)
}

async fn legacy_policy_replace(business: &PgPool, org: Uuid, user: Uuid, role_ids: Vec<Uuid>) {
    use console_identity_application::{
        CreatePolicyAssignmentPreviewReceiptCommand, ReplacePolicyRoleAssignmentsCommand,
    };
    console_platform_request_context::scope_org(OrgId::from_uuid(org), async {
        let store = console_identity_adapter_postgres::PgOrgStore::new(business.clone());
        let who = UserId::from_uuid(user);
        let now = OffsetDateTime::now_utc();
        let current = store.list_policy_role_assignments(who).await.unwrap();
        let version = store.get_policy_version().await.unwrap().version;
        let preview = store
            .create_policy_assignment_preview_receipt(CreatePolicyAssignmentPreviewReceiptCommand {
                actor: who,
                user_id: who,
                current_branch_ids: vec![],
                current_system_roles: vec!["SUPER_ADMIN".into()],
                current_role_ids: current.iter().map(|r| r.role_id).collect(),
                branch_ids: vec![],
                system_roles: vec!["SUPER_ADMIN".into()],
                role_ids: role_ids.clone(),
                policy_version: version,
                expires_at: now + Duration::minutes(5),
            })
            .await
            .unwrap();
        let assigned = store
            .replace_policy_role_assignments(ReplacePolicyRoleAssignmentsCommand {
                actor: who,
                user_id: who,
                role_ids: role_ids.clone(),
                preview_receipt_id: preview.id,
                trace: TraceContext::generate(),
                occurred_at: now,
            })
            .await
            .unwrap();
        assert_eq!(
            assigned.iter().map(|r| r.role_id).collect::<BTreeSet<_>>(),
            role_ids.into_iter().collect()
        );
    })
    .await;
}

pub(super) fn legacy_values_preserved(
    before: &BTreeMap<String, String>,
    after: &BTreeMap<String, String>,
) -> bool {
    use serde_json::value::RawValue;
    let roots = |raw: &str, extras: &[&str]| -> Option<Vec<Vec<(String, String)>>> {
        let rows: Vec<&RawValue> = serde_json::from_str(raw).ok()?;
        let mut projected = Vec::new();
        for row in rows {
            let mut columns: BTreeMap<String, &RawValue> = serde_json::from_str(row.get()).ok()?;
            for extra in extras {
                let value = columns.remove(*extra)?;
                if matches!(*extra, "subject_protocol" | "attribution_protocol") {
                    if value.get() != r#""LEGACY_USER""# {
                        return None;
                    }
                } else if matches!(*extra, "membership_id" | "incarnation") {
                    let id: String = serde_json::from_str(value.get()).ok()?;
                    let parsed: Uuid = id.parse().ok()?;
                    if parsed.is_nil() || parsed.hyphenated().to_string() != id {
                        return None;
                    }
                } else if *extra == "current_revision" {
                    if value.get() != "1" {
                        return None;
                    }
                } else if value.get() != "null" {
                    return None;
                }
            }
            projected.push(
                columns
                    .into_iter()
                    .map(|(key, value)| (key, value.get().to_owned()))
                    .collect(),
            );
        }
        projected.sort();
        Some(projected)
    };
    for (table, old) in before {
        let Some(new) = after.get(table) else {
            return false;
        };
        let extras: &[&str] = match table.as_str() {
            "policy_roles" => &[
                "subject_protocol",
                "native_current_revision",
                "created_by_account_id",
                "updated_by_account_id",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "user_role_assignments" => &[
                "subject_protocol",
                "account_id",
                "native_current_revision",
                "assigned_by_account_id",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "organizations" | "groups" => &[
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "group_memberships" => &["membership_id", "current_revision", "incarnation"],
            "ont_object_types" | "ont_object_policies" => &[
                "attribution_protocol",
                "created_by_account_id",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "ont_builtin_catalog_installs" => &[
                "attribution_protocol",
                "installed_by_account_id",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "cedar_policy_catalog_entries" => &[
                "attribution_protocol",
                "created_by_account_id",
                "updated_by_account_id",
                "origin_account_id",
                "origin_command_id",
                "origin_receipt_id",
            ],
            "ont_builtin_catalog_allowlist" => {
                if !catalog_allowlist_preserved(old, new) {
                    return false;
                }
                continue;
            }
            _ => {
                if old != new {
                    return false;
                }
                continue;
            }
        };
        let (Some(old), Some(new)) = (roots(old, &[]), roots(new, extras)) else {
            return false;
        };
        if old != new {
            return false;
        }
    }
    true
}

#[test]
fn identity_preservation_oracle_keeps_historical_number_bytes() {
    // Pure oracle controls only; these are not database/authority fixtures.
    for (old, new) in [
        (
            r#"[{"amount":18446744073709551616}]"#,
            r#"[{"amount":18446744073709551617}]"#,
        ),
        (
            r#"[{"amount":0.123456789012345678901}]"#,
            r#"[{"amount":0.123456789012345678902}]"#,
        ),
    ] {
        let before = BTreeMap::from([("unchanged_history".to_owned(), old.to_owned())]);
        assert!(legacy_values_preserved(&before, &before));
        let after = BTreeMap::from([("unchanged_history".to_owned(), new.to_owned())]);
        assert!(
            !legacy_values_preserved(&before, &after),
            "oracle collapsed distinct exact-number history"
        );
        let old_root = old.replace("amount", "root_value");
        let additions = r#", "subject_protocol":"LEGACY_USER","native_current_revision":null,"created_by_account_id":null,"updated_by_account_id":null,"origin_account_id":null,"origin_command_id":null,"origin_receipt_id":null"#;
        let extend = |raw: &str| raw.replace("}]", &format!("{additions}}}]"));
        let before = BTreeMap::from([("policy_roles".to_owned(), old_root.clone())]);
        let after = BTreeMap::from([("policy_roles".to_owned(), extend(&old_root))]);
        assert!(legacy_values_preserved(&before, &after));
        let changed = BTreeMap::from([(
            "policy_roles".to_owned(),
            extend(&new.replace("amount", "root_value")),
        )]);
        assert!(
            !legacy_values_preserved(&before, &changed),
            "root projection collapsed retained numeric bytes"
        );
    }
}

#[sqlx::test(migrations = false)]
async fn identity_atomic_extension_preserves_populated_legacy_roots_and_owner_mutations(
    pool: PgPool,
) {
    // Deliberately invoke the retained pre-extension production prerequisite;
    // root must keep this helper pinned to its source-locked original resource.
    prepare_company_preextension_database(&pool).await;
    let absent:bool=sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM pg_attribute WHERE attrelid='public.policy_roles'::regclass AND attname='subject_protocol' AND NOT attisdropped)")
        .fetch_one(&pool).await.unwrap();
    assert!(
        absent,
        "populated migration requires exact pre-extension fixture, not already upgraded database"
    );
    let (org, user, role) = genuine_legacy_policy_fixture(&pool).await;
    populate_legacy_catalog(&pool, org, user).await;
    let registry_before = legacy_catalog_shape(&pool).await;
    let before = all_rows(&pool).await;
    let catalog_before = legacy_identity_catalog(&pool).await;
    for table in [
        "policy_roles",
        "user_role_assignments",
        "policy_role_permissions",
        "policy_role_conditions",
    ] {
        assert!(
            !identity_rows(&before, table, org).is_empty(),
            "populated legacy prerequisite absent: {table}"
        );
    }
    let migration_lower: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let mut tx = pool.begin().await.unwrap();
    sqlx::raw_sql(NATIVE_IDENTITY_COMPLETE_RESOURCE)
        .execute(tx.as_mut())
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let after = all_rows(&pool).await;
    let migration_upper: OffsetDateTime = sqlx::query_scalar("SELECT clock_timestamp()")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_catalog_migration(&pool, &before, &after, migration_lower, migration_upper).await;
    assert!(legacy_catalog_shape_preserved(
        &registry_before,
        &legacy_catalog_shape(&pool).await
    ));
    assert!(
        legacy_values_preserved(&before, &after),
        "extension rewrote populated legacy values/history"
    );
    assert_topology_backfill(&before, &after);
    assert_native_topology_metadata(&pool).await;
    assert!(
        legacy_identity_catalog_preserved(&catalog_before, &legacy_identity_catalog(&pool).await),
        "extension changed legacy type/default/FK/owner/ACL beyond exact reviewed additions"
    );
    assert!(identity_catalog_matches(
        &identity_catalog_observation(&pool).await
    ));
    let catalog = identity_catalog_contract();
    for table in catalog["new_identity_tables"]
        .as_array()
        .unwrap()
        .iter()
        .chain(catalog["new_ontology_mapping_tables"].as_array().unwrap())
    {
        let table = table.as_str().unwrap();
        assert_eq!(
            after[table], "[]",
            "schema extension fabricated native success"
        );
    }
    let mut omitted = after.clone();
    omitted.insert("policy_role_conditions".into(), "[]".into());
    assert!(
        !legacy_values_preserved(&before, &omitted),
        "preservation oracle accepted omitted legacy conditions"
    );
    let mut changed = after.clone();
    let mut roots: Vec<Value> = serde_json::from_str(&changed["policy_roles"]).unwrap();
    roots.iter_mut().find(|r| r["id"] == json!(role)).unwrap()["created_at"] =
        json!("2000-01-01T00:00:00+00:00");
    changed.insert(
        "policy_roles".into(),
        serde_json::to_string(&roots).unwrap(),
    );
    assert!(
        !legacy_values_preserved(&before, &changed),
        "preservation oracle accepted rewritten legacy time"
    );
    legacy_catalog_replay_after_extension(&pool, org, user).await;
    legacy_catalog_attachment_after_extension(&pool, org, user).await;
    let business = login_test_pool(&pool, TestDatabaseLogin::Business).await;
    legacy_policy_replace(&business, org, user, vec![]).await;
    legacy_policy_replace(&business, org, user, vec![role]).await;
    console_platform_request_context::scope_org(OrgId::from_uuid(org), async {
        use console_identity_application::UpdatePolicyRoleCommand;
        let store = console_identity_adapter_postgres::PgOrgStore::new(business.clone());
        let listed = store.list_policy_roles().await.unwrap();
        let previous = listed.into_iter().find(|r| r.id == role).unwrap();
        let edited = store
            .update_policy_role(UpdatePolicyRoleCommand {
                actor: UserId::from_uuid(user),
                role_id: role,
                display_name: "수정된 기존 역할".into(),
                description: previous.description,
                permissions: previous.permissions,
                conditions: previous.conditions,
                trace: TraceContext::generate(),
                occurred_at: OffsetDateTime::now_utc(),
            })
            .await
            .unwrap();
        assert_eq!(edited.display_name, "수정된 기존 역할");
    })
    .await;
    let mut tx = business.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_org',$1,true)")
        .bind(org.to_string())
        .execute(tx.as_mut())
        .await
        .unwrap();
    let denied =
        sqlx::query("UPDATE public.user_role_assignments SET user_id=NULL WHERE org_id=$1")
            .bind(org)
            .execute(tx.as_mut())
            .await
            .expect_err("legacy User nullability weakened");
    assert_eq!(
        denied.as_database_error().and_then(|e| e.code()).as_deref(),
        Some("23514")
    );
    tx.rollback().await.unwrap();
    business.close().await;
}

async fn legacy_identity_catalog(pool: &PgPool) -> Value {
    sqlx::query_scalar(r#"
      SELECT jsonb_agg(jsonb_build_object('table',c.relname,'owner',pg_get_userbyid(c.relowner),
        'rls',c.relrowsecurity,'force_rls',c.relforcerowsecurity,
        'acl',(SELECT jsonb_agg(jsonb_build_object('role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
          'privilege',a.privilege_type,'grantable',a.is_grantable) ORDER BY a.grantee,a.privilege_type)
          FROM aclexplode(coalesce(c.relacl,acldefault('r',c.relowner))) a),
        'columns',(SELECT jsonb_agg(jsonb_build_object('name',p.attname,'type',format_type(p.atttypid,p.atttypmod),
          'not_null',p.attnotnull,'default',pg_get_expr(d.adbin,d.adrelid),
          'acl',(SELECT coalesce(jsonb_agg(jsonb_build_object('role',CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
             'privilege',a.privilege_type,'grantable',a.is_grantable) ORDER BY a.grantee,a.privilege_type),'[]'::jsonb) FROM aclexplode(p.attacl) a)) ORDER BY p.attnum)
          FROM pg_attribute p LEFT JOIN pg_attrdef d ON d.adrelid=p.attrelid AND d.adnum=p.attnum
          WHERE p.attrelid=c.oid AND p.attnum>0 AND NOT p.attisdropped),
        'constraints',(SELECT jsonb_agg(jsonb_build_object('name',k.conname,'kind',k.contype,'definition',pg_get_constraintdef(k.oid,true),
          'validated',k.convalidated,'deferrable',k.condeferrable,'deferred',k.condeferred) ORDER BY k.conname)
          FROM pg_constraint k WHERE k.conrelid=c.oid)) ORDER BY c.relname)
      FROM pg_class c WHERE c.relnamespace='public'::regnamespace
        AND c.relname=ANY(ARRAY['policy_roles','user_role_assignments','policy_role_permissions','policy_role_conditions'])
    "#).fetch_one(pool).await.unwrap()
}

fn legacy_identity_catalog_preserved(before: &Value, after: &Value) -> bool {
    let contract = identity_catalog_contract();
    let check = || -> Option<()> {
        let before = before.as_array()?;
        let after = after.as_array()?;
        if before.len() != 4 || after.len() != 4 {
            return None;
        }
        for old in before {
            let table = old["table"].as_str()?;
            let new = after.iter().find(|r| r["table"] == table)?;
            for key in ["owner", "rls", "force_rls"] {
                if old[key] != new[key] {
                    return None;
                }
            }
            let mut acl = old["acl"].as_array()?.clone();
            if ["policy_roles", "user_role_assignments"].contains(&table) {
                acl.extend(["SELECT", "REFERENCES"].map(
                    |p| json!({"role":"console_account_owner","privilege":p,"grantable":false}),
                ));
            }
            let sorted = |rows: Vec<Value>| {
                let mut v: Vec<_> = rows.into_iter().map(|r| r.to_string()).collect();
                v.sort();
                v.dedup();
                v
            };
            if sorted(acl) != sorted(new["acl"].as_array()?.clone()) {
                return None;
            }
            for old_column in old["columns"].as_array()? {
                let name = old_column["name"].as_str()?;
                let column = new["columns"]
                    .as_array()?
                    .iter()
                    .find(|c| c["name"] == name)?;
                let mut expected = old_column.clone();
                if table == "user_role_assignments" && name == "user_id" {
                    expected["not_null"] = json!(false)
                }
                let mut acl = expected["acl"].as_array()?.clone();
                if let Some(grants) = contract["root_column_acl_additions"][table][name].as_object()
                {
                    for (role, privileges) in grants {
                        for privilege in privileges.as_array()? {
                            acl.push(json!({"role":role,"privilege":privilege,"grantable":false}));
                        }
                    }
                }
                expected["acl"] = json!(sorted(acl));
                let mut actual = column.clone();
                actual["acl"] = json!(sorted(actual["acl"].as_array()?.clone()));
                if expected != actual {
                    return None;
                }
            }
            for constraint in old["constraints"].as_array()? {
                if table == "user_role_assignments"
                    && constraint["kind"] == "n"
                    && constraint["definition"] == "NOT NULL user_id"
                {
                    continue;
                }
                if !new["constraints"].as_array()?.contains(constraint) {
                    return None;
                }
            }
        }
        Some(())
    };
    check().is_some()
}

// Append in native_identity_migration_tests.rs; no historical source changes.
#[sqlx::test(migrations = false)]
async fn identity_atomic_extension_rejects_nil_and_nonfinite_topology_preimage_without_partial_upgrade(
    pool: PgPool,
) {
    prepare_company_preextension_database(&pool).await;
    let (org, _, _) = genuine_legacy_policy_fixture(&pool).await;
    let clean = all_rows(&pool).await;
    let created_at: String =
        sqlx::query_scalar("SELECT created_at::text FROM public.group_memberships WHERE org_id=$1")
            .bind(org)
            .fetch_one(&pool)
            .await
            .unwrap();
    let routine_snapshot = "SELECT coalesce(jsonb_agg(to_jsonb(p) ORDER BY p.oid),'[]'::jsonb)::text FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE n.nspname !~ '^pg_' AND n.nspname<>'information_schema'";
    for fault in ["nil_group", "infinite_attachment_time"] {
        // These are explicit privileged corruption fixtures on the real pinned
        // predecessor, never an alternative successful enrollment/history path.
        if fault == "nil_group" {
            sqlx::query("INSERT INTO public.groups(id,slug,name) VALUES('00000000-0000-0000-0000-000000000000'::uuid,'preimage-nil-group','Invalid preimage probe')")
                .execute(&pool).await.unwrap();
        } else {
            let changed=sqlx::query("UPDATE public.group_memberships SET created_at='infinity'::timestamptz WHERE org_id=$1")
                .bind(org).execute(&pool).await.unwrap();
            assert_eq!(changed.rows_affected(), 1);
        }
        let before = all_rows(&pool).await;
        let routines_before: String = sqlx::query_scalar(routine_snapshot)
            .fetch_one(&pool)
            .await
            .unwrap();
        let catalog_before = legacy_identity_catalog(&pool).await;
        let mut tx = pool.begin().await.unwrap();
        let failed = sqlx::raw_sql(NATIVE_IDENTITY_COMPLETE_RESOURCE)
            .execute(tx.as_mut())
            .await
            .expect_err("corrupt topology preimage must refuse complete atomic extension");
        let error = failed
            .as_database_error()
            .expect("actual preflight database refusal");
        assert_eq!(error.code().as_deref(), Some("23514"));
        assert_eq!(
            error.message(),
            "company_enrollment.topology_preimage_invalid"
        );
        tx.rollback().await.unwrap();
        assert_eq!(
            before,
            all_rows(&pool).await,
            "failed migration changed original rows, columns or table census"
        );
        assert_eq!(catalog_before, legacy_identity_catalog(&pool).await);
        assert_eq!(
            routines_before,
            sqlx::query_scalar::<_, String>(routine_snapshot)
                .fetch_one(&pool)
                .await
                .unwrap(),
            "failed migration retained routine/profile changes"
        );
        if fault == "nil_group" {
            let changed = sqlx::query(
                "DELETE FROM public.groups WHERE id='00000000-0000-0000-0000-000000000000'::uuid",
            )
            .execute(&pool)
            .await
            .unwrap();
            assert_eq!(changed.rows_affected(), 1);
        } else {
            sqlx::query(
                "UPDATE public.group_memberships SET created_at=$2::timestamptz WHERE org_id=$1",
            )
            .bind(org)
            .bind(&created_at)
            .execute(&pool)
            .await
            .unwrap();
        }
        assert_eq!(
            clean,
            all_rows(&pool).await,
            "negative fixture cleanup changed authentic predecessor"
        );
    }
}
