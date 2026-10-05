#!/usr/bin/env python3
"""Freeze the shared metadata query into the standalone operator statement.

No runtime SQL inputs or database-derived expectations. --check verifies bytes
without writing. Numbered migration checksums match SQLx 0.9's SHA-384 format.
"""
from pathlib import Path
import hashlib
import sys

ROOT = Path(__file__).resolve().parents[1]
TABLES = ('accounts', 'account_security', 'account_security_events',
          'account_terms_acceptances', 'account_terms_head', 'account_terms_release_receipts')


FENCE_BODY = """BEGIN
    IF subject_account_id IS NULL THEN
        RAISE EXCEPTION USING MESSAGE='account_fence_projection.null_identity', ERRCODE='22004';
    END IF;
    RETURN EXISTS(SELECT 1 FROM public.account_security WHERE account_id=subject_account_id);
END;"""

# Installed only by the existing operator transaction. This is a presence
# projection, never credential or lifecycle write authority. Valid replay is
# read-only; any incompatible existing routine is drift, not repair input.
GUARD_BODY = """BEGIN
    RAISE EXCEPTION USING MESSAGE='account_terms_receipts.immutable', ERRCODE='P0001';
END;"""

CURRENT_BODY = 'SELECT h.manifest_sha256,h.revision FROM public.account_terms_head AS h WHERE h.id=1'

ROOT_GUARD_BODY = """BEGIN
    RAISE EXCEPTION USING MESSAGE='account_roots.immutable', ERRCODE='P0001';
END;"""

ROOT_BRIDGE_BODY = """BEGIN
    INSERT INTO public.accounts(id,created_at) VALUES(NEW.id,NEW.created_at);
    RETURN NEW;
END;"""

USER_ID_GUARD_BODY = """BEGIN
    IF NEW.id IS DISTINCT FROM OLD.id THEN
        RAISE EXCEPTION USING MESSAGE='account_legacy_user_id.immutable', ERRCODE='P0001';
    END IF;
    RETURN NEW;
END;"""

DEACTIVATION_GUARD_BODY = """DECLARE
    fenced boolean;
BEGIN
    IF pg_catalog.current_setting('transaction_isolation') <> 'read committed' THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.unsupported_isolation', ERRCODE='P0001';
    END IF;
    IF company_id IS NULL OR subject_id IS NULL THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.null_identity', ERRCODE='22004';
    END IF;
    IF company_id IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.company_context_mismatch', ERRCODE='42501';
    END IF;
    -- Keep Company RLS active and hold both locks in the caller's transaction.
    PERFORM u.id FROM public.users u
        WHERE u.id=subject_id AND u.org_id=company_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.subject_not_found', ERRCODE='P0002';
    END IF;
    PERFORM a.id FROM public.accounts a WHERE a.id=subject_id FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.missing_root', ERRCODE='P0001';
    END IF;
    -- Separate SPI statement: READ COMMITTED takes a fresh snapshot after the
    -- root wait. The strict Auth projection refuses hidden authority as well.
    SELECT public.account_legacy_fenced_v1(subject_id) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING MESSAGE='account_company_deactivation.invalid_fence', ERRCODE='P0001';
    END IF;
    RETURN NOT fenced;
END;"""

DEACTIVATION_INSTALL = """
    -- Only a completely absent guard and users ACL reach this narrow upgrade.
    GRANT SELECT(id,org_id), UPDATE(id) ON public.users TO console_account_owner;
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_company_deactivation_guard_v1(company_id uuid,subject_id uuid)
        RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER
        SET search_path=pg_catalog,pg_temp AS %L', expected_deactivation_guard_body);
    ALTER FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) FROM PUBLIC;
    GRANT EXECUTE ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) TO console_rt;
"""

INSTALL = """
    IF NOT guard_present THEN
        EXECUTE pg_catalog.format('CREATE FUNCTION public.account_terms_receipts_immutable_v1()
            RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER
            SET search_path=pg_catalog,pg_temp AS %L', expected_guard_body);
        ALTER FUNCTION public.account_terms_receipts_immutable_v1() OWNER TO console_terms_owner;
        REVOKE ALL ON FUNCTION public.account_terms_receipts_immutable_v1() FROM PUBLIC, console_terms_owner;
        CREATE TRIGGER account_terms_receipts_immutable_v1
            BEFORE UPDATE OR DELETE OR TRUNCATE ON public.account_terms_release_receipts
            FOR EACH STATEMENT EXECUTE FUNCTION public.account_terms_receipts_immutable_v1();
        ALTER TABLE public.account_terms_release_receipts ENABLE ALWAYS TRIGGER account_terms_receipts_immutable_v1;
        -- FK key-share checks need these ordinary privileges. Install the
        -- unconditional statement guard before granting any UPDATE authority.
        GRANT SELECT(id,revision,manifest_sha256), UPDATE(id)
            ON public.account_terms_release_receipts TO console_terms_owner;
    END IF;
    IF NOT fence_present THEN
        GRANT SELECT ON public.accounts, public.account_security TO console_account_owner;
        GRANT UPDATE(id) ON public.accounts TO console_account_owner;
        EXECUTE pg_catalog.format('CREATE FUNCTION public.account_legacy_fenced_v1(subject_account_id uuid)
            RETURNS boolean LANGUAGE plpgsql STABLE SECURITY DEFINER
            SET search_path=pg_catalog,pg_temp SET row_security=off AS %L', expected_fence_body);
        ALTER FUNCTION public.account_legacy_fenced_v1(uuid) OWNER TO console_account_owner;
        REVOKE ALL ON FUNCTION public.account_legacy_fenced_v1(uuid) FROM PUBLIC;
        GRANT EXECUTE ON FUNCTION public.account_legacy_fenced_v1(uuid) TO console_auth_rt;
    END IF;
    IF NOT current_present THEN
        GRANT SELECT(id,manifest_sha256,revision) ON public.account_terms_head TO console_terms_owner;
        EXECUTE pg_catalog.format('CREATE FUNCTION public.account_terms_current_v1()
            RETURNS TABLE(manifest_sha256 bytea,revision bigint) LANGUAGE sql STABLE SECURITY DEFINER
            SET search_path=pg_catalog,pg_temp AS %L', expected_current_body);
        ALTER FUNCTION public.account_terms_current_v1() OWNER TO console_terms_owner;
        REVOKE ALL ON FUNCTION public.account_terms_current_v1() FROM PUBLIC;
        GRANT EXECUTE ON FUNCTION public.account_terms_current_v1() TO console_auth_rt;
    END IF;
"""

ROOT_INSTALL = """
    -- The old profile was certified under users-first exclusive locks. Reject
    -- all initial UUID overlaps before reaching this plain, one-time copy.
    INSERT INTO public.accounts(id,created_at)
        SELECT id,created_at FROM public.users;
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_roots_immutable_v1()
        RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER
        SET search_path=pg_catalog,pg_temp AS %L', expected_root_guard_body);
    ALTER FUNCTION public.account_roots_immutable_v1() OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_roots_immutable_v1() FROM PUBLIC, console_account_owner;
    CREATE TRIGGER account_roots_immutable_v1
        BEFORE UPDATE OR DELETE OR TRUNCATE ON public.accounts
        FOR EACH STATEMENT EXECUTE FUNCTION public.account_roots_immutable_v1();
    ALTER TABLE public.accounts ENABLE ALWAYS TRIGGER account_roots_immutable_v1;
    -- UPDATE(id) remains only for native FK/key-share semantics. The statement
    -- guard refuses every actual UPDATE, including no-op and zero-row writes.
    GRANT INSERT(id,created_at) ON public.accounts TO console_account_owner;
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_legacy_user_root_v1()
        RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY DEFINER
        SET search_path=pg_catalog,pg_temp AS %L', expected_root_bridge_body);
    ALTER FUNCTION public.account_legacy_user_root_v1() OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_legacy_user_root_v1() FROM PUBLIC, console_account_owner;
    CREATE TRIGGER "00_account_legacy_user_root_v1" AFTER INSERT ON public.users
        FOR EACH ROW EXECUTE FUNCTION public.account_legacy_user_root_v1();
    ALTER TABLE public.users ENABLE ALWAYS TRIGGER "00_account_legacy_user_root_v1";
    EXECUTE pg_catalog.format('CREATE FUNCTION public.account_legacy_user_id_immutable_v1()
        RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER
        SET search_path=pg_catalog,pg_temp AS %L', expected_user_id_guard_body);
    ALTER FUNCTION public.account_legacy_user_id_immutable_v1() OWNER TO console_account_owner;
    REVOKE ALL ON FUNCTION public.account_legacy_user_id_immutable_v1() FROM PUBLIC, console_account_owner;
    -- AFTER UPDATE without OF observes the final NEW.id, including rewrites
    -- made by other BEFORE triggers. Numeric names precede immediate RI checks.
    CREATE TRIGGER "00_account_legacy_user_id_immutable_v1" AFTER UPDATE ON public.users
        FOR EACH ROW EXECUTE FUNCTION public.account_legacy_user_id_immutable_v1();
    ALTER TABLE public.users ENABLE ALWAYS TRIGGER "00_account_legacy_user_id_immutable_v1";
    ALTER TABLE public.users ADD CONSTRAINT users_account_root_v1
        FOREIGN KEY(id) REFERENCES public.accounts(id)
        ON DELETE RESTRICT ON UPDATE RESTRICT DEFERRABLE INITIALLY DEFERRED;
"""


def generated_files():
    query = root_state_query()
    for name, body in (('account_legacy_fenced_v1', FENCE_BODY),
                       ('account_terms_receipts_immutable_v1', GUARD_BODY),
                       ('account_terms_current_v1', CURRENT_BODY),
                       ('account_roots_immutable_v1', ROOT_GUARD_BODY),
                       ('account_legacy_user_root_v1', ROOT_BRIDGE_BODY),
                       ('account_legacy_user_id_immutable_v1', USER_ID_GUARD_BODY),
                       ('account_company_deactivation_guard_v1', DEACTIVATION_GUARD_BODY)):
        expected = f"('{name}','{hashlib.sha256(body.encode()).hexdigest()}')"
        if (LEGACY_ROOT_QUERY.count(expected) != 1
                or NATIVE_SNAPSHOT_QUERY.count(expected) != 1):
            raise SystemExit('reviewed custody routine digest differs: ' + name)
    names = ','.join("'" + name + "'" for name in TABLES)
    locks = ', '.join('ONLY public.' + name for name in TABLES)
    inspect = 'state := (\n' + query + '\n);'
    sql = f"""-- Generated by ops/generate-account-custody.py; review the canonical query.
-- One atomic statement for SQLx and psql. The operator transport must set its
-- statement_timeout before issuing this DO; changing it inside a running
-- statement would not bound that statement. Relation waits are bounded here.
DO $account_custody$
DECLARE
    state text;
    relation_name text;
    target_owner text;
    populated boolean;
    fence_present boolean;
    guard_present boolean;
    current_present boolean;
    root_present boolean;
    expected_fence_body text := $fence_body${FENCE_BODY}$fence_body$;
    expected_guard_body text := $guard_body${GUARD_BODY}$guard_body$;
    expected_current_body text := $current_body${CURRENT_BODY}$current_body$;
    expected_root_guard_body text := $root_guard_body${ROOT_GUARD_BODY}$root_guard_body$;
    expected_root_bridge_body text := $root_bridge_body${ROOT_BRIDGE_BODY}$root_bridge_body$;
    expected_user_id_guard_body text := $user_id_guard_body${USER_ID_GUARD_BODY}$user_id_guard_body$;
    expected_deactivation_guard_body text := $deactivation_guard_body${DEACTIVATION_GUARD_BODY}$deactivation_guard_body$;
BEGIN
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    PERFORM pg_catalog.set_config('lock_timeout','5s',true);
    IF session_user<>current_user
       OR NOT (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user)
       OR session_user IN ('console_app','console_rt','console_auth_rt',
          'console_leave_cmd','console_leave_definer','console_ontology_cmd',
          'console_ontology_writer','console_platform_force_cmd',
          'console_account_owner','console_terms_owner','console_credential_owner') THEN
        RAISE EXCEPTION USING MESSAGE='account_custody.operator_identity_mismatch', ERRCODE='P0001';
    END IF;
    -- Before locking, inspect only existence and kind. Catalog helpers in the
    -- full verdict can wait across another finalizer's ownership/ACL commit.
    IF (SELECT count(*) FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN ({names})) <> {len(TABLES)} THEN
        RAISE EXCEPTION USING MESSAGE='account_custody.catalog_missing', ERRCODE='P0001';
    END IF;
    IF EXISTS (SELECT 1 FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN ({names}) AND c.relkind<>'r') THEN
        RAISE EXCEPTION USING MESSAGE='account_custody.catalog_shape_mismatch', ERRCODE='P0001';
    END IF;
    IF NOT EXISTS(SELECT 1 FROM pg_catalog.pg_class
        WHERE oid=pg_catalog.to_regclass('public.users') AND relkind='r') THEN
        RAISE EXCEPTION USING MESSAGE='account_root_transition.profile_mismatch', ERRCODE='P0001';
    END IF;
    LOCK TABLE ONLY public.organizations IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE ONLY public.users IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE {locks} IN ACCESS EXCLUSIVE MODE;
    -- Freeze legacy writers first, then certify the complete historical or
    -- current profile under all seven locks. Never infer identity from overlap.
    {inspect}
    IF state IS DISTINCT FROM 'account_custody.finalized'
       AND state IS DISTINCT FROM 'account_custody.native_finalized'
       AND state IS DISTINCT FROM 'account_custody.native_upgrade_required'
       AND state IS DISTINCT FROM 'account_custody.pending'
       AND state IS DISTINCT FROM 'account_custody.upgrade_required' THEN
        RAISE EXCEPTION USING MESSAGE=COALESCE(state,'account_custody.catalog_missing'), ERRCODE='P0001';
    END IF;
    -- Auth LOGIN topology is an operator precondition, including replay.
    -- Serving availability belongs to the retained Auth transport's health
    -- check; temporarily stopping that LOGIN does not corrupt custody metadata.
    IF NOT EXISTS (
        SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_auth_rt'
          AND rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit
          AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication
    ) OR EXISTS (
        SELECT 1 FROM pg_catalog.pg_auth_members m
        JOIN pg_catalog.pg_roles r ON r.oid=m.member OR r.oid=m.roleid
        WHERE r.rolname='console_auth_rt'
    ) THEN
        RAISE EXCEPTION 'account_fence_projection.role_mismatch';
    END IF;
    -- The complete verdict above certified every present root/profile field.
    -- Presence now selects the already-certified upgrade path, never authority.
    SELECT pg_catalog.to_regprocedure('public.account_roots_immutable_v1()') IS NOT NULL INTO root_present;
    IF root_present THEN
        -- Serving admission is metadata-only. The privileged operator alone
        -- detects corrupt missing roots and refuses rather than repairing them.
        IF EXISTS(SELECT 1 FROM public.users u LEFT JOIN public.accounts a ON a.id=u.id WHERE a.id IS NULL) THEN
            RAISE EXCEPTION USING MESSAGE='account_root_transition.missing_root', ERRCODE='P0001';
        END IF;
    END IF;
    IF state IN ('account_custody.finalized','account_custody.native_finalized','account_custody.native_upgrade_required') THEN
        RETURN;
    END IF;
    IF NOT root_present THEN
    -- The same complete read-only contract certifies historical input before
    -- any mutation and current output afterward. No second routine validator.
    SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') INTO fence_present;
    SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') INTO guard_present;
    SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_proc p
      JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
      WHERE n.nspname='public' AND p.proname='account_terms_current_v1') INTO current_present;
    IF state='account_custody.pending' THEN
    FOREACH relation_name IN ARRAY ARRAY[{names}] LOOP
        EXECUTE format('SELECT EXISTS(SELECT 1 FROM public.%I)',relation_name) INTO populated;
        IF populated IS DISTINCT FROM false THEN
            RAISE EXCEPTION USING MESSAGE='account_custody.nonempty_staging', ERRCODE='P0001';
        END IF;
    END LOOP;
    END IF;
    IF EXISTS(SELECT 1 FROM public.users u JOIN public.accounts a ON a.id=u.id) THEN
        RAISE EXCEPTION USING MESSAGE='account_root_transition.overlap_requires_admission', ERRCODE='P0001';
    END IF;
    IF state='account_custody.pending' THEN
    FOREACH relation_name IN ARRAY ARRAY[{names}] LOOP
        target_owner := CASE WHEN relation_name IN ('account_terms_head','account_terms_release_receipts')
            THEN 'console_terms_owner' ELSE 'console_account_owner' END;
        EXECUTE format('ALTER TABLE public.%I OWNER TO %I',relation_name,target_owner);
    END LOOP;
    {inspect}
    IF state IS DISTINCT FROM 'account_custody.upgrade_required' THEN
        RAISE EXCEPTION USING MESSAGE=COALESCE(state,'account_custody.catalog_missing'), ERRCODE='P0001';
    END IF;
    END IF;
{INSTALL}
{ROOT_INSTALL}
    END IF;
{DEACTIVATION_INSTALL}
    -- Certify the complete installed state, not merely the routine definition.
    {inspect}
    IF state IS DISTINCT FROM 'account_custody.finalized' THEN
        RAISE EXCEPTION USING MESSAGE=COALESCE(state,'account_custody.catalog_missing'), ERRCODE='P0001';
    END IF;
END
$account_custody$;
"""
    migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
    ledger = ''.join(str(int(path.name.split('_', 1)[0])) + '\t'
                     + hashlib.sha384(path.read_bytes()).hexdigest() + '\n' for path in migrations)
    return {'backend/app/src/account_custody_state.sql': query+';\n',
            'ops/postgres-finalize-account-custody.sql': sql,
            'ops/account-custody-migrations.sha384': ledger,
            'ops/postgres-verify-account-native.sql': native_postcondition_sql(),
            'ops/postgres-company-enrollment-input.sql': company_enrollment_input_sql(),
            'ops/postgres-company-enrollment-schema.sql': company_enrollment_schema_sql(),
            'ops/postgres-company-enrollment-intake.sql': company_enrollment_intake_sql(),
            'ops/postgres-company-enrollment-guards.sql': company_enrollment_guards_sql(),
            'ops/postgres-company-enrollment-owner.sql': company_enrollment_owner_sql(),
            'ops/postgres-capture-company-enrollment-custody.sql': company_enrollment_snapshot_query()+';\n',
            'backend/app/src/company_enrollment_custody_state.sql': company_enrollment_state_query()+';\n',
            'ops/postgres-finalize-company-enrollment.sql': company_enrollment_finalizer_sql(),
            **credential_generated_files(),
            **native_company_policy_generated_files(),
            **native_company_policy_v2_capture_files(),
            **native_company_policy_v2_finalized_files(),
            **native_people_directory_capture_files(),
            **native_people_directory_classifier_files(),
            'ops/postgres-finalize-native-people-directory.sql': native_people_directory_finalizer_sql(),
            **native_people_directory_row_lock_files()}


# Additive Company candidate; historical serializers and fingerprints retain
# their exact meaning. This resource is not a finalized serving profile.
COMPANY_OWNER_SOURCE_SHA256 = 'c6b7047727914d1d29fd77bd50355e7e2f5f918c9b6771658a0103475f5d8a44'
COMPANY_CUSTODY_ADDITIONAL_RELATIONS = (
    'cedar_policy_catalog_entries',
    'company_authority_heads',
    'company_enrollment_effect_bindings',
    'company_enrollment_receipts',
    'company_enrollment_request_events',
    'company_enrollment_requests',
    'group_authority_heads',
    'group_membership_revisions',
    'group_memberships',
    'group_role_grants',
    'groups',
    'native_company_action_refs',
    'native_company_catalog_installs',
    'native_company_object_refs',
    'native_company_property_refs',
    'ont_action_types',
    'ont_analytics',
    'ont_builtin_catalog_allowlist',
    'ont_builtin_catalog_installs',
    'ont_link_types',
    'ont_object_policies',
    'ont_object_type_key_revisions',
    'ont_object_types',
    'ont_property_defs',
    'organizations',
    'platform_force_removal_effect_bindings',
    'platform_force_removal_receipts',
    'platform_legacy_catalog_effect_bindings',
    'platform_legacy_membership_effect_bindings',
    'platform_legacy_topology_effect_bindings',
    'platform_legacy_topology_receipts',
    'platform_legacy_user_birth_witnesses',
    'policy_assignment_revisions',
    'policy_capability_clause_fields',
    'policy_capability_clauses',
    'policy_role_conditions',
    'policy_role_permissions',
    'policy_role_revisions',
    'policy_roles',
    'user_role_assignments',
    'users',
)
COMPANY_CUSTODY_ROUTINE_NAMES = (
    'ont_policy_api.attach_object_policy_rows',
    'ont_policy_api.attach_object_policy_rows_core_v1',
    'ont_policy_api.install_native_company_policy_v1',
    'ontology_api.insert_children',
    'ontology_api.install_builtin_catalog',
    'ontology_api.install_builtin_catalog_core_v1',
    'ontology_api.install_native_company_catalog_v1',
    'ontology_api.lock_native_company_catalog_current_v1',
    'ontology_api.native_catalog_attribution_guard_v1',
    'ontology_api.prepare_legacy_object_type_write',
    'ontology_api.protected_audit_writer_guard',
    'ontology_api.require_current_transaction_audit',
    'public.account_company_context_candidates_v1',
    'public.account_company_native_rows_present_v1',
    'public.account_context_presence_v1',
    'public.account_legacy_topology_roots_lock_v1',
    'public.auth_legacy_bootstrap_issue_v1',
    'public.auth_legacy_bootstrap_issued_v1',
    'public.auth_legacy_bootstrap_receipt_matches_v1',
    'public.company_actor_entitlement_shape_v2',
    'public.company_effect_binding_guard_v1',
    'public.company_enrollment_assert_closure_v1',
    'public.company_enrollment_audit_guard_v1',
    'public.company_enrollment_audit_v1',
    'public.company_enrollment_binding_v1',
    'public.company_enrollment_cancel_v1',
    'public.company_enrollment_catalog_binding_v1',
    'public.company_enrollment_event_guard_v1',
    'public.company_enrollment_execute_v1',
    'public.company_enrollment_intake_closure_v1',
    'public.company_enrollment_ontology_audit_guard_v1',
    'public.company_enrollment_ontology_audit_v1',
    'public.company_enrollment_prepare_v1',
    'public.company_enrollment_receipt_intake_guard_v1',
    'public.company_enrollment_request_guard_v1',
    'public.company_enrollment_status_v1',
    'public.company_enrollment_topology_v1',
    'public.company_native_topology_birth_closure_v1',
    'public.company_topology_history_immutable_v1',
    'public.company_topology_truncate_guard_v1',
    'public.company_topology_write_guard_v1',
    'public.group_authority_lock_exclusive_v1',
    'public.group_authority_lock_shared_v1',
    'public.identity_company_actor_birth_guard_v1',
    'public.identity_company_candidate_birth_guard_v1',
    'public.identity_company_context_generation_guard_v1',
    'public.identity_company_existing_catalog_closure_v1',
    'public.identity_company_projection_v1',
    'public.identity_enroll_company_administration_v1',
    'public.identity_native_any_origin_v1',
    'public.identity_native_birth_closure_v1',
    'public.identity_native_birth_row_guard_v1',
    'public.identity_native_immutable_v1',
    'public.identity_native_legacy_child_guard_v1',
    'public.identity_native_root_guard_v1',
    'public.identity_native_truncate_guard_v1',
    'public.native_company_catalog_birth_closure_v1',
    'public.native_company_catalog_birth_row_guard_v1',
    'public.native_company_catalog_immutable_v1',
    'public.platform_assign_org_to_group',
    'public.platform_attach_membership',
    'public.platform_company_removal_cohort_v1',
    'public.platform_create_organization_core_v1',
    'public.platform_force_effect_admit_v1',
    'public.platform_force_effect_closed_v1',
    'public.platform_force_frame_closed_v1',
    'public.platform_force_frame_guard_v1',
    'public.platform_force_receipt_guard_v1',
    'public.platform_force_remove_command_v1',
    'public.platform_force_remove_decode_input_v1',
    'public.platform_force_remove_direct_org_children',
    'public.platform_force_remove_lock_plan_v1',
    'public.platform_force_remove_plan_v1',
    'public.platform_legacy_catalog_binding_closed_v1',
    'public.platform_legacy_catalog_binding_write_guard_v1',
    'public.platform_legacy_catalog_live_audit_v1',
    'public.platform_legacy_catalog_receipt_audit_v1',
    'public.platform_legacy_command_frame_closed_v1',
    'public.platform_legacy_command_frame_guard_v1',
    'public.platform_legacy_entity_effect_closed_v1',
    'public.platform_legacy_grant_effect_closed_v1',
    'public.platform_legacy_grant_write_guard_v1',
    'public.platform_legacy_head_effect_closed_v1',
    'public.platform_legacy_membership_binding_closed_v1',
    'public.platform_legacy_membership_binding_v1',
    'public.platform_legacy_membership_binding_write_guard_v1',
    'public.platform_legacy_receipt_closed_v1',
    'public.platform_legacy_topology_command_v1',
    'public.platform_legacy_topology_decode_input_v1',
    'public.platform_legacy_topology_lock_plan_v1',
    'public.platform_legacy_topology_plan_v1',
    'public.platform_legacy_topology_receipts_immutable_v1',
    'public.platform_legacy_user_birth_capture_v1',
    'public.platform_legacy_user_birth_witness_closed_v1',
    'public.platform_legacy_user_birth_witness_guard_v1',
    'public.platform_legacy_user_delete_guard_v1',
    'public.platform_legacy_user_update_guard_v1',
    'public.platform_mint_group_row',
)


def company_enrollment_owner_sql():
    source = (ROOT / 'ops/company-enrollment/owner-v1.sql').read_bytes()
    if hashlib.sha256(source).hexdigest() != COMPANY_OWNER_SOURCE_SHA256:
        raise SystemExit('Company owner source requires independent successor review')
    sql = source.decode('utf-8')
    # The operator fixes search_path to pg_catalog,pg_temp. Qualify the two
    # legacy DDL references without touching the retained function body or wire.
    for declaration in ('CREATE OR REPLACE FUNCTION ', 'REVOKE ALL ON FUNCTION '):
        before = declaration + 'platform_force_remove_direct_org_children('
        if sql.count(before) != 1:
            raise ValueError('Company owner force-helper declaration drift')
        sql = sql.replace(before, declaration + 'public.platform_force_remove_direct_org_children(')
    return ('-- Generated by ops/generate-account-custody.py; uninstalled Company candidate.\n'
            '-- Preserves the reviewed owner source; not a finalized custody profile.\n'
            + sql)


def company_enrollment_snapshot_query():
    """Extend custody without changing historical scope or accepting live hashes.

    Include every overload of a named routine so an additional granted overload
    cannot evade the capture. New owner/command roles use the existing complete
    role and membership serializer. Finalized profile admission is separate.
    """
    query = platform_source_snapshot_query()
    relation_anchor = " ('audit_events')\n), relations AS ("
    routine_anchor = " WHERE p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid=to_regclass('public.audit_events') AND NOT tgisinternal) OR"
    owner_anchor = " SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')\n), protected_roles AS ("
    role_anchor = " SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup')\n), related_fks AS ("
    for label, anchor in (('relations', relation_anchor), ('routines', routine_anchor),
                          ('owners', owner_anchor), ('roles', role_anchor)):
        if query.count(anchor) != 1:
            raise ValueError('Company custody snapshot anchor drift: ' + label)
    relation_rows = ',\n'.join(" ('" + name + "')" for name in COMPANY_CUSTODY_ADDITIONAL_RELATIONS)
    query = query.replace(relation_anchor,
        " ('audit_events'),\n" + relation_rows + "\n), relations AS (")
    routine_rows = ','.join("('" + name.replace('.', "','", 1) + "')"
                            for name in COMPANY_CUSTODY_ROUTINE_NAMES)
    query = query.replace(routine_anchor,
        " WHERE (n.nspname,p.proname) IN (VALUES " + routine_rows + ") OR"
        " p.proowner IN (SELECT oid FROM pg_roles WHERE rolname='console_ontology_writer') OR"
        " p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid IN"
        " (SELECT oid FROM relations) AND NOT tgisinternal) OR")
    query = query.replace(owner_anchor,
        " SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner',"
        "'console_credential_owner','console_ontology_writer')\n), protected_roles AS (")
    query = query.replace(role_anchor,
        " SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner',"
        "'console_credential_owner','console_auth_rt','console_auth_startup',"
        "'console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app')\n"
        "), related_fks AS (")
    # Keep the historical serialized flag and all captured metadata bytes intact.
    # The Company scope has 59 relations plus nine legacy-right entries (five
    # overlap), so its validity uses 68 entries, 544 table checks and 63 distinct
    # column-bearing relations. Reuse every existing ACL/role/function rule.
    flag = "   'startup_final_rights_valid',\n"
    boundary_end = "\n ) AS record\n), snapshots AS ("
    output = "SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256 FROM snapshots"
    if any(query.count(anchor) != 1 for anchor in (flag, boundary_end, output)):
        raise ValueError('Company startup-rights predicate anchor drift')
    predicate = query.split(flag, 1)[1].split(boundary_end, 1)[0]
    for before, after in (
        ("count(*)=18 AND count(oid)=18", "count(*)=59 AND count(oid)=59"),
        ("count(*)=27 AND count(oid)=27", "count(*)=68 AND count(oid)=68"),
        ("count(*)=216 AND bool_and", "count(*)=544 AND bool_and"),
        ("count(DISTINCT name)=27", "count(DISTINCT name)=63"),
    ):
        if predicate.count(before) != 1:
            raise ValueError('Company startup-rights cardinality anchor drift')
        predicate = predicate.replace(before, after)
    query = query.replace(boundary_end,
        "\n ) AS record\n), company_startup_rights AS (\n SELECT\n"
        + predicate + " AS valid\n), snapshots AS (")
    query = query.replace(output,
        "SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256,"
        "(SELECT valid FROM company_startup_rights) AS company_startup_rights_valid FROM snapshots")
    return query


def company_enrollment_guards_sql():
    sql = r"""-- Generated by ops/generate-account-custody.py. UNINSTALLED candidate.
-- Empty private intake only. Complete effects and populated rollout are separate.
SET LOCAL lock_timeout='1s';
DO $install$
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
END
$install$;
LOCK TABLE public.company_enrollment_requests IN ACCESS EXCLUSIVE MODE;
LOCK TABLE public.company_enrollment_request_events IN ACCESS EXCLUSIVE MODE;
LOCK TABLE public.company_enrollment_receipts IN ACCESS EXCLUSIVE MODE;
DO $install$
BEGIN
    IF EXISTS(SELECT 1 FROM public.company_enrollment_requests)
        OR EXISTS(SELECT 1 FROM public.company_enrollment_request_events)
        OR EXISTS(SELECT 1 FROM public.company_enrollment_receipts) THEN
        RAISE EXCEPTION 'company_enrollment.guard_install_requires_empty';
    END IF;
END
$install$;

CREATE FUNCTION public.company_enrollment_request_guard_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE candidate record;
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_requests'
        OR TG_LEVEL<>'ROW' OR TG_WHEN<>'BEFORE' OR TG_OP NOT IN ('INSERT','UPDATE') THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    IF current_user<>'console_account_owner' THEN
        RAISE EXCEPTION 'company_enrollment.writer_required';
    END IF;
    IF TG_OP='INSERT' THEN
        IF (NEW.state='PENDING' AND NEW.input_bytes IS NOT NULL
            AND NEW.terminal_at IS NULL AND NEW.committed_receipt_id IS NULL
            AND isfinite(NEW.created_at) AND NEW.created_at>=statement_timestamp()
            AND NEW.created_at<=clock_timestamp()
            AND isfinite(NEW.expires_at) AND NEW.expires_at>NEW.created_at
            AND NEW.expires_at<=NEW.created_at+interval '168 hours') IS NOT TRUE THEN
            RAISE EXCEPTION 'company_enrollment.request_transition_invalid';
        END IF;
        SELECT * INTO STRICT candidate FROM public.company_enrollment_decode_input_v1(NEW.input_bytes);
        IF NEW.account_id IS DISTINCT FROM candidate.account_id
            OR NEW.command_id IS DISTINCT FROM candidate.command_id OR NEW.codec_version IS DISTINCT FROM 1
            OR NEW.input_digest IS DISTINCT FROM candidate.input_digest
            OR NOT EXISTS(SELECT 1 FROM public.deployment_operator_receipts r
                WHERE r.receipt_id=NEW.designation_receipt_id AND r.account_id=NEW.account_id
                    AND r.kind='DESIGNATE') THEN
            RAISE EXCEPTION 'company_enrollment.request_transition_invalid';
        END IF;
    ELSE
        IF ROW(NEW.account_id,NEW.command_id,NEW.codec_version,NEW.input_digest,
            NEW.designation_receipt_id,NEW.created_at,NEW.expires_at)
            IS DISTINCT FROM ROW(OLD.account_id,OLD.command_id,OLD.codec_version,OLD.input_digest,
                OLD.designation_receipt_id,OLD.created_at,OLD.expires_at) THEN
            RAISE EXCEPTION 'company_enrollment.request_immutable';
        END IF;
        IF NEW.state='COMMITTED' THEN RAISE EXCEPTION 'company_enrollment.effect_unavailable'; END IF;
        IF (OLD.state='PENDING' AND NEW.committed_receipt_id IS NULL
            AND isfinite(NEW.terminal_at) AND NEW.terminal_at>=OLD.created_at
            AND NEW.terminal_at>=statement_timestamp() AND NEW.terminal_at<=clock_timestamp()
            AND ((NEW.state='CANCELLED' AND NEW.terminal_at<OLD.expires_at
                    AND NEW.input_bytes IS NOT DISTINCT FROM OLD.input_bytes)
                OR (NEW.state='EXPIRED' AND NEW.terminal_at>=OLD.expires_at AND NEW.input_bytes IS NULL))) IS NOT TRUE THEN
            RAISE EXCEPTION 'company_enrollment.request_transition_invalid';
        END IF;
    END IF;
    RETURN NEW;
END
$body$;

CREATE FUNCTION public.company_enrollment_request_preserve_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_requests'
        OR TG_LEVEL<>'STATEMENT' OR TG_WHEN<>'BEFORE' OR TG_OP NOT IN ('DELETE','TRUNCATE') THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    RAISE EXCEPTION 'company_enrollment.request_immutable';
END
$body$;

CREATE FUNCTION public.company_enrollment_event_immutable_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_request_events'
        OR TG_LEVEL<>'STATEMENT' OR TG_WHEN<>'BEFORE' OR TG_OP NOT IN ('UPDATE','DELETE','TRUNCATE') THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    RAISE EXCEPTION 'company_enrollment.event_immutable';
END
$body$;

CREATE FUNCTION public.company_enrollment_event_guard_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    request public.company_enrollment_requests%ROWTYPE;
    first_event public.company_enrollment_request_events%ROWTYPE;
    control public.account_security%ROWTYPE;
    family record;
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_request_events'
        OR TG_LEVEL<>'ROW' OR TG_WHEN<>'AFTER' OR TG_OP<>'INSERT' THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    IF current_user<>'console_account_owner' THEN RAISE EXCEPTION 'company_enrollment.writer_required'; END IF;
    SELECT r.* INTO request FROM public.company_enrollment_requests r
        WHERE r.account_id=NEW.account_id AND r.command_id=NEW.command_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'company_enrollment.event_invalid'; END IF;
    IF NEW.event_revision=1 THEN
        IF (NEW.from_state IS NULL AND NEW.to_state='PENDING' AND NEW.reason_code='PREPARED'
            AND request.state='PENDING' AND NEW.occurred_at=request.created_at) IS NOT TRUE THEN
            RAISE EXCEPTION 'company_enrollment.event_invalid';
        END IF;
    ELSIF NEW.event_revision=2 THEN
        SELECT e.* INTO first_event FROM public.company_enrollment_request_events e
            WHERE e.account_id=NEW.account_id AND e.command_id=NEW.command_id AND e.event_revision=1;
        IF (first_event.from_state IS NULL AND first_event.to_state='PENDING'
            AND first_event.reason_code='PREPARED' AND first_event.occurred_at=request.created_at
            AND first_event.actor_account_id=request.account_id AND first_event.session_id IS NOT NULL
            AND first_event.session_id<>'00000000-0000-0000-0000-000000000000'::uuid
            AND NEW.from_state='PENDING' AND NEW.to_state IN ('CANCELLED','EXPIRED')
            AND NEW.to_state=request.state AND NEW.reason_code=request.state
            AND NEW.occurred_at=request.terminal_at) IS NOT TRUE THEN
            RAISE EXCEPTION 'company_enrollment.event_invalid';
        END IF;
    ELSE RAISE EXCEPTION 'company_enrollment.event_invalid';
    END IF;
    IF NEW.to_state='EXPIRED' THEN
        IF NEW.actor_account_id IS NOT NULL OR NEW.session_id IS NOT NULL THEN
            RAISE EXCEPTION 'company_enrollment.event_invalid';
        END IF;
    ELSE
        IF NEW.actor_account_id IS DISTINCT FROM request.account_id OR NEW.session_id IS NULL
            OR NEW.session_id='00000000-0000-0000-0000-000000000000'::uuid THEN
            RAISE EXCEPTION 'company_enrollment.event_invalid';
        END IF;
        -- The canonical owner already holds Account then family guards before its fence.
        -- Do not acquire a new Account guard here or revalidate historical sessions at commit.
        SELECT s.* INTO control FROM public.account_security s WHERE s.account_id=request.account_id;
        BEGIN
            SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(request.account_id,NEW.session_id);
        EXCEPTION WHEN no_data_found THEN RAISE EXCEPTION 'company_enrollment.event_invalid';
            WHEN raise_exception THEN
                IF SQLERRM='account.authentication_invalid' THEN RAISE EXCEPTION 'company_enrollment.event_invalid'; END IF;
                RAISE;
        END;
        IF (control.security_state='ACTIVE' AND control.security_generation>0
            AND family.user_id=request.account_id AND family.protocol='ACCOUNT_V1'
            AND family.account_security_generation=control.security_generation
            AND family.org_id IS NULL AND family.revoked_at IS NULL
            AND family.assurance='PASSKEY_PRIMARY' AND isfinite(family.auth_time)
            AND isfinite(family.created_at) AND family.auth_time<=family.created_at
            AND family.created_at<=clock_timestamp()) IS NOT TRUE THEN
            RAISE EXCEPTION 'company_enrollment.event_invalid';
        END IF;
    END IF;
    RETURN NEW;
END
$body$;

CREATE FUNCTION public.company_enrollment_receipt_intake_guard_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_receipts'
        OR TG_LEVEL<>'STATEMENT' OR TG_WHEN<>'BEFORE' OR TG_OP<>'INSERT' THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    RAISE EXCEPTION 'company_enrollment.effect_unavailable';
END
$body$;

CREATE FUNCTION public.company_enrollment_intake_closure_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    request public.company_enrollment_requests%ROWTYPE;
    first_event public.company_enrollment_request_events%ROWTYPE;
    last_event public.company_enrollment_request_events%ROWTYPE;
    event_count bigint;
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_LEVEL<>'ROW' OR TG_WHEN<>'AFTER'
        OR NOT ((TG_TABLE_NAME='company_enrollment_requests' AND TG_OP IN ('INSERT','UPDATE'))
            OR (TG_TABLE_NAME IN ('company_enrollment_request_events','company_enrollment_receipts') AND TG_OP='INSERT')) THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    SELECT r.* INTO request FROM public.company_enrollment_requests r
        WHERE r.account_id=NEW.account_id AND r.command_id=NEW.command_id;
    IF NOT FOUND THEN RAISE EXCEPTION 'company_enrollment.intake_closure_invalid'; END IF;
    IF request.state='COMMITTED' THEN RAISE EXCEPTION 'company_enrollment.effect_unavailable'; END IF;
    IF EXISTS(SELECT 1 FROM public.company_enrollment_receipts r
        WHERE r.account_id=NEW.account_id AND r.command_id=NEW.command_id) THEN
        RAISE EXCEPTION 'company_enrollment.intake_closure_invalid';
    END IF;
    SELECT count(*) INTO event_count FROM public.company_enrollment_request_events e
        WHERE e.account_id=NEW.account_id AND e.command_id=NEW.command_id;
    SELECT e.* INTO first_event FROM public.company_enrollment_request_events e
        WHERE e.account_id=NEW.account_id AND e.command_id=NEW.command_id AND e.event_revision=1;
    IF (first_event.from_state IS NULL AND first_event.to_state='PENDING'
        AND first_event.reason_code='PREPARED' AND first_event.occurred_at=request.created_at
        AND first_event.actor_account_id=request.account_id AND first_event.session_id IS NOT NULL
        AND first_event.session_id<>'00000000-0000-0000-0000-000000000000'::uuid
        AND request.committed_receipt_id IS NULL) IS NOT TRUE THEN
        RAISE EXCEPTION 'company_enrollment.intake_closure_invalid';
    END IF;
    IF request.state='PENDING' THEN
        IF event_count<>1 OR request.input_bytes IS NULL OR request.terminal_at IS NOT NULL THEN
            RAISE EXCEPTION 'company_enrollment.intake_closure_invalid';
        END IF;
    ELSE
        SELECT e.* INTO last_event FROM public.company_enrollment_request_events e
            WHERE e.account_id=NEW.account_id AND e.command_id=NEW.command_id AND e.event_revision=2;
        IF (event_count=2 AND last_event.from_state='PENDING' AND last_event.to_state=request.state
            AND last_event.reason_code=request.state AND last_event.occurred_at=request.terminal_at
            AND ((request.state='CANCELLED' AND request.input_bytes IS NOT NULL AND request.terminal_at<request.expires_at
                    AND last_event.actor_account_id=request.account_id AND last_event.session_id IS NOT NULL
                    AND last_event.session_id<>'00000000-0000-0000-0000-000000000000'::uuid)
                OR (request.state='EXPIRED' AND request.input_bytes IS NULL AND request.terminal_at>=request.expires_at
                    AND last_event.actor_account_id IS NULL AND last_event.session_id IS NULL))) IS NOT TRUE THEN
            RAISE EXCEPTION 'company_enrollment.intake_closure_invalid';
        END IF;
    END IF;
    RETURN NEW;
END
$body$;
"""
    for name in ('request_guard', 'request_preserve', 'event_immutable', 'event_guard',
                 'receipt_intake_guard', 'intake_closure'):
        signature = f'public.company_enrollment_{name}_v1()'
        sql += f'''ALTER FUNCTION {signature} OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION {signature} FROM PUBLIC,console_account_owner;
DO $guard_acl$
DECLARE grantee_name text;
BEGIN
    FOR grantee_name IN
        SELECT DISTINCT role.rolname FROM pg_catalog.pg_proc routine
        CROSS JOIN LATERAL pg_catalog.aclexplode(routine.proacl) acl
        JOIN pg_catalog.pg_roles role ON role.oid=acl.grantee
        WHERE routine.oid='{signature}'::regprocedure
    LOOP
        EXECUTE pg_catalog.format('REVOKE ALL ON FUNCTION {signature} FROM %I',grantee_name);
    END LOOP;
END
$guard_acl$;
'''
    for table, name, operations, level, function in (
        ('requests', 'request_guard', 'INSERT OR UPDATE', 'ROW', 'request_guard'),
        ('requests', 'request_preserve', 'DELETE OR TRUNCATE', 'STATEMENT', 'request_preserve'),
        ('request_events', 'event_immutable', 'UPDATE OR DELETE OR TRUNCATE', 'STATEMENT', 'event_immutable'),
        ('request_events', 'event_guard', 'INSERT', 'ROW', 'event_guard'),
        ('receipts', 'receipt_intake_guard', 'INSERT', 'STATEMENT', 'receipt_intake_guard'),
        ('requests', 'request_closure', 'INSERT OR UPDATE', 'ROW', 'intake_closure'),
        ('request_events', 'event_closure', 'INSERT', 'ROW', 'intake_closure'),
        ('receipts', 'receipt_closure', 'INSERT', 'ROW', 'intake_closure'),
    ):
        deferred = function == 'intake_closure'
        phase = 'AFTER' if deferred or function == 'event_guard' else 'BEFORE'
        sql += f"CREATE {'CONSTRAINT ' if deferred else ''}TRIGGER company_enrollment_{name}_v1\n"
        sql += f'    {phase} {operations} ON public.company_enrollment_{table}\n'
        if deferred:
            sql += '    DEFERRABLE INITIALLY DEFERRED\n'
        sql += f'    FOR EACH {level} EXECUTE FUNCTION public.company_enrollment_{function}_v1();\n'
        sql += f'ALTER TABLE public.company_enrollment_{table} ENABLE ALWAYS TRIGGER company_enrollment_{name}_v1;\n'
    return sql


def company_enrollment_intake_sql():
    # These four entrypoints share generated recovery text, not an extra SQL API.
    material = r"""CREATE FUNCTION public.company_enrollment_session_material_v1(
    p_account uuid,p_family uuid,p_command uuid,p_input bytea)
RETURNS TABLE(security_state text,security_generation bigint,revision bigint,
    context_generation bigint,user_id uuid,protocol text,account_security_generation bigint,
    auth_time timestamptz,assurance text,created_at timestamptz,revoked_at timestamptz,
    org_id uuid,planned_recipient uuid,planned_input_digest bytea)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='1s'
AS $body$
DECLARE
    candidate record;
    original public.company_enrollment_requests%ROWTYPE;
    control record;
    family record;
    recipient uuid;
    guarded_recipient uuid;
    guard_id uuid;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    IF p_account IS NULL OR p_family IS NULL OR p_command IS NULL
        OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR p_family='00000000-0000-0000-0000-000000000000'::uuid
        OR p_command='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    IF p_input IS NOT NULL THEN
        SELECT * INTO STRICT candidate FROM public.company_enrollment_decode_input_v1(p_input);
        IF candidate.account_id IS DISTINCT FROM p_account OR candidate.command_id IS DISTINCT FROM p_command THEN
            RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='company_enrollment.invalid_input';
        END IF;
    END IF;
    SELECT r.* INTO original FROM public.company_enrollment_requests r
        WHERE r.account_id=p_account AND r.command_id=p_command;
    IF p_input IS NOT NULL THEN
        IF NOT FOUND THEN
            recipient := candidate.administrative_account_id;
        ELSIF original.input_bytes IS NOT NULL THEN
            SELECT d.administrative_account_id INTO recipient
                FROM public.company_enrollment_decode_input_v1(original.input_bytes) d;
        END IF;
    END IF;
    guarded_recipient := recipient;
    -- UUID ordering is the same byte ordering used by all enrollment participants.
    FOR guard_id IN SELECT DISTINCT id FROM unnest(ARRAY[p_account,recipient]) id
        WHERE id IS NOT NULL ORDER BY id
    LOOP
        PERFORM 1 FROM public.account_security_lock_exclusive_v1(guard_id);
    END LOOP;
    SELECT * INTO STRICT control FROM public.account_security_lock_exclusive_v1(p_account);
    SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
    IF control.security_state IS DISTINCT FROM 'ACTIVE'
        OR family.user_id IS DISTINCT FROM p_account OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
        OR family.account_security_generation IS DISTINCT FROM control.security_generation
        OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
        OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' OR family.auth_time IS NULL
        OR NOT isfinite(family.auth_time) OR NOT isfinite(family.created_at)
        OR family.auth_time>family.created_at OR family.created_at>clock_timestamp() THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    IF control.security_generation IS NULL OR control.security_generation<1
        OR control.revision IS NULL OR control.revision<1
        OR control.context_generation IS NULL OR control.context_generation<1 THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    -- Fresh SPI snapshot after every wait; never add an earlier-class lock late.
    SELECT r.* INTO original FROM public.company_enrollment_requests r
        WHERE r.account_id=p_account AND r.command_id=p_command;
    recipient := NULL;
    IF p_input IS NOT NULL THEN
        IF NOT FOUND THEN
            recipient := candidate.administrative_account_id;
        ELSIF original.input_bytes IS NOT NULL THEN
            SELECT d.administrative_account_id INTO recipient
                FROM public.company_enrollment_decode_input_v1(original.input_bytes) d;
        END IF;
    END IF;
    IF recipient IS DISTINCT FROM guarded_recipient THEN
        RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
    END IF;
    RETURN QUERY SELECT control.security_state,control.security_generation,control.revision,
        control.context_generation,family.user_id,family.protocol,family.account_security_generation,
        family.auth_time,family.assurance,family.created_at,family.revoked_at,family.org_id,
        recipient,original.input_digest;
END
$body$;
"""
    fence = r"""    PERFORM pg_advisory_xact_lock(hashtextextended(
        'console.company.enrollment.command/1:'||p_account::text||':'||p_command::text,0));
    SELECT r.* INTO request FROM public.company_enrollment_requests r
        WHERE r.account_id=p_account AND r.command_id=p_command;
"""
    expiry = r"""    observed_at := clock_timestamp();
    IF request.state='PENDING' AND observed_at>=request.expires_at THEN
        UPDATE public.company_enrollment_requests r SET state='EXPIRED',input_bytes=NULL,terminal_at=observed_at
            WHERE r.account_id=p_account AND r.command_id=p_command;
        INSERT INTO public.company_enrollment_request_events
            (account_id,command_id,event_revision,from_state,to_state,occurred_at,actor_account_id,session_id,reason_code)
            VALUES(p_account,p_command,2,'PENDING','EXPIRED',observed_at,NULL,NULL,'EXPIRED');
        request.state := 'EXPIRED';
        request.input_bytes := NULL;
        request.terminal_at := observed_at;
    END IF;
    -- Complete effect/receipt reconciliation is deliberately not installed yet.
    IF request.state='COMMITTED' THEN RAISE EXCEPTION 'account.authority_unavailable'; END IF;
"""
    prepare = r"""CREATE FUNCTION public.company_enrollment_prepare_v1(
    p_account uuid,p_family uuid,p_command uuid,p_input bytea)
RETURNS TABLE(state text,receipt_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='1s'
AS $body$
DECLARE
    candidate record;
    material record;
    recipient_control record;
    request public.company_enrollment_requests%ROWTYPE;
    origin uuid;
    observed_at timestamptz;
BEGIN
    SELECT * INTO STRICT candidate FROM public.company_enrollment_decode_input_v1(p_input);
    SELECT * INTO STRICT material FROM public.company_enrollment_session_material_v1(p_account,p_family,p_command,p_input);
    IF material.planned_input_digest IS NULL THEN
        IF candidate.group_id IS NOT NULL THEN RAISE EXCEPTION 'company_enrollment.group_unavailable'; END IF;
        IF NOT public.account_company_setup_eligibility_v1(p_account) THEN
            RAISE EXCEPTION 'company_enrollment.forbidden';
        END IF;
        -- The Account guards are already held, before the designation head guard.
        SELECT s.* INTO STRICT recipient_control FROM public.account_security s
            WHERE s.account_id=candidate.administrative_account_id;
        IF recipient_control.security_state IS DISTINCT FROM 'ACTIVE' THEN
            RAISE EXCEPTION 'company_enrollment.forbidden';
        END IF;
        PERFORM 1 FROM public.account_login_consent_v1(p_account);
        PERFORM 1 FROM public.account_login_consent_v1(candidate.administrative_account_id);
        SELECT h.receipt_id INTO STRICT origin FROM public.deployment_operator_head h
            WHERE h.singleton=1 AND h.account_id=p_account;
        -- ponytail: serializes new deployment intake only; replace after measured admission contention.
        PERFORM pg_advisory_xact_lock(1128615506,1);
    END IF;
""" + fence + r"""    IF request.account_id IS NOT NULL THEN
        IF request.input_digest IS DISTINCT FROM candidate.input_digest THEN
            RAISE EXCEPTION 'company_enrollment.conflict';
        END IF;
""" + expiry + r"""        RETURN QUERY SELECT request.state,NULL::uuid;
        RETURN;
    END IF;
    IF material.planned_input_digest IS NOT NULL OR origin IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
    END IF;
    IF (SELECT count(*) FROM public.company_enrollment_requests r WHERE r.account_id=p_account AND r.state='PENDING')>=16
        OR (SELECT count(*) FROM public.company_enrollment_requests r WHERE r.state='PENDING')>=256 THEN
        RAISE EXCEPTION 'company_enrollment.capacity';
    END IF;
    observed_at := clock_timestamp();
    INSERT INTO public.company_enrollment_requests
        (account_id,command_id,codec_version,input_bytes,input_digest,designation_receipt_id,created_at,expires_at,state)
        VALUES(p_account,p_command,1,p_input,candidate.input_digest,origin,observed_at,observed_at+interval '168 hours','PENDING');
    INSERT INTO public.company_enrollment_request_events
        (account_id,command_id,event_revision,from_state,to_state,occurred_at,actor_account_id,session_id,reason_code)
        VALUES(p_account,p_command,1,NULL,'PENDING',observed_at,p_account,p_family,'PREPARED');
    RETURN QUERY SELECT 'PENDING'::text,NULL::uuid;
END
$body$;
"""
    recovery = []
    for name in ('status', 'cancel'):
        result = 'state text,' + ('codec_version smallint,input_bytes bytea,' if name == 'status' else '')
        result += 'receipt_id uuid,org_id uuid,group_id uuid,administrative_account_id uuid'
        body = f"""CREATE FUNCTION public.company_enrollment_{name}_v1(p_account uuid,p_family uuid,p_command uuid)
RETURNS TABLE({result})
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='1s'
AS $body$
DECLARE
    request public.company_enrollment_requests%ROWTYPE;
    observed_at timestamptz;
BEGIN
    PERFORM 1 FROM public.company_enrollment_session_material_v1(p_account,p_family,p_command,NULL);
""" + fence + "    IF NOT FOUND THEN RETURN; END IF;\n" + expiry
        if name == 'cancel':
            body += r"""    IF request.state='PENDING' THEN
        UPDATE public.company_enrollment_requests r SET state='CANCELLED',terminal_at=observed_at
            WHERE r.account_id=p_account AND r.command_id=p_command;
        INSERT INTO public.company_enrollment_request_events
            (account_id,command_id,event_revision,from_state,to_state,occurred_at,actor_account_id,session_id,reason_code)
            VALUES(p_account,p_command,2,'PENDING','CANCELLED',observed_at,p_account,p_family,'CANCELLED');
        request.state := 'CANCELLED';
    END IF;
"""
        body += '    RETURN QUERY SELECT request.state,'
        if name == 'status':
            body += "request.codec_version,CASE WHEN request.state='PENDING' THEN request.input_bytes ELSE NULL::bytea END,"
        body += 'NULL::uuid,NULL::uuid,NULL::uuid,NULL::uuid;\nEND\n$body$;\n'
        recovery.append(body)
    sql = '-- Generated by ops/generate-account-custody.py. UNINSTALLED candidate.\n'
    sql += '-- Private pending owner only; serving activation and signed credential validation remain separate.\n'
    sql += material + prepare + ''.join(recovery)
    for name, args in [('session_material', 'uuid,uuid,uuid,bytea'), ('prepare', 'uuid,uuid,uuid,bytea'),
                       ('status', 'uuid,uuid,uuid'), ('cancel', 'uuid,uuid,uuid')]:
        signature = f'public.company_enrollment_{name}_v1({args})'
        sql += f'''ALTER FUNCTION {signature} OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION {signature} FROM PUBLIC,console_app,console_rt,console_auth_rt,
    console_auth_startup,console_account_owner,console_terms_owner,console_credential_owner,
    console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
DO $intake_acl$
DECLARE grantee_name text;
BEGIN
    FOR grantee_name IN
        SELECT DISTINCT role.rolname FROM pg_catalog.pg_proc routine
        CROSS JOIN LATERAL pg_catalog.aclexplode(routine.proacl) acl
        JOIN pg_catalog.pg_roles role ON role.oid=acl.grantee
        WHERE routine.oid='{signature}'::regprocedure
    LOOP
        EXECUTE pg_catalog.format('REVOKE ALL ON FUNCTION {signature} FROM %I',grantee_name);
    END LOOP;
END
$intake_acl$;
GRANT EXECUTE ON FUNCTION {signature} TO console_account_owner,console_rt;
'''
    return sql


def company_enrollment_schema_sql():
    return r"""-- Generated by ops/generate-account-custody.py. UNINSTALLED candidate.
-- Empty private schema prerequisite only; no serving finalizer may install it.
-- Request/event transitions and complete effect closure remain separate requirements.
CREATE TABLE public.company_enrollment_requests (
    account_id uuid NOT NULL,
    command_id uuid NOT NULL,
    codec_version smallint NOT NULL CHECK (codec_version=1),
    input_bytes bytea,
    input_digest bytea NOT NULL CHECK (octet_length(input_digest)=32),
    designation_receipt_id uuid NOT NULL,
    created_at timestamptz NOT NULL CHECK (isfinite(created_at)),
    expires_at timestamptz NOT NULL CHECK (isfinite(expires_at)),
    state text NOT NULL CHECK (state IN ('PENDING','COMMITTED','CANCELLED','EXPIRED')),
    terminal_at timestamptz CHECK (isfinite(terminal_at)),
    committed_receipt_id uuid,
    PRIMARY KEY(account_id,command_id),
    UNIQUE(account_id,command_id,codec_version,input_digest,designation_receipt_id),
    CHECK (account_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND command_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND designation_receipt_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    CHECK (expires_at>created_at AND expires_at<=created_at+interval '168 hours'),
    CHECK ((state='PENDING')=(terminal_at IS NULL)),
    CHECK ((state='COMMITTED')=(committed_receipt_id IS NOT NULL)),
    CHECK (terminal_at IS NULL OR terminal_at>=created_at),
    CHECK (state<>'PENDING' OR input_bytes IS NOT NULL),
    CHECK (input_bytes IS NULL OR pg_catalog.sha256(input_bytes)=input_digest),
    CONSTRAINT company_enrollment_request_account_fk FOREIGN KEY(account_id)
        REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    CONSTRAINT company_enrollment_request_designation_fk FOREIGN KEY(designation_receipt_id)
        REFERENCES public.deployment_operator_receipts(receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT
);
CREATE INDEX company_enrollment_pending_account_idx
    ON public.company_enrollment_requests(account_id,created_at,command_id) WHERE state='PENDING';
CREATE INDEX company_enrollment_pending_expiry_idx
    ON public.company_enrollment_requests(expires_at,account_id,command_id) WHERE state='PENDING';
CREATE INDEX company_enrollment_terminal_payload_idx
    ON public.company_enrollment_requests(terminal_at,account_id,command_id)
    WHERE state<>'PENDING' AND input_bytes IS NOT NULL;

CREATE TABLE public.company_enrollment_receipts (
    receipt_id uuid PRIMARY KEY,
    account_id uuid NOT NULL REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    command_id uuid NOT NULL,
    codec_version smallint NOT NULL CHECK(codec_version=1),
    input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32),
    designation_receipt_id uuid NOT NULL REFERENCES public.deployment_operator_receipts(receipt_id)
        ON UPDATE RESTRICT ON DELETE RESTRICT,
    org_id uuid NOT NULL REFERENCES public.organizations(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    group_id uuid NOT NULL REFERENCES public.groups(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    administrative_account_id uuid NOT NULL REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    root_assignment_id uuid NOT NULL REFERENCES public.user_role_assignments(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    root_revision bigint NOT NULL CHECK(root_revision=1),
    catalog_version text NOT NULL CHECK(catalog_version='native-company-identity-2026-09-19.1'),
    manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
    action_refs jsonb NOT NULL,
    property_refs jsonb NOT NULL,
    session_id uuid NOT NULL,
    committed_at timestamptz NOT NULL CHECK(isfinite(committed_at)),
    UNIQUE(account_id,command_id),
    UNIQUE(account_id,command_id,receipt_id),
    UNIQUE(account_id,command_id,receipt_id,codec_version,input_digest,designation_receipt_id),
    CHECK (receipt_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND account_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND command_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND org_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND group_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND administrative_account_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND root_assignment_id<>'00000000-0000-0000-0000-000000000000'::uuid
       AND session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
    -- Shape is necessary, NOT sufficient for installed physical-ref authority.
    CHECK (CASE WHEN jsonb_typeof(action_refs)='array' THEN jsonb_array_length(action_refs)=5 ELSE false END),
    CHECK (CASE WHEN jsonb_typeof(property_refs)='array' THEN jsonb_array_length(property_refs)=10 ELSE false END),
    CONSTRAINT company_enrollment_receipt_request_fk
      FOREIGN KEY(account_id,command_id,codec_version,input_digest,designation_receipt_id)
      REFERENCES public.company_enrollment_requests(account_id,command_id,codec_version,input_digest,designation_receipt_id)
      ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);
ALTER TABLE public.company_enrollment_requests ADD CONSTRAINT company_enrollment_request_receipt_fk
    FOREIGN KEY(account_id,command_id,committed_receipt_id,codec_version,input_digest,designation_receipt_id)
    REFERENCES public.company_enrollment_receipts(account_id,command_id,receipt_id,codec_version,input_digest,designation_receipt_id)
    ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE public.company_enrollment_request_events (
    account_id uuid NOT NULL,
    command_id uuid NOT NULL,
    event_revision bigint NOT NULL CHECK(event_revision BETWEEN 1 AND 2),
    from_state text,
    to_state text NOT NULL CHECK(to_state IN ('PENDING','COMMITTED','CANCELLED','EXPIRED')),
    occurred_at timestamptz NOT NULL CHECK(isfinite(occurred_at)),
    actor_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    session_id uuid,
    reason_code text NOT NULL CHECK(reason_code IN ('PREPARED','COMMITTED','CANCELLED','EXPIRED')),
    PRIMARY KEY(account_id,command_id,event_revision),
    FOREIGN KEY(account_id,command_id) REFERENCES public.company_enrollment_requests(account_id,command_id)
      ON UPDATE RESTRICT ON DELETE RESTRICT,
    CHECK (COALESCE((event_revision=1 AND from_state IS NULL AND to_state='PENDING' AND reason_code='PREPARED')
        OR (event_revision=2 AND from_state='PENDING' AND to_state IN ('COMMITTED','CANCELLED','EXPIRED') AND reason_code=to_state),false)),
    CHECK ((to_state='EXPIRED' AND actor_account_id IS NULL AND session_id IS NULL)
        OR (to_state<>'EXPIRED' AND actor_account_id=account_id AND actor_account_id IS NOT NULL
            AND session_id IS NOT NULL AND session_id<>'00000000-0000-0000-0000-000000000000'::uuid))
);

-- Private schema only; the current serving profile deliberately rejects this catalog.
ALTER TABLE public.company_enrollment_requests OWNER TO console_account_owner;
ALTER TABLE public.company_enrollment_receipts OWNER TO console_account_owner;
ALTER TABLE public.company_enrollment_request_events OWNER TO console_account_owner;
REVOKE ALL ON TABLE public.company_enrollment_requests,public.company_enrollment_receipts,
    public.company_enrollment_request_events FROM PUBLIC,console_account_owner,console_app,
    console_rt,console_auth_rt,console_auth_startup,console_terms_owner,console_credential_owner,
    console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
-- Receipt key-share needs a narrow UPDATE grant, protected even for zero rows.
CREATE FUNCTION public.company_enrollment_receipts_immutable_v1()
RETURNS trigger LANGUAGE plpgsql SECURITY INVOKER
SET search_path = pg_catalog, pg_temp
AS $receipt_immutable$
BEGIN
    RAISE EXCEPTION USING MESSAGE='company_enrollment_receipts.immutable', ERRCODE='P0001';
END
$receipt_immutable$;
ALTER FUNCTION public.company_enrollment_receipts_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_receipts_immutable_v1() FROM PUBLIC, console_account_owner;
CREATE TRIGGER company_enrollment_receipts_immutable_v1
    BEFORE UPDATE OR DELETE OR TRUNCATE ON public.company_enrollment_receipts
    FOR EACH STATEMENT EXECUTE FUNCTION public.company_enrollment_receipts_immutable_v1();
ALTER TABLE public.company_enrollment_receipts
    ENABLE ALWAYS TRIGGER company_enrollment_receipts_immutable_v1;
-- Remove every default or inherited direct ACL before granting the exact owner set.
-- Reuse migration0228 custody enumeration, including any column grants.
DO $company_schema_acl$
DECLARE relation_name text; grantee_name text; column_name text;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY['company_enrollment_requests','company_enrollment_receipts','company_enrollment_request_events'] LOOP
        FOR grantee_name IN
            SELECT DISTINCT role.rolname FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace ns ON ns.oid=relation.relnamespace
            CROSS JOIN LATERAL pg_catalog.aclexplode(relation.relacl) acl
            JOIN pg_catalog.pg_roles role ON role.oid=acl.grantee
            WHERE ns.nspname='public' AND relation.relname=relation_name
        LOOP
            EXECUTE pg_catalog.format('REVOKE ALL ON TABLE public.%I FROM %I',relation_name,grantee_name);
        END LOOP;
        FOR column_name,grantee_name IN
            SELECT att.attname,CASE WHEN acl.grantee=0 THEN NULL ELSE role.rolname END
            FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace ns ON ns.oid=relation.relnamespace
            JOIN pg_catalog.pg_attribute att ON att.attrelid=relation.oid
            CROSS JOIN LATERAL pg_catalog.aclexplode(att.attacl) acl
            LEFT JOIN pg_catalog.pg_roles role ON role.oid=acl.grantee
            WHERE ns.nspname='public' AND relation.relname=relation_name
              AND att.attnum>0 AND NOT att.attisdropped
        LOOP
            EXECUTE pg_catalog.format('REVOKE ALL (%I) ON TABLE public.%I FROM %s',
                column_name,relation_name,CASE WHEN grantee_name IS NULL THEN 'PUBLIC' ELSE pg_catalog.quote_ident(grantee_name) END);
        END LOOP;
    END LOOP;
END
$company_schema_acl$;
GRANT SELECT,INSERT ON TABLE public.company_enrollment_requests,public.company_enrollment_receipts,
    public.company_enrollment_request_events TO console_account_owner;
-- Reuse existing custody pattern (generate-account-custody.py:4568): FK
-- request->receipt checks need key-share locking with narrow key UPDATE.
-- Actual UPDATE, including SET receipt_id=receipt_id, is refused by ALWAYS guard.
GRANT UPDATE(receipt_id) ON public.company_enrollment_receipts TO console_account_owner;
GRANT UPDATE(input_bytes,state,terminal_at,committed_receipt_id)
    ON public.company_enrollment_requests TO console_account_owner;
-- No default privileges, serving grants, request/event owners or profile activation.
"""


def company_enrollment_input_sql():
    return r"""-- Generated by ops/generate-account-custody.py. UNINSTALLED candidate.
-- Pure immutable v1 decoder. Identifiers are data, never authority.
-- Do not install through the current serving finalizer or reseal its profile.
CREATE FUNCTION public.company_enrollment_decode_input_v1(p_input bytea)
RETURNS TABLE (
    codec_version smallint, account_id uuid, command_id uuid, group_id uuid,
    administrative_account_id uuid, company_slug text, company_name text,
    input_digest bytea
)
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER CALLED ON NULL INPUT
SET search_path = pg_catalog, pg_temp
AS $company_input$
DECLARE
    byte_count integer := octet_length(p_input);
    cursor_pos integer := 62; -- Zero-based, immediately after Group discriminator.
    field_number integer;
    field_length bigint;
    field_value text;
    codepoint integer;
    character_index integer;
    nonblank boolean := false;
BEGIN
    -- Bound every read/allocation, including hostile four-byte lengths.
    IF p_input IS NULL OR byte_count NOT BETWEEN 88 AND 421
       OR substring(p_input FROM 1 FOR 29)
          <> decode('636f6e736f6c652e636f6d70616e792e656e726f6c6c6d656e74000001','hex') THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
    END IF;
    account_id := encode(substring(p_input FROM 30 FOR 16),'hex')::uuid;
    command_id := encode(substring(p_input FROM 46 FOR 16),'hex')::uuid;
    CASE get_byte(p_input,61)
        WHEN 0 THEN group_id := NULL;
        WHEN 1 THEN
            group_id := encode(substring(p_input FROM 63 FOR 16),'hex')::uuid;
            cursor_pos := 78;
            IF group_id = '00000000-0000-0000-0000-000000000000'::uuid THEN
                RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
            END IF;
        ELSE
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
    END CASE;
    administrative_account_id := encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid;
    cursor_pos := cursor_pos+16;
    IF '00000000-0000-0000-0000-000000000000'::uuid
       IN (account_id,command_id,administrative_account_id) THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
    END IF;
    FOR field_number IN 1..2 LOOP
        IF byte_count-cursor_pos < 4 THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
        END IF;
        field_length := get_byte(p_input,cursor_pos)*16777216::bigint
                      + get_byte(p_input,cursor_pos+1)*65536::bigint
                      + get_byte(p_input,cursor_pos+2)*256::bigint
                      + get_byte(p_input,cursor_pos+3);
        cursor_pos := cursor_pos+4;
        IF field_length NOT BETWEEN 1 AND (CASE field_number WHEN 1 THEN 63 ELSE 256 END)
           OR field_length > byte_count-cursor_pos THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
        END IF;
        field_value := convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');
        cursor_pos := cursor_pos+field_length::integer;
        IF field_number=1 THEN company_slug := field_value;
        ELSE company_name := field_value;
        END IF;
    END LOOP;
    IF cursor_pos <> byte_count
       OR company_slug COLLATE "C" !~ '^[a-z0-9]([a-z0-9-]*[a-z0-9])?$' THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
    END IF;
    -- Match Rust char::is_control and str::trim, independent of database locale.
    -- Controls include U+0085 even though Unicode also classifies it as whitespace.
    FOR character_index IN 1..char_length(company_name) LOOP
        codepoint := ascii(substring(company_name FROM character_index FOR 1));
        IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159 THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
        END IF;
        IF codepoint NOT IN (32,160,5760,8232,8233,8239,8287,12288)
           AND codepoint NOT BETWEEN 8192 AND 8202 THEN
            nonblank := true;
        END IF;
    END LOOP;
    IF NOT nonblank THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
    END IF;
    codec_version := 1;
    input_digest := sha256(p_input);
    RETURN NEXT;
EXCEPTION WHEN data_exception THEN
    -- Invalid UTF-8, embedded NUL, truncated UUIDs and bounds errors disclose no input.
    RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='company_enrollment.invalid_input';
END
$company_input$;
ALTER FUNCTION public.company_enrollment_decode_input_v1(bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_decode_input_v1(bytea) FROM PUBLIC;
"""


# Auth7 constants freeze the admitted226 catalog and explicit final mutations.
CREDENTIAL_TABLES = ('auth_bootstrap_credentials', 'auth_device_login_handoffs', 'auth_refresh_token_families', 'auth_refresh_tokens', 'auth_webauthn_ceremonies', 'auth_webauthn_ceremony_bindings', 'auth_webauthn_credentials')
CREDENTIAL_STATE_QUERY = r"""-- Generated from admitted source226 plus the fixed Auth7 transition in ops/generate-account-custody.py.
-- Read-only metadata verdict; no credential/user/audit row reads and no expected-data learning.
WITH wanted(name) AS (VALUES
 ('auth_webauthn_credentials'), ('auth_webauthn_ceremonies'),
 ('auth_refresh_token_families'), ('auth_refresh_tokens'),
 ('auth_bootstrap_credentials'), ('auth_webauthn_ceremony_bindings'),
 ('auth_device_login_handoffs')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE n.nspname='public' AND p.proname IN ('auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org')
), credential_role AS (
 SELECT oid,NOT (rolsuper OR rolcanlogin OR rolinherit OR rolbypassrls OR rolcreatedb OR rolcreaterole OR rolreplication)
   AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=r.oid OR m.member=r.oid)
   AND NOT EXISTS(SELECT 1 FROM pg_default_acl d WHERE d.defaclrole=r.oid)
   AND NOT has_schema_privilege(r.oid,'public','CREATE') AS valid
 FROM pg_roles r WHERE rolname='console_credential_owner'
), role_boundaries AS (
 SELECT NOT EXISTS(SELECT 1 FROM credential_role WHERE NOT valid)
   AND NOT EXISTS(SELECT 1 FROM pg_roles r WHERE r.rolname='console_auth_rt'
    AND (r.rolsuper OR r.rolinherit OR r.rolbypassrls OR r.rolcreatedb OR r.rolcreaterole OR r.rolreplication))
   AND EXISTS(SELECT 1 FROM pg_roles WHERE rolname='console_auth_rt')
   AND NOT EXISTS(SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.roleid OR r.oid=m.member WHERE r.rolname='console_auth_rt')
   AND NOT EXISTS(SELECT 1 FROM credential_role owner_role JOIN pg_class c ON c.relowner=owner_role.oid
      WHERE c.relkind IN ('r','p','v','m','S','f') AND c.oid NOT IN (SELECT oid FROM relations))
   AND NOT EXISTS(SELECT 1 FROM credential_role r JOIN pg_namespace n ON n.nspowner=r.oid)
   AND NOT EXISTS(SELECT 1 FROM credential_role r CROSS JOIN pg_class c
     CROSS JOIN LATERAL aclexplode(c.relacl) x WHERE x.grantee=r.oid AND c.oid NOT IN (SELECT oid FROM relations))
   AND NOT EXISTS(SELECT 1 FROM credential_role r CROSS JOIN pg_attribute a
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE x.grantee=r.oid AND a.attrelid NOT IN (SELECT oid FROM relations))
   AND NOT EXISTS(SELECT 1 FROM credential_role owner_role JOIN pg_proc p ON p.proowner=owner_role.oid
      WHERE p.oid NOT IN (SELECT oid FROM routine_records))
   AND NOT EXISTS(SELECT 1 FROM credential_role r CROSS JOIN (VALUES('users'),('audit_events'),('accounts'),('account_security'),
     ('account_security_events'),('account_terms_acceptances'),('account_terms_head'),('account_terms_release_receipts')) denied(name)
     WHERE has_table_privilege(r.oid,to_regclass('public.'||denied.name),'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
      OR has_any_column_privilege(r.oid,to_regclass('public.'||denied.name),'SELECT,INSERT,UPDATE,REFERENCES')) AS valid
), foreign_keys AS (
 SELECT k.*,
   k.connamespace='public'::regnamespace AND k.contypid=0 AND k.conparentid=0
   AND k.convalidated AND k.conenforced AND NOT k.conperiod AND k.connoinherit
   AND k.conislocal AND k.coninhcount=0 AND NOT k.condeferrable AND NOT k.condeferred
   AND k.confmatchtype='s' AND k.confdelsetcols IS NULL AND k.conbin IS NULL AND k.conexclop IS NULL
   AND k.conpfeqop=array_fill('pg_catalog.=(uuid,uuid)'::regoperator::oid,ARRAY[cardinality(k.conkey)])
   AND k.conppeqop=k.conpfeqop AND k.conffeqop=k.conpfeqop
   AND i.indrelid=k.confrelid AND i.indisunique AND i.indisvalid AND i.indisready
   AND i.indislive AND i.indimmediate AND NOT i.indisexclusion
   AND i.indexprs IS NULL AND i.indpred IS NULL AND i.indnkeyatts=cardinality(k.confkey)
   AND i.indnatts=i.indnkeyatts AND i.indkey::text=array_to_string(k.confkey,' ')
   AND NOT EXISTS(SELECT 1 FROM unnest(k.conkey) key_column LEFT JOIN pg_attribute a
      ON a.attrelid=k.conrelid AND a.attnum=key_column
      WHERE a.atttypid IS DISTINCT FROM 'pg_catalog.uuid'::regtype::oid OR a.attisdropped) AS valid
 FROM pg_constraint k LEFT JOIN pg_index i ON i.indexrelid=k.conindid
 WHERE k.contype='f' AND k.conrelid IN (SELECT oid FROM relations)
), foreign_ri AS (
 SELECT f.oid,
   (SELECT count(*)=4 FROM pg_trigger t WHERE t.tgconstraint=f.oid)
   AND (SELECT count(*)=4 AND count(DISTINCT e.function_name)=4 AND bool_and(
      t.tgrelid=CASE WHEN e.child THEN f.conrelid ELSE f.confrelid END
      AND t.tgconstrrelid=CASE WHEN e.child THEN f.confrelid ELSE f.conrelid END
      AND t.tgconstrindid=f.conindid AND t.tgconstraint=f.oid
      AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
      AND NOT t.tgdeferrable AND NOT t.tginitdeferred AND t.tgparentid=0
      AND t.tgname::text ~ CASE WHEN e.child THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
      AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
      AND t.tgqual IS NULL AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
    FROM (VALUES(true,'RI_FKey_check_ins',5),(true,'RI_FKey_check_upd',17),
      (false,'RI_FKey_'||CASE f.confdeltype WHEN 'r' THEN 'restrict' WHEN 'a' THEN 'noaction' WHEN 'c' THEN 'cascade' WHEN 'n' THEN 'setnull' END||'_del',9),
      (false,'RI_FKey_'||CASE f.confupdtype WHEN 'r' THEN 'restrict' WHEN 'a' THEN 'noaction' END||'_upd',17)) e(child,function_name,trigger_type)
    JOIN pg_trigger t ON t.tgconstraint=f.oid AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM foreign_keys f
), metadata_boundary AS (
 SELECT NOT EXISTS(SELECT 1 FROM foreign_keys WHERE valid IS DISTINCT FROM true)
   AND NOT EXISTS(SELECT 1 FROM foreign_ri WHERE valid IS DISTINCT FROM true)
   AND NOT EXISTS(SELECT 1 FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)
      AND (NOT k.conenforced OR k.conperiod OR k.contypid<>0 OR k.conparentid<>0 OR k.coninhcount<>0 OR NOT k.conislocal))
   AND NOT EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgrelid IN (SELECT oid FROM relations)
     AND (t.tgparentid<>0 OR t.tgnargs<>0 OR octet_length(t.tgargs)<>0 OR t.tgattr<>''::int2vector
       OR t.tgoldtable IS NOT NULL OR t.tgnewtable IS NOT NULL OR (t.tgisinternal AND t.tgqual IS NOT NULL))) AS valid
), snapshots AS (
 SELECT (SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records) AS tables,
        (SELECT jsonb_agg(record ORDER BY record->>'name') FROM routine_records) AS routines
)
SELECT CASE
 WHEN (SELECT valid FROM role_boundaries) IS DISTINCT FROM true OR (SELECT valid FROM metadata_boundary) IS DISTINCT FROM true
   THEN CASE WHEN EXISTS(SELECT 1 FROM relations WHERE relowner=(SELECT oid FROM credential_role)) THEN 'account_credentials.profile_mismatch' ELSE 'account_credentials.legacy_profile_mismatch' END
 WHEN s.tables='[{"name":"auth_bootstrap_credentials","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"registration_ceremony_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"registration_started_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[11,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[12,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_bootstrap_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_pkey ON public.auth_bootstrap_credentials USING btree (id)"],["auth_bootstrap_credentials_registration_ceremony_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_registration_ceremony_id_key ON public.auth_bootstrap_credentials USING btree (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_token_hash_key ON public.auth_bootstrap_credentials USING btree (token_hash)"],["idx_auth_bootstrap_credentials_active_hash",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_active_hash ON public.auth_bootstrap_credentials USING btree (token_hash, expires_at) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_one_open_per_user",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX idx_auth_bootstrap_credentials_one_open_per_user ON public.auth_bootstrap_credentials USING btree (user_id) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_user ON public.auth_bootstrap_credentials USING btree (user_id, issued_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_bootstrap_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_bootstrap_credentials_org_immutable BEFORE UPDATE ON public.auth_bootstrap_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null]],"constraints":[["auth_bootstrap_credentials_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_bootstrap_credentials_check1","c",true,false,false,false,true,0,true,[6,7],null," "," "," ",null,null,"CHECK (((registration_ceremony_id IS NULL) OR (registration_started_at IS NOT NULL)))"],["auth_bootstrap_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[11],null," "," "," ",null,null,"NOT NULL created_at"],["auth_bootstrap_credentials_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_bootstrap_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_bootstrap_credentials_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_bootstrap_credentials_org_fk","f",true,false,false,true,true,0,true,[12],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_bootstrap_credentials_org_id_not_null","n",true,false,false,false,true,0,true,[12],null," "," "," ",null,null,"NOT NULL org_id"],["auth_bootstrap_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_bootstrap_credentials_registration_ceremony_id_fkey","f",true,false,false,true,true,0,true,[6],[1],"a","n","s","public","auth_webauthn_ceremonies","FOREIGN KEY (registration_ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON DELETE SET NULL"],["auth_bootstrap_credentials_registration_ceremony_id_key","u",true,false,false,true,true,0,true,[6],null," "," "," ",null,null,"UNIQUE (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_bootstrap_credentials_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_bootstrap_credentials_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_bootstrap_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"],["auth_bootstrap_credentials_user_same_org_fk","f",true,false,false,true,true,0,true,[2,12],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"registration_ceremony_id","number":6,"acl_is_null":true},{"acl":null,"name":"registration_started_at","number":7,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":10,"acl_is_null":true},{"acl":null,"name":"created_at","number":11,"acl_is_null":true},{"acl":null,"name":"org_id","number":12,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]},{"name":"auth_device_login_handoffs","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"poll_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[3,"approve_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"target_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"target_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[8,"approved_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"approved_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[10,"approved_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[11,"approved_passkey_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[12,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[13,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_device_login_handoffs_approve_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_approve_token_hash_key ON public.auth_device_login_handoffs USING btree (approve_token_hash)"],["auth_device_login_handoffs_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_pkey ON public.auth_device_login_handoffs USING btree (id)"],["auth_device_login_handoffs_poll_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_poll_token_hash_key ON public.auth_device_login_handoffs USING btree (poll_token_hash)"],["idx_auth_device_login_handoffs_approve_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_approve_active ON public.auth_device_login_handoffs USING btree (approve_token_hash, expires_at) WHERE ((approved_at IS NULL) AND (consumed_at IS NULL))"],["idx_auth_device_login_handoffs_poll_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_poll_active ON public.auth_device_login_handoffs USING btree (poll_token_hash, expires_at) WHERE (consumed_at IS NULL)"],["idx_auth_device_login_handoffs_target",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_target ON public.auth_device_login_handoffs USING btree (target_user_id, issued_at DESC) WHERE (target_user_id IS NOT NULL)"],["idx_auth_device_login_handoffs_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_user ON public.auth_device_login_handoffs USING btree (approved_user_id, approved_at DESC) WHERE (approved_user_id IS NOT NULL)"]],"policies":0,"relation":["r","p",false,false,false,"d",null],"triggers":null,"constraints":[["auth_device_login_handoffs_approve_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (approve_token_hash)"],["auth_device_login_handoffs_approve_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL approve_token_hash"],["auth_device_login_handoffs_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_device_login_handoffs_check1","c",true,false,false,false,true,0,true,[6,7],null," "," "," ",null,null,"CHECK (((target_user_id IS NULL) = (target_org_id IS NULL)))"],["auth_device_login_handoffs_check2","c",true,false,false,false,true,0,true,[8,9,10],null," "," "," ",null,null,"CHECK ((((approved_at IS NULL) AND (approved_user_id IS NULL) AND (approved_org_id IS NULL)) OR ((approved_at IS NOT NULL) AND (approved_user_id IS NOT NULL) AND (approved_org_id IS NOT NULL))))"],["auth_device_login_handoffs_created_at_not_null","n",true,false,false,false,true,0,true,[13],null," "," "," ",null,null,"NOT NULL created_at"],["auth_device_login_handoffs_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_device_login_handoffs_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_device_login_handoffs_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_device_login_handoffs_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_device_login_handoffs_poll_token_hash_key","u",true,false,false,true,true,0,true,[2],null," "," "," ",null,null,"UNIQUE (poll_token_hash)"],["auth_device_login_handoffs_poll_token_hash_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL poll_token_hash"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"poll_token_hash","number":2,"acl_is_null":true},{"acl":null,"name":"approve_token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"target_user_id","number":6,"acl_is_null":true},{"acl":null,"name":"target_org_id","number":7,"acl_is_null":true},{"acl":null,"name":"approved_at","number":8,"acl_is_null":true},{"acl":null,"name":"approved_user_id","number":9,"acl_is_null":true},{"acl":null,"name":"approved_org_id","number":10,"acl_is_null":true},{"acl":null,"name":"approved_passkey_id","number":11,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":12,"acl_is_null":true},{"acl":null,"name":"created_at","number":13,"acl_is_null":true}],"policies":[]},{"name":"auth_refresh_token_families","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[4,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[5,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[6,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_refresh_token_families_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_token_families_pkey ON public.auth_refresh_token_families USING btree (id)"],["idx_auth_refresh_token_families_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_token_families_user ON public.auth_refresh_token_families USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_token_families_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_token_families_org_immutable BEFORE UPDATE ON public.auth_refresh_token_families FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_cascade_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_token_families_created_at_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL created_at"],["auth_refresh_token_families_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_token_families_org_fk","f",true,false,false,true,true,0,true,[6],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_token_families_org_id_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL org_id"],["auth_refresh_token_families_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_token_families_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_refresh_token_families_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"],["auth_refresh_token_families_user_same_org_fk","f",true,false,false,true,true,0,true,[2,6],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"created_at","number":3,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":4,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":5,"acl_is_null":true},{"acl":null,"name":"org_id","number":6,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]},{"name":"auth_refresh_tokens","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"family_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[5,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"replaced_by","pg_catalog","uuid",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"reuse_detected_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[11,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_refresh_tokens_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_tokens_pkey ON public.auth_refresh_tokens USING btree (id)"],["auth_refresh_tokens_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_refresh_tokens_token_hash_key ON public.auth_refresh_tokens USING btree (token_hash)"],["idx_auth_refresh_tokens_family",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_family ON public.auth_refresh_tokens USING btree (family_id, issued_at)"],["idx_auth_refresh_tokens_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_user_active ON public.auth_refresh_tokens USING btree (user_id, expires_at) WHERE ((used_at IS NULL) AND (revoked_at IS NULL))"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_tokens_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_tokens_org_immutable BEFORE UPDATE ON public.auth_refresh_tokens FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_token_families",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_tokens",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_token_families",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_noaction_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_tokens_check","c",true,false,false,false,true,0,true,[6,5],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_refresh_tokens_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_refresh_tokens_family_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","auth_refresh_token_families","FOREIGN KEY (family_id) REFERENCES public.auth_refresh_token_families(id) ON DELETE CASCADE"],["auth_refresh_tokens_family_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL family_id"],["auth_refresh_tokens_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_tokens_issued_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_refresh_tokens_org_fk","f",true,false,false,true,true,0,true,[11],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_tokens_org_id_not_null","n",true,false,false,false,true,0,true,[11],null," "," "," ",null,null,"NOT NULL org_id"],["auth_refresh_tokens_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_tokens_replaced_by_fkey","f",true,false,false,true,true,0,true,[8],[1],"a","a","s","public","auth_refresh_tokens","FOREIGN KEY (replaced_by) REFERENCES public.auth_refresh_tokens(id)"],["auth_refresh_tokens_token_hash_key","u",true,false,false,true,true,0,true,[4],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_refresh_tokens_token_hash_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_refresh_tokens_user_id_fkey","f",true,false,false,true,true,0,true,[3],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_refresh_tokens_user_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL user_id"],["auth_refresh_tokens_user_same_org_fk","f",true,false,false,true,true,0,true,[3,11],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"family_id","number":2,"acl_is_null":true},{"acl":null,"name":"user_id","number":3,"acl_is_null":true},{"acl":null,"name":"token_hash","number":4,"acl_is_null":true},{"acl":null,"name":"issued_at","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"used_at","number":7,"acl_is_null":true},{"acl":null,"name":"replaced_by","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"reuse_detected_at","number":10,"acl_is_null":true},{"acl":null,"name":"org_id","number":11,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]},{"name":"auth_webauthn_ceremonies","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[3,"ceremony_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"challenge_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"state_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremonies_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremonies_pkey ON public.auth_webauthn_ceremonies USING btree (id)"],["idx_auth_webauthn_ceremonies_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremonies_user_active ON public.auth_webauthn_ceremonies USING btree (user_id, ceremony_kind, expires_at) WHERE (consumed_at IS NULL)"]],"policies":0,"relation":["r","p",false,false,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_bootstrap_credentials",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_cascade_del","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_setnull_del","public","auth_bootstrap_credentials",null]],"constraints":[["auth_webauthn_ceremonies_ceremony_kind_check","c",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"CHECK ((ceremony_kind = ANY (ARRAY[''registration''::text, ''authentication''::text])))"],["auth_webauthn_ceremonies_ceremony_kind_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL ceremony_kind"],["auth_webauthn_ceremonies_challenge_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL challenge_json"],["auth_webauthn_ceremonies_check","c",true,false,false,false,true,0,true,[6,8],null," "," "," ",null,null,"CHECK ((expires_at > created_at))"],["auth_webauthn_ceremonies_created_at_not_null","n",true,false,false,false,true,0,true,[8],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremonies_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_webauthn_ceremonies_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_ceremonies_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_ceremonies_state_json_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL state_json"],["auth_webauthn_ceremonies_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"ceremony_kind","number":3,"acl_is_null":true},{"acl":null,"name":"challenge_json","number":4,"acl_is_null":true},{"acl":null,"name":"state_json","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":7,"acl_is_null":true},{"acl":null,"name":"created_at","number":8,"acl_is_null":true}],"policies":[]},{"name":"auth_webauthn_ceremony_bindings","owner":"console_app","shape":{"rules":null,"columns":[[1,"ceremony_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[2,"action_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[3,"object_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"reason_key","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[5,"replay_attempt","pg_catalog","int4",-1,false,false,"","",null,null,null],[6,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremony_bindings_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremony_bindings_pkey ON public.auth_webauthn_ceremony_bindings USING btree (ceremony_id)"],["idx_auth_webauthn_ceremony_bindings_action",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremony_bindings_action ON public.auth_webauthn_ceremony_bindings USING btree (action_kind, object_id, replay_attempt)"]],"policies":0,"relation":["r","p",false,false,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null]],"constraints":[["auth_webauthn_ceremony_bindings_action_kind_check","c",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"CHECK ((action_kind = ANY (ARRAY[''APPROVAL_DECISION''::text, ''POLL_VOTE''::text])))"],["auth_webauthn_ceremony_bindings_action_kind_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL action_kind"],["auth_webauthn_ceremony_bindings_ceremony_id_fkey","f",true,false,false,true,true,0,true,[1],[1],"a","c","s","public","auth_webauthn_ceremonies","FOREIGN KEY (ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON DELETE CASCADE"],["auth_webauthn_ceremony_bindings_ceremony_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL ceremony_id"],["auth_webauthn_ceremony_bindings_check","c",true,false,false,false,true,0,true,[2,4],null," "," "," ",null,null,"CHECK ((((action_kind = ''APPROVAL_DECISION''::text) AND (reason_key = ''operations_passkey_approval_decision''::text)) OR ((action_kind = ''POLL_VOTE''::text) AND (reason_key = ''operations_passkey_poll_vote''::text))))"],["auth_webauthn_ceremony_bindings_created_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremony_bindings_object_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL object_id"],["auth_webauthn_ceremony_bindings_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (ceremony_id)"],["auth_webauthn_ceremony_bindings_reason_key_check","c",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"CHECK ((reason_key = ANY (ARRAY[''operations_passkey_approval_decision''::text, ''operations_passkey_poll_vote''::text])))"],["auth_webauthn_ceremony_bindings_reason_key_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL reason_key"],["auth_webauthn_ceremony_bindings_replay_attempt_check","c",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"CHECK (((replay_attempt IS NULL) OR (replay_attempt >= 1)))"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"ceremony_id","number":1,"acl_is_null":true},{"acl":null,"name":"action_kind","number":2,"acl_is_null":true},{"acl":null,"name":"object_id","number":3,"acl_is_null":true},{"acl":null,"name":"reason_key","number":4,"acl_is_null":true},{"acl":null,"name":"replay_attempt","number":5,"acl_is_null":true},{"acl":null,"name":"created_at","number":6,"acl_is_null":true}],"policies":[]},{"name":"auth_webauthn_credentials","owner":"console_app","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"credential_id","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"passkey_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[6,"last_used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[7,"org_id","pg_catalog","uuid",-1,true,false,"","",null,null,null]],"indexes":[["auth_webauthn_credentials_credential_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_credential_id_key ON public.auth_webauthn_credentials USING btree (credential_id)"],["auth_webauthn_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_pkey ON public.auth_webauthn_credentials USING btree (id)"],["idx_auth_webauthn_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_credentials_user ON public.auth_webauthn_credentials USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_webauthn_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_webauthn_credentials_org_immutable BEFORE UPDATE ON public.auth_webauthn_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","users",null]],"constraints":[["auth_webauthn_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_credentials_credential_id_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (credential_id)"],["auth_webauthn_credentials_credential_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL credential_id"],["auth_webauthn_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_credentials_org_fk","f",true,false,false,true,true,0,true,[7],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_webauthn_credentials_org_id_not_null","n",true,false,false,false,true,0,true,[7],null," "," "," ",null,null,"NOT NULL org_id"],["auth_webauthn_credentials_passkey_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL passkey_json"],["auth_webauthn_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_credentials_user_id_fkey","f",true,false,false,true,true,0,true,[2],[1],"a","c","s","public","users","FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE"],["auth_webauthn_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"],["auth_webauthn_credentials_user_same_org_fk","f",true,false,false,true,true,0,true,[2,7],[1,8],"a","r","s","public","users","FOREIGN KEY (user_id, org_id) REFERENCES public.users(id, org_id) ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_app","console_app","DELETE",false],["console_app","console_app","INSERT",false],["console_app","console_app","MAINTAIN",false],["console_app","console_app","REFERENCES",false],["console_app","console_app","SELECT",false],["console_app","console_app","TRIGGER",false],["console_app","console_app","TRUNCATE",false],["console_app","console_app","UPDATE",false],["console_app","console_rt","DELETE",false],["console_app","console_rt","INSERT",false],["console_app","console_rt","SELECT",false],["console_app","console_rt","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"credential_id","number":3,"acl_is_null":true},{"acl":null,"name":"passkey_json","number":4,"acl_is_null":true},{"acl":null,"name":"created_at","number":5,"acl_is_null":true},{"acl":null,"name":"last_used_at","number":6,"acl_is_null":true},{"acl":null,"name":"org_id","number":7,"acl_is_null":true}],"policies":[{"name":"org_isolation","check":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","roles":["PUBLIC"],"using":"(org_id = (NULLIF(current_setting(''app.current_org''::text, true), ''''::text))::uuid)","command":"*","permissive":true}]}]'::jsonb AND s.routines='[{"name":"enforce_org_id_immutable","identity_arguments":"","result":"trigger","owner":"console_app","language":"plpgsql","kind":"f","security_definer":false,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":null,"argnames":null,"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"a46ee31ccf7fae026153b8bea7419777f055a4028e9ecfe0632e262a6f41f3ba","acl":[["console_app","PUBLIC","EXECUTE",false],["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_direct_org_children","identity_arguments":"p_id uuid","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"8c1cc63c439dd5aafe28090526099c33966e882e0b28e647a9f8ecec34458ba6","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"5d6c71c6b4d393d90e51a187e3e016bd02d4fdc47ff684cfee396d752efe5224","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_list_group_accounts","identity_arguments":"p_group_id uuid","result":"TABLE(user_id uuid, display_name text, phone text, tenant_roles text[], is_active boolean, has_passkey boolean, account_status text, org_id uuid, org_slug text, org_name text, group_roles text[], created_at timestamp with time zone)","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_group_id","user_id","display_name","phone","tenant_roles","is_active","has_passkey","account_status","org_id","org_slug","org_name","group_roles","created_at"],"argmodes":["i","t","t","t","t","t","t","t","t","t","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"6a7ff12b1e25edd0ec06ab431f897601a01731078620c363dcdbe5cb5387af71","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"b8b9c005155e036ad206fdbd2426ab39591726cf5dee2e89e1e02186b716d77a","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_bootstrap_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"27f96d16d7e825bed30402da9d9e4d1bdd0695ebec669b438d3805250d956233","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_credential_org","identity_arguments":"p_credential_id text","result":"uuid","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_credential_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"360c2f961244beee18e21607fd0e38635a137b568ae15889b11a2c91993d98bf","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_token_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_app","language":"sql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"125b8ba9b42757a439cb9c74e175bb11d2af62e3df99d34d71b88f05ab8a155c","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]}]'::jsonb
   AND NOT EXISTS(SELECT 1 FROM routine_records WHERE NOT extra_valid)
   THEN 'account_credentials.pending'
 WHEN s.tables='[{"name":"auth_bootstrap_credentials","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"registration_ceremony_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"registration_started_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[11,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[12,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_bootstrap_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_pkey ON public.auth_bootstrap_credentials USING btree (id)"],["auth_bootstrap_credentials_registration_ceremony_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_registration_ceremony_id_key ON public.auth_bootstrap_credentials USING btree (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_bootstrap_credentials_token_hash_key ON public.auth_bootstrap_credentials USING btree (token_hash)"],["idx_auth_bootstrap_credentials_active_hash",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_active_hash ON public.auth_bootstrap_credentials USING btree (token_hash, expires_at) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_one_open_per_user",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX idx_auth_bootstrap_credentials_one_open_per_user ON public.auth_bootstrap_credentials USING btree (user_id) WHERE ((consumed_at IS NULL) AND (revoked_at IS NULL))"],["idx_auth_bootstrap_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_bootstrap_credentials_user ON public.auth_bootstrap_credentials USING btree (user_id, issued_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_bootstrap_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_bootstrap_credentials_org_immutable BEFORE UPDATE ON public.auth_bootstrap_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null]],"constraints":[["auth_bootstrap_credentials_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_bootstrap_credentials_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_bootstrap_credentials_check1","c",true,false,false,false,true,0,true,[6,7],null," "," "," ",null,null,"CHECK (((registration_ceremony_id IS NULL) OR (registration_started_at IS NOT NULL)))"],["auth_bootstrap_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[11],null," "," "," ",null,null,"NOT NULL created_at"],["auth_bootstrap_credentials_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_bootstrap_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_bootstrap_credentials_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_bootstrap_credentials_org_fk","f",true,false,false,true,true,0,true,[12],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_bootstrap_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_bootstrap_credentials_registration_ceremony_id_fkey","f",true,false,false,true,true,0,true,[6],[1],"r","r","s","public","auth_webauthn_ceremonies","FOREIGN KEY (registration_ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_bootstrap_credentials_registration_ceremony_id_key","u",true,false,false,true,true,0,true,[6],null," "," "," ",null,null,"UNIQUE (registration_ceremony_id)"],["auth_bootstrap_credentials_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_bootstrap_credentials_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_bootstrap_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","INSERT",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"registration_ceremony_id","number":6,"acl_is_null":true},{"acl":null,"name":"registration_started_at","number":7,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":10,"acl_is_null":true},{"acl":null,"name":"created_at","number":11,"acl_is_null":true},{"acl":null,"name":"org_id","number":12,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_device_login_handoffs","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"poll_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[3,"approve_token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[4,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[5,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"target_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[7,"target_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[8,"approved_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[9,"approved_user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[10,"approved_org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[11,"approved_passkey_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[12,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[13,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_device_login_handoffs_approve_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_approve_token_hash_key ON public.auth_device_login_handoffs USING btree (approve_token_hash)"],["auth_device_login_handoffs_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_pkey ON public.auth_device_login_handoffs USING btree (id)"],["auth_device_login_handoffs_poll_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_device_login_handoffs_poll_token_hash_key ON public.auth_device_login_handoffs USING btree (poll_token_hash)"],["idx_auth_device_login_handoffs_approve_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_approve_active ON public.auth_device_login_handoffs USING btree (approve_token_hash, expires_at) WHERE ((approved_at IS NULL) AND (consumed_at IS NULL))"],["idx_auth_device_login_handoffs_poll_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_poll_active ON public.auth_device_login_handoffs USING btree (poll_token_hash, expires_at) WHERE (consumed_at IS NULL)"],["idx_auth_device_login_handoffs_target",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_target ON public.auth_device_login_handoffs USING btree (target_user_id, issued_at DESC) WHERE (target_user_id IS NOT NULL)"],["idx_auth_device_login_handoffs_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_device_login_handoffs_user ON public.auth_device_login_handoffs USING btree (approved_user_id, approved_at DESC) WHERE (approved_user_id IS NOT NULL)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null]],"constraints":[["auth_device_login_handoffs_approve_token_hash_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (approve_token_hash)"],["auth_device_login_handoffs_approve_token_hash_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL approve_token_hash"],["auth_device_login_handoffs_approved_account_v1","f",true,false,false,true,true,0,true,[9],[1],"r","r","s","public","accounts","FOREIGN KEY (approved_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_device_login_handoffs_check","c",true,false,false,false,true,0,true,[5,4],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_device_login_handoffs_check1","c",true,false,false,false,true,0,true,[7,6],null," "," "," ",null,null,"CHECK (((target_org_id IS NULL) OR (target_user_id IS NOT NULL)))"],["auth_device_login_handoffs_check2","c",true,false,false,false,true,0,true,[8,9,10],null," "," "," ",null,null,"CHECK ((((approved_at IS NULL) AND (approved_user_id IS NULL) AND (approved_org_id IS NULL)) OR ((approved_at IS NOT NULL) AND (approved_user_id IS NOT NULL))))"],["auth_device_login_handoffs_created_at_not_null","n",true,false,false,false,true,0,true,[13],null," "," "," ",null,null,"NOT NULL created_at"],["auth_device_login_handoffs_expires_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_device_login_handoffs_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_device_login_handoffs_issued_at_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_device_login_handoffs_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_device_login_handoffs_poll_token_hash_key","u",true,false,false,true,true,0,true,[2],null," "," "," ",null,null,"UNIQUE (poll_token_hash)"],["auth_device_login_handoffs_poll_token_hash_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL poll_token_hash"],["auth_device_login_handoffs_target_account_v1","f",true,false,false,true,true,0,true,[6],[1],"r","r","s","public","accounts","FOREIGN KEY (target_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","SELECT",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"poll_token_hash","number":2,"acl_is_null":true},{"acl":null,"name":"approve_token_hash","number":3,"acl_is_null":true},{"acl":null,"name":"issued_at","number":4,"acl_is_null":true},{"acl":null,"name":"expires_at","number":5,"acl_is_null":true},{"acl":null,"name":"target_user_id","number":6,"acl_is_null":true},{"acl":null,"name":"target_org_id","number":7,"acl_is_null":true},{"acl":null,"name":"approved_at","number":8,"acl_is_null":true},{"acl":null,"name":"approved_user_id","number":9,"acl_is_null":true},{"acl":null,"name":"approved_org_id","number":10,"acl_is_null":true},{"acl":null,"name":"approved_passkey_id","number":11,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":12,"acl_is_null":true},{"acl":null,"name":"created_at","number":13,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_refresh_token_families","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[4,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[5,"revoked_reason","pg_catalog","text",-1,false,false,"","","pg_catalog","default",null],[6,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_refresh_token_families_id_user_v1",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_refresh_token_families_id_user_v1 ON public.auth_refresh_token_families USING btree (id, user_id)"],["auth_refresh_token_families_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_token_families_pkey ON public.auth_refresh_token_families USING btree (id)"],["idx_auth_refresh_token_families_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_token_families_user ON public.auth_refresh_token_families USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_token_families_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_token_families_org_immutable BEFORE UPDATE ON public.auth_refresh_token_families FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_restrict_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_restrict_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_token_families_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_refresh_token_families_created_at_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL created_at"],["auth_refresh_token_families_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_token_families_id_user_v1","u",true,false,false,true,true,0,true,[1,2],null," "," "," ",null,null,"UNIQUE (id, user_id)"],["auth_refresh_token_families_org_fk","f",true,false,false,true,true,0,true,[6],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_token_families_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_token_families_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"created_at","number":3,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":4,"acl_is_null":true},{"acl":null,"name":"revoked_reason","number":5,"acl_is_null":true},{"acl":null,"name":"org_id","number":6,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_refresh_tokens","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"family_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"token_hash","pg_catalog","bytea",-1,true,false,"","",null,null,null],[5,"issued_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"replaced_by","pg_catalog","uuid",-1,false,false,"","",null,null,null],[9,"revoked_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[10,"reuse_detected_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[11,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_refresh_tokens_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_refresh_tokens_pkey ON public.auth_refresh_tokens USING btree (id)"],["auth_refresh_tokens_token_hash_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_refresh_tokens_token_hash_key ON public.auth_refresh_tokens USING btree (token_hash)"],["idx_auth_refresh_tokens_family",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_family ON public.auth_refresh_tokens USING btree (family_id, issued_at)"],["idx_auth_refresh_tokens_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_refresh_tokens_user_active ON public.auth_refresh_tokens USING btree (user_id, expires_at) WHERE ((used_at IS NULL) AND (revoked_at IS NULL))"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_refresh_tokens_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_refresh_tokens_org_immutable BEFORE UPDATE ON public.auth_refresh_tokens FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_token_families",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_refresh_tokens",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_token_families",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_refresh_tokens",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_noaction_del","public","auth_refresh_tokens",null]],"constraints":[["auth_refresh_tokens_account_v1","f",true,false,false,true,true,0,true,[3],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_refresh_tokens_check","c",true,false,false,false,true,0,true,[6,5],null," "," "," ",null,null,"CHECK ((expires_at > issued_at))"],["auth_refresh_tokens_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_refresh_tokens_family_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL family_id"],["auth_refresh_tokens_family_subject_v1","f",true,false,false,true,true,0,true,[2,3],[1,2],"r","r","s","public","auth_refresh_token_families","FOREIGN KEY (family_id, user_id) REFERENCES public.auth_refresh_token_families(id, user_id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_refresh_tokens_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_refresh_tokens_issued_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL issued_at"],["auth_refresh_tokens_org_fk","f",true,false,false,true,true,0,true,[11],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_refresh_tokens_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_refresh_tokens_replaced_by_fkey","f",true,false,false,true,true,0,true,[8],[1],"a","a","s","public","auth_refresh_tokens","FOREIGN KEY (replaced_by) REFERENCES public.auth_refresh_tokens(id)"],["auth_refresh_tokens_token_hash_key","u",true,false,false,true,true,0,true,[4],null," "," "," ",null,null,"UNIQUE (token_hash)"],["auth_refresh_tokens_token_hash_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL token_hash"],["auth_refresh_tokens_user_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"family_id","number":2,"acl_is_null":true},{"acl":null,"name":"user_id","number":3,"acl_is_null":true},{"acl":null,"name":"token_hash","number":4,"acl_is_null":true},{"acl":null,"name":"issued_at","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"used_at","number":7,"acl_is_null":true},{"acl":null,"name":"replaced_by","number":8,"acl_is_null":true},{"acl":null,"name":"revoked_at","number":9,"acl_is_null":true},{"acl":null,"name":"reuse_detected_at","number":10,"acl_is_null":true},{"acl":null,"name":"org_id","number":11,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_webauthn_ceremonies","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,false,false,"","",null,null,null],[3,"ceremony_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"challenge_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"state_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[6,"expires_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,null],[7,"consumed_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[8,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremonies_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremonies_pkey ON public.auth_webauthn_ceremonies USING btree (id)"],["idx_auth_webauthn_ceremonies_user_active",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremonies_user_active ON public.auth_webauthn_ceremonies USING btree (user_id, ceremony_kind, expires_at) WHERE (consumed_at IS NULL)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_noaction_upd","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_restrict_upd","public","auth_bootstrap_credentials",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_cascade_del","public","auth_webauthn_ceremony_bindings",null],[null,true,"O",9,0,"",false,false,"pg_catalog","RI_FKey_restrict_del","public","auth_bootstrap_credentials",null]],"constraints":[["auth_webauthn_ceremonies_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_webauthn_ceremonies_ceremony_kind_check","c",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"CHECK ((ceremony_kind = ANY (ARRAY[''registration''::text, ''authentication''::text])))"],["auth_webauthn_ceremonies_ceremony_kind_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL ceremony_kind"],["auth_webauthn_ceremonies_challenge_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL challenge_json"],["auth_webauthn_ceremonies_check","c",true,false,false,false,true,0,true,[6,8],null," "," "," ",null,null,"CHECK ((expires_at > created_at))"],["auth_webauthn_ceremonies_created_at_not_null","n",true,false,false,false,true,0,true,[8],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremonies_expires_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL expires_at"],["auth_webauthn_ceremonies_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_ceremonies_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_ceremonies_state_json_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL state_json"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false],["console_credential_owner","console_credential_owner","UPDATE",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"ceremony_kind","number":3,"acl_is_null":true},{"acl":null,"name":"challenge_json","number":4,"acl_is_null":true},{"acl":null,"name":"state_json","number":5,"acl_is_null":true},{"acl":null,"name":"expires_at","number":6,"acl_is_null":true},{"acl":null,"name":"consumed_at","number":7,"acl_is_null":true},{"acl":null,"name":"created_at","number":8,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_webauthn_ceremony_bindings","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"ceremony_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[2,"action_kind","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[3,"object_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[4,"reason_key","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[5,"replay_attempt","pg_catalog","int4",-1,false,false,"","",null,null,null],[6,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"]],"indexes":[["auth_webauthn_ceremony_bindings_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_ceremony_bindings_pkey ON public.auth_webauthn_ceremony_bindings USING btree (ceremony_id)"],["idx_auth_webauthn_ceremony_bindings_action",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_ceremony_bindings_action ON public.auth_webauthn_ceremony_bindings USING btree (action_kind, object_id, replay_attempt)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","auth_webauthn_ceremonies",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","auth_webauthn_ceremonies",null]],"constraints":[["auth_webauthn_ceremony_bindings_action_kind_check","c",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"CHECK ((action_kind = ANY (ARRAY[''APPROVAL_DECISION''::text, ''POLL_VOTE''::text])))"],["auth_webauthn_ceremony_bindings_action_kind_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL action_kind"],["auth_webauthn_ceremony_bindings_ceremony_id_fkey","f",true,false,false,true,true,0,true,[1],[1],"a","c","s","public","auth_webauthn_ceremonies","FOREIGN KEY (ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON DELETE CASCADE"],["auth_webauthn_ceremony_bindings_ceremony_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL ceremony_id"],["auth_webauthn_ceremony_bindings_check","c",true,false,false,false,true,0,true,[2,4],null," "," "," ",null,null,"CHECK ((((action_kind = ''APPROVAL_DECISION''::text) AND (reason_key = ''operations_passkey_approval_decision''::text)) OR ((action_kind = ''POLL_VOTE''::text) AND (reason_key = ''operations_passkey_poll_vote''::text))))"],["auth_webauthn_ceremony_bindings_created_at_not_null","n",true,false,false,false,true,0,true,[6],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_ceremony_bindings_object_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL object_id"],["auth_webauthn_ceremony_bindings_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (ceremony_id)"],["auth_webauthn_ceremony_bindings_reason_key_check","c",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"CHECK ((reason_key = ANY (ARRAY[''operations_passkey_approval_decision''::text, ''operations_passkey_poll_vote''::text])))"],["auth_webauthn_ceremony_bindings_reason_key_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL reason_key"],["auth_webauthn_ceremony_bindings_replay_attempt_check","c",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"CHECK (((replay_attempt IS NULL) OR (replay_attempt >= 1)))"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false]],"column_security":[{"acl":null,"name":"ceremony_id","number":1,"acl_is_null":true},{"acl":null,"name":"action_kind","number":2,"acl_is_null":true},{"acl":null,"name":"object_id","number":3,"acl_is_null":true},{"acl":null,"name":"reason_key","number":4,"acl_is_null":true},{"acl":null,"name":"replay_attempt","number":5,"acl_is_null":true},{"acl":null,"name":"created_at","number":6,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]},{"name":"auth_webauthn_credentials","owner":"console_credential_owner","shape":{"rules":null,"columns":[[1,"id","pg_catalog","uuid",-1,true,false,"","",null,null,"gen_random_uuid()"],[2,"user_id","pg_catalog","uuid",-1,true,false,"","",null,null,null],[3,"credential_id","pg_catalog","text",-1,true,false,"","","pg_catalog","default",null],[4,"passkey_json","pg_catalog","jsonb",-1,true,false,"","",null,null,null],[5,"created_at","pg_catalog","timestamptz",-1,true,false,"","",null,null,"now()"],[6,"last_used_at","pg_catalog","timestamptz",-1,false,false,"","",null,null,null],[7,"org_id","pg_catalog","uuid",-1,false,false,"","",null,null,null]],"indexes":[["auth_webauthn_credentials_credential_id_key",true,true,true,true,true,false,false,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_credential_id_key ON public.auth_webauthn_credentials USING btree (credential_id)"],["auth_webauthn_credentials_pkey",true,true,true,true,true,false,true,false,"CREATE UNIQUE INDEX auth_webauthn_credentials_pkey ON public.auth_webauthn_credentials USING btree (id)"],["idx_auth_webauthn_credentials_user",true,true,true,true,false,false,false,false,"CREATE INDEX idx_auth_webauthn_credentials_user ON public.auth_webauthn_credentials USING btree (user_id, created_at DESC)"]],"policies":1,"relation":["r","p",true,true,false,"d",null],"triggers":[["trg_auth_webauthn_credentials_org_immutable",false,"O",19,0,"",false,false,"public","enforce_org_id_immutable",null,null,"CREATE TRIGGER trg_auth_webauthn_credentials_org_immutable BEFORE UPDATE ON public.auth_webauthn_credentials FOR EACH ROW EXECUTE FUNCTION public.enforce_org_id_immutable()"],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","accounts",null],[null,true,"O",17,0,"",false,false,"pg_catalog","RI_FKey_check_upd","public","organizations",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","accounts",null],[null,true,"O",5,0,"",false,false,"pg_catalog","RI_FKey_check_ins","public","organizations",null]],"constraints":[["auth_webauthn_credentials_account_v1","f",true,false,false,true,true,0,true,[2],[1],"r","r","s","public","accounts","FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT"],["auth_webauthn_credentials_created_at_not_null","n",true,false,false,false,true,0,true,[5],null," "," "," ",null,null,"NOT NULL created_at"],["auth_webauthn_credentials_credential_id_key","u",true,false,false,true,true,0,true,[3],null," "," "," ",null,null,"UNIQUE (credential_id)"],["auth_webauthn_credentials_credential_id_not_null","n",true,false,false,false,true,0,true,[3],null," "," "," ",null,null,"NOT NULL credential_id"],["auth_webauthn_credentials_id_not_null","n",true,false,false,false,true,0,true,[1],null," "," "," ",null,null,"NOT NULL id"],["auth_webauthn_credentials_org_fk","f",true,false,false,true,true,0,true,[7],[1],"a","r","s","public","organizations","FOREIGN KEY (org_id) REFERENCES public.organizations(id) ON DELETE RESTRICT"],["auth_webauthn_credentials_passkey_json_not_null","n",true,false,false,false,true,0,true,[4],null," "," "," ",null,null,"NOT NULL passkey_json"],["auth_webauthn_credentials_pkey","p",true,false,false,true,true,0,true,[1],null," "," "," ",null,null,"PRIMARY KEY (id)"],["auth_webauthn_credentials_user_id_not_null","n",true,false,false,false,true,0,true,[2],null," "," "," ",null,null,"NOT NULL user_id"]],"inheritance":0},"acl":[["console_credential_owner","console_auth_rt","DELETE",false],["console_credential_owner","console_auth_rt","INSERT",false],["console_credential_owner","console_auth_rt","SELECT",false],["console_credential_owner","console_auth_rt","UPDATE",false],["console_credential_owner","console_credential_owner","DELETE",false],["console_credential_owner","console_credential_owner","SELECT",false]],"column_security":[{"acl":null,"name":"id","number":1,"acl_is_null":true},{"acl":null,"name":"user_id","number":2,"acl_is_null":true},{"acl":null,"name":"credential_id","number":3,"acl_is_null":true},{"acl":null,"name":"passkey_json","number":4,"acl_is_null":true},{"acl":null,"name":"created_at","number":5,"acl_is_null":true},{"acl":null,"name":"last_used_at","number":6,"acl_is_null":true},{"acl":null,"name":"org_id","number":7,"acl_is_null":true}],"policies":[{"name":"auth_credential_custody_v1","check":"true","roles":["console_auth_rt","console_credential_owner"],"using":"true","command":"*","permissive":true}]}]'::jsonb AND s.routines='[{"name":"auth_legacy_audit_append_v1","identity_arguments":"p_id uuid, p_actor uuid, p_action text, p_target_type text, p_target_id text, p_branch_id uuid, p_before_snap jsonb, p_after_snap jsonb, p_trace_id character, p_span_id character, p_occurred_at timestamp with time zone, p_org_id uuid, p_ip text, p_user_agent text, p_auth_method text, p_device text, p_classification_badges text[], p_anomaly boolean, p_reason text","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_id","p_actor","p_action","p_target_type","p_target_id","p_branch_id","p_before_snap","p_after_snap","p_trace_id","p_span_id","p_occurred_at","p_org_id","p_ip","p_user_agent","p_auth_method","p_device","p_classification_badges","p_anomaly","p_reason"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"f4f6716bcafffcdcd1cb094a3d2b0fd538984851de426d73eb5b0e4c2a8e3d83","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false]]},{"name":"auth_legacy_bootstrap_issue_v1","identity_arguments":"p_company uuid, p_subject uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"text","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","p_id","p_token_hash","p_issued_at","p_expires_at"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"400055092e8e5ed2151682a44cb20f55c667b6bfa8d1f5f5e6d5852d25668555","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_bootstrap_seed_v1","identity_arguments":"p_subject uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_subject","p_id","p_token_hash","p_issued_at","p_expires_at"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"dae7ca9e08dcceb6148b560ecc21fc9c3ad66bc8ec7b700137b8e68c8a9ca523","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_cold_start_admin_v1","identity_arguments":"","result":"uuid","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":null,"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"93fd18fda5db1d5f1674a249a1891cf67e3c2b62a3afbe93087a772c86e81667","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_company_lock_v1","identity_arguments":"p_company uuid","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"4bc29fb7d1029e49eadb217b81bb1fc44235603178079e1b8e5010ae0159a8fd","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"auth_legacy_deactivate_credentials_v1","identity_arguments":"p_company uuid, p_subject uuid, p_occurred_at timestamp with time zone","result":"TABLE(revoked_credentials bigint, revoked_families bigint)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","p_occurred_at","revoked_credentials","revoked_families"],"argmodes":["i","i","i","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"6daa76dec864b9cf83ebd98a7124a49eb9a541ed2c4aba0a9844e70cb91d22bc","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_group_passkey_flag_v1","identity_arguments":"p_subject uuid","result":"boolean","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_subject"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"3a8e4eecf8f48f4bcde9cbfd26c838c586a102bf870da95659070dfa9a228988","acl":[["console_credential_owner","console_app","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_purge_company_v1","identity_arguments":"p_company uuid","result":"void","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"3219296a90b6a8e48c08ed72d16a0cd66739d0b187bc4edccacb682612d61ae3","acl":[["console_credential_owner","console_app","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_purge_subjects_v1","identity_arguments":"p_company uuid","result":"uuid[]","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"afc2a82b13a3ac6198d2a8aafef3ea9ea1acc8372a32b72ea866af1401bd375f","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_reset_credentials_v1","identity_arguments":"p_company uuid, p_subject uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"TABLE(deleted_key_id uuid, deleted_credential_id text)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","p_id","p_token_hash","p_issued_at","p_expires_at","deleted_key_id","deleted_credential_id"],"argmodes":["i","i","i","i","i","i","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"f29a2bc7d7cad98b496f0659129648d8306c09b90e0dec7a62acd10aa461cc79","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_bootstrap_replace_v1","identity_arguments":"p_company uuid, p_actor uuid, p_id uuid, p_token_hash bytea, p_issued_at timestamp with time zone, p_expires_at timestamp with time zone","result":"void","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","p_id","p_token_hash","p_issued_at","p_expires_at"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"e022b62f445985d8801137e7eebb7c04746f3247da85458dc9339531c4123de5","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkey_count_v1","identity_arguments":"p_company uuid, p_actor uuid","result":"bigint","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"1f9d74da4666ae6120a4d0710db4a65694138978e10aca0acf3cb8c79c486776","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkey_delete_v1","identity_arguments":"p_company uuid, p_actor uuid, p_key_id uuid","result":"TABLE(outcome text, credential_id text)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","p_key_id","outcome","credential_id"],"argmodes":["i","i","i","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"4570434174614ee74a75f88ef9193184cfcb018956f7b8db8216fe30fc65ed10","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkey_state_v1","identity_arguments":"p_company uuid, p_actor uuid, p_key_id uuid","result":"TABLE(last_used_at timestamp with time zone)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","p_key_id","last_used_at"],"argmodes":["i","i","i","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"4238746c6b34f3028f9fae8da99f647218c0d28f3382f76905d8f626835a5476","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_self_passkeys_v1","identity_arguments":"p_company uuid, p_actor uuid","result":"TABLE(id uuid, created_at timestamp with time zone, last_used_at timestamp with time zone)","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_actor","id","created_at","last_used_at"],"argmodes":["i","i","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"76d95d0ea27f02c9ea77627345eb3ff11f2ee40ad32efbc4018ee50a1fe1eb93","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"auth_legacy_session_context_v1","identity_arguments":"p_company uuid, p_subject uuid","result":"TABLE(display_name text, username text, roles text[], branches uuid[], group_roles text[], authz_subject_version bigint, authz_policy_version bigint, session_generation bigint)","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject","display_name","username","roles","branches","group_roles","authz_subject_version","authz_policy_version","session_generation"],"argmodes":["i","i","t","t","t","t","t","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"92f7dfc3df3ad17f6d3f9fbbe8065c4722fac414efb9b1db738909c8652ffbeb","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false]]},{"name":"auth_legacy_user_active_v1","identity_arguments":"p_company uuid, p_subject uuid","result":"boolean","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"6493b7a81d16126abdc41eed831fb61057864e09e373cb0d44eed56655eb7e46","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_auth_rt","EXECUTE",false],["console_app","console_credential_owner","EXECUTE",false]]},{"name":"auth_legacy_user_has_passkey_v1","identity_arguments":"p_company uuid, p_subject uuid","result":"boolean","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_company","p_subject"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"90a86775826a7e5012204c43fde83d08cc20020a0f1181369afe2855e8ea64ab","acl":[["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"enforce_org_id_immutable","identity_arguments":"","result":"trigger","owner":"console_app","language":"plpgsql","kind":"f","security_definer":false,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":null,"argnames":null,"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"a46ee31ccf7fae026153b8bea7419777f055a4028e9ecfe0632e262a6f41f3ba","acl":[["console_app","PUBLIC","EXECUTE",false],["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_direct_org_children","identity_arguments":"p_id uuid","result":"void","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"77fde115be5e5de1f1e75f84e86b59cf82496d5dd063cd3b9fe9f6f7ceff6b95","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_force_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"ef0d500cef7ee5d283e6d471587bb96bbe1e0d6aa391524cda9acdd4277d4797","acl":[["console_app","console_app","EXECUTE",false]]},{"name":"platform_list_group_accounts","identity_arguments":"p_group_id uuid","result":"TABLE(user_id uuid, display_name text, phone text, tenant_roles text[], is_active boolean, has_passkey boolean, account_status text, org_id uuid, org_slug text, org_name text, group_roles text[], created_at timestamp with time zone)","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":true,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_group_id","user_id","display_name","phone","tenant_roles","is_active","has_passkey","account_status","org_id","org_slug","org_name","group_roles","created_at"],"argmodes":["i","t","t","t","t","t","t","t","t","t","t","t","t"],"argdefaults":null,"binary":null,"cost":100,"rows":1000,"source_sha256":"27f6f2f8fc4f8326bb5274fc1faf03d682cfe5be72000ca6e5653625087ba88b","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_remove_organization","identity_arguments":"p_id uuid","result":"text","owner":"console_app","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"v","parallel":"u","support":null,"config":["search_path=public, pg_temp"],"argnames":["p_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"57c4a8b2ad1fe05a9a620d0d45dfaeaaaac24f006c9f6a97937ead5fb48db21a","acl":[["console_app","console_app","EXECUTE",false],["console_app","console_rt","EXECUTE",false]]},{"name":"platform_resolve_bootstrap_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"c6b61cbb4aae4f8fb1daee70995be892020e5b906c46f0f327a30404ff87e7e7","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"platform_resolve_credential_org","identity_arguments":"p_credential_id text","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_credential_id"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"58b08df5e3f2a7b171ec80098832bd394d16b3d9f4a0a286639e7f1417808312","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]},{"name":"platform_resolve_token_org","identity_arguments":"p_token_hash bytea","result":"uuid","owner":"console_credential_owner","language":"plpgsql","kind":"f","security_definer":true,"strict":false,"returns_set":false,"leakproof":false,"volatility":"s","parallel":"u","support":null,"config":["search_path=pg_catalog, pg_temp","row_security=on"],"argnames":["p_token_hash"],"argmodes":null,"argdefaults":null,"binary":null,"cost":100,"rows":0,"source_sha256":"300cbab098531971f5832bba377e0c9d1e14bb94dd4035a0c2e86c3d7d1b5bfb","acl":[["console_credential_owner","console_auth_rt","EXECUTE",false],["console_credential_owner","console_credential_owner","EXECUTE",false],["console_credential_owner","console_rt","EXECUTE",false]]}]'::jsonb
   AND (SELECT count(*)=1 AND bool_and(valid) FROM credential_role)
   AND NOT EXISTS(SELECT 1 FROM routine_records WHERE NOT extra_valid)
   THEN 'account_credentials.finalized'
 ELSE CASE WHEN EXISTS(SELECT 1 FROM relations WHERE relowner=(SELECT oid FROM credential_role)) THEN 'account_credentials.profile_mismatch' ELSE 'account_credentials.legacy_profile_mismatch' END
 END FROM snapshots s;
"""
CREDENTIAL_INSTALL = r"""ALTER TABLE public.auth_bootstrap_credentials DROP CONSTRAINT auth_bootstrap_credentials_user_id_fkey;
ALTER TABLE public.auth_bootstrap_credentials DROP CONSTRAINT auth_bootstrap_credentials_user_same_org_fk;
ALTER TABLE public.auth_bootstrap_credentials ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_bootstrap_credentials;
ALTER TABLE public.auth_bootstrap_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_bootstrap_credentials FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_bootstrap_credentials TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_bootstrap_credentials OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_bootstrap_credentials FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,INSERT,UPDATE,DELETE ON public.auth_bootstrap_credentials TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_bootstrap_credentials TO console_auth_rt;
ALTER TABLE public.auth_device_login_handoffs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_device_login_handoffs FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_device_login_handoffs TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_device_login_handoffs OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_device_login_handoffs FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT ON public.auth_device_login_handoffs TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_device_login_handoffs TO console_auth_rt;
ALTER TABLE public.auth_refresh_token_families DROP CONSTRAINT auth_refresh_token_families_user_id_fkey;
ALTER TABLE public.auth_refresh_token_families DROP CONSTRAINT auth_refresh_token_families_user_same_org_fk;
ALTER TABLE public.auth_refresh_token_families ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_refresh_token_families;
ALTER TABLE public.auth_refresh_token_families ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_refresh_token_families FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_refresh_token_families TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_refresh_token_families OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_refresh_token_families FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,UPDATE,DELETE ON public.auth_refresh_token_families TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_refresh_token_families TO console_auth_rt;
ALTER TABLE public.auth_refresh_tokens DROP CONSTRAINT auth_refresh_tokens_user_id_fkey;
ALTER TABLE public.auth_refresh_tokens DROP CONSTRAINT auth_refresh_tokens_user_same_org_fk;
ALTER TABLE public.auth_refresh_tokens ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_refresh_tokens;
ALTER TABLE public.auth_refresh_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_refresh_tokens FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_refresh_tokens TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_refresh_tokens OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_refresh_tokens FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,UPDATE,DELETE ON public.auth_refresh_tokens TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_refresh_tokens TO console_auth_rt;
ALTER TABLE public.auth_webauthn_ceremonies DROP CONSTRAINT auth_webauthn_ceremonies_user_id_fkey;
ALTER TABLE public.auth_webauthn_ceremonies ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_webauthn_ceremonies FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_webauthn_ceremonies TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_webauthn_ceremonies OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_webauthn_ceremonies FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,UPDATE,DELETE ON public.auth_webauthn_ceremonies TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE ON public.auth_webauthn_ceremonies TO console_auth_rt;
ALTER TABLE public.auth_webauthn_ceremony_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_webauthn_ceremony_bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_webauthn_ceremony_bindings TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_webauthn_ceremony_bindings OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_webauthn_ceremony_bindings FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,DELETE ON public.auth_webauthn_ceremony_bindings TO console_credential_owner;
GRANT SELECT,INSERT ON public.auth_webauthn_ceremony_bindings TO console_auth_rt;
ALTER TABLE public.auth_webauthn_credentials DROP CONSTRAINT auth_webauthn_credentials_user_id_fkey;
ALTER TABLE public.auth_webauthn_credentials DROP CONSTRAINT auth_webauthn_credentials_user_same_org_fk;
ALTER TABLE public.auth_webauthn_credentials ALTER COLUMN org_id DROP NOT NULL;
DROP POLICY org_isolation ON public.auth_webauthn_credentials;
ALTER TABLE public.auth_webauthn_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.auth_webauthn_credentials FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_credential_custody_v1 ON public.auth_webauthn_credentials TO console_auth_rt,console_credential_owner USING (true) WITH CHECK (true);
ALTER TABLE public.auth_webauthn_credentials OWNER TO console_credential_owner;
REVOKE ALL ON public.auth_webauthn_credentials FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT SELECT,DELETE ON public.auth_webauthn_credentials TO console_credential_owner;
GRANT SELECT,INSERT,UPDATE,DELETE ON public.auth_webauthn_credentials TO console_auth_rt;
ALTER TABLE public.auth_bootstrap_credentials ADD CONSTRAINT auth_bootstrap_credentials_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_refresh_token_families ADD CONSTRAINT auth_refresh_token_families_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_refresh_tokens ADD CONSTRAINT auth_refresh_tokens_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_webauthn_ceremonies ADD CONSTRAINT auth_webauthn_ceremonies_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_webauthn_credentials ADD CONSTRAINT auth_webauthn_credentials_account_v1 FOREIGN KEY (user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_target_account_v1 FOREIGN KEY (target_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_approved_account_v1 FOREIGN KEY (approved_user_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_refresh_token_families ADD CONSTRAINT auth_refresh_token_families_id_user_v1 UNIQUE(id,user_id);
ALTER TABLE public.auth_refresh_tokens DROP CONSTRAINT auth_refresh_tokens_family_id_fkey;
ALTER TABLE public.auth_refresh_tokens ADD CONSTRAINT auth_refresh_tokens_family_subject_v1 FOREIGN KEY (family_id, user_id) REFERENCES public.auth_refresh_token_families(id, user_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_bootstrap_credentials DROP CONSTRAINT auth_bootstrap_credentials_registration_ceremony_id_fkey;
ALTER TABLE public.auth_bootstrap_credentials ADD CONSTRAINT auth_bootstrap_credentials_registration_ceremony_id_fkey FOREIGN KEY (registration_ceremony_id) REFERENCES public.auth_webauthn_ceremonies(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.auth_device_login_handoffs DROP CONSTRAINT auth_device_login_handoffs_check1;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_check1 CHECK (((target_org_id IS NULL) OR (target_user_id IS NOT NULL)));
ALTER TABLE public.auth_device_login_handoffs DROP CONSTRAINT auth_device_login_handoffs_check2;
ALTER TABLE public.auth_device_login_handoffs ADD CONSTRAINT auth_device_login_handoffs_check2 CHECK ((((approved_at IS NULL) AND (approved_user_id IS NULL) AND (approved_org_id IS NULL)) OR ((approved_at IS NOT NULL) AND (approved_user_id IS NOT NULL))));
CREATE OR REPLACE FUNCTION public.auth_legacy_company_lock_v1(p_company uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF pg_catalog.current_setting('transaction_isolation') <> 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.unsupported_isolation';
    END IF;
    PERFORM o.id FROM public.organizations o WHERE o.id=p_company FOR KEY SHARE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.company_not_found';
    END IF;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_company_lock_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_company_lock_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_company_lock_v1(uuid) TO console_app,console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_user_active_v1(p_company uuid,p_subject uuid) RETURNS boolean
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE active boolean;
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF p_subject IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    SELECT u.is_active INTO active FROM public.users u WHERE u.id=p_subject AND u.org_id=p_company;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.subject_not_found';
    END IF;
    IF active IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    RETURN active;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_user_active_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_user_active_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_user_active_v1(uuid,uuid) TO console_app,console_auth_rt,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkeys_v1(p_company uuid,p_actor uuid) RETURNS TABLE(id uuid,created_at timestamptz,last_used_at timestamptz)
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_actor);
    SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN QUERY SELECT k.id,k.created_at,k.last_used_at
        FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company ORDER BY k.created_at;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkeys_v1(uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_state_v1(p_company uuid,p_actor uuid,p_key_id uuid) RETURNS TABLE(last_used_at timestamptz)
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_actor);
    SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN QUERY SELECT k.last_used_at FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_count_v1(p_company uuid,p_actor uuid) RETURNS bigint
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_actor);
    SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN (SELECT count(*) FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_count_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_count_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_count_v1(uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_user_has_passkey_v1(p_company uuid,p_subject uuid) RETURNS boolean
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    PERFORM public.auth_legacy_user_active_v1(p_company,p_subject);
    SELECT public.account_legacy_fenced_v1(p_subject) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RETURN false;
    END IF;
    RETURN EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_subject AND k.org_id=p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_user_has_passkey_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_user_has_passkey_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_user_has_passkey_v1(uuid,uuid) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_group_passkey_flag_v1(p_subject uuid) RETURNS boolean
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE fenced boolean;
BEGIN
    IF p_subject IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    SELECT public.account_legacy_fenced_v1(p_subject) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN RETURN false; END IF;
    RETURN EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_group_passkey_flag_v1(uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_group_passkey_flag_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_group_passkey_flag_v1(uuid) TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_delete_v1(p_company uuid,p_actor uuid,p_key_id uuid) RETURNS TABLE(outcome text,credential_id text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean; victim text; total bigint;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_actor) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    SELECT k.credential_id INTO victim FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company FOR UPDATE;
    IF NOT FOUND THEN
        RETURN QUERY SELECT 'not_found'::text,NULL::text;
        RETURN;
    END IF;
    SELECT count(*) INTO total FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company;
    IF total <= 1 THEN
        RETURN QUERY SELECT 'last_key'::text,NULL::text;
        RETURN;
    END IF;
    DELETE FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company;
    RETURN QUERY SELECT 'deleted'::text,victim;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_bootstrap_issue_v1(p_company uuid,p_subject uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS text
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='expired'
        WHERE b.user_id=p_subject AND b.org_id=p_company AND b.consumed_at IS NULL
          AND b.revoked_at IS NULL AND b.expires_at<=p_issued_at;
    IF EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject AND k.org_id=p_company) THEN
        RETURN 'has_passkey';
    END IF;
    IF EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b WHERE b.user_id=p_subject
        AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL) THEN
        RETURN 'open_exists';
    END IF;
    INSERT INTO public.auth_bootstrap_credentials(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_subject,p_token_hash,p_issued_at,p_expires_at,p_company);
    RETURN 'issued';
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_reset_credentials_v1(p_company uuid,p_subject uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS TABLE(deleted_key_id uuid,deleted_credential_id text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    RETURN QUERY DELETE FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_subject AND k.org_id=p_company RETURNING k.id,k.credential_id;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='expired'
        WHERE b.user_id=p_subject AND b.org_id=p_company AND b.consumed_at IS NULL
          AND b.revoked_at IS NULL AND b.expires_at<=p_issued_at;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='reset'
        WHERE b.user_id=p_subject AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL;
    INSERT INTO public.auth_bootstrap_credentials(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_subject,p_token_hash,p_issued_at,p_expires_at,p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_self_bootstrap_replace_v1(p_company uuid,p_actor uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_actor) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='expired'
        WHERE b.user_id=p_actor AND b.org_id=p_company AND b.consumed_at IS NULL
          AND b.revoked_at IS NULL AND b.expires_at<=p_issued_at;
    UPDATE public.auth_bootstrap_credentials b SET revoked_at=p_issued_at,revoked_reason='reset'
        WHERE b.user_id=p_actor AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL;
    INSERT INTO public.auth_bootstrap_credentials(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_actor,p_token_hash,p_issued_at,p_expires_at,p_company);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_deactivate_credentials_v1(p_company uuid,p_subject uuid,p_occurred_at timestamptz) RETURNS TABLE(revoked_credentials bigint,revoked_families bigint)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean; key_count bigint; family_count bigint;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_occurred_at IS NULL OR NOT isfinite(p_occurred_at) THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    DELETE FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject AND k.org_id=p_company;
    GET DIAGNOSTICS key_count = ROW_COUNT;
    UPDATE public.auth_refresh_token_families f SET revoked_at=p_occurred_at,revoked_reason='user_deactivated'
        WHERE f.user_id=p_subject AND f.org_id=p_company AND f.revoked_at IS NULL;
    GET DIAGNOSTICS family_count = ROW_COUNT;
    UPDATE public.auth_refresh_tokens t SET revoked_at=COALESCE(t.revoked_at,p_occurred_at)
        WHERE t.user_id=p_subject AND t.org_id=p_company;
    RETURN QUERY SELECT key_count,family_count;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_cold_start_admin_v1() RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    IF NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid
       IS DISTINCT FROM '00000000-0000-0000-0000-00000000face'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    RETURN (SELECT u.id FROM public.users u
        WHERE u.org_id='00000000-0000-0000-0000-00000000face'::uuid
          AND u.display_name='Cold Start Admin' AND u.roles @> ARRAY['SUPER_ADMIN']::text[]
        ORDER BY u.id LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_cold_start_admin_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_cold_start_admin_v1() FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_cold_start_admin_v1() TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_bootstrap_seed_v1(p_subject uuid,p_id uuid,p_token_hash bytea,p_issued_at timestamptz,p_expires_at timestamptz) RETURNS uuid
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE
    allowed boolean;
    p_company constant uuid := '00000000-0000-0000-0000-00000000face'::uuid;
    selected uuid;
    opened uuid;
BEGIN
    SELECT public.auth_legacy_cold_start_admin_v1() INTO selected;
    IF selected IS NULL OR selected IS DISTINCT FROM p_subject THEN RETURN NULL; END IF;
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_subject) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    IF p_id IS NULL OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
       OR p_issued_at IS NULL OR p_expires_at IS NULL
       OR NOT isfinite(p_issued_at) OR NOT isfinite(p_expires_at) OR p_expires_at<=p_issued_at THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy.invalid_input';
    END IF;
    SELECT public.auth_legacy_cold_start_admin_v1() INTO selected;
    IF selected IS NULL OR selected IS DISTINCT FROM p_subject THEN RETURN NULL; END IF;
    IF EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k WHERE k.user_id=p_subject AND k.org_id=p_company)
       OR EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b WHERE b.user_id=p_subject
         AND b.org_id=p_company AND b.consumed_at IS NULL AND b.revoked_at IS NULL AND b.expires_at>p_issued_at) THEN
        RETURN NULL;
    END IF;
    INSERT INTO public.auth_bootstrap_credentials AS b(id,user_id,token_hash,issued_at,expires_at,org_id)
        VALUES(p_id,p_subject,p_token_hash,p_issued_at,p_expires_at,p_company)
        ON CONFLICT(token_hash) DO UPDATE SET issued_at=EXCLUDED.issued_at,expires_at=EXCLUDED.expires_at,
            revoked_at=NULL,revoked_reason=NULL,consumed_at=NULL
        WHERE b.user_id=EXCLUDED.user_id AND b.consumed_at IS NULL
          AND (b.revoked_at IS NOT NULL OR b.expires_at<=EXCLUDED.issued_at)
        RETURNING b.id INTO opened;
    RETURN opened;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_session_context_v1(p_company uuid,p_subject uuid) RETURNS TABLE(display_name text,username text,roles text[],branches uuid[],group_roles text[],authz_subject_version bigint,authz_policy_version bigint,session_generation bigint)
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE subject public.users%ROWTYPE; fenced boolean;
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF p_subject IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    SELECT u.* INTO subject FROM public.users u WHERE u.id=p_subject AND u.org_id=p_company;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.subject_not_found';
    END IF;
    IF subject.is_active IS DISTINCT FROM true THEN
        RAISE EXCEPTION USING ERRCODE='28000', MESSAGE='auth_legacy.subject_inactive';
    END IF;
    IF subject.roles IS NULL OR cardinality(subject.roles)=0 THEN
        RAISE EXCEPTION USING ERRCODE='28000', MESSAGE='auth_legacy.subject_has_no_roles';
    END IF;
    SELECT public.account_legacy_fenced_v1(p_subject) INTO fenced;
    IF fenced IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF fenced THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    RETURN QUERY SELECT subject.display_name,COALESCE(subject.phone,p_subject::text),subject.roles,
        ARRAY(SELECT b.branch_id FROM public.user_branches b
            WHERE b.user_id=p_subject AND b.org_id=p_company ORDER BY b.branch_id),
        ARRAY(SELECT DISTINCT g.group_role FROM public.group_role_grants g
            WHERE g.user_id=p_subject ORDER BY g.group_role),
        GREATEST(COALESCE(v.version,0),0),
        GREATEST(COALESCE((SELECT pv.version FROM public.policy_versions pv WHERE pv.org_id=p_company),0),0),
        GREATEST(COALESCE(v.session_generation,0),0)
    FROM (SELECT 1) singleton LEFT JOIN public.subject_authz_versions v
        ON v.user_id=p_subject AND v.org_id=p_company;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) TO console_app,console_auth_rt;
CREATE OR REPLACE FUNCTION public.platform_resolve_bootstrap_org(p_token_hash bytea) RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    RETURN (SELECT x.org_id FROM public.auth_bootstrap_credentials x WHERE x.token_hash=p_token_hash LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.platform_resolve_bootstrap_org(bytea) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.platform_resolve_bootstrap_org(bytea) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.platform_resolve_bootstrap_org(bytea) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.platform_resolve_token_org(p_token_hash bytea) RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    RETURN (SELECT x.org_id FROM public.auth_refresh_tokens x WHERE x.token_hash=p_token_hash LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.platform_resolve_token_org(bytea) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.platform_resolve_token_org(bytea) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.platform_resolve_token_org(bytea) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.platform_resolve_credential_org(p_credential_id text) RETURNS uuid
LANGUAGE plpgsql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$BEGIN
    RETURN (SELECT x.org_id FROM public.auth_webauthn_credentials x WHERE x.credential_id=p_credential_id LIMIT 1);
END;$auth7_body$;
ALTER FUNCTION public.platform_resolve_credential_org(text) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.platform_resolve_credential_org(text) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.platform_resolve_credential_org(text) TO console_auth_rt,console_credential_owner,console_rt;
CREATE OR REPLACE FUNCTION public.auth_legacy_purge_subjects_v1(p_company uuid) RETURNS uuid[]
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE subjects uuid[];
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_company IS DISTINCT FROM NULLIF(pg_catalog.current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF pg_catalog.current_setting('transaction_isolation') <> 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.unsupported_isolation';
    END IF;
    PERFORM o.id FROM public.organizations o WHERE o.id=p_company FOR UPDATE;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.company_not_found';
    END IF;
    SELECT COALESCE(array_agg(locked.id ORDER BY locked.id),ARRAY[]::uuid[]) INTO subjects
        FROM (SELECT u.id FROM public.users u WHERE u.org_id=p_company ORDER BY u.id FOR UPDATE) locked;
    RETURN subjects;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_purge_subjects_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_purge_subjects_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_purge_subjects_v1(uuid) TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_purge_company_v1(p_company uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE
    subjects uuid[];
    subject uuid;
    allowed boolean;
    previous_company text := pg_catalog.current_setting('app.current_org',true);
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004', MESSAGE='auth_legacy.null_identity';
    END IF;
    PERFORM pg_catalog.set_config('app.current_org',p_company::text,true);
    SELECT public.auth_legacy_purge_subjects_v1(p_company) INTO subjects;
    FOREACH subject IN ARRAY subjects LOOP
        SELECT public.account_company_deactivation_guard_v1(p_company,subject) INTO allowed;
        IF allowed IS DISTINCT FROM true THEN
            RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy_purge.retained_custody';
        END IF;
    END LOOP;
    -- Account custody cannot be inferred from a retained Company provenance.
    -- Classify every Company-attributed subject before any credential deletion.
    IF EXISTS(SELECT 1 FROM (
        SELECT k.user_id FROM public.auth_webauthn_credentials k WHERE k.org_id=p_company
        UNION ALL SELECT f.user_id FROM public.auth_refresh_token_families f WHERE f.org_id=p_company
        UNION ALL SELECT t.user_id FROM public.auth_refresh_tokens t WHERE t.org_id=p_company
        UNION ALL SELECT b.user_id FROM public.auth_bootstrap_credentials b WHERE b.org_id=p_company
        UNION ALL SELECT h.target_user_id FROM public.auth_device_login_handoffs h WHERE h.target_org_id=p_company
        UNION ALL SELECT h.approved_user_id FROM public.auth_device_login_handoffs h WHERE h.approved_org_id=p_company
    ) attributed WHERE attributed.user_id IS NULL OR NOT (attributed.user_id=ANY(subjects))) THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy_purge.retained_custody';
    END IF;
    DELETE FROM public.auth_bootstrap_credentials b WHERE b.org_id=p_company;
    DELETE FROM public.auth_refresh_tokens t WHERE t.org_id=p_company;
    DELETE FROM public.auth_refresh_token_families f WHERE f.org_id=p_company;
    DELETE FROM public.auth_webauthn_credentials k WHERE k.org_id=p_company;
    DELETE FROM public.auth_webauthn_ceremonies c WHERE c.user_id=ANY(subjects);
    -- The original purge retained global handoff history; preserve it.
    PERFORM pg_catalog.set_config('app.current_org',COALESCE(previous_company,''),true);
EXCEPTION WHEN OTHERS THEN
    PERFORM pg_catalog.set_config('app.current_org',COALESCE(previous_company,''),true);
    RAISE;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_purge_company_v1(uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_purge_company_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_purge_company_v1(uuid) TO console_app,console_credential_owner;
CREATE OR REPLACE FUNCTION public.auth_legacy_audit_append_v1(p_id uuid,p_actor uuid,p_action text,p_target_type text,p_target_id text,p_branch_id uuid,p_before_snap jsonb,p_after_snap jsonb,p_trace_id char(32),p_span_id char(16),p_occurred_at timestamptz,p_org_id uuid,p_ip text,p_user_agent text,p_auth_method text,p_device text,p_classification_badges text[],p_anomaly boolean,p_reason text) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE
    keys text[];
    expected_keys text[];
    uuid_fields text[] := ARRAY[]::text[];
    bool_fields text[] := ARRAY[]::text[];
    time_fields text[] := ARRAY[]::text[];
    field text;
    target text;
    anonymous boolean := false;
    fenced boolean;
    value jsonb;
    parts bigint[];
    year_value bigint;
    ordinal_max bigint;
BEGIN
    IF p_id IS NULL OR p_action IS NULL OR p_target_type IS NULL OR p_target_id IS NULL
       OR p_before_snap IS NOT NULL OR jsonb_typeof(p_after_snap) IS DISTINCT FROM 'object'
       OR p_occurred_at IS NULL OR NOT isfinite(p_occurred_at)
       OR p_trace_id IS NULL OR p_trace_id::text !~ '^[0-9a-f]{32}$'
       OR p_span_id IS NULL OR p_span_id::text !~ '^[0-9a-f]{16}$'
       OR p_branch_id IS NOT NULL OR p_ip IS NOT NULL OR p_user_agent IS NOT NULL
       OR p_auth_method IS NOT NULL OR p_device IS NOT NULL OR p_classification_badges IS NOT NULL
       OR p_anomaly IS NOT NULL OR p_reason IS NOT NULL THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
    END IF;
    CASE p_action
    WHEN 'auth.passkey.register' THEN
        target := 'auth_webauthn_credential'; expected_keys := ARRAY['credential_id','user_id']; uuid_fields := ARRAY['user_id'];
        IF jsonb_typeof(p_after_snap->'credential_id') IS DISTINCT FROM 'string' OR p_after_snap->>'credential_id'='' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.refresh.issue' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','token_id','user_id','expires_at'];
        uuid_fields := ARRAY['family_id','token_id','user_id']; time_fields := ARRAY['expires_at'];
    WHEN 'auth.refresh' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','used_token_id','replacement_token_id','expires_at'];
        uuid_fields := ARRAY['family_id','used_token_id','replacement_token_id']; time_fields := ARRAY['expires_at'];
    WHEN 'auth.refresh.absolute_ttl_revoked' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','revoked_reason','family_created_at'];
        uuid_fields := ARRAY['family_id']; time_fields := ARRAY['family_created_at'];
        IF p_after_snap->'revoked_reason' IS DISTINCT FROM '"absolute_ttl_exceeded"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.refresh.reuse_detected' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','revoked_reason','reused_token_id'];
        uuid_fields := ARRAY['family_id','reused_token_id'];
        IF p_after_snap->'revoked_reason' IS DISTINCT FROM '"reuse_detected"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.logout' THEN
        target := 'auth_refresh_token_family'; expected_keys := ARRAY['family_id','revoked_reason']; uuid_fields := ARRAY['family_id'];
        IF p_after_snap->'revoked_reason' IS DISTINCT FROM '"logout"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.otp.redeem' THEN
        target := 'auth_bootstrap_credential'; expected_keys := ARRAY['user_id','requires_passkey_setup'];
        uuid_fields := ARRAY['user_id']; bool_fields := ARRAY['requires_passkey_setup'];
    WHEN 'auth.otp.consume' THEN
        target := 'auth_bootstrap_credential'; expected_keys := ARRAY['user_id']; uuid_fields := ARRAY['user_id'];
    WHEN 'auth.passkey.enroll_handoff_issued' THEN
        target := 'auth_bootstrap_credential'; expected_keys := ARRAY['user_id','expires_at','purpose'];
        uuid_fields := ARRAY['user_id']; time_fields := ARRAY['expires_at'];
        IF p_after_snap->'purpose' IS DISTINCT FROM '"passkey_enrollment_handoff"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.login' THEN
        target := 'users'; expected_keys := ARRAY['passkey_id','refresh_family_id']; uuid_fields := expected_keys;
    WHEN 'auth.otp.signin' THEN
        target := 'users'; expected_keys := ARRAY['refresh_family_id','requires_passkey_setup'];
        uuid_fields := ARRAY['refresh_family_id']; bool_fields := ARRAY['requires_passkey_setup'];
    WHEN 'auth.device_login.approve' THEN
        target := 'users'; expected_keys := ARRAY['handoff_id','passkey_id']; uuid_fields := expected_keys;
    WHEN 'auth.device_login.approve_session' THEN
        target := 'users'; expected_keys := ARRAY['handoff_id','passkey_id']; uuid_fields := ARRAY['handoff_id'];
        IF p_after_snap->'passkey_id' IS DISTINCT FROM 'null'::jsonb THEN uuid_fields := array_append(uuid_fields,'passkey_id'); END IF;
    WHEN 'auth.device_login.consume' THEN
        target := 'users'; expected_keys := ARRAY['handoff_id','passkey_id','refresh_family_id']; uuid_fields := ARRAY['handoff_id','refresh_family_id'];
        IF p_after_snap->'passkey_id' IS DISTINCT FROM 'null'::jsonb THEN uuid_fields := array_append(uuid_fields,'passkey_id'); END IF;
    WHEN 'auth.otp.redeem_failed' THEN
        anonymous := true; target := 'auth_bootstrap_credential'; expected_keys := ARRAY['outcome'];
        IF p_after_snap->'outcome' IS DISTINCT FROM '"rejected"'::jsonb THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    WHEN 'auth.device_login.start' THEN
        anonymous := true; target := 'auth_bootstrap_credential'; expected_keys := ARRAY['handoff_id','expires_at'];
        uuid_fields := ARRAY['handoff_id']; time_fields := ARRAY['expires_at'];
    ELSE
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
    END CASE;
    SELECT array_agg(k ORDER BY k) INTO keys FROM jsonb_object_keys(p_after_snap) k;
    IF keys IS DISTINCT FROM ARRAY(SELECT k FROM unnest(expected_keys) k ORDER BY k)
       OR p_target_type IS DISTINCT FROM target THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
    END IF;
    FOREACH field IN ARRAY uuid_fields LOOP
        IF jsonb_typeof(p_after_snap->field) IS DISTINCT FROM 'string'
           OR (p_after_snap->>field) !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END LOOP;
    FOREACH field IN ARRAY bool_fields LOOP
        IF jsonb_typeof(p_after_snap->field) IS DISTINCT FROM 'boolean' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END LOOP;
    FOREACH field IN ARRAY time_fields LOOP
        value := p_after_snap->field;
        IF jsonb_typeof(value) IS DISTINCT FROM 'array' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
        IF jsonb_array_length(value)<>9 OR EXISTS(SELECT 1 FROM jsonb_array_elements(value) part
            WHERE jsonb_typeof(part)<>'number' OR part::text !~ '^-?[0-9]{1,10}$') THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
        SELECT array_agg(part::text::bigint ORDER BY ordinal) INTO parts
            FROM jsonb_array_elements(value) WITH ORDINALITY elements(part,ordinal);
        year_value := parts[1];
        ordinal_max := CASE WHEN year_value%4=0 AND (year_value%100<>0 OR year_value%400=0) THEN 366 ELSE 365 END;
        IF year_value NOT BETWEEN -9999 AND 9999 OR parts[2] NOT BETWEEN 1 AND ordinal_max
           OR parts[3] NOT BETWEEN 0 AND 23 OR parts[4] NOT BETWEEN 0 AND 59 OR parts[5] NOT BETWEEN 0 AND 59
           OR parts[6] NOT BETWEEN 0 AND 999999999 OR parts[7] NOT BETWEEN -23 AND 23
           OR parts[8] NOT BETWEEN -59 AND 59 OR parts[9] NOT BETWEEN -59 AND 59
           OR parts[7]*parts[8]<0 OR parts[7]*parts[9]<0 OR parts[8]*parts[9]<0 THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END LOOP;
    IF anonymous THEN
        IF p_actor IS NOT NULL OR p_org_id IS NOT NULL OR p_target_id<>'redeem' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    ELSE
        IF p_actor IS NULL OR p_org_id IS NULL
           OR p_target_id !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
           OR (target='users' AND p_target_id IS DISTINCT FROM p_actor::text)
           OR (p_after_snap ? 'user_id' AND p_after_snap->>'user_id' IS DISTINCT FROM p_actor::text)
           OR (p_after_snap ? 'family_id' AND p_after_snap->>'family_id' IS DISTINCT FROM p_target_id) THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
        PERFORM public.auth_legacy_user_active_v1(p_org_id,p_actor);
        SELECT public.account_legacy_fenced_v1(p_actor) INTO fenced;
        IF fenced IS DISTINCT FROM false THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='auth_legacy_audit.invalid_event';
        END IF;
    END IF;
    INSERT INTO public.audit_events(id,actor,action,target_type,target_id,branch_id,before_snap,after_snap,
        trace_id,span_id,occurred_at,org_id,ip,user_agent,auth_method,device,classification_badges,anomaly,reason)
    VALUES(p_id,p_actor,p_action,p_target_type,p_target_id,p_branch_id,p_before_snap,p_after_snap,
        p_trace_id,p_span_id,p_occurred_at,p_org_id,p_ip,p_user_agent,p_auth_method,p_device,p_classification_badges,p_anomaly,p_reason);
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text) TO console_app,console_auth_rt;
CREATE OR REPLACE FUNCTION public.platform_remove_organization(p_id uuid)
 RETURNS text
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
DECLARE
    sentinel_org CONSTANT UUID := '00000000-0000-0000-0000-00000000face'::uuid;
    org_exists   BOOLEAN;
    has_data     BOOLEAN;
BEGIN
    IF p_id = sentinel_org THEN
        RETURN 'not_found';
    END IF;

    SET LOCAL row_security = off;

    SELECT EXISTS (SELECT 1 FROM organizations WHERE id = p_id) INTO org_exists;
    IF NOT org_exists THEN
        SET LOCAL row_security = on;
        RETURN 'not_found';
    END IF;

    SELECT
        EXISTS (SELECT 1 FROM registry_equipment           WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM work_orders                   WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM registry_sites               WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM registry_customers           WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM inspection_rounds            WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM regular_inspection_schedules WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM sales_listings               WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM customer_inquiries           WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_rental_quotes      WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_purchase_requests  WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_purchase_attachments WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_regular_purchase_prices WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM financial_expense_ledger      WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM equipment_cost_ledger        WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM messenger_threads            WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM location_consents            WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM site_attendance_events       WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM governance_findings          WHERE org_id = p_id)
     OR EXISTS (SELECT 1 FROM registered_devices rd
                JOIN users u ON u.id = rd.user_id WHERE u.org_id = p_id)
    INTO has_data;

    IF has_data THEN
        SET LOCAL row_security = on;
        RETURN 'blocked_has_data';
    END IF;

    PERFORM public.auth_legacy_purge_company_v1(p_id);

    DELETE FROM user_branches WHERE org_id = p_id;

    PERFORM set_config('app.audit_rehome', 'on', true);
    UPDATE audit_events
    SET org_id    = sentinel_org,
        actor     = NULL,
        branch_id = NULL
    WHERE org_id = p_id;
    PERFORM set_config('app.audit_rehome', 'off', true);

    DELETE FROM users    WHERE org_id = p_id;
    DELETE FROM branches WHERE org_id = p_id;
    DELETE FROM regions  WHERE org_id = p_id;

    DELETE FROM organizations WHERE id = p_id;

    SET LOCAL row_security = on;
    RETURN 'removed';
END;
$function$;
CREATE OR REPLACE FUNCTION public.platform_force_remove_organization(p_id uuid)
 RETURNS text
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
DECLARE
    sentinel_org CONSTANT UUID := '00000000-0000-0000-0000-00000000face'::uuid;
    org_status   TEXT;
BEGIN
    IF p_id = sentinel_org THEN
        RETURN 'not_found';
    END IF;

    SET LOCAL row_security = off;
    PERFORM set_config('app.maintenance_force_remove', 'on', true);

    SELECT status INTO org_status FROM organizations WHERE id = p_id;
    IF NOT FOUND THEN
        SET LOCAL row_security = on;
        RETURN 'not_found';
    END IF;

    IF org_status <> 'ARCHIVED' THEN
        SET LOCAL row_security = on;
        RETURN 'blocked_active';
    END IF;

    PERFORM set_config('app.platform_force_remove_org', 'on', true);
    -- Close direct restrictive tenant edges before deleting employee/user roots.
    -- The catalog query excludes only the explicitly ordered roots below.
    PERFORM platform_force_remove_direct_org_children(p_id);
    PERFORM public.auth_legacy_purge_company_v1(p_id);
    DELETE FROM attendance_direct_import_events  WHERE org_id = p_id;
    DELETE FROM data_import_rows                 WHERE org_id = p_id;
    DELETE FROM data_import_runs                 WHERE org_id = p_id;
    DELETE FROM payroll_draft_lines             WHERE org_id = p_id;
    DELETE FROM annual_leave_obligations        WHERE org_id = p_id;
    DELETE FROM payroll_draft_runs              WHERE org_id = p_id;
    DELETE FROM employee_lifecycle_events       WHERE org_id = p_id;
    UPDATE users SET employee_id = NULL         WHERE org_id = p_id;
    DELETE FROM employees                       WHERE org_id = p_id;
    PERFORM set_config('app.platform_force_remove_org', 'off', true);


    DELETE FROM comms_send_rate                 WHERE org_id = p_id;

    DELETE FROM customer_inquiries              WHERE org_id = p_id;
    DELETE FROM daily_work_plan_items           WHERE org_id = p_id;
    DELETE FROM daily_work_plans                WHERE org_id = p_id;

    DELETE FROM email_attachments               WHERE org_id = p_id;
    DELETE FROM email_messages                  WHERE org_id = p_id;
    DELETE FROM email_threads                   WHERE org_id = p_id;
    DELETE FROM email_folders                   WHERE org_id = p_id;
    DELETE FROM email_accounts                  WHERE org_id = p_id;
    DELETE FROM mailbox_deliveries             WHERE org_id = p_id;
    DELETE FROM mailbox_messages               WHERE org_id = p_id;
    DELETE FROM mailbox_aliases                WHERE org_id = p_id;
    DELETE FROM mailboxes                      WHERE org_id = p_id;
    DELETE FROM mailbox_domains                WHERE org_id = p_id;

    DELETE FROM equipment_maintenance_history_costs WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history_evidence WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history WHERE org_id = p_id;
    DELETE FROM equipment_cost_ledger           WHERE org_id = p_id;
    DELETE FROM equipment_substitutions         WHERE org_id = p_id;
    DELETE FROM excel_export_logs               WHERE org_id = p_id;

    DELETE FROM user_feature_preferences        WHERE org_id = p_id;

    DELETE FROM financial_regular_purchase_prices WHERE org_id = p_id;
    DELETE FROM financial_expense_ledger          WHERE org_id = p_id;
    DELETE FROM financial_purchase_attachments    WHERE org_id = p_id;
    DELETE FROM financial_purchase_request_lines  WHERE org_id = p_id;

    DELETE FROM financial_purchase_history      WHERE org_id = p_id;
    DELETE FROM financial_purchase_requests     WHERE org_id = p_id;
    DELETE FROM financial_rental_quote_lines    WHERE org_id = p_id;
    DELETE FROM financial_rental_quotes         WHERE org_id = p_id;

    DELETE FROM governance_findings             WHERE org_id = p_id;

    PERFORM set_config('app.audit_rehome', 'on', true);
    UPDATE audit_events
    SET org_id    = sentinel_org,
        actor     = NULL,
        branch_id = NULL
    WHERE org_id = p_id;
    PERFORM set_config('app.audit_rehome', 'off', true);

    DELETE FROM inspection_rounds               WHERE org_id = p_id;
    DELETE FROM kpi_exclusions                  WHERE org_id = p_id;

    DELETE FROM location_collection_logs        WHERE org_id = p_id;
    DELETE FROM location_consent_ledger         WHERE org_id = p_id;
    DELETE FROM location_consents               WHERE org_id = p_id;
    DELETE FROM location_pings                  WHERE org_id = p_id;

    DELETE FROM messenger_message_attachments   WHERE org_id = p_id;
    DELETE FROM messenger_read_receipts         WHERE org_id = p_id;
    DELETE FROM messenger_messages              WHERE org_id = p_id;
    DELETE FROM messenger_thread_members        WHERE org_id = p_id;
    DELETE FROM messenger_threads               WHERE org_id = p_id;

    DELETE FROM evidence_media                  WHERE org_id = p_id;

    DELETE FROM offline_sync_requests           WHERE org_id = p_id;

    DELETE FROM outsource_works                 WHERE org_id = p_id;
    DELETE FROM outsource_vendors               WHERE org_id = p_id;

    DELETE FROM p1_dispatch_alerts              WHERE org_id = p_id;
    DELETE FROM p1_dispatch_responses           WHERE org_id = p_id;
    DELETE FROM p1_dispatch_targets             WHERE org_id = p_id;
    DELETE FROM p1_dispatches                   WHERE org_id = p_id;

    DELETE FROM registered_devices              WHERE org_id = p_id;

    DELETE FROM regular_inspection_schedules    WHERE org_id = p_id;

    DELETE FROM sales_listing_media             WHERE org_id = p_id;
    DELETE FROM sales_listings                  WHERE org_id = p_id;

    DELETE FROM site_attendance_events          WHERE org_id = p_id;
    DELETE FROM site_geofence_presence          WHERE org_id = p_id;

    DELETE FROM support_ticket_comments         WHERE org_id = p_id;
    DELETE FROM support_tickets                 WHERE org_id = p_id;

    DELETE FROM target_change_requests          WHERE org_id = p_id;

    DELETE FROM user_branches                   WHERE org_id = p_id;

    DELETE FROM work_diary_drafts               WHERE org_id = p_id;
    DELETE FROM work_order_approval_steps       WHERE org_id = p_id;
    DELETE FROM work_order_assignments          WHERE org_id = p_id;
    DELETE FROM work_order_request_counters     WHERE org_id = p_id;
    DELETE FROM work_order_status_history       WHERE org_id = p_id;
    DELETE FROM work_orders                     WHERE org_id = p_id;

    DELETE FROM registry_equipment              WHERE org_id = p_id;
    DELETE FROM registry_sites                  WHERE org_id = p_id;
    DELETE FROM registry_customers              WHERE org_id = p_id;

    DELETE FROM users    WHERE org_id = p_id;
    DELETE FROM branches WHERE org_id = p_id;
    DELETE FROM regions  WHERE org_id = p_id;

    DELETE FROM organizations WHERE id = p_id;

    SET LOCAL row_security = on;
    RETURN 'removed';
END;
$function$;
CREATE OR REPLACE FUNCTION public.platform_force_remove_direct_org_children(p_id uuid)
 RETURNS void
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
DECLARE
    target RECORD;
BEGIN
    PERFORM public.auth_legacy_purge_company_v1(p_id);
    -- The maintenance-history subtree goes FIRST, before the catalog sweep below can reach any
    -- of the tables it points at.
    --
    -- These three are structurally invisible to that sweep: `equipment_maintenance_history` is
    -- `org_id ... ON DELETE CASCADE` (0193:19) and the sweep admits only 'a' and 'r', while
    -- `_costs` and `_evidence` reach their parents through COMPOSITE foreign keys and the sweep
    -- requires `cardinality(fk.conkey) = 1`. So the sweep deletes `work_orders`,
    -- `registry_equipment`, `evidence_media` and `equipment_cost_ledger` while rows in this
    -- subtree still reference them, and every one of those raises 23001 in turn.
    --
    -- Deleting the subtree here, child-first, removes the whole class in one place. 0196's
    -- hand-ordered block still deletes these three later; by then they are already empty, which
    -- is why this is additive rather than a reordering of that block.
    DELETE FROM equipment_maintenance_history_costs    WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history_evidence WHERE org_id = p_id;
    DELETE FROM equipment_maintenance_history          WHERE org_id = p_id;

    FOR target IN
        SELECT child_ns.nspname AS schema_name, child.relname AS relation_name
        FROM pg_catalog.pg_constraint AS fk
        JOIN pg_catalog.pg_class AS child ON child.oid = fk.conrelid
        JOIN pg_catalog.pg_namespace AS child_ns ON child_ns.oid = child.relnamespace
        JOIN pg_catalog.pg_class AS parent ON parent.oid = fk.confrelid
        JOIN pg_catalog.pg_namespace AS parent_ns ON parent_ns.oid = parent.relnamespace
        JOIN pg_catalog.pg_attribute AS child_attr
          ON child_attr.attrelid = child.oid
         AND child_attr.attnum = fk.conkey[1]
         AND NOT child_attr.attisdropped
        WHERE fk.contype = 'f'
          AND fk.confdeltype IN ('a', 'r')
          AND parent_ns.nspname = 'public'
          AND parent.relname = 'organizations'
          AND child_ns.nspname = 'public'
          AND child.relkind IN ('r', 'p')
          -- These roots have specialized ordering: the audit ledger must be
          -- re-homed, and employee/user/branch/region references are released
          -- only after their direct children have been closed by this pass.
          --
          -- equipment_cost_ledger joins them for the same reason and was missing:
          -- its children reach it by COMPOSITE FK and are therefore invisible to
          -- this catalog sweep, so deleting it here raises 23001 before the
          -- hand-ordered child-first block below can run.
          -- These roots have specialized ordering: the audit ledger must be
          -- re-homed, and employee/user/branch/region references are released
          -- only after their direct children have been closed by this pass.
          AND child.relname NOT IN (
              'audit_events', 'employees', 'users', 'branches', 'regions',
              'auth_bootstrap_credentials', 'auth_device_login_handoffs', 'auth_refresh_token_families', 'auth_refresh_tokens', 'auth_webauthn_ceremonies', 'auth_webauthn_ceremony_bindings', 'auth_webauthn_credentials'
          )
          AND cardinality(fk.conkey) = 1
          AND child_attr.attname = 'org_id'
        -- New tenant-facing tables normally reference older roots.  Descending
        -- OID gives those children priority before their direct parents.
        ORDER BY child.oid DESC
    LOOP
        EXECUTE format('DELETE FROM %I.%I WHERE org_id = $1',
                       target.schema_name, target.relation_name)
            USING p_id;
    END LOOP;
END;
$function$;
CREATE OR REPLACE FUNCTION public.platform_list_group_accounts(p_group_id uuid)
 RETURNS TABLE(user_id uuid, display_name text, phone text, tenant_roles text[], is_active boolean, has_passkey boolean, account_status text, org_id uuid, org_slug text, org_name text, group_roles text[], created_at timestamp with time zone)
 LANGUAGE plpgsql
 SECURITY DEFINER
 SET search_path TO 'public', 'pg_temp'
AS $function$
BEGIN
    SET LOCAL row_security = off;

    RETURN QUERY
        SELECT
            u.id AS user_id,
            u.display_name,
            u.phone,
            u.roles AS tenant_roles,
            u.is_active,
            public.auth_legacy_group_passkey_flag_v1(u.id) AS has_passkey,
            CASE
                WHEN NOT u.is_active THEN 'DEACTIVATED'
                WHEN public.auth_legacy_group_passkey_flag_v1(u.id) THEN 'ACTIVE'
                ELSE 'PENDING_SETUP'
            END AS account_status,
            o.id AS org_id,
            o.slug AS org_slug,
            o.name AS org_name,
            array_agg(gr.group_role ORDER BY gr.group_role) AS group_roles,
            u.created_at
        FROM group_role_grants gr
        JOIN group_memberships gm
          ON gm.group_id = gr.group_id
        JOIN users u
          ON u.id = gr.user_id
         AND u.org_id = gm.org_id
        JOIN organizations o
          ON o.id = u.org_id
         AND o.id <> '00000000-0000-0000-0000-00000000face'::uuid
        WHERE gr.group_id = p_group_id
        GROUP BY
            u.id,
            u.display_name,
            u.phone,
            u.roles,
            u.is_active,
            o.id,
            o.slug,
            o.name,
            u.created_at
        ORDER BY o.name ASC, u.display_name ASC, u.id ASC;

    SET LOCAL row_security = on;
EXCEPTION WHEN OTHERS THEN
    SET LOCAL row_security = on;
    RAISE;
END;
$function$;
GRANT EXECUTE ON FUNCTION public.account_legacy_fenced_v1(uuid) TO console_credential_owner,console_app;
GRANT EXECUTE ON FUNCTION public.account_company_deactivation_guard_v1(uuid,uuid) TO console_credential_owner,console_auth_rt;
"""


def credential_generated_files():
    query = credential_state_query()
    root_query = root_state_query()
    native_query = platform_source_state_query()
    root_inspect = 'root_state := (\n' + root_query + '\n);'
    inspect = 'state := (\n' + query + '\n);'
    native_inspect = 'native_state := (\n' + native_query + '\n);'
    historical_inspect = 'native_state := (\n' + native227_state_query() + '\n);'
    prior_audit, _ = audit_attribution_fingerprints()
    prior_audit_literals = ','.join("'" + value + "'" for value in prior_audit)
    locks = ', '.join('ONLY public.' + name for name in CREDENTIAL_TABLES)
    sql = f"""-- Generated by ops/generate-account-custody.py; one atomic operator statement.
-- Exact historical226/Auth7 -> dormant227 -> native. No row repair on replay.
DO $account_credentials$
DECLARE
    state text;
    root_state text;
    native_state text;
    new_catalog_count bigint;
    deployment_catalog_count bigint;
BEGIN
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    PERFORM pg_catalog.set_config('lock_timeout','5s',true);
    IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
       OR session_user IN ('console_app','console_rt','console_auth_rt','console_credential_owner','console_account_owner','console_terms_owner') THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account_credentials.operator_identity_mismatch';
    END IF;
    LOCK TABLE ONLY public.organizations IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE ONLY public.users IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE ONLY public.accounts, ONLY public.account_security, ONLY public.account_security_events,
        ONLY public.account_terms_acceptances, ONLY public.account_terms_head, ONLY public.account_terms_release_receipts IN ACCESS EXCLUSIVE MODE;
    LOCK TABLE {locks} IN ACCESS EXCLUSIVE MODE;
    SELECT count(*) INTO new_catalog_count FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN ('company_actors','account_context_candidates');
    IF new_catalog_count=2 THEN
        LOCK TABLE ONLY public.company_actors,ONLY public.account_context_candidates IN ACCESS EXCLUSIVE MODE;
    ELSIF new_catalog_count<>0 THEN
        RAISE EXCEPTION 'account_native.catalog_missing';
    END IF;
    -- Replay and upgrade certify the same complete locked relation set.
    SELECT count(*) INTO deployment_catalog_count FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN
            ('deployment_operator_head','deployment_operator_receipts');
    IF deployment_catalog_count=2 THEN
        LOCK TABLE ONLY public.deployment_operator_receipts,
            ONLY public.deployment_operator_head IN ACCESS EXCLUSIVE MODE;
    ELSIF deployment_catalog_count<>0 THEN
        RAISE EXCEPTION 'deployment_operator.catalog_missing';
    END IF;
    LOCK TABLE ONLY public.audit_events IN ACCESS EXCLUSIVE MODE NOWAIT;
    {root_inspect}
    {inspect}
    IF root_state='account_custody.native_finalized' AND state='account_credentials.native_finalized' THEN RETURN; END IF;
    <<prepare_platform_source>>
    BEGIN
    -- Exact populated predecessor skips historical installers and retains every row.
    IF root_state='account_custody.native_upgrade_required'
       AND state='account_credentials.native_upgrade_required'
       AND COALESCE((SELECT snapshot_sha256 IN ({','.join("'" + value + "'" for value in platform_source_fingerprints()[0])})
           FROM (
{platform_source_snapshot_query()}
           ) captured),false) THEN
        EXIT prepare_platform_source;
    END IF;
    <<prepare_company_eligibility>>
    BEGIN
    -- The exact current Business-session predecessor may already hold live
    -- Accounts, sessions and receipts. Skip only its historical installers.
    IF root_state='account_custody.native_upgrade_required'
       AND state='account_credentials.native_upgrade_required'
       AND COALESCE((SELECT snapshot_sha256 IN ({','.join("'" + value + "'" for value in company_eligibility_fingerprints()[0])})
           FROM (
{company_eligibility_snapshot_query()}
           ) captured),false) THEN
        EXIT prepare_company_eligibility;
    END IF;
    <<prepare_business_session>>
    BEGIN
    IF root_state='account_custody.native_upgrade_required'
       AND state='account_credentials.native_upgrade_required' THEN
        -- Already audited228 may contain live designation receipts and sessions.
        -- Add only the bounded helper pair; preserve all rows and prior owners.
        IF COALESCE((SELECT snapshot_sha256 IN ({','.join("'" + value + "'" for value in BUSINESS_SESSION_PRIOR_SHA256)})
            FROM (
{business_session_snapshot_query()}
            ) captured),false) THEN
            EXIT prepare_business_session;
        END IF;
        -- Initialized228 has real designation receipts. Never route it through
        -- the empty deployment staging installer.
        IF COALESCE((SELECT snapshot_sha256 IN ({prior_audit_literals})
            AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
            FROM (
{audit_attribution_snapshot_query()}
            ) captured),false) THEN
{audit_attribution_upgrade_sql()}
            EXIT prepare_business_session;
        END IF;
        -- Both immutable227 predecessors are upgrade inputs. Finish the exact
        -- three-helper transition when necessary, then certify228 staging before
        -- adding any deployment capability. All effects roll back on refusal.
        {historical_inspect}
        IF native_state='account_native.extension_required' THEN
{NATIVE_EXTENSION_INSTALL}
        ELSIF native_state IS DISTINCT FROM 'account_native.finalized' THEN
            RAISE EXCEPTION 'account_native.extension_predecessor_mismatch';
        END IF;
        {historical_inspect}
        IF native_state IS DISTINCT FROM 'account_native.finalized' THEN
            RAISE EXCEPTION 'account_native.extension_profile_mismatch';
        END IF;
{deployment_upgrade_sql()}
{audit_attribution_upgrade_sql()}
        EXIT prepare_business_session;
    END IF;
    IF root_state IS DISTINCT FROM 'account_custody.finalized' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE=COALESCE(root_state,'account_credentials.root_prerequisite');
    END IF;
    IF state NOT IN ('account_credentials.pending','account_credentials.finalized') OR state IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE=COALESCE(state,'account_credentials.legacy_profile_mismatch');
    END IF;
    IF state='account_credentials.pending' THEN
        IF pg_catalog.to_regprocedure('public.account_company_deactivation_guard_v1(uuid,uuid)') IS NULL
           OR EXISTS(SELECT 1 FROM public.users u LEFT JOIN public.accounts a ON a.id=u.id WHERE a.id IS NULL) THEN
            RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account_credentials.root_prerequisite';
        END IF;
        IF NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname='console_credential_owner') THEN
            CREATE ROLE console_credential_owner NOLOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
        END IF;
        GRANT USAGE ON SCHEMA public TO console_credential_owner;
{CREDENTIAL_INSTALL}
        {root_inspect}
        {inspect}
        IF root_state IS DISTINCT FROM 'account_custody.finalized' OR state IS DISTINCT FROM 'account_credentials.finalized' THEN
            RAISE EXCEPTION 'account_credentials.profile_mismatch';
        END IF;
    END IF;
    -- Retained historical direct tests may install226 without227. Packaged
    -- production operator independently requires the exact migration ledger.
    -- This state cannot admit native-capable serving.
    IF new_catalog_count=0 THEN RETURN; END IF;
    {native_inspect}
    IF native_state IS DISTINCT FROM 'account_native.staged' THEN
        RAISE EXCEPTION 'account_native.staging_profile_mismatch';
    END IF;
    IF EXISTS(SELECT 1 FROM public.company_actors) OR EXISTS(SELECT 1 FROM public.account_context_candidates) THEN
        RAISE EXCEPTION 'account_native.nonempty_staging';
    END IF;
{NATIVE_INSTALL}
{NATIVE_EXTENSION_INSTALL}
    {historical_inspect}
    IF native_state IS DISTINCT FROM 'account_native.finalized' THEN
        RAISE EXCEPTION 'account_native.extension_profile_mismatch';
    END IF;
{deployment_upgrade_sql()}
{audit_attribution_upgrade_sql()}
    END prepare_business_session;
{business_session_upgrade_sql()}
    END prepare_company_eligibility;
{company_eligibility_upgrade_sql()}
    END prepare_platform_source;
{platform_source_upgrade_sql()}
    {native_inspect}
    IF native_state IS DISTINCT FROM 'account_native.finalized' THEN
        RAISE EXCEPTION 'account_native.profile_mismatch';
    END IF;
END
$account_credentials$;
"""
    return {'backend/app/src/account_credential_custody_state.sql': query+';\n',
            'ops/postgres-finalize-account-credentials.sql': sql}


# Native metadata uses the existing Auth7 serializer and finite reviewed whole
# profiles. These fingerprints are source authority, never learned by serving or
# installer execution. A capture is reviewed against declared DDL before pinning.
NATIVE_STAGED_SHA256 = 'd148c9adeaa0f22286816d9bdb8fe8be3b899b9dfd1f5831d42fb1f41bc24200'
NATIVE_FINALIZED9_SHA256 = 'a453c9f30950a3f3ff9f545d6fb40bfc35ba0f79b97b94902be19cce2736d481'
# SOURCE-PROPOSAL HOLD: root independently captures and reviews the declared
# extension before replacing None with its fixed complete metadata digest.
# This is deliberately not a placeholder accepted by generation or serving.
NATIVE_FINALIZED_SHA256: str | None = 'dc8e947c67171c1e10c268d4a1d54f18c1b5b39947d13b20a604d7e6f8de6766'


def native227_state_query():
    for fingerprint in (NATIVE_STAGED_SHA256, NATIVE_FINALIZED9_SHA256, NATIVE_FINALIZED_SHA256):
        if not isinstance(fingerprint, str) or len(fingerprint) != 64 or any(c not in '0123456789abcdef' for c in fingerprint):
            raise SystemExit('native metadata fingerprints require independent source/capture review')
    if len({NATIVE_STAGED_SHA256, NATIVE_FINALIZED9_SHA256, NATIVE_FINALIZED_SHA256}) != 3:
        raise SystemExit('native staged, predecessor and extended profiles must be distinct')
    return f"""WITH captured AS (
{NATIVE_SNAPSHOT_QUERY}
)
SELECT CASE WHEN snapshot_sha256='{NATIVE_FINALIZED_SHA256}' THEN 'account_native.finalized'
 WHEN snapshot_sha256='{NATIVE_FINALIZED9_SHA256}' THEN 'account_native.extension_required'
 WHEN snapshot_sha256='{NATIVE_STAGED_SHA256}' THEN 'account_native.staged'
 WHEN EXISTS(SELECT 1 FROM pg_catalog.pg_attribute a WHERE NOT a.attisdropped AND
    ((a.attrelid=pg_catalog.to_regclass('public.auth_refresh_token_families') AND a.attname='protocol')
     OR (a.attrelid=pg_catalog.to_regclass('public.auth_webauthn_ceremonies') AND a.attname='account_browser_flow')))
   OR EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname IN ('account_registration_begin_v1',
      'account_security_lock_exclusive_v1','account_security_lock_shared_v1',
      'account_terms_registration_head_v1','auth_account_registration_material_v1',
      'account_registration_activate_v1','account_context_presence_v1','account_security_events_immutable_v1',
      'account_session_logout_v1','auth_account_logout_revoke_v1',
      'account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1'))
   OR EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
    JOIN pg_catalog.pg_roles r ON r.oid=c.relowner WHERE n.nspname='public'
      AND c.relname IN ('company_actors','account_context_candidates') AND r.rolname='console_account_owner')
 THEN 'account_native.profile_mismatch' ELSE 'account_native.historical' END AS state
FROM captured"""



# Independent declared-source captures are required before generation. These
# profiles use the current serializer; the three immutable227 pins stay above.
DEPLOYMENT_STAGED_SHA256: tuple[str | None, str | None] = ('e3dbc30c9d54e4f3a6c02bb97cf38178e275ee85d4fd4e37639ecb1568839c1b', '7b60d45fd5376efa03c067cb953299a53ded8023b8f32e9bd1476925c29e8ced')
DEPLOYMENT_FINALIZED_SHA256: tuple[str | None, str | None] = ('272042276dd83c00742a9d78682091471d2b3786e10d65c79477d38b70ef7370', 'cc8a0d42cb0231e5522e21f60e139ee921bd66c9996a157944ca83dd02d6015c')


def deployment_fingerprints():
    values = (*DEPLOYMENT_STAGED_SHA256, *DEPLOYMENT_FINALIZED_SHA256)
    if any(not isinstance(value, str) or len(value) != 64
           or any(c not in '0123456789abcdef' for c in value) for value in values):
        raise SystemExit('deployment metadata fingerprints require independent source/capture review')
    if len(set(values)) != 4:
        raise SystemExit('deployment plain/observer staged/finalized profiles must be distinct')
    return DEPLOYMENT_STAGED_SHA256, DEPLOYMENT_FINALIZED_SHA256


# Four independently captured profiles include the immutable audit guards and
# every incoming/outgoing FK. Preserve the original227/228 serializers and pins.
AUDIT_PRIOR_SHA256 = ('4833199d04f5dbed4f6af3f7d01e1f9adc3bdbe9ed487eb21f651ffbfa9b4565', '8983667107231012c3e79a98656a9330e36c3437072bfc4b47739472d61f8def')
AUDIT_FINALIZED_SHA256 = ('9dfd940f4018b543d52efc15745839004754ed5e427345aee04cdb5675d7c2de', '1702ba9c1b2722e6b9dd1aadd2fe2b314cb6739732926e3fc370f0a348e2d457')


def audit_attribution_fingerprints():
    values = (*AUDIT_PRIOR_SHA256, *AUDIT_FINALIZED_SHA256)
    if any(not isinstance(value, str) or len(value) != 64
           or any(c not in '0123456789abcdef' for c in value) for value in values):
        raise SystemExit('audit metadata fingerprints require independent source/capture review')
    if len(set(values)) != 4:
        raise SystemExit('audit plain/observer prior/finalized profiles must be distinct')
    return AUDIT_PRIOR_SHA256, AUDIT_FINALIZED_SHA256


def audit_attribution_snapshot_query():
    query = deployment228_snapshot_query()
    replacements = [
        (" ('deployment_operator_head')\n), relations AS (", " ('deployment_operator_head'),\n ('audit_events')\n), relations AS ("),
        (" WHERE p.proowner IN (SELECT oid FROM deployment_observer_active)", " WHERE p.oid IN (SELECT tgfoid FROM pg_trigger WHERE tgrelid=to_regclass('public.audit_events') AND NOT tgisinternal) OR p.proowner IN (SELECT oid FROM deployment_observer_active)"),
        ("count(*)=17 AND count(oid)=17", "count(*)=18 AND count(oid)=18"),
        ("count(*)=26 AND count(oid)=26", "count(*)=27 AND count(oid)=27"),
        ("count(*)=208 AND bool_and(allowed IS FALSE)", "count(*)=216 AND bool_and(allowed IS FALSE)"),
        ("count(DISTINCT name)=26 AND bool_and(allowed IS FALSE)", "count(DISTINCT name)=27 AND bool_and(allowed IS FALSE)"),
    ]
    for before, after in replacements:
        if query.count(before) != 1:
            raise ValueError('audit snapshot anchor missing or duplicate')
        query = query.replace(before, after)
    return query


# Bounded same-transaction projection; no Business table or raw Account grants.
BUSINESS_SESSION_INSTALL = r"""
CREATE FUNCTION public.auth_account_session_shared_material_v1(p_account uuid, p_family uuid)
RETURNS TABLE(user_id uuid, protocol text, account_security_generation bigint,
    auth_time timestamptz, assurance text, created_at timestamptz,
    revoked_at timestamptz, org_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    IF p_account IS NULL OR p_family IS NULL
        OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR p_family='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
    RETURN QUERY SELECT f.user_id,f.protocol,f.account_security_generation,
        f.auth_time,f.assurance,f.created_at,f.revoked_at,f.org_id
        FROM public.auth_refresh_token_families f
        WHERE f.id=p_family AND f.user_id=p_account FOR SHARE OF f;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
END
$body$;
ALTER FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid)
    FROM PUBLIC,console_app,console_rt,console_auth_rt,console_auth_startup,
        console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_account_session_shared_material_v1(uuid,uuid)
    TO console_credential_owner,console_account_owner;

CREATE FUNCTION public.account_session_shared_material_v1(p_account uuid, p_family uuid)
RETURNS TABLE(security_state text, security_generation bigint, revision bigint,
    context_generation bigint, user_id uuid, protocol text,
    account_security_generation bigint, auth_time timestamptz, assurance text,
    created_at timestamptz, revoked_at timestamptz, org_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    control record;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    IF p_account IS NULL OR p_family IS NULL
        OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR p_family='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
    SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
    RETURN QUERY SELECT control.security_state,control.security_generation,
        control.revision,control.context_generation,
        f.user_id,f.protocol,f.account_security_generation,f.auth_time,
        f.assurance,f.created_at,f.revoked_at,f.org_id
        FROM public.auth_account_session_shared_material_v1(p_account,p_family) f;
EXCEPTION WHEN no_data_found THEN
    RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
END
$body$;
ALTER FUNCTION public.account_session_shared_material_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_session_shared_material_v1(uuid,uuid)
    FROM PUBLIC,console_app,console_rt,console_auth_rt,console_auth_startup,
        console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_session_shared_material_v1(uuid,uuid)
    TO console_account_owner,console_auth_rt,console_rt;
"""


def business_session_snapshot_query():
    # Preserve previous serializers and include foreign-owner overloads by name.
    query = audit_attribution_snapshot_query()
    replacements = [
        ("OR (n.nspname='public' AND p.proname IN (", "OR (n.nspname='public' AND p.proname IN ('account_session_shared_material_v1','auth_account_session_shared_material_v1',"),
        ("   ('public.account_context_presence_v1(uuid)'),", "   ('public.account_session_shared_material_v1(uuid,uuid)'),\n   ('public.auth_account_session_shared_material_v1(uuid,uuid)'),\n   ('public.account_context_presence_v1(uuid)'),"),
        ("count(*)=57 AND bool_and(present", "count(*)=59 AND bool_and(present"),
    ]
    for before, after in replacements:
        if query.count(before) != 1:
            raise ValueError('Business session snapshot anchor missing or duplicate')
        query = query.replace(before, after)
    return query


def audit_native_state_query():
    prior, finalized = audit_attribution_fingerprints()
    prior_literals = ",".join("'" + value + "'" for value in prior)
    final_literals = ",".join("'" + value + "'" for value in finalized)
    return f"""WITH audit AS (
{audit_attribution_snapshot_query()}
), historical AS (
{native227_state_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM audit) IN ({final_literals})
 AND (SELECT snapshot->'deployment_operator_boundary'->'startup_final_rights_valid' FROM audit)='true'::jsonb
 THEN 'account_native.finalized'
 WHEN (SELECT snapshot_sha256 FROM audit) IN ({prior_literals})
 AND (SELECT snapshot->'deployment_operator_boundary'->'startup_final_rights_valid' FROM audit)='true'::jsonb
 THEN 'account_native.extension_required'
 WHEN (SELECT historical.state FROM historical) IN ('account_native.finalized','account_native.extension_required')
 THEN 'account_native.extension_required'
 ELSE (SELECT historical.state FROM historical) END AS state"""


BUSINESS_SESSION_PRIOR_SHA256 = ('46882185abbec62cf868718cb5e3d72a9002fd436c042ce3f3293b1dc56d48a5', 'ace188362c3ff000a534f5abd5359c588d9e508ec5192eac6eda852d0a130629')
BUSINESS_SESSION_FINALIZED_SHA256 = ('c4582399f122012f1bf7e0edc0fe7639c6403d30d4ce6d78bd6864c50ad3a531', '7429d460a604d3dfe5bf9472fefcb8505371f6060efbf07aa5bdc477e66dcf31')


def native_state_query():
    prior_literals = ",".join("'" + value + "'" for value in BUSINESS_SESSION_PRIOR_SHA256)
    final_literals = ",".join("'" + value + "'" for value in BUSINESS_SESSION_FINALIZED_SHA256)
    return f"""WITH current_profile AS (
{business_session_snapshot_query()}
), historical AS (
{audit_native_state_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM current_profile) IN ({final_literals})
 AND (SELECT snapshot->'deployment_operator_boundary'->'startup_final_rights_valid' FROM current_profile)='true'::jsonb
 THEN 'account_native.finalized'
 WHEN (SELECT snapshot_sha256 FROM current_profile) IN ({prior_literals})
 THEN 'account_native.extension_required'
 WHEN EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname='public' AND p.proname IN ('account_session_shared_material_v1','auth_account_session_shared_material_v1'))
 THEN 'account_native.profile_mismatch'
 WHEN (SELECT historical.state FROM historical) IN ('account_native.finalized','account_native.extension_required')
 THEN 'account_native.extension_required'
 ELSE (SELECT historical.state FROM historical) END AS state"""



# Exact reviewed single-owner source; no history or privilege repair.
COMPANY_ELIGIBILITY_INSTALL = r"""-- One prospective read owner. Apply only after exact predecessor validation
-- inside the existing custody finalizer transaction; never standalone rollout.
CREATE FUNCTION public.account_company_setup_eligibility_v1(p_account uuid)
RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    control record;
    head public.deployment_operator_head%ROWTYPE;
    receipt public.deployment_operator_receipts%ROWTYPE;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
    BEGIN
        SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
    EXCEPTION WHEN no_data_found THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END;
    IF control.security_state IS DISTINCT FROM 'ACTIVE' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authentication_invalid';
    END IF;
    IF control.security_generation IS NULL OR control.security_generation < 1
        OR control.revision IS NULL OR control.revision < 1
        OR control.context_generation IS NULL OR control.context_generation < 1 THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;

    -- Account SHARE also serializes an absent head against same-Account genesis.
    -- Keep the selected head SHARE until the caller's transaction ends.
    SELECT h.* INTO head FROM public.deployment_operator_head h
        WHERE h.singleton=1 AND h.account_id=p_account FOR SHARE OF h;
    IF NOT FOUND THEN RETURN false; END IF;

    -- Separate SPI statement gets fresh visibility after a possible head wait.
    SELECT r.* INTO receipt FROM public.deployment_operator_receipts r
        WHERE r.receipt_id=head.receipt_id;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;
    IF (receipt.receipt_id,receipt.revision,receipt.account_id,
            receipt.system_identifier,receipt.database_name,receipt.database_oid)
        IS DISTINCT FROM
        (head.receipt_id,head.revision,p_account,
            head.system_identifier,head.database_name,head.database_oid)
        OR head.system_identifier IS DISTINCT FROM
            (SELECT c.system_identifier::text FROM pg_catalog.pg_control_system() c)
        OR head.database_name IS DISTINCT FROM pg_catalog.current_database()::text
        OR head.database_oid IS DISTINCT FROM
            (SELECT d.oid::bigint FROM pg_catalog.pg_database d
                WHERE d.datname=pg_catalog.current_database())
        OR receipt.receipt_id='00000000-0000-0000-0000-000000000000'::uuid
        OR receipt.command_id='00000000-0000-0000-0000-000000000000'::uuid
        OR receipt.revision < 1 OR receipt.expected_revision < 0
        OR receipt.expected_revision IS DISTINCT FROM receipt.revision-1 THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
    END IF;
    IF receipt.kind='DESIGNATE'
        AND receipt.revision=1 AND receipt.expected_revision=0
        AND receipt.expected_security_generation IS NOT NULL
        AND receipt.expected_security_generation > 0 AND receipt.reason IS NULL THEN
        -- Admission generation is immutable provenance, not a live-session fence.
        RETURN true;
    END IF;
    IF receipt.kind='REVOKE' AND receipt.revision > 1
        AND receipt.expected_security_generation IS NULL
        AND receipt.reason IS NOT NULL
        AND pg_catalog.octet_length(receipt.reason) BETWEEN 1 AND 512
        AND pg_catalog.length(pg_catalog.btrim(receipt.reason,E' \t\n\r\f\013')) > 0 THEN
        RETURN false;
    END IF;
    RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account.authority_unavailable';
END
$body$;
ALTER FUNCTION public.account_company_setup_eligibility_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_company_setup_eligibility_v1(uuid)
    FROM PUBLIC,console_app,console_rt,console_auth_rt,console_auth_startup,
        console_account_owner,console_terms_owner,console_credential_owner,
        console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
GRANT EXECUTE ON FUNCTION public.account_company_setup_eligibility_v1(uuid)
    TO console_account_owner,console_auth_rt;
"""


def company_eligibility_snapshot_query():
    # Extend the unchanged Business-session serializer; historical bytes and
    # fingerprints stay valid only for the historical contract they describe.
    query = business_session_snapshot_query()
    replacements = [
        ("OR (n.nspname='public' AND p.proname IN (", "OR (n.nspname='public' AND p.proname IN ('account_company_setup_eligibility_v1',"),
        ("   ('public.account_context_presence_v1(uuid)'),", "   ('public.account_company_setup_eligibility_v1(uuid)'),\n   ('public.account_context_presence_v1(uuid)'),"),
        ("count(*)=59 AND bool_and(present", "count(*)=60 AND bool_and(present"),
    ]
    for before, after in replacements:
        if query.count(before) != 1:
            raise ValueError('Company eligibility snapshot anchor missing or duplicate')
        query = query.replace(before, after)
    return query


# Prospective plain/observer profiles captured from exact reviewed source.
# Historical Business-session serializers and fingerprint tuples stay unchanged.
COMPANY_ELIGIBILITY_PRIOR_SHA256 = ('e60fe3970fbf628d292791a645ac69856ef0f4d3a5179adeea8b8fbe6367d1e9', '548bb9642cee4cd3c31d8ad6475575a78f0560e9f08c0f60b49163dc07d9f9f2')
COMPANY_ELIGIBILITY_FINALIZED_SHA256 = ('ffdd70fc9ea8f96509a9614826415226896d7deed857930b1257f47d920a6131', '88a0df75fdb3e18280b9a64951fb12fea491febb1ec3868c8016b86f8ea8f963')


def company_eligibility_fingerprints():
    values = (*COMPANY_ELIGIBILITY_PRIOR_SHA256, *COMPANY_ELIGIBILITY_FINALIZED_SHA256)
    if any(not isinstance(value, str) or len(value) != 64
           or any(c not in '0123456789abcdef' for c in value) for value in values):
        raise SystemExit('Company eligibility fingerprints require independent source/capture review')
    if len(set(values)) != 4:
        raise SystemExit('Company eligibility plain/observer prior/finalized profiles must be distinct')
    return COMPANY_ELIGIBILITY_PRIOR_SHA256, COMPANY_ELIGIBILITY_FINALIZED_SHA256


def company_eligibility_state_query():
    prior, finalized = company_eligibility_fingerprints()
    prior_literals = ','.join("'" + value + "'" for value in prior)
    final_literals = ','.join("'" + value + "'" for value in finalized)
    return f"""WITH current_profile AS (
{company_eligibility_snapshot_query()}
), historical AS (
{native_state_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM current_profile) IN ({final_literals})
 AND (SELECT snapshot->'deployment_operator_boundary'->'startup_final_rights_valid' FROM current_profile)='true'::jsonb
 THEN 'account_native.finalized'
 WHEN (SELECT snapshot_sha256 FROM current_profile) IN ({prior_literals})
 THEN 'account_native.extension_required'
 WHEN EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname='public' AND p.proname='account_company_setup_eligibility_v1')
 THEN 'account_native.profile_mismatch'
 WHEN (SELECT historical.state FROM historical) IN ('account_native.finalized','account_native.extension_required')
 THEN 'account_native.extension_required'
 ELSE (SELECT historical.state FROM historical) END AS state"""


PLATFORM_SOURCE_INSTALL = r"""-- Prospective retained source facts; installed only by the locked custody finalizer.
CREATE FUNCTION public.auth_legacy_platform_source_material_v1(p_subject uuid,p_family uuid)
RETURNS TABLE(roles text[],family_id uuid,family_user_id uuid,family_org_id uuid,
    family_protocol text,family_created_at timestamptz,family_revoked_at timestamptz,
    family_account_security_generation bigint,family_auth_time timestamptz,family_assurance text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    home constant uuid := '00000000-0000-0000-0000-00000000face';
    current_roles text[];
    guarded boolean;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy.unsupported_isolation';
    END IF;
    IF home IS DISTINCT FROM NULLIF(current_setting('app.current_org',true),'')::uuid THEN
        RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='auth_legacy.company_context_mismatch';
    END IF;
    IF p_subject IS NULL OR p_subject='00000000-0000-0000-0000-000000000000'::uuid
        OR p_family='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='auth_legacy_platform.invalid_identity';
    END IF;
    PERFORM public.auth_legacy_company_lock_v1(home);
    SELECT public.account_company_deactivation_guard_v1(home,p_subject) INTO guarded;
    IF guarded IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_platform.invalid_guard';
    END IF;
    IF NOT guarded THEN
        RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='auth_legacy_platform.source_fenced';
    END IF;
    SELECT public.auth_legacy_user_active_v1(home,p_subject) INTO guarded;
    IF guarded IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_platform.invalid_guard';
    END IF;
    IF NOT guarded THEN
        RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='auth_legacy_platform.source_inactive';
    END IF;
    SELECT c.roles INTO STRICT current_roles FROM public.auth_legacy_session_context_v1(home,p_subject) c;
    IF current_roles IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_platform.invalid_roles';
    END IF;
    IF cardinality(current_roles)=0 THEN
        RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='auth_legacy.subject_has_no_roles';
    END IF;
    IF p_family IS NULL THEN
        RETURN QUERY SELECT current_roles,NULL::uuid,NULL::uuid,NULL::uuid,NULL::text,
            NULL::timestamptz,NULL::timestamptz,NULL::bigint,NULL::timestamptz,NULL::text;
    ELSE
        RETURN QUERY SELECT current_roles,f.id,f.user_id,f.org_id,f.protocol,f.created_at,
            f.revoked_at,f.account_security_generation,f.auth_time,f.assurance
            FROM public.auth_refresh_token_families f WHERE f.id=p_family FOR SHARE OF f;
        IF NOT FOUND THEN
            RAISE EXCEPTION USING ERRCODE='P0002',MESSAGE='auth_legacy_platform.family_not_found';
        END IF;
    END IF;
END
$body$;
ALTER FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid)
    FROM PUBLIC,console_app,console_rt,console_auth_rt,console_auth_startup,
        console_account_owner,console_terms_owner,console_credential_owner,
        console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid)
    TO console_credential_owner,console_rt;
GRANT EXECUTE ON FUNCTION public.auth_legacy_session_context_v1(uuid,uuid) TO console_credential_owner;
"""


def platform_source_snapshot_query():
    # Historical serializers and their pins retain their original meaning.
    query = company_eligibility_snapshot_query()
    replacements = [
        ("OR (n.nspname='public' AND p.proname IN (", "OR (n.nspname='public' AND p.proname IN ('auth_legacy_platform_source_material_v1',"),
        ("   ('public.account_context_presence_v1(uuid)'),", "   ('public.auth_legacy_platform_source_material_v1(uuid,uuid)'),\n   ('public.account_context_presence_v1(uuid)'),"),
        ("count(*)=60 AND bool_and(present", "count(*)=61 AND bool_and(present"),
    ]
    for before, after in replacements:
        if query.count(before) != 1:
            raise ValueError('Platform source snapshot anchor missing or duplicate')
        query = query.replace(before, after)
    return query



# Filled only from independently reviewed source-bound captures.
PLATFORM_SOURCE_PRIOR_SHA256 = ('0f2db80a4afeacb1d6d6aaf0bbaef9d8f7968e77500322cf691774f02d88e8ea', '93236c9a9e850db234f4dc777553e4eb829bbb76a0f6cc27551f761cc28bffe7')
PLATFORM_SOURCE_FINALIZED_SHA256 = ('5aee358ff0bea94268d40b5f16f86779952ea95a74cab48cbf99984eedf6e2da', '475c1f9a43b402539e0469f31adc6f06848669abfa34c08350aa1ee006664103')


def platform_source_fingerprints():
    values = (*PLATFORM_SOURCE_PRIOR_SHA256, *PLATFORM_SOURCE_FINALIZED_SHA256)
    if any(not isinstance(value, str) or len(value) != 64
           or any(c not in '0123456789abcdef' for c in value) for value in values):
        raise SystemExit('Platform source fingerprints require independent source/capture review')
    if len(set(values)) != 4:
        raise SystemExit('Platform source plain/observer prior/finalized profiles must be distinct')
    return PLATFORM_SOURCE_PRIOR_SHA256, PLATFORM_SOURCE_FINALIZED_SHA256


def platform_source_state_query():
    prior, finalized = platform_source_fingerprints()
    prior_literals = ','.join("'" + value + "'" for value in prior)
    final_literals = ','.join("'" + value + "'" for value in finalized)
    return f"""WITH current_profile AS (
{platform_source_snapshot_query()}
), historical AS (
{company_eligibility_state_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM current_profile) IN ({final_literals})
 AND (SELECT snapshot->'deployment_operator_boundary'->'startup_final_rights_valid' FROM current_profile)='true'::jsonb
 THEN 'account_native.finalized'
 WHEN (SELECT snapshot_sha256 FROM current_profile) IN ({prior_literals})
 THEN 'account_native.extension_required'
 WHEN EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname='public' AND p.proname='auth_legacy_platform_source_material_v1')
 THEN 'account_native.profile_mismatch'
 WHEN (SELECT historical.state FROM historical) IN ('account_native.finalized','account_native.extension_required')
 THEN 'account_native.extension_required'
 ELSE (SELECT historical.state FROM historical) END AS state"""


def platform_source_upgrade_sql():
    prior, finalized = platform_source_fingerprints()
    prior_literals = ','.join("'" + value + "'" for value in prior)
    final_literals = ','.join("'" + value + "'" for value in finalized)
    inspect = '(\n' + platform_source_snapshot_query() + '\n)'
    return f"""
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({prior_literals}) FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'auth_legacy_platform.predecessor_mismatch';
    END IF;
{PLATFORM_SOURCE_INSTALL}
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({final_literals})
        AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
        FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'auth_legacy_platform.profile_mismatch';
    END IF;
"""

def company_eligibility_upgrade_sql():
    prior, finalized = company_eligibility_fingerprints()
    prior_literals = ','.join("'" + value + "'" for value in prior)
    final_literals = ','.join("'" + value + "'" for value in finalized)
    inspect = '(\n' + company_eligibility_snapshot_query() + '\n)'
    return f"""
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({prior_literals}) FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'account_company_eligibility.predecessor_mismatch';
    END IF;
{COMPANY_ELIGIBILITY_INSTALL}
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({final_literals})
        AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
        FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'account_company_eligibility.profile_mismatch';
    END IF;
"""



def business_session_upgrade_sql():
    prior_literals = ",".join("'" + value + "'" for value in BUSINESS_SESSION_PRIOR_SHA256)
    final_literals = ",".join("'" + value + "'" for value in BUSINESS_SESSION_FINALIZED_SHA256)
    inspect = '(\n' + business_session_snapshot_query() + '\n)'
    return f"""
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({prior_literals}) FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'account_business_session.predecessor_mismatch';
    END IF;
{BUSINESS_SESSION_INSTALL}
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({final_literals})
        AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
        FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'account_business_session.profile_mismatch';
    END IF;
"""


def audit_attribution_upgrade_sql():
    prior, finalized = audit_attribution_fingerprints()
    prior_literals = ",".join("'" + value + "'" for value in prior)
    final_literals = ",".join("'" + value + "'" for value in finalized)
    inspect = '(\n' + audit_attribution_snapshot_query() + '\n)'
    return f"""
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({prior_literals})
        AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
        FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'account_audit.predecessor_mismatch';
    END IF;
    IF EXISTS(SELECT 1 FROM public.users u LEFT JOIN public.accounts a ON a.id=u.id WHERE a.id IS NULL) THEN
        RAISE EXCEPTION USING MESSAGE='account_root_transition.missing_root', ERRCODE='P0001';
    END IF;
    ALTER TABLE ONLY public.audit_events
        DROP CONSTRAINT audit_events_actor_fkey,
        ADD CONSTRAINT audit_events_actor_fkey FOREIGN KEY (actor)
            REFERENCES public.accounts(id) ON UPDATE NO ACTION ON DELETE RESTRICT NOT DEFERRABLE;
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({final_literals})
        AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
        FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'account_audit.profile_mismatch';
    END IF;
"""


def deployment_upgrade_sql():
    staged, finalized = deployment_fingerprints()
    stage_literals = ",".join("'" + value + "'" for value in staged)
    final_literals = ",".join("'" + value + "'" for value in finalized)
    inspect = '(\n' + deployment228_snapshot_query() + '\n)'
    return f"""
    -- The ordinary228 migration creates only local staging. The operator owns
    -- Account FK attachment and capabilities, in this same locked transaction.
    IF (SELECT count(*) FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relname IN
            ('deployment_operator_head','deployment_operator_receipts'))<>2 THEN
        RAISE EXCEPTION 'deployment_operator.catalog_missing';
    END IF;
    LOCK TABLE ONLY public.deployment_operator_receipts,
        ONLY public.deployment_operator_head IN ACCESS EXCLUSIVE MODE;
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({stage_literals}) FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'deployment_operator.staging_profile_mismatch';
    END IF;
    IF EXISTS(SELECT 1 FROM public.deployment_operator_receipts)
       OR EXISTS(SELECT 1 FROM public.deployment_operator_head) THEN
        RAISE EXCEPTION 'deployment_operator.nonempty_staging';
    END IF;
{DEPLOYMENT_INSTALL}
    IF NOT COALESCE((SELECT snapshot_sha256 IN ({final_literals})
        AND snapshot->'deployment_operator_boundary'->'startup_final_rights_valid'='true'::jsonb
        FROM {inspect} captured),false) THEN
        RAISE EXCEPTION 'deployment_operator.profile_mismatch';
    END IF;
"""


def native_postcondition_sql():
    return f"""-- Generated from the same native classifier used by serving admission.
-- Packaged transport executes this last, before its one owning transaction commits.
-- Historical direct root/Auth7 installers intentionally do not execute this check.
DO $account_native_postcondition$
BEGIN
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    IF (
{platform_source_state_query()}
    ) IS DISTINCT FROM 'account_native.finalized' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='account_native.finalization_incomplete';
    END IF;
END
$account_native_postcondition$;
"""


def root_state_query():
    return f"""WITH native AS ({platform_source_state_query()})
SELECT CASE WHEN (SELECT native.state FROM native)='account_native.finalized' THEN 'account_custody.native_finalized'
 WHEN (SELECT native.state FROM native)='account_native.extension_required' THEN 'account_custody.native_upgrade_required'
 WHEN (SELECT native.state FROM native)='account_native.profile_mismatch' THEN 'account_native.profile_mismatch'
 ELSE ({LEGACY_ROOT_QUERY}) END"""


def credential_state_query():
    legacy = CREDENTIAL_STATE_QUERY.strip().removesuffix(';')
    return f"""WITH native AS ({platform_source_state_query()})
SELECT CASE WHEN (SELECT native.state FROM native)='account_native.finalized' THEN 'account_credentials.native_finalized'
 WHEN (SELECT native.state FROM native)='account_native.extension_required' THEN 'account_credentials.native_upgrade_required'
 WHEN (SELECT native.state FROM native)='account_native.profile_mismatch' THEN 'account_native.profile_mismatch'
 ELSE ({legacy}) END"""

LEGACY_ROOT_QUERY = r"""-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT CASE
 WHEN (SELECT count(*) FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner')
   AND NOT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolinherit
   AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) <> 2
   OR EXISTS (SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid=m.roleid OR r.oid=m.member
     WHERE r.rolname IN ('console_account_owner','console_terms_owner'))
 THEN 'account_custody.owner_topology_mismatch'
 WHEN EXISTS (SELECT 1 FROM relations WHERE oid IS NULL)
 THEN 'account_custody.catalog_missing'
 WHEN (SELECT present AND NOT valid FROM auth7_root_profile)
 THEN 'account_credentials.root_boundary_mismatch'
 WHEN (SELECT present AND NOT valid FROM root_profile)
 THEN 'account_root_transition.profile_mismatch'
 WHEN EXISTS (SELECT 1 FROM relations WHERE actual_shape IS DISTINCT FROM
   CASE WHEN name='account_terms_release_receipts' AND
     ((SELECT present FROM receipt_guard) OR (SELECT guarded OR ready FROM acl_profiles))
     THEN 'b72be303c47b094e2291e00bc0a98345ac5cc63a8be1520373ea4cb07c90c410'
     ELSE shape_sha256 END)
   OR ((SELECT present FROM receipt_guard) AND NOT COALESCE((SELECT valid FROM guard_trigger),false))
   OR EXISTS (SELECT 1 FROM pg_class c JOIN expected e ON e.name=c.relname
     JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname<>'public')
 THEN 'account_custody.catalog_shape_mismatch'
 WHEN NOT COALESCE((SELECT pending OR finalized FROM ownership),false)
 THEN 'account_custody.owner_mismatch'
 WHEN ((SELECT dormant FROM table_acl_profiles) AND (SELECT present FROM projection))
   OR ((SELECT prepared OR guarded OR ready FROM acl_profiles) AND NOT (SELECT present FROM projection))
 THEN 'account_fence_projection.profile_mismatch'
 WHEN (SELECT present AND NOT valid FROM projection)
 THEN 'account_fence_projection.definition_mismatch'
 WHEN (SELECT present AND NOT valid FROM receipt_guard)
 THEN 'account_terms_receipts.definition_mismatch'
 WHEN (SELECT present AND NOT valid FROM terms_current)
 THEN 'account_terms_current.definition_mismatch'
 WHEN (((SELECT ready FROM acl_profiles) OR (SELECT present FROM root_profile)) AND NOT (SELECT present FROM terms_current))
   OR ((SELECT present FROM terms_current) AND (SELECT dormant OR prepared OR guarded FROM acl_profiles))
 THEN 'account_terms_current.profile_mismatch'
 WHEN (SELECT present AND NOT valid FROM deactivation_guard)
 THEN 'account_company_deactivation.definition_mismatch'
 WHEN ((SELECT present FROM deactivation_guard)
     AND (NOT (SELECT valid FROM root_profile) OR NOT COALESCE((SELECT valid FROM deactivation_users_acl),false)))
   OR (NOT (SELECT present FROM deactivation_guard) AND NOT COALESCE((SELECT dormant FROM deactivation_users_acl),false))
 THEN 'account_company_deactivation.profile_mismatch'
 WHEN NOT COALESCE((SELECT dormant OR prepared OR guarded OR ready FROM acl_profiles),false)
   -- INSERT normalization must not hide duplicate or empty raw ACL items.
   OR ((SELECT present FROM root_profile) AND EXISTS(
     SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     WHERE c.name='accounts' AND COALESCE(cardinality(a.attacl),0)<>
       (SELECT count(DISTINCT (x.grantee,x.grantor)) FROM aclexplode(a.attacl) x)))
   OR EXISTS (SELECT 1 FROM relations c CROSS JOIN pg_roles r
     WHERE r.rolname NOT LIKE 'pg\_%' ESCAPE '\' AND r.oid<>c.relowner AND NOT r.rolsuper
     AND (has_table_privilege(r.oid,c.oid,'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
       OR has_any_column_privilege(r.oid,c.oid,
         CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile)
           THEN 'SELECT,UPDATE,REFERENCES' ELSE 'SELECT,INSERT,UPDATE,REFERENCES' END)
       OR pg_has_role(r.oid,c.relowner,'MEMBER') OR pg_has_role(r.oid,c.relowner,'SET')))
 THEN 'account_custody.unexpected_privilege'
 WHEN (SELECT present FROM root_profile) AND NOT (SELECT valid FROM root_insert_acl)
 THEN 'account_root_transition.profile_mismatch'
 WHEN (SELECT finalized FROM ownership) AND (SELECT ready FROM acl_profiles)
   AND (SELECT valid FROM projection) AND (SELECT valid FROM receipt_guard)
   AND (SELECT valid FROM terms_current)
   AND COALESCE((SELECT valid FROM guard_trigger),false)
 THEN CASE WHEN (SELECT valid FROM root_profile) AND (SELECT valid FROM deactivation_guard)
     AND (SELECT valid FROM deactivation_users_acl) THEN 'account_custody.finalized'
   ELSE 'account_custody.upgrade_required' END
 -- Historical profiles are complete upgrade inputs, never serving profiles.
 WHEN (SELECT finalized FROM ownership) AND (SELECT guarded FROM acl_profiles)
   AND NOT (SELECT present FROM terms_current)
   AND (SELECT valid FROM projection) AND (SELECT valid FROM receipt_guard)
   AND COALESCE((SELECT valid FROM guard_trigger),false)
 THEN 'account_custody.upgrade_required'
 WHEN NOT (SELECT present FROM receipt_guard) AND NOT (SELECT present FROM terms_current)
   AND ((SELECT dormant FROM acl_profiles) OR
     ((SELECT prepared FROM acl_profiles) AND (SELECT valid FROM projection)))
 THEN CASE WHEN (SELECT pending FROM ownership) THEN 'account_custody.pending'
   ELSE 'account_custody.upgrade_required' END
 ELSE 'account_custody.unexpected_privilege'
END"""

NATIVE_SNAPSHOT_QUERY = r"""-- READ-ONLY disposable capture. Freeze only after independent declared-source comparison.
WITH wanted(name) AS (VALUES
 ('accounts'),
 ('account_security'),
 ('account_security_events'),
 ('account_terms_acceptances'),
 ('account_terms_head'),
 ('account_terms_release_receipts'),
 ('auth_bootstrap_credentials'),
 ('auth_device_login_handoffs'),
 ('auth_refresh_token_families'),
 ('auth_refresh_tokens'),
 ('auth_webauthn_ceremonies'),
 ('auth_webauthn_ceremony_bindings'),
 ('auth_webauthn_credentials'),
 ('company_actors'),
 ('account_context_candidates')
), relations AS (
 SELECT w.name, c.* FROM wanted w
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=w.name
), relation_shapes AS (
 SELECT r.name, jsonb_build_object(
  'relation',jsonb_build_array(r.relkind,r.relpersistence,r.relrowsecurity,r.relforcerowsecurity,r.relispartition,r.relreplident,r.reloptions),
  'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,co.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
    FROM pg_attribute a LEFT JOIN pg_type t ON t.oid=a.atttypid LEFT JOIN pg_namespace tn ON tn.oid=t.typnamespace
    LEFT JOIN pg_collation co ON co.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=co.collnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum
    WHERE a.attrelid=r.oid AND a.attnum>0),
  'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
    FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=r.oid),
  'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
    FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=r.oid),
  'triggers',(SELECT jsonb_agg(item ORDER BY item::text COLLATE "C") FROM (
    SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,CASE WHEN t.tgisinternal THEN t.tgqual::text ELSE pg_get_triggerdef(t.oid) END) AS item
    FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
    LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
    WHERE t.tgrelid=r.oid) items),
  'rules',(SELECT jsonb_agg(pg_get_ruledef(x.oid) ORDER BY x.rulename) FROM pg_rewrite x WHERE x.ev_class=r.oid),
  'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=r.oid),
  'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=r.oid OR i.inhparent=r.oid)
 ) AS shape FROM relations r
), relation_records AS (
 SELECT jsonb_build_object('name',r.name,'owner',pg_get_userbyid(r.relowner),'shape',s.shape,
  'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
   ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
   FROM aclexplode(COALESCE(r.relacl,acldefault('r',r.relowner))) a),'[]'::jsonb),
  'column_security',(SELECT jsonb_agg(jsonb_build_object('number',a.attnum,'name',a.attname,'acl_is_null',a.attacl IS NULL,
    'acl',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type,x.is_grantable)
       ORDER BY pg_get_userbyid(x.grantor),CASE WHEN x.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END,x.privilege_type) FROM aclexplode(a.attacl) x)) ORDER BY a.attnum)
    FROM pg_attribute a WHERE a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped),
  'policies',COALESCE((SELECT jsonb_agg(jsonb_build_object('name',p.polname,'permissive',p.polpermissive,'command',p.polcmd,
   'roles',(SELECT jsonb_agg(CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END ORDER BY CASE WHEN role_oid=0 THEN 'PUBLIC' ELSE pg_get_userbyid(role_oid) END) FROM unnest(p.polroles) role_oid),
   'using',pg_get_expr(p.polqual,p.polrelid),'check',pg_get_expr(p.polwithcheck,p.polrelid)) ORDER BY p.polname)
   FROM pg_policy p WHERE p.polrelid=r.oid),'[]'::jsonb)) AS record
 FROM relations r JOIN relation_shapes s ON s.name=r.name
), routine_records AS (
 SELECT p.oid,p.proowner,jsonb_build_object('schema',n.nspname,'name',p.proname,'identity_arguments',pg_get_function_identity_arguments(p.oid),
   'result',pg_get_function_result(p.oid),'owner',pg_get_userbyid(p.proowner),'language',l.lanname,
   'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,'returns_set',p.proretset,
   'leakproof',p.proleakproof,'volatility',p.provolatile,'parallel',p.proparallel,
   'support',CASE WHEN p.prosupport=0 THEN NULL ELSE p.prosupport::regprocedure::text END,
   'config',p.proconfig,'argnames',p.proargnames,'argmodes',p.proargmodes,
   'argdefaults',pg_get_expr(p.proargdefaults,0),'binary',p.probin,'cost',p.procost,'rows',p.prorows,
   'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
   'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type,a.is_grantable)
    ORDER BY pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,a.privilege_type)
    FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb)) AS record,
   p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL AND p.prosqlbody IS NULL AND p.protrftypes IS NULL AS extra_valid
 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_language l ON l.oid=p.prolang
 WHERE p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ('account_login_consent_v1','auth_account_refresh_reuse_revoke_v1','account_session_refresh_reuse_v1','auth_legacy_audit_append_v1','auth_legacy_bootstrap_issue_v1','auth_legacy_bootstrap_seed_v1','auth_legacy_cold_start_admin_v1','auth_legacy_company_lock_v1','auth_legacy_deactivate_credentials_v1','auth_legacy_group_passkey_flag_v1','auth_legacy_purge_company_v1','auth_legacy_purge_subjects_v1','auth_legacy_reset_credentials_v1','auth_legacy_self_bootstrap_replace_v1','auth_legacy_self_passkey_count_v1','auth_legacy_self_passkey_delete_v1','auth_legacy_self_passkey_state_v1','auth_legacy_self_passkeys_v1','auth_legacy_session_context_v1','auth_legacy_user_active_v1','auth_legacy_user_has_passkey_v1','enforce_org_id_immutable','platform_force_remove_direct_org_children','platform_force_remove_organization','platform_list_group_accounts','platform_remove_organization','platform_resolve_bootstrap_org','platform_resolve_credential_org','platform_resolve_token_org'))
), owner_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')
), protected_roles AS (
 SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt')
), related_fks AS (
 SELECT k.* FROM pg_constraint k WHERE k.contype='f'
   AND (k.conrelid IN (SELECT oid FROM relations) OR k.confrelid IN (SELECT oid FROM relations))
), foreign_key_records AS (
 SELECT jsonb_build_object('schema',ns.nspname,'name',k.conname,
   'source',k.conrelid::regclass::text,'target',k.confrelid::regclass::text,
   'namespace',kn.nspname,'type',k.contypid::regtype::text,
   'parent',CASE WHEN k.conparentid=0 THEN NULL ELSE parent.conname END,
   'validated',k.convalidated,'enforced',k.conenforced,'period',k.conperiod,
   'local',k.conislocal,'inherited',k.coninhcount,'noinherit',k.connoinherit,
   'deferrable',k.condeferrable,'deferred',k.condeferred,
   'keys',k.conkey,'foreign_keys',k.confkey,'update',k.confupdtype,
   'delete',k.confdeltype,'match',k.confmatchtype,'delete_columns',k.confdelsetcols,
   'binary_expression',pg_get_expr(k.conbin,k.conrelid),
   'primary_foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conpfeqop) WITH ORDINALITY a(op,ordinal)),
   'primary_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conppeqop) WITH ORDINALITY a(op,ordinal)),
   'foreign_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conffeqop) WITH ORDINALITY a(op,ordinal)),
   'exclusion_operators',(SELECT jsonb_agg(op::regoperator::text ORDER BY ordinal) FROM unnest(k.conexclop) WITH ORDINALITY a(op,ordinal)),
   'supporting_index',k.conindid::regclass::text,
   'supporting_index_definition',pg_get_indexdef(k.conindid),
   'index_flags',jsonb_build_array(i.indrelid=k.confrelid,i.indisunique,i.indisvalid,i.indisready,i.indislive,
       i.indimmediate,i.indisexclusion,i.indnkeyatts,i.indnatts,i.indkey::text,
       pg_get_expr(i.indexprs,i.indrelid),pg_get_expr(i.indpred,i.indrelid)),
   'ri',(SELECT jsonb_agg(jsonb_build_object(
       'on',t.tgrelid::regclass::text,'other',t.tgconstrrelid::regclass::text,
       'index',t.tgconstrindid::regclass::text,'function',t.tgfoid::regprocedure::text,
       'name_valid',t.tgname::text ~ '^RI_ConstraintTrigger_[ac]_[0-9]+$',
       'internal',t.tgisinternal,'enabled',t.tgenabled,'type',t.tgtype,
       'arguments',t.tgnargs,'args',encode(t.tgargs,'hex'),'attributes',t.tgattr::text,
       'deferrable',t.tgdeferrable,'deferred',t.tginitdeferred,
       'parent_present',t.tgparentid<>0,'old_table',t.tgoldtable,'new_table',t.tgnewtable,
       'condition',pg_get_expr(t.tgqual,t.tgrelid))
       ORDER BY t.tgrelid::regclass::text,t.tgfoid::regprocedure::text)
     FROM pg_trigger t WHERE t.tgconstraint=k.oid)) AS record
 FROM related_fks k JOIN pg_class c ON c.oid=k.conrelid
 JOIN pg_namespace ns ON ns.oid=c.relnamespace
 JOIN pg_namespace kn ON kn.oid=k.connamespace
 LEFT JOIN pg_constraint parent ON parent.oid=k.conparentid
 LEFT JOIN pg_index i ON i.indexrelid=k.conindid
), legacy_root_boundary AS (
-- Read-only complete custody verdict, including the legacy user root bridge.
-- Caller must use search_path=pg_catalog,pg_temp. No Account/user rows are read.
-- Historical shapes are fixed from reviewed0226. The only projected-out root
-- objects are independently certified below; body hashes derive from source.
WITH auth7_expected(relation_name,column_name,constraint_name) AS (VALUES
 ('auth_bootstrap_credentials','user_id','auth_bootstrap_credentials_account_v1'),
 ('auth_refresh_token_families','user_id','auth_refresh_token_families_account_v1'),
 ('auth_refresh_tokens','user_id','auth_refresh_tokens_account_v1'),
 ('auth_webauthn_ceremonies','user_id','auth_webauthn_ceremonies_account_v1'),
 ('auth_webauthn_credentials','user_id','auth_webauthn_credentials_account_v1'),
 ('auth_device_login_handoffs','target_user_id','auth_device_login_handoffs_target_account_v1'),
 ('auth_device_login_handoffs','approved_user_id','auth_device_login_handoffs_approved_account_v1')
), auth7_keys AS (
 SELECT e.*, c.oid AS source_oid,a.oid AS account_oid,ak.conindid AS account_index_oid,
   ca.attnum AS source_attnum,aa.attnum AS account_attnum,
   COALESCE(c.relkind='r' AND c.relpersistence='p' AND NOT c.relispartition
     AND a.relkind='r' AND a.relpersistence='p' AND NOT a.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i
       WHERE i.inhrelid IN(c.oid,a.oid) OR i.inhparent IN(c.oid,a.oid))
     AND ca.atttypid='pg_catalog.uuid'::regtype AND NOT ca.attisdropped AND ca.attnum>0
     AND aa.atttypid='pg_catalog.uuid'::regtype AND aa.attnotnull AND NOT aa.attisdropped AND aa.attnum>0
     AND ak.conkey=ARRAY[aa.attnum]::smallint[] AND ak.convalidated AND ak.conenforced
     AND NOT ak.condeferrable AND NOT ak.condeferred
     AND ai.indisprimary AND ai.indisunique AND ai.indisvalid AND ai.indisready
     AND ai.indislive AND ai.indimmediate AND ai.indexprs IS NULL AND ai.indpred IS NULL,false) AS valid
 FROM auth7_expected e
 LEFT JOIN pg_class c ON c.oid=to_regclass('public.'||e.relation_name)
 LEFT JOIN pg_class a ON a.oid=to_regclass('public.accounts')
 LEFT JOIN pg_attribute ca ON ca.attrelid=c.oid AND ca.attname=e.column_name
 LEFT JOIN pg_attribute aa ON aa.attrelid=a.oid AND aa.attname='id'
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
 LEFT JOIN pg_index ai ON ai.indexrelid=ak.conindid
), auth7_fk AS (
 SELECT k.*,f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.source_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.source_attnum]::smallint[] AND f.confkey=ARRAY[k.account_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND NOT f.condeferrable AND NOT f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND (SELECT count(*)=1 FROM pg_constraint other WHERE other.conname=k.constraint_name),false) AS valid_fk
 FROM auth7_keys k LEFT JOIN pg_constraint f
   ON f.conrelid=k.source_oid AND f.conname=k.constraint_name
), auth7_ri_expected(function_name,on_source,trigger_type) AS (VALUES
 ('RI_FKey_check_ins',true,5),('RI_FKey_check_upd',true,17),
 ('RI_FKey_restrict_del',false,9),('RI_FKey_restrict_upd',false,17)
), auth7_ri AS (
 SELECT f.fk_oid,e.function_name,(SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_source THEN f.source_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_source THEN f.account_oid ELSE f.source_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgname::text ~ CASE WHEN e.on_source THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM auth7_fk f CROSS JOIN auth7_ri_expected e
), auth7_root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_constraint c JOIN auth7_expected e ON c.conname=e.constraint_name) AS present,
   COALESCE((SELECT count(*)=7 AND bool_and(valid_fk) FROM auth7_fk)
     -- Count every incoming Account FK on these six credential relations, even
     -- an extra named FK whose Account-side RI triggers were removed by drift.
     AND (SELECT count(*)=7 FROM pg_constraint c
       WHERE c.contype='f' AND c.confrelid=to_regclass('public.accounts')
         AND c.conrelid IN (SELECT source_oid FROM auth7_keys))
     AND (SELECT count(*)=28 AND bool_and(valid) FROM auth7_ri)
     AND (SELECT count(*)=28 FROM pg_trigger t JOIN auth7_fk f ON f.fk_oid=t.tgconstraint),false) AS valid
),
 expected(name, owner_name, shape_sha256) AS (VALUES
 ('accounts','console_account_owner','bf8b3a765aca8473b0bdcb977a3c2adbb2c1fe0dd775cc151faae1271427d3f9'),
 ('account_security','console_account_owner','6d97077ecd0b70761f3ac9396e862bb3906f0f3f20b9927356f3127da607bd25'),
 ('account_security_events','console_account_owner','4ede3fbfc37609d90f0288d26192baa8cb2f2893052233483767f91d90545e8d'),
 ('account_terms_acceptances','console_account_owner','98dc2c0ee2e6179f7cf901cba1907228f800132e3dee0a9b2981657c67973a5a'),
 ('account_terms_head','console_terms_owner','ab06ca878b3c1dea752eb53306311a3317ec2edb4df2e9178a09a16858b327e5'),
 ('account_terms_release_receipts','console_terms_owner','bbfff3cb2895d8adf363bf83db2f3838cc0ab58091d1da775812ad7c3e2ec356')), observed AS (
SELECT wanted.name, jsonb_build_object(
 'relation',jsonb_build_array(c.relkind,c.relpersistence,c.relrowsecurity,c.relforcerowsecurity,c.relispartition,c.relreplident,c.reloptions),
 'columns',(SELECT jsonb_agg(jsonb_build_array(a.attnum,a.attname,tn.nspname,t.typname,a.atttypmod,a.attnotnull,a.attisdropped,a.attidentity,a.attgenerated,cn.nspname,coll.collname,pg_get_expr(d.adbin,d.adrelid)) ORDER BY a.attnum)
 FROM pg_attribute a JOIN pg_type t ON t.oid=a.atttypid JOIN pg_namespace tn ON tn.oid=t.typnamespace
 LEFT JOIN pg_collation coll ON coll.oid=a.attcollation LEFT JOIN pg_namespace cn ON cn.oid=coll.collnamespace
 LEFT JOIN pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE a.attrelid=c.oid AND a.attnum>0),
 'constraints',(SELECT jsonb_agg(jsonb_build_array(k.conname,k.contype,k.convalidated,k.condeferrable,k.condeferred,k.connoinherit,k.conislocal,k.coninhcount,k.conparentid=0,k.conkey,k.confkey,k.confupdtype,k.confdeltype,k.confmatchtype,fn.nspname,f.relname,pg_get_constraintdef(k.oid)) ORDER BY k.conname)
 FROM pg_constraint k LEFT JOIN pg_class f ON f.oid=k.confrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace WHERE k.conrelid=c.oid),
 'indexes',(SELECT jsonb_agg(jsonb_build_array(ic.relname,i.indisvalid,i.indisready,i.indislive,i.indimmediate,i.indisunique,i.indisexclusion,i.indisprimary,i.indnullsnotdistinct,pg_get_indexdef(i.indexrelid)) ORDER BY ic.relname)
 FROM pg_index i JOIN pg_class ic ON ic.oid=i.indexrelid WHERE i.indrelid=c.oid),
 'triggers',(SELECT jsonb_agg(item ORDER BY item::text) FROM (
 SELECT jsonb_build_array(CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgisinternal,t.tgenabled,t.tgtype,t.tgnargs,encode(t.tgargs,'hex'),t.tgdeferrable,t.tginitdeferred,pn.nspname,p.proname,fn.nspname,f.relname,pg_get_expr(t.tgqual,t.tgrelid)) AS item
 FROM pg_trigger t JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace pn ON pn.oid=p.pronamespace
 LEFT JOIN pg_class f ON f.oid=t.tgconstrrelid LEFT JOIN pg_namespace fn ON fn.oid=f.relnamespace
 WHERE t.tgrelid=c.oid AND NOT (wanted.name='accounts' AND (
   t.tgname='account_roots_immutable_v1' OR t.tgconstraint IN (
     SELECT root_key.oid FROM pg_constraint root_key
     WHERE root_key.conrelid=to_regclass('public.users') AND root_key.conname='users_account_root_v1')
   OR ((SELECT valid FROM auth7_root_profile) AND t.tgconstraint IN (SELECT fk_oid FROM auth7_fk))))) triggers),
 'rules',(SELECT jsonb_agg(pg_get_ruledef(r.oid) ORDER BY r.rulename) FROM pg_rewrite r WHERE r.ev_class=c.oid),
 'policies',(SELECT count(*) FROM pg_policy p WHERE p.polrelid=c.oid),
 'inheritance',(SELECT count(*) FROM pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)
)::text AS shape
FROM expected wanted LEFT JOIN pg_namespace n ON n.nspname='public'
LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=wanted.name
ORDER BY wanted.name), relations AS (
 SELECT e.*, c.oid, c.relowner, c.relacl, r.rolname AS actual_owner,
        encode(sha256(convert_to(o.shape,'UTF8')),'hex') AS actual_shape
 FROM expected e LEFT JOIN observed o ON o.name=e.name
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=e.name
 LEFT JOIN pg_roles r ON r.oid=c.relowner
), routine_bodies(name,sha256) AS (VALUES
 ('account_legacy_fenced_v1','0ea5ca5ecadcdef895d525dfc552fd35dfda06add0705b3ef202f4099debd8d9'),
 ('account_terms_receipts_immutable_v1','dac65dd11a1031196794f94f445205aad1ed804c09e0326c896b94af7d991b7c'),
 ('account_terms_current_v1','e39c2c73c35b1be6ca7379b08c684879ab831df369f264ec63552490057563ec'),
 ('account_roots_immutable_v1','0ccca6c1b15d5ad3f95f25b8ef88db47f11622a89699326908a7a957fa5fe7fa'),
 ('account_legacy_user_root_v1','2d0643734b149d32b7f81ce052746b2c414ab64439161fc3d64214c680299f31'),
 ('account_legacy_user_id_immutable_v1','77f85eea3c295aae356a4a3aa9925d1e2a2a8d7696cbfeedaa6f706882422b56'),
 ('account_company_deactivation_guard_v1','07deace275ef849889d86de62c08bce171974d35d713d67fad8a1fc1899543cf')
), root_names(name, relation_name, trigger_name, trigger_type, definer) AS (VALUES
 ('account_roots_immutable_v1','accounts','account_roots_immutable_v1',58,false),
 ('account_legacy_user_root_v1','users','00_account_legacy_user_root_v1',5,true),
 ('account_legacy_user_id_immutable_v1','users','00_account_legacy_user_id_immutable_v1',17,false)
), root_functions AS (
 SELECT e.*, p.oid, p.proowner,
   (SELECT count(*)=1 FROM pg_proc candidate JOIN pg_namespace n ON n.oid=candidate.pronamespace
     WHERE n.nspname='public' AND candidate.proname=e.name) AND COALESCE(
     owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef=e.definer AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name=e.name)
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0,false) AS valid
 FROM root_names e
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_proc p ON p.pronamespace=n.oid AND p.proname=e.name
   AND p.pronargs=0 AND p.proargtypes=''::oidvector
 LEFT JOIN pg_roles owner_role ON owner_role.oid=p.proowner
 LEFT JOIN pg_language language ON language.oid=p.prolang
), root_triggers AS (
 SELECT f.name, (SELECT count(*)=1 AND bool_and(
     t.tgname=f.trigger_name AND t.tgrelid=to_regclass('public.'||f.relation_name)
     AND t.tgfoid=f.oid AND NOT t.tgisinternal AND t.tgenabled='A' AND t.tgtype=f.trigger_type
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
     AND t.tgconstrrelid=0 AND t.tgconstrindid=0
     AND NOT t.tgdeferrable AND NOT t.tginitdeferred
     AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgname=f.trigger_name OR t.tgfoid=f.oid) AS valid
 FROM root_functions f
), root_user_key AS (
 -- Certify only the native key boundary, not users' legacy columns, ACLs,
 -- policies or unrelated triggers. Inheritance would evade this parent FK.
 SELECT u.oid AS user_oid, a.oid AS account_oid, uk.oid AS user_key_oid,
   ak.conindid AS account_index_oid, uid.attnum AS user_id_attnum, aid.attnum AS account_id_attnum,
   COALESCE(u.relkind='r' AND NOT u.relispartition
     AND NOT EXISTS(SELECT 1 FROM pg_inherits i WHERE i.inhrelid=u.oid OR i.inhparent=u.oid)
     AND uid.atttypid='pg_catalog.uuid'::regtype AND uid.attnotnull AND NOT uid.attisdropped
     AND uk.contype='p' AND uk.conkey=ARRAY[uid.attnum]::smallint[]
     AND uk.convalidated AND NOT uk.condeferrable AND NOT uk.condeferred
     AND ui.indisprimary AND ui.indisunique AND ui.indisvalid AND ui.indisready
     AND ui.indislive AND ui.indimmediate AND ui.indexprs IS NULL AND ui.indpred IS NULL
     AND ak.contype='p' AND ak.conkey=ARRAY[aid.attnum]::smallint[]
     AND ak.convalidated AND NOT ak.condeferrable AND NOT ak.condeferred,false) AS valid
 FROM (SELECT to_regclass('public.users') AS user_oid,to_regclass('public.accounts') AS account_oid) names
 LEFT JOIN pg_class u ON u.oid=names.user_oid
 LEFT JOIN pg_class a ON a.oid=names.account_oid
 LEFT JOIN pg_attribute uid ON uid.attrelid=u.oid AND uid.attname='id'
 LEFT JOIN pg_attribute aid ON aid.attrelid=a.oid AND aid.attname='id'
 LEFT JOIN pg_constraint uk ON uk.conrelid=u.oid AND uk.contype='p'
 LEFT JOIN pg_index ui ON ui.indexrelid=uk.conindid
 LEFT JOIN pg_constraint ak ON ak.conrelid=a.oid AND ak.contype='p'
), root_fk AS (
 SELECT k.*, f.oid AS fk_oid,
   COALESCE(k.valid AND f.contype='f' AND f.connamespace='public'::regnamespace
     AND f.conrelid=k.user_oid AND f.confrelid=k.account_oid AND f.contypid=0
     AND f.conkey=ARRAY[k.user_id_attnum]::smallint[] AND f.confkey=ARRAY[k.account_id_attnum]::smallint[]
     AND f.conindid=k.account_index_oid AND f.convalidated AND f.conenforced
     AND f.condeferrable AND f.condeferred AND f.connoinherit
     AND f.conislocal AND f.coninhcount=0 AND f.conparentid=0 AND NOT f.conperiod
     AND f.confupdtype='r' AND f.confdeltype='r' AND f.confmatchtype='s'
     AND f.confdelsetcols IS NULL AND f.conbin IS NULL AND f.conexclop IS NULL
     AND f.conpfeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conppeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid]
     AND f.conffeqop=ARRAY['pg_catalog.=(uuid,uuid)'::regoperator::oid],false) AS valid_fk
 FROM root_user_key k LEFT JOIN pg_constraint f
   ON f.conrelid=k.user_oid AND f.conname='users_account_root_v1'
), root_ri_expected(function_name, on_users, trigger_type, deferred) AS (VALUES
 ('RI_FKey_check_ins',true,5,true),('RI_FKey_check_upd',true,17,true),
 ('RI_FKey_restrict_del',false,9,false),('RI_FKey_restrict_upd',false,17,false)
), root_ri_triggers AS (
 -- Every field projected out of the historical accounts fingerprint is
 -- independently bound here, including native function identity and timing.
 SELECT e.function_name, (SELECT count(*)=1 AND bool_and(
     t.tgrelid=CASE WHEN e.on_users THEN f.user_oid ELSE f.account_oid END
     AND t.tgconstrrelid=CASE WHEN e.on_users THEN f.account_oid ELSE f.user_oid END
     AND t.tgconstrindid=f.account_index_oid AND t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')
     AND t.tgisinternal AND t.tgenabled='O' AND t.tgtype=e.trigger_type
     AND t.tgdeferrable=e.deferred AND t.tginitdeferred=e.deferred
     AND t.tgname::text ~ CASE WHEN e.on_users THEN '^RI_ConstraintTrigger_c_[0-9]+$' ELSE '^RI_ConstraintTrigger_a_[0-9]+$' END
     AND t.tgname::text COLLATE "C">'00_account_legacy_user_root_v1' COLLATE "C"
     AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
     AND t.tgqual IS NULL AND t.tgparentid=0 AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL)
   FROM pg_trigger t WHERE t.tgconstraint=f.fk_oid
     AND t.tgfoid=to_regprocedure('pg_catalog.'||quote_ident(e.function_name)||'()')) AS valid
 FROM root_ri_expected e CROSS JOIN root_fk f
), root_insert_acl AS (
 -- INSERT is the only new ordinary right. Old SELECT/UPDATE drift keeps the
 -- existing custody diagnostics and is checked by the old ACL profile below.
 SELECT EXISTS(SELECT 1 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
     OR EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
       WHERE c.name='accounts' AND x.privilege_type='INSERT') AS present,
   (SELECT count(*)=2 AND count(DISTINCT a.attname)=2 AND bool_and(
       a.attname IN ('id','created_at') AND x.grantor=c.relowner AND x.grantee=c.relowner
       AND NOT x.is_grantable)
     FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid
     CROSS JOIN LATERAL aclexplode(a.attacl) x WHERE c.name='accounts' AND x.privilege_type='INSERT')
   AND NOT EXISTS(SELECT 1 FROM relations c CROSS JOIN LATERAL aclexplode(c.relacl) x
     WHERE c.name='accounts' AND x.privilege_type='INSERT') AS valid
), root_profile AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname IN (SELECT name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_trigger t WHERE t.tgname IN (SELECT trigger_name FROM root_names))
     OR EXISTS(SELECT 1 FROM pg_constraint f WHERE f.conrelid=to_regclass('public.users') AND f.conname='users_account_root_v1')
     OR (SELECT present FROM root_insert_acl) AS present,
   COALESCE((SELECT count(*)=3 AND bool_and(valid) FROM root_functions)
     AND (SELECT count(*)=3 AND bool_and(valid) FROM root_triggers)
     AND (SELECT count(*)=1 AND bool_and(valid_fk) FROM root_fk)
     AND (SELECT count(*)=4 AND bool_and(valid) FROM root_ri_triggers)
     AND (SELECT count(*)=4 FROM pg_trigger t JOIN root_fk f ON f.fk_oid=t.tgconstraint),false) AS valid

), projection AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1') AND EXISTS (
        SELECT p.oid FROM pg_catalog.pg_proc p
        JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
        JOIN pg_catalog.pg_roles owner_role ON owner_role.oid=p.proowner
        JOIN pg_catalog.pg_language language ON language.oid=p.prolang
        WHERE n.nspname='public' AND p.proname='account_legacy_fenced_v1'
          AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
          AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
          AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
          AND p.pronargs=1 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector
          AND p.proargnames=ARRAY['subject_account_id']::text[]
          AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
          AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
          AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
          AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
          AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_legacy_fenced_v1')
          AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp','row_security=off']::text[]
          AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
                AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(a.grantor=p.proowner
                AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
                AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt'))
                  OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
                    (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_app')))),false))
               FROM pg_catalog.aclexplode(COALESCE(p.proacl,pg_catalog.acldefault('f',p.proowner))) a)
 ) AS valid
), deactivation_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_company_deactivation_guard_v1'
     AND owner_role.rolname='console_account_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=2 AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector
     AND p.proargnames=ARRAY['company_id','subject_id']::text[]
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.bool'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND p.procost=100 AND p.prorows=0
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_company_deactivation_guard_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
     AND (SELECT count(*)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END
       AND count(DISTINCT a.grantee)=CASE WHEN (SELECT valid FROM auth7_root_profile) THEN 4 ELSE 2 END AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE((a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_rt'))
         OR ((SELECT valid FROM auth7_root_profile) AND a.grantee IN
           (SELECT oid FROM pg_roles WHERE rolname IN ('console_credential_owner','console_auth_rt')))),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), deactivation_users AS (
 SELECT c.oid,c.relowner,c.relacl,r.oid AS guard_owner
 FROM pg_class c CROSS JOIN pg_roles r
 WHERE c.oid=to_regclass('public.users') AND r.rolname='console_account_owner'
), deactivation_user_grants AS (
 -- Other Company grants remain outside this extension. PUBLIC would widen
 -- the definer's effective rights and is never a valid guard grant.
 SELECT a.attname,x.* FROM deactivation_users c
 JOIN pg_attribute a ON a.attrelid=c.oid
 CROSS JOIN LATERAL aclexplode(a.attacl) x
 WHERE x.grantee IN (0,c.guard_owner)
), deactivation_users_acl AS (
 SELECT COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND NOT has_any_column_privilege(c.guard_owner,c.oid,'SELECT,INSERT,UPDATE,REFERENCES')
     AND NOT EXISTS(SELECT 1 FROM deactivation_user_grants),false) AS dormant,
   COALESCE(NOT has_table_privilege(c.guard_owner,c.oid,
       'SELECT,INSERT,UPDATE,DELETE,TRUNCATE,REFERENCES,TRIGGER,MAINTAIN')
     AND NOT EXISTS(SELECT 1 FROM aclexplode(COALESCE(c.relacl,acldefault('r',c.relowner))) x
       WHERE x.grantee IN (0,c.guard_owner))
     AND (SELECT count(*)=3 AND count(DISTINCT (x.attname,x.privilege_type))=3
       AND bool_and(x.grantor=c.relowner AND x.grantee=c.guard_owner AND NOT x.is_grantable
         AND ((x.privilege_type='SELECT' AND x.attname IN ('id','org_id'))
           OR (x.privilege_type='UPDATE' AND x.attname='id')))
       FROM deactivation_user_grants x)
     AND (SELECT bool_and(
       has_column_privilege(c.guard_owner,c.oid,a.attname,'SELECT')=(a.attname IN ('id','org_id'))
       AND has_column_privilege(c.guard_owner,c.oid,a.attname,'UPDATE')=(a.attname='id')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,'INSERT,REFERENCES')
       AND NOT has_column_privilege(c.guard_owner,c.oid,a.attname,
         'SELECT WITH GRANT OPTION,INSERT WITH GRANT OPTION,UPDATE WITH GRANT OPTION,REFERENCES WITH GRANT OPTION'))
       FROM pg_attribute a WHERE a.attrelid=c.oid AND a.attnum>0 AND NOT a.attisdropped),false) AS valid
 FROM deactivation_users c
), receipt_guard AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='plpgsql'
     AND p.prokind='f' AND NOT p.prosecdef AND NOT p.proisstrict AND NOT p.proretset
     AND NOT p.proleakproof AND p.provolatile='v' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector AND p.proargnames IS NULL
     AND p.proallargtypes IS NULL AND p.proargmodes IS NULL AND p.provariadic=0
     AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.trigger'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_receipts_immutable_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=0
 ) AS valid
), terms_current AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AS present,
 (SELECT count(*)=1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1') AND EXISTS (
   SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
   JOIN pg_roles owner_role ON owner_role.oid=p.proowner
   JOIN pg_language language ON language.oid=p.prolang
   WHERE n.nspname='public' AND p.proname='account_terms_current_v1'
     AND owner_role.rolname='console_terms_owner' AND language.lanname='sql'
     AND p.prokind='f' AND p.prosecdef AND NOT p.proisstrict AND p.proretset
     AND NOT p.proleakproof AND p.provolatile='s' AND p.proparallel='u' AND p.prosupport=0
     AND p.pronargs=0 AND p.proargtypes=''::oidvector
     AND p.proallargtypes=ARRAY['pg_catalog.bytea'::regtype::oid,'pg_catalog.int8'::regtype::oid]
     AND p.proargmodes=ARRAY['t','t']::"char"[]
     AND p.proargnames=ARRAY['manifest_sha256','revision']::text[]
     AND p.provariadic=0 AND p.pronargdefaults=0 AND p.proargdefaults IS NULL
     AND p.prorettype='pg_catalog.record'::regtype AND p.probin IS NULL
     AND p.prosqlbody IS NULL AND p.protrftypes IS NULL
     AND encode(sha256(convert_to(p.prosrc,'UTF8')),'hex')=(SELECT sha256 FROM routine_bodies WHERE name='account_terms_current_v1')
     AND p.proconfig=ARRAY['search_path=pg_catalog, pg_temp']::text[]
     AND p.proacl IS NOT NULL AND cardinality(p.proacl)=2
     AND (SELECT count(*)=2 AND count(DISTINCT a.grantee)=2 AND bool_and(
       a.grantor=p.proowner AND a.privilege_type='EXECUTE' AND NOT a.is_grantable
       AND COALESCE(a.grantee IN (p.proowner,(SELECT oid FROM pg_roles WHERE rolname='console_auth_rt')),false))
       FROM aclexplode(p.proacl) a)
 ) AS valid
), guard_trigger AS (
 -- Pin fields outside the relation fingerprint too. Function OIDs are resolved
 -- through the exact zero-argument routine, never learned as expected values.
 SELECT count(*)=1 AND bool_and(
   t.tgname='account_terms_receipts_immutable_v1' AND t.tgenabled='A' AND t.tgtype=58
   AND t.tgnargs=0 AND octet_length(t.tgargs)=0 AND t.tgattr=''::int2vector
   AND t.tgqual IS NULL AND t.tgconstraint=0 AND t.tgparentid=0
   AND t.tgconstrrelid=0 AND t.tgconstrindid=0
   AND NOT t.tgdeferrable AND NOT t.tginitdeferred
   AND t.tgoldtable IS NULL AND t.tgnewtable IS NULL
   AND n.nspname='public' AND p.proname='account_terms_receipts_immutable_v1'
   AND p.pronargs=0 AND p.proargtypes=''::oidvector) AS valid
 FROM pg_trigger t JOIN relations r ON r.oid=t.tgrelid
 JOIN pg_proc p ON p.oid=t.tgfoid JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE r.name='account_terms_release_receipts' AND NOT t.tgisinternal
), ownership AS (
 SELECT bool_and(actual_owner='console_app') AS pending,
        bool_and(actual_owner=owner_name) AS finalized FROM relations
), custody_columns AS (
 -- Normalize only the added INSERT bits when checking the retained historical
 -- SELECT/UPDATE contracts. Their original grantor/grantee/options still fail
 -- under the original diagnostics, even if a corrupt ACL also loses INSERT.
 SELECT a.attrelid,a.attname,CASE WHEN c.name='accounts' AND (SELECT present FROM root_profile) THEN
   (SELECT array_agg(makeaclitem(x.grantee,x.grantor,x.privileges,x.is_grantable)
       ORDER BY x.grantee,x.grantor,x.is_grantable)
     FROM (SELECT acl.grantee,acl.grantor,acl.is_grantable,
       string_agg(acl.privilege_type,',' ORDER BY acl.privilege_type) AS privileges
       FROM aclexplode(a.attacl) acl WHERE acl.privilege_type<>'INSERT'
       GROUP BY acl.grantee,acl.grantor,acl.is_grantable) x)
   ELSE a.attacl END AS attacl
 FROM relations c JOIN pg_attribute a ON a.attrelid=c.oid

), column_acl_profiles AS (
 SELECT bool_and(COALESCE(cardinality(a.attacl),0)=0) AS dormant,
        bool_and(CASE WHEN c.name='accounts' AND a.attname='id' THEN
          COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
            acl.grantor=c.relowner AND acl.grantee=c.relowner
            AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
            FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS prepared,
        -- Common guarded profile, with the entire head ACL checked separately.
        bool_and(CASE
          WHEN c.name='account_terms_head' THEN true
          WHEN c.name='account_terms_release_receipts' AND a.attname IN ('id','revision','manifest_sha256') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT
              count(*)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND count(DISTINCT acl.privilege_type)=CASE WHEN a.attname='id' THEN 2 ELSE 1 END
              AND bool_and(acl.grantor=c.relowner AND acl.grantee=c.relowner
                AND NOT acl.is_grantable AND (acl.privilege_type='SELECT'
                  OR (a.attname='id' AND acl.privilege_type='UPDATE')))
              FROM aclexplode(a.attacl) acl)
          WHEN c.name='accounts' AND a.attname='id' THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='UPDATE' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS guarded,
        bool_and(CASE WHEN c.name='account_terms_head'
          THEN COALESCE(cardinality(a.attacl),0)=0 ELSE true END) AS head_dormant,
        bool_and(CASE WHEN c.name<>'account_terms_head' THEN true
          WHEN a.attname IN ('id','manifest_sha256','revision') THEN
            COALESCE(cardinality(a.attacl),0)=1 AND (SELECT count(*)=1 AND bool_and(
              acl.grantor=c.relowner AND acl.grantee=c.relowner
              AND acl.privilege_type='SELECT' AND NOT acl.is_grantable)
              FROM aclexplode(a.attacl) acl)
          ELSE COALESCE(cardinality(a.attacl),0)=0 END) AS head_ready
 FROM relations c JOIN custody_columns a ON a.attrelid=c.oid
), table_acl_profiles AS (
 -- Profiles are collective: accepting either ACL independently per table would
 -- admit a partially installed projection. NULL table ACLs are never empty.
 SELECT bool_and(relacl IS NOT NULL AND cardinality(relacl)=0) AS dormant,
        bool_and(actual_owner=owner_name AND relacl IS NOT NULL AND
          CASE WHEN name IN ('accounts','account_security') THEN
            cardinality(relacl)=1 AND (SELECT count(*)=1 AND bool_and(
              a.grantor=c.relowner AND a.grantee=c.relowner
              AND a.privilege_type='SELECT' AND NOT a.is_grantable)
              FROM aclexplode(c.relacl) a)
          ELSE cardinality(relacl)=0 END) AS prepared
 FROM relations c
), acl_profiles AS (
 SELECT t.dormant AND c.dormant AS dormant,
        t.prepared AND c.prepared AS prepared,
        t.prepared AND c.guarded AND c.head_dormant AS guarded,
        t.prepared AND c.guarded AND c.head_ready AS ready
 FROM table_acl_profiles t CROSS JOIN column_acl_profiles c
)
SELECT jsonb_build_object('root_profile',(SELECT valid FROM root_profile),'projection',(SELECT valid FROM projection),'deactivation_guard',(SELECT valid FROM deactivation_guard),'deactivation_users_acl',(SELECT valid FROM deactivation_users_acl),'root_insert_acl',(SELECT valid FROM root_insert_acl),'receipt_guard',(SELECT valid FROM receipt_guard),'terms_current',(SELECT valid FROM terms_current),'auth7_root_profile',(SELECT valid FROM auth7_root_profile)) AS boundary
), snapshots AS (
 SELECT jsonb_build_object(
  'legacy_root_boundary',(SELECT boundary FROM legacy_root_boundary),
  'tables',(SELECT jsonb_agg(record ORDER BY record->>'name') FROM relation_records),
  'routines',(SELECT jsonb_agg(jsonb_build_object('metadata',record,'extra_valid',extra_valid)
      ORDER BY record->>'schema',record->>'name',record->>'identity_arguments') FROM routine_records),
  'foreign_keys',(SELECT jsonb_agg(record ORDER BY record->>'schema',record->>'source',record->>'name') FROM foreign_key_records),
  'constraint_flags',(SELECT jsonb_agg(jsonb_build_array(k.conrelid::regclass::text,k.conname,
      k.conenforced,k.conperiod,k.contypid=0,k.conparentid=0,k.coninhcount,k.conislocal)
      ORDER BY k.conrelid::regclass::text,k.conname)
    FROM pg_constraint k WHERE k.conrelid IN (SELECT oid FROM relations)),
  'trigger_links',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM (
    SELECT jsonb_build_array(t.tgrelid::regclass::text,
      CASE WHEN t.tgisinternal THEN NULL ELSE t.tgname END,t.tgfoid::regprocedure::text,
      t.tgparentid=0,t.tgattr::text,t.tgoldtable,t.tgnewtable,
      k.conname,CASE WHEN k.conrelid IS NULL THEN NULL ELSE k.conrelid::regclass::text END,
      CASE WHEN t.tgconstrindid=0 THEN NULL ELSE t.tgconstrindid::regclass::text END) AS record
    FROM pg_trigger t LEFT JOIN pg_constraint k ON k.oid=t.tgconstraint
    WHERE t.tgrelid IN (SELECT oid FROM relations)) triggers),
  'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,
      CASE WHEN rolname='console_auth_rt' THEN NULL ELSE rolcanlogin END,
      rolinherit,rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconfig) ORDER BY rolname) FROM protected_roles),
  'memberships',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
      pg_get_userbyid(m.grantor),m.admin_option,m.inherit_option,m.set_option)
      ORDER BY pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),pg_get_userbyid(m.grantor))
    FROM pg_auth_members m WHERE m.roleid IN (SELECT oid FROM protected_roles) OR m.member IN (SELECT oid FROM protected_roles)),
  'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner))
      ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
    WHERE c.relowner IN (SELECT oid FROM owner_roles) AND c.relkind IN ('r','p','v','m','S','f')),
  'owner_schemas',(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname)
    FROM pg_namespace n WHERE n.nspowner IN (SELECT oid FROM owner_roles)),
  'schema_create',(SELECT jsonb_agg(jsonb_build_array(r.rolname,n.nspname) ORDER BY r.rolname,n.nspname)
    FROM owner_roles r CROSS JOIN pg_namespace n
    -- The current temporary namespace derives CREATE from database TEMP, not
    -- persistent schema authority. Keep every other namespace/ACL check exact.
    WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(r.oid,n.oid,'CREATE')),
  'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
      pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
      a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_default_acl d LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace
    CROSS JOIN LATERAL aclexplode(d.defaclacl) a
    WHERE d.defaclrole IN (SELECT oid FROM protected_roles) OR a.grantee IN (SELECT oid FROM owner_roles)),
  'external_owner_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(c.relacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL)),
  'external_owner_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
      pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
      ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantee),a.privilege_type)
    FROM pg_attribute col JOIN pg_class c ON c.oid=col.attrelid JOIN pg_namespace n ON n.oid=c.relnamespace
    CROSS JOIN LATERAL aclexplode(col.attacl) a
    WHERE a.grantee IN (SELECT oid FROM owner_roles) AND c.oid NOT IN (SELECT oid FROM relations WHERE oid IS NOT NULL))
 ) AS snapshot
)
SELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256 FROM snapshots"""

NATIVE_INSTALL = r"""-- Fixed additive transition. The surrounding operator must certify exact
-- finalized Auth7 plus the two exact dormant catalogs under its held locks.
ALTER TABLE public.auth_refresh_token_families
    ADD COLUMN protocol text NOT NULL DEFAULT 'LEGACY_COMPANY',
    ADD COLUMN account_security_generation bigint,
    ADD COLUMN auth_time timestamptz,
    ADD COLUMN assurance text,
    ADD CONSTRAINT auth_refresh_families_native_tuple_v1 CHECK (COALESCE(
        (protocol='LEGACY_COMPANY' AND account_security_generation IS NULL
            AND auth_time IS NULL AND assurance IS NULL)
        OR (protocol='ACCOUNT_V1' AND org_id IS NULL AND account_security_generation>=1
            AND account_security_generation IS NOT NULL AND auth_time IS NOT NULL
            AND assurance='PASSKEY_PRIMARY'),false));
ALTER TABLE public.auth_webauthn_ceremonies
    ADD COLUMN account_browser_flow text,
    ADD COLUMN browser_nonce_sha256 bytea,
    ADD COLUMN browser_origin text,
    ADD COLUMN terms_manifest_sha256 bytea,
    ADD COLUMN terms_head_revision bigint,
    ADD CONSTRAINT auth_webauthn_native_tuple_v1 CHECK (COALESCE(
        (account_browser_flow IS NULL AND browser_nonce_sha256 IS NULL
            AND browser_origin IS NULL AND terms_manifest_sha256 IS NULL AND terms_head_revision IS NULL)
        OR (account_browser_flow='ACCOUNT_REGISTRATION' AND ceremony_kind='registration'
            AND user_id IS NOT NULL AND user_id<>'00000000-0000-0000-0000-000000000000'::uuid
            AND octet_length(browser_nonce_sha256)=32 AND octet_length(browser_origin) BETWEEN 1 AND 2048
            AND octet_length(terms_manifest_sha256)=32 AND terms_head_revision>=1)
        OR (account_browser_flow='ACCOUNT_LOGIN' AND ceremony_kind='authentication'
            AND octet_length(browser_nonce_sha256)=32 AND octet_length(browser_origin) BETWEEN 1 AND 2048
            AND terms_manifest_sha256 IS NULL AND terms_head_revision IS NULL),false));
CREATE UNIQUE INDEX auth_webauthn_native_nonce_v1
    ON public.auth_webauthn_ceremonies(account_browser_flow,browser_nonce_sha256)
    WHERE account_browser_flow IS NOT NULL;
CREATE INDEX auth_webauthn_native_expiry_v1
    ON public.auth_webauthn_ceremonies(expires_at,id) WHERE account_browser_flow IS NOT NULL;

ALTER TABLE public.company_actors ADD CONSTRAINT company_actors_account_v1
    FOREIGN KEY(account_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.account_context_candidates ADD CONSTRAINT account_context_candidates_account_v1
    FOREIGN KEY(account_id) REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.company_actors OWNER TO console_account_owner;
ALTER TABLE public.account_context_candidates OWNER TO console_account_owner;
-- Numbered migration already removed every default grant; retain no writers.
REVOKE ALL ON public.company_actors,public.account_context_candidates FROM PUBLIC,
    console_account_owner,console_app,console_rt,console_auth_rt,console_terms_owner,console_credential_owner;
GRANT SELECT ON public.account_context_candidates TO console_account_owner;

GRANT INSERT ON public.account_security,public.account_security_events,
    public.account_terms_acceptances TO console_account_owner;
GRANT UPDATE(security_state,revision,updated_at) ON public.account_security TO console_account_owner;
GRANT SELECT(id,account_id,kind,occurred_at,actor_account_id,session_id,evidence_ref,payload)
    ON public.account_security_events TO console_account_owner;
GRANT SELECT(account_id,terms_kind,terms_version,content_sha256,accepted_at,security_event_id,
    terms_release_receipt_id,terms_release_revision,terms_manifest_sha256)
    ON public.account_terms_acceptances TO console_account_owner;
GRANT SELECT(id,revision,manifest_sha256) ON public.account_terms_release_receipts TO console_account_owner;
GRANT SELECT(release_receipt_ref),UPDATE(id) ON public.account_terms_head TO console_terms_owner;

COMMENT ON COLUMN public.auth_refresh_token_families.protocol IS 'pd:personal — Account authentication protocol';
COMMENT ON COLUMN public.auth_refresh_token_families.account_security_generation IS 'pd:personal — Account authentication generation';
COMMENT ON COLUMN public.auth_refresh_token_families.auth_time IS 'pd:personal — Account primary authentication time';
COMMENT ON COLUMN public.auth_refresh_token_families.assurance IS 'pd:personal — Account primary authentication assurance';
COMMENT ON COLUMN public.auth_webauthn_ceremonies.account_browser_flow IS 'pd:personal — Account authentication flow';
COMMENT ON COLUMN public.auth_webauthn_ceremonies.browser_nonce_sha256 IS 'pd:sensitive — Account enrollment/login nonce digest';
COMMENT ON COLUMN public.auth_webauthn_ceremonies.browser_origin IS 'pd:personal — Account authentication origin';
COMMENT ON COLUMN public.auth_webauthn_ceremonies.terms_manifest_sha256 IS 'pd:personal — Account terms release binding';
COMMENT ON COLUMN public.auth_webauthn_ceremonies.terms_head_revision IS 'pd:personal — Account terms release revision';

-- SEPARATE CONDITIONAL PREREQUISITE, NOT PART OF THE SEVEN-HELPER CANDIDATE.
-- Root must reproduce the real Account-owner acceptance INSERT and determine
-- whether its FK check needs this narrow UPDATE privilege. Install/certify only
-- if the actual restricted-role experiment requires it. No blanket UPDATE.
-- Root serializes the fixed function, owner, empty EXECUTE ACL, ALWAYS statement
-- trigger, table/column ACL and body fingerprint into the native profile.
CREATE FUNCTION public.account_security_events_immutable_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
    RAISE EXCEPTION 'account.security_events_immutable';
END
$body$;
ALTER FUNCTION public.account_security_events_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_security_events_immutable_v1() FROM PUBLIC,console_account_owner;
CREATE TRIGGER account_security_events_immutable_v1
    BEFORE UPDATE OR DELETE OR TRUNCATE ON public.account_security_events
    FOR EACH STATEMENT EXECUTE FUNCTION public.account_security_events_immutable_v1();
ALTER TABLE public.account_security_events ENABLE ALWAYS TRIGGER account_security_events_immutable_v1;
GRANT UPDATE(id) ON public.account_security_events TO console_account_owner;

-- PRIVATE SOURCE CANDIDATE. Root owns installation, exact custody/profile
-- certification, grants and runtime verification. These routines require the
-- reviewed native additive schema. None is a stand-alone deployment script.

CREATE FUNCTION public.account_registration_begin_v1()
RETURNS TABLE(account_id uuid, security_generation bigint, revision bigint,
    context_generation bigint, created_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    allocated_id uuid;
    allocated_at timestamptz;
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    allocated_id := gen_random_uuid();
    allocated_at := clock_timestamp();
    INSERT INTO public.accounts(id,created_at) VALUES(allocated_id,allocated_at);
    INSERT INTO public.account_security(account_id,security_state,security_generation,
        revision,updated_at,context_generation)
        VALUES(allocated_id,'PENDING_ENROLLMENT',1,1,allocated_at,1);
    RETURN QUERY SELECT allocated_id,1::bigint,1::bigint,1::bigint,allocated_at;
END
$body$;

CREATE FUNCTION public.account_security_lock_exclusive_v1(p_account uuid)
RETURNS TABLE(security_state text, security_generation bigint, revision bigint,
    context_generation bigint)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
BEGIN
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    RETURN QUERY SELECT s.security_state,s.security_generation,s.revision,s.context_generation
        FROM public.account_security s WHERE s.account_id=p_account FOR UPDATE OF s;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002',MESSAGE='account.authority_unavailable';
    END IF;
END
$body$;

CREATE FUNCTION public.account_security_lock_shared_v1(p_account uuid)
RETURNS TABLE(security_state text, security_generation bigint, revision bigint,
    context_generation bigint)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
BEGIN
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    RETURN QUERY SELECT s.security_state,s.security_generation,s.revision,s.context_generation
        FROM public.account_security s WHERE s.account_id=p_account FOR SHARE OF s;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0002',MESSAGE='account.authority_unavailable';
    END IF;
END
$body$;

CREATE FUNCTION public.account_terms_registration_head_v1()
RETURNS TABLE(manifest_sha256 bytea, revision bigint, release_receipt_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
BEGIN
    IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    RETURN QUERY SELECT h.manifest_sha256,h.revision,h.release_receipt_ref
        FROM public.account_terms_head h
        JOIN public.account_terms_release_receipts r ON r.id=h.release_receipt_ref
          AND r.revision=h.revision AND r.manifest_sha256=h.manifest_sha256
        WHERE h.id=1 AND h.revision>=1 AND octet_length(h.manifest_sha256)=32
          AND h.release_receipt_ref<>'00000000-0000-0000-0000-000000000000'::uuid
        FOR SHARE OF h;
    IF NOT FOUND THEN RAISE EXCEPTION 'account.authority_unavailable'; END IF;
END
$body$;

CREATE FUNCTION public.auth_account_registration_material_v1(
    p_account uuid, p_ceremony uuid, p_key uuid, p_family uuid)
RETURNS TABLE(ceremony_created_at timestamptz, terms_manifest_sha256 bytea,
    terms_head_revision bigint, account_security_generation bigint,
    auth_time timestamptz, assurance text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    observed record;
    observed_at timestamptz;
BEGIN
    IF p_account IS NULL OR p_ceremony IS NULL OR p_key IS NULL OR p_family IS NULL
        OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(ARRAY[p_account,p_ceremony,p_key,p_family])
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.enrollment_invalid';
    END IF;
    observed_at := clock_timestamp();
    SELECT c.created_at AS ceremony_created_at,c.terms_manifest_sha256,c.terms_head_revision,
        c.expires_at AS ceremony_expires_at,t.expires_at AS token_expires_at,
        f.account_security_generation,f.auth_time,f.assurance
        INTO STRICT observed
        FROM public.auth_webauthn_ceremonies c
        JOIN public.auth_webauthn_credentials k ON k.id=p_key AND k.user_id=c.user_id
        JOIN public.auth_refresh_token_families f ON f.id=p_family AND f.user_id=c.user_id
        JOIN public.auth_refresh_tokens t ON t.family_id=f.id AND t.user_id=f.user_id
        WHERE c.id=p_ceremony AND c.user_id=p_account
          AND c.ceremony_kind='registration' AND c.account_browser_flow='ACCOUNT_REGISTRATION'
          AND c.consumed_at IS NOT NULL AND c.consumed_at>=c.created_at
          AND c.consumed_at<=observed_at AND c.expires_at>observed_at
          AND c.expires_at<=c.created_at+interval '5 minutes'
          AND octet_length(c.browser_nonce_sha256)=32
          AND octet_length(c.browser_origin) BETWEEN 1 AND 2048
          AND octet_length(c.terms_manifest_sha256)=32 AND c.terms_head_revision>=1
          AND k.org_id IS NULL AND f.org_id IS NULL AND t.org_id IS NULL
          AND f.protocol='ACCOUNT_V1' AND f.account_security_generation>=1
          AND f.auth_time>=c.created_at AND f.auth_time<=observed_at
          AND f.assurance='PASSKEY_PRIMARY' AND f.revoked_at IS NULL AND f.revoked_reason IS NULL
          AND t.used_at IS NULL AND t.revoked_at IS NULL AND t.replaced_by IS NULL
          AND t.reuse_detected_at IS NULL AND t.expires_at>observed_at
          AND t.issued_at<=observed_at AND octet_length(t.token_hash)=32
          AND (SELECT count(*) FROM public.auth_refresh_tokens all_tokens
               WHERE all_tokens.family_id=f.id)=1;
    observed_at := clock_timestamp();
    IF observed.ceremony_expires_at<=observed_at OR observed.token_expires_at<=observed_at THEN
        RAISE EXCEPTION 'account.enrollment_invalid';
    END IF;
    RETURN QUERY SELECT observed.ceremony_created_at,observed.terms_manifest_sha256,
        observed.terms_head_revision,observed.account_security_generation,
        observed.auth_time,observed.assurance;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.enrollment_invalid';
END
$body$;

CREATE FUNCTION public.account_registration_activate_v1(
    p_account uuid, p_ceremony uuid, p_key uuid, p_family uuid, p_receipt uuid,
    p_release_revision bigint, p_manifest_sha256 bytea,
    p_terms_kinds text[], p_content_sha256 bytea[])
RETURNS TABLE(security_generation bigint, revision bigint)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    control record;
    head record;
    material record;
    item record;
    birth_at timestamptz;
    activated_at timestamptz;
    terms_evidence jsonb;
    enrolled_evidence jsonb;
    event_id uuid;
    first_event_id uuid;
BEGIN
    IF p_account IS NULL OR p_ceremony IS NULL OR p_key IS NULL OR p_family IS NULL OR p_receipt IS NULL
        OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(ARRAY[p_account,p_ceremony,p_key,p_family,p_receipt])
        OR p_release_revision IS NULL OR p_release_revision<1
        OR p_manifest_sha256 IS NULL OR octet_length(p_manifest_sha256)<>32
        OR p_terms_kinds IS NULL OR p_content_sha256 IS NULL
        OR array_ndims(p_terms_kinds) IS DISTINCT FROM 1
        OR array_ndims(p_content_sha256) IS DISTINCT FROM 1
        OR cardinality(p_terms_kinds) NOT BETWEEN 1 AND 8
        OR cardinality(p_terms_kinds)<>cardinality(p_content_sha256)
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.enrollment_invalid';
    END IF;
    IF EXISTS(SELECT 1 FROM unnest(p_terms_kinds) AS kinds(value)
        WHERE value IS NULL OR value !~ '^[a-z][a-z0-9_.-]{0,63}$')
        OR (SELECT count(DISTINCT value) FROM unnest(p_terms_kinds) AS kinds(value))<>cardinality(p_terms_kinds)
        OR EXISTS(SELECT 1 FROM unnest(p_content_sha256) AS digests(value)
            WHERE value IS NULL OR octet_length(value)<>32) THEN
        RAISE EXCEPTION 'account.enrollment_invalid';
    END IF;
    SELECT * INTO STRICT control FROM public.account_security_lock_exclusive_v1(p_account);
    IF control.security_state IS DISTINCT FROM 'PENDING_ENROLLMENT'
        OR control.security_generation IS DISTINCT FROM 1
        OR control.revision IS DISTINCT FROM 1 OR control.context_generation IS DISTINCT FROM 1 THEN
        RAISE EXCEPTION 'account.enrollment_invalid';
    END IF;
    SELECT * INTO STRICT head FROM public.account_terms_registration_head_v1();
    IF head.release_receipt_id IS DISTINCT FROM p_receipt
        OR head.revision IS DISTINCT FROM p_release_revision
        OR head.manifest_sha256 IS DISTINCT FROM p_manifest_sha256 THEN
        RAISE EXCEPTION 'account.terms_changed';
    END IF;
    SELECT a.created_at INTO STRICT birth_at FROM public.accounts a WHERE a.id=p_account;
    SELECT * INTO STRICT material
        FROM public.auth_account_registration_material_v1(p_account,p_ceremony,p_key,p_family);
    activated_at := clock_timestamp();
    IF material.ceremony_created_at IS DISTINCT FROM birth_at
        OR material.terms_manifest_sha256 IS DISTINCT FROM p_manifest_sha256
        OR material.terms_head_revision IS DISTINCT FROM p_release_revision
        OR material.account_security_generation IS DISTINCT FROM 1
        OR material.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY'
        OR activated_at<birth_at OR activated_at>=birth_at+interval '5 minutes'
        OR EXISTS(SELECT 1 FROM public.account_security_events e WHERE e.account_id=p_account)
        OR EXISTS(SELECT 1 FROM public.account_terms_acceptances a WHERE a.account_id=p_account) THEN
        RAISE EXCEPTION 'account.enrollment_invalid';
    END IF;
    terms_evidence := jsonb_build_object('kind','ACCOUNT_TERMS_RELEASE','receipt_id',p_receipt::text,
        'revision',p_release_revision::text,'manifest_sha256',encode(p_manifest_sha256,'hex'));
    FOR item IN SELECT kinds.value AS kind,digests.value AS digest
        FROM unnest(p_terms_kinds) WITH ORDINALITY AS kinds(value,ordinal)
        JOIN unnest(p_content_sha256) WITH ORDINALITY AS digests(value,ordinal) USING(ordinal)
        ORDER BY kinds.ordinal
    LOOP
        event_id := gen_random_uuid();
        IF first_event_id IS NULL THEN first_event_id := event_id; END IF;
        INSERT INTO public.account_security_events(id,account_id,kind,occurred_at,
            actor_account_id,session_id,evidence_ref,payload)
            VALUES(event_id,p_account,'TERMS_ACCEPTED',activated_at,p_account,p_family,terms_evidence,
                jsonb_build_object('kind','TERMS_ACCEPTED','account_id',p_account::text,
                    'before_generation','1','after_generation','1','credential_id',p_key::text,
                    'session_id',p_family::text,'evidence',terms_evidence));
        INSERT INTO public.account_terms_acceptances(account_id,terms_kind,terms_version,
            content_sha256,accepted_at,security_event_id,terms_release_receipt_id,
            terms_release_revision,terms_manifest_sha256)
            VALUES(p_account,item.kind,encode(p_manifest_sha256,'hex'),item.digest,
                activated_at,event_id,p_receipt,p_release_revision,p_manifest_sha256);
    END LOOP;
    enrolled_evidence := jsonb_build_object('kind','ACCOUNT_SECURITY_EVENT',
        'account_id',p_account::text,'event_id',first_event_id::text);
    INSERT INTO public.account_security_events(id,account_id,kind,occurred_at,
        actor_account_id,session_id,evidence_ref,payload)
        VALUES(gen_random_uuid(),p_account,'ENROLLED',activated_at,p_account,p_family,enrolled_evidence,
            jsonb_build_object('kind','ENROLLED','account_id',p_account::text,
                'before_generation','1','after_generation','1','credential_id',p_key::text,
                'session_id',p_family::text,'evidence',enrolled_evidence));
    UPDATE public.account_security s SET security_state='ACTIVE',revision=2,updated_at=activated_at
        WHERE s.account_id=p_account AND s.security_state='PENDING_ENROLLMENT'
          AND s.security_generation=1 AND s.revision=1 AND s.context_generation=1;
    IF NOT FOUND THEN RAISE EXCEPTION 'account.enrollment_invalid'; END IF;
    RETURN QUERY SELECT 1::bigint,2::bigint;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.enrollment_invalid';
END
$body$;

CREATE FUNCTION public.account_context_presence_v1(p_account uuid)
RETURNS TABLE(context_generation bigint, has_candidates boolean)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    control record;
    enrolled record;
    first_terms record;
    first_acceptance record;
    item record;
    birth_at timestamptz;
    key_id text;
    referenced_event uuid;
    terms_evidence jsonb;
    enrolled_evidence jsonb;
    batch_size bigint;
    seen_kinds text[] := ARRAY[]::text[];
BEGIN
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
    IF control.security_state IS DISTINCT FROM 'ACTIVE' OR control.context_generation IS DISTINCT FROM 1 THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    SELECT a.created_at INTO STRICT birth_at FROM public.accounts a WHERE a.id=p_account;
    SELECT e.id,e.account_id,e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        INTO STRICT enrolled FROM public.account_security_events e
        WHERE e.account_id=p_account AND e.kind='ENROLLED';
    key_id := enrolled.payload->>'credential_id';
    IF enrolled.actor_account_id IS DISTINCT FROM p_account OR enrolled.session_id IS NULL
        OR enrolled.session_id='00000000-0000-0000-0000-000000000000'::uuid
        OR key_id IS NULL OR key_id !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        OR key_id='00000000-0000-0000-0000-000000000000'
        OR NOT COALESCE((enrolled.evidence_ref->>'event_id') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',false)
        OR enrolled.occurred_at<birth_at OR enrolled.occurred_at>=birth_at+interval '5 minutes' THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    referenced_event := (enrolled.evidence_ref->>'event_id')::uuid;
    IF referenced_event='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    enrolled_evidence := jsonb_build_object('kind','ACCOUNT_SECURITY_EVENT',
        'account_id',p_account::text,'event_id',referenced_event::text);
    IF enrolled.evidence_ref IS DISTINCT FROM enrolled_evidence
        OR enrolled.payload IS DISTINCT FROM jsonb_build_object('kind','ENROLLED','account_id',p_account::text,
            'before_generation','1','after_generation','1','credential_id',key_id,
            'session_id',enrolled.session_id::text,'evidence',enrolled_evidence) THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    SELECT e.id,e.account_id,e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        INTO STRICT first_terms FROM public.account_security_events e
        WHERE e.account_id=p_account AND e.id=referenced_event AND e.kind='TERMS_ACCEPTED';
    SELECT a.account_id,a.terms_kind,a.terms_version,a.content_sha256,a.accepted_at,a.security_event_id,
        a.terms_release_receipt_id,a.terms_release_revision,a.terms_manifest_sha256
        INTO STRICT first_acceptance FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.security_event_id=first_terms.id;
    IF first_acceptance.terms_release_receipt_id IS NULL
        OR first_acceptance.terms_release_receipt_id='00000000-0000-0000-0000-000000000000'::uuid
        OR first_acceptance.terms_release_revision IS NULL OR first_acceptance.terms_release_revision<1
        OR first_acceptance.terms_manifest_sha256 IS NULL OR octet_length(first_acceptance.terms_manifest_sha256)<>32
        OR first_acceptance.accepted_at IS DISTINCT FROM enrolled.occurred_at
        OR NOT EXISTS(SELECT 1 FROM public.account_terms_release_receipts r
            WHERE r.id=first_acceptance.terms_release_receipt_id
              AND r.revision=first_acceptance.terms_release_revision
              AND r.manifest_sha256=first_acceptance.terms_manifest_sha256)
        OR EXISTS(SELECT 1 FROM public.account_terms_acceptances a
            WHERE a.account_id=p_account AND a.accepted_at<enrolled.occurred_at) THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    terms_evidence := jsonb_build_object('kind','ACCOUNT_TERMS_RELEASE',
        'receipt_id',first_acceptance.terms_release_receipt_id::text,
        'revision',first_acceptance.terms_release_revision::text,
        'manifest_sha256',encode(first_acceptance.terms_manifest_sha256,'hex'));
    SELECT count(*) INTO batch_size FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at;
    IF batch_size NOT BETWEEN 1 AND 8
        OR (SELECT count(DISTINCT a.security_event_id) FROM public.account_terms_acceptances a
            WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at)<>batch_size
        OR (SELECT count(*) FROM public.account_security_events e
            WHERE e.account_id=p_account AND e.kind='TERMS_ACCEPTED'
              AND e.occurred_at=enrolled.occurred_at)<>batch_size THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
    END IF;
    FOR item IN SELECT a.terms_kind,a.terms_version,a.content_sha256,a.accepted_at,
        a.terms_release_receipt_id,a.terms_release_revision,a.terms_manifest_sha256,
        e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        FROM public.account_terms_acceptances a
        LEFT JOIN public.account_security_events e ON e.account_id=a.account_id AND e.id=a.security_event_id
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at
    LOOP
        IF item.terms_kind IS NULL OR item.terms_kind !~ '^[a-z][a-z0-9_.-]{0,63}$'
            OR item.terms_kind=ANY(seen_kinds)
            OR item.terms_version IS DISTINCT FROM encode(first_acceptance.terms_manifest_sha256,'hex')
            OR item.content_sha256 IS NULL OR octet_length(item.content_sha256)<>32
            OR item.terms_release_receipt_id IS DISTINCT FROM first_acceptance.terms_release_receipt_id
            OR item.terms_release_revision IS DISTINCT FROM first_acceptance.terms_release_revision
            OR item.terms_manifest_sha256 IS DISTINCT FROM first_acceptance.terms_manifest_sha256
            OR item.kind IS DISTINCT FROM 'TERMS_ACCEPTED'
            OR item.occurred_at IS DISTINCT FROM enrolled.occurred_at
            OR item.actor_account_id IS DISTINCT FROM p_account
            OR item.session_id IS DISTINCT FROM enrolled.session_id
            OR item.evidence_ref IS DISTINCT FROM terms_evidence
            OR item.payload IS DISTINCT FROM jsonb_build_object('kind','TERMS_ACCEPTED','account_id',p_account::text,
                'before_generation','1','after_generation','1','credential_id',key_id,
                'session_id',enrolled.session_id::text,'evidence',terms_evidence) THEN
            RAISE EXCEPTION 'account.navigation_unavailable';
        END IF;
        seen_kinds := array_append(seen_kinds,item.terms_kind);
    END LOOP;
    RETURN QUERY SELECT control.context_generation,EXISTS(
        SELECT 1 FROM public.account_context_candidates c WHERE c.account_id=p_account);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.navigation_unavailable';
END
$body$;

ALTER FUNCTION public.account_registration_begin_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_registration_begin_v1() FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_registration_begin_v1() TO console_account_owner,console_auth_rt;
ALTER FUNCTION public.account_security_lock_exclusive_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_security_lock_exclusive_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_security_lock_exclusive_v1(uuid) TO console_account_owner,console_auth_rt;
ALTER FUNCTION public.account_security_lock_shared_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_security_lock_shared_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_security_lock_shared_v1(uuid) TO console_account_owner,console_auth_rt;
ALTER FUNCTION public.account_terms_registration_head_v1() OWNER TO console_terms_owner;
REVOKE ALL ON FUNCTION public.account_terms_registration_head_v1() FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_terms_registration_head_v1() TO console_terms_owner,console_auth_rt,console_account_owner;
ALTER FUNCTION public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid) TO console_credential_owner,console_account_owner;
ALTER FUNCTION public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[]) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[]) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[]) TO console_account_owner,console_auth_rt;
ALTER FUNCTION public.account_context_presence_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_context_presence_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_context_presence_v1(uuid) TO console_account_owner,console_auth_rt;

-- PRIVATE ADDITIVE CANDIDATE. Preserve original seven-helper source unchanged.
-- Exact owners, EXECUTE/column ACLs and fingerprints belong to root finalizer.
CREATE FUNCTION public.auth_account_logout_revoke_v1(
    p_account uuid, p_family uuid, p_generation bigint, p_absolute_ttl interval)
RETURNS TABLE(account_security_generation bigint, auth_time timestamptz,
    assurance text, revoked_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    family record;
    observed_at timestamptz;
BEGIN
    -- Called only by the Account owner after its EXCLUSIVE guard; not executable
    -- by Auth runtime. Duration is trusted server configuration, never browser data.
    IF p_account IS NULL OR p_family IS NULL
        OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(ARRAY[p_account,p_family])
        OR p_generation IS NULL OR p_generation<1 OR p_absolute_ttl IS NULL
        OR p_absolute_ttl<=interval '0 seconds'
        OR extract(year FROM p_absolute_ttl)<>0 OR extract(month FROM p_absolute_ttl)<>0
        OR extract(day FROM p_absolute_ttl)<>0
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT f.user_id,f.org_id,f.protocol,f.account_security_generation,f.auth_time,
        f.assurance,f.created_at,f.revoked_at,f.revoked_reason INTO STRICT family
        FROM public.auth_refresh_token_families f WHERE f.id=p_family FOR UPDATE OF f;
    IF family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL
        OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
        OR family.account_security_generation IS DISTINCT FROM p_generation
        OR family.auth_time IS NULL OR family.auth_time>family.created_at
        OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY'
        OR family.revoked_at IS NOT NULL OR family.revoked_reason IS NOT NULL THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    -- Deterministic final lock class. Auth already holds these guards, but the
    -- fixed owner independently acquires them rather than trusting a proof bit.
    PERFORM t.id FROM public.auth_refresh_tokens t WHERE t.family_id=p_family
        ORDER BY t.id FOR UPDATE OF t;
    observed_at := clock_timestamp();
    IF family.created_at>observed_at
        OR family.created_at+p_absolute_ttl<=observed_at
        OR EXISTS(SELECT 1 FROM public.auth_refresh_tokens t WHERE t.family_id=p_family
            AND (t.user_id IS DISTINCT FROM p_account OR t.org_id IS NOT NULL
                 OR t.revoked_at>observed_at)) THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    UPDATE public.auth_refresh_token_families f
        SET revoked_at=observed_at,revoked_reason='logout'
        WHERE f.id=p_family AND f.user_id=p_account AND f.revoked_at IS NULL;
    IF NOT FOUND THEN RAISE EXCEPTION 'account.authentication_invalid'; END IF;
    UPDATE public.auth_refresh_tokens t SET revoked_at=COALESCE(t.revoked_at,observed_at)
        WHERE t.family_id=p_family AND t.user_id=p_account AND t.org_id IS NULL;
    IF EXISTS(SELECT 1 FROM public.auth_refresh_tokens t
        WHERE t.family_id=p_family AND t.revoked_at IS NULL) THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    RETURN QUERY SELECT family.account_security_generation,family.auth_time,
        family.assurance,observed_at;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.authentication_invalid';
END
$body$;

CREATE FUNCTION public.account_session_logout_v1(
    p_account uuid, p_family uuid, p_generation bigint, p_absolute_ttl interval)
RETURNS TABLE(event_id uuid, revoked_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    control record;
    transition record;
    evidence jsonb;
    allocated_event uuid;
BEGIN
    IF p_account IS NULL OR p_family IS NULL
        OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(ARRAY[p_account,p_family])
        OR p_generation IS NULL OR p_generation<1
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT * INTO STRICT control FROM public.account_security_lock_exclusive_v1(p_account);
    IF control.security_state IS DISTINCT FROM 'ACTIVE'
        OR control.security_generation IS DISTINCT FROM p_generation
        OR EXISTS(SELECT 1 FROM public.account_security_events e
            WHERE e.account_id=p_account AND e.session_id=p_family AND e.kind='SESSION_REVOKED') THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT * INTO STRICT transition
        FROM public.auth_account_logout_revoke_v1(p_account,p_family,p_generation,p_absolute_ttl);
    IF transition.account_security_generation IS DISTINCT FROM p_generation
        OR transition.auth_time IS NULL OR transition.revoked_at IS NULL
        OR transition.auth_time>transition.revoked_at
        OR transition.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    evidence := jsonb_build_object(
        'kind','ACCOUNT_SESSION_TRANSITION','operation','LOGOUT',
        'account_id',p_account::text,'session_id',p_family::text,
        'security_generation',p_generation::text,
        'auth_time',((extract(epoch FROM transition.auth_time)*1000000)::bigint)::text,
        'assurance',transition.assurance,'before_state','LIVE','after_state','REVOKED',
        'revoked_at',((extract(epoch FROM transition.revoked_at)*1000000)::bigint)::text,
        'reason','logout');
    allocated_event := gen_random_uuid();
    INSERT INTO public.account_security_events(id,account_id,kind,occurred_at,
        actor_account_id,session_id,evidence_ref,payload)
        VALUES(allocated_event,p_account,'SESSION_REVOKED',transition.revoked_at,
            p_account,p_family,evidence,jsonb_build_object(
                'kind','SESSION_REVOKED','account_id',p_account::text,
                'before_generation',p_generation::text,'after_generation',p_generation::text,
                'credential_id',NULL,'session_id',p_family::text,'evidence',evidence));
    RETURN QUERY SELECT allocated_event,transition.revoked_at;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.authentication_invalid';
END
$body$;

ALTER FUNCTION public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval) TO console_credential_owner,console_account_owner;
ALTER FUNCTION public.account_session_logout_v1(uuid,uuid,bigint,interval) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_session_logout_v1(uuid,uuid,bigint,interval) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_session_logout_v1(uuid,uuid,bigint,interval) TO console_account_owner,console_auth_rt;

-- Separately admitted retained42501 repair. Historical Auth7 remains unchanged.
CREATE OR REPLACE FUNCTION public.auth_legacy_self_passkey_delete_v1(p_company uuid,p_actor uuid,p_key_id uuid) RETURNS TABLE(outcome text,credential_id text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $auth7_body$DECLARE allowed boolean; victim text; total bigint;
BEGIN
    PERFORM public.auth_legacy_company_lock_v1(p_company);
    SELECT public.account_company_deactivation_guard_v1(p_company,p_actor) INTO allowed;
    IF allowed IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.invalid_fence';
    END IF;
    IF NOT allowed THEN
        RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='auth_legacy.fenced';
    END IF;
    SELECT k.credential_id INTO victim FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company;
    IF NOT FOUND THEN
        RETURN QUERY SELECT 'not_found'::text,NULL::text;
        RETURN;
    END IF;
    SELECT count(*) INTO total FROM public.auth_webauthn_credentials k
        WHERE k.user_id=p_actor AND k.org_id=p_company;
    IF total <= 1 THEN
        RETURN QUERY SELECT 'last_key'::text,NULL::text;
        RETURN;
    END IF;
    DELETE FROM public.auth_webauthn_credentials k
        WHERE k.id=p_key_id AND k.user_id=p_actor AND k.org_id=p_company;
    RETURN QUERY SELECT 'deleted'::text,victim;
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid) TO console_auth_rt,console_credential_owner,console_rt;
"""

# Exact additive helper source. No old helper or table is replaced.
NATIVE_EXTENSION_INSTALL = r"""-- PRIVATE PROPOSAL ONLY. Root owns custody/profile installation and runtime.
-- Called after cryptographically valid native primary proof, with Account guard
-- already held. This helper validates immutable owner evidence; it does not
-- validate browser signatures or claim local artifacts are available.
-- No current-head, context-generation or Company-presence condition.
-- Missing/malformed historical evidence is P0001 terms_acceptance_required;
-- database/resolver errors propagate as unavailable, never as empty consent.
CREATE FUNCTION public.account_login_consent_v1(p_account uuid)
RETURNS TABLE(manifest_sha256 bytea, terms_kind text, content_sha256 bytea, accepted_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    control record;
    enrolled record;
    first_terms record;
    first_acceptance record;
    item record;
    birth_at timestamptz;
    key_id text;
    referenced_event uuid;
    terms_evidence jsonb;
    enrolled_evidence jsonb;
    batch_size bigint;
    seen_kinds text[] := ARRAY[]::text[];
BEGIN
    IF p_account IS NULL OR p_account='00000000-0000-0000-0000-000000000000'::uuid
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
    IF control.security_state IS DISTINCT FROM 'ACTIVE' THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT a.created_at INTO STRICT birth_at FROM public.accounts a WHERE a.id=p_account;
    -- Only missing historical evidence maps to authenticated consent refusal.
    -- The Account/security guard and root read above preserve owner failures.
    BEGIN
    SELECT e.id,e.account_id,e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        INTO STRICT enrolled FROM public.account_security_events e
        WHERE e.account_id=p_account AND e.kind='ENROLLED';
    key_id := enrolled.payload->>'credential_id';
    IF enrolled.actor_account_id IS DISTINCT FROM p_account OR enrolled.session_id IS NULL
        OR enrolled.session_id='00000000-0000-0000-0000-000000000000'::uuid
        OR key_id IS NULL OR key_id !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        OR key_id='00000000-0000-0000-0000-000000000000'
        OR NOT COALESCE((enrolled.evidence_ref->>'event_id') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$',false)
        OR enrolled.occurred_at<birth_at OR enrolled.occurred_at>=birth_at+interval '5 minutes'
        OR enrolled.occurred_at>clock_timestamp() THEN
        RAISE EXCEPTION 'account.terms_acceptance_required';
    END IF;
    referenced_event := (enrolled.evidence_ref->>'event_id')::uuid;
    IF referenced_event='00000000-0000-0000-0000-000000000000'::uuid THEN
        RAISE EXCEPTION 'account.terms_acceptance_required';
    END IF;
    enrolled_evidence := jsonb_build_object('kind','ACCOUNT_SECURITY_EVENT',
        'account_id',p_account::text,'event_id',referenced_event::text);
    IF enrolled.evidence_ref IS DISTINCT FROM enrolled_evidence
        OR enrolled.payload IS DISTINCT FROM jsonb_build_object('kind','ENROLLED','account_id',p_account::text,
            'before_generation','1','after_generation','1','credential_id',key_id,
            'session_id',enrolled.session_id::text,'evidence',enrolled_evidence) THEN
        RAISE EXCEPTION 'account.terms_acceptance_required';
    END IF;
    SELECT e.id,e.account_id,e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        INTO STRICT first_terms FROM public.account_security_events e
        WHERE e.account_id=p_account AND e.id=referenced_event AND e.kind='TERMS_ACCEPTED';
    SELECT a.account_id,a.terms_kind,a.terms_version,a.content_sha256,a.accepted_at,a.security_event_id,
        a.terms_release_receipt_id,a.terms_release_revision,a.terms_manifest_sha256
        INTO STRICT first_acceptance FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.security_event_id=first_terms.id;
    IF first_acceptance.terms_release_receipt_id IS NULL
        OR first_acceptance.terms_release_receipt_id='00000000-0000-0000-0000-000000000000'::uuid
        OR first_acceptance.terms_release_revision IS NULL OR first_acceptance.terms_release_revision<1
        OR first_acceptance.terms_manifest_sha256 IS NULL OR octet_length(first_acceptance.terms_manifest_sha256)<>32
        OR first_acceptance.accepted_at IS DISTINCT FROM enrolled.occurred_at
        OR NOT EXISTS(SELECT 1 FROM public.account_terms_release_receipts r
            WHERE r.id=first_acceptance.terms_release_receipt_id
              AND r.revision=first_acceptance.terms_release_revision
              AND r.manifest_sha256=first_acceptance.terms_manifest_sha256)
        OR EXISTS(SELECT 1 FROM public.account_terms_acceptances a
            WHERE a.account_id=p_account AND a.accepted_at<enrolled.occurred_at) THEN
        RAISE EXCEPTION 'account.terms_acceptance_required';
    END IF;
    terms_evidence := jsonb_build_object('kind','ACCOUNT_TERMS_RELEASE',
        'receipt_id',first_acceptance.terms_release_receipt_id::text,
        'revision',first_acceptance.terms_release_revision::text,
        'manifest_sha256',encode(first_acceptance.terms_manifest_sha256,'hex'));
    SELECT count(*) INTO batch_size FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at;
    IF batch_size NOT BETWEEN 1 AND 8
        OR (SELECT count(DISTINCT a.security_event_id) FROM public.account_terms_acceptances a
            WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at)<>batch_size
        OR (SELECT count(*) FROM public.account_security_events e
            WHERE e.account_id=p_account AND e.kind='TERMS_ACCEPTED'
              AND e.occurred_at=enrolled.occurred_at)<>batch_size THEN
        RAISE EXCEPTION 'account.terms_acceptance_required';
    END IF;
    FOR item IN SELECT a.terms_kind,a.terms_version,a.content_sha256,a.accepted_at,
        a.terms_release_receipt_id,a.terms_release_revision,a.terms_manifest_sha256,
        e.kind,e.occurred_at,e.actor_account_id,e.session_id,e.evidence_ref,e.payload
        FROM public.account_terms_acceptances a
        LEFT JOIN public.account_security_events e ON e.account_id=a.account_id AND e.id=a.security_event_id
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at
    LOOP
        IF item.terms_kind IS NULL OR item.terms_kind !~ '^[a-z][a-z0-9_.-]{0,63}$'
            OR item.terms_kind=ANY(seen_kinds)
            OR item.terms_version IS DISTINCT FROM encode(first_acceptance.terms_manifest_sha256,'hex')
            OR item.content_sha256 IS NULL OR octet_length(item.content_sha256)<>32
            OR item.terms_release_receipt_id IS DISTINCT FROM first_acceptance.terms_release_receipt_id
            OR item.terms_release_revision IS DISTINCT FROM first_acceptance.terms_release_revision
            OR item.terms_manifest_sha256 IS DISTINCT FROM first_acceptance.terms_manifest_sha256
            OR item.kind IS DISTINCT FROM 'TERMS_ACCEPTED'
            OR item.occurred_at IS DISTINCT FROM enrolled.occurred_at
            OR item.actor_account_id IS DISTINCT FROM p_account
            OR item.session_id IS DISTINCT FROM enrolled.session_id
            OR item.evidence_ref IS DISTINCT FROM terms_evidence
            OR item.payload IS DISTINCT FROM jsonb_build_object('kind','TERMS_ACCEPTED','account_id',p_account::text,
                'before_generation','1','after_generation','1','credential_id',key_id,
                'session_id',enrolled.session_id::text,'evidence',terms_evidence) THEN
            RAISE EXCEPTION 'account.terms_acceptance_required';
        END IF;
        seen_kinds := array_append(seen_kinds,item.terms_kind);
    END LOOP;
    RETURN QUERY SELECT a.terms_manifest_sha256,a.terms_kind,a.content_sha256,a.accepted_at
        FROM public.account_terms_acceptances a
        WHERE a.account_id=p_account AND a.accepted_at=enrolled.occurred_at
        ORDER BY a.terms_kind;
    EXCEPTION WHEN no_data_found OR too_many_rows THEN
        RAISE EXCEPTION 'account.terms_acceptance_required';
    END;
END
$body$;
ALTER FUNCTION public.account_login_consent_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_login_consent_v1(uuid) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_login_consent_v1(uuid) TO console_account_owner,console_auth_rt;

-- Private root-owned proposal; no integration installer or grants are executed.
-- Additive beside the exact final9 helpers. Root owns their reviewed upgrade,
-- fingerprints and operational custody. Existing logout/Auth7 sources unchanged.
CREATE FUNCTION public.auth_account_refresh_reuse_revoke_v1(
    p_account uuid, p_family uuid, p_token uuid, p_token_hash bytea,
    p_generation bigint, p_absolute_ttl interval)
RETURNS TABLE(account_security_generation bigint, auth_time timestamptz,
    assurance text, revoked_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    family record;
    reused record;
    replacement record;
    observed_at timestamptz;
BEGIN
    -- Only Account owner may call, after its exclusive current-security guard.
    IF p_account IS NULL OR p_family IS NULL OR p_token IS NULL
        OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(ARRAY[p_account,p_family,p_token])
        OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
        OR p_generation IS NULL OR p_generation<1 OR p_absolute_ttl IS NULL
        OR p_absolute_ttl<=interval '0 seconds'
        OR extract(year FROM p_absolute_ttl)<>0 OR extract(month FROM p_absolute_ttl)<>0
        OR extract(day FROM p_absolute_ttl)<>0
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT f.user_id,f.org_id,f.protocol,f.account_security_generation,f.auth_time,
        f.assurance,f.created_at,f.revoked_at,f.revoked_reason INTO STRICT family
        FROM public.auth_refresh_token_families f WHERE f.id=p_family FOR UPDATE OF f;
    IF family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL
        OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
        OR family.account_security_generation IS DISTINCT FROM p_generation
        OR family.auth_time IS NULL OR family.auth_time>family.created_at
        OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY'
        OR family.revoked_at IS NOT NULL OR family.revoked_reason IS NOT NULL THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    -- The Account guard serializes all native security writers; take every
    -- existing sibling token in deterministic order before the family effect.
    PERFORM t.id FROM public.auth_refresh_tokens t WHERE t.family_id=p_family
        ORDER BY t.id FOR UPDATE OF t;
    SELECT t.id,t.user_id,t.family_id,t.org_id,t.issued_at,t.expires_at,t.used_at,
        t.replaced_by,t.revoked_at,t.reuse_detected_at INTO STRICT reused
        FROM public.auth_refresh_tokens t
        WHERE t.id=p_token AND t.token_hash=p_token_hash;
    IF reused.user_id IS DISTINCT FROM p_account OR reused.family_id IS DISTINCT FROM p_family
        OR reused.org_id IS NOT NULL OR reused.used_at IS NULL
        OR reused.replaced_by IS NULL OR reused.replaced_by=p_token
        OR reused.revoked_at IS NOT NULL OR reused.reuse_detected_at IS NOT NULL
        OR reused.issued_at<family.created_at OR reused.used_at<reused.issued_at
        OR reused.expires_at<=reused.issued_at OR reused.used_at>=reused.expires_at THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT t.user_id,t.family_id,t.org_id,t.issued_at INTO STRICT replacement
        FROM public.auth_refresh_tokens t WHERE t.id=reused.replaced_by;
    IF replacement.user_id IS DISTINCT FROM p_account
        OR replacement.family_id IS DISTINCT FROM p_family OR replacement.org_id IS NOT NULL
        OR replacement.issued_at IS DISTINCT FROM reused.used_at THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    observed_at := clock_timestamp();
    IF family.created_at>observed_at OR family.created_at+p_absolute_ttl<=observed_at
        OR reused.used_at>observed_at
        OR reused.expires_at>family.created_at+p_absolute_ttl
        OR EXISTS(SELECT 1 FROM public.auth_refresh_tokens t WHERE t.family_id=p_family
            AND (t.user_id IS DISTINCT FROM p_account OR t.org_id IS NOT NULL
                 OR t.revoked_at>observed_at OR t.reuse_detected_at>observed_at)) THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    -- AS1 retains reuse precedence before token expiry. A used+expired token is
    -- evidence only under current bound proof/live family, never a renewal.
    UPDATE public.auth_refresh_token_families f
        SET revoked_at=observed_at,revoked_reason='reuse_detected'
        WHERE f.id=p_family AND f.user_id=p_account AND f.revoked_at IS NULL;
    IF NOT FOUND THEN RAISE EXCEPTION 'account.authentication_invalid'; END IF;
    UPDATE public.auth_refresh_tokens t
        SET revoked_at=COALESCE(t.revoked_at,observed_at),
            reuse_detected_at=CASE WHEN t.id=p_token THEN observed_at ELSE t.reuse_detected_at END
        WHERE t.family_id=p_family AND t.user_id=p_account AND t.org_id IS NULL;
    IF NOT EXISTS(SELECT 1 FROM public.auth_refresh_tokens t
        WHERE t.id=p_token AND t.family_id=p_family AND t.user_id=p_account
            AND t.revoked_at=observed_at AND t.reuse_detected_at=observed_at)
        OR EXISTS(SELECT 1 FROM public.auth_refresh_tokens t
            WHERE t.family_id=p_family AND t.revoked_at IS NULL) THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    RETURN QUERY SELECT family.account_security_generation,family.auth_time,
        family.assurance,observed_at;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.authentication_invalid';
END
$body$;

CREATE FUNCTION public.account_session_refresh_reuse_v1(
    p_account uuid, p_family uuid, p_token uuid, p_token_hash bytea,
    p_generation bigint, p_absolute_ttl interval)
RETURNS TABLE(event_id uuid, revoked_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=off
AS $body$
DECLARE
    control record;
    transition record;
    evidence jsonb;
    allocated_event uuid;
BEGIN
    IF p_account IS NULL OR p_family IS NULL OR p_token IS NULL
        OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(ARRAY[p_account,p_family,p_token])
        OR p_token_hash IS NULL OR octet_length(p_token_hash)<>32
        OR p_generation IS NULL OR p_generation<1
        OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    -- A missing/invalid authority root is unavailable, not bad replay evidence.
    -- Keep its P0002 separate from later token/family lookup refusal mapping.
    BEGIN
        SELECT * INTO STRICT control FROM public.account_security_lock_exclusive_v1(p_account);
    EXCEPTION WHEN no_data_found OR too_many_rows THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END;
    IF control.security_state IS DISTINCT FROM 'ACTIVE'
        OR control.security_generation IS DISTINCT FROM p_generation
        OR EXISTS(SELECT 1 FROM public.account_security_events e
            WHERE e.account_id=p_account AND e.session_id=p_family AND e.kind='SESSION_REVOKED') THEN
        RAISE EXCEPTION 'account.authentication_invalid';
    END IF;
    SELECT * INTO STRICT transition FROM public.auth_account_refresh_reuse_revoke_v1(
        p_account,p_family,p_token,p_token_hash,p_generation,p_absolute_ttl);
    IF transition.account_security_generation IS DISTINCT FROM p_generation
        OR transition.auth_time IS NULL OR transition.revoked_at IS NULL
        OR transition.auth_time>transition.revoked_at
        OR transition.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' THEN
        RAISE EXCEPTION 'account.authority_unavailable';
    END IF;
    evidence := jsonb_build_object(
        'kind','ACCOUNT_SESSION_TRANSITION','operation','REFRESH_REUSE',
        'account_id',p_account::text,'session_id',p_family::text,
        'security_generation',p_generation::text,
        'auth_time',((extract(epoch FROM transition.auth_time)*1000000)::bigint)::text,
        'assurance',transition.assurance,'before_state','LIVE','after_state','REVOKED',
        'revoked_at',((extract(epoch FROM transition.revoked_at)*1000000)::bigint)::text,
        'reason','reuse_detected','reused_token_id',p_token::text);
    allocated_event := gen_random_uuid();
    INSERT INTO public.account_security_events(id,account_id,kind,occurred_at,
        actor_account_id,session_id,evidence_ref,payload)
        VALUES(allocated_event,p_account,'SESSION_REVOKED',transition.revoked_at,
            p_account,p_family,evidence,jsonb_build_object(
                'kind','SESSION_REVOKED','account_id',p_account::text,
                'before_generation',p_generation::text,'after_generation',p_generation::text,
                'credential_id',NULL,'session_id',p_family::text,'evidence',evidence));
    RETURN QUERY SELECT allocated_event,transition.revoked_at;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    RAISE EXCEPTION 'account.authentication_invalid';
END
$body$;

ALTER FUNCTION public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval) TO console_credential_owner,console_account_owner;
ALTER FUNCTION public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_account_owner,console_terms_owner,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval) TO console_account_owner,console_auth_rt;
"""

DEPLOYMENT_INSTALL = r"""-- PRIVATE DECLARED SOURCE: insert into the canonical custody generator only
-- after independent review. This is NOT a standalone deployment operation.
-- Root's owning transaction must validate the exact approved 227 predecessor
-- plus empty 228 staging, strict externally provisioned startup role, historical
-- data preservation and complete new metadata postcondition.

ALTER TABLE public.deployment_operator_receipts
    ADD CONSTRAINT deployment_operator_receipts_account_v1
    FOREIGN KEY (account_id) REFERENCES public.accounts(id)
    ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.deployment_operator_receipts OWNER TO console_account_owner;
ALTER TABLE public.deployment_operator_head OWNER TO console_account_owner;
REVOKE ALL ON TABLE public.deployment_operator_receipts, public.deployment_operator_head
    FROM PUBLIC, console_account_owner, console_app, console_rt, console_auth_rt,
        console_auth_startup, console_terms_owner, console_credential_owner,
        console_leave_cmd, console_ontology_cmd, console_platform_force_cmd;

CREATE FUNCTION public.deployment_operator_receipts_immutable_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path = pg_catalog, pg_temp
AS $body$
BEGIN
    RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.receipt_immutable';
END
$body$;
ALTER FUNCTION public.deployment_operator_receipts_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.deployment_operator_receipts_immutable_v1() FROM PUBLIC, console_account_owner;
CREATE TRIGGER deployment_operator_receipts_immutable_v1
    BEFORE UPDATE OR DELETE OR TRUNCATE ON public.deployment_operator_receipts
    FOR EACH STATEMENT EXECUTE FUNCTION public.deployment_operator_receipts_immutable_v1();
ALTER TABLE public.deployment_operator_receipts ENABLE ALWAYS TRIGGER deployment_operator_receipts_immutable_v1;

CREATE FUNCTION public.deployment_operator_head_immutable_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path = pg_catalog, pg_temp
AS $body$
BEGIN
    IF TG_OP <> 'UPDATE' THEN
        RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.head_immutable';
    END IF;
    IF (NEW.singleton, NEW.system_identifier, NEW.database_name, NEW.database_oid, NEW.account_id)
        IS DISTINCT FROM
        (OLD.singleton, OLD.system_identifier, OLD.database_name, OLD.database_oid, OLD.account_id) THEN
        RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.head_immutable';
    END IF;
    RETURN NEW;
END
$body$;
ALTER FUNCTION public.deployment_operator_head_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.deployment_operator_head_immutable_v1() FROM PUBLIC, console_account_owner;
CREATE TRIGGER deployment_operator_head_binding_v1
    BEFORE UPDATE ON public.deployment_operator_head
    FOR EACH ROW EXECUTE FUNCTION public.deployment_operator_head_immutable_v1();
CREATE TRIGGER deployment_operator_head_preserve_v1
    BEFORE DELETE OR TRUNCATE ON public.deployment_operator_head
    FOR EACH STATEMENT EXECUTE FUNCTION public.deployment_operator_head_immutable_v1();
ALTER TABLE public.deployment_operator_head ENABLE ALWAYS TRIGGER deployment_operator_head_binding_v1;
ALTER TABLE public.deployment_operator_head ENABLE ALWAYS TRIGGER deployment_operator_head_preserve_v1;

-- Install immutable guards before native FK key-share's required key UPDATE.
GRANT SELECT, INSERT ON TABLE public.deployment_operator_receipts TO console_account_owner;
GRANT UPDATE (receipt_id) ON TABLE public.deployment_operator_receipts TO console_account_owner;
GRANT SELECT, INSERT, UPDATE ON TABLE public.deployment_operator_head TO console_account_owner;
-- PostgreSQL18 defaults this builtin to PUBLIC. Close that known staging
-- capability before granting the exact dedicated owner, like the observer installer.
REVOKE EXECUTE ON FUNCTION pg_catalog.pg_control_system() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION pg_catalog.pg_control_system() TO console_account_owner;

CREATE FUNCTION public.deployment_operator_designate_v1(
    p_system_identifier text, p_database_name text, p_database_oid bigint,
    p_command_id uuid, p_account_id uuid, p_security_generation bigint, p_expected_revision bigint
)
RETURNS TABLE (receipt_id uuid, revision bigint, replayed boolean)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path = pg_catalog, pg_temp
AS $body$
DECLARE
    stored public.deployment_operator_receipts%ROWTYPE;
    account_guard record;
    guard_locked boolean := false;
    allocated_receipt uuid;
BEGIN
    IF p_system_identifier IS NULL OR p_database_name IS NULL OR p_database_oid IS NULL
        OR p_command_id IS NULL OR p_account_id IS NULL OR p_security_generation IS NULL
        OR p_expected_revision IS NULL OR p_command_id = '00000000-0000-0000-0000-000000000000'::uuid
        OR p_account_id = '00000000-0000-0000-0000-000000000000'::uuid
        OR pg_catalog.octet_length(p_system_identifier) NOT BETWEEN 1 AND 20
        OR p_system_identifier !~ '^[1-9][0-9]{0,19}$'
        OR (pg_catalog.octet_length(p_system_identifier) = 20
            AND p_system_identifier COLLATE "C" > '18446744073709551615' COLLATE "C")
        OR pg_catalog.octet_length(p_database_name) NOT BETWEEN 1 AND 63
        OR p_database_oid NOT BETWEEN 1 AND 4294967295
        OR p_security_generation < 1 OR p_expected_revision < 0
        OR pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.invalid_command';
    END IF;
    IF p_system_identifier IS DISTINCT FROM
            (SELECT c.system_identifier::text FROM pg_catalog.pg_control_system() c)
        OR p_database_name IS DISTINCT FROM pg_catalog.current_database()::text
        OR p_database_oid IS DISTINCT FROM
            (SELECT d.oid::bigint FROM pg_catalog.pg_database d WHERE d.datname = pg_catalog.current_database()) THEN
        RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_mismatch';
    END IF;

    LOOP
        -- Re-read after every blocking operation. The immutable unique command
        -- can acquire one committed receipt only; conflict retries converge.
        SELECT r.* INTO stored FROM public.deployment_operator_receipts r WHERE r.command_id = p_command_id;
        IF FOUND THEN
            IF (stored.system_identifier, stored.database_name, stored.database_oid, stored.account_id,
                stored.kind, stored.expected_revision, stored.expected_security_generation, stored.reason)
                IS DISTINCT FROM
                (p_system_identifier, p_database_name, p_database_oid, p_account_id,
                    'DESIGNATE'::text, p_expected_revision, p_security_generation, NULL::text) THEN
                RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.command_conflict';
            END IF;
            RETURN QUERY SELECT stored.receipt_id, stored.revision, true;
            RETURN;
        END IF;
        IF p_expected_revision <> 0 THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.invalid_command';
        END IF;
        -- Nonmutating refusal needs no head lock. Every real writer still
        -- obtains the Account guard before the singleton insertion below.
        IF EXISTS (SELECT 1 FROM public.deployment_operator_head h WHERE h.singleton = 1) THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.already_initialized';
        END IF;
        IF NOT guard_locked THEN
            BEGIN
                SELECT g.* INTO account_guard FROM public.account_security_lock_exclusive_v1(p_account_id) g;
            EXCEPTION WHEN no_data_found THEN
                IF SQLERRM <> 'account.authority_unavailable' THEN RAISE; END IF;
                RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_ineligible';
            END;
            guard_locked := true;
            CONTINUE;
        END IF;
        -- Existing head/tombstone permanently closes genesis. Account guard is
        -- acquired first; the native unique head insertion handles an absent row.
        IF account_guard.security_state IS DISTINCT FROM 'ACTIVE'
            OR account_guard.security_generation IS DISTINCT FROM p_security_generation THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_ineligible';
        END IF;
        allocated_receipt := pg_catalog.gen_random_uuid();
        INSERT INTO public.deployment_operator_receipts AS r
            (receipt_id, command_id, system_identifier, database_name, database_oid, account_id,
                kind, expected_revision, revision, expected_security_generation, reason, recorded_at)
        VALUES (allocated_receipt, p_command_id, p_system_identifier, p_database_name, p_database_oid,
            p_account_id, 'DESIGNATE', p_expected_revision, 1, p_security_generation, NULL, pg_catalog.clock_timestamp())
        ON CONFLICT (command_id) DO NOTHING;
        IF NOT FOUND THEN CONTINUE; END IF;
        INSERT INTO public.deployment_operator_head
            (singleton, system_identifier, database_name, database_oid, account_id, revision, receipt_id)
        VALUES (1, p_system_identifier, p_database_name, p_database_oid, p_account_id, 1, allocated_receipt)
        ON CONFLICT (singleton) DO NOTHING;
        IF NOT FOUND THEN
            -- The exception rolls the speculative receipt back as part of this
            -- statement; immutable history is never repaired by DELETE.
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.already_initialized';
        END IF;
        RETURN QUERY SELECT allocated_receipt, 1::bigint, false;
        RETURN;
    END LOOP;
END
$body$;

CREATE FUNCTION public.deployment_operator_revoke_v1(
    p_system_identifier text, p_database_name text, p_database_oid bigint,
    p_command_id uuid, p_account_id uuid, p_expected_revision bigint, p_reason text
)
RETURNS TABLE (receipt_id uuid, revision bigint, replayed boolean)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path = pg_catalog, pg_temp
AS $body$
DECLARE
    stored public.deployment_operator_receipts%ROWTYPE;
    head public.deployment_operator_head%ROWTYPE;
    guard_locked boolean := false;
    head_locked boolean := false;
    head_kind text;
    allocated_receipt uuid;
BEGIN
    IF p_system_identifier IS NULL OR p_database_name IS NULL OR p_database_oid IS NULL
        OR p_command_id IS NULL OR p_account_id IS NULL OR p_expected_revision IS NULL OR p_reason IS NULL
        OR p_command_id = '00000000-0000-0000-0000-000000000000'::uuid
        OR p_account_id = '00000000-0000-0000-0000-000000000000'::uuid
        OR pg_catalog.octet_length(p_system_identifier) NOT BETWEEN 1 AND 20
        OR p_system_identifier !~ '^[1-9][0-9]{0,19}$'
        OR (pg_catalog.octet_length(p_system_identifier) = 20
            AND p_system_identifier COLLATE "C" > '18446744073709551615' COLLATE "C")
        OR pg_catalog.octet_length(p_database_name) NOT BETWEEN 1 AND 63
        OR p_database_oid NOT BETWEEN 1 AND 4294967295 OR p_expected_revision < 0
        OR pg_catalog.octet_length(p_reason) NOT BETWEEN 1 AND 512
        OR pg_catalog.length(pg_catalog.btrim(p_reason, E' \t\n\r\f\013')) = 0
        OR pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
        RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.invalid_command';
    END IF;
    IF p_system_identifier IS DISTINCT FROM
            (SELECT c.system_identifier::text FROM pg_catalog.pg_control_system() c)
        OR p_database_name IS DISTINCT FROM pg_catalog.current_database()::text
        OR p_database_oid IS DISTINCT FROM
            (SELECT d.oid::bigint FROM pg_catalog.pg_database d WHERE d.datname = pg_catalog.current_database()) THEN
        RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_mismatch';
    END IF;

    LOOP
        SELECT r.* INTO stored FROM public.deployment_operator_receipts r WHERE r.command_id = p_command_id;
        IF FOUND THEN
            IF (stored.system_identifier, stored.database_name, stored.database_oid, stored.account_id,
                stored.kind, stored.expected_revision, stored.expected_security_generation, stored.reason)
                IS DISTINCT FROM
                (p_system_identifier, p_database_name, p_database_oid, p_account_id,
                    'REVOKE'::text, p_expected_revision, NULL::bigint, p_reason) THEN
                RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.command_conflict';
            END IF;
            RETURN QUERY SELECT stored.receipt_id, stored.revision, true;
            RETURN;
        END IF;
        IF NOT guard_locked THEN
            BEGIN
                PERFORM 1 FROM public.account_security_lock_exclusive_v1(p_account_id);
            EXCEPTION WHEN no_data_found THEN
                IF SQLERRM <> 'account.authority_unavailable' THEN RAISE; END IF;
                RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_ineligible';
            END;
            guard_locked := true;
            CONTINUE;
        END IF;
        IF NOT head_locked THEN
            SELECT h.* INTO head FROM public.deployment_operator_head h WHERE h.singleton = 1 FOR UPDATE OF h;
            IF NOT FOUND THEN
                RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_ineligible';
            END IF;
            head_locked := true;
            -- Separate statement after any row wait: exact replay takes priority
            -- over CAS and state; the pointed receipt is read with fresh visibility.
            CONTINUE;
        END IF;
        IF (head.system_identifier, head.database_name, head.database_oid, head.account_id)
            IS DISTINCT FROM (p_system_identifier, p_database_name, p_database_oid, p_account_id) THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.target_ineligible';
        END IF;
        IF head.revision <> p_expected_revision THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.stale_revision';
        END IF;
        SELECT r.kind INTO head_kind FROM public.deployment_operator_receipts r WHERE r.receipt_id = head.receipt_id;
        IF head_kind IS DISTINCT FROM 'DESIGNATE' THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.already_revoked';
        END IF;
        IF head.revision = 9223372036854775807 THEN
            RAISE EXCEPTION USING ERRCODE = 'P0001', MESSAGE = 'deployment_operator.invalid_command';
        END IF;
        allocated_receipt := pg_catalog.gen_random_uuid();
        INSERT INTO public.deployment_operator_receipts
            (receipt_id, command_id, system_identifier, database_name, database_oid, account_id,
                kind, expected_revision, revision, expected_security_generation, reason, recorded_at)
        VALUES (allocated_receipt, p_command_id, p_system_identifier, p_database_name, p_database_oid,
            p_account_id, 'REVOKE', p_expected_revision, head.revision + 1, NULL, p_reason, pg_catalog.clock_timestamp())
        ON CONFLICT (command_id) DO NOTHING;
        IF NOT FOUND THEN CONTINUE; END IF;
        UPDATE public.deployment_operator_head AS h
            SET revision = head.revision + 1, receipt_id = allocated_receipt WHERE h.singleton = 1;
        RETURN QUERY SELECT allocated_receipt, head.revision + 1, false;
        RETURN;
    END LOOP;
END
$body$;

ALTER FUNCTION public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint) OWNER TO console_account_owner;
ALTER FUNCTION public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint) TO console_account_owner, console_auth_startup;
GRANT EXECUTE ON FUNCTION public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text) TO console_account_owner, console_auth_startup;
"""

"""Private canonical-generator addition. No SQL execution or profile pinning."""

def deployment228_snapshot_query() -> str:
    """Reuse the exact historical serializer and extend only declared228 scope.

    Native227 input stays immutable. Fixed-source anchors are cardinality checked;
    a drifted generator requires review instead of silently losing a census.
    Final rights validity is false for staging. A final profile must require true,
    not merely pin the hash of whatever capabilities were observed in a fixture.
    """
    native_query = NATIVE_SNAPSHOT_QUERY
    wanted = " ('account_context_candidates')\n), relations AS ("
    routine_start = "), routine_records AS (\n"
    routine_owners = " WHERE p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN ("
    protected = " SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt')\n), related_fks AS ("
    snapshot = "), snapshots AS (\n SELECT jsonb_build_object(\n  'legacy_root_boundary'"
    for name, anchor in [('wanted', wanted), ('routine_start', routine_start), ('routine_owners', routine_owners), ('protected_roles', protected), ('snapshot', snapshot)]:
        if native_query.count(anchor) != 1:
            raise ValueError('deployment228 snapshot anchor drift: ' + name)
    query = native_query.replace(wanted,
        " ('account_context_candidates'),\n ('deployment_operator_receipts'),\n ('deployment_operator_head')\n), relations AS (")
    query = query.replace(protected,
        " SELECT * FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup')\n), related_fks AS (")
    # Local presence includes malformed overloads and either side of a builtin
    # ACL edge. A role used only by another database must not alter plain mode.
    query = query.replace(routine_start, r"""), deployment_observer_role AS (
 SELECT * FROM pg_roles WHERE rolname='console_durability_observer'
), deployment_observer_builtin_acl AS (
 SELECT a.* FROM pg_proc p
 CROSS JOIN LATERAL aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_local AS (
 SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
    WHERE n.nspname='public' AND p.proname='console_durability_observation_v1')
   OR EXISTS(SELECT 1 FROM deployment_observer_builtin_acl a
    JOIN deployment_observer_role r ON a.grantor=r.oid OR a.grantee=r.oid) AS present
), deployment_observer_active AS (
 SELECT r.* FROM deployment_observer_role r WHERE (SELECT present FROM deployment_observer_local)
), routine_records AS (
""")
    query = query.replace(routine_owners,
        " WHERE p.proowner IN (SELECT oid FROM deployment_observer_active) OR (n.nspname='public' AND p.proname='console_durability_observation_v1') OR p.proowner IN (SELECT oid FROM pg_roles WHERE rolname IN ('console_account_owner','console_terms_owner','console_credential_owner')) OR (n.nspname='public' AND p.proname IN (")
    extra = r"""), deployment_startup AS (
 SELECT role.* FROM (VALUES ('console_auth_startup')) required(name)
 LEFT JOIN pg_roles role ON role.rolname=required.name
), deployment_rights_relations AS (
 SELECT name,oid,relkind FROM relations
 UNION ALL
 SELECT required.name,c.oid,c.relkind
 FROM (VALUES ('users'),('organizations'),('groups'),('employees'),('persons'),
   ('person_revisions'),('employee_person_bindings'),('group_memberships'),('group_role_grants')) required(name)
 LEFT JOIN pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
), deployment_startup_database_settings AS (
 SELECT setting FROM pg_db_role_setting d
 JOIN deployment_startup startup ON startup.oid=d.setrole
 CROSS JOIN LATERAL unnest(d.setconfig) setting
 WHERE d.setdatabase<>0
), deployment_startup_database_overrides AS (
 SELECT wanted.key,count(actual.setting) AS override_count
 FROM (VALUES ('statement_timeout'),('idle_in_transaction_session_timeout'),('transaction_timeout')) wanted(key)
 LEFT JOIN deployment_startup_database_settings actual ON split_part(actual.setting,'=',1)=wanted.key
 GROUP BY wanted.key
), deployment_table_rights AS (
 SELECT r.name, privilege.name AS privilege,
   has_table_privilege(s.oid,r.oid,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('DELETE'),('TRUNCATE'),('REFERENCES'),('TRIGGER'),('MAINTAIN')) privilege(name)
), deployment_column_rights AS (
 SELECT r.name, a.attname, privilege.name AS privilege,
   has_column_privilege(s.oid,r.oid,a.attnum,privilege.name) AS allowed
 FROM deployment_startup s CROSS JOIN deployment_rights_relations r
 JOIN pg_attribute a ON a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped
 CROSS JOIN (VALUES ('SELECT'),('INSERT'),('UPDATE'),('REFERENCES')) privilege(name)
), deployment_function_rights AS (
 SELECT p.oid,n.nspname,p.proname,pg_get_function_identity_arguments(p.oid) AS identity_arguments,
   has_function_privilege(s.oid,p.oid,'EXECUTE') AS allowed,
   has_function_privilege(s.oid,p.oid,'EXECUTE WITH GRANT OPTION') AS grantable,
   p.oid IN (
     to_regprocedure('public.deployment_operator_designate_v1(text,text,bigint,uuid,uuid,bigint,bigint)'),
     to_regprocedure('public.deployment_operator_revoke_v1(text,text,bigint,uuid,uuid,bigint,text)')) AS expected_execute
 FROM deployment_startup s CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE p.proowner IN (SELECT oid FROM owner_roles)
    OR (n.nspname='public' AND p.prosecdef)
), deployment_mandatory_functions AS (
 SELECT required.identity,p.oid IS NOT NULL AS present,
   p.prokind='f' AND p.prosecdef AND p.prorettype NOT IN ('pg_catalog.trigger'::regtype,'pg_catalog.event_trigger'::regtype) AS callable_definer,
   checked.oid IS NOT NULL AS included, checked.allowed, checked.grantable
 FROM (VALUES
   ('public.account_context_presence_v1(uuid)'),
   ('public.account_login_consent_v1(uuid)'),
   ('public.account_registration_activate_v1(uuid,uuid,uuid,uuid,uuid,bigint,bytea,text[],bytea[])'),
   ('public.account_registration_begin_v1()'),
   ('public.account_security_lock_exclusive_v1(uuid)'),
   ('public.account_security_lock_shared_v1(uuid)'),
   ('public.account_session_logout_v1(uuid,uuid,bigint,interval)'),
   ('public.account_session_refresh_reuse_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.account_terms_current_v1()'),
   ('public.account_terms_registration_head_v1()'),
   ('public.auth_account_logout_revoke_v1(uuid,uuid,bigint,interval)'),
   ('public.auth_account_refresh_reuse_revoke_v1(uuid,uuid,uuid,bytea,bigint,interval)'),
   ('public.auth_account_registration_material_v1(uuid,uuid,uuid,uuid)'),
   ('public.auth_legacy_audit_append_v1(uuid,uuid,text,text,text,uuid,jsonb,jsonb,character,character,timestamp with time zone,uuid,text,text,text,text,text[],boolean,text)'),
   ('public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_bootstrap_seed_v1(uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_cold_start_admin_v1()'),
   ('public.auth_legacy_company_lock_v1(uuid)'),
   ('public.auth_legacy_deactivate_credentials_v1(uuid,uuid,timestamp with time zone)'),
   ('public.auth_legacy_group_passkey_flag_v1(uuid)'),
   ('public.auth_legacy_purge_company_v1(uuid)'),
   ('public.auth_legacy_purge_subjects_v1(uuid)'),
   ('public.auth_legacy_reset_credentials_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_bootstrap_replace_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone)'),
   ('public.auth_legacy_self_passkey_count_v1(uuid,uuid)'),
   ('public.auth_legacy_self_passkey_delete_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkey_state_v1(uuid,uuid,uuid)'),
   ('public.auth_legacy_self_passkeys_v1(uuid,uuid)'),
   ('public.auth_legacy_session_context_v1(uuid,uuid)'),
   ('public.auth_legacy_user_active_v1(uuid,uuid)'),
   ('public.auth_legacy_user_has_passkey_v1(uuid,uuid)'),
   ('public.group_member_org_ids(uuid,uuid)'),
   ('public.group_role_grants_for_user(uuid)'),
   ('public.platform_assign_org_to_group(uuid,uuid)'),
   ('public.platform_attach_group_of_one(uuid)'),
   ('public.platform_attach_membership(uuid,uuid)'),
   ('public.platform_create_group(text,text)'),
   ('public.platform_create_group_account(uuid,uuid,text,text,text[],text,uuid)'),
   ('public.platform_create_organization(text,text)'),
   ('public.platform_force_remove_direct_org_children(uuid)'),
   ('public.platform_force_remove_organization(uuid)'),
   ('public.platform_force_remove_organization_command(uuid,uuid,character,character,timestamp with time zone)'),
   ('public.platform_get_group(uuid)'),
   ('public.platform_get_organization(uuid)'),
   ('public.platform_list_group_accounts(uuid)'),
   ('public.platform_list_groups()'),
   ('public.platform_list_organizations()'),
   ('public.platform_mint_group_row(uuid,text,text)'),
   ('public.platform_mint_missing_group_of_one(uuid)'),
   ('public.platform_remove_org_from_group(uuid,uuid)'),
   ('public.platform_remove_organization(uuid)'),
   ('public.platform_resolve_bootstrap_org(bytea)'),
   ('public.platform_resolve_credential_org(text)'),
   ('public.platform_resolve_token_org(bytea)'),
   ('public.platform_revoke_group_role(uuid,uuid,text)'),
   ('public.platform_set_organization_status(uuid,text)'),
   ('public.platform_update_group(uuid,text,text,text)')) required(identity)
 LEFT JOIN pg_proc p ON p.oid=to_regprocedure(required.identity)
 LEFT JOIN deployment_function_rights checked ON checked.oid=p.oid
), deployment_builtin AS (
 SELECT p.oid, p.proowner, owner.rolsuper AS owner_superuser,
   jsonb_build_object(
     'identity',p.oid::regprocedure::text,'owner',jsonb_build_array('builtin_owner'),
     'owner_superuser',owner.rolsuper,'language',language.lanname,
     'kind',p.prokind,'security_definer',p.prosecdef,'strict',p.proisstrict,
     'returns_set',p.proretset,'leakproof',p.proleakproof,
     'volatility',p.provolatile,'parallel',p.proparallel,
     'result',pg_get_function_result(p.oid),'config',p.proconfig,
     'source_sha256',encode(sha256(convert_to(p.prosrc,'UTF8')),'hex'),
     'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
         CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       ORDER BY CASE WHEN a.grantor=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
         CASE WHEN a.grantee=0 THEN jsonb_build_array('public') WHEN a.grantee=p.proowner THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
         a.privilege_type,a.is_grantable)
       FROM aclexplode(COALESCE(p.proacl,acldefault('f',p.proowner))) a),'[]'::jsonb),
     'account_owner_execute',has_function_privilege((SELECT oid FROM pg_roles WHERE rolname='console_account_owner'),p.oid,'EXECUTE'),
     'startup_execute',has_function_privilege((SELECT oid FROM deployment_startup),p.oid,'EXECUTE')
   ) AS record
 FROM pg_proc p JOIN pg_roles owner ON owner.oid=p.proowner
 JOIN pg_language language ON language.oid=p.prolang
 WHERE p.oid=to_regprocedure('pg_catalog.pg_control_system()')
), deployment_observer_memberships AS (
 SELECT jsonb_build_array(pg_get_userbyid(m.roleid),pg_get_userbyid(m.member),
   CASE WHEN m.roleid=(SELECT oid FROM pg_roles WHERE rolname='pg_read_all_stats')
         AND m.member=o.oid AND NOT m.admin_option AND m.inherit_option AND NOT m.set_option
         AND grantor.rolsuper AND grantor.rolcanlogin
         AND grantor.rolname NOT IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer')
     THEN jsonb_build_array('trusted_maintenance_grantor')
     ELSE jsonb_build_array('role',pg_get_userbyid(m.grantor)) END,
   m.admin_option,m.inherit_option,m.set_option) AS record
 FROM deployment_observer_active o JOIN pg_auth_members m ON m.roleid=o.oid OR m.member=o.oid
 JOIN pg_roles grantor ON grantor.oid=m.grantor
), deployment_observer_settings AS (
 SELECT CASE WHEN d.setdatabase=0 THEN 'global'
          WHEN d.setdatabase=(SELECT oid FROM pg_database WHERE datname=current_database()) THEN 'current'
          ELSE 'other' END AS scope,
        d.setconfig IS NULL AS config_is_null,
        (SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
           FROM unnest(d.setconfig) setting) AS config_hashes
 FROM deployment_observer_active o JOIN pg_db_role_setting d ON d.setrole=o.oid
), deployment_observer_external_routine_grants AS (
 SELECT jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),
    CASE WHEN p.oid=to_regprocedure('pg_catalog.pg_control_system()')
          AND a.grantor=p.proowner AND owner.rolsuper
      THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
    CASE WHEN a.grantee=0 THEN jsonb_build_array('public') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
    a.privilege_type,a.is_grantable) AS record
 FROM deployment_observer_active o CROSS JOIN pg_proc p
 JOIN pg_namespace n ON n.oid=p.pronamespace JOIN pg_roles owner ON owner.oid=p.proowner
 CROSS JOIN LATERAL aclexplode(p.proacl) a
 WHERE p.proowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)
), deployment_observer_boundary AS (
 SELECT CASE WHEN NOT (SELECT present FROM deployment_observer_local) THEN jsonb_build_object('present',false)
 ELSE jsonb_build_object(
   'present',true,
   'roles',(SELECT jsonb_agg(jsonb_build_array(rolname,rolsuper,rolcanlogin,rolinherit,
       rolbypassrls,rolcreatedb,rolcreaterole,rolreplication,rolconnlimit,rolvaliduntil,
       rolconfig IS NULL,(SELECT jsonb_agg(encode(sha256(convert_to(setting,'UTF8')),'hex') ORDER BY setting COLLATE "C")
         FROM unnest(rolconfig) setting)) ORDER BY rolname) FROM deployment_observer_active),
   'memberships',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_memberships),
   'settings',(SELECT jsonb_agg(jsonb_build_array(scope,config_is_null,config_hashes)
       ORDER BY scope,config_is_null,config_hashes::text COLLATE "C") FROM deployment_observer_settings),
   'external_routine_grants',(SELECT jsonb_agg(record ORDER BY record::text COLLATE "C") FROM deployment_observer_external_routine_grants),
   'owner_objects',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind)
       ORDER BY n.nspname,c.relname) FROM deployment_observer_active o JOIN pg_class c ON c.relowner=o.oid
       JOIN pg_namespace n ON n.oid=c.relnamespace),
   'owner_schemas',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o JOIN pg_namespace n ON n.nspowner=o.oid),
   'schema_create',(SELECT jsonb_agg(n.nspname ORDER BY n.nspname)
       FROM deployment_observer_active o CROSS JOIN pg_namespace n
       WHERE n.oid<>pg_catalog.pg_my_temp_schema() AND has_schema_privilege(o.oid,n.oid,'CREATE')),
   'default_privileges',(SELECT jsonb_agg(jsonb_build_array(pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY pg_get_userbyid(d.defaclrole),n.nspname,d.defaclobjtype,
       pg_get_userbyid(a.grantor),pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_default_acl d
       LEFT JOIN pg_namespace n ON n.oid=d.defaclnamespace CROSS JOIN LATERAL aclexplode(d.defaclacl) a
       WHERE d.defaclrole=o.oid OR a.grantee=o.oid OR a.grantor=o.oid),
   'external_table_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
       CROSS JOIN LATERAL aclexplode(c.relacl) a WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid)),
   'external_column_grants',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,col.attname,
       pg_get_userbyid(a.grantor),CASE WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable) ORDER BY n.nspname,c.relname,col.attname,pg_get_userbyid(a.grantor),
       pg_get_userbyid(a.grantee),a.privilege_type,a.is_grantable)
       FROM deployment_observer_active o CROSS JOIN pg_attribute col JOIN pg_class c ON c.oid=col.attrelid
       JOIN pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL aclexplode(col.attacl) a
       WHERE c.relowner<>o.oid AND (a.grantee=o.oid OR a.grantor=o.oid))
 ) END AS record
), deployment_boundary AS (
 SELECT jsonb_build_object(
   'builtin',(SELECT record FROM deployment_builtin),
   'observer',(SELECT record FROM deployment_observer_boundary),
   'startup_table_rights',(SELECT jsonb_agg(jsonb_build_array(name,privilege,allowed) ORDER BY name,privilege) FROM deployment_table_rights),
   'startup_column_rights',(SELECT jsonb_agg(jsonb_build_array(name,attname,privilege,allowed) ORDER BY name,attname,privilege) FROM deployment_column_rights),
   'startup_function_rights',(SELECT jsonb_agg(jsonb_build_array(nspname,proname,identity_arguments,allowed,grantable,expected_execute) ORDER BY nspname,proname,identity_arguments) FROM deployment_function_rights),
   'startup_mandatory_function_rights',(SELECT jsonb_agg(jsonb_build_array(identity,present,callable_definer,included,allowed,grantable) ORDER BY identity) FROM deployment_mandatory_functions),
   'startup_managed_database_overrides',(SELECT jsonb_agg(jsonb_build_array(key,override_count) ORDER BY key) FROM deployment_startup_database_overrides),
   'startup_final_rights_valid',
     (SELECT count(*)=1 AND bool_and(oid IS NOT NULL AND rolcanlogin AND NOT rolsuper AND NOT rolinherit AND NOT rolbypassrls AND NOT rolcreatedb AND NOT rolcreaterole AND NOT rolreplication) FROM deployment_startup)
     AND NOT EXISTS(SELECT 1 FROM pg_auth_members m WHERE m.roleid=(SELECT oid FROM deployment_startup) OR m.member=(SELECT oid FROM deployment_startup))
     AND (SELECT count(*)=17 AND count(oid)=17 AND bool_and(relkind='r') FROM relations)
     AND (SELECT count(*)=3 AND bool_and(override_count=0) FROM deployment_startup_database_overrides)
     AND (SELECT count(*)=26 AND count(oid)=26 AND bool_and(relkind='r') FROM deployment_rights_relations)
     AND (SELECT count(*)=208 AND bool_and(allowed IS FALSE) FROM deployment_table_rights)
     AND (SELECT count(*)>0 AND count(DISTINCT name)=26 AND bool_and(allowed IS FALSE) FROM deployment_column_rights)
     AND (SELECT count(*)>0 AND count(*) FILTER (WHERE expected_execute)=2
          AND bool_and(allowed IS NOT DISTINCT FROM expected_execute AND grantable IS FALSE) FROM deployment_function_rights)
     AND (SELECT count(*)=57 AND bool_and(present AND callable_definer IS TRUE AND included AND allowed IS FALSE AND grantable IS FALSE) FROM deployment_mandatory_functions)
     AND (SELECT count(*)=1 AND bool_and(owner_superuser
          AND record->'account_owner_execute'='true'::jsonb
          AND record->'startup_execute'='false'::jsonb) FROM deployment_builtin)
 ) AS record
), snapshots AS (
 SELECT jsonb_build_object(
  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),
  'legacy_root_boundary'"""
    return query.replace(snapshot, extra)


# Additive, source-bound Company profiles. Historical Account serializers and
# profile identifiers above remain unchanged and retain their original meaning.
COMPANY_CUSTODY_PRIOR_SHA256 = (
    '912c7a57f185790aa75665c9aade23f4cb45a81763a3be9e96b366d3672c686b',
    'ce2204d8fda7a316b2f5fa9cce6ac30d454180569aaef873e385f0c381eca100',
)
COMPANY_CUSTODY_FINALIZED_SHA256 = (
    '884e9a57e9ecfd9940b3c6e8e9a9a4a9c028ea62e43a10a6b11c64dada3fd62b',
    '0b0e857e53ab1122a2879b972cd701732c5d0296c20f4693ab813894b0bc72dc',
)

COMPANY_CUSTODY_CREATED_RELATIONS = (
    'company_authority_heads',
    'company_enrollment_effect_bindings',
    'company_enrollment_receipts',
    'company_enrollment_request_events',
    'company_enrollment_requests',
    'group_authority_heads',
    'group_membership_revisions',
    'native_company_action_refs',
    'native_company_catalog_installs',
    'native_company_object_refs',
    'native_company_property_refs',
    'platform_force_removal_effect_bindings',
    'platform_force_removal_receipts',
    'platform_legacy_catalog_effect_bindings',
    'platform_legacy_membership_effect_bindings',
    'platform_legacy_topology_effect_bindings',
    'platform_legacy_topology_receipts',
    'platform_legacy_user_birth_witnesses',
    'policy_assignment_revisions',
    'policy_capability_clause_fields',
    'policy_capability_clauses',
    'policy_role_revisions',
)


def company_enrollment_state_query():
    final = ','.join("'" + value + "'" for value in COMPANY_CUSTODY_FINALIZED_SHA256)
    created = ','.join("'" + name + "'" for name in COMPANY_CUSTODY_CREATED_RELATIONS)
    return f"""WITH company_profile AS (
{company_enrollment_snapshot_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM company_profile) IN ({final})
 AND (SELECT company_startup_rights_valid FROM company_profile) IS TRUE
 THEN 'company_enrollment.finalized'
 WHEN EXISTS(SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
       WHERE n.nspname='public' AND c.relname IN ({created}))
   OR EXISTS(SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
       WHERE n.nspname='public' AND p.proname='company_enrollment_execute_v1')
 THEN 'company_enrollment.profile_mismatch'
 ELSE 'company_enrollment.absent' END AS state"""


def company_enrollment_finalizer_sql():
    """One atomic upgrade of an already finalized Account, plain or observer.

    Uses the existing operator transport. A finalized replay is read-only;
    partial, unknown or populated dormant-intake state is never adopted.
    """
    profiles = (*COMPANY_CUSTODY_PRIOR_SHA256, *COMPANY_CUSTODY_FINALIZED_SHA256)
    if (len(set(profiles)) != 4 or any(len(value) != 64
            or any(c not in '0123456789abcdef' for c in value) for value in profiles)):
        raise SystemExit('Company custody profiles require exact independent source/capture review')
    final = ','.join("'" + value + "'" for value in COMPANY_CUSTODY_FINALIZED_SHA256)
    names = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts',
        'deployment_operator_head', 'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS))
    if len(names) != 59 or len(set(names)) != len(names):
        raise ValueError('Company custody relation roster drift')
    literals = ','.join("'" + name + "'" for name in names)
    inspect = ('SELECT snapshot_sha256,company_startup_rights_valid INTO observed,rights_valid FROM (\n'
               + company_enrollment_snapshot_query() + '\n) captured;')
    return f"""-- Generated by ops/generate-account-custody.py; one atomic Company upgrade.
-- Prerequisite: exact finalized Account/Platform plain or observer profile.
-- Existing operator transport owns authentication/target/migration checks.
DO $company_custody$
DECLARE
    observed text;
    rights_valid boolean;
    expected_final text;
    relation_name text;
BEGIN
    IF session_user<>current_user OR NOT EXISTS(
        SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
       OR session_user IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
           'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
           'console_platform_force_cmd','console_account_owner','console_terms_owner',
           'console_credential_owner','console_durability_observer') THEN
        RAISE EXCEPTION 'company_custody.operator_identity_mismatch';
    END IF;
    IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
        RAISE EXCEPTION 'company_custody.unsupported_isolation';
    END IF;
    PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
    PERFORM pg_catalog.set_config('lock_timeout','1s',true);
    -- Stable ordered locks precede the exact predecessor observation. Missing
    -- future tables are expected; the complete metadata hash distinguishes them
    -- from unknown or partially installed objects. Operator DDL is serialized.
    FOR relation_name IN SELECT c.relname::text FROM pg_catalog.pg_class c
        JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
        WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname IN ({literals})
        ORDER BY c.relname COLLATE "C"
    LOOP
        EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
    END LOOP;
    {inspect}
    IF observed IN ({final}) AND rights_valid IS TRUE THEN RETURN; END IF;
    expected_final := CASE observed
        WHEN '{COMPANY_CUSTODY_PRIOR_SHA256[0]}' THEN '{COMPANY_CUSTODY_FINALIZED_SHA256[0]}'
        WHEN '{COMPANY_CUSTODY_PRIOR_SHA256[1]}' THEN '{COMPANY_CUSTODY_FINALIZED_SHA256[1]}'
        ELSE NULL END;
    IF expected_final IS NULL THEN
        RAISE EXCEPTION 'company_custody.predecessor_mismatch';
    END IF;
{company_enrollment_input_sql()}
{company_enrollment_schema_sql()}
{company_enrollment_intake_sql()}
{company_enrollment_guards_sql()}
{company_enrollment_owner_sql()}
    SET CONSTRAINTS ALL IMMEDIATE;
    {inspect}
    IF observed IS DISTINCT FROM expected_final OR rights_valid IS NOT TRUE THEN
        RAISE EXCEPTION 'company_custody.profile_mismatch';
    END IF;
END
$company_custody$;
"""

# Additive native Company policy custody. This serializer extends the exact
# Company profile; historical generators, command bytes and seals stay intact.
NATIVE_POLICY_SOURCE_ORDER = (
    'schema-v1.sql', 'shape-expansion-v1.sql', 'codec-v1.sql',
    'registration-custody-v1.sql', 'material-v1.sql', 'commands-v1.sql',
    'catalog-v1.sql', 'assignment-v1.sql', 'guards-v1.sql', 'closure-v1.sql',
    'audit-v1.sql', 'current-read-v2.sql', 'legacy-branches-v2.sql',
    'bindings-v1.sql', 'trigger-handover-v1.sql', 'acl-v1.sql',
)
NATIVE_POLICY_RELATIONS = ('native_company_policy_inputs_v1', 'native_company_policy_receipts_v1')
# Exact hashes are populated only from independently reviewed declared sources.
NATIVE_POLICY_SOURCE_SHA256 = {'acl-v1.sql': '45e09f61e7a9e9b5d4486824f7a262e91c086f696666fe5ea42fb284e3500f84', 'assignment-v1.sql': 'e26b560d2c8d0473525b738c4e7e9d7476289ef423b0fa195c2d0ce139ad0888', 'audit-v1.sql': '59c7e7f13df648b24e97dc931904c2a811394db0e066abb1d936833cd58295c3', 'bindings-v1.sql': '80f6b5dc382d3c082cd9cdc84818c7b3b3bce35a7c672826b7b3e012a008431e', 'catalog-v1.sql': '97d8a51894be8a03e2910fe55afa5f5da9c943f9a8d72b03ae2699d03165fc05', 'closure-v1.sql': '3f830d83c586b8eee14b2ae531561b830b6da66b3fcbdde6b41b0ab10aa0b381', 'codec-v1.sql': '011e0a92b4ef9c89d68444bae5214fb7da96ef096ab7c3c24a0938a27d4844c4', 'commands-v1.sql': '7067e4e958385fdc7e76022013fb39142ed23d1767c97e770ad82749fb84f851', 'current-read-v2.sql': '5995df8bbc8c402d90e55049309a3c3970131d8d6db1f93ceebc94ece4bd4879', 'guards-v1.sql': 'd7856e06a8695155d34a588b3eaf305f22c1b495207822e3621dc61f853ae91d', 'legacy-branches-v2.sql': '5fe8eb2784c66279374516075d162b0f0b9a89db238e0941de55de90de7b1a97', 'material-v1.sql': '7002e75b83e7057b7513e28d35ebbb06409e95e6d8b6722aeaf88a8bc1f1ed1d', 'registration-custody-v1.sql': '5d4847a8e9ad0ea69f5ce5f6b03a4d2971660e885aaf918bbcc0a8ebfb04e11a', 'schema-v1.sql': 'de2159505dc3e731ecd27b240f0587f27421768bb56fc02e12a7205dbcbd579d', 'shape-expansion-v1.sql': '944a7f31ac39486b57af769bbbc4bee69ce861473d532714a03a5e2cf1caff88', 'trigger-handover-v1.sql': '3e737ce2c0456023dfaa32cd5ba7f2e5ff11574eaa6c9c2b310559b0b3fa876c'}
# Retained exact predecessor profiles: never accepted as corrected runtime custody.
NATIVE_POLICY_PREVIOUS_SHA256 = (
    'ea8096dd2574571567c2d1d791798d30c9c3d10a2b6065c3f698e618564c53d2',
    '5166d303ad07aea9cc179e9f56dc51cb788f4fa743dbf4c89aaa82b5d915f005',
)

NATIVE_POLICY_GUARD_ONLY_SHA256 = (
    '8ef3574e68d4ebfc89d8f80dda4c5dac89fa4206230b3c664433ad0902b74aa0',
    'adfbb054c68208ece4bb4ddd1a97c154d053af51c6e9bd82db3a8ab42735ad0a',
)

NATIVE_POLICY_FINALIZED_SHA256 = (
    '3755792f52a4f78236a70e509f4f1546588049daa53f7545d8a101c74684d843',
    '9bde6410d51f8b665e5ca4400820e6d8a549a6c19b7b4d84feb26ef32c4bd604',
)


NATIVE_POLICY_CATALOG_REFERENCE_SQL = "-- Immutable product catalogue reference, installed only by the verified operator upgrade.\n-- Existing rows must match; replay never replaces their digest or timestamp.\nDO $native_policy_catalog_reference$\nBEGIN\n INSERT INTO public.ont_builtin_catalog_allowlist(catalog_version,manifest_digest)\n VALUES('native-payroll-collection-read-v1',decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex'))\n ON CONFLICT(catalog_version) DO NOTHING;\n IF (SELECT manifest_digest FROM public.ont_builtin_catalog_allowlist\n     WHERE catalog_version='native-payroll-collection-read-v1') IS DISTINCT FROM\n     decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex') THEN\n  RAISE EXCEPTION 'native_company_policy.catalog_reference_mismatch';\n END IF;\nEND\n$native_policy_catalog_reference$;\n"


def native_company_policy_source_sql():
    if set(NATIVE_POLICY_SOURCE_SHA256) != set(NATIVE_POLICY_SOURCE_ORDER):
        raise SystemExit('Native policy sources require independent exact review')
    parts = []
    for name in NATIVE_POLICY_SOURCE_ORDER:
        path = ROOT / 'ops/native-company-policy' / name
        if not path.is_file() or path.is_symlink():
            raise SystemExit('Native policy source must be an exact regular file: ' + name)
        source = path.read_bytes()
        if hashlib.sha256(source).hexdigest() != NATIVE_POLICY_SOURCE_SHA256[name]:
            raise SystemExit('Native policy source requires independent successor review: ' + name)
        parts.append('-- source: ' + name + '\n' + source.decode('utf-8'))
    parts.append(NATIVE_POLICY_CATALOG_REFERENCE_SQL)
    return '\n'.join(parts)


def native_company_policy_snapshot_query():
    source = native_company_policy_source_sql()
    # Enumerate all overloads of every declared successor, including unused
    # originals retained by trigger handover. No live function-name adoption.
    import re
    routine_names = sorted(set(re.findall(r'CREATE FUNCTION ([a-z_]+\.[a-z_0-9]+)\(', source)))
    if not routine_names:
        raise ValueError('Native policy routine roster is empty')
    query = company_enrollment_snapshot_query()
    # Capture nonnull system-column grants as well as every ordinary column.
    # Historical Account/Company snapshots retain their original grammar.
    column_anchor = 'FROM pg_attribute a WHERE a.attrelid=r.oid AND a.attnum>0 AND NOT a.attisdropped'
    if query.count(column_anchor) != 1:
        raise ValueError('Native policy column ACL boundary drift')
    query = query.replace(column_anchor,
        'FROM pg_attribute a WHERE a.attrelid=r.oid AND (a.attnum>0 OR (a.attnum<0 AND a.attacl IS NOT NULL)) AND NOT a.attisdropped')
    relation_anchor = " ('audit_events'),\n"
    routine_anchor = ' WHERE (n.nspname,p.proname) IN (VALUES '
    if query.count(relation_anchor) != 1 or query.count(routine_anchor) != 1:
        raise ValueError('Native policy snapshot anchor drift')
    relation_rows = ',\n'.join(" ('" + name + "')" for name in NATIVE_POLICY_RELATIONS)
    query = query.replace(relation_anchor, relation_anchor + relation_rows + ',\n')
    routines = ','.join("('" + name.replace('.', "','", 1) + "')" for name in routine_names)
    query = query.replace(routine_anchor, routine_anchor + routines + ',')
    rights_start = '), company_startup_rights AS (\n SELECT\n'
    rights_end = ' AS valid\n), snapshots AS ('
    if query.count(rights_start) != 1 or query.count(rights_end) != 1:
        raise ValueError('Native policy startup-rights boundary drift')
    prefix, rest = query.split(rights_start, 1)
    predicate, suffix = rest.split(rights_end, 1)
    for before, after in (
        ('count(*)=59 AND count(oid)=59', 'count(*)=61 AND count(oid)=61'),
        ('count(*)=68 AND count(oid)=68', 'count(*)=70 AND count(oid)=70'),
        ('count(*)=544 AND bool_and', 'count(*)=560 AND bool_and'),
        ('count(DISTINCT name)=63', 'count(DISTINCT name)=65'),
    ):
        if predicate.count(before) != 1:
            raise ValueError('Native policy startup-rights cardinality drift')
        predicate = predicate.replace(before, after)
    return (prefix + rights_start + predicate + rights_end + suffix).replace(
        'AS company_startup_rights_valid FROM snapshots', 'AS native_policy_startup_rights_valid FROM snapshots')


def native_company_policy_profiles():
    profiles = (*COMPANY_CUSTODY_FINALIZED_SHA256, *NATIVE_POLICY_PREVIOUS_SHA256, *NATIVE_POLICY_GUARD_ONLY_SHA256, *NATIVE_POLICY_FINALIZED_SHA256)
    if len(profiles) != 8 or len(set(profiles)) != 8 or any(
            len(value) != 64 or any(c not in '0123456789abcdef' for c in value) for value in profiles):
        raise SystemExit('Native policy profiles require independent declared-source capture review')
    return profiles


def native_company_policy_state_query():
    native_company_policy_profiles()
    final = ','.join("'" + value + "'" for value in NATIVE_POLICY_FINALIZED_SHA256)
    import re
    routine_names = sorted(set(re.findall(r'CREATE FUNCTION ([a-z_]+\.[a-z_0-9]+)\(', native_company_policy_source_sql())))
    routines = ','.join("('" + name.replace('.', "','", 1) + "')" for name in routine_names)
    return f"""WITH native_profile AS (
{native_company_policy_snapshot_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM native_profile) IN ({final})
 AND (SELECT native_policy_startup_rights_valid FROM native_profile) IS TRUE
 THEN 'native_company_policy.finalized'
 WHEN to_regclass('public.native_company_policy_inputs_v1') IS NULL
 AND to_regclass('public.native_company_policy_receipts_v1') IS NULL
 AND NOT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
  WHERE (n.nspname,p.proname) IN (VALUES {routines})
    OR (n.nspname='public' AND p.proname LIKE 'native_company_policy_%'))
 AND NOT EXISTS(SELECT 1 FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid
  JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public'
   AND a.attname IN ('policy_receipt_id','current_policy_receipt_id') AND a.attnum>0 AND NOT a.attisdropped)
 THEN 'native_company_policy.absent'
 ELSE 'native_company_policy.profile_mismatch' END AS state"""


def native_company_policy_finalizer_sql():
    native_company_policy_profiles()
    source = native_company_policy_source_sql()
    # Exact declared correction; CREATE OR REPLACE preserves the existing
    # routine identity, owner and grants. No live function body is adopted.
    start = source.index('CREATE FUNCTION public.native_company_policy_participant_receipt_v1(')
    end = source.index('$body$;', start) + len('$body$;')
    correction = source[start:end].replace('CREATE FUNCTION', 'CREATE OR REPLACE FUNCTION', 1)
    prior = ','.join("'" + value + "'" for value in (*NATIVE_POLICY_PREVIOUS_SHA256, *NATIVE_POLICY_GUARD_ONLY_SHA256))
    final = ','.join("'" + value + "'" for value in NATIVE_POLICY_FINALIZED_SHA256)
    names = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS))
    if len(names) != 61 or len(set(names)) != len(names):
        raise ValueError('Native policy custody relation roster drift')
    literals = ','.join("'" + name + "'" for name in names)
    inspect = ('SELECT snapshot_sha256,native_policy_startup_rights_valid INTO observed,rights_valid FROM (\n'
               + native_company_policy_snapshot_query() + '\n) captured;')
    previous = ('SELECT snapshot_sha256,company_startup_rights_valid INTO predecessor,prior_rights_valid FROM (\n'
                + company_enrollment_snapshot_query() + '\n) captured;')
    return f"""-- Generated additive atomic native Company policy custody upgrade.
-- Existing superuser operator transport owns target/authentication/drain checks.
DO $native_policy_custody$
DECLARE observed text; rights_valid boolean; predecessor text; prior_rights_valid boolean;
 expected_final text; relation_name text;
BEGIN
 IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
  OR session_user IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
   'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
   'console_platform_force_cmd','console_account_owner','console_terms_owner','console_credential_owner',
   'console_durability_observer') THEN RAISE EXCEPTION 'native_company_policy.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
  RAISE EXCEPTION 'native_company_policy.unsupported_isolation'; END IF;
 PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
 PERFORM pg_catalog.set_config('lock_timeout','1s',true);
 FOR relation_name IN SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname IN ({literals}) ORDER BY c.relname COLLATE "C"
 LOOP EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name); END LOOP;
 {inspect}
 IF observed IN ({final}) AND rights_valid IS TRUE THEN
{NATIVE_POLICY_CATALOG_REFERENCE_SQL}
  RETURN;
 END IF;
 IF observed IN ({prior}) AND rights_valid IS TRUE THEN
  expected_final:=CASE observed
   WHEN '{NATIVE_POLICY_PREVIOUS_SHA256[0]}' THEN '{NATIVE_POLICY_FINALIZED_SHA256[0]}'
   WHEN '{NATIVE_POLICY_PREVIOUS_SHA256[1]}' THEN '{NATIVE_POLICY_FINALIZED_SHA256[1]}'
   WHEN '{NATIVE_POLICY_GUARD_ONLY_SHA256[0]}' THEN '{NATIVE_POLICY_FINALIZED_SHA256[0]}'
   WHEN '{NATIVE_POLICY_GUARD_ONLY_SHA256[1]}' THEN '{NATIVE_POLICY_FINALIZED_SHA256[1]}' END;
{correction}
 GRANT SELECT(xmin) ON public.audit_events TO console_account_owner;
{NATIVE_POLICY_CATALOG_REFERENCE_SQL}
 ELSE
 {previous}
 expected_final:=CASE predecessor
  WHEN '{COMPANY_CUSTODY_FINALIZED_SHA256[0]}' THEN '{NATIVE_POLICY_FINALIZED_SHA256[0]}'
  WHEN '{COMPANY_CUSTODY_FINALIZED_SHA256[1]}' THEN '{NATIVE_POLICY_FINALIZED_SHA256[1]}' ELSE NULL END;
 IF expected_final IS NULL OR prior_rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_company_policy.predecessor_mismatch'; END IF;
{source}
 END IF;
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF observed IS DISTINCT FROM expected_final OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_company_policy.profile_mismatch'; END IF;
END
$native_policy_custody$;
"""


def native_company_policy_generated_files():
    return {
        'ops/postgres-native-company-policy-owner.sql': native_company_policy_source_sql(),
        'ops/postgres-capture-native-company-policy-custody.sql': native_company_policy_snapshot_query() + ';\n',
        'ops/postgres-native-company-policy-custody-state.sql': native_company_policy_state_query() + ';\n',
        'backend/app/src/native_company_policy_custody_state.sql': native_company_policy_state_query() + ';\n',
        'ops/postgres-finalize-native-company-policy.sql': native_company_policy_finalizer_sql(),
    }


NATIVE_POLICY_V2_NAMESPACE_PREDICATE = """(n.nspname='public' AND
 (starts_with(p.proname,'native_company_policy_') OR starts_with(p.proname,'native_company_people_')))
 OR (n.nspname='ontology_api' AND starts_with(p.proname,'install_native_company_people_'))"""


NATIVE_POLICY_V2_SOURCE_SHA256 = {
    'schema-v2.sql': '6377f9bac8e28fc2b4e4160f747e39fc1e8e6c50de15c27a06d8d389ee5e4aa8',
    'codec-v2.sql': '5aa8c474e02e40fcfa7e3174b2eb3d1cbb9d1c897088f8a02710d0e24241a496',
    'material-v2.sql': '0ebd2c9b5b566bc14b81f8d0224a0e2b6dca0fa2a1a2c457e7385cd989f2c05a',
    'catalog-v2.sql': '61d7da2866e366050682709d950369d5b9ab622d49bf5880652086b50ca8d367',
    'assignment-v2.sql': '3ebd08957a582e0904703c8796c504d24e2a4e907d4e3ac0e7d9521ab4d2f482',
    'guards-v2.sql': '4a30121ecefddd640caecbffd6b02ae1a7bf44f894707813892679e8f079a5af',
    'closure-v2.sql': '8c405dd8dcdb785d03ded26b5574b2294b327b85644a6707e5749fa09ab6cb69',
    'audit-v2.sql': 'd78aac42e2b00fe90a8ca6e174f8ea9f709bd925de025579f589ffcb32c6c30e',
    'current-read-v3.sql': 'd3fe94fbb9a6db4b2d6d22f789f092f6f136c49a35831ab425135a25d8873a9d',
    'commands-v2.sql': '264f41bc4b8a2c11cef08931fd3fe6455b93b08ca43a9d62706e555e171a52f2',
    'acl-v2.sql': '6f941ac9b29d0628d430bc5dee7cb6add59497d97200b2dc8c4c7441341b323f',
}


def native_company_policy_v2_source_sql():
    """Declared successor for isolated capture; not a custody finalizer."""
    parts = ['-- Generated declared-source candidate. Not a finalized custody installer.']
    for name, expected in NATIVE_POLICY_V2_SOURCE_SHA256.items():
        path = ROOT / 'ops/native-company-policy' / name
        if not path.is_file() or path.is_symlink():
            raise SystemExit('Native policy successor source must be a regular file: ' + name)
        source = path.read_bytes()
        if hashlib.sha256(source).hexdigest() != expected:
            raise SystemExit('Native policy successor source differs from reviewed bytes: ' + name)
        parts.append('-- source: ' + name + '\n' + source.decode('utf-8'))
    parts.append("-- Immutable People reference; no grants or business records are created.\nDO $people_reference$\nBEGIN\n INSERT INTO public.ont_builtin_catalog_allowlist(catalog_version,manifest_digest)\n VALUES('native-people-directory-v1',decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex'))\n ON CONFLICT(catalog_version) DO NOTHING;\n IF (SELECT manifest_digest FROM public.ont_builtin_catalog_allowlist\n     WHERE catalog_version='native-people-directory-v1') IS DISTINCT FROM\n     decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex') THEN\n  RAISE EXCEPTION 'native_company_policy.catalog_reference_mismatch';\n END IF;\nEND\n$people_reference$;\n")
    return '\n'.join(parts)


def native_company_policy_v2_snapshot_query():
    """Preserve historical serialization and capture all declared new routines."""
    import re
    names = set(re.findall(r'CREATE(?: OR REPLACE)? FUNCTION ([a-z_]+\.[a-z_0-9]+)\(',
                          native_company_policy_v2_source_sql()))
    prior = set(re.findall(r'CREATE FUNCTION ([a-z_]+\.[a-z_0-9]+)\(',
                          native_company_policy_source_sql()))
    added = sorted(names - prior)
    if len(added) != 7:
        raise ValueError('Native policy successor new routine roster drift')
    query = native_company_policy_snapshot_query()
    anchor = ' WHERE (n.nspname,p.proname) IN (VALUES '
    if query.count(anchor) != 1:
        raise ValueError('Native policy successor routine capture boundary drift')
    rows = ','.join("('" + name.replace('.', "','", 1) + "')" for name in added)
    query = query.replace(anchor, anchor + rows + ',')
    # Include unexpected routines regardless of their owner or security mode.
    boundary = '), owner_roles AS ('
    if query.count(boundary) != 1:
        raise ValueError('Native policy successor namespace capture boundary drift')
    query = query.replace(boundary, ' OR (' + NATIVE_POLICY_V2_NAMESPACE_PREDICATE + ')\n' + boundary)
    # Namespace lookup is an independent prerequisite to function EXECUTE.
    # Freeze all declared application namespaces, including missing-schema rows.
    schema_anchor = "  'deployment_operator_boundary',(SELECT record FROM deployment_boundary),"
    if query.count(schema_anchor) != 1:
        raise ValueError('Native policy successor schema capture boundary drift')
    schema_record = """  'required_schemas',(SELECT jsonb_agg(jsonb_build_object(
    'name',required.name,'present',n.oid IS NOT NULL,
    'owner',CASE WHEN required.name='pg_catalog' AND n.nspowner=(SELECT proowner FROM deployment_builtin)
       AND owner.rolsuper THEN jsonb_build_array('builtin_owner')
      ELSE jsonb_build_array('role',owner.rolname) END,
    'owner_superuser',owner.rolsuper,'acl_is_null',n.nspacl IS NULL,
    'acl',COALESCE((SELECT jsonb_agg(jsonb_build_array(
       CASE WHEN required.name='pg_catalog' AND a.grantor=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantor)) END,
       CASE WHEN a.grantee=0 THEN jsonb_build_array('public')
         WHEN required.name='pg_catalog' AND a.grantee=n.nspowner
          AND n.nspowner=(SELECT proowner FROM deployment_builtin) AND owner.rolsuper
         THEN jsonb_build_array('builtin_owner') ELSE jsonb_build_array('role',pg_get_userbyid(a.grantee)) END,
       a.privilege_type,a.is_grantable) ORDER BY
       CASE WHEN a.grantor=n.nspowner THEN '' ELSE pg_get_userbyid(a.grantor) END,
       CASE WHEN a.grantee=n.nspowner THEN '' WHEN a.grantee=0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END,
       a.privilege_type,a.is_grantable)
      FROM aclexplode(COALESCE(n.nspacl,acldefault('n',n.nspowner))) a),'[]'::jsonb),
    'effective_rights',(SELECT jsonb_agg(jsonb_build_array(r.rolname,
       has_schema_privilege(r.oid,n.oid,'USAGE'),has_schema_privilege(r.oid,n.oid,'CREATE'),
       has_schema_privilege(r.oid,n.oid,'USAGE WITH GRANT OPTION'),
       has_schema_privilege(r.oid,n.oid,'CREATE WITH GRANT OPTION')) ORDER BY r.rolname)
      FROM (SELECT oid,rolname FROM protected_roles UNION SELECT oid,rolname FROM deployment_observer_active) r)
    ) ORDER BY required.name)
    FROM (VALUES ('public'),('ontology_api'),('ont_policy_api'),('leave_api'),('pg_catalog')) required(name)
    LEFT JOIN pg_namespace n ON n.nspname=required.name LEFT JOIN pg_roles owner ON owner.oid=n.nspowner),
"""
    return query.replace(schema_anchor, schema_record + schema_anchor)


def native_company_policy_v2_capture_files():
    # Capture precedes a reviewed finalizer. Existing serving profile admission
    # and operator entrypoints remain unchanged until that successor is proven.
    return {
        'ops/postgres-native-company-policy-v2-owner.sql': native_company_policy_v2_source_sql(),
        'ops/postgres-capture-native-company-policy-v2-custody.sql': native_company_policy_v2_snapshot_query() + ';\n',
    }


# Independently captured profiles; historical serializers remain unchanged.
NATIVE_POLICY_V2_FINALIZED_SHA256 = (
    'e94251d48fdee392d7632709b19d80cc04d53ea6742e0f2d6538d68391ab8bf2',
    'f132054a56641dcb0cc5875d6485846df2091e631d0d902a18356d2202343fd9',
)


def native_company_policy_v2_profiles():
    profiles = (*NATIVE_POLICY_FINALIZED_SHA256, *NATIVE_POLICY_V2_FINALIZED_SHA256)
    if len(profiles) != 4 or len(set(profiles)) != 4 or any(
            len(value) != 64 or any(c not in '0123456789abcdef' for c in value) for value in profiles):
        raise SystemExit('Native policy v2 profiles require independent declared-source capture review')
    return profiles


def native_company_policy_v2_presence_query():
    import re
    prior = sorted(set(re.findall(r'CREATE FUNCTION ([a-z_]+\.[a-z_0-9]+)\(',
                                 native_company_policy_source_sql())))
    if not prior:
        raise ValueError('Native policy predecessor routine roster empty')
    routines = ','.join("('" + name.replace('.', "','", 1) + "')" for name in prior)
    return f"""SELECT EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE ({NATIVE_POLICY_V2_NAMESPACE_PREDICATE})
 AND (n.nspname,p.proname) NOT IN (VALUES {routines}))
 OR EXISTS(SELECT 1 FROM pg_constraint c WHERE c.conrelid IN
   (to_regclass('public.native_company_policy_inputs_v1'),to_regclass('public.native_company_policy_receipts_v1'),
    to_regclass('public.native_company_catalog_installs'),to_regclass('public.native_company_object_refs'),
    to_regclass('public.native_company_action_refs'),to_regclass('public.native_company_property_refs'))
  AND c.conname IN ('native_company_policy_inputs_v1_codec_shape_v2',
   'native_company_policy_receipts_v1_codec_shape_v2','native_company_catalog_installs_catalog_policy_shape_v2',
   'native_company_object_refs_object_key_v3','native_company_action_refs_action_key_v3',
   'native_company_property_refs_property_key_v3')) AS present"""


def native_company_policy_v2_state_query():
    native_company_policy_v2_profiles()
    final = ','.join("'" + value + "'" for value in NATIVE_POLICY_V2_FINALIZED_SHA256)
    return f"""WITH native_profile AS (
{native_company_policy_v2_snapshot_query()}
), successor_presence AS (
{native_company_policy_v2_presence_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM native_profile) IN ({final})
 AND (SELECT native_policy_startup_rights_valid FROM native_profile) IS TRUE
 THEN 'native_company_policy_v2.finalized'
 WHEN (SELECT present FROM successor_presence) IS FALSE
 THEN 'native_company_policy_v2.absent'
 ELSE 'native_company_policy_v2.profile_mismatch' END AS state"""


def native_company_policy_v2_finalizer_sql():
    native_company_policy_v2_profiles()
    source = native_company_policy_v2_source_sql()
    marker = '-- Immutable People reference; no grants or business records are created.\n'
    if source.count(marker) != 1:
        raise ValueError('Native policy successor reference boundary drift')
    reference = source[source.index(marker):]
    final = ','.join("'" + value + "'" for value in NATIVE_POLICY_V2_FINALIZED_SHA256)
    names = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS))
    if len(names) != 61 or len(set(names)) != len(names):
        raise ValueError('Native policy custody relation roster drift')
    literals = ','.join("'" + name + "'" for name in names)
    inspect = ('SELECT snapshot_sha256,native_policy_startup_rights_valid INTO observed,rights_valid FROM (\n'
               + native_company_policy_v2_snapshot_query() + '\n) captured;')
    previous = ('SELECT snapshot_sha256,native_policy_startup_rights_valid INTO predecessor,prior_rights_valid FROM (\n'
                + native_company_policy_snapshot_query() + '\n) captured;')
    return f"""-- Generated atomic native Company policy dual-codec custody upgrade.
-- Requires compatible readers before this separately operated database change.
DO $native_policy_v2_custody$
DECLARE observed text; rights_valid boolean; predecessor text; prior_rights_valid boolean;
 expected_final text; relation_name text;
BEGIN
 IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
  OR session_user IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
   'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
   'console_platform_force_cmd','console_account_owner','console_terms_owner','console_credential_owner',
   'console_durability_observer') THEN RAISE EXCEPTION 'native_company_policy_v2.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
  RAISE EXCEPTION 'native_company_policy_v2.unsupported_isolation'; END IF;
 PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
 PERFORM pg_catalog.set_config('lock_timeout','1s',true);
 FOR relation_name IN SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname IN ({literals}) ORDER BY c.relname COLLATE "C"
 LOOP EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name); END LOOP;
 {inspect}
 IF observed IN ({final}) AND rights_valid IS TRUE THEN
{reference}
  RETURN;
 END IF;
 IF ({native_company_policy_v2_presence_query()}) THEN
  RAISE EXCEPTION 'native_company_policy_v2.profile_mismatch'; END IF;
 {previous}
 expected_final:=CASE predecessor
  WHEN '{NATIVE_POLICY_FINALIZED_SHA256[0]}' THEN '{NATIVE_POLICY_V2_FINALIZED_SHA256[0]}'
  WHEN '{NATIVE_POLICY_FINALIZED_SHA256[1]}' THEN '{NATIVE_POLICY_V2_FINALIZED_SHA256[1]}' ELSE NULL END;
 IF expected_final IS NULL OR prior_rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_company_policy_v2.predecessor_mismatch'; END IF;
{source}
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF observed IS DISTINCT FROM expected_final OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_company_policy_v2.profile_mismatch'; END IF;
END
$native_policy_v2_custody$;
"""


def native_company_policy_v2_finalized_files():
    return {
        'ops/postgres-native-company-policy-v2-custody-state.sql': native_company_policy_v2_state_query() + ';\n',
        'backend/app/src/native_company_policy_v2_custody_state.sql': native_company_policy_v2_state_query() + ';\n',
        'ops/postgres-finalize-native-company-policy-v2.sql': native_company_policy_v2_finalizer_sql(),
    }




# Additive directory capture only. No serving profile/finalizer without reviewed captures.
NATIVE_DIRECTORY_SOURCE_SHA256 = {'expansion-v1.sql': 'd71c70c9c349c41fe4e90b262093c15909a304608526f3bcdb10e99637d970db', 'codec-v1.sql': '78908ae7a9b976efbebf57f6b13070c3e17a60713cb61d3219d11d4c5dfb85e6', 'current-source-v1.sql': 'da0011ca29712d345804081414d7a8bc5e12cbf64692b9086a76e9eb24f9db04', 'commands-v1.sql': 'e73265703ac95078bd6ac825ef566405244253fca6bc8b482fe8fd446062b7b2', 'guards-v1.sql': '0cd5a02bd5eefc2b2cd0b871fa7bf417ab7489ff63c83750f191af738101eaa2', 'closure-v1.sql': 'fe69b4f546bc1a71adb3db8c6c6d3012a7ebd869fb2a3b13509db82ecfb3a3b5', 'legacy-import-v1.sql': 'c675bfc7a9d7e29c94479464136fc209e12b19ac1fbd702c4c283622dc9e9f21', 'activation-v1.sql': '60a0e446bd06d69958a9dd66f792e78563bf2af26371361be6d9ea8be27b9c70'}
NATIVE_DIRECTORY_MIGRATION = 'backend/crates/platform/db/migrations/0230_native_people_directory_storage.sql'
NATIVE_DIRECTORY_MIGRATION_SHA256 = '679fa6e9c2a8056278b712ae99ca7758a43b9ddaa03038623991c153c54378a2'
NATIVE_DIRECTORY_ADDED_RELATIONS = (
    'native_people_inputs_v1', 'native_people_terminals_v1', 'employees', 'persons',
    'person_revisions', 'employee_person_bindings', 'ont_action_command_receipts',
    'employee_employment_profiles', 'employee_lifecycle_events',
    'employment_source_bindings', 'employment_revisions', 'leave_balance_import_receipts',
)


def native_people_directory_source_files():
    files = [(NATIVE_DIRECTORY_MIGRATION, NATIVE_DIRECTORY_MIGRATION_SHA256)]
    files.extend(('ops/native-people-directory/' + name, digest)
                 for name, digest in NATIVE_DIRECTORY_SOURCE_SHA256.items())
    sources = {}
    for name, expected in files:
        path = ROOT / name
        if path.is_symlink() or not path.is_file():
            raise SystemExit('Directory declared source must be a regular file: ' + name)
        raw = path.read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected:
            raise SystemExit('Directory declared source differs from reviewed bytes: ' + name)
        sources[name] = raw.decode('utf-8')
    return sources


def native_people_directory_source_sql():
    sources = native_people_directory_source_files()
    return '-- Declared source for disposable capture only; not a custody finalizer.\n' + '\n'.join(
        '-- source: ' + name + '\n' + source for name, source in sources.items()
        if name != NATIVE_DIRECTORY_MIGRATION)


def native_people_directory_routine_names():
    import re
    names = sorted(set(re.findall(r'CREATE(?: OR REPLACE)? FUNCTION ([a-z_]+\.[a-z_0-9]+)\(',
                                  '\n'.join(native_people_directory_source_files().values()))))
    if len(names) != 27:
        raise ValueError('Directory declared routine roster drift')
    return names


def native_people_directory_snapshot_query():
    query = native_company_policy_v2_snapshot_query()
    anchor = " ('audit_events'),\n"
    if query.count(anchor) != 1:
        raise ValueError('Directory capture relation anchor drift')
    query = query.replace(anchor, anchor + ''.join(" ('" + name + "'),\n"
                         for name in NATIVE_DIRECTORY_ADDED_RELATIONS), 1)
    anchor = ' WHERE (n.nspname,p.proname) IN (VALUES '
    if query.count(anchor) != 1:
        raise ValueError('Directory routine capture boundary drift')
    routines = ','.join("('" + name.replace('.', "','", 1) + "')"
                        for name in native_people_directory_routine_names())
    query = query.replace(anchor, anchor + routines + ',', 1)
    anchor = '), owner_roles AS ('
    if query.count(anchor) != 1:
        raise ValueError('Directory routine namespace boundary drift')
    query = query.replace(anchor, " OR (n.nspname='public' AND starts_with(p.proname,'native_people_'))\n" + anchor, 1)
    # Import is an existing protected writer whose retained owner must be captured.
    for before, after in (
        ("rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer')",
         "rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_ontology_writer','console_leave_definer')"),
        ("rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app')",
         "rolname IN ('console_account_owner','console_terms_owner','console_credential_owner','console_auth_rt','console_auth_startup','console_ontology_writer','console_ontology_cmd','console_platform_force_cmd','console_rt','console_app','console_leave_definer','console_leave_cmd')"),
    ):
        if query.count(before) != 1:
            raise ValueError('Directory import owner role boundary drift')
        query = query.replace(before, after, 1)
    start = '), company_startup_rights AS (\n SELECT\n'
    end = ' AS valid\n), snapshots AS ('
    if query.count(start) != 1 or query.count(end) != 1:
        raise ValueError('Directory startup rights boundary drift')
    prefix, rest = query.split(start, 1)
    predicate, suffix = rest.split(end, 1)
    for before, after in (
        ('count(*)=61 AND count(oid)=61', 'count(*)=73 AND count(oid)=73'),
        ('count(*)=70 AND count(oid)=70', 'count(*)=82 AND count(oid)=82'),
        ('count(*)=560 AND bool_and', 'count(*)=656 AND bool_and'),
        ('count(DISTINCT name)=65', 'count(DISTINCT name)=73'),
    ):
        if predicate.count(before) != 1:
            raise ValueError('Directory startup rights cardinality drift')
        predicate = predicate.replace(before, after, 1)
    query = prefix + start + predicate + end + suffix
    anchor = '), snapshots AS (\n SELECT jsonb_build_object(\n'
    if query.count(anchor) != 1:
        raise ValueError('Directory relation namespace capture boundary drift')
    query = query.replace(anchor, anchor +
        "  'native_directory_relation_namespace',(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE n.nspname='public' AND starts_with(c.relname,'native_people_')),\n", 1)
    return query.replace('AS native_policy_startup_rights_valid FROM snapshots',
                         'AS native_directory_startup_rights_valid FROM snapshots')


def native_people_directory_staged_snapshot_query():
    return '-- New closed230 extension-scope capture; NEVER an old profile projection.\n-- Existing historical classifier must independently accept the whole base.\nWITH complete_directory_capture AS (\n' + native_people_directory_snapshot_query() + "\n), extension_capture AS (\n SELECT jsonb_build_object(\n  'protocol','native_people_directory_closed230_v1',\n  'tables',(SELECT jsonb_agg(t ORDER BY t->>'name') FROM jsonb_array_elements(snapshot->'tables') t WHERE t->>'name' IN ('native_people_inputs_v1','native_people_terminals_v1','employees','persons','person_revisions','employee_person_bindings','ont_action_command_receipts')),\n  'routines',(SELECT jsonb_agg(r ORDER BY r#>>'{metadata,schema}',r#>>'{metadata,name}',r#>>'{metadata,identity_arguments}') FROM jsonb_array_elements(snapshot->'routines') r WHERE (r#>>'{metadata,schema}',r#>>'{metadata,name}') IN (VALUES ('leave_api','apply_employee_import_batch'),('public','console_employee_number_unique'),('public','identity_company_people_projection_v1'),('public','native_people_accept_snapshot_v1'),('public','native_people_assert_closed_v1'),('public','native_people_audit_guard_v1'),('public','native_people_audit_material_v1'),('public','native_people_canonical_guard_v1'),('public','native_people_current_v1'),('public','native_people_decode_v1'),('public','native_people_deferred_closure_v1'),('public','native_people_effect_digest_v1'),('public','native_people_employee_guard_v1'),('public','native_people_employee_shape_v1'),('public','native_people_encode_v1'),('public','native_people_expectations_match_v1'),('public','native_people_frame_v1'),('public','native_people_history_immutable_v1'),('public','native_people_input_guard_v1'),('public','native_people_non_directory_effect_guard_v1'),('public','native_people_preflight_v1'),('public','native_people_prepare_v1'),('public','native_people_result_v1'),('public','native_people_terminal_guard_v1'),('public','native_people_terminal_open_v1'),('public','native_people_terminal_snapshot_v1'),('public','native_people_text_valid_v1')) OR (r#>>'{metadata,schema}'='public' AND starts_with(r#>>'{metadata,name}','native_people_'))),\n  'native_relation_namespace',snapshot->'native_directory_relation_namespace'\n ) AS snapshot FROM complete_directory_capture\n)\nSELECT snapshot,encode(sha256(convert_to(snapshot::text,'UTF8')),'hex') AS snapshot_sha256\nFROM extension_capture"


def native_people_directory_capture_files():
    return {
        'ops/postgres-native-people-directory-owner.sql': native_people_directory_source_sql(),
        'ops/postgres-capture-native-people-directory-custody.sql': native_people_directory_snapshot_query() + ';\n',
        'ops/postgres-capture-native-people-directory-staged-custody.sql': native_people_directory_staged_snapshot_query() + ';\n',
    }



# New directory profiles captured from the complete declared source; historical
# serializers and fingerprints above remain unchanged. No finalizer is added.
NATIVE_DIRECTORY_CLOSED_SHA256 = 'b8799fe7ffa4c99c6e9109af5220102da6c9f03ec122240b437a26c7c363e6aa'
NATIVE_DIRECTORY_FINALIZED_SHA256 = ('e9891784422768abcb07f731adb1295c17b40c026236c1c4ea5b9993f6ddba9b', 'bf87ef1475ec983c4e1bd286337687ead135b76fe70e28f79fe8cd430a1c95bc')


def native_people_directory_fingerprints():
    values = (NATIVE_DIRECTORY_CLOSED_SHA256, *NATIVE_DIRECTORY_FINALIZED_SHA256)
    if len(NATIVE_DIRECTORY_FINALIZED_SHA256) != 2 or len(set(values)) != 3 or any(
        not isinstance(value, str) or len(value) != 64
        or any(c not in '0123456789abcdef' for c in value) for value in values
    ):
        raise SystemExit('Directory fingerprints require independently reviewed closed/active captures')
    return NATIVE_DIRECTORY_CLOSED_SHA256, NATIVE_DIRECTORY_FINALIZED_SHA256


def native_people_directory_presence_query():
    # The caller supplies both exact capture CTEs. Derive their relation scopes
    # from those records rather than maintaining a second table-name roster.
    return """WITH structural_markers AS (
 SELECT relation_namespace.nspname::text AS schema_name, c.relname::text AS relation_name
 FROM pg_constraint k JOIN pg_namespace n ON n.oid=k.connamespace
 LEFT JOIN pg_class c ON c.oid=k.conrelid
 LEFT JOIN pg_namespace relation_namespace ON relation_namespace.oid=c.relnamespace
 WHERE n.nspname='public' AND (starts_with(k.conname,'native_people_') OR k.conname IN (
  'employee_person_bindings_actor_protocol_v1','employee_person_bindings_actor_staged_user_v1',
  'employee_person_bindings_native_actor_v1','employees_native_intake_v1',
  'employees_provenance_protocol_v1','employees_provenance_staged_legacy_v1',
  'ont_action_receipts_actor_protocol_v1','ont_action_receipts_actor_staged_user_v1',
  'ont_action_receipts_native_actor_v1','person_revisions_actor_protocol_v1',
  'person_revisions_actor_staged_user_v1','person_revisions_native_actor_v1'))
 UNION ALL
 SELECT n.nspname::text,c.relname::text
 FROM pg_trigger t JOIN pg_class c ON c.oid=t.tgrelid
 JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='public' AND starts_with(t.tgname,'native_people_')
 UNION ALL
 SELECT n.nspname::text,c.relname::text
 FROM pg_policy p JOIN pg_class c ON c.oid=p.polrelid
 JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='public' AND starts_with(p.polname,'native_people_')
)
SELECT EXISTS(SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='public' AND starts_with(c.relname,'native_people_'))
 OR EXISTS(SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace
 WHERE n.nspname='public' AND (starts_with(p.proname,'native_people_')
  OR p.proname='identity_company_people_projection_v1'))
 OR EXISTS(SELECT 1 FROM pg_attribute a JOIN pg_class c ON c.oid=a.attrelid
 JOIN pg_namespace n ON n.oid=c.relnamespace
 WHERE n.nspname='public' AND a.attnum>0 AND NOT a.attisdropped
 AND ((c.relname IN ('person_revisions','employee_person_bindings','ont_action_command_receipts')
       AND a.attname IN ('actor_kind','actor_account_id'))
   OR (c.relname='employees' AND a.attname IN ('source_kind','native_command_id'))))
 OR EXISTS(SELECT 1 FROM structural_markers) AS present,
 NOT EXISTS(SELECT 1 FROM structural_markers marker WHERE NOT EXISTS(
  SELECT 1 FROM staged_capture captured
  CROSS JOIN LATERAL jsonb_array_elements(captured.snapshot->'tables') item
  WHERE marker.schema_name='public' AND item->>'name'=marker.relation_name
 )) AS closed_namespace_valid,
 NOT EXISTS(SELECT 1 FROM structural_markers marker WHERE NOT EXISTS(
  SELECT 1 FROM full_capture captured
  CROSS JOIN LATERAL jsonb_array_elements(captured.snapshot->'tables') item
  WHERE marker.schema_name='public' AND item->>'name'=marker.relation_name
 )) AS active_namespace_valid"""


def native_people_directory_state_query():
    closed, finalized = native_people_directory_fingerprints()
    finalized = ','.join("'" + value + "'" for value in finalized)
    return f"""WITH full_capture AS (
{native_people_directory_snapshot_query()}
), staged_capture AS (
{native_people_directory_staged_snapshot_query()}
), directory_presence AS (
{native_people_directory_presence_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM full_capture) IN ({finalized})
 AND (SELECT native_directory_startup_rights_valid FROM full_capture) IS TRUE
 AND (SELECT active_namespace_valid FROM directory_presence) IS TRUE
 THEN 'native_people_directory.finalized'
 WHEN (SELECT snapshot_sha256 FROM staged_capture)='{closed}'
 AND (SELECT closed_namespace_valid FROM directory_presence) IS TRUE
 THEN 'native_people_directory.staged_closed'
 WHEN (SELECT present FROM directory_presence) IS FALSE
 THEN 'native_people_directory.absent'
 ELSE 'native_people_directory.profile_mismatch' END AS state"""


def native_people_directory_classifier_files():
    query = native_people_directory_state_query() + ';\n'
    return {
        'ops/postgres-native-people-directory-custody-state.sql': query,
        'backend/app/src/native_people_directory_custody_state.sql': query,
    }


# Exact staged profiles captured independently with the closed230 extension.
# Pair order is plain, then durability observer; historical profiles are unchanged.
NATIVE_DIRECTORY_STAGED_SHA256 = (
    'eaff3623d29f22768cb8056f27dd97cb9827a71194dedc59b289744fee7fdb0f',
    'd5cad51f05a3cd9bf8abd6a4a9a5f154ff6e82df512966d333e75bba52f22761',
)


def native_people_directory_finalizer_sql():
    closed, finalized = native_people_directory_fingerprints()
    profiles = (closed, *NATIVE_DIRECTORY_STAGED_SHA256, *finalized)
    if len(NATIVE_DIRECTORY_STAGED_SHA256) != 2 or len(set(profiles)) != 5 or any(
            not isinstance(value, str) or len(value) != 64
            or any(c not in '0123456789abcdef' for c in value) for value in profiles):
        raise SystemExit('Directory finalizer requires independently reviewed paired captures')
    source = native_people_directory_source_sql()
    names = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS,
        *NATIVE_DIRECTORY_ADDED_RELATIONS))
    if len(names) != 73 or len(set(names)) != len(names):
        raise ValueError('Directory custody relation roster drift')
    literals = ','.join("'" + name + "'" for name in names)
    final = ','.join("'" + value + "'" for value in finalized)
    # Both existing complete queries share one statement snapshot. Reuse the
    # classifier, including its reserved structural markers outside full73.
    inspect = ('SELECT classified.state,captured.snapshot_sha256,captured.native_directory_startup_rights_valid\n'
               ' INTO phase,observed,rights_valid FROM (\n'
               + native_people_directory_state_query() + '\n) classified CROSS JOIN (\n'
               + native_people_directory_snapshot_query() + '\n) captured;')
    return f"""-- Generated atomic native People Directory custody activation.
-- Metadata owner only: packaged operator must validate target/TLS and lock/check
-- the exact applied migration ledger in the same transaction before this SQL.
DO $native_people_directory_custody$
DECLARE phase text; observed text; rights_valid boolean; expected_final text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
  OR session_user IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
   'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
   'console_platform_force_cmd','console_account_owner','console_terms_owner','console_credential_owner',
   'console_durability_observer') THEN RAISE EXCEPTION 'native_people_directory.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
  RAISE EXCEPTION 'native_people_directory.unsupported_isolation'; END IF;
 PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
 PERFORM pg_catalog.set_config('lock_timeout','1s',true);
 FOR relation_name IN SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname IN ({literals}) ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;
 IF locked_relations<>73 THEN RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
 {inspect}
 IF phase='native_people_directory.finalized' AND observed IN ({final}) AND rights_valid IS TRUE THEN
  RETURN;
 END IF;
 expected_final:=CASE observed
  WHEN '{NATIVE_DIRECTORY_STAGED_SHA256[0]}' THEN '{finalized[0]}'
  WHEN '{NATIVE_DIRECTORY_STAGED_SHA256[1]}' THEN '{finalized[1]}' ELSE NULL END;
 IF phase IS DISTINCT FROM 'native_people_directory.staged_closed'
  OR expected_final IS NULL OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
{source}
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF phase IS DISTINCT FROM 'native_people_directory.finalized'
  OR observed IS DISTINCT FROM expected_final OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
END
$native_people_directory_custody$;
"""


# Additive correction; original Directory sources, profiles and serializers stay frozen.
NATIVE_DIRECTORY_ROW_LOCK_SOURCE = 'ops/native-people-directory/input-row-lock-v1.sql'
NATIVE_DIRECTORY_ROW_LOCK_SOURCE_SHA256 = 'd3c48ec3134fd8f67241f0eb4a19d76b02ab51266821a1a1faec530893f67f79'
NATIVE_DIRECTORY_ROW_LOCK_FINALIZED_SHA256 = (
    'b0d8ced14929a0c4ef041dfceb57519c64663cb87f39c1dccf61d61227e2278e',
    '2c69786d88b784ca348725dc85730b9d8be1bc835e1069f64de7b7d3ec6ac80d',
)
NATIVE_DIRECTORY_V1_FINALIZER_SHA256 = 'f4f99cf873c2ab970789e44ccf9737f2dd38f6dc6b05f1849fbd4461bf6a2357'


def native_people_directory_row_lock_profiles():
    closed, prior = native_people_directory_fingerprints()
    corrected = NATIVE_DIRECTORY_ROW_LOCK_FINALIZED_SHA256
    values = (closed, *NATIVE_DIRECTORY_STAGED_SHA256, *prior, *corrected)
    if len(corrected) != 2 or len(values) != 7 or len(set(values)) != 7 or any(
            not isinstance(value, str) or len(value) != 64
            or any(c not in '0123456789abcdef' for c in value) for value in values):
        raise SystemExit('Directory row-lock correction requires independently reviewed paired captures')
    return prior, corrected


def native_people_directory_row_lock_source_sql():
    path = ROOT / NATIVE_DIRECTORY_ROW_LOCK_SOURCE
    if path.is_symlink() or not path.is_file():
        raise SystemExit('Directory row-lock source must be a regular file')
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != NATIVE_DIRECTORY_ROW_LOCK_SOURCE_SHA256:
        raise SystemExit('Directory row-lock source differs from reviewed bytes')
    return raw.decode('utf-8')


def native_people_directory_row_lock_state_query():
    prior, corrected = native_people_directory_row_lock_profiles()
    old = ','.join("'" + value + "'" for value in prior)
    final = ','.join("'" + value + "'" for value in corrected)
    # Keep the historical query byte-for-byte as a predecessor classifier.
    # Corrected profiles reuse the exact complete serializer and namespace checks.
    return f"""WITH full_capture AS (
{native_people_directory_snapshot_query()}
), staged_capture AS (
{native_people_directory_staged_snapshot_query()}
), directory_presence AS (
{native_people_directory_presence_query()}
), predecessor AS (
{native_people_directory_state_query()}
)
SELECT CASE WHEN (SELECT snapshot_sha256 FROM full_capture) IN ({final})
 AND (SELECT native_directory_startup_rights_valid FROM full_capture) IS TRUE
 AND (SELECT active_namespace_valid FROM directory_presence) IS TRUE
 THEN 'native_people_directory.finalized'
 WHEN (SELECT snapshot_sha256 FROM full_capture) IN ({old})
 AND (SELECT state FROM predecessor)='native_people_directory.finalized'
 THEN 'native_people_directory.row_lock_required'
 WHEN (SELECT state FROM predecessor) IN (
  'native_people_directory.absent','native_people_directory.staged_closed')
 THEN (SELECT state FROM predecessor)
 ELSE 'native_people_directory.profile_mismatch' END AS state"""


def native_people_directory_row_lock_finalizer_sql():
    prior, corrected = native_people_directory_row_lock_profiles()
    correction = native_people_directory_row_lock_source_sql()
    historical = native_people_directory_finalizer_sql()
    if hashlib.sha256(historical.encode('utf-8')).hexdigest() != NATIVE_DIRECTORY_V1_FINALIZER_SHA256:
        raise SystemExit('Directory v1 finalizer differs from reviewed historical bytes')
    delimiter = '$native_people_directory_v1_input$'
    if delimiter in historical:
        raise SystemExit('Directory v1 finalizer embedding delimiter collision')
    names = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS,
        *NATIVE_DIRECTORY_ADDED_RELATIONS))
    if len(names) != 73 or len(set(names)) != 73:
        raise ValueError('Directory row-lock custody relation roster drift')
    literals = ','.join("'" + name + "'" for name in names)
    final = ','.join("'" + value + "'" for value in corrected)
    inspect = ('SELECT classified.state,captured.snapshot_sha256,captured.native_directory_startup_rights_valid\n'
               ' INTO phase,observed,rights_valid FROM (\n'
               + native_people_directory_row_lock_state_query() + '\n) classified CROSS JOIN (\n'
               + native_people_directory_snapshot_query() + '\n) captured;')
    return f"""-- Generated additive Directory input row-lock custody correction.
-- The packaged operator must validate TLS/target and lock/check the exact applied
-- migration ledger in this same transaction before invoking this metadata owner.
DO $native_people_directory_row_lock_custody$
DECLARE phase text; observed text; rights_valid boolean;
 expected_prior text; expected_final text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user<>current_user OR NOT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname=session_user AND rolsuper)
  OR session_user IN ('console_app','console_rt','console_auth_rt','console_auth_startup',
   'console_leave_cmd','console_leave_definer','console_ontology_cmd','console_ontology_writer',
   'console_platform_force_cmd','console_account_owner','console_terms_owner','console_credential_owner',
   'console_durability_observer') THEN RAISE EXCEPTION 'native_people_directory.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
  RAISE EXCEPTION 'native_people_directory.unsupported_isolation'; END IF;
 PERFORM pg_catalog.set_config('search_path','pg_catalog,pg_temp',true);
 PERFORM pg_catalog.set_config('lock_timeout','1s',true);
 FOR relation_name IN SELECT c.relname::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relkind IN ('r','p') AND c.relname IN ({literals}) ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;
 IF locked_relations<>73 THEN RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
 {inspect}
 IF phase='native_people_directory.finalized' AND observed IN ({final}) AND rights_valid IS TRUE THEN
  RETURN;
 END IF;
 expected_prior:=CASE observed
  WHEN '{NATIVE_DIRECTORY_STAGED_SHA256[0]}' THEN '{prior[0]}'
  WHEN '{NATIVE_DIRECTORY_STAGED_SHA256[1]}' THEN '{prior[1]}'
  WHEN '{prior[0]}' THEN '{prior[0]}'
  WHEN '{prior[1]}' THEN '{prior[1]}' ELSE NULL END;
 expected_final:=CASE expected_prior
  WHEN '{prior[0]}' THEN '{corrected[0]}'
  WHEN '{prior[1]}' THEN '{corrected[1]}' ELSE NULL END;
 IF (phase IS DISTINCT FROM 'native_people_directory.staged_closed'
     AND phase IS DISTINCT FROM 'native_people_directory.row_lock_required')
  OR expected_prior IS NULL OR expected_final IS NULL OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
 -- Original v1 SQL executes unchanged, with the same session and retained locks.
 EXECUTE {delimiter}{historical}{delimiter};
 {inspect}
 IF phase IS DISTINCT FROM 'native_people_directory.row_lock_required'
  OR observed IS DISTINCT FROM expected_prior OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
{correction}
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF phase IS DISTINCT FROM 'native_people_directory.finalized'
  OR observed IS DISTINCT FROM expected_final OR rights_valid IS NOT TRUE THEN
  RAISE EXCEPTION 'native_people_directory.profile_mismatch'; END IF;
END
$native_people_directory_row_lock_custody$;
"""


def native_people_directory_row_lock_files():
    query = native_people_directory_row_lock_state_query() + ';\n'
    return {
        'ops/postgres-native-people-directory-row-lock-custody-state.sql': query,
        'backend/app/src/native_people_directory_row_lock_custody_state.sql': query,
        'ops/postgres-finalize-native-people-directory-row-lock.sql': native_people_directory_row_lock_finalizer_sql(),
    }


COMPANY_PROVENANCE_SOURCE_SHA256 = {
    'ops/company-enrollment/provenance-v1.sql': '58f6275c37c6b3722871b9cf89980ebae4f651fb2979f3fe97b17371d006a284',
    'ops/company-enrollment/provenance-acl-v1.sql': '9a09acfe856cf76ac4491a3460da366240cd8aa3ecf88ca052492e2bf8fb791c',
}


def company_provenance_regular_path(name, *, required):
    path = ROOT / name
    for parent in path.parents:
        if parent == ROOT:
            break
        if parent.is_symlink() or not parent.is_dir():
            raise SystemExit('Company provenance regular file path required: ' + name)
    if path.is_symlink() or (path.exists() and not path.is_file()) or (required and not path.is_file()):
        raise SystemExit('Company provenance regular file required: ' + name)
    return path


def company_provenance_capture_files():
    sources = {}
    for name, digest in COMPANY_PROVENANCE_SOURCE_SHA256.items():
        raw = company_provenance_regular_path(name, required=True).read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest:
            raise SystemExit('Company provenance source differs from reviewed bytes: ' + name)
        sources[name] = raw.decode('utf-8')
    query = native_people_directory_snapshot_query()
    anchor = '), owner_roles AS ('
    if query.count(anchor) != 1:
        raise ValueError('Company provenance routine capture boundary drift')
    query = query.replace(anchor,
        " OR (n.nspname='public' AND p.proname IN ('account_company_provenance_v1','account_company_provenance_lock_v1'))\n" + anchor)
    return {
        'ops/postgres-company-provenance-v1-owner.sql':
            '-- Generated UNINSTALLED Company provenance source; not a custody finalizer.\n'
            '-- No finalized profile or installation is authorized by this artifact.\n'
            + '\n'.join('-- source: ' + name + '\n' + source for name, source in sources.items()),
        'ops/postgres-capture-company-provenance-v1-custody.sql': query + ';\n',
    }


# Independently captured classifier-only successors; native OrgUnit stays closed.
# Pair order is plain, then durability observer. Historical profiles stay frozen.
COMPANY_PROVENANCE_INSTALLED_SHA256 = (
    'de87fafa527398d64a1930288ef1a0a56d017db6b56bc877f8b716c714afd90a',
    '8011bd8141ec1a0497319773d73bc1df97a924f99e8aa84b8ef7c9cb4c821b37',
)


def company_provenance_custody_files():
    _, predecessors = native_people_directory_row_lock_profiles()
    successors = COMPANY_PROVENANCE_INSTALLED_SHA256
    values = (*predecessors, *successors)
    if len(successors) != 2 or len(set(values)) != 4 or any(
            not isinstance(value, str) or len(value) != 64
            or any(c not in '0123456789abcdef' for c in value) for value in values):
        raise SystemExit('Company provenance custody requires independently reviewed paired captures')
    # Reuse the unchanged declared complete serializer, including source binding,
    # routine bodies/ABI/security closure and all existing startup rights.
    capture = company_provenance_capture_files()[
        'ops/postgres-capture-company-provenance-v1-custody.sql'].removesuffix(';\n')
    old = ','.join("'" + value + "'" for value in predecessors)
    installed = ','.join("'" + value + "'" for value in successors)
    query = f"""-- Generated read-only Company provenance bridge custody; no OrgUnit activation.
WITH full_capture AS (
{capture}
), staged_capture AS (
 -- Only the existing active namespace predicate below is consumed.
 SELECT snapshot FROM full_capture
), directory_presence AS (
{native_people_directory_presence_query()}
), provenance_namespace AS (
 -- Do not filter by schema, routine kind, owner or SECURITY DEFINER: an
 -- unexpected invoker outside the full serializer must still close serving.
 SELECT count(*)=0 AS absent,
 count(*)=2 AND bool_and(n.nspname='public' AND p.prokind='f' AND (
  (p.proname='account_company_provenance_v1'
   AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector)
  OR (p.proname='account_company_provenance_lock_v1'
   AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector)
 )) AS installed
 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
 WHERE starts_with(p.proname,'account_company_provenance')
)
SELECT CASE
 WHEN (SELECT absent FROM provenance_namespace) IS FALSE
  AND (SELECT installed FROM provenance_namespace) IS NOT TRUE
 THEN 'company_provenance.profile_mismatch'
 WHEN (SELECT absent FROM provenance_namespace) IS TRUE
  AND (SELECT snapshot_sha256 FROM full_capture) IN ({old})
  AND (SELECT native_directory_startup_rights_valid FROM full_capture) IS TRUE
  AND (SELECT active_namespace_valid FROM directory_presence) IS TRUE
 THEN 'company_provenance.predecessor_compatible'
 WHEN (SELECT installed FROM provenance_namespace) IS TRUE
  AND (SELECT snapshot_sha256 FROM full_capture) IN ({installed})
  AND (SELECT native_directory_startup_rights_valid FROM full_capture) IS TRUE
  AND (SELECT active_namespace_valid FROM directory_presence) IS TRUE
 THEN 'company_provenance.installed_compatible'
 WHEN (SELECT absent FROM provenance_namespace) IS TRUE
 THEN 'company_provenance.absent'
 ELSE 'company_provenance.profile_mismatch' END AS state;\n"""
    return {
        'ops/postgres-company-provenance-v1-custody-state.sql': query,
        'backend/app/src/company_provenance_v1_custody_state.sql': query,
    }


NATIVE_ORG_UNIT_CLOSED_PERIMETER_SOURCE_SHA256 = {
    'ops/native-org-unit/closed-perimeter-v1.sql': 'd57ec1df27184c7c0f8b6359a4ea72c08b7ce89e88c4d6054ca335d7e744cbcb',
}


def native_org_unit_closed_perimeter_capture_files():
    name, digest = next(iter(NATIVE_ORG_UNIT_CLOSED_PERIMETER_SOURCE_SHA256.items()))
    raw = company_provenance_regular_path(name, required=True).read_bytes()
    if hashlib.sha256(raw).hexdigest() != digest:
        raise SystemExit('Native OrgUnit source differs from reviewed bytes: ' + name)
    query = company_provenance_capture_files()[
        'ops/postgres-capture-company-provenance-v1-custody.sql']
    relation_anchor = " ('users')\n), relations AS ("
    routine_anchor = '), owner_roles AS ('
    if query.count(relation_anchor) != 1 or query.count(routine_anchor) != 1:
        raise ValueError('Native OrgUnit capture boundary drift')
    query = query.replace(relation_anchor,
        " ('users'),\n ('org_units'),\n ('org_unit_revisions'),\n"
        " ('org_unit_source_bindings')\n), relations AS (").replace(routine_anchor,
        " OR (starts_with(p.proname,'native_org_unit_') OR p.proname IN "
        "('canonical_org_structure_row_immutable','ont_action_command_receipts_immutable'))\n"
        + routine_anchor)
    return {
        'ops/postgres-native-org-unit-closed-perimeter-v1-owner.sql':
            '-- Generated UNINSTALLED native OrgUnit closed-perimeter source; not a custody finalizer.\n'
            '-- No finalized profile or installation is authorized by this artifact.\n'
            + '-- source: ' + name + '\n' + raw.decode('utf-8'),
        'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql': query,
    }


NATIVE_ORG_UNIT_CLOSED_PHASE_PAIRS = (
    ('plain',
     'de87fafa527398d64a1930288ef1a0a56d017db6b56bc877f8b716c714afd90a',
     'efc7f14dee39011c6ed5e68b97bd8374543b1307afe3d936b828ef5a52af51e7',
     'fe0f65aebe362a969202e13d79c21d4e49f75834fd7b254a16a85a270e4e3c98',
     'be4e86175dcd561150beb68db36de84fd3f224ca39d3128b1dcc958fa319a46d'),
    ('observer',
     '8011bd8141ec1a0497319773d73bc1df97a924f99e8aa84b8ef7c9cb4c821b37',
     'a47330ee0efb705f72744a93263e65d400fbe525d70b829d2c7eedf5bfa75bdf',
     'be18fc18f7bc6df0b5371ff01140d6a06c0a6a0438e7323d3eddaa7b070851e1',
     '7bf64f46c07608b2fba7a39be765a80e45db727b424caa55342103dc180dfcc9'),
)
NATIVE_ORG_UNIT_CLOSED_CAPTURE_SHA256 = {
    'ops/postgres-capture-company-provenance-v1-custody.sql':
        '0fc02c2bd70375acb0b6ddc86b66892af4069003c88dc58455347887cdf28ab2',
    'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql':
        '6be2e3d095d59bbdb9e1b932dac8da48bde261601455cdcd166c6f2a649e6010',
}


def native_org_unit_closed_custody_inputs():
    # Only independently measured phase pairs are accepted. Neither half may
    # be borrowed from another variant or inferred from checkout metadata.
    pairs = NATIVE_ORG_UNIT_CLOSED_PHASE_PAIRS
    values = [value for row in pairs for value in row[1:]]
    if (len(pairs) != 2 or [row[0] for row in pairs] != ['plain', 'observer']
            or any(len(row) != 5 for row in pairs) or len(set(values)) != 8
            or any(not isinstance(value, str) or len(value) != 64
                   or any(c not in '0123456789abcdef' for c in value) for value in values)):
        raise SystemExit('Native OrgUnit custody requires independently reviewed paired captures')
    computed = {
        **company_provenance_capture_files(),
        **native_org_unit_closed_perimeter_capture_files(),
    }
    captures = []
    for name, digest in NATIVE_ORG_UNIT_CLOSED_CAPTURE_SHA256.items():
        raw = company_provenance_regular_path(name, required=True).read_bytes()
        if (hashlib.sha256(raw).hexdigest() != digest
                or computed[name].encode('utf-8') != raw):
            raise SystemExit('Native OrgUnit capture differs from reviewed bytes: ' + name)
        captures.append(raw.decode('utf-8').removesuffix(';\n'))
    name, digest = next(iter(NATIVE_ORG_UNIT_CLOSED_PERIMETER_SOURCE_SHA256.items()))
    raw = company_provenance_regular_path(name, required=True).read_bytes()
    if hashlib.sha256(raw).hexdigest() != digest:
        raise SystemExit('Native OrgUnit source differs from reviewed bytes: ' + name)
    source = raw.decode('utf-8')
    if '$native_org_unit_closed_source$' in source or '$native_org_unit_closed_custody$' in source:
        raise SystemExit('Native OrgUnit source embedding delimiter collision')
    ledger_name = 'ops/account-custody-migrations.sha384'
    ledger = company_provenance_regular_path(ledger_name, required=True).read_bytes()
    records = ledger.splitlines(keepends=True)
    if (len(records) != 231
            or hashlib.sha256(ledger).hexdigest() !=
                '42079d3f1b8077e163960adc65f35f1959c22a67bf42acf43d6b816721ba1357'
            or hashlib.sha256(b''.join(records[:230])).hexdigest() !=
                '25e02488cdaf864f6d15ee21d62df98eb263bb82de1e2a283470ca659d160325'):
        raise SystemExit('Native OrgUnit migration ledger differs from reviewed bytes')
    migrations = sorted((ROOT / 'backend/crates/platform/db/migrations').glob('*.sql'))
    actual = ''.join(str(int(path.name.split('_', 1)[0])) + '\t'
                     + hashlib.sha384(company_provenance_regular_path(
                         str(path.relative_to(ROOT)), required=True).read_bytes()).hexdigest() + '\n'
                     for path in migrations)
    if actual.encode('utf-8') != ledger:
        raise SystemExit('Native OrgUnit migration sources differ from reviewed bytes')
    expected_migrations = [(int(version), checksum)
                           for version, checksum in
                           (line.decode('utf-8').strip().split('\t') for line in records)]
    if [version for version, _ in expected_migrations] != list(range(1, 232)):
        raise SystemExit('Native OrgUnit migration ledger roster differs from reviewed bytes')
    return pairs, captures, source, expected_migrations


def native_org_unit_closed_values(rows):
    return 'VALUES\n ' + ',\n '.join(
        '(' + ','.join(str(value) if isinstance(value, int)
                        else "'" + value.replace("'", "''") + "'" for value in row) + ')'
        for row in rows)


def native_org_unit_closed_state_query(pairs, captures):
    columns = [
        (relation, number, name)
        for relation, names in (
            ('org_unit_revisions', ('org_id', 'id', 'org_unit_id', 'version', 'command_id',
                'actor_id', 'payload_digest', 'attributes', 'receipt', 'created_at')),
            ('org_unit_source_bindings', ('org_id', 'source_kind', 'source_id', 'org_unit_id',
                'actor_id', 'payload_digest', 'created_at')),
            ('org_units', ('org_id', 'id', 'created_at')))
        for number, name in enumerate(names, 1)]
    return f"""-- Generated read-only closed OrgUnit custody. No native writer is admitted.
-- Original73 TRUE remains authoritative; wider76 raw FALSE is diagnostic only.
WITH original73 AS (
{captures[0]}
), wider76 AS (
{captures[1]}
), phase_pairs(variant,predecessor73,predecessor76,closed73,closed76) AS (
{native_org_unit_closed_values(pairs)}
), matching_phase AS (
 SELECT p.variant,'closed'::text AS phase
 FROM phase_pairs p CROSS JOIN original73 o CROSS JOIN wider76 w
 WHERE o.snapshot_sha256=p.closed73 AND w.snapshot_sha256=p.closed76
 UNION ALL
 SELECT p.variant,'predecessor'::text AS phase
 FROM phase_pairs p CROSS JOIN original73 o CROSS JOIN wider76 w
 WHERE o.snapshot_sha256=p.predecessor73 AND w.snapshot_sha256=p.predecessor76
), required_org_relations(name) AS (
{native_org_unit_closed_values([(name,) for name in ('org_unit_revisions', 'org_unit_source_bindings', 'org_units')])}
), required_org_columns(name,attnum,attname) AS (
{native_org_unit_closed_values(columns)}
), startup_role AS (
 SELECT oid,rolname FROM pg_catalog.pg_roles WHERE rolname='console_auth_startup'
), org_relations AS (
 SELECT required.name,c.oid,c.relkind,c.relispartition,n.nspname,r.rolname AS owner_name
 FROM required_org_relations required
 LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
 LEFT JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
), org_columns AS (
 SELECT r.name,r.oid,a.attnum,a.attname,a.atttypid,a.atttypmod,a.attnotnull
 FROM org_relations r JOIN pg_catalog.pg_attribute a ON a.attrelid=r.oid
 WHERE a.attnum>0 AND NOT a.attisdropped
), table_privileges(privilege) AS (
{native_org_unit_closed_values([(v,) for v in ('SELECT', 'INSERT', 'UPDATE', 'DELETE', 'TRUNCATE', 'REFERENCES', 'TRIGGER', 'MAINTAIN')])}
), column_privileges(privilege) AS (
{native_org_unit_closed_values([(v,) for v in ('SELECT', 'INSERT', 'UPDATE', 'REFERENCES')])}
), table_checks AS (
 SELECT r.name,r.oid,p.privilege,
  pg_catalog.has_table_privilege(s.oid,r.oid,p.privilege) AS allowed
 FROM startup_role s CROSS JOIN org_relations r CROSS JOIN table_privileges p
), column_checks AS (
 SELECT c.name,c.oid,c.attnum,c.attname,p.privilege,
  pg_catalog.has_column_privilege(s.oid,c.oid,c.attnum,p.privilege) AS allowed
 FROM startup_role s CROSS JOIN org_columns c CROSS JOIN column_privileges p
), added3_valid AS (
 SELECT (SELECT count(*) FROM startup_role)=1
  AND (SELECT bool_and(oid IS NOT NULL AND oid>0 AND rolname='console_auth_startup')
       FROM startup_role) IS TRUE
  AND (SELECT count(*) FROM org_relations)=3
  AND (SELECT count(DISTINCT oid) FROM org_relations)=3
  AND (SELECT bool_and(oid IS NOT NULL AND oid>0 AND relkind='r'
       AND NOT relispartition AND nspname='public' AND owner_name='console_app')
       FROM org_relations) IS TRUE
  AND (SELECT count(*) FROM org_columns)=20
  AND (SELECT count(DISTINCT (oid,attnum)) FROM org_columns)=20
  AND (SELECT bool_and(oid IS NOT NULL AND oid>0 AND attnum>0 AND attname IS NOT NULL
       AND atttypid>0 AND attnotnull IS NOT NULL) FROM org_columns) IS TRUE
  AND (SELECT count(*) FROM org_columns c JOIN required_org_columns e
       ON e.name=c.name AND e.attnum=c.attnum AND e.attname=c.attname)=20
  AND (SELECT count(*) FROM table_checks)=24
  AND (SELECT count(DISTINCT (oid,privilege)) FROM table_checks)=24
  AND (SELECT count(*) FROM column_checks)=80
  AND (SELECT count(DISTINCT (oid,attnum,privilege)) FROM column_checks)=80
  AND (SELECT bool_and(allowed IS FALSE) FROM table_checks) IS TRUE
  AND (SELECT bool_and(allowed IS FALSE) FROM column_checks) IS TRUE AS valid
), reserved_relations AS (
 SELECT count(*)=0 AS valid FROM pg_catalog.pg_class c
 JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
 WHERE c.relname='native_org_unit' OR starts_with(c.relname,'native_org_unit_')
), reserved_schemas AS (
 SELECT count(*)=0 AS valid FROM pg_catalog.pg_namespace n
 WHERE n.nspname='native_org_unit' OR starts_with(n.nspname,'native_org_unit_')
), required_guarded_relations(name) AS (
{native_org_unit_closed_values([(name,) for name in ('ont_action_command_receipts', 'org_unit_revisions', 'org_unit_source_bindings', 'org_units')])}
), guarded_relations_valid AS (
 SELECT count(*)=4 AND count(DISTINCT c.oid)=4
  AND count(DISTINCT n.oid)=1 AND count(DISTINCT r.oid)=1
  AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
       AND NOT c.relispartition AND n.nspname='public' AND r.rolname='console_app')
       IS TRUE AS valid
 FROM required_guarded_relations required
 LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
 LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
 LEFT JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
), native_org_routine_namespace AS (
 -- Inspect every schema/kind/owner, supplementing the frozen complete captures.
 SELECT ((SELECT count(*)=0 OR (count(*)=1 AND bool_and(
    n.nspname='public' AND p.proname='native_org_unit_closed_guard_v1'
    AND p.prokind='f' AND p.pronargs=0 AND p.prorettype='pg_catalog.trigger'::regtype)) IS TRUE
   FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
   WHERE starts_with(p.proname,'native_org_unit_')) IS TRUE
  AND (SELECT count(*)=2 AND bool_and(n.nspname='public' AND p.prokind='f' AND (
    (p.proname='account_company_provenance_v1'
     AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid]::oidvector)
    OR (p.proname='account_company_provenance_lock_v1'
     AND p.proargtypes=ARRAY['pg_catalog.uuid'::regtype::oid,'pg_catalog.uuid'::regtype::oid]::oidvector)
   )) IS TRUE
   FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid=p.pronamespace
   WHERE starts_with(p.proname,'account_company_provenance')) IS TRUE) AS valid
)
SELECT CASE
 WHEN (SELECT native_directory_startup_rights_valid FROM original73) IS TRUE
  AND (SELECT valid FROM added3_valid) IS TRUE
  AND (SELECT valid FROM guarded_relations_valid) IS TRUE
  AND (SELECT valid FROM reserved_relations) IS TRUE
  AND (SELECT valid FROM reserved_schemas) IS TRUE
  AND (SELECT valid FROM native_org_routine_namespace) IS TRUE
  AND (SELECT count(*) FROM matching_phase)=1
 THEN CASE (SELECT matching_phase.phase FROM matching_phase)
  WHEN 'closed' THEN 'native_org_unit.closed_perimeter_compatible'
  WHEN 'predecessor' THEN 'native_org_unit.closed_perimeter_required'
  ELSE 'native_org_unit.profile_mismatch' END
 ELSE 'native_org_unit.profile_mismatch' END AS state;\n"""


def native_org_unit_closed_finalizer_sql(query, source, expected_migrations):
    names = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS,
        *NATIVE_DIRECTORY_ADDED_RELATIONS, 'org_unit_revisions', 'org_unit_source_bindings', 'org_units'))
    if len(names) != 76 or len(set(names)) != 76:
        raise SystemExit('Native OrgUnit custody relation roster differs from reviewed bytes')
    required = native_org_unit_closed_values([(name,) for name in names])
    ledger = f"""IF (WITH expected_migrations(version,checksum) AS (
{native_org_unit_closed_values(expected_migrations)}
 ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
    AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
   FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_org_unit.migration_ledger_mismatch'; END IF;"""
    inspect = ('SELECT classified.state,classified.variant INTO phase,variant_name FROM (\n'
               + query.removesuffix(' AS state;\n')
               + ' AS state,(SELECT variant FROM matching_phase) AS variant\n) classified;')
    bounds = '\n'.join(f""" IF (pg_catalog.current_setting('{name}') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='{name}')
      BETWEEN 1 AND {limit}) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_org_unit.entry_bounds_mismatch'; END IF;"""
        for name, limit in (('lock_timeout', 1000), ('statement_timeout', 60000),
                           ('idle_in_transaction_session_timeout', 30000), ('transaction_timeout', 120000)))
    return f"""-- Generated isolated-fixture-only atomic closed OrgUnit finalizer.
-- Caller freezes and verifies the actual database name/OID/system_identifier;
-- owns the cluster schema/role maintenance lease; BEGINs READ COMMITTED; sets
-- LOCAL search_path=pg_catalog,pg_temp, jit=off and positive reviewed bounds in
-- a separate statement; locks the exact ledger SHARE then the actual
-- console_account_owner pg_authid row FOR UPDATE. Retain every lock on this
-- direct connection through verification and COMMIT/ROLLBACK. Role-row custody
-- is a trusted caller obligation, not a fabricated pg_locks tuple assertion.
-- Arbitrary administrator schema/role writers must be operationally absent.
-- No production transport, packaging, exposure or native writer is qualified.
DO $native_org_unit_closed_custody$
DECLARE phase text; variant_name text; expected_variant text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user IS DISTINCT FROM current_user OR current_user<>'console_buck_admin'
  OR (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) IS NOT TRUE
  OR starts_with(current_database(),'_sqlx_test_') IS NOT TRUE
  OR pg_catalog.current_setting('console.sqlx_test_bootstrap',true)
      IS DISTINCT FROM 'buck-sqlx-superuser-v1' THEN
  RAISE EXCEPTION 'native_org_unit.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR pg_catalog.current_setting('search_path') IS DISTINCT FROM 'pg_catalog, pg_temp'
  OR pg_catalog.current_setting('jit') IS DISTINCT FROM 'off' THEN
  RAISE EXCEPTION 'native_org_unit.entry_settings_mismatch'; END IF;
{bounds}
 IF (SELECT count(*)=1 AND bool_and(c.oid IS NOT NULL AND c.oid>0
      AND c.relkind='r' AND NOT c.relispartition AND r.rolname='console_app') IS TRUE
     FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
     JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
     WHERE n.nspname='public' AND c.relname='_sqlx_migrations') IS NOT TRUE
  OR (SELECT count(*)=1 FROM pg_catalog.pg_locks
      WHERE pid=pg_backend_pid() AND locktype='relation'
       AND relation=pg_catalog.to_regclass('public._sqlx_migrations')
       AND mode='ShareLock' AND granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_org_unit.migration_ledger_lock_missing'; END IF;
 {ledger}
 IF (WITH required_relations(name) AS (
{required}
 ) SELECT count(*)=76 AND count(DISTINCT c.oid)=76 AND count(DISTINCT n.oid)=1
    AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
        AND NOT c.relispartition AND n.nspname='public') IS TRUE
   FROM required_relations required
   LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_org_unit.profile_mismatch'; END IF;
 FOR relation_name IN WITH required_relations(name) AS (
{required}
 ) SELECT c.relname::text FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;
 IF locked_relations<>76 OR (WITH required_relations(name) AS (
{required}
 ) SELECT count(*)=76 AND count(DISTINCT l.relation)=76
   FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   JOIN pg_catalog.pg_locks l ON l.relation=c.oid
   WHERE l.pid=pg_backend_pid() AND l.locktype='relation'
    AND l.mode='AccessExclusiveLock' AND l.granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_org_unit.relation_locks_missing'; END IF;
 {inspect}
 IF phase='native_org_unit.closed_perimeter_compatible' THEN
  {ledger}
  RETURN; END IF;
 IF phase IS DISTINCT FROM 'native_org_unit.closed_perimeter_required'
  OR variant_name IS NULL THEN RAISE EXCEPTION 'native_org_unit.profile_mismatch'; END IF;
 expected_variant:=variant_name;
 EXECUTE $native_org_unit_closed_source${source}$native_org_unit_closed_source$;
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF phase IS DISTINCT FROM 'native_org_unit.closed_perimeter_compatible'
  OR variant_name IS DISTINCT FROM expected_variant THEN
  RAISE EXCEPTION 'native_org_unit.profile_mismatch'; END IF;
 {ledger}
END
$native_org_unit_closed_custody$;
"""


def native_org_unit_closed_perimeter_custody_files():
    pairs, captures, source, migrations = native_org_unit_closed_custody_inputs()
    query = native_org_unit_closed_state_query(pairs, captures)
    return {
        'ops/postgres-native-org-unit-closed-perimeter-v1-custody-state.sql': query,
        'backend/app/src/native_org_unit_closed_perimeter_v1_custody_state.sql': query,
        'ops/postgres-finalize-native-org-unit-closed-perimeter-v1.sql':
            native_org_unit_closed_finalizer_sql(query, source, migrations),
    }


# Additive capture inputs, in the reviewed dependency order. No installed
# fingerprint or serving profile is inferred from these source identities.
NATIVE_GROUP_PROCESS_SOURCE_SHA256 = {
    'ops/native-group-process/schema-v1.sql': '7b0278ac5993e7ef972740f86c0d969b3f0a0b724abd5a873f3241cd637926c4',
    'ops/native-group-process/codec-v1.sql': 'feb28474e77d423da87a0eb2e8cd1588791a2c9ecf6e6112e00afc6e12153219',
    'ops/native-group-process/result-codec-v2.sql': '032c8222d7a123ac5614ee29053662c0e543949d557c96b27b865106a6ed9e47',
    'ops/native-group-process/source-v1.sql': '6cda12aa3051768db0418b0b0aecb82a590ddfac0c753ef01d07e027720beea7',
    'ops/native-group-process/locks-v1.sql': 'a74237a27cc270a00bf8fd82595e8f3cfeb79f3f2bb2c981719cef4d0d8abe01',
    'ops/native-group-process/closure-v1.sql': '4ca58c102d9ed803e0decff8b5e4eaa130f3c594a55b251facc2e98d6042ca2f',
    'ops/native-group-process/context-v1.sql': '0b51c44a6401f5eaf5452b25458cd9cbddb1c321d10e3199133a5441e6c80a2a',
    'ops/native-group-process/material-v1.sql': '1d78d9336ca88f482bd157da2b18324e6ed3bc1374add4f898867ee753d53ff3',
    'ops/native-group-process/transition-v1.sql': '55b3fe3456e0e1d92565621856ccf9fbffda64f3b619881ef2e224f17eb7effb',
    'ops/native-group-process/guards-v1.sql': '011700957a179bb1b4d862512eb53e30c37a5fb15262e3d831eb32c8b4f17949',
    'ops/native-group-process/commands-v1.sql': '9b86d73c28b5f5f98da684b7b8da6ba8225c115228d3d7bb437f64bb133f28c7',
    'ops/native-group-process/audit-v1.sql': '5ed8aa96a668de24cd0b0daef13bc5084bf3951fac581597d3bf3c746ac4607f',
    'ops/native-group-process/discovery-v1.sql': '58908576d4790f9e7ced9b1040484de9c9594dcf4560f8439c6378cee2e61779',
    'ops/native-group-process/acl-v1.sql': '5661c038e1d07076d0ced512cdc43ce9c04e5bb918f8bf3a33c3bc0821bf30e0',
}
NATIVE_GROUP_PROCESS_COMPILED_SOURCE_SHA256 = {
    'backend/crates/platform/authz/src/group_process/process-v1.cedarschema': 'c711017368596094ad0ff9b1123eb17573722df47f1b4abb15a31ce1ce8b1e44',
    'backend/crates/platform/authz/src/group_process/process-v1.cedar': '2453684b70134124a8cf77f2d882fb7497d8c1498325fac1321ca7e616791ed8',
    'ops/native-group-process/codec-contract-v1.json': '595376f9edea8ebd5a698bd27d6f7310a470f5e5d95522d7eed7c01db8169067',
}
NATIVE_GROUP_PROCESS_NAMESPACE_EXPRESSIONS = {
    'native_group_process_relation_namespace': "(SELECT jsonb_agg(jsonb_build_array(n.nspname,c.relname,c.relkind,pg_get_userbyid(c.relowner)) ORDER BY n.nspname,c.relname) FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE starts_with(c.relname,'native_group_process') OR starts_with(c.relname,'native_group_identity_policy') OR starts_with(c.relname,'identity_native_group_process'))",
    'native_group_process_schema_namespace': "(SELECT jsonb_agg(jsonb_build_array(n.nspname,pg_get_userbyid(n.nspowner)) ORDER BY n.nspname) FROM pg_namespace n WHERE starts_with(n.nspname,'native_group_process') OR starts_with(n.nspname,'native_group_identity_policy') OR starts_with(n.nspname,'identity_native_group_process'))",
    'native_group_process_type_namespace': "(SELECT jsonb_agg(jsonb_build_array(n.nspname,t.typname,t.typtype,pg_get_userbyid(t.typowner)) ORDER BY n.nspname,t.typname) FROM pg_type t JOIN pg_namespace n ON n.oid=t.typnamespace WHERE starts_with(t.typname,'native_group_process') OR starts_with(t.typname,'native_group_identity_policy') OR starts_with(t.typname,'identity_native_group_process'))",
    'native_group_process_routine_namespace': "(SELECT jsonb_agg(jsonb_build_array(n.nspname,p.proname,pg_get_function_identity_arguments(p.oid),p.prokind,pg_get_userbyid(p.proowner)) ORDER BY n.nspname,p.proname,pg_get_function_identity_arguments(p.oid)) FROM pg_proc p JOIN pg_namespace n ON n.oid=p.pronamespace WHERE starts_with(p.proname,'native_group_process') OR starts_with(p.proname,'native_group_identity_policy') OR starts_with(p.proname,'identity_native_group_process'))",
}


def native_group_process_capture_files():
    import re
    sources = {}
    for name, digest in {**NATIVE_GROUP_PROCESS_SOURCE_SHA256,
                         **NATIVE_GROUP_PROCESS_COMPILED_SOURCE_SHA256}.items():
        raw = company_provenance_regular_path(name, required=True).read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest:
            raise SystemExit('Native Group source differs from reviewed bytes: ' + name)
        if name in NATIVE_GROUP_PROCESS_SOURCE_SHA256:
            sources[name] = raw.decode('utf-8')
    owner = ('-- Generated UNINSTALLED native Group source; not a custody finalizer.\n'
             '-- No installed profile or serving readiness is asserted by this artifact.\n'
             + '\n'.join('-- source: ' + name + '\n' + source
                         for name, source in sources.items()))
    tables = re.findall(r'CREATE TABLE\s+public\.([a-z_0-9]+)\s*\(', owner)
    routines = sorted(re.findall(r'CREATE(?: OR REPLACE)? FUNCTION\s+public\.([a-z_0-9]+)\(', owner))
    if len(tables) != 7 or len(set(tables)) != 7 or len(routines) != 41 or len(set(routines)) != 41:
        raise ValueError('Native Group source roster drift')
    query = native_org_unit_closed_perimeter_capture_files()[
        'ops/postgres-capture-native-org-unit-closed-perimeter-v1-custody.sql']
    relation_begin, relation_end = 'WITH wanted(name) AS (VALUES\n', '), relations AS (\n'
    routine_begin, routine_end = '), routine_records AS (\n', '), owner_roles AS (\n'
    rights_begin = '\n ) AS record\n), company_startup_rights AS (\n SELECT\n'
    rights_end = ' AS valid\n), snapshots AS ('
    snapshot_begin = '), snapshots AS (\n SELECT jsonb_build_object(\n'
    for anchor in (relation_begin, routine_begin, routine_end,
                   rights_begin, rights_end, snapshot_begin):
        if query.count(anchor) != 1:
            raise ValueError('Native Group predecessor capture boundary drift')
    old_relations = query.split(relation_begin, 1)[1].split(relation_end, 1)[0]
    old_routines = query.split(routine_begin, 1)[1].split(routine_end, 1)[0]
    old_rights = query.split(rights_begin, 1)[1].split(rights_end, 1)[0]
    relations = old_relations.rstrip() + ',\n ' + ',\n '.join(
        "('" + name + "')" for name in tables) + '\n'
    routines = old_routines + ' OR (n.nspname,p.proname) IN (VALUES ' + ','.join(
        "('public','" + name + "')" for name in routines) + ')\n'
    rights = old_rights
    for before, after in (
        ('count(*)=73 AND count(oid)=73', 'count(*)=83 AND count(oid)=83'),
        ('count(*)=82 AND count(oid)=82', 'count(*)=92 AND count(oid)=92'),
        ('count(*)=656 AND bool_and(allowed IS FALSE)', 'count(*)=736 AND bool_and(allowed IS FALSE)'),
        ('count(DISTINCT name)=73', 'count(DISTINCT name)=83'),
    ):
        if rights.count(before) != 1:
            raise ValueError('Native Group predecessor rights boundary drift')
        rights = rights.replace(before, after)
    namespaces = ''.join("  '" + key + "'," + expression + ',\n'
                         for key, expression in NATIVE_GROUP_PROCESS_NAMESPACE_EXPRESSIONS.items())
    for before, after in (
        (relation_begin + old_relations + relation_end, relation_begin + relations + relation_end),
        (routine_begin + old_routines + routine_end, routine_begin + routines + routine_end),
        (snapshot_begin, snapshot_begin + namespaces),
        (rights_begin + old_rights + rights_end, rights_begin + rights + rights_end),
        ('AS native_directory_startup_rights_valid FROM snapshots',
         'AS native_group_process_startup_rights_valid FROM snapshots'),
    ):
        if query.count(before) != 1:
            raise ValueError('Native Group capture replacement boundary drift')
        query = query.replace(before, after)
    return {
        'ops/postgres-native-group-process-v1-owner.sql': owner,
        'ops/postgres-capture-native-group-process-v1-custody.sql': query,
    }


# Measured and independently reviewed plain/observer pairs. These do not alter
# historical captures, their serializers, or the migration ledger.
NATIVE_GROUP_PROCESS_PHASE_PAIRS = (
    ('plain',
     'e14842248916f3d79770fba90adb18a6eb947ec4be052f04103b87acae1dd651',
     'ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862'),
    ('observer',
     '342aedf98ddcc8646cb50576abdf0cacbdd3a0cc5adf56e3931b678b91250a99',
     'cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6'),
)
NATIVE_GROUP_PROCESS_CAPTURE_SHA256 = {
    'ops/postgres-native-group-process-v1-owner.sql':
        'cbf641175a7bf589bd46fc21dc775fe2fab8b1a8ab7e46b04dee3c32422038f9',
    'ops/postgres-capture-native-group-process-v1-custody.sql':
        '3406bac381896fab4e9a1c079110d3dc770a9d89b0b564e7744034379f60cd3b',
}


def native_group_process_custody_inputs():
    import json
    pairs = NATIVE_GROUP_PROCESS_PHASE_PAIRS
    if (not isinstance(pairs, tuple) or len(pairs) != 2
            or any(not isinstance(row, tuple) or len(row) != 3 for row in pairs)
            or hashlib.sha256(json.dumps(pairs, separators=(',', ':')).encode()).hexdigest()
            != '75313fa4903cd4b1804355f4046036496eac81276afa51246f37cd47b74e2bca'):
        raise SystemExit('Native Group custody requires the reviewed measured phase pairs')
    computed = native_group_process_capture_files()
    inputs = []
    for name, digest in NATIVE_GROUP_PROCESS_CAPTURE_SHA256.items():
        raw = company_provenance_regular_path(name, required=True).read_bytes()
        if hashlib.sha256(raw).hexdigest() != digest or computed[name].encode() != raw:
            raise SystemExit('Native Group custody input differs from reviewed bytes: ' + name)
        inputs.append(raw.decode())
    # Reuse the existing complete predecessor/231-ledger/source verification.
    _, _, _, migrations = native_org_unit_closed_custody_inputs()
    if '$native_group_process_source$' in inputs[0]:
        raise SystemExit('Native Group source embedding delimiter collision')
    return pairs, inputs[1].removesuffix(';\n'), inputs[0], migrations


def native_group_process_state_query(pairs, capture):
    absence = '\n  AND '.join("snapshot->>'" + key + "' IS NULL"
                              for key in NATIVE_GROUP_PROCESS_NAMESPACE_EXPRESSIONS)
    return f"""-- Generated read-only complete Group serving custody.
-- Absent namespaces allow historical verification, never arbitrary installation.
WITH full83 AS (
{capture}
), phase_pairs(variant,predecessor83,installed83) AS (
{native_org_unit_closed_values(pairs)}
), matching_phase(variant,phase) AS (
 SELECT p.variant,'installed'::text FROM phase_pairs p CROSS JOIN full83 f
 WHERE f.snapshot_sha256=p.installed83
  AND f.native_group_process_startup_rights_valid IS TRUE
 UNION ALL
 SELECT p.variant,'predecessor'::text FROM phase_pairs p CROSS JOIN full83 f
 WHERE f.snapshot_sha256=p.predecessor83
  AND f.native_group_process_startup_rights_valid IS FALSE
), namespace_absence AS (
 SELECT {absence} AS valid FROM full83
)
SELECT CASE
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='installed'
 THEN 'native_group_process.finalized'
 WHEN (SELECT count(*) FROM matching_phase)=1
  AND (SELECT phase FROM matching_phase)='predecessor'
  AND (SELECT valid FROM namespace_absence) IS TRUE
 THEN 'native_group_process.install_required'
 WHEN (SELECT count(*) FROM matching_phase)=0
  AND (SELECT valid FROM namespace_absence) IS TRUE
 THEN 'native_group_process.absent'
 ELSE 'native_group_process.profile_mismatch' END AS state;
"""


def native_group_process_finalizer_sql(query, source, migrations):
    import re
    predecessor = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS,
        *NATIVE_DIRECTORY_ADDED_RELATIONS, 'org_unit_revisions', 'org_unit_source_bindings', 'org_units'))
    installed = sorted((*predecessor, *re.findall(r'CREATE TABLE\s+public\.([a-z_0-9]+)\s*\(', source)))
    if len(predecessor) != 76 or len(set(predecessor)) != 76 or len(installed) != 83 or len(set(installed)) != 83:
        raise SystemExit('Native Group finalizer relation roster differs')
    ledger = f"""IF (WITH expected_migrations(version,checksum) AS (
{native_org_unit_closed_values(migrations)}
 ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
    AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
   FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process.migration_ledger_mismatch'; END IF;"""
    inspect = ('SELECT classified.state,classified.variant INTO observed_phase,variant_name FROM (\n'
               + query.removesuffix(' AS state;\n')
               + ' AS state,(SELECT variant FROM matching_phase) AS variant\n) classified;')

    def lock_relations(names):
        required = native_org_unit_closed_values([(name,) for name in names])
        count = len(names)
        return f""" IF (WITH required_relations(name) AS (
{required}
 ) SELECT count(*)={count} AND count(DISTINCT c.oid)={count} AND count(DISTINCT n.oid)=1
    AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
        AND NOT c.relispartition AND n.nspname='public') IS TRUE
   FROM required_relations required
   LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process.profile_mismatch'; END IF;
 locked_relations:=0;
 FOR relation_name IN WITH required_relations(name) AS (
{required}
 ) SELECT c.relname::text FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;
 IF locked_relations<>{count} OR (WITH required_relations(name) AS (
{required}
 ) SELECT count(*)={count} AND count(DISTINCT l.relation)={count}
   FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   JOIN pg_catalog.pg_locks l ON l.relation=c.oid
   WHERE l.pid=pg_backend_pid() AND l.locktype='relation'
    AND l.mode='AccessExclusiveLock' AND l.granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process.relation_locks_missing'; END IF;
"""

    bounds = '\n'.join(f""" IF (pg_catalog.current_setting('{name}') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='{name}')
      BETWEEN 1 AND {limit}) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process.entry_bounds_mismatch'; END IF;"""
        for name, limit in (('lock_timeout', 1000), ('statement_timeout', 60000),
                           ('idle_in_transaction_session_timeout', 30000), ('transaction_timeout', 120000)))
    return f"""-- Generated isolated-fixture-only atomic Group finalizer.
-- Caller freezes/verifies actual database name/OID/system_identifier, owns the
-- cluster schema/role maintenance lease, BEGINs READ COMMITTED, sets LOCAL
-- search_path=pg_catalog,pg_temp, jit=off and positive bounded timeouts in a
-- separate statement, locks the ledger SHARE and actual console_account_owner
-- pg_authid row FOR UPDATE. Retain all locks through COMMIT/ROLLBACK. Role-row
-- custody is a trusted caller obligation; no fabricated pg_locks tuple proof.
-- This fixture protocol does not authorize production DDL or client exposure.
DO $native_group_process_custody$
DECLARE observed_phase text; variant_name text; expected_variant text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user IS DISTINCT FROM current_user OR current_user<>'console_buck_admin'
  OR (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) IS NOT TRUE
  OR starts_with(current_database(),'_sqlx_test_') IS NOT TRUE
  OR pg_catalog.current_setting('console.sqlx_test_bootstrap',true)
      IS DISTINCT FROM 'buck-sqlx-superuser-v1' THEN
  RAISE EXCEPTION 'native_group_process.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR pg_catalog.current_setting('search_path') IS DISTINCT FROM 'pg_catalog, pg_temp'
  OR pg_catalog.current_setting('jit') IS DISTINCT FROM 'off' THEN
  RAISE EXCEPTION 'native_group_process.entry_settings_mismatch'; END IF;
{bounds}
 IF (SELECT count(*)=1 AND bool_and(c.oid IS NOT NULL AND c.oid>0
      AND c.relkind='r' AND NOT c.relispartition AND r.rolname='console_app') IS TRUE
     FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
     JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
     WHERE n.nspname='public' AND c.relname='_sqlx_migrations') IS NOT TRUE
  OR (SELECT count(*)=1 FROM pg_catalog.pg_locks
      WHERE pid=pg_backend_pid() AND locktype='relation'
       AND relation=pg_catalog.to_regclass('public._sqlx_migrations')
       AND mode='ShareLock' AND granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process.migration_ledger_lock_missing'; END IF;
 {ledger}
{lock_relations(predecessor)}
 {inspect}
 IF (observed_phase IS DISTINCT FROM 'native_group_process.install_required'
     AND observed_phase IS DISTINCT FROM 'native_group_process.finalized') OR variant_name IS NULL THEN
  RAISE EXCEPTION 'native_group_process.profile_mismatch'; END IF;
 expected_variant:=variant_name;
 IF observed_phase='native_group_process.install_required' THEN
  EXECUTE $native_group_process_source${source}$native_group_process_source$;
 END IF;
{lock_relations(installed)}
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF observed_phase IS DISTINCT FROM 'native_group_process.finalized'
  OR variant_name IS DISTINCT FROM expected_variant THEN
  RAISE EXCEPTION 'native_group_process.profile_mismatch'; END IF;
 {ledger}
END
$native_group_process_custody$;
"""


def native_group_process_custody_files():
    pairs, capture, source, migrations = native_group_process_custody_inputs()
    query = native_group_process_state_query(pairs, capture)
    return {
        'ops/postgres-native-group-process-v1-custody-state.sql': query,
        'backend/app/src/native_group_process_v1_custody_state.sql': query,
        'ops/postgres-finalize-native-group-process-v1.sql':
            native_group_process_finalizer_sql(query, source, migrations),
    }


# Additive navigation correction: frozen v1 custody remains independently valid.
NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE = 'ops/native-group-process/navigation-head-revision-v1.sql'
NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256 = '83164dd8ac21cc30bbb7e6ed07c37fa184b148c4377fde22414740483cdb2f90'
NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS = (
    ('plain',
     'ec5c2d1523e69520ac32f3d253c1c222d52e1b01bba12040328502ab319d6862',
     '3f5972d2e5c1d7277e71b4f716b79a405d152ca5bbab25b614dbc1fc67c5fd7f'),
    ('observer',
     'cda9967f7b8b267e5294551314c80fd9fc1795f962450b7b0f3d6353f11043a6',
     '304e176d646edf62e8767a9f986879abc74753a3b2a8812c1a8cec8447a20b78'),
)


def native_group_process_navigation_custody_files():
    import json
    pairs = NATIVE_GROUP_PROCESS_NAVIGATION_PHASE_PAIRS
    if (not isinstance(pairs, tuple) or len(pairs) != 2
            or any(not isinstance(row, tuple) or len(row) != 3 for row in pairs)
            or hashlib.sha256(json.dumps(pairs, separators=(',', ':')).encode()).hexdigest()
            != '00b0e19c4be0978b0ed9c3c0f21cc119890a9cbb28ed0f41bab94fde452534bd'):
        raise SystemExit('Native Group navigation requires the reviewed measured phase pairs')
    # Reuse all original14/compiled3/capture/complete231 input validation.
    prior, capture, owner, _ = native_group_process_custody_inputs()
    if tuple((row[0], row[1]) for row in pairs) != tuple((row[0], row[2]) for row in prior):
        raise SystemExit('Native Group navigation predecessors differ from accepted v1 installed profiles')
    raw = company_provenance_regular_path(
        NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE, required=True).read_bytes()
    if hashlib.sha256(raw).hexdigest() != NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256:
        raise SystemExit('Native Group navigation correction differs from reviewed bytes')
    header = 'CREATE FUNCTION public.identity_native_group_process_navigation_candidates_v1(\n'
    terminator = '\n$body$;\n'
    if owner.count(header) != 1:
        raise SystemExit('Native Group navigation original function boundary drift')
    remainder = owner.split(header, 1)[1]
    original = header + remainder[:remainder.index(terminator) + len(terminator)]
    if (hashlib.sha256(original.encode()).hexdigest()
            != '2e58ddbe2c6d4ee8b82f6a96aec9d8343a97a6db9d7074759d17c9d8bf38c1db'
            or 'v_navigation_head_revision' in original):
        raise SystemExit('Native Group navigation original function differs')
    corrected = original.replace('CREATE FUNCTION', 'CREATE OR REPLACE FUNCTION', 1)
    for before, after, count in (
        ('head_revision bigint', 'v_navigation_head_revision bigint', 1),
        ('head_revision:=', 'v_navigation_head_revision:=', 5),
        ('<>head_revision::numeric', '<>v_navigation_head_revision::numeric', 2),
        ('IF head_revision>0', 'IF v_navigation_head_revision>0', 2),
        ('h.head_revision<=head_revision ORDER BY h.head_revision',
         'h.head_revision<=v_navigation_head_revision ORDER BY h.head_revision', 1),
    ):
        if corrected.count(before) != count:
            raise SystemExit('Native Group navigation exact local rename boundary drift')
        corrected = corrected.replace(before, after)
    inverse = corrected.replace('v_navigation_head_revision', 'head_revision').replace(
        'CREATE OR REPLACE FUNCTION', 'CREATE FUNCTION', 1)
    if corrected.count('v_navigation_head_revision') != 11 or inverse != original or raw != corrected.encode():
        raise SystemExit('Native Group navigation correction must preserve the exact eleven-site inverse')
    # Keep the frozen serializer and existing bounded paired-query construction.
    # Only the rights verdict, installed-v1 maintenance branch and state prefix
    # differ; neither historical query/output nor its accepted profile changes.
    query = native_group_process_state_query(pairs, capture)
    for before, after, count in (
        ('AND f.native_group_process_startup_rights_valid IS FALSE',
         'AND f.native_group_process_startup_rights_valid IS TRUE', 1),
        ("  AND (SELECT valid FROM namespace_absence) IS TRUE\n THEN 'native_group_process.install_required'",
         " THEN 'native_group_process.head_revision_required'", 1),
        ("'native_group_process.", "'native_group_process_navigation.", 4),
    ):
        if query.count(before) != count:
            raise SystemExit('Native Group navigation classifier boundary drift')
        query = query.replace(before, after)
    return {
        'ops/postgres-native-group-process-navigation-v1-custody-state.sql': query,
        'backend/app/src/native_group_process_navigation_v1_custody_state.sql': query,
    }


def native_group_process_navigation_finalizer_files():
    import re
    query = native_group_process_navigation_custody_files()[
        'ops/postgres-native-group-process-navigation-v1-custody-state.sql']
    if hashlib.sha256(query.encode()).hexdigest() != '6a7a721c434486582ffba11d79a818088c3f3a41b93248b4bb1504ecd484dc39':
        raise SystemExit('Native Group navigation finalizer classifier differs from reviewed bytes')
    raw = company_provenance_regular_path(
        NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE, required=True).read_bytes()
    if hashlib.sha256(raw).hexdigest() != NATIVE_GROUP_PROCESS_NAVIGATION_SOURCE_SHA256:
        raise SystemExit('Native Group navigation finalizer correction differs from reviewed bytes')
    source = raw.decode('utf-8')
    if '$native_group_process_navigation_source$' in source:
        raise SystemExit('Native Group navigation finalizer correction delimiter collision')
    # Reuse frozen source/complete231 validation and the original Group relation
    # derivation. All83 already exist; this correction never installs Group7.
    _, _, owner, migrations = native_group_process_custody_inputs()
    relations = sorted((*TABLES, *CREDENTIAL_TABLES, 'company_actors',
        'account_context_candidates', 'deployment_operator_receipts', 'deployment_operator_head',
        'audit_events', *COMPANY_CUSTODY_ADDITIONAL_RELATIONS, *NATIVE_POLICY_RELATIONS,
        *NATIVE_DIRECTORY_ADDED_RELATIONS, 'org_unit_revisions', 'org_unit_source_bindings', 'org_units',
        *re.findall(r'CREATE TABLE\s+public\.([a-z_0-9]+)\s*\(', owner)))
    if len(relations) != 83 or len(set(relations)) != 83:
        raise SystemExit('Native Group navigation finalizer relation roster differs')
    required = native_org_unit_closed_values([(name,) for name in relations])
    ledger = f"""IF (WITH expected_migrations(version,checksum) AS (
{native_org_unit_closed_values(migrations)}
 ) SELECT count(*)=231 AND bool_and(e.version IS NOT NULL AND m.version IS NOT NULL
    AND m.success IS TRUE AND (encode(m.checksum,'hex')=e.checksum) IS TRUE) IS TRUE
   FROM expected_migrations e FULL JOIN public._sqlx_migrations m ON m.version=e.version) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_mismatch'; END IF;"""
    inspect = ('SELECT classified.state,classified.variant INTO observed_phase,variant_name FROM (\n'
               + query.removesuffix(' AS state;\n')
               + ' AS state,(SELECT variant FROM matching_phase) AS variant\n) classified;')
    retained_locks = f"""IF locked_relations<>83 OR (WITH required_relations(name) AS (
{required}
 ) SELECT count(*)=83 AND count(DISTINCT l.relation)=83
   FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   JOIN pg_catalog.pg_locks l ON l.relation=c.oid
   WHERE l.pid=pg_backend_pid() AND l.locktype='relation'
    AND l.mode='AccessExclusiveLock' AND l.granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.relation_locks_missing'; END IF;"""
    bounds = '\n'.join(f""" IF (pg_catalog.current_setting('{name}') IS NOT NULL
  AND (SELECT setting::bigint FROM pg_catalog.pg_settings WHERE name='{name}')
      BETWEEN 1 AND {limit}) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_bounds_mismatch'; END IF;"""
        for name, limit in (('lock_timeout', 1000), ('statement_timeout', 60000),
                           ('idle_in_transaction_session_timeout', 30000), ('transaction_timeout', 120000)))
    finalizer = f"""-- Generated isolated-fixture-only atomic Group navigation correction.
-- Caller freezes/verifies actual database name/OID/system_identifier, owns the
-- cluster schema/role maintenance lease, BEGINs READ COMMITTED, sets LOCAL
-- search_path=pg_catalog,pg_temp, jit=off and positive bounded timeouts in a
-- separate statement, locks the ledger SHARE and actual console_account_owner
-- pg_authid row FOR UPDATE. Retain all locks through COMMIT/ROLLBACK. Role-row
-- custody is a trusted caller obligation; no fabricated pg_locks tuple proof.
-- This fixture protocol does not authorize production DDL or client exposure.
DO $native_group_process_navigation_custody$
DECLARE observed_phase text; variant_name text; expected_variant text;
 relation_name text; locked_relations integer:=0;
BEGIN
 IF session_user IS DISTINCT FROM current_user OR current_user<>'console_buck_admin'
  OR (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=current_user) IS NOT TRUE
  OR starts_with(current_database(),'_sqlx_test_') IS NOT TRUE
  OR pg_catalog.current_setting('console.sqlx_test_bootstrap',true)
      IS DISTINCT FROM 'buck-sqlx-superuser-v1' THEN
  RAISE EXCEPTION 'native_group_process_navigation.operator_identity_mismatch'; END IF;
 IF pg_catalog.current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR pg_catalog.current_setting('search_path') IS DISTINCT FROM 'pg_catalog, pg_temp'
  OR pg_catalog.current_setting('jit') IS DISTINCT FROM 'off' THEN
  RAISE EXCEPTION 'native_group_process_navigation.entry_settings_mismatch'; END IF;
{bounds}
 IF (SELECT count(*)=1 AND bool_and(c.oid IS NOT NULL AND c.oid>0
      AND c.relkind='r' AND NOT c.relispartition AND r.rolname='console_app') IS TRUE
     FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
     JOIN pg_catalog.pg_roles r ON r.oid=c.relowner
     WHERE n.nspname='public' AND c.relname='_sqlx_migrations') IS NOT TRUE
  OR (SELECT count(*)=1 FROM pg_catalog.pg_locks
      WHERE pid=pg_backend_pid() AND locktype='relation'
       AND relation=pg_catalog.to_regclass('public._sqlx_migrations')
       AND mode='ShareLock' AND granted) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.migration_ledger_lock_missing'; END IF;
 {ledger}
 IF (WITH required_relations(name) AS (
{required}
 ) SELECT count(*)=83 AND count(DISTINCT c.oid)=83 AND count(DISTINCT n.oid)=1
    AND bool_and(c.oid IS NOT NULL AND c.oid>0 AND c.relkind='r'
        AND NOT c.relispartition AND n.nspname='public') IS TRUE
   FROM required_relations required
   LEFT JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   LEFT JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name) IS NOT TRUE THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 locked_relations:=0;
 FOR relation_name IN WITH required_relations(name) AS (
{required}
 ) SELECT c.relname::text FROM required_relations required
   JOIN pg_catalog.pg_namespace n ON n.nspname='public'
   JOIN pg_catalog.pg_class c ON c.relnamespace=n.oid AND c.relname=required.name
   ORDER BY c.relname COLLATE "C"
 LOOP
  EXECUTE pg_catalog.format('LOCK TABLE ONLY public.%I IN ACCESS EXCLUSIVE MODE',relation_name);
  locked_relations:=locked_relations+1;
 END LOOP;
 {retained_locks}
 {inspect}
 IF (observed_phase IS DISTINCT FROM 'native_group_process_navigation.head_revision_required'
     AND observed_phase IS DISTINCT FROM 'native_group_process_navigation.finalized') OR variant_name IS NULL THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 expected_variant:=variant_name;
 IF observed_phase='native_group_process_navigation.head_revision_required' THEN
  EXECUTE $native_group_process_navigation_source${source}$native_group_process_navigation_source$;
 END IF;
 SET CONSTRAINTS ALL IMMEDIATE;
 {inspect}
 IF observed_phase IS DISTINCT FROM 'native_group_process_navigation.finalized'
  OR variant_name IS DISTINCT FROM expected_variant THEN
  RAISE EXCEPTION 'native_group_process_navigation.profile_mismatch'; END IF;
 {ledger}
 {retained_locks}
END
$native_group_process_navigation_custody$;
"""
    return {'ops/postgres-finalize-native-group-process-navigation-v1.sql': finalizer}


def main():
    arguments = sys.argv[1:]
    if arguments in (['--native-group-process-navigation-finalizer'],
                     ['--native-group-process-navigation-finalizer', '--check'],
                     ['--native-group-process-navigation-custody'],
                     ['--native-group-process-navigation-custody', '--check'],
                     ['--native-group-process-custody'], ['--native-group-process-custody', '--check'],
                     ['--native-group-process-capture'], ['--native-group-process-capture', '--check'],
                     ['--company-provenance-capture'], ['--company-provenance-capture', '--check'],
                     ['--company-provenance-custody'], ['--company-provenance-custody', '--check'],
                     ['--native-org-unit-closed-perimeter-capture'],
                     ['--native-org-unit-closed-perimeter-capture', '--check'],
                     ['--native-org-unit-closed-perimeter-custody'],
                     ['--native-org-unit-closed-perimeter-custody', '--check']):
        if arguments[0] == '--native-group-process-navigation-finalizer':
            files = native_group_process_navigation_finalizer_files()
        elif arguments[0] == '--native-group-process-navigation-custody':
            files = native_group_process_navigation_custody_files()
        elif arguments[0] == '--native-group-process-custody':
            files = native_group_process_custody_files()
        elif arguments[0] == '--native-group-process-capture':
            files = native_group_process_capture_files()
        elif arguments[0] == '--native-org-unit-closed-perimeter-custody':
            files = native_org_unit_closed_perimeter_custody_files()
        elif arguments[0] == '--native-org-unit-closed-perimeter-capture':
            files = native_org_unit_closed_perimeter_capture_files()
        else:
            files = (company_provenance_custody_files() if arguments[0] == '--company-provenance-custody'
                     else company_provenance_capture_files())
        paths = {name: company_provenance_regular_path(name, required=False) for name in files}
        if '--check' in arguments:
            for name, expected in files.items():
                if not paths[name].is_file() or paths[name].read_bytes() != expected.encode():
                    raise SystemExit('generated Company provenance artifact differs: ' + name)
        else:
            for name, expected in files.items():
                paths[name].write_bytes(expected.encode())
        return
    if arguments not in ([], ['--check']):
        raise SystemExit('usage: generate-account-custody.py [--native-group-process-navigation-finalizer | --native-group-process-navigation-custody | --native-group-process-custody | --native-group-process-capture | --company-provenance-capture | --company-provenance-custody | --native-org-unit-closed-perimeter-capture | --native-org-unit-closed-perimeter-custody] [--check]')
    for name, expected in generated_files().items():
        path = ROOT / name
        if arguments:
            if not path.is_file() or path.read_bytes() != expected.encode():
                raise SystemExit('generated Account custody artifact differs: ' + name)
        else:
            path.write_bytes(expected.encode())


if __name__ == '__main__':
    main()
