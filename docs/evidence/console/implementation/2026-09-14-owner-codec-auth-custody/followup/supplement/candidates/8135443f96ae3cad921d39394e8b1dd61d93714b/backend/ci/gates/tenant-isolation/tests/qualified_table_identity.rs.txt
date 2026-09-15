//! Public scanner regressions, not PostgreSQL execution or arbitrary-SQL proof.
use console_gate_tenant_isolation::{GateResult, ViolationKind, check_migrations_root};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new(sql: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "console-qualified-table-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("migrations")).unwrap();
        fs::write(path.join("migrations/0001_identity.sql"), sql).unwrap();
        Self(path)
    }
    fn check(&self) -> GateResult {
        check_migrations_root(&self.0)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn clean(sql: &str) {
    let result = Fixture::new(sql).check();
    assert!(result.violations.is_empty(), "{:#?}", result.violations);
}
fn violation(sql: &str, kind: ViolationKind, table: &str) {
    let result = Fixture::new(sql).check();
    assert!(
        result
            .violations
            .iter()
            .any(|v| v.kind == kind && v.detail.to_ascii_lowercase().contains(table)),
        "expected {kind:?} for {table}, got {:#?}",
        result.violations
    );
    assert!(
        !result
            .violations
            .iter()
            .any(|v| v.kind == ViolationKind::ScanError)
    );
}
fn protection(table: &str) -> String {
    format!(
        "ALTER TABLE {table} ENABLE ROW LEVEL SECURITY; ALTER TABLE {table} FORCE ROW LEVEL SECURITY; CREATE POLICY org_isolation ON {table} USING (org_id = current_setting('app.current_org', true)::uuid);"
    )
}
#[test]
fn public_global_table_resolves_to_existing_classification() {
    for table in [
        "public.object_types",
        "public . object_types",
        "\"public\" . \"object_types\"",
    ] {
        clean(&format!(
            "CREATE TABLE IF NOT EXISTS {table} (id uuid primary key);"
        ));
    }
}
#[test]
fn mixed_public_create_and_unqualified_rls_share_identity() {
    clean(&format!(
        "CREATE TABLE public.widgets (org_id uuid NOT NULL); {}",
        protection("widgets")
    ));
}
#[test]
fn mixed_unqualified_create_and_quoted_public_rls_share_identity() {
    clean(&format!(
        "CREATE TABLE widgets (org_id uuid NOT NULL); {}",
        protection("\"public\" . \"widgets\"")
    ));
}
#[test]
fn qualified_add_column_and_set_not_null_update_same_table() {
    clean(&format!(
        "CREATE TABLE widgets (id uuid); ALTER TABLE IF EXISTS public.widgets ADD COLUMN org_id uuid; ALTER TABLE \"public\" . \"widgets\" ALTER COLUMN org_id SET NOT NULL; {}",
        protection("widgets")
    ));
}
#[test]
fn qualified_nullable_exception_is_same_public_table() {
    clean(&format!(
        "CREATE TABLE public.audit_events (org_id uuid); {}",
        protection("audit_events")
    ));
}
#[test]
fn qualified_unknown_table_remains_unclassified() {
    violation(
        "CREATE TABLE public.mystery (id uuid);",
        ViolationKind::UnclassifiedTable,
        "mystery",
    );
}
#[test]
fn other_schema_does_not_inherit_global_or_nullable_allowlists() {
    violation(
        "CREATE TABLE private.object_types (id uuid);",
        ViolationKind::UnclassifiedTable,
        "object_types",
    );
    violation(
        &format!(
            "CREATE TABLE private.audit_events (org_id uuid); {}",
            protection("private.audit_events")
        ),
        ViolationKind::NullableOrgIdWithoutAllowlist,
        "audit_events",
    );
}
#[test]
fn same_basename_in_another_schema_cannot_supply_rls_evidence() {
    violation(
        &format!(
            "CREATE TABLE public.widgets (org_id uuid NOT NULL); CREATE TABLE private.widgets (org_id uuid NOT NULL); {}",
            protection("public.widgets")
        ),
        ViolationKind::OrgColumnWithoutRls,
        "private.widgets",
    );
}
#[test]
fn two_tables_in_one_schema_cannot_share_rls_evidence() {
    violation(
        &format!(
            "CREATE TABLE public.protected (org_id uuid NOT NULL); CREATE TABLE public.unprotected (org_id uuid NOT NULL); {}",
            protection("public.protected")
        ),
        ViolationKind::OrgColumnWithoutRls,
        "unprotected",
    );
}
#[test]
fn quoted_case_or_dot_does_not_alias_existing_global_table() {
    for table in [
        "public.\"Object_types\"",
        "\"Public\".object_types",
        "\"public.object_types\"",
    ] {
        violation(
            &format!("CREATE TABLE {table} (id uuid);"),
            ViolationKind::UnclassifiedTable,
            "object_types",
        );
    }
}
#[test]
fn qualified_owner_only_grants_are_rejected_for_both_recipients() {
    for table in [
        "public.group_memberships",
        "\"public\" . \"group_memberships\"",
    ] {
        for recipient in ["console_rt", "PUBLIC"] {
            violation(
                &format!("GRANT SELECT ON TABLE {table} TO {recipient};"),
                ViolationKind::OwnerOnlyTableGrant,
                "group_memberships",
            );
        }
    }
}
#[test]
fn other_schema_or_column_name_is_not_an_owner_only_grant_target() {
    clean("GRANT SELECT ON private.group_memberships TO console_rt;");
    clean("GRANT SELECT (group_memberships) ON public.widgets TO console_rt;");
}

#[test]
fn unqualified_tenant_positive_control_remains_clean() {
    clean(&format!(
        "CREATE TABLE widgets (org_id uuid NOT NULL); {}",
        protection("widgets")
    ));
}

#[test]
fn qualified_create_reuses_existing_unqualified_dynamic_rls_evidence() {
    clean("CREATE TABLE public.widgets (org_id uuid NOT NULL);
        DO $$ DECLARE t text; BEGIN
        FOREACH t IN ARRAY ARRAY['widgets'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
        EXECUTE format('CREATE POLICY org_isolation ON %I USING (org_id = current_setting(''app.current_org'', true)::uuid)', t);
        END LOOP; END $$;");
}

#[test]
fn account_catalog_requires_explicit_classification_outside_parser_repair() {
    violation(
        "CREATE TABLE public.accounts (id uuid);",
        ViolationKind::UnclassifiedTable,
        "accounts",
    );
}

fn dynamic_protection(literal: &str) -> String {
    format!("DO $$ DECLARE t text; BEGIN
        FOREACH t IN ARRAY ARRAY['{literal}'] LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
        EXECUTE format('CREATE POLICY org_isolation ON %I USING (org_id = current_setting(''app.current_org'', true)::uuid)', t);
        END LOOP; END $$;")
}
#[test]
fn dynamic_single_identifier_dot_is_not_schema_qualification() {
    violation(
        &format!(
            "CREATE TABLE public.widgets (org_id uuid NOT NULL); {}",
            dynamic_protection("public.widgets")
        ),
        ViolationKind::OrgColumnWithoutRls,
        "widgets",
    );
}
#[test]
fn dynamic_case_sensitive_identifier_cannot_supply_lowercase_rls() {
    violation(
        &format!(
            "CREATE TABLE widgets (org_id uuid NOT NULL); {}",
            dynamic_protection("Widgets")
        ),
        ViolationKind::OrgColumnWithoutRls,
        "widgets",
    );
}
