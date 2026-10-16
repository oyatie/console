//! Fixed actual Company/Manager source-row roster; no fixture population.
use console_identity_application::company_policy::CurrentCompanyAuthority;
use sqlx::PgPool;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Clone)]
pub(super) struct Probe {
    pub(super) label: &'static str,
    pub(super) sql: &'static str,
    pub(super) ids: Vec<Uuid>,
}

pub(super) async fn probes(
    pool: &PgPool,
    company: Uuid,
    group: Uuid,
    actor: Uuid,
    family: Uuid,
    authority: &CurrentCompanyAuthority,
) -> Vec<Probe> {
    let mut probes = Vec::new();
    for (label, sql, ids) in [
        (
            "Group head",
            "SELECT group_id FROM public.group_authority_heads WHERE group_id=$1 FOR UPDATE NOWAIT",
            vec![group],
        ),
        (
            "Group",
            "SELECT id FROM public.groups WHERE id=$1 FOR UPDATE NOWAIT",
            vec![group],
        ),
        (
            "Account",
            "SELECT account_id FROM public.account_security WHERE account_id=$1 FOR UPDATE NOWAIT",
            vec![actor],
        ),
        (
            "family",
            "SELECT id FROM public.auth_refresh_token_families WHERE user_id=$1 AND id=$2 FOR UPDATE NOWAIT",
            vec![actor, family],
        ),
        (
            "Company",
            "SELECT id FROM public.organizations WHERE id=$1 FOR UPDATE NOWAIT",
            vec![company],
        ),
        (
            "membership",
            "SELECT org_id FROM public.group_memberships WHERE group_id=$1 AND org_id=$2 FOR UPDATE NOWAIT",
            vec![group, company],
        ),
        (
            "membership revision",
            "SELECT h.org_id FROM public.group_membership_revisions h JOIN public.group_memberships m ON (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)=(m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation) WHERE m.group_id=$1 AND m.org_id=$2 FOR UPDATE OF h NOWAIT",
            vec![group, company],
        ),
        (
            "Company head",
            "SELECT org_id FROM public.company_authority_heads WHERE org_id=$1 FOR UPDATE NOWAIT",
            vec![company],
        ),
        (
            "assignment",
            "SELECT id FROM public.user_role_assignments WHERE org_id=$1 AND id=$2 FOR UPDATE NOWAIT",
            vec![company, authority.assignment_id()],
        ),
        (
            "role",
            "SELECT id FROM public.policy_roles WHERE org_id=$1 AND id=$2 FOR UPDATE NOWAIT",
            vec![company, authority.role_id()],
        ),
    ] {
        probes.push(Probe { label, sql, ids });
    }
    // Same actual registry discovery as the accepted Company projection lock test.
    let sources: Vec<(String,Uuid)> = sqlx::query_as("SELECT 'key'::text,k.validator_id FROM public.ont_object_type_key_revisions k JOIN public.ont_object_types o ON o.org_id=k.org_id AND o.stable_key=k.stable_key JOIN public.native_company_object_refs r ON r.org_id=o.org_id AND r.object_type_id=o.id WHERE r.org_id=$1 UNION ALL SELECT 'object',object_type_id FROM public.native_company_object_refs WHERE org_id=$1 UNION ALL SELECT 'property',property_id FROM public.native_company_property_refs WHERE org_id=$1 UNION ALL SELECT 'action',action_type_id FROM public.native_company_action_refs WHERE org_id=$1 ORDER BY 1,2")
        .bind(company).fetch_all(pool).await.unwrap();
    assert_eq!(
        sources.len(),
        19,
        "STOP: actual birth catalog registry roster"
    );
    for (kind, count) in [("key", 2), ("object", 2), ("property", 10), ("action", 5)] {
        assert_eq!(
            sources.iter().filter(|(actual, _)| actual == kind).count(),
            count
        );
    }
    assert_eq!(sources.iter().collect::<BTreeSet<_>>().len(), sources.len());
    for (kind, id) in sources {
        let (label, sql) = match kind.as_str() {
            "key" => (
                "catalog key",
                "SELECT validator_id FROM public.ont_object_type_key_revisions WHERE org_id=$1 AND validator_id=$2 FOR UPDATE NOWAIT",
            ),
            "object" => (
                "catalog object",
                "SELECT id FROM public.ont_object_types WHERE org_id=$1 AND id=$2 FOR UPDATE NOWAIT",
            ),
            "property" => (
                "catalog property",
                "SELECT id FROM public.ont_property_defs WHERE org_id=$1 AND id=$2 FOR UPDATE NOWAIT",
            ),
            "action" => (
                "catalog action",
                "SELECT id FROM public.ont_action_types WHERE org_id=$1 AND id=$2 FOR UPDATE NOWAIT",
            ),
            _ => panic!("STOP: unknown actual registry source kind"),
        };
        probes.push(Probe {
            label,
            sql,
            ids: vec![company, id],
        });
    }
    assert_eq!(probes.len(), 29);
    probes
}
