-- UNINSTALLED provenance candidate. Install only through a separately reviewed
-- declared-artifact fixture and successor custody; this source grants no authority.
-- console_app is the existing trusted topology owner with BYPASSRLS. Its private
-- helper returns only a boolean and grants the Account owner no raw topology DML.
CREATE FUNCTION public.account_company_provenance_lock_v1(p_company uuid,p_group uuid)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); answer boolean:=false;
 head record; company_row public.organizations%ROWTYPE;
 membership public.group_memberships%ROWTYPE; history public.group_membership_revisions%ROWTYPE;
 receipt public.platform_legacy_topology_receipts%ROWTYPE; decoded record;
BEGIN
 <<classification>>
 BEGIN
  IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
   OR p_company IS NULL OR p_group IS NULL
   OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_group) THEN
   EXIT classification;
  END IF;
  PERFORM set_config('app.current_org',p_company::text,true);
  SELECT * INTO STRICT head FROM public.group_authority_lock_shared_v1(p_group);
  IF head.revision IS NULL OR head.revision<1 OR head.incarnation IS NULL
   OR head.incarnation='00000000-0000-0000-0000-000000000000'::uuid
   OR head.state NOT IN('ACTIVE','RETIRED') THEN EXIT classification; END IF;
  -- Match the existing topology owner: head, Group, Company, membership/history.
  PERFORM g.id FROM public.groups g WHERE g.id=p_group FOR SHARE OF g;
  IF NOT FOUND THEN EXIT classification; END IF;
  SELECT o.* INTO STRICT company_row FROM public.organizations o WHERE o.id=p_company FOR SHARE OF o;
  IF company_row.group_id IS DISTINCT FROM p_group THEN EXIT classification; END IF;
  SELECT m.* INTO STRICT membership FROM public.group_memberships m
   WHERE m.org_id=p_company AND m.group_id=p_group FOR SHARE OF m;
  SELECT h.* INTO STRICT history FROM public.group_membership_revisions h
   WHERE (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)
    =(membership.group_id,membership.org_id,membership.membership_id,membership.current_revision,membership.incarnation)
   FOR SHARE OF h;
  IF history.state IS DISTINCT FROM 'ACTIVE' OR history.to_time IS NOT NULL
   OR history.from_time IS DISTINCT FROM membership.created_at
   OR EXISTS(SELECT 1 FROM public.organizations o WHERE o.group_id=p_group
    AND NOT EXISTS(SELECT 1 FROM public.group_memberships m WHERE m.org_id=o.id AND m.group_id=p_group))
   OR EXISTS(SELECT 1 FROM public.group_memberships m LEFT JOIN public.organizations o ON o.id=m.org_id
    WHERE m.group_id=p_group AND (o.id IS NULL OR o.group_id IS DISTINCT FROM p_group)) THEN
   EXIT classification;
  END IF;
  CASE history.provenance_kind
  WHEN 'LEGACY_BACKFILL' THEN
   answer:=history.revision=1 AND history.native_account_id IS NULL
    AND history.legacy_actor_user_id IS NULL AND history.force_actor_user_id IS NULL
    AND history.command_id IS NULL AND history.command_receipt IS NULL;
  WHEN 'COMPANY_ENROLLMENT_V1' THEN
   answer:=history.revision=1 AND history.native_account_id IS NOT NULL
    AND history.command_id IS NOT NULL AND history.command_receipt IS NOT NULL
    AND history.legacy_actor_user_id IS NULL AND history.force_actor_user_id IS NULL;
  WHEN 'LEGACY_TOPOLOGY_V1' THEN
   IF history.native_account_id IS NOT NULL OR history.force_actor_user_id IS NOT NULL THEN
    EXIT classification;
   END IF;
   SELECT r.* INTO STRICT receipt FROM public.platform_legacy_topology_receipts r
    WHERE (r.actor_user_id,r.command_id,r.receipt_id)
     =(history.legacy_actor_user_id,history.command_id,history.command_receipt);
   SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(receipt.input_bytes);
   answer:=receipt.outcome='APPLIED' AND receipt.result_code='applied' AND receipt.kind IN(1,4,5)
    AND receipt.codec_version=1 AND receipt.input_digest=sha256(receipt.input_bytes)
    AND (decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest)
     =(receipt.actor_user_id,receipt.command_id,receipt.kind,receipt.input_digest)
    AND receipt.result_org_id=p_company AND receipt.result_group_id=p_group
    AND receipt.occurred_at=history.from_time AND history.revision=1
    AND EXISTS(SELECT 1 FROM jsonb_array_elements(receipt.heads_after) entry
     WHERE (entry->>'group_id')::uuid=p_group
      AND (entry->>'incarnation')::uuid=head.incarnation
      AND (entry->>'revision')::bigint BETWEEN 1 AND head.revision);
  ELSE
   answer:=false;
  END CASE;
 END classification;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN coalesce(answer,false);
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN false;
END
$body$;

