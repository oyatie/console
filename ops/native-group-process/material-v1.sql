-- Retained, purpose-specific material. Selectors and signed namespace hints
-- confer no authority; the adapter independently checks current Auth and the
-- actual transaction/time, hashes every encoded projection, and consumes finish.
CREATE FUNCTION public.native_group_process_account_material_v1(p_account uuid,p_family uuid) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE control record; family record; enrollment uuid; terms uuid; now_value timestamptz;
BEGIN
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 now_value:=clock_timestamp();
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation
  OR family.assurance IS DISTINCT FROM 'PASSKEY_PRIMARY' OR family.auth_time IS NULL
  OR NOT isfinite(family.auth_time) OR NOT isfinite(family.created_at)
  OR family.auth_time>family.created_at OR family.created_at>now_value THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 PERFORM 1 FROM public.account_login_consent_v1(p_account);
 SELECT e.id INTO STRICT enrollment FROM public.account_security_events e
  WHERE e.account_id=p_account AND e.kind='ENROLLED';
 -- Retain the terms owner's validated head through commit; the Account owner
 -- must neither read the protected table directly nor race a terms release.
 SELECT h.release_receipt_id INTO STRICT terms FROM public.account_terms_registration_head_v1() h;
 RETURN jsonb_build_object('actor_account_id',p_account,'session_id',p_family,
  'account_security_generation',control.security_generation,'account_state',control.security_state,
  'registration_receipt',enrollment,'current_terms_receipt',terms,
  'observed_at_us',public.native_group_process_micros_v1(clock_timestamp()),
  'source_xid',pg_current_xact_id()::text,'source_backend_pid',pg_backend_pid());
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.account_material_unavailable';
END
$body$;

CREATE FUNCTION public.native_group_process_original_material_v1(p_actor uuid,p_group uuid,p_incarnation uuid,p_command uuid) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE accepted public.native_group_process_inputs_v1; terminal public.native_group_process_results_v1;
BEGIN
 SELECT i.* INTO accepted FROM public.native_group_process_inputs_v1 i
  WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
 IF NOT FOUND THEN RETURN NULL; END IF;
 IF (accepted.group_id,accepted.group_incarnation) IS DISTINCT FROM (p_group,p_incarnation) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
 END IF;
 PERFORM public.native_group_process_assert_input_v1(p_actor,p_command);
 SELECT r.* INTO terminal FROM public.native_group_process_results_v1 r
  WHERE r.actor_account_id=p_actor AND r.command_id=p_command;
 IF FOUND THEN
  PERFORM public.native_group_process_assert_result_v1(p_actor,p_command);
  RETURN jsonb_build_object('accepted',to_jsonb(accepted),'terminal',to_jsonb(terminal));
 END IF;
 IF EXISTS(SELECT 1 FROM public.native_group_process_effects_v1 f
  WHERE f.actor_account_id=p_actor AND f.command_id=p_command) THEN
  RAISE EXCEPTION 'native_group_process.closure_unavailable';
 END IF;
 RETURN jsonb_build_object('accepted',to_jsonb(accepted),'terminal',NULL);
END
$body$;

CREATE FUNCTION public.identity_native_group_process_material_v1(
 p_account uuid,p_family uuid,p_group uuid,p_incarnation uuid,p_command uuid,p_mode smallint,p_input bytea)
RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE account_material jsonb; original jsonb; decoded jsonb; source jsonb; own_receipt boolean; current_context jsonb;
 optional_topology record; policy public.native_group_identity_policy_heads_v1;
 head public.native_group_process_heads_v1; content public.native_group_process_versions_v1;
 history jsonb; final_terminal public.native_group_process_results_v1;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR num_nonnulls(p_account,p_family,p_group,p_incarnation,p_mode)<>5
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_account,p_family,p_group,p_incarnation)
  OR p_mode NOT BETWEEN 1 AND 8
  OR ((p_mode IN (1,2,7))<>(p_command IS NULL))
  OR ((p_mode=3)<>(p_input IS NOT NULL))
  OR p_command='00000000-0000-0000-0000-000000000000'::uuid THEN
  RAISE EXCEPTION 'native_group_process.material_unavailable';
 END IF;
 IF p_mode=3 THEN
  decoded:=public.native_group_process_decode_v1(p_input);
  IF (decoded->>'actor_account_id',decoded->>'command_id',decoded->>'group_id',decoded->>'group_incarnation')
   IS DISTINCT FROM (p_account::text,p_command::text,p_group::text,p_incarnation::text) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
  END IF;
 END IF;
 -- The historical key has no FK to a deletable live head. Fixed Group then
 -- Account/family then command ordering also applies to the retry branch.
 PERFORM public.native_group_process_group_guard_v1(p_group,p_mode IN (3,4,6));
 -- Take an existing live-head guard before Account, matching existing Company
 -- writers. Absence is permitted for every historical receipt purpose.
 SELECT h.* INTO optional_topology FROM public.group_authority_lock_shared_v1(p_group) h;
 account_material:=public.native_group_process_account_material_v1(p_account,p_family);
 IF p_command IS NOT NULL THEN
  PERFORM public.native_group_process_command_guard_v1(p_account,p_command,p_mode IN (3,4,6));
  original:=public.native_group_process_original_material_v1(p_account,p_group,p_incarnation,p_command);
 END IF;
 source:=public.native_group_process_source_v1(p_group,p_incarnation);
 own_receipt:=p_mode IN (5,8);
 IF p_mode=6 AND original->'terminal' IS DISTINCT FROM 'null'::jsonb
  AND original IS NOT NULL THEN
  SELECT r.* INTO STRICT final_terminal FROM public.native_group_process_results_v1 r
   WHERE r.actor_account_id=p_account AND r.command_id=p_command;
  -- A terminal already committed elsewhere is read-own replay. The same-B
  -- postimage remains current material for its checked consuming finalizer.
  own_receipt:=(final_terminal.effect_xid,final_terminal.effect_backend_pid)
   IS DISTINCT FROM (pg_current_xact_id(),pg_backend_pid());
 END IF;
 IF own_receipt THEN
  IF original IS NULL THEN RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found'; END IF;
  RETURN jsonb_build_object('variant','OwnReceipt','mode',p_mode,'account',account_material,
   'group_id',p_group,'group_incarnation',p_incarnation,'original',original,'source',source);
 END IF;
 IF p_mode IN (4,6) AND original IS NULL THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
 END IF;
 current_context:=public.native_group_process_current_context_v1(p_account,p_family,p_group,p_incarnation,p_mode IN (3,4,6));
 PERFORM public.native_group_process_assert_current_v1(p_group,p_incarnation);
 SELECT h.* INTO policy FROM public.native_group_identity_policy_heads_v1 h
  WHERE h.group_id=p_group AND h.group_incarnation=p_incarnation;
 SELECT h.* INTO head FROM public.native_group_process_heads_v1 h
  WHERE h.group_id=p_group AND h.group_incarnation=p_incarnation;
 IF head.process_id IS NOT NULL THEN
  SELECT v.* INTO STRICT content FROM public.native_group_process_versions_v1 v
   WHERE (v.group_id,v.group_incarnation,v.process_id,v.version)=
    (head.group_id,head.group_incarnation,head.process_id,head.content_version);
 END IF;
 SELECT coalesce(jsonb_agg(jsonb_build_object('head',to_jsonb(h),'reason',
   CASE i.operation WHEN 6 THEN public.native_group_process_decode_v1(i.input_bytes)->>'reason' END)
   ORDER BY h.head_revision),'[]'::jsonb) INTO history
  FROM public.native_group_process_head_revisions_v1 h
  JOIN public.native_group_process_inputs_v1 i ON i.actor_account_id=h.last_actor_account_id AND i.command_id=h.last_command_id
  WHERE h.group_id=p_group AND h.group_incarnation=p_incarnation;
 RETURN current_context||jsonb_build_object('variant','CurrentMutation','mode',p_mode,
  'policy',CASE WHEN policy.group_id IS NOT NULL THEN to_jsonb(policy) END,
  'head',CASE WHEN head.process_id IS NOT NULL THEN to_jsonb(head) END,
  'version',CASE WHEN content.process_id IS NOT NULL THEN to_jsonb(content) END,
  'history',history,'original',original);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;
