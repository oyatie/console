-- Pure transition classification over original bytes and the actual locked
-- preimage; it grants no authority and produces no effect.
CREATE FUNCTION public.native_group_process_classify_v1(r public.native_group_process_effects_v1,decoded jsonb) RETURNS text
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE stale boolean; expected_code text;
BEGIN
 stale:=(decoded->>'expected_group_revision')::bigint<>r.observed_group_revision
  OR (decoded->>'expected_policy_revision')::bigint<>coalesce(r.policy_before_revision,0);
 IF r.operation=1 THEN
  stale:=stale OR (decoded->>'expected_prior_head_revision')::bigint<>coalesce(r.before_head_revision,0)
   OR (r.before_process_id IS NOT NULL AND r.before_process_id<>r.requested_process_id);
 ELSE
  stale:=stale OR r.before_process_id IS NULL OR r.before_process_id<>r.requested_process_id
   OR (decoded->>'content_version')::bigint IS DISTINCT FROM r.before_content_version
   OR (decoded->>'content_digest')::bytea IS DISTINCT FROM r.before_content_digest
   OR (decoded->>'expected_head_revision')::bigint IS DISTINCT FROM r.before_head_revision
   OR (decoded->>'expected_head_digest')::bytea IS DISTINCT FROM r.before_head_digest;
 END IF;
 IF stale THEN expected_code:='REJECTED_STALE_EXPECTATION';
 ELSIF r.operation=1 THEN
  IF (decoded->>'expiry_us')::bigint<=public.native_group_process_micros_v1(r.executed_at) THEN
   expected_code:='REJECTED_PROCESS_EXPIRED';
  ELSIF r.before_process_id IS NULL THEN expected_code:='ADOPTED';
  ELSE expected_code:='REPLACED'; END IF;
 ELSIF r.before_state='SUSPENDED' THEN expected_code:='REJECTED_ALREADY_SUSPENDED';
 ELSIF r.before_expires_at<=r.executed_at THEN expected_code:='REJECTED_PROCESS_EXPIRED';
 ELSE expected_code:='SUSPENDED'; END IF;
 RETURN expected_code;
END
$body$;