CREATE FUNCTION public.account_company_provenance_v1(p_company uuid)
RETURNS text LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); answer text:='UNKNOWN';
 planned_group uuid; company_row record; group_row record; member_row record; member uuid;
 origin record; receipt public.company_enrollment_receipts%ROWTYPE;
 request public.company_enrollment_requests%ROWTYPE; effect public.company_enrollment_effect_bindings%ROWTYPE;
 source_company record; decoded record; actions jsonb; properties jsonb;
BEGIN
 <<classification>>
 BEGIN
  IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
   OR p_company IS NULL OR p_company='00000000-0000-0000-0000-000000000000'::uuid THEN
   EXIT classification;
  END IF;
  PERFORM set_config('app.current_org',p_company::text,true);
  SELECT o.group_id INTO STRICT planned_group FROM public.organizations o WHERE o.id=p_company;
  IF public.account_company_provenance_lock_v1(p_company,planned_group) IS NOT TRUE THEN
   EXIT classification;
  END IF;
  SELECT o.id,o.group_id,o.origin_account_id,o.origin_command_id,o.origin_receipt_id
   INTO STRICT company_row FROM public.organizations o WHERE o.id=p_company;
  SELECT g.id,g.origin_account_id,g.origin_command_id,g.origin_receipt_id
   INTO STRICT group_row FROM public.groups g WHERE g.id=planned_group;
  SELECT h.* INTO STRICT member_row FROM public.group_memberships m
   JOIN public.group_membership_revisions h ON (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)
    =(m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation)
   WHERE m.org_id=p_company AND m.group_id=planned_group;
  IF company_row.group_id IS DISTINCT FROM planned_group THEN EXIT classification; END IF;

  IF company_row.origin_account_id IS NULL AND company_row.origin_command_id IS NULL
   AND company_row.origin_receipt_id IS NULL AND group_row.origin_account_id IS NULL
   AND group_row.origin_command_id IS NULL AND group_row.origin_receipt_id IS NULL THEN
   IF member_row.provenance_kind NOT IN('LEGACY_BACKFILL','LEGACY_TOPOLOGY_V1')
    OR EXISTS(SELECT 1 FROM public.company_enrollment_receipts r WHERE r.group_id=planned_group)
    OR EXISTS(SELECT 1 FROM public.company_enrollment_effect_bindings b WHERE b.group_id=planned_group) THEN
    EXIT classification;
   END IF;
   -- Owner-only membership IDs are captured under the retained Group head lock.
   -- Each RLS Company is armed separately; a single tenant-scoped join hides siblings.
   FOR member IN SELECT m.org_id FROM public.group_memberships m
    WHERE m.group_id=planned_group ORDER BY m.org_id LOOP
    IF public.account_company_provenance_lock_v1(member,planned_group) IS NOT TRUE THEN
     EXIT classification;
    END IF;
    PERFORM set_config('app.current_org',member::text,true);
    IF NOT EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=member AND o.group_id=planned_group
      AND o.origin_account_id IS NULL AND o.origin_command_id IS NULL AND o.origin_receipt_id IS NULL)
     OR NOT EXISTS(SELECT 1 FROM public.group_memberships m JOIN public.group_membership_revisions h
      ON (h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation)
       =(m.group_id,m.org_id,m.membership_id,m.current_revision,m.incarnation)
      WHERE m.org_id=member AND m.group_id=planned_group
       AND h.provenance_kind IN('LEGACY_BACKFILL','LEGACY_TOPOLOGY_V1'))
     OR EXISTS(SELECT 1 FROM public.company_enrollment_receipts r WHERE r.org_id=member)
     OR EXISTS(SELECT 1 FROM public.company_enrollment_effect_bindings b WHERE b.org_id=member)
     OR EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=member)
     OR EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=member)
     OR EXISTS(SELECT 1 FROM public.native_company_catalog_installs c WHERE c.org_id=member) THEN
     EXIT classification;
    END IF;
   END LOOP;
   answer:='LEGACY'; EXIT classification;
  END IF;

  IF company_row.origin_account_id IS NULL OR company_row.origin_command_id IS NULL
   OR company_row.origin_receipt_id IS NULL OR group_row.origin_account_id IS NULL
   OR group_row.origin_command_id IS NULL OR group_row.origin_receipt_id IS NULL
   OR member_row.provenance_kind IS DISTINCT FROM 'COMPANY_ENROLLMENT_V1'
   OR (member_row.native_account_id,member_row.command_id,member_row.command_receipt)
    IS DISTINCT FROM (company_row.origin_account_id,company_row.origin_command_id,company_row.origin_receipt_id) THEN
   EXIT classification;
  END IF;
  -- Independently prove each origin. A Group's founding Company need not be the
  -- selected Company; no equality between their origin triples is assumed.
  FOR origin IN SELECT * FROM (VALUES
   (company_row.origin_account_id,company_row.origin_command_id,company_row.origin_receipt_id,true),
   (group_row.origin_account_id,group_row.origin_command_id,group_row.origin_receipt_id,false))
   AS sources(account_id,command_id,receipt_id,is_company) LOOP
   SELECT r.* INTO STRICT receipt FROM public.company_enrollment_receipts r
    WHERE (r.account_id,r.command_id,r.receipt_id)=(origin.account_id,origin.command_id,origin.receipt_id);
   SELECT q.* INTO STRICT request FROM public.company_enrollment_requests q
    WHERE q.account_id=receipt.account_id AND q.command_id=receipt.command_id;
   SELECT b.* INTO STRICT effect FROM public.company_enrollment_effect_bindings b
    WHERE (b.account_id,b.command_id,b.receipt_id)=(receipt.account_id,receipt.command_id,receipt.receipt_id);
   IF (origin.is_company AND receipt.org_id IS DISTINCT FROM p_company)
    OR receipt.group_id IS DISTINCT FROM planned_group
    OR request.state IS DISTINCT FROM 'COMMITTED'
    OR request.committed_receipt_id IS DISTINCT FROM receipt.receipt_id
    OR request.terminal_at IS DISTINCT FROM receipt.committed_at
    OR (request.codec_version,request.input_digest,request.designation_receipt_id)
     IS DISTINCT FROM (receipt.codec_version,receipt.input_digest,receipt.designation_receipt_id)
    OR (effect.org_id,effect.group_id,effect.administrative_account_id,effect.codec_version,
      effect.designation_receipt_id,effect.input_digest,effect.started_at,effect.catalog_version,
      effect.manifest_digest,effect.session_id)
     IS DISTINCT FROM (receipt.org_id,receipt.group_id,receipt.administrative_account_id,receipt.codec_version,
      receipt.designation_receipt_id,receipt.input_digest,receipt.committed_at,receipt.catalog_version,
      receipt.manifest_digest,receipt.session_id)
    OR receipt.codec_version<>1 OR receipt.root_revision<>1
   OR receipt.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
    OR receipt.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex') THEN
    EXIT classification;
   END IF;
   IF origin.is_company AND member_row.from_time IS DISTINCT FROM receipt.committed_at THEN
    EXIT classification;
   END IF;
   IF request.input_bytes IS NOT NULL THEN
    SELECT * INTO STRICT decoded FROM public.company_enrollment_decode_input_v1(request.input_bytes);
    IF (decoded.account_id,decoded.command_id,decoded.input_digest,decoded.administrative_account_id)
     IS DISTINCT FROM (receipt.account_id,receipt.command_id,receipt.input_digest,receipt.administrative_account_id)
     OR decoded.group_id IS NOT NULL THEN EXIT classification; END IF;
   END IF;
   IF public.account_company_provenance_lock_v1(receipt.org_id,planned_group) IS NOT TRUE THEN
    EXIT classification;
   END IF;
   PERFORM set_config('app.current_org',receipt.org_id::text,true);
   SELECT o.id,o.origin_account_id,o.origin_command_id,o.origin_receipt_id INTO STRICT source_company
    FROM public.organizations o WHERE o.id=receipt.org_id;
   IF (source_company.origin_account_id,source_company.origin_command_id,source_company.origin_receipt_id)
     IS DISTINCT FROM (receipt.account_id,receipt.command_id,receipt.receipt_id)
    OR NOT EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=receipt.org_id AND h.epoch>=1
     AND (h.origin_account_id,h.origin_command_id,h.origin_receipt_id)
      =(receipt.account_id,receipt.command_id,receipt.receipt_id))
    OR NOT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=receipt.org_id
     AND a.account_id=receipt.administrative_account_id AND a.admission_receipt_id=receipt.receipt_id
     AND a.created_at=receipt.committed_at
     AND a.entitlement_ref=jsonb_build_object('kind','COMPANY_ENROLLMENT_V1',
      'account_id',receipt.account_id::text,'command_id',receipt.command_id::text,
      'org_id',receipt.org_id::text,'receipt_id',receipt.receipt_id::text))
    OR NOT EXISTS(SELECT 1 FROM public.native_company_catalog_installs c WHERE c.org_id=receipt.org_id
     AND c.catalog_version=receipt.catalog_version AND c.manifest_digest=receipt.manifest_digest
     AND (c.origin_account_id,c.origin_command_id,c.origin_receipt_id)
      =(receipt.account_id,receipt.command_id,receipt.receipt_id) AND c.installed_at=receipt.committed_at)
    OR NOT EXISTS(SELECT 1 FROM public.policy_assignment_revisions a WHERE a.org_id=receipt.org_id
     AND a.assignment_id=receipt.root_assignment_id AND a.revision=receipt.root_revision
     AND a.subject_protocol='NATIVE_ACCOUNT' AND a.account_id=receipt.administrative_account_id
     AND (a.origin_account_id,a.origin_command_id,a.origin_receipt_id)
      =(receipt.account_id,receipt.command_id,receipt.receipt_id)
     AND a.actor_account_id=receipt.account_id AND a.session_id=receipt.session_id
     AND a.created_at=receipt.committed_at AND a.valid_from=receipt.committed_at)
    OR (SELECT count(*) FROM public.company_enrollment_request_events e
     WHERE e.account_id=receipt.account_id AND e.command_id=receipt.command_id)<>2
    OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE e.account_id=receipt.account_id AND e.command_id=receipt.command_id AND e.event_revision=2
      AND e.from_state='PENDING' AND e.to_state='COMMITTED' AND e.reason_code='COMMITTED'
      AND e.occurred_at=receipt.committed_at AND e.actor_account_id=receipt.account_id
      AND e.session_id=receipt.session_id) THEN EXIT classification; END IF;
   SELECT jsonb_agg(jsonb_build_object('org_id',a.org_id::text,'object_type_id',a.object_type_id::text,
    'action_type_id',a.action_type_id::text,'registration_revision',a.registration_revision::text,
    'manifest_digest',encode(a.manifest_digest,'hex'))
    ORDER BY a.org_id,a.object_type_id,a.action_type_id,a.registration_revision,a.manifest_digest)
    INTO actions FROM public.native_company_action_refs a
    WHERE a.org_id=receipt.org_id AND a.catalog_version=receipt.catalog_version;
   SELECT jsonb_agg(jsonb_build_object('org_id',p.org_id::text,'object_type_id',p.object_type_id::text,
    'property_id',p.property_id::text,'schema_revision',p.schema_revision::text)
    ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision)
    INTO properties FROM public.native_company_property_refs p
    WHERE p.org_id=receipt.org_id AND p.catalog_version=receipt.catalog_version;
   IF actions IS DISTINCT FROM receipt.action_refs OR properties IS DISTINCT FROM receipt.property_refs THEN
    EXIT classification;
   END IF;
  END LOOP;
  answer:='NATIVE';
 END classification;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN answer;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN 'UNKNOWN';
END
$body$;
