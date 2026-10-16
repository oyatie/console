-- New source definitions generated once for this owner revision. These UUIDs
-- are the four action definitions, never seeded business or authority rows.
-- Installing this file alone grants no serving authority: complete independent
-- custody verifies the exact source/ACL/ABI before any material may use it.
CREATE FUNCTION public.native_group_process_registered_actions_v1() RETURNS jsonb
LANGUAGE sql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$ SELECT $actions$[{"key":"identity.verifier.process.adopt/1","action_id":"47c0f0fe-92d6-473a-b0b5-1c0d218f919f","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","action_id":"12865e98-68d3-4086-a516-2cc9e4f75cf0","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","action_id":"fb8eaa7c-f7b5-4a0f-8fba-d6bed0f98e31","revision":1,"fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","action_id":"13dec8e4-5451-4d09-b7e1-a1e043e67d53","revision":1,"fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]$actions$::jsonb $body$;

CREATE FUNCTION public.native_group_process_source_v1(p_group uuid,p_incarnation uuid) RETURNS jsonb
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE schema_digest bytea:=decode('c711017368596094ad0ff9b1123eb17573722df47f1b4abb15a31ce1ce8b1e44','hex');
 policy_digest bytea:=decode('2453684b70134124a8cf77f2d882fb7497d8c1498325fac1321ca7e616791ed8','hex');
 codec_digest bytea:=decode('595376f9edea8ebd5a698bd27d6f7310a470f5e5d95522d7eed7c01db8169067','hex');
 actions jsonb:=public.native_group_process_registered_actions_v1(); registration bytea;
BEGIN
 registration:=public.native_group_process_registration_bytes_v1(p_group,p_incarnation,
  'native-group-process-v1',schema_digest,policy_digest,codec_digest,actions);
 RETURN jsonb_build_object('schema_id','native-group-process-v1','schema_digest',schema_digest,
  'policy_digest',policy_digest,'codec_contract_digest',codec_digest,
  'registration_manifest_version',1,'registration_manifest_digest',sha256(registration),
  'cedar_sdk_version','4.13.0','cedar_language_version','4.5','registered_actions',actions);
END
$body$;
