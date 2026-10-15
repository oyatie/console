-- Uninstalled Account-only legacy removal owner successors, revision 1.
-- Apply after legacy-removal-guard-v1.sql in the same exact-predecessor transaction.
-- No Company head/schema dependency is introduced. Later bridges remain separate.
-- CREATE OR REPLACE below retains each existing console_app owner and caller ACL.

CREATE FUNCTION public.account_legacy_actor_presence_v1(p_company uuid) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET app.current_org=''
AS $actor_presence$
BEGIN
    IF p_company IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004',MESSAGE='auth_legacy.null_identity';
    END IF;
    -- The function SET clause restores the caller's GUC on return or error.
    PERFORM pg_catalog.set_config('app.current_org',p_company::text,true);
    RETURN EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company);
END;
$actor_presence$;
ALTER FUNCTION public.account_legacy_actor_presence_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_legacy_actor_presence_v1(uuid)
    FROM PUBLIC,console_account_owner,console_app,console_rt,console_auth_rt,
         console_auth_startup,console_terms_owner,console_credential_owner,
         console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
GRANT EXECUTE ON FUNCTION public.account_legacy_actor_presence_v1(uuid) TO console_app;

CREATE FUNCTION public.platform_legacy_removal_membership_claim_v1(
    p_company uuid,p_expected_group uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $membership_claim$
DECLARE
    claimed_company uuid;
    current_group uuid;
BEGIN
    -- Membership assignment can hold this tuple before requesting the org lock.
    -- Claim exactly the required tuple without waiting into that inversion.
    SELECT m.org_id INTO claimed_company FROM public.group_memberships m
        WHERE m.org_id=p_company AND m.group_id=p_expected_group
        FOR UPDATE SKIP LOCKED;
    IF NOT FOUND THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_purge.topology_conflict';
    END IF;
    SELECT m.group_id INTO current_group FROM public.group_memberships m WHERE m.org_id=p_company;
    IF NOT FOUND OR current_group IS DISTINCT FROM p_expected_group THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_purge.topology_conflict';
    END IF;
END;
$membership_claim$;
ALTER FUNCTION public.platform_legacy_removal_membership_claim_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_removal_membership_claim_v1(uuid,uuid)
    FROM PUBLIC,console_account_owner,console_app,console_rt,console_auth_rt,
         console_auth_startup,console_terms_owner,console_credential_owner,
         console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
GRANT EXECUTE ON FUNCTION public.platform_legacy_removal_membership_claim_v1(uuid,uuid) TO console_app;

CREATE FUNCTION public.platform_legacy_removal_admit_v1(p_id uuid) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $removal_admit$
DECLARE
    sentinel_org CONSTANT uuid := '00000000-0000-0000-0000-00000000face'::uuid;
    discovered_group uuid;
    current_group uuid;
    claimed_company uuid;
    group_locked boolean := false;
BEGIN
    IF p_id IS NULL THEN
        RAISE EXCEPTION USING ERRCODE='22004',MESSAGE='auth_legacy.null_identity';
    END IF;
    IF p_id=sentinel_org THEN RETURN false; END IF;
    IF pg_catalog.current_setting('transaction_isolation')<>'read committed' THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy.unsupported_isolation';
    END IF;

    SELECT o.group_id INTO discovered_group FROM public.organizations o WHERE o.id=p_id;
    IF NOT FOUND THEN RETURN false; END IF;
    IF discovered_group IS NOT NULL THEN
        PERFORM g.id FROM public.groups g WHERE g.id=discovered_group FOR UPDATE;
        group_locked := FOUND;
    END IF;
    PERFORM o.id FROM public.organizations o WHERE o.id=p_id FOR UPDATE;
    IF NOT FOUND THEN RETURN false; END IF;

    -- A separate post-wait statement observes the committed organization version.
    -- Never acquire a newly discovered Group while holding this organization.
    SELECT o.group_id INTO current_group FROM public.organizations o WHERE o.id=p_id;
    IF NOT FOUND THEN RETURN false; END IF;
    IF NOT group_locked OR discovered_group IS NULL
       OR current_group IS DISTINCT FROM discovered_group THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_purge.topology_conflict';
    END IF;

    PERFORM public.platform_legacy_removal_membership_claim_v1(p_id,discovered_group);

    -- Fresh scoped owner read after all admission waits, before user/Account locks.
    IF public.account_legacy_actor_presence_v1(p_id) THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy_purge.native_actors_present';
    END IF;
    -- Existing callers decide status from fresh reads under the retained org lock.
    -- Reentrant calls reacquire only this same held Group/org/membership lockset.
    RETURN true;
END;
$removal_admit$;
ALTER FUNCTION public.platform_legacy_removal_admit_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_removal_admit_v1(uuid)
    FROM PUBLIC,console_account_owner,console_app,console_rt,console_auth_rt,
         console_auth_startup,console_terms_owner,console_credential_owner,
         console_leave_cmd,console_ontology_cmd,console_platform_force_cmd;
-- console_rt is only the retained ordinary remover's pre-receipt caller.
-- The force-command login reaches this port through its existing definer.
GRANT EXECUTE ON FUNCTION public.platform_legacy_removal_admit_v1(uuid) TO console_app,console_rt;

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
    IF NOT public.platform_legacy_removal_admit_v1(p_company) THEN
        RAISE EXCEPTION USING ERRCODE='P0002', MESSAGE='auth_legacy.company_not_found';
    END IF;
    SELECT COALESCE(array_agg(locked.id ORDER BY locked.id),ARRAY[]::uuid[]) INTO subjects
        FROM (SELECT u.id FROM public.users u WHERE u.org_id=p_company ORDER BY u.id FOR UPDATE) locked;
    RETURN subjects;
END;$auth7_body$;

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

    org_exists := public.platform_legacy_removal_admit_v1(p_id);
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

    IF NOT public.platform_legacy_removal_admit_v1(p_id) THEN
        SET LOCAL row_security = on;
        RETURN 'not_found';
    END IF;

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
    IF NOT public.platform_legacy_removal_admit_v1(p_id) THEN
        RAISE EXCEPTION USING ERRCODE='P0002',MESSAGE='auth_legacy.company_not_found';
    END IF;
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
              'audit_events', 'employees', 'users', 'branches', 'regions', 'company_actors',
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

CREATE OR REPLACE FUNCTION public.platform_force_remove_organization_command(
    p_id UUID, p_actor UUID, p_trace_id CHAR(32), p_span_id CHAR(16), p_occurred_at TIMESTAMPTZ
) RETURNS TEXT
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    v_slug TEXT;
    v_name TEXT;
    v_outcome TEXT;
    v_wiped JSONB;
BEGIN
    SET LOCAL row_security = off;
    IF NOT public.platform_legacy_removal_admit_v1(p_id) THEN
        RETURN 'not_found';
    END IF;
    -- One admitted pre-delete observation; counts are not measured DELETE effects.
    SELECT o.slug, o.name, jsonb_build_object(
        'users', (SELECT count(*) FROM users WHERE org_id = p_id),
        'registry_customers', (SELECT count(*) FROM registry_customers WHERE org_id = p_id),
        'registry_sites', (SELECT count(*) FROM registry_sites WHERE org_id = p_id),
        'registry_equipment', (SELECT count(*) FROM registry_equipment WHERE org_id = p_id),
        'work_orders', (SELECT count(*) FROM work_orders WHERE org_id = p_id),
        'financial_rental_quotes', (SELECT count(*) FROM financial_rental_quotes WHERE org_id = p_id),
        'financial_purchase_requests', (SELECT count(*) FROM financial_purchase_requests WHERE org_id = p_id),
        'messenger_threads', (SELECT count(*) FROM messenger_threads WHERE org_id = p_id),
        'evidence_media', (SELECT count(*) FROM evidence_media WHERE org_id = p_id),
        'audit_events', (SELECT count(*) FROM audit_events WHERE org_id = p_id)
    ) INTO v_slug, v_name, v_wiped
    FROM public.organizations o WHERE o.id=p_id;
    v_outcome := platform_force_remove_organization(p_id);
    IF v_outcome <> 'removed' THEN
        RETURN v_outcome;
    END IF;
    INSERT INTO audit_events (
        org_id, actor, action, target_type, target_id, branch_id,
        before_snap, after_snap, trace_id, span_id, occurred_at
    ) VALUES (
        NULL, p_actor, 'platform.tenant.force_remove', 'organizations', p_id::text, NULL,
        jsonb_build_object('org_id', p_id, 'slug', v_slug, 'name', v_name, 'wiped', v_wiped), NULL,
        p_trace_id, p_span_id, p_occurred_at
    );
    RETURN 'removed';
END;
$$;
