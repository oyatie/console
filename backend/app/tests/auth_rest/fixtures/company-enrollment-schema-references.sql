-- Independent expected FK roster, from actual reviewed declarations; no invented revision target.
DO $references$
DECLARE actual jsonb; expected jsonb := '[["company_enrollment_requests",["account_id"],"accounts",["id"],false],["company_enrollment_requests",["designation_receipt_id"],"deployment_operator_receipts",["receipt_id"],false],["company_enrollment_requests",["account_id","command_id","committed_receipt_id","codec_version","input_digest","designation_receipt_id"],"company_enrollment_receipts",["account_id","command_id","receipt_id","codec_version","input_digest","designation_receipt_id"],true],["company_enrollment_receipts",["account_id"],"accounts",["id"],false],["company_enrollment_receipts",["designation_receipt_id"],"deployment_operator_receipts",["receipt_id"],false],["company_enrollment_receipts",["org_id"],"organizations",["id"],false],["company_enrollment_receipts",["group_id"],"groups",["id"],false],["company_enrollment_receipts",["administrative_account_id"],"accounts",["id"],false],["company_enrollment_receipts",["root_assignment_id"],"user_role_assignments",["id"],false],["company_enrollment_receipts",["account_id","command_id","codec_version","input_digest","designation_receipt_id"],"company_enrollment_requests",["account_id","command_id","codec_version","input_digest","designation_receipt_id"],true],["company_enrollment_request_events",["actor_account_id"],"accounts",["id"],false],["company_enrollment_request_events",["account_id","command_id"],"company_enrollment_requests",["account_id","command_id"],false]]'::jsonb;
BEGIN
  SELECT jsonb_agg(jsonb_build_array(source.relname,
    (SELECT jsonb_agg(a.attname ORDER BY k.ord) FROM unnest(c.conkey) WITH ORDINALITY k(num,ord)
      JOIN pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.num),
    target.relname,
    (SELECT jsonb_agg(a.attname ORDER BY k.ord) FROM unnest(c.confkey) WITH ORDINALITY k(num,ord)
      JOIN pg_attribute a ON a.attrelid=c.confrelid AND a.attnum=k.num),
    c.condeferrable)) INTO actual
  FROM pg_constraint c JOIN pg_class source ON source.oid=c.conrelid
  JOIN pg_class target ON target.oid=c.confrelid
  WHERE c.contype='f' AND source.relnamespace='public'::regnamespace
    AND source.relname IN ('company_enrollment_requests','company_enrollment_receipts','company_enrollment_request_events');
  IF actual IS NULL OR jsonb_array_length(actual)<>12
     OR EXISTS((SELECT value FROM jsonb_array_elements(actual) EXCEPT SELECT value FROM jsonb_array_elements(expected))
          UNION ALL (SELECT value FROM jsonb_array_elements(expected) EXCEPT SELECT value FROM jsonb_array_elements(actual)))
  THEN RAISE EXCEPTION 'company.schema_fk_roster'; END IF;
  IF EXISTS(SELECT 1 FROM pg_constraint c JOIN pg_class source ON source.oid=c.conrelid
    JOIN pg_class target ON target.oid=c.confrelid
    WHERE c.contype='f' AND source.relnamespace='public'::regnamespace
      AND source.relname IN ('company_enrollment_requests','company_enrollment_receipts','company_enrollment_request_events')
      AND (NOT c.convalidated OR c.confmatchtype<>'s' OR c.confupdtype<>'r' OR c.confdeltype<>'r'
        OR c.condeferred<>c.condeferrable OR target.relnamespace<>'public'::regnamespace))
  THEN RAISE EXCEPTION 'company.schema_fk_mode'; END IF;
  IF EXISTS(SELECT 1 FROM pg_proc WHERE pronamespace='public'::regnamespace AND proname IN
    ('company_enrollment_prepare_v1','company_enrollment_status_v1','company_enrollment_cancel_v1',
     'company_enrollment_execute_v1','company_enrollment_session_material_v1'))
  THEN RAISE EXCEPTION 'company.schema_unreviewed_serving_participant'; END IF;
  RAISE NOTICE 'company.schema_references fk_count=12 serving_participants=0';
END
$references$;
