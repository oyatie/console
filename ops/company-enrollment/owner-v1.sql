-- source: preamble
-- PRIVATE DIAGNOSTIC ASSEMBLY. No custody serving activation or rollout.
-- Requires numbered migrations through0228, current Account/credential custody
-- finalizers, and existing Company schema/input/guard/intake resources.
-- Native+legacy+force source composition is for isolated installation diagnostics;
-- current Rust callers, runtime ACL cutover and sealed rollout remain pending. Do not install in a
-- shared or serving database or use as the complete accepted resource.
SET LOCAL lock_timeout='1s';
SET LOCAL statement_timeout='30s';
SET LOCAL row_security=on;
LOCK TABLE public.groups,public.organizations,public.group_memberships IN ACCESS EXCLUSIVE MODE;
LOCK TABLE public.policy_roles,public.user_role_assignments,public.policy_role_permissions,public.policy_role_conditions IN ACCESS EXCLUSIVE MODE;


-- source: legacy-input.sql
CREATE FUNCTION public.platform_legacy_topology_decode_input_v1(p_input bytea)
RETURNS TABLE(actor_user_id uuid,command_id uuid,kind smallint,primary_target uuid,expected_groups jsonb,payload jsonb,input_digest bytea)
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE CALLED ON NULL INPUT
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE byte_count integer:=octet_length(p_input); cursor_pos integer:=28;
 head_count integer; index_value integer; group_value uuid; incarnation_value uuid; previous_group uuid;
 revision_value numeric; byte_index integer; field_length bigint; char_index integer; codepoint integer;
 slug_value text; name_value text; phone_value text; status_value text; tag integer; status_tag integer;
 company_value uuid; user_value uuid; role_count integer; role_tag integer; role_value text; roles text[]:=ARRAY[]::text[];
BEGIN
 IF p_input IS NULL OR byte_count NOT BETWEEN 62 AND 4096
  OR substring(p_input FROM 1 FOR 28)<>decode('636f6e736f6c652e706c6174666f726d2e746f706f6c6f6779000001','hex') THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 actor_user_id:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF actor_user_id='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 command_id:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF command_id='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 kind:=tag::smallint;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 head_count:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF kind NOT BETWEEN 1 AND 9 OR head_count NOT BETWEEN 0 AND 2 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 expected_groups:='[]'::jsonb;
 FOR index_value IN 1..head_count LOOP
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 group_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF group_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 incarnation_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF incarnation_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 IF byte_count-cursor_pos<8 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 revision_value:=0;
 FOR byte_index IN 0..7 LOOP revision_value:=revision_value*256+get_byte(p_input,cursor_pos+byte_index); END LOOP;
 cursor_pos:=cursor_pos+8;
 IF revision_value NOT BETWEEN 1 AND 9223372036854775807 OR group_value<=previous_group THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 previous_group:=group_value;
 expected_groups:=expected_groups||jsonb_build_array(jsonb_build_object('group_id',group_value::text,'incarnation',incarnation_value::text,'revision',revision_value::text));
 END LOOP;
 group_value:=NULL;
 CASE kind
 WHEN 1,2 THEN
IF byte_count-cursor_pos<4 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 field_length:=get_byte(p_input,cursor_pos)*16777216::bigint+get_byte(p_input,cursor_pos+1)*65536::bigint
  +get_byte(p_input,cursor_pos+2)*256::bigint+get_byte(p_input,cursor_pos+3); cursor_pos:=cursor_pos+4;
 IF field_length NOT BETWEEN 1 AND 40 OR field_length>byte_count-cursor_pos THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 slug_value:=convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');cursor_pos:=cursor_pos+field_length::integer;
 FOR char_index IN 1..char_length(slug_value) LOOP
  codepoint:=ascii(substring(slug_value FROM char_index FOR 1));
  IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159
   OR ((char_index=1 OR char_index=char_length(slug_value)) AND
    (codepoint IN (32,160,5760,8232,8233,8239,8287,12288) OR codepoint BETWEEN 8192 AND 8202)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
  END IF;
 END LOOP;
 IF slug_value COLLATE "C" !~ '^[a-z0-9][a-z0-9-]{1,38}[a-z0-9]$' THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<4 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 field_length:=get_byte(p_input,cursor_pos)*16777216::bigint+get_byte(p_input,cursor_pos+1)*65536::bigint
  +get_byte(p_input,cursor_pos+2)*256::bigint+get_byte(p_input,cursor_pos+3); cursor_pos:=cursor_pos+4;
 IF field_length NOT BETWEEN 1 AND 256 OR field_length>byte_count-cursor_pos THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 name_value:=convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');cursor_pos:=cursor_pos+field_length::integer;
 FOR char_index IN 1..char_length(name_value) LOOP
  codepoint:=ascii(substring(name_value FROM char_index FOR 1));
  IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159
   OR ((char_index=1 OR char_index=char_length(name_value)) AND
    (codepoint IN (32,160,5760,8232,8233,8239,8287,12288) OR codepoint BETWEEN 8192 AND 8202)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
  END IF;
 END LOOP;
 payload:=jsonb_build_object('slug',slug_value,'name',name_value);
 WHEN 3 THEN
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 group_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF group_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF tag=1 THEN
IF byte_count-cursor_pos<4 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 field_length:=get_byte(p_input,cursor_pos)*16777216::bigint+get_byte(p_input,cursor_pos+1)*65536::bigint
  +get_byte(p_input,cursor_pos+2)*256::bigint+get_byte(p_input,cursor_pos+3); cursor_pos:=cursor_pos+4;
 IF field_length NOT BETWEEN 1 AND 40 OR field_length>byte_count-cursor_pos THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 slug_value:=convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');cursor_pos:=cursor_pos+field_length::integer;
 FOR char_index IN 1..char_length(slug_value) LOOP
  codepoint:=ascii(substring(slug_value FROM char_index FOR 1));
  IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159
   OR ((char_index=1 OR char_index=char_length(slug_value)) AND
    (codepoint IN (32,160,5760,8232,8233,8239,8287,12288) OR codepoint BETWEEN 8192 AND 8202)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
  END IF;
 END LOOP;
 IF slug_value COLLATE "C" !~ '^[a-z0-9][a-z0-9-]{1,38}[a-z0-9]$' THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 ELSIF tag<>0 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF tag=1 THEN
IF byte_count-cursor_pos<4 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 field_length:=get_byte(p_input,cursor_pos)*16777216::bigint+get_byte(p_input,cursor_pos+1)*65536::bigint
  +get_byte(p_input,cursor_pos+2)*256::bigint+get_byte(p_input,cursor_pos+3); cursor_pos:=cursor_pos+4;
 IF field_length NOT BETWEEN 1 AND 256 OR field_length>byte_count-cursor_pos THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 name_value:=convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');cursor_pos:=cursor_pos+field_length::integer;
 FOR char_index IN 1..char_length(name_value) LOOP
  codepoint:=ascii(substring(name_value FROM char_index FOR 1));
  IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159
   OR ((char_index=1 OR char_index=char_length(name_value)) AND
    (codepoint IN (32,160,5760,8232,8233,8239,8287,12288) OR codepoint BETWEEN 8192 AND 8202)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
  END IF;
 END LOOP;
 ELSIF tag<>0 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF tag=1 THEN
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 status_tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF status_tag NOT BETWEEN 1 AND 3 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 status_value:=(ARRAY['ACTIVE','SUSPENDED','ARCHIVED'])[status_tag];
 ELSIF tag<>0 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 IF slug_value IS NULL AND name_value IS NULL AND status_value IS NULL THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 primary_target:=group_value;
 payload:=jsonb_build_object('group_id',group_value::text,'slug',slug_value,'name',name_value,'status',status_value);
 WHEN 4,5 THEN
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 group_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF group_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 company_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF company_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 primary_target:=company_value;
 payload:=jsonb_build_object('group_id',group_value::text,'org_id',company_value::text);
 WHEN 6 THEN
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 group_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF group_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 company_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF company_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<4 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 field_length:=get_byte(p_input,cursor_pos)*16777216::bigint+get_byte(p_input,cursor_pos+1)*65536::bigint
  +get_byte(p_input,cursor_pos+2)*256::bigint+get_byte(p_input,cursor_pos+3); cursor_pos:=cursor_pos+4;
 IF field_length NOT BETWEEN 1 AND 256 OR field_length>byte_count-cursor_pos THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 name_value:=convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');cursor_pos:=cursor_pos+field_length::integer;
 FOR char_index IN 1..char_length(name_value) LOOP
  codepoint:=ascii(substring(name_value FROM char_index FOR 1));
  IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159
   OR ((char_index=1 OR char_index=char_length(name_value)) AND
    (codepoint IN (32,160,5760,8232,8233,8239,8287,12288) OR codepoint BETWEEN 8192 AND 8202)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
  END IF;
 END LOOP;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF tag=1 THEN
IF byte_count-cursor_pos<4 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 field_length:=get_byte(p_input,cursor_pos)*16777216::bigint+get_byte(p_input,cursor_pos+1)*65536::bigint
  +get_byte(p_input,cursor_pos+2)*256::bigint+get_byte(p_input,cursor_pos+3); cursor_pos:=cursor_pos+4;
 IF field_length NOT BETWEEN 1 AND 64 OR field_length>byte_count-cursor_pos THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 phone_value:=convert_from(substring(p_input FROM cursor_pos+1 FOR field_length::integer),'UTF8');cursor_pos:=cursor_pos+field_length::integer;
 FOR char_index IN 1..char_length(phone_value) LOOP
  codepoint:=ascii(substring(phone_value FROM char_index FOR 1));
  IF codepoint BETWEEN 0 AND 31 OR codepoint BETWEEN 127 AND 159
   OR ((char_index=1 OR char_index=char_length(phone_value)) AND
    (codepoint IN (32,160,5760,8232,8233,8239,8287,12288) OR codepoint BETWEEN 8192 AND 8202)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
  END IF;
 END LOOP;
 ELSIF tag<>0 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 role_count:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF role_count NOT BETWEEN 1 AND 64 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 FOR index_value IN 1..role_count LOOP
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 role_tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF role_tag NOT BETWEEN 1 AND 6 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 roles:=array_append(roles,(ARRAY['SUPER_ADMIN','ADMIN','MECHANIC','RECEPTIONIST','EXECUTIVE','MEMBER'])[role_tag]);
 END LOOP;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 role_tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF role_tag NOT BETWEEN 1 AND 3 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 role_value:=(ARRAY['GROUP_ADMIN','GROUP_VIEWER','GROUP_FINANCE'])[role_tag];
 primary_target:=group_value;
 payload:=jsonb_build_object('group_id',group_value::text,'org_id',company_value::text,'display_name',name_value,'phone',phone_value,'tenant_roles',to_jsonb(roles),'group_role',role_value);
 WHEN 7 THEN
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 group_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF group_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 user_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF user_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 role_tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF role_tag NOT BETWEEN 1 AND 3 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 role_value:=(ARRAY['GROUP_ADMIN','GROUP_VIEWER','GROUP_FINANCE'])[role_tag];
 primary_target:=group_value;
 payload:=jsonb_build_object('group_id',group_value::text,'user_id',user_value::text,'group_role',role_value);
 WHEN 8,9 THEN
IF byte_count-cursor_pos<16 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 company_value:=encode(substring(p_input FROM cursor_pos+1 FOR 16),'hex')::uuid; cursor_pos:=cursor_pos+16;
 IF company_value='00000000-0000-0000-0000-000000000000'::uuid THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 primary_target:=company_value;
 IF kind=9 THEN
IF byte_count-cursor_pos<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 status_tag:=get_byte(p_input,cursor_pos); cursor_pos:=cursor_pos+1;
 IF status_tag NOT BETWEEN 1 AND 3 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 status_value:=(ARRAY['ACTIVE','SUSPENDED','ARCHIVED'])[status_tag];
 END IF;
 payload:=jsonb_build_object('org_id',company_value::text);
 IF kind=9 THEN payload:=payload||jsonb_build_object('status',status_value); END IF;
 END CASE;
 IF cursor_pos<>byte_count OR (kind IN (1,2) AND head_count<>0)
  OR (kind IN (3,5,6,8,9) AND head_count<>1) OR (kind IN (4,7) AND head_count NOT BETWEEN 1 AND 2)
  OR (group_value IS NOT NULL AND NOT EXISTS(SELECT 1 FROM jsonb_array_elements(expected_groups) e WHERE e->>'group_id'=group_value::text)) THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input'; END IF;
 input_digest:=sha256(p_input);
 RETURN NEXT;
 EXCEPTION WHEN data_exception THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_input';
END
$body$;


-- source: legacy-receipts-user-successor.sql
CREATE TABLE public.platform_legacy_topology_receipts (
 actor_user_id uuid NOT NULL, command_id uuid NOT NULL,
 receipt_id uuid NOT NULL UNIQUE,
 codec_version smallint NOT NULL CHECK(codec_version=1),
 kind smallint NOT NULL CHECK(kind BETWEEN 1 AND 9),
 input_bytes bytea NOT NULL CHECK(octet_length(input_bytes) BETWEEN 62 AND 4096),
 input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32 AND input_digest=sha256(input_bytes)),
 primary_target uuid,
 outcome text NOT NULL CHECK(outcome IN ('APPLIED','REJECTED')),
 result_code text NOT NULL CHECK(result_code IN ('applied','not_found','revision_conflict','native_origin_denied',
  'slug_conflict','group_inactive','company_inactive','blocked_has_data','invalid_membership','credential_state_conflict')),
 result_org_id uuid, result_group_id uuid, result_user_id uuid,
 effect_family_id uuid NOT NULL, occurred_at timestamptz NOT NULL CHECK(isfinite(occurred_at)),
 effect_xid xid8 NOT NULL, effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 trace_id char(32) NOT NULL CHECK(trace_id ~ '^[0-9a-f]{32}$' AND trace_id<>repeat('0',32)),
 span_id char(16) NOT NULL CHECK(span_id ~ '^[0-9a-f]{16}$' AND span_id<>repeat('0',16)),
 bootstrap_credential_id uuid, bootstrap_expires_at timestamptz CHECK(isfinite(bootstrap_expires_at)),
 builtin_catalog_version text, builtin_catalog_digest bytea,
 heads_before jsonb NOT NULL, heads_after jsonb NOT NULL,
 PRIMARY KEY(actor_user_id,command_id),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (actor_user_id,command_id,receipt_id,effect_family_id)),
 CHECK((kind IN (1,2))=(primary_target IS NULL)),
 CHECK(primary_target IS NULL OR primary_target<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK(result_org_id IS NULL OR result_org_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK(result_group_id IS NULL OR result_group_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK(result_user_id IS NULL OR result_user_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK(bootstrap_credential_id IS NULL OR bootstrap_credential_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK(CASE WHEN jsonb_typeof(heads_before)='array' THEN jsonb_array_length(heads_before)<=CASE WHEN kind=8 THEN 128 ELSE 2 END ELSE false END),
 CHECK(CASE WHEN jsonb_typeof(heads_after)='array' THEN jsonb_array_length(heads_after)<=CASE WHEN kind=8 THEN 128 ELSE 2 END ELSE false END),
 CHECK((outcome='APPLIED')=(result_code='applied')),
 CHECK((outcome='REJECTED' AND result_org_id IS NULL AND result_group_id IS NULL AND result_user_id IS NULL
  AND bootstrap_credential_id IS NULL AND bootstrap_expires_at IS NULL AND builtin_catalog_version IS NULL
  AND builtin_catalog_digest IS NULL AND heads_after='[]'::jsonb)
 OR (outcome='APPLIED' AND
  (result_org_id IS NOT NULL)=(kind IN (1,4,5,6,8,9))
  AND result_group_id IS NOT NULL AND (result_user_id IS NOT NULL)=(kind IN (1,6,7))
  AND (bootstrap_credential_id IS NOT NULL)=(kind IN (1,6))
  AND (bootstrap_expires_at IS NOT NULL)=(kind IN (1,6))
  AND (builtin_catalog_version IS NOT NULL)=(kind=1) AND (builtin_catalog_digest IS NOT NULL)=(kind=1))),
 CHECK(builtin_catalog_version IS NULL OR (builtin_catalog_version='2026-07-19.1'
  AND builtin_catalog_digest=decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex')))
);
-- Current-transaction lookups also run on ordinary User writes with no frame.
-- Nonunique: no new cross-command transaction invariant is introduced by ACL metadata.
CREATE INDEX platform_legacy_topology_receipts_effect_idx
 ON public.platform_legacy_topology_receipts(effect_xid,effect_backend_pid);
ALTER TABLE public.platform_legacy_topology_receipts OWNER TO console_app;
REVOKE ALL ON public.platform_legacy_topology_receipts FROM PUBLIC;

CREATE FUNCTION public.platform_legacy_topology_receipts_immutable_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_immutable';
END
$body$;
CREATE TRIGGER platform_legacy_topology_receipts_immutable_v1
 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.platform_legacy_topology_receipts
 FOR EACH STATEMENT EXECUTE FUNCTION public.platform_legacy_topology_receipts_immutable_v1();
ALTER TABLE public.platform_legacy_topology_receipts ENABLE ALWAYS TRIGGER platform_legacy_topology_receipts_immutable_v1;


-- source: legacy-force-input.sql
-- Private new force protocol candidate. Never applied to historical audit or
-- ordinary nine-kind command bytes; freeze only after independent codec review.
-- Layout: ASCII domain+NUL, actor UUID, command UUID, Company UUID, current Group
-- UUID, expected Group incarnation UUID, positive i64 revision, all network order.
CREATE FUNCTION public.platform_force_remove_decode_input_v1(p_input bytea)
RETURNS TABLE(actor_user_id uuid,command_id uuid,org_id uuid,group_id uuid,
 expected_incarnation uuid,expected_revision bigint,input_digest bytea)
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE prefix bytea:=convert_to('console.platform.force_remove','UTF8')||decode('000001','hex');
 offset_value integer;
BEGIN
 offset_value:=octet_length(prefix);
 IF p_input IS NULL OR octet_length(p_input)<>offset_value+88
  OR substring(p_input FROM 1 FOR offset_value) IS DISTINCT FROM prefix THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_force.invalid_command';
 END IF;
 actor_user_id:=encode(substring(p_input FROM offset_value+1 FOR 16),'hex')::uuid;
 command_id:=encode(substring(p_input FROM offset_value+17 FOR 16),'hex')::uuid;
 org_id:=encode(substring(p_input FROM offset_value+33 FOR 16),'hex')::uuid;
 group_id:=encode(substring(p_input FROM offset_value+49 FOR 16),'hex')::uuid;
 expected_incarnation:=encode(substring(p_input FROM offset_value+65 FOR 16),'hex')::uuid;
 expected_revision:=('x'||encode(substring(p_input FROM offset_value+81 FOR 8),'hex'))::bit(64)::bigint;
 IF '00000000-0000-0000-0000-000000000000'::uuid IN
   (actor_user_id,command_id,org_id,group_id,expected_incarnation)
  OR expected_revision<1 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_force.invalid_command';
 END IF;
 input_digest:=sha256(p_input);
 RETURN NEXT;
END
$body$;
ALTER FUNCTION public.platform_force_remove_decode_input_v1(bytea) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_force_remove_decode_input_v1(bytea) FROM PUBLIC;


-- source: legacy-force-owner-source-successor.sql:schema
-- Private force-only owner component, not an installer. Requires independent
-- protocol review, current-source Rust bridge, exact plan/guard/closure sources.
CREATE TABLE public.platform_force_removal_receipts (
 actor_user_id uuid NOT NULL,command_id uuid NOT NULL,receipt_id uuid NOT NULL UNIQUE,
 input_bytes bytea NOT NULL,input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32 AND input_digest=sha256(input_bytes)),
 org_id uuid NOT NULL,group_id uuid NOT NULL,
 family_id uuid NOT NULL,effect_xid xid8 NOT NULL,effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 outcome text NOT NULL CHECK(outcome IN('APPLIED','REJECTED')),
 result_code text NOT NULL CHECK(result_code IN('removed','not_found','blocked_active','native_origin_denied',
  'revision_conflict','blocked_has_data')),
 occurred_at timestamptz NOT NULL CHECK(isfinite(occurred_at)),
 trace_id char(32) NOT NULL CHECK(trace_id~'^[0-9a-f]{32}$' AND trace_id<>repeat('0',32)),
 span_id char(16) NOT NULL CHECK(span_id~'^[0-9a-f]{16}$' AND span_id<>repeat('0',16)),
 heads_before jsonb NOT NULL,heads_after jsonb NOT NULL,
 PRIMARY KEY(actor_user_id,command_id),UNIQUE(actor_user_id,command_id,receipt_id),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN(actor_user_id,command_id,receipt_id,org_id,group_id,family_id)),
 CHECK(octet_length(input_bytes)=120),
 CHECK((outcome='APPLIED')=(result_code='removed')),
 CHECK(CASE WHEN jsonb_typeof(heads_before)='array' THEN jsonb_array_length(heads_before)<=128 ELSE false END),
 CHECK(CASE WHEN jsonb_typeof(heads_after)='array' THEN jsonb_array_length(heads_after)<=128 ELSE false END),
 CHECK(outcome='APPLIED' OR heads_after='[]'::jsonb)
);
CREATE INDEX platform_force_removal_receipts_effect_idx
 ON public.platform_force_removal_receipts(effect_xid,effect_backend_pid);
ALTER TABLE public.platform_force_removal_receipts OWNER TO console_app;
REVOKE ALL ON public.platform_force_removal_receipts FROM PUBLIC,console_rt,console_auth_rt,
 console_auth_startup,console_leave_cmd,console_ontology_cmd,console_platform_force_cmd,
 console_account_owner,console_terms_owner,console_credential_owner,console_ontology_writer;

CREATE TABLE public.platform_force_removal_effect_bindings (
 actor_user_id uuid NOT NULL,command_id uuid NOT NULL,receipt_id uuid NOT NULL UNIQUE,
 input_bytes bytea NOT NULL,input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32 AND input_digest=sha256(input_bytes)),
 org_id uuid NOT NULL,group_id uuid NOT NULL,family_id uuid NOT NULL,
 effect_xid xid8 NOT NULL,effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 occurred_at timestamptz NOT NULL CHECK(isfinite(occurred_at)),
 trace_id char(32) NOT NULL CHECK(trace_id~'^[0-9a-f]{32}$' AND trace_id<>repeat('0',32)),
 span_id char(16) NOT NULL CHECK(span_id~'^[0-9a-f]{16}$' AND span_id<>repeat('0',16)),
 plan_snapshot jsonb NOT NULL CHECK(jsonb_typeof(plan_snapshot)='object'),
 PRIMARY KEY(actor_user_id,command_id),UNIQUE(effect_xid,effect_backend_pid),
 CHECK(octet_length(input_bytes)=120),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN(actor_user_id,command_id,receipt_id,org_id,group_id,family_id))
);
ALTER TABLE public.platform_force_removal_effect_bindings OWNER TO console_app;
REVOKE ALL ON public.platform_force_removal_effect_bindings FROM PUBLIC,console_rt,console_auth_rt,
 console_auth_startup,console_leave_cmd,console_ontology_cmd,console_platform_force_cmd,
 console_account_owner,console_terms_owner,console_credential_owner,console_ontology_writer;



-- source: topology-force-successor-round2.sql
-- Proposed shape only; never execute separately from complete owning guards,
-- legacy receipt owner, native graph closure and atomic custody successor.
-- All tables public. FKs MATCH SIMPLE / ON UPDATE RESTRICT unless stated.
ALTER TABLE public.platform_legacy_topology_receipts
 ADD CONSTRAINT platform_legacy_topology_receipt_origin_key
 UNIQUE(actor_user_id,command_id,receipt_id);

CREATE TABLE public.company_enrollment_effect_bindings (
 account_id uuid NOT NULL, command_id uuid NOT NULL,
 codec_version smallint NOT NULL CHECK(codec_version=1),
 effect_xid xid8 NOT NULL, effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 org_id uuid NOT NULL UNIQUE, group_id uuid NOT NULL, receipt_id uuid NOT NULL UNIQUE,
 administrative_account_id uuid NOT NULL,
 designation_receipt_id uuid NOT NULL, input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32),
 started_at timestamptz NOT NULL CHECK(isfinite(started_at)),
 catalog_version text NOT NULL CHECK(catalog_version='native-company-identity-2026-09-19.1'),
 manifest_digest bytea NOT NULL CHECK(manifest_digest=decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex')),
 session_id uuid NOT NULL,
 recipient_context_generation_before bigint NOT NULL CHECK(recipient_context_generation_before>=1),
 PRIMARY KEY(account_id,command_id),
 UNIQUE(org_id,account_id,command_id,receipt_id),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
   (account_id,command_id,org_id,group_id,receipt_id,administrative_account_id,designation_receipt_id,session_id)),
 CONSTRAINT company_effect_request_fk FOREIGN KEY(account_id,command_id,codec_version,input_digest,designation_receipt_id)
 REFERENCES public.company_enrollment_requests(account_id,command_id,codec_version,input_digest,designation_receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT,
 CONSTRAINT company_effect_receipt_fk FOREIGN KEY(account_id,command_id,receipt_id)
 REFERENCES public.company_enrollment_receipts(account_id,command_id,receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CONSTRAINT company_effect_org_fk FOREIGN KEY(org_id) REFERENCES public.organizations(id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CONSTRAINT company_effect_group_fk FOREIGN KEY(group_id) REFERENCES public.groups(id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CONSTRAINT company_effect_recipient_fk FOREIGN KEY(administrative_account_id) REFERENCES public.accounts(id)
 ON UPDATE RESTRICT ON DELETE RESTRICT
);
-- session_id deliberately has NO FK to live refresh families.

ALTER TABLE public.organizations
 ADD COLUMN origin_account_id uuid, ADD COLUMN origin_command_id uuid, ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT organizations_native_origin_shape CHECK(
  (origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL)
  OR (origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT organizations_native_origin_fk FOREIGN KEY(origin_account_id,origin_command_id,origin_receipt_id)
 REFERENCES public.company_enrollment_receipts(account_id,command_id,receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE public.groups
 ADD COLUMN origin_account_id uuid, ADD COLUMN origin_command_id uuid, ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT groups_native_origin_shape CHECK(
  (origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL)
  OR (origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT groups_native_origin_fk FOREIGN KEY(origin_account_id,origin_command_id,origin_receipt_id)
 REFERENCES public.company_enrollment_receipts(account_id,command_id,receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE public.group_authority_heads (
 group_id uuid PRIMARY KEY REFERENCES public.groups(id) ON UPDATE RESTRICT ON DELETE CASCADE,
 CONSTRAINT group_authority_heads_group_nonnil CHECK(group_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 revision bigint NOT NULL CHECK(revision>0),
 incarnation uuid NOT NULL CHECK(incarnation<>'00000000-0000-0000-0000-000000000000'::uuid),
 state text NOT NULL CHECK(state IN ('ACTIVE','RETIRED'))
);
CREATE TABLE public.group_membership_revisions (
 group_id uuid NOT NULL, org_id uuid NOT NULL, membership_id uuid NOT NULL,
 revision bigint NOT NULL CHECK(revision>0), incarnation uuid NOT NULL,
 from_time timestamptz NOT NULL CHECK(isfinite(from_time)),
 to_time timestamptz CHECK(isfinite(to_time)),
 state text NOT NULL CHECK(state IN ('ACTIVE','REMOVED')),
 provenance_kind text NOT NULL CHECK(provenance_kind IN
   ('LEGACY_BACKFILL','COMPANY_ENROLLMENT_V1','LEGACY_TOPOLOGY_V1','LEGACY_FORCE_REMOVAL_V1')),
 native_account_id uuid, legacy_actor_user_id uuid, force_actor_user_id uuid, command_id uuid, command_receipt uuid,
 PRIMARY KEY(membership_id,revision),
 UNIQUE(group_id,org_id,membership_id,revision,incarnation),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN(group_id,org_id,membership_id,incarnation)),
 CHECK((state='ACTIVE' AND to_time IS NULL) OR (state='REMOVED' AND to_time IS NOT NULL AND to_time>=from_time)),
 CHECK(
  (provenance_kind='LEGACY_BACKFILL' AND revision=1 AND state='ACTIVE'
    AND native_account_id IS NULL AND legacy_actor_user_id IS NULL AND force_actor_user_id IS NULL AND command_id IS NULL AND command_receipt IS NULL)
  OR (provenance_kind='COMPANY_ENROLLMENT_V1' AND revision=1 AND state='ACTIVE'
    AND native_account_id IS NOT NULL AND legacy_actor_user_id IS NULL AND force_actor_user_id IS NULL AND command_id IS NOT NULL AND command_receipt IS NOT NULL)
  OR (provenance_kind='LEGACY_TOPOLOGY_V1' AND native_account_id IS NULL AND force_actor_user_id IS NULL
    AND legacy_actor_user_id IS NOT NULL AND command_id IS NOT NULL AND command_receipt IS NOT NULL)
  OR (provenance_kind='LEGACY_FORCE_REMOVAL_V1' AND native_account_id IS NULL AND legacy_actor_user_id IS NULL
    AND force_actor_user_id IS NOT NULL AND command_id IS NOT NULL AND command_receipt IS NOT NULL)),
 CHECK((native_account_id IS NULL OR native_account_id<>'00000000-0000-0000-0000-000000000000'::uuid)
   AND (legacy_actor_user_id IS NULL OR legacy_actor_user_id<>'00000000-0000-0000-0000-000000000000'::uuid)
   AND (force_actor_user_id IS NULL OR force_actor_user_id<>'00000000-0000-0000-0000-000000000000'::uuid)
   AND (command_id IS NULL OR command_id<>'00000000-0000-0000-0000-000000000000'::uuid)
   AND (command_receipt IS NULL OR command_receipt<>'00000000-0000-0000-0000-000000000000'::uuid)),
 CONSTRAINT group_history_native_receipt_fk FOREIGN KEY(native_account_id,command_id,command_receipt)
 REFERENCES public.company_enrollment_receipts(account_id,command_id,receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CONSTRAINT group_history_legacy_receipt_fk FOREIGN KEY(legacy_actor_user_id,command_id,command_receipt)
 REFERENCES public.platform_legacy_topology_receipts(actor_user_id,command_id,receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CONSTRAINT group_history_force_receipt_fk FOREIGN KEY(force_actor_user_id,command_id,command_receipt)
 REFERENCES public.platform_force_removal_receipts(actor_user_id,command_id,receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);
-- Nonunique lookup support; this adds no command/history cardinality invariant.
CREATE INDEX group_membership_revisions_force_command_idx
 ON public.group_membership_revisions(force_actor_user_id,command_id);
-- Historical group_id/org_id/actor identifiers intentionally have NO live-root FKs.
-- Mutually exclusive actor columns select exactly one immutable receipt FK.
ALTER TABLE public.group_memberships
 ADD COLUMN membership_id uuid, ADD COLUMN current_revision bigint, ADD COLUMN incarnation uuid;
-- Populate only structural identities; every original legacy column stays.
INSERT INTO public.group_authority_heads(group_id,revision,incarnation,state)
 SELECT id,1,gen_random_uuid(),'ACTIVE' FROM public.groups ORDER BY id;
UPDATE public.group_memberships SET membership_id=gen_random_uuid(),current_revision=1,incarnation=gen_random_uuid();
INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
 state,provenance_kind,native_account_id,legacy_actor_user_id,command_id,command_receipt)
 SELECT group_id,org_id,membership_id,1,incarnation,created_at,NULL,'ACTIVE','LEGACY_BACKFILL',NULL,NULL,NULL,NULL
 FROM public.group_memberships ORDER BY group_id,org_id;
ALTER TABLE public.group_memberships
 ALTER COLUMN membership_id SET NOT NULL, ALTER COLUMN current_revision SET NOT NULL, ALTER COLUMN incarnation SET NOT NULL,
 ADD CONSTRAINT group_memberships_revision_positive CHECK(current_revision>0),
 ADD CONSTRAINT group_memberships_current_history_fk FOREIGN KEY(group_id,org_id,membership_id,current_revision,incarnation)
 REFERENCES public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;


-- source: identity.sql
-- PROPOSED PHYSICAL SHAPE ONLY. NOT AN INSTALLER. Do not run separately.
-- Install transaction must also install DESIGN.md guards/closure, exact ACLs,
-- catalog/CompanyActor native attribution and compatible current readers.
-- Existing receipt/binding/topology owners are prerequisites, never fixtures.

ALTER TABLE public.company_enrollment_receipts ADD CONSTRAINT company_enrollment_identity_origin_key
 UNIQUE(org_id,account_id,command_id,receipt_id);

CREATE TABLE public.native_company_catalog_installs (
 org_id uuid NOT NULL, catalog_version text NOT NULL,
 manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
 origin_account_id uuid NOT NULL, origin_command_id uuid NOT NULL, origin_receipt_id uuid NOT NULL,
 installed_at timestamptz NOT NULL CHECK(isfinite(installed_at)),
 PRIMARY KEY(org_id,catalog_version), UNIQUE(org_id,catalog_version,manifest_digest),
 CHECK(catalog_version='native-company-identity-2026-09-19.1'),
 CHECK(manifest_digest=decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex')),
 FOREIGN KEY(org_id,catalog_version,manifest_digest)
  REFERENCES public.ont_builtin_catalog_installs(org_id,catalog_version,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id)
  REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);
-- These three composite keys expose exact physical identity to native FK maps;
-- they add no writer, reinterpretation, removal or change to old rows.
ALTER TABLE public.ont_object_types ADD CONSTRAINT ont_native_object_ref_key UNIQUE(org_id,id,schema_version);
ALTER TABLE public.ont_action_types ADD CONSTRAINT ont_native_action_ref_key UNIQUE(org_id,object_type_id,id);
ALTER TABLE public.ont_property_defs ADD CONSTRAINT ont_native_property_ref_key UNIQUE(org_id,object_type_id,id);

CREATE TABLE public.native_company_object_refs (
 org_id uuid NOT NULL, catalog_version text NOT NULL, manifest_digest bytea NOT NULL,
 object_key text NOT NULL CHECK(object_key IN ('company_workspace','company_policy_assignment')),
 object_type_id uuid NOT NULL, schema_revision bigint NOT NULL CHECK(schema_revision=1),
 content_digest bytea NOT NULL CHECK(octet_length(content_digest)=32),
 PRIMARY KEY(org_id,catalog_version,object_key),
 UNIQUE(org_id,object_type_id,schema_revision,manifest_digest),
 FOREIGN KEY(org_id,catalog_version,manifest_digest)
  REFERENCES public.native_company_catalog_installs(org_id,catalog_version,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,object_type_id,schema_revision)
  REFERENCES public.ont_object_types(org_id,id,schema_version) ON UPDATE RESTRICT ON DELETE RESTRICT
);
CREATE TABLE public.native_company_action_refs (
 org_id uuid NOT NULL, catalog_version text NOT NULL,
 action_key text NOT NULL CHECK(action_key IN ('context.discover','company.identity.read','company.policy.read','company.policy.assign','company.policy.revoke')),
 object_type_id uuid NOT NULL, action_type_id uuid NOT NULL,
 registration_revision bigint NOT NULL CHECK(registration_revision=1),
 manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
 content_digest bytea NOT NULL CHECK(octet_length(content_digest)=32),
 PRIMARY KEY(org_id,catalog_version,action_key),
 UNIQUE(org_id,object_type_id,action_type_id,registration_revision,manifest_digest),
 FOREIGN KEY(org_id,catalog_version,manifest_digest)
  REFERENCES public.native_company_catalog_installs(org_id,catalog_version,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,object_type_id,registration_revision,manifest_digest)
  REFERENCES public.native_company_object_refs(org_id,object_type_id,schema_revision,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,object_type_id,action_type_id)
  REFERENCES public.ont_action_types(org_id,object_type_id,id) ON UPDATE RESTRICT ON DELETE RESTRICT
);
CREATE TABLE public.native_company_property_refs (
 org_id uuid NOT NULL, catalog_version text NOT NULL,
 property_key text NOT NULL CHECK(property_key IN ('company.name','company.slug','assignment.account_id','assignment.scope','assignment.actions','assignment.fields','assignment.valid_from','assignment.valid_until','assignment.state','assignment.revision')),
 object_type_id uuid NOT NULL, property_id uuid NOT NULL,
 schema_revision bigint NOT NULL CHECK(schema_revision=1),
 manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
 content_digest bytea NOT NULL CHECK(octet_length(content_digest)=32),
 PRIMARY KEY(org_id,catalog_version,property_key),
 UNIQUE(org_id,object_type_id,property_id,schema_revision,manifest_digest,content_digest),
 FOREIGN KEY(org_id,catalog_version,manifest_digest)
  REFERENCES public.native_company_catalog_installs(org_id,catalog_version,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,object_type_id,schema_revision,manifest_digest)
  REFERENCES public.native_company_object_refs(org_id,object_type_id,schema_revision,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,object_type_id,property_id)
  REFERENCES public.ont_property_defs(org_id,object_type_id,id) ON UPDATE RESTRICT ON DELETE RESTRICT
);

ALTER TABLE public.policy_roles
 ADD COLUMN subject_protocol text NOT NULL DEFAULT 'LEGACY_USER',
 ADD COLUMN native_current_revision bigint,
 ADD COLUMN created_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN updated_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN origin_account_id uuid, ADD COLUMN origin_command_id uuid, ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT policy_roles_protocol_shape CHECK(
  (subject_protocol='LEGACY_USER' AND native_current_revision IS NULL
    AND created_by_account_id IS NULL AND updated_by_account_id IS NULL
    AND origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL)
  OR (subject_protocol='NATIVE_ACCOUNT' AND native_current_revision IS NOT NULL AND native_current_revision>=1
    AND created_by IS NULL AND updated_by IS NULL AND created_by_account_id IS NOT NULL
    AND updated_by_account_id IS NOT NULL AND origin_account_id IS NOT NULL
    AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT policy_roles_native_key UNIQUE(org_id,id,subject_protocol),
 ADD CONSTRAINT policy_roles_native_origin FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id)
  REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE public.user_role_assignments
 ALTER COLUMN user_id DROP NOT NULL,
 ADD COLUMN subject_protocol text NOT NULL DEFAULT 'LEGACY_USER',
 ADD COLUMN account_id uuid, ADD COLUMN native_current_revision bigint,
 ADD COLUMN assigned_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN origin_account_id uuid, ADD COLUMN origin_command_id uuid, ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT user_role_assignments_protocol_shape CHECK(
  (subject_protocol='LEGACY_USER' AND user_id IS NOT NULL AND account_id IS NULL
    AND native_current_revision IS NULL AND assigned_by_account_id IS NULL
    AND origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL)
  OR (subject_protocol='NATIVE_ACCOUNT' AND user_id IS NULL AND assigned_by IS NULL AND account_id IS NOT NULL
    AND native_current_revision IS NOT NULL AND native_current_revision>=1 AND assigned_by_account_id IS NOT NULL
    AND origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT user_role_assignments_native_key UNIQUE(org_id,id,subject_protocol),
 ADD CONSTRAINT user_role_assignments_native_actor_fk FOREIGN KEY(org_id,account_id)
  REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 ADD CONSTRAINT user_role_assignments_role_protocol_fk FOREIGN KEY(org_id,role_id,subject_protocol)
  REFERENCES public.policy_roles(org_id,id,subject_protocol) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD CONSTRAINT user_role_assignments_native_origin FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id)
  REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
CREATE UNIQUE INDEX user_role_assignments_native_subject_role
 ON public.user_role_assignments(org_id,account_id,role_id) WHERE subject_protocol='NATIVE_ACCOUNT';

CREATE TABLE public.policy_role_revisions (
 org_id uuid NOT NULL, role_id uuid NOT NULL, revision bigint NOT NULL CHECK(revision>=1),
 subject_protocol text NOT NULL CHECK(subject_protocol='NATIVE_ACCOUNT'),
 state text NOT NULL CHECK(state IN ('ACTIVE','RETIRED')),
 valid_from timestamptz NOT NULL CHECK(isfinite(valid_from)),
 valid_until timestamptz CHECK(valid_until IS NULL OR (isfinite(valid_until) AND valid_until>valid_from)),
 catalog_version text NOT NULL, manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
 clause_digest bytea NOT NULL CHECK(octet_length(clause_digest)=32),
 origin_account_id uuid NOT NULL, origin_command_id uuid NOT NULL, origin_receipt_id uuid NOT NULL,
 actor_account_id uuid NOT NULL REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 session_id uuid NOT NULL CHECK(session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 created_at timestamptz NOT NULL CHECK(isfinite(created_at)),
 PRIMARY KEY(org_id,role_id,revision),
 UNIQUE(org_id,role_id,revision,manifest_digest),
 UNIQUE(org_id,role_id,revision,clause_digest),
 FOREIGN KEY(org_id,role_id,subject_protocol) REFERENCES public.policy_roles(org_id,id,subject_protocol)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(org_id,catalog_version,manifest_digest)
  REFERENCES public.native_company_catalog_installs(org_id,catalog_version,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id)
  REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK(actor_account_id=origin_account_id)
);
CREATE TABLE public.policy_capability_clauses (
 org_id uuid NOT NULL,role_id uuid NOT NULL,role_revision bigint NOT NULL,
 clause_index smallint NOT NULL CHECK(clause_index BETWEEN 1 AND 7),
 effect text NOT NULL CHECK(effect='ALLOW'),
 action_object_type_id uuid NOT NULL,action_type_id uuid NOT NULL,
 registration_revision bigint NOT NULL CHECK(registration_revision>=1),
 manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
 resource_org_id uuid NOT NULL CHECK(resource_org_id=org_id),
 valid_from timestamptz NOT NULL CHECK(isfinite(valid_from)),
 valid_until timestamptz CHECK(valid_until IS NULL OR (isfinite(valid_until) AND valid_until>valid_from)),
 delegable boolean NOT NULL,
 clause_digest bytea NOT NULL CHECK(octet_length(clause_digest)=32),
 PRIMARY KEY(org_id,role_id,role_revision,clause_index),
 UNIQUE(org_id,role_id,role_revision,action_type_id,delegable),
 UNIQUE(org_id,role_id,role_revision,clause_index,action_object_type_id,manifest_digest),
 FOREIGN KEY(org_id,role_id,role_revision,manifest_digest)
  REFERENCES public.policy_role_revisions(org_id,role_id,revision,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,action_object_type_id,action_type_id,registration_revision,manifest_digest)
  REFERENCES public.native_company_action_refs(org_id,object_type_id,action_type_id,registration_revision,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT
);
CREATE TABLE public.policy_capability_clause_fields (
 org_id uuid NOT NULL,role_id uuid NOT NULL,role_revision bigint NOT NULL,clause_index smallint NOT NULL,
 object_type_id uuid NOT NULL,property_id uuid NOT NULL,schema_revision bigint NOT NULL CHECK(schema_revision>=1),
 manifest_digest bytea NOT NULL CHECK(octet_length(manifest_digest)=32),
 content_digest bytea NOT NULL CHECK(octet_length(content_digest)=32),
 PRIMARY KEY(org_id,role_id,role_revision,clause_index,property_id),
 FOREIGN KEY(org_id,role_id,role_revision,clause_index,object_type_id,manifest_digest)
  REFERENCES public.policy_capability_clauses(org_id,role_id,role_revision,clause_index,action_object_type_id,manifest_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,object_type_id,property_id,schema_revision,manifest_digest,content_digest)
  REFERENCES public.native_company_property_refs(org_id,object_type_id,property_id,schema_revision,manifest_digest,content_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT
);
CREATE TABLE public.policy_assignment_revisions (
 org_id uuid NOT NULL,assignment_id uuid NOT NULL,revision bigint NOT NULL CHECK(revision>=1),
 subject_protocol text NOT NULL CHECK(subject_protocol='NATIVE_ACCOUNT'),
 account_id uuid NOT NULL, role_id uuid NOT NULL,role_revision bigint NOT NULL CHECK(role_revision>=1),
 state text NOT NULL CHECK(state IN ('ACTIVE','REVOKED')),
 valid_from timestamptz NOT NULL CHECK(isfinite(valid_from)),
 valid_until timestamptz CHECK(valid_until IS NULL OR (isfinite(valid_until) AND valid_until>valid_from)),
 ceiling_digest bytea NOT NULL CHECK(octet_length(ceiling_digest)=32),
 origin_account_id uuid NOT NULL,origin_command_id uuid NOT NULL,origin_receipt_id uuid NOT NULL,
 actor_account_id uuid NOT NULL REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 session_id uuid NOT NULL CHECK(session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 created_at timestamptz NOT NULL CHECK(isfinite(created_at)),
 PRIMARY KEY(org_id,assignment_id,revision),
 FOREIGN KEY(org_id,assignment_id,subject_protocol)
  REFERENCES public.user_role_assignments(org_id,id,subject_protocol) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(org_id,account_id) REFERENCES public.company_actors(org_id,account_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(org_id,role_id,role_revision,ceiling_digest)
  REFERENCES public.policy_role_revisions(org_id,role_id,revision,clause_digest) ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id)
  REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK(actor_account_id=origin_account_id)
);
ALTER TABLE public.policy_roles ADD CONSTRAINT policy_roles_native_current_fk
 FOREIGN KEY(org_id,id,native_current_revision) REFERENCES public.policy_role_revisions(org_id,role_id,revision)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE public.user_role_assignments ADD CONSTRAINT user_role_assignments_native_current_fk
 FOREIGN KEY(org_id,id,native_current_revision) REFERENCES public.policy_assignment_revisions(org_id,assignment_id,revision)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE public.company_enrollment_receipts ADD CONSTRAINT company_enrollment_native_assignment_revision_fk
 FOREIGN KEY(org_id,root_assignment_id,root_revision) REFERENCES public.policy_assignment_revisions(org_id,assignment_id,revision)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
CREATE TABLE public.company_authority_heads (
 org_id uuid PRIMARY KEY REFERENCES public.organizations(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 epoch bigint NOT NULL CHECK(epoch>=1),
 origin_account_id uuid NOT NULL,origin_command_id uuid NOT NULL,origin_receipt_id uuid NOT NULL,
 FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id)
  REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);
-- Minimum lock privileges, protected by always-on native row/statement guards:
-- console_account_owner UPDATE(native_current_revision) on both existing roots,
-- UPDATE(epoch) on company_authority_heads; no valid UPDATE operation in this increment.
-- Existing legacy table grants are retained. Full privileges/guard source and
-- metadata are mandatory in final integrated resource; this DDL is not standalone.

-- Session attribution is historical, not a permanent credential-retention FK.
-- INSERT guards prove exact live bound family, operator and current xid.
-- Legitimate later credential cleanup cannot erase or rewrite this attribution.


-- source: attribution.sql
-- PROPOSED source shape, not standalone installer. Existing owners/RLS/User FKs remain.

ALTER TABLE public.ont_object_types
 ADD COLUMN attribution_protocol text NOT NULL DEFAULT 'LEGACY_USER',
 ADD COLUMN created_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN origin_account_id uuid,
 ADD COLUMN origin_command_id uuid,
 ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT ont_object_types_attribution_shape CHECK((attribution_protocol='LEGACY_USER' AND created_by_account_id IS NULL AND origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL) OR (attribution_protocol='NATIVE_ACCOUNT' AND created_by IS NULL AND created_by_account_id IS NOT NULL AND origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT ont_object_types_enrollment_origin_fk FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id) REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE public.ont_builtin_catalog_installs
 ADD COLUMN attribution_protocol text NOT NULL DEFAULT 'LEGACY_USER',
 ADD COLUMN installed_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN origin_account_id uuid,
 ADD COLUMN origin_command_id uuid,
 ADD COLUMN origin_receipt_id uuid,
 ALTER COLUMN installed_by DROP NOT NULL,
 ADD CONSTRAINT ont_builtin_catalog_installs_attribution_shape CHECK((attribution_protocol='LEGACY_USER' AND installed_by_account_id IS NULL AND origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL AND installed_by IS NOT NULL) OR (attribution_protocol='NATIVE_ACCOUNT' AND installed_by IS NULL AND installed_by_account_id IS NOT NULL AND origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT ont_builtin_catalog_installs_enrollment_origin_fk FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id) REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE public.cedar_policy_catalog_entries
 ADD COLUMN attribution_protocol text NOT NULL DEFAULT 'LEGACY_USER',
 ADD COLUMN created_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN updated_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN origin_account_id uuid,
 ADD COLUMN origin_command_id uuid,
 ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT cedar_policy_catalog_entries_attribution_shape CHECK((attribution_protocol='LEGACY_USER' AND created_by_account_id IS NULL AND updated_by_account_id IS NULL AND origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL) OR (attribution_protocol='NATIVE_ACCOUNT' AND created_by IS NULL AND updated_by IS NULL AND created_by_account_id IS NOT NULL AND updated_by_account_id IS NOT NULL AND origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT cedar_policy_catalog_entries_enrollment_origin_fk FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id) REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

ALTER TABLE public.ont_object_policies
 ADD COLUMN attribution_protocol text NOT NULL DEFAULT 'LEGACY_USER',
 ADD COLUMN created_by_account_id uuid REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 ADD COLUMN origin_account_id uuid,
 ADD COLUMN origin_command_id uuid,
 ADD COLUMN origin_receipt_id uuid,
 ADD CONSTRAINT ont_object_policies_attribution_shape CHECK((attribution_protocol='LEGACY_USER' AND created_by_account_id IS NULL AND origin_account_id IS NULL AND origin_command_id IS NULL AND origin_receipt_id IS NULL) OR (attribution_protocol='NATIVE_ACCOUNT' AND created_by IS NULL AND created_by_account_id IS NOT NULL AND origin_account_id IS NOT NULL AND origin_command_id IS NOT NULL AND origin_receipt_id IS NOT NULL)),
 ADD CONSTRAINT ont_object_policies_enrollment_origin_fk FOREIGN KEY(org_id,origin_account_id,origin_command_id,origin_receipt_id) REFERENCES public.company_enrollment_receipts(org_id,account_id,command_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;


-- source: actor-check.sql
CREATE FUNCTION public.company_actor_entitlement_shape_v2(p_evidence jsonb) RETURNS boolean
LANGUAGE sql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT (pg_column_size(p_evidence) <= 16384 AND COALESCE(((jsonb_typeof(p_evidence)='object'
 AND p_evidence ?& ARRAY['kind','evidence']
 AND p_evidence - ARRAY['kind','evidence']='{}'::jsonb
 AND ((p_evidence->'kind')='"COMPANY"'::jsonb)
 AND ((jsonb_typeof((p_evidence->'evidence'))='object'
 AND (p_evidence->'evidence') ?& ARRAY['kind','revision']
 AND (p_evidence->'evidence') - ARRAY['kind','revision']='{}'::jsonb
 AND (((p_evidence->'evidence')->'kind')='"GOVERNED"'::jsonb)
 AND (jsonb_typeof(((p_evidence->'evidence')->'revision'))='object'
 AND ((p_evidence->'evidence')->'revision') ?& ARRAY['org_id','object_type_id','instance_id','revision_id','version','row_hash']
 AND ((p_evidence->'evidence')->'revision') - ARRAY['org_id','object_type_id','instance_id','revision_id','version','row_hash']='{}'::jsonb
 AND (NOT ((((p_evidence->'evidence')->'revision')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((p_evidence->'evidence')->'revision')->'org_id'))='string'
 AND ((((p_evidence->'evidence')->'revision')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((p_evidence->'evidence')->'revision')->'object_type_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((p_evidence->'evidence')->'revision')->'object_type_id'))='string'
 AND ((((p_evidence->'evidence')->'revision')->'object_type_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((p_evidence->'evidence')->'revision')->'instance_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((p_evidence->'evidence')->'revision')->'instance_id'))='string'
 AND ((((p_evidence->'evidence')->'revision')->'instance_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((p_evidence->'evidence')->'revision')->'revision_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((p_evidence->'evidence')->'revision')->'revision_id'))='string'
 AND ((((p_evidence->'evidence')->'revision')->'revision_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof((((p_evidence->'evidence')->'revision')->'version'))='string'
 AND ((((p_evidence->'evidence')->'revision')->'version')#>>'{}') ~ '^[1-9][0-9]{0,18}$'
 AND CASE WHEN ((((p_evidence->'evidence')->'revision')->'version')#>>'{}') ~ '^[1-9][0-9]{0,18}$' THEN ((((p_evidence->'evidence')->'revision')->'version')#>>'{}')::numeric <= 9223372036854775807 ELSE false END)
 AND (jsonb_typeof((((p_evidence->'evidence')->'revision')->'row_hash'))='string'
 AND ((((p_evidence->'evidence')->'revision')->'row_hash')#>>'{}') ~ '^[0-9a-f]{64}$')))
 OR (jsonb_typeof((p_evidence->'evidence'))='object'
 AND (p_evidence->'evidence') ?& ARRAY['kind','evidence']
 AND (p_evidence->'evidence') - ARRAY['kind','evidence']='{}'::jsonb
 AND (((p_evidence->'evidence')->'kind')='"RETAINED"'::jsonb)
 AND (jsonb_typeof(((p_evidence->'evidence')->'evidence'))='object'
 AND ((p_evidence->'evidence')->'evidence') ?& ARRAY['unit','evidence_digest','encoded_size']
 AND ((p_evidence->'evidence')->'evidence') - ARRAY['unit','evidence_digest','encoded_size']='{}'::jsonb
 AND (jsonb_typeof((((p_evidence->'evidence')->'evidence')->'unit'))='object'
 AND (((p_evidence->'evidence')->'evidence')->'unit') ?& ARRAY['org_id','unit_id','identity_digest']
 AND (((p_evidence->'evidence')->'evidence')->'unit') - ARRAY['org_id','unit_id','identity_digest']='{}'::jsonb
 AND (NOT (((((p_evidence->'evidence')->'evidence')->'unit')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((p_evidence->'evidence')->'evidence')->'unit')->'org_id'))='string'
 AND (((((p_evidence->'evidence')->'evidence')->'unit')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((((p_evidence->'evidence')->'evidence')->'unit')->'unit_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((p_evidence->'evidence')->'evidence')->'unit')->'unit_id'))='string'
 AND (((((p_evidence->'evidence')->'evidence')->'unit')->'unit_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof(((((p_evidence->'evidence')->'evidence')->'unit')->'identity_digest'))='string'
 AND (((((p_evidence->'evidence')->'evidence')->'unit')->'identity_digest')#>>'{}') ~ '^[0-9a-f]{64}$'))
 AND (jsonb_typeof((((p_evidence->'evidence')->'evidence')->'evidence_digest'))='string'
 AND ((((p_evidence->'evidence')->'evidence')->'evidence_digest')#>>'{}') ~ '^[0-9a-f]{64}$')
 AND (CASE WHEN jsonb_typeof((((p_evidence->'evidence')->'evidence')->'encoded_size'))='number' THEN ((((p_evidence->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric=trunc(((((p_evidence->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric) AND ((((p_evidence->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric>=0 AND ((((p_evidence->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric<=9223372036854775807 ELSE false END)))
 OR (jsonb_typeof((p_evidence->'evidence'))='object'
 AND (p_evidence->'evidence') ?& ARRAY['kind','receipt']
 AND (p_evidence->'evidence') - ARRAY['kind','receipt']='{}'::jsonb
 AND (((p_evidence->'evidence')->'kind')='"OWNER_RECEIPT"'::jsonb)
 AND ((jsonb_typeof(((p_evidence->'evidence')->'receipt'))='object'
 AND ((p_evidence->'evidence')->'receipt') ?& ARRAY['kind','command','receipt_id']
 AND ((p_evidence->'evidence')->'receipt') - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND ((((p_evidence->'evidence')->'receipt')->'kind')='"COMPANY"'::jsonb)
 AND (jsonb_typeof((((p_evidence->'evidence')->'receipt')->'command'))='object'
 AND (((p_evidence->'evidence')->'receipt')->'command') ?& ARRAY['org_id','command_id']
 AND (((p_evidence->'evidence')->'receipt')->'command') - ARRAY['org_id','command_id']='{}'::jsonb
 AND (NOT (((((p_evidence->'evidence')->'receipt')->'command')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((p_evidence->'evidence')->'receipt')->'command')->'org_id'))='string'
 AND (((((p_evidence->'evidence')->'receipt')->'command')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((((p_evidence->'evidence')->'receipt')->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((p_evidence->'evidence')->'receipt')->'command')->'command_id'))='string'
 AND (((((p_evidence->'evidence')->'receipt')->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT ((((p_evidence->'evidence')->'receipt')->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((p_evidence->'evidence')->'receipt')->'receipt_id'))='string'
 AND ((((p_evidence->'evidence')->'receipt')->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(((p_evidence->'evidence')->'receipt'))='object'
 AND ((p_evidence->'evidence')->'receipt') ?& ARRAY['kind','command','receipt_id']
 AND ((p_evidence->'evidence')->'receipt') - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND ((((p_evidence->'evidence')->'receipt')->'kind')='"GROUP"'::jsonb)
 AND (jsonb_typeof((((p_evidence->'evidence')->'receipt')->'command'))='object'
 AND (((p_evidence->'evidence')->'receipt')->'command') ?& ARRAY['group_id','command_id']
 AND (((p_evidence->'evidence')->'receipt')->'command') - ARRAY['group_id','command_id']='{}'::jsonb
 AND (NOT (((((p_evidence->'evidence')->'receipt')->'command')->'group_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((p_evidence->'evidence')->'receipt')->'command')->'group_id'))='string'
 AND (((((p_evidence->'evidence')->'receipt')->'command')->'group_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((((p_evidence->'evidence')->'receipt')->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((p_evidence->'evidence')->'receipt')->'command')->'command_id'))='string'
 AND (((((p_evidence->'evidence')->'receipt')->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT ((((p_evidence->'evidence')->'receipt')->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((p_evidence->'evidence')->'receipt')->'receipt_id'))='string'
 AND ((((p_evidence->'evidence')->'receipt')->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))))))
 OR (jsonb_typeof(p_evidence)='object'
 AND p_evidence ?& ARRAY['kind','command','receipt_id']
 AND p_evidence - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND ((p_evidence->'kind')='"GROUP_RECEIPT"'::jsonb)
 AND (jsonb_typeof((p_evidence->'command'))='object'
 AND (p_evidence->'command') ?& ARRAY['group_id','command_id']
 AND (p_evidence->'command') - ARRAY['group_id','command_id']='{}'::jsonb
 AND (NOT (((p_evidence->'command')->'group_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((p_evidence->'command')->'group_id'))='string'
 AND (((p_evidence->'command')->'group_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((p_evidence->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((p_evidence->'command')->'command_id'))='string'
 AND (((p_evidence->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT ((p_evidence->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((p_evidence->'receipt_id'))='string'
 AND ((p_evidence->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(p_evidence)='object'
 AND p_evidence ?& ARRAY['kind','account_id','event_id']
 AND p_evidence - ARRAY['kind','account_id','event_id']='{}'::jsonb
 AND ((p_evidence->'kind')='"ACCOUNT_SECURITY_EVENT"'::jsonb)
 AND (NOT ((p_evidence->'account_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((p_evidence->'account_id'))='string'
 AND ((p_evidence->'account_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((p_evidence->'event_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((p_evidence->'event_id'))='string'
 AND ((p_evidence->'event_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(p_evidence)='object'
 AND p_evidence ?& ARRAY['kind','manifest']
 AND p_evidence - ARRAY['kind','manifest']='{}'::jsonb
 AND ((p_evidence->'kind')='"INDEPENDENT_CUSTODY"'::jsonb)
 AND (jsonb_typeof((p_evidence->'manifest'))='object'
 AND (p_evidence->'manifest') ?& ARRAY['holder_object_id','immutable_version','content_sha256','encoded_bytes','encryption_key_binding_id']
 AND (p_evidence->'manifest') - ARRAY['holder_object_id','immutable_version','content_sha256','encoded_bytes','encryption_key_binding_id']='{}'::jsonb
 AND (NOT (((p_evidence->'manifest')->'holder_object_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((p_evidence->'manifest')->'holder_object_id'))='string'
 AND (((p_evidence->'manifest')->'holder_object_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof(((p_evidence->'manifest')->'immutable_version'))='string'
 AND char_length((((p_evidence->'manifest')->'immutable_version')#>>'{}'))>=1
 AND char_length((((p_evidence->'manifest')->'immutable_version')#>>'{}'))<=1024)
 AND (jsonb_typeof(((p_evidence->'manifest')->'content_sha256'))='string'
 AND (((p_evidence->'manifest')->'content_sha256')#>>'{}') ~ '^[0-9a-f]{64}$')
 AND (CASE WHEN jsonb_typeof(((p_evidence->'manifest')->'encoded_bytes'))='number' THEN (((p_evidence->'manifest')->'encoded_bytes')#>>'{}')::numeric=trunc((((p_evidence->'manifest')->'encoded_bytes')#>>'{}')::numeric) AND (((p_evidence->'manifest')->'encoded_bytes')#>>'{}')::numeric>=1 AND (((p_evidence->'manifest')->'encoded_bytes')#>>'{}')::numeric<=4194304 ELSE false END)
 AND (NOT (((p_evidence->'manifest')->'encryption_key_binding_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((p_evidence->'manifest')->'encryption_key_binding_id'))='string'
 AND (((p_evidence->'manifest')->'encryption_key_binding_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')))),false)) OR (pg_column_size(p_evidence)<=16384 AND coalesce((jsonb_typeof(p_evidence)='object'
 AND p_evidence ?& ARRAY['kind','account_id','command_id','org_id','receipt_id']
 AND p_evidence - ARRAY['kind','account_id','command_id','org_id','receipt_id']='{}'::jsonb
 AND p_evidence->'kind'='"COMPANY_ENROLLMENT_V1"'::jsonb
 AND jsonb_typeof(p_evidence->'account_id')='string'
 AND p_evidence->>'account_id' ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
 AND p_evidence->>'account_id'<>'00000000-0000-0000-0000-000000000000'
 AND jsonb_typeof(p_evidence->'command_id')='string'
 AND p_evidence->>'command_id' ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
 AND p_evidence->>'command_id'<>'00000000-0000-0000-0000-000000000000'
 AND jsonb_typeof(p_evidence->'org_id')='string'
 AND p_evidence->>'org_id' ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
 AND p_evidence->>'org_id'<>'00000000-0000-0000-0000-000000000000'
 AND jsonb_typeof(p_evidence->'receipt_id')='string'
 AND p_evidence->>'receipt_id' ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
 AND p_evidence->>'receipt_id'<>'00000000-0000-0000-0000-000000000000'
),false));
$body$;
ALTER TABLE public.company_actors DROP CONSTRAINT company_actors_entitlement_ref_check;
ALTER TABLE public.company_actors ADD CONSTRAINT company_actors_entitlement_ref_v2
 CHECK(public.company_actor_entitlement_shape_v2(entitlement_ref));

ALTER TABLE public.organizations DROP CONSTRAINT organizations_slug_check;
ALTER TABLE public.organizations ADD CONSTRAINT organizations_slug_native_v1 CHECK(
 (origin_account_id IS NULL AND slug ~ '^[a-z0-9][a-z0-9-]{1,38}[a-z0-9]$')
 OR (origin_account_id IS NOT NULL AND slug ~ '^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$'));


-- source: native-allowlist
INSERT INTO public.ont_builtin_catalog_allowlist(catalog_version,manifest_digest)
 VALUES('native-company-identity-2026-09-19.1',decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex'));

-- source: binding.sql
CREATE FUNCTION public.company_enrollment_binding_v1(p_account uuid,p_command uuid)
RETURNS TABLE(account_id uuid,command_id uuid,effect_xid xid8,effect_backend_pid integer,
 org_id uuid,group_id uuid,receipt_id uuid,administrative_account_id uuid,
 designation_receipt_id uuid,input_digest bytea,started_at timestamptz,catalog_version text,
 manifest_digest bytea,request_state text,session_id uuid,company_name text,company_slug text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b public.company_enrollment_effect_bindings%ROWTYPE;
 q public.company_enrollment_requests%ROWTYPE; d record;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 SELECT x.* INTO STRICT b FROM public.company_enrollment_effect_bindings x
  WHERE x.account_id=p_account AND x.command_id=p_command;
 SELECT x.* INTO STRICT q FROM public.company_enrollment_requests x
  WHERE x.account_id=p_account AND x.command_id=p_command;
 IF b.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR b.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR b.codec_version IS DISTINCT FROM q.codec_version
  OR b.input_digest IS DISTINCT FROM q.input_digest
  OR b.designation_receipt_id IS DISTINCT FROM q.designation_receipt_id
  OR q.state NOT IN ('PENDING','COMMITTED') OR q.input_bytes IS NULL
  OR (q.state='COMMITTED' AND q.committed_receipt_id IS DISTINCT FROM b.receipt_id)
  OR (q.state='PENDING' AND (q.committed_receipt_id IS NOT NULL OR q.terminal_at IS NOT NULL)) THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 SELECT * INTO STRICT d FROM public.company_enrollment_decode_input_v1(q.input_bytes);
 IF d.account_id IS DISTINCT FROM b.account_id OR d.command_id IS DISTINCT FROM b.command_id
  OR d.input_digest IS DISTINCT FROM b.input_digest OR d.group_id IS NOT NULL
  OR d.administrative_account_id IS DISTINCT FROM b.administrative_account_id THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 RETURN QUERY SELECT b.account_id,b.command_id,b.effect_xid,b.effect_backend_pid,
  b.org_id,b.group_id,b.receipt_id,b.administrative_account_id,b.designation_receipt_id,
  b.input_digest,b.started_at,b.catalog_version,b.manifest_digest,q.state,b.session_id,d.company_name,d.company_slug;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'company_enrollment.binding_invalid';
END
$body$;

CREATE FUNCTION public.company_effect_binding_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE q public.company_enrollment_requests%ROWTYPE; d record; recipient record; source record;
BEGIN
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.owner_required';
 END IF;
 IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_effect_bindings'
  OR TG_OP<>'INSERT' OR TG_LEVEL<>'ROW' OR TG_WHEN<>'BEFORE' THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 SELECT x.* INTO STRICT q FROM public.company_enrollment_requests x
  WHERE x.account_id=NEW.account_id AND x.command_id=NEW.command_id;
 SELECT * INTO STRICT d FROM public.company_enrollment_decode_input_v1(q.input_bytes);
 SELECT x.* INTO STRICT recipient FROM public.account_security x WHERE x.account_id=NEW.administrative_account_id;
 SELECT * INTO STRICT source FROM public.auth_account_session_shared_material_v1(NEW.account_id,NEW.session_id);
 IF q.state IS DISTINCT FROM 'PENDING' OR q.terminal_at IS NOT NULL OR q.committed_receipt_id IS NOT NULL
  OR NEW.codec_version IS DISTINCT FROM q.codec_version OR NEW.input_digest IS DISTINCT FROM q.input_digest
  OR NEW.designation_receipt_id IS DISTINCT FROM q.designation_receipt_id
  OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR NEW.started_at<statement_timestamp() OR NEW.started_at>clock_timestamp() OR NEW.started_at>=q.expires_at
  OR NEW.administrative_account_id IS DISTINCT FROM d.administrative_account_id OR d.group_id IS NOT NULL
  OR d.account_id IS DISTINCT FROM NEW.account_id OR d.command_id IS DISTINCT FROM NEW.command_id
  OR d.input_digest IS DISTINCT FROM NEW.input_digest
  OR recipient.security_state IS DISTINCT FROM 'ACTIVE'
  OR recipient.context_generation IS DISTINCT FROM NEW.recipient_context_generation_before
  OR source.protocol IS DISTINCT FROM 'ACCOUNT_V1' OR source.user_id IS DISTINCT FROM NEW.account_id
  OR source.revoked_at IS NOT NULL OR source.org_id IS NOT NULL
  OR NOT EXISTS(SELECT 1 FROM public.account_security s WHERE s.account_id=NEW.account_id
    AND s.security_state='ACTIVE' AND s.security_generation=source.account_security_generation)
  OR NOT EXISTS(SELECT 1 FROM public.deployment_operator_head h WHERE h.singleton=1
    AND h.account_id=NEW.account_id AND h.receipt_id=NEW.designation_receipt_id) THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'company_enrollment.binding_invalid';
END
$body$;

CREATE FUNCTION public.company_topology_history_immutable_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='company_enrollment.history_immutable';
END
$body$;


-- source: catalog-binding.sql
CREATE FUNCTION public.company_enrollment_catalog_binding_v1(p_org uuid)
RETURNS TABLE(account_id uuid,command_id uuid,effect_xid xid8,effect_backend_pid integer,
 org_id uuid,group_id uuid,receipt_id uuid,administrative_account_id uuid,
 designation_receipt_id uuid,input_digest bytea,started_at timestamptz,catalog_version text,
 manifest_digest bytea,request_state text,session_id uuid,company_name text,company_slug text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE origin public.company_enrollment_effect_bindings%ROWTYPE; b record;
BEGIN
 IF p_org IS NULL OR p_org='00000000-0000-0000-0000-000000000000'::uuid THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 SELECT x.* INTO origin FROM public.company_enrollment_effect_bindings x WHERE x.org_id=p_org;
 IF NOT FOUND THEN RETURN; END IF;
 -- A persisted native origin is never a legacy fallback. The canonical pair
 -- helper verifies current full transaction/backend and the exact request.
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(origin.account_id,origin.command_id);
 IF b.org_id IS DISTINCT FROM p_org THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 RETURN QUERY SELECT b.account_id,b.command_id,b.effect_xid,b.effect_backend_pid,b.org_id,b.group_id,b.receipt_id,
  b.administrative_account_id,b.designation_receipt_id,b.input_digest,b.started_at,b.catalog_version,b.manifest_digest,
  b.request_state,b.session_id,b.company_name,b.company_slug;
END
$body$;
ALTER FUNCTION public.company_enrollment_catalog_binding_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_catalog_binding_v1(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.company_enrollment_catalog_binding_v1(uuid) TO console_account_owner,console_ontology_writer;


-- source: topology-owner-force-successor.sql
CREATE FUNCTION public.platform_create_organization_core_v1(p_org uuid,p_group uuid,p_slug text,p_name text)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record;
BEGIN
 IF p_group IS NULL THEN
  PERFORM set_config('app.current_org',p_org::text,true);
  INSERT INTO public.organizations(id,slug,name) VALUES(p_org,p_slug,p_name);
 ELSE
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(
   (SELECT origin_account_id FROM public.groups WHERE id=p_group),
   (SELECT origin_command_id FROM public.groups WHERE id=p_group));
  IF b.request_state<>'PENDING' OR b.org_id IS DISTINCT FROM p_org OR b.group_id IS DISTINCT FROM p_group
   OR b.company_slug IS DISTINCT FROM p_slug OR b.company_name IS DISTINCT FROM p_name THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
  PERFORM set_config('app.current_org',b.org_id::text,true);
  INSERT INTO public.organizations(id,group_id,slug,name,status,created_at,updated_at,
   origin_account_id,origin_command_id,origin_receipt_id)
  VALUES(p_org,p_group,p_slug,p_name,'ACTIVE',b.started_at,b.started_at,b.account_id,b.command_id,b.receipt_id);
 END IF;
 RETURN p_org;
END
$body$;

CREATE FUNCTION public.company_enrollment_topology_v1(p_account uuid,p_command uuid)
RETURNS TABLE(org_id uuid,group_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; candidate_slug text; suffix integer; inserted uuid;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 IF b.request_state<>'PENDING' THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 FOR suffix IN 1..9 LOOP
  candidate_slug := CASE WHEN suffix=1 THEN 'go-' ELSE 'g'||suffix::text||'-' END
   ||replace(b.org_id::text,'-','');
  INSERT INTO public.groups(id,slug,name,status,created_at,updated_at,
   origin_account_id,origin_command_id,origin_receipt_id)
  VALUES(b.group_id,candidate_slug,b.company_name,'ACTIVE',b.started_at,b.started_at,
   b.account_id,b.command_id,b.receipt_id)
  ON CONFLICT(slug) DO NOTHING RETURNING id INTO inserted;
  EXIT WHEN inserted IS NOT NULL;
 END LOOP;
 IF inserted IS NULL THEN RAISE EXCEPTION USING ERRCODE='23505',MESSAGE='company_enrollment.group_slug_exhausted'; END IF;
 INSERT INTO public.group_authority_heads(group_id,revision,incarnation,state)
  VALUES(b.group_id,1,gen_random_uuid(),'ACTIVE');
 PERFORM public.platform_create_organization_core_v1(b.org_id,b.group_id,b.company_slug,b.company_name);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN QUERY SELECT b.org_id,b.group_id;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.group_authority_lock_shared_v1(p_group uuid)
RETURNS TABLE(revision bigint,incarnation uuid,state text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 PERFORM 1 FROM public.group_authority_heads h WHERE h.group_id=p_group FOR SHARE;
 RETURN QUERY SELECT h.revision,h.incarnation,h.state FROM public.group_authority_heads h WHERE h.group_id=p_group;
END
$body$;
CREATE FUNCTION public.group_authority_lock_exclusive_v1(p_group uuid)
RETURNS TABLE(revision bigint,incarnation uuid,state text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 PERFORM 1 FROM public.group_authority_heads h WHERE h.group_id=p_group FOR UPDATE;
 RETURN QUERY SELECT h.revision,h.incarnation,h.state FROM public.group_authority_heads h WHERE h.group_id=p_group;
END
$body$;

CREATE FUNCTION public.company_topology_truncate_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 PERFORM public.platform_force_effect_admit_v1(TG_TABLE_NAME,TG_OP,NULL,NULL);
 IF EXISTS(SELECT 1 FROM public.groups WHERE origin_account_id IS NOT NULL)
  OR EXISTS(SELECT 1 FROM public.organizations WHERE origin_account_id IS NOT NULL) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='company_enrollment.native_origin';
 END IF;
 RETURN NULL;
END
$body$;

CREATE FUNCTION public.company_native_topology_birth_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_value jsonb:=to_jsonb(NEW); account uuid; command uuid;
BEGIN
 IF TG_TABLE_NAME='company_enrollment_effect_bindings' THEN
  account:=NEW.account_id; command:=NEW.command_id;
 ELSIF TG_TABLE_NAME IN ('organizations','groups') THEN
  account:=NEW.origin_account_id; command:=NEW.origin_command_id;
 ELSIF TG_TABLE_NAME='group_membership_revisions' THEN
  account:=NEW.native_account_id; command:=NEW.command_id;
 ELSE
  SELECT b.account_id,b.command_id INTO account,command
   FROM public.company_enrollment_effect_bindings b
   WHERE b.group_id=(row_value->>'group_id')::uuid
    AND (TG_TABLE_NAME='group_authority_heads' OR b.org_id=(row_value->>'org_id')::uuid);
 END IF;
 IF account IS NOT NULL THEN
  PERFORM public.company_enrollment_assert_closure_v1(account,command);
 END IF;
 RETURN NEW;
END
$body$;


-- source: topology-guards-force-successor.sql
CREATE FUNCTION public.company_topology_write_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE value jsonb; previous jsonb; b record; origin_account uuid; origin_command uuid; origin_receipt uuid;
 group_row public.groups%ROWTYPE; organization_row public.organizations%ROWTYPE;
BEGIN
 IF current_user<>'console_app' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.topology_owner_required';
 END IF;
 IF public.platform_force_effect_admit_v1(TG_TABLE_NAME,TG_OP,
  CASE WHEN TG_OP='INSERT' THEN NULL ELSE to_jsonb(OLD) END,
  CASE WHEN TG_OP='DELETE' THEN NULL ELSE to_jsonb(NEW) END) THEN
  IF TG_OP='DELETE' THEN RETURN OLD;ELSE RETURN NEW;END IF;
 END IF;
 IF TG_OP<>'INSERT' THEN previous:=to_jsonb(OLD); END IF;
 IF TG_OP<>'DELETE' THEN value:=to_jsonb(NEW); ELSE value:=previous; END IF;
 IF TG_TABLE_NAME IN ('organizations','groups') THEN
  origin_account:=(value->>'origin_account_id')::uuid;
  origin_command:=(value->>'origin_command_id')::uuid;
  origin_receipt:=(value->>'origin_receipt_id')::uuid;
  IF TG_OP<>'INSERT' AND (previous->>'origin_account_id' IS NOT NULL
    OR value->'origin_account_id' IS DISTINCT FROM previous->'origin_account_id'
    OR value->'origin_command_id' IS DISTINCT FROM previous->'origin_command_id'
    OR value->'origin_receipt_id' IS DISTINCT FROM previous->'origin_receipt_id') THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.native_origin_denied';
  END IF;
  IF origin_account IS NOT NULL THEN
   SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(origin_account,origin_command);
   IF b.request_state<>'PENDING' OR b.receipt_id IS DISTINCT FROM origin_receipt OR TG_OP<>'INSERT'
    OR (value->>'id')::uuid IS DISTINCT FROM (CASE WHEN TG_TABLE_NAME='organizations' THEN b.org_id ELSE b.group_id END)
    OR (value->>'created_at')::timestamptz IS DISTINCT FROM b.started_at
    OR (value->>'updated_at')::timestamptz IS DISTINCT FROM b.started_at
    OR value->>'name' IS DISTINCT FROM b.company_name OR value->>'status' IS DISTINCT FROM 'ACTIVE'
    OR (TG_TABLE_NAME='organizations' AND ((value->>'group_id')::uuid IS DISTINCT FROM b.group_id OR value->>'slug' IS DISTINCT FROM b.company_slug)) THEN
    RAISE EXCEPTION 'company_enrollment.binding_invalid';
   END IF;
  ELSIF TG_TABLE_NAME='organizations' AND (
   EXISTS(SELECT 1 FROM public.groups g WHERE g.id=(value->>'group_id')::uuid AND g.origin_account_id IS NOT NULL)
   OR EXISTS(SELECT 1 FROM public.groups g WHERE g.id=(previous->>'group_id')::uuid AND g.origin_account_id IS NOT NULL)) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.native_origin_denied';
  END IF;
 ELSIF TG_TABLE_NAME IN ('group_memberships','group_authority_heads','group_membership_revisions') THEN
  SELECT g.* INTO group_row FROM public.groups g WHERE g.id=(value->>'group_id')::uuid;
  IF TG_TABLE_NAME<>'group_authority_heads' THEN
   SELECT o.* INTO organization_row FROM public.organizations o WHERE o.id=(value->>'org_id')::uuid;
  END IF;
  IF group_row.origin_account_id IS NOT NULL THEN
   SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(group_row.origin_account_id,group_row.origin_command_id);
   IF TG_OP<>'INSERT' OR b.request_state<>'PENDING' OR b.group_id IS DISTINCT FROM group_row.id
    OR group_row.origin_receipt_id IS DISTINCT FROM b.receipt_id THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.native_origin_denied';
   END IF;
   IF TG_TABLE_NAME='group_authority_heads' THEN
    IF NEW.revision<>1 OR NEW.state<>'ACTIVE' THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
   ELSIF organization_row.id IS DISTINCT FROM b.org_id OR organization_row.origin_account_id IS DISTINCT FROM b.account_id
    OR organization_row.origin_command_id IS DISTINCT FROM b.command_id OR organization_row.origin_receipt_id IS DISTINCT FROM b.receipt_id THEN
    RAISE EXCEPTION 'company_enrollment.binding_invalid';
   ELSIF TG_TABLE_NAME='group_memberships' THEN
    IF NEW.current_revision<>1 OR NEW.created_at IS DISTINCT FROM b.started_at THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
   ELSIF NEW.revision<>1 OR NEW.state<>'ACTIVE' OR NEW.from_time IS DISTINCT FROM b.started_at OR NEW.to_time IS NOT NULL
    OR NEW.provenance_kind IS DISTINCT FROM 'COMPANY_ENROLLMENT_V1' OR NEW.native_account_id IS DISTINCT FROM b.account_id
    OR NEW.command_id IS DISTINCT FROM b.command_id OR NEW.command_receipt IS DISTINCT FROM b.receipt_id THEN
    RAISE EXCEPTION 'company_enrollment.binding_invalid';
   END IF;
  ELSE
   IF TG_TABLE_NAME<>'group_authority_heads' AND organization_row.origin_account_id IS NOT NULL THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.native_origin_denied';
   END IF;
   IF TG_OP<>'INSERT' AND EXISTS(SELECT 1 FROM public.groups g WHERE g.id=(previous->>'group_id')::uuid AND g.origin_account_id IS NOT NULL) THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.native_origin_denied';
   END IF;
   IF TG_TABLE_NAME='group_authority_heads' AND TG_OP='UPDATE' THEN
    IF NEW.group_id IS DISTINCT FROM OLD.group_id OR NEW.incarnation IS DISTINCT FROM OLD.incarnation
     OR NEW.revision IS DISTINCT FROM OLD.revision+1 OR NEW.state IS DISTINCT FROM OLD.state THEN
     RAISE EXCEPTION 'company_enrollment.topology_revision_invalid';
    END IF;
   ELSIF TG_TABLE_NAME='group_membership_revisions' THEN
    IF NEW.provenance_kind IS DISTINCT FROM 'LEGACY_TOPOLOGY_V1' THEN
     RAISE EXCEPTION 'company_enrollment.topology_revision_invalid';
    END IF;
    -- Actual kind/result/heads are closed against the deferred owning receipt;
    -- the app-only history writer cannot use backfill as runtime provenance.
   END IF;
  END IF;
 ELSE RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 RETURN NEW;
END
$body$;

CREATE OR REPLACE FUNCTION public.platform_attach_membership(p_group_id uuid,p_org_id uuid)
RETURNS void LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE organization_row public.organizations%ROWTYPE; b record; prior public.group_memberships%ROWTYPE;
 member uuid; incarnation_value uuid; event_time timestamptz;
BEGIN
 SELECT x.* INTO STRICT organization_row FROM public.organizations x WHERE x.id=p_org_id;
 SELECT x.* INTO prior FROM public.group_memberships x WHERE x.org_id=p_org_id;
 IF prior.group_id=p_group_id THEN RETURN; END IF;
 member:=gen_random_uuid(); incarnation_value:=gen_random_uuid(); event_time:=clock_timestamp();
 IF organization_row.origin_account_id IS NOT NULL THEN
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(organization_row.origin_account_id,organization_row.origin_command_id);
  IF b.request_state<>'PENDING' OR b.group_id IS DISTINCT FROM p_group_id OR b.org_id IS DISTINCT FROM p_org_id OR prior.org_id IS NOT NULL THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
  event_time:=b.started_at;
  INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
   state,provenance_kind,native_account_id,legacy_actor_user_id,command_id,command_receipt)
  VALUES(p_group_id,p_org_id,member,1,incarnation_value,event_time,NULL,'ACTIVE','COMPANY_ENROLLMENT_V1',b.account_id,NULL,b.command_id,b.receipt_id);
 END IF;
 INSERT INTO public.group_memberships(group_id,org_id,created_at,membership_id,current_revision,incarnation)
 VALUES(p_group_id,p_org_id,event_time,member,1,incarnation_value)
 ON CONFLICT(org_id) DO UPDATE SET group_id=EXCLUDED.group_id,created_at=EXCLUDED.created_at,
  membership_id=EXCLUDED.membership_id,current_revision=EXCLUDED.current_revision,incarnation=EXCLUDED.incarnation;
END
$body$;


-- source: catalog-core.sql
CREATE OR REPLACE FUNCTION ontology_api.install_builtin_catalog_core_v1(
    p_org_id UUID,
    p_catalog_version TEXT,
    p_manifest JSONB,
    p_actor UUID,
    p_trace_id TEXT,
    p_span_id TEXT,
    p_native_command UUID
)
RETURNS TABLE(installed BOOLEAN, object_type_count BIGINT)
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, pg_temp
SET row_security = on
AS $$
DECLARE
    b RECORD;
    native_receipt UUID;
    native_digest BYTEA;
    v_digest BYTEA;
    v_allowed_digest BYTEA;
    v_snapshot JSONB;
    v_link JSONB;
    v_links JSONB;
    v_stable_key TEXT;
    v_target_key TEXT;
    v_target_id UUID;
    v_id UUID;
    v_count BIGINT;
    v_upgrade BOOLEAN;
    v_new_keys TEXT[] := ARRAY[]::TEXT[];
    v_retained_keys TEXT[] := ARRAY[]::TEXT[];
    v_existing_backing_kind TEXT;
    v_existing_backing_table TEXT;
    v_existing_primary_key TEXT;
    v_occurred_at TIMESTAMPTZ := pg_catalog.statement_timestamp();
BEGIN
    IF p_native_command IS NULL THEN
        PERFORM ontology_api.assert_write_context(p_org_id,p_actor,p_trace_id,p_span_id);
    ELSE
        SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_actor,p_native_command);
        IF b.request_state<>'PENDING' OR b.org_id IS DISTINCT FROM p_org_id
          OR b.catalog_version IS DISTINCT FROM p_catalog_version
          OR p_trace_id !~ '^[0-9a-f]{32}$' OR p_span_id !~ '^[0-9a-f]{16}$'
          OR p_trace_id=repeat('0',32) OR p_span_id=repeat('0',16)
          OR p_trace_id IS NULL OR p_span_id IS NULL THEN
            RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        v_occurred_at:=b.started_at;
        native_receipt:=b.receipt_id;
        native_digest:=b.manifest_digest;
    END IF;
    IF pg_catalog.jsonb_typeof(p_manifest) <> 'object'
       OR p_manifest->>'catalog_version' IS DISTINCT FROM p_catalog_version
       OR pg_catalog.jsonb_typeof(p_manifest->'object_types') <> 'array' THEN
        RAISE EXCEPTION USING ERRCODE = '22023', MESSAGE = 'ontology_builtin.invalid_manifest_shape';
    END IF;

    v_digest := public.digest(pg_catalog.convert_to(p_manifest::TEXT, 'UTF8'), 'sha256');
    SELECT a.manifest_digest INTO v_allowed_digest
      FROM public.ont_builtin_catalog_allowlist a
     WHERE a.catalog_version = p_catalog_version;
    IF v_allowed_digest IS NULL OR v_allowed_digest <> v_digest THEN
        RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_builtin.manifest_not_allowlisted';
    END IF;

    -- Share the org-scoped bootstrap/write lock with ordinary creation. The
    -- lock intentionally excludes catalog version so two versions cannot race
    -- one another or race a first custom type.
    PERFORM pg_catalog.pg_advisory_xact_lock(
        pg_catalog.hashtextextended('ontology-bootstrap:' || p_org_id::TEXT, 0)
    );

    v_count := pg_catalog.jsonb_array_length(p_manifest->'object_types')::BIGINT;

    -- Exact re-application of a version this tenant already recorded is a
    -- database-owned no-op, which is what makes install idempotent under retry.
    IF EXISTS (
        SELECT 1 FROM public.ont_builtin_catalog_installs i
         WHERE i.org_id = p_org_id
           AND i.catalog_version = p_catalog_version
           AND i.manifest_digest = v_digest
    ) THEN
        IF p_native_command IS NOT NULL THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
        RETURN QUERY SELECT FALSE, v_count;
        RETURN;
    END IF;

    v_upgrade := EXISTS (
        SELECT 1 FROM public.ont_builtin_catalog_installs i WHERE i.org_id = p_org_id
    );
    IF p_native_command IS NOT NULL AND (v_upgrade OR v_digest IS DISTINCT FROM native_digest OR v_count<>2) THEN
        RAISE EXCEPTION 'company_enrollment.binding_invalid';
    END IF;
    IF NOT v_upgrade AND EXISTS (
        SELECT 1 FROM public.ont_object_types o WHERE o.org_id = p_org_id
    ) THEN
        RAISE EXCEPTION USING ERRCODE = '23514', MESSAGE = 'ontology_builtin.empty_org_required';
    END IF;

    -- Pass 1 creates every key this tenant does not already hold, plus that
    -- key's non-link children. IDs are generated in the database, and every
    -- built-in starts published without exposing a general draft->published
    -- capability. Keys the tenant already holds are recorded as retained and
    -- otherwise untouched.
    FOR v_snapshot IN
        SELECT value FROM pg_catalog.jsonb_array_elements(p_manifest->'object_types')
    LOOP
        v_stable_key := pg_catalog.btrim(v_snapshot->>'stable_key');
        IF EXISTS (
            SELECT 1 FROM pg_catalog.jsonb_array_elements(COALESCE(v_snapshot->'links', '[]'::JSONB)) l
            WHERE l ? 'to_object_type_id' AND l->>'to_object_type_id' IS NOT NULL
        ) THEN
            RAISE EXCEPTION USING ERRCODE = '22023', MESSAGE = 'ontology_builtin.physical_link_id_forbidden';
        END IF;

        SELECT o.backing_kind, o.backing_table, o.primary_key_property
          INTO v_existing_backing_kind, v_existing_backing_table, v_existing_primary_key
          FROM public.ont_object_types o
         WHERE o.org_id = p_org_id AND o.stable_key = v_stable_key
         ORDER BY o.schema_version DESC
         LIMIT 1;
        IF FOUND THEN
            IF p_native_command IS NOT NULL THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
            IF v_existing_backing_kind IS DISTINCT FROM v_snapshot->>'backing_kind'
               OR v_existing_backing_table IS DISTINCT FROM NULLIF(v_snapshot->>'backing_table', '')
               OR v_existing_primary_key IS DISTINCT FROM NULLIF(v_snapshot->>'primary_key_property', '') THEN
                RAISE EXCEPTION USING ERRCODE = '23514',
                    MESSAGE = 'ontology_builtin.existing_key_projection_conflict';
            END IF;
            v_retained_keys := v_retained_keys || v_stable_key;
            CONTINUE;
        END IF;

        v_new_keys := v_new_keys || v_stable_key;
        v_id := public.gen_random_uuid();
        INSERT INTO public.ont_object_type_key_revisions (org_id, stable_key,created_at,updated_at)
        VALUES (p_org_id,v_stable_key,
          CASE WHEN p_native_command IS NULL THEN now() ELSE v_occurred_at END,
          CASE WHEN p_native_command IS NULL THEN now() ELSE v_occurred_at END);
        INSERT INTO public.ont_object_types
            (id, org_id, stable_key, title, title_property_key, backing_kind,
             backing_table, primary_key_property, schema_version, lifecycle_state,
             created_by, created_at, updated_at,attribution_protocol,created_by_account_id,
             origin_account_id,origin_command_id,origin_receipt_id)
        VALUES
            (v_id, p_org_id, v_stable_key, pg_catalog.btrim(v_snapshot->>'title'),
             NULLIF(v_snapshot->>'title_property_key', ''), v_snapshot->>'backing_kind',
             NULLIF(v_snapshot->>'backing_table', ''), NULLIF(v_snapshot->>'primary_key_property', ''),
             1,'published',CASE WHEN p_native_command IS NULL THEN p_actor END,v_occurred_at,v_occurred_at,
             CASE WHEN p_native_command IS NULL THEN 'LEGACY_USER' ELSE 'NATIVE_ACCOUNT' END,
             CASE WHEN p_native_command IS NOT NULL THEN p_actor END,
             CASE WHEN p_native_command IS NOT NULL THEN p_actor END,p_native_command,
             native_receipt);
        PERFORM ontology_api.insert_children(
            p_org_id, v_id, pg_catalog.jsonb_set(v_snapshot, '{links}', '[]'::JSONB, TRUE), FALSE);
        IF p_native_command IS NOT NULL THEN
            PERFORM public.company_enrollment_ontology_audit_v1(p_actor,p_native_command,
              'ontology.object_type.builtin_install',v_id,p_trace_id,p_span_id);
        ELSE
        PERFORM ontology_api.write_audit(
            p_org_id, p_actor, 'ontology.object_type.builtin_install', v_id, NULL,
            pg_catalog.jsonb_build_object('stable_key', v_stable_key,
                                          'schema_version', 1,
                                          'lifecycle_state', 'published',
                                          'catalog_version', p_catalog_version,
                                          'manifest_digest', pg_catalog.encode(v_digest, 'hex')),
            p_trace_id, p_span_id, v_occurred_at);
        END IF;
    END LOOP;

    -- Pass 2 resolves logical link targets for the newly installed types only,
    -- against this tenant's published heads — which may have arrived with an
    -- earlier catalog version — then enters the same private child validator.
    -- Types the tenant already held gain no links: retained means untouched.
    FOR v_snapshot IN
        SELECT value FROM pg_catalog.jsonb_array_elements(p_manifest->'object_types')
    LOOP
        v_stable_key := pg_catalog.btrim(v_snapshot->>'stable_key');
        IF NOT (v_stable_key = ANY (v_new_keys)) THEN
            CONTINUE;
        END IF;
        SELECT o.id INTO v_id
          FROM public.ont_object_types o
         WHERE o.org_id = p_org_id AND o.stable_key = v_stable_key AND o.schema_version = 1;
        v_links := '[]'::JSONB;
        FOR v_link IN
            SELECT value FROM pg_catalog.jsonb_array_elements(COALESCE(v_snapshot->'links', '[]'::JSONB))
        LOOP
            v_target_key := NULLIF(pg_catalog.btrim(v_link->>'to_stable_key'), '');
            v_target_id := NULL;
            IF v_target_key IS NOT NULL THEN
                SELECT o.id INTO v_target_id
                  FROM public.ont_object_types o
                 WHERE o.org_id = p_org_id
                   AND o.stable_key = v_target_key
                   AND o.lifecycle_state = 'published';
                IF v_target_id IS NULL THEN
                    RAISE EXCEPTION USING ERRCODE = '23503', MESSAGE = 'ontology_builtin.link_target_not_found';
                END IF;
            END IF;
            v_links := v_links || pg_catalog.jsonb_build_array(
                (v_link - 'to_stable_key' - 'to_object_type_id')
                || pg_catalog.jsonb_build_object('to_object_type_id', v_target_id)
            );
        END LOOP;
        PERFORM ontology_api.insert_children(
            p_org_id, v_id, pg_catalog.jsonb_set(v_snapshot, '{links}', v_links, TRUE), TRUE);
    END LOOP;

    INSERT INTO public.ont_builtin_catalog_installs
        (org_id,catalog_version,manifest_digest,installed_by,installed_at,attribution_protocol,
         installed_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
    VALUES
        (p_org_id,p_catalog_version,v_digest,CASE WHEN p_native_command IS NULL THEN p_actor END,v_occurred_at,
         CASE WHEN p_native_command IS NULL THEN 'LEGACY_USER' ELSE 'NATIVE_ACCOUNT' END,
         CASE WHEN p_native_command IS NOT NULL THEN p_actor END,
         CASE WHEN p_native_command IS NOT NULL THEN p_actor END,p_native_command,
         native_receipt);

    -- The marker append is itself a governed state change — on an upgrade that
    -- adds no key it is the ONLY change — and the retained-key list is the
    -- record of what the installer deliberately did not touch. Neither action
    -- name is in the protected set enforced by
    -- ontology_api.protected_audit_writer_guard, and neither satisfies the
    -- per-mutation audit requirement: the per-type builtin_install rows above
    -- still do that on their own.
    IF p_native_command IS NULL THEN
    INSERT INTO public.audit_events
        (id, actor, action, target_type, target_id, branch_id, before_snap, after_snap,
         trace_id, span_id, occurred_at, org_id)
    VALUES
        (public.gen_random_uuid(), p_actor,
         CASE WHEN v_upgrade THEN 'ontology.builtin_catalog.upgrade'
              ELSE 'ontology.builtin_catalog.install' END,
         'ont_builtin_catalog_installs', p_org_id::TEXT, NULL, NULL,
         pg_catalog.jsonb_build_object('catalog_version', p_catalog_version,
                                       'manifest_digest', pg_catalog.encode(v_digest, 'hex'),
                                       'installed_keys', pg_catalog.to_jsonb(v_new_keys),
                                       'retained_keys', pg_catalog.to_jsonb(v_retained_keys)),
         p_trace_id, p_span_id, v_occurred_at, p_org_id);
    END IF;

    RETURN QUERY SELECT TRUE, v_count;
END;
$$;
CREATE OR REPLACE FUNCTION ontology_api.install_builtin_catalog(
 p_org_id uuid,p_catalog_version text,p_manifest jsonb,p_actor uuid,p_trace_id text,p_span_id text)
RETURNS TABLE(installed boolean,object_type_count bigint)
LANGUAGE sql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
 SELECT * FROM ontology_api.install_builtin_catalog_core_v1($1,$2,$3,$4,$5,$6,NULL);
$body$;


-- source: policy-core.sql
CREATE FUNCTION ont_policy_api.attach_object_policy_rows_core_v1(
    p_org_id UUID,
    p_created_by UUID,
    p_object_type_id UUID,
    p_effect TEXT,
    p_normalized_row JSONB,
    p_schema_version TEXT,
    p_native_command UUID
)
RETURNS UUID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog, pg_temp
SET row_security = on
AS $$
DECLARE
    b RECORD;
    native_receipt UUID;
    event_time timestamptz:=now();
    v_policy_id UUID;
    v_type_key TEXT;
BEGIN
    -- The attached type, resolved under the caller's org floor. There is no FK
    -- on ont_object_policies.object_type_id (0154:29), so an unresolvable or
    -- foreign id would otherwise attach an enforced policy to nothing.
    SELECT stable_key INTO v_type_key
      FROM public.ont_object_types
     WHERE id = p_object_type_id;
    IF v_type_key IS NULL THEN
        RAISE EXCEPTION 'attach refused: unknown object type %', p_object_type_id;
    END IF;

    -- A row whose resource_type disagrees with the type it is attached to never
    -- matches `applicable_object_policies`, so it denies forever at HTTP 200 []
    -- — a silent failure no post-hoc test can see.
    IF p_normalized_row ->> 'resource_type' IS DISTINCT FROM v_type_key THEN
        RAISE EXCEPTION 'attach refused: normalized_row resource_type % is not the attached object type %',
            p_normalized_row ->> 'resource_type', v_type_key;
    END IF;

    -- The route pins this to `authoring::OBJECT_POLICY_ACTION`; so does the
    -- read path's `applicable_object_policies` filter.
    IF p_normalized_row ->> 'action' IS DISTINCT FROM 'view' THEN
        RAISE EXCEPTION 'attach refused: normalized_row action % is not the object-policy action',
            p_normalized_row ->> 'action';
    END IF;

    -- The 0170 trigger only compares CATALOG to ATTACHMENT, and both of those
    -- are bound from p_effect below, so a blocks/catalog disagreement sails
    -- through it. This is the only place that comparison happens.
    IF p_normalized_row ->> 'effect' IS DISTINCT FROM p_effect THEN
        RAISE EXCEPTION 'attach refused: normalized_row effect % disagrees with the attachment effect %',
            p_normalized_row ->> 'effect', p_effect;
    END IF;

    -- Mirrors MAX_ATTACHED_CONDITIONS (`ontology/rest/src/lib.rs:494`). Both
    -- tables are append-only, so an oversized list is permanent work charged to
    -- every later read of the type.
    IF jsonb_typeof(p_normalized_row -> 'conditions') IS DISTINCT FROM 'array'
       OR jsonb_array_length(p_normalized_row -> 'conditions') > 32 THEN
        RAISE EXCEPTION 'attach refused: conditions must be an array of at most 32 entries';
    END IF;

    IF p_native_command IS NOT NULL THEN
        SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_created_by,p_native_command);
        IF b.request_state<>'PENDING' OR b.org_id IS DISTINCT FROM p_org_id
          OR v_type_key NOT IN ('company_workspace','company_policy_assignment')
          OR p_effect IS DISTINCT FROM 'forbid'
          OR p_schema_version IS DISTINCT FROM '2026-07-ontology-authoring-v1'
          OR p_normalized_row IS DISTINCT FROM jsonb_build_object('effect','forbid','action','view','resource_type',v_type_key,'conditions','[]'::jsonb)
          OR NOT EXISTS(SELECT 1 FROM public.ont_object_types o WHERE o.org_id=b.org_id AND o.id=p_object_type_id
            AND o.attribution_protocol='NATIVE_ACCOUNT' AND o.origin_account_id=b.account_id
            AND o.origin_command_id=b.command_id AND o.origin_receipt_id=b.receipt_id) THEN
            RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        native_receipt:=b.receipt_id;
        event_time:=b.started_at;
    END IF;
    INSERT INTO public.cedar_policy_catalog_entries
        (org_id, stable_key, title, natural_language_rule, effect, status, source,
         principal, action, resource, conditions,
         policy_version, schema_version, bundle_digest,
         validation_status, normalized_row, generated_policy_text,
         created_by,updated_by,created_at,updated_at,attribution_protocol,
         created_by_account_id,updated_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
    VALUES
        (p_org_id,
         -- Dotted with >= 2 segments (0150:11) and UNIQUE per (org, key, status),
         -- so it carries the type it policies plus a fresh discriminator: two
         -- policies on one type is a supported shape, not a conflict.
         'object_policy.' || v_type_key || '.' || replace(gen_random_uuid()::text, '-', ''),
         -- title is bounded to 120 chars and the rule to 1000 (0150:12-13),
         -- while an object-type key has no length CHECK at all (0152:20 is
         -- shape-only). Bound the human-readable label; never the matching key.
         'Object policy: ' || left(v_type_key, 80),
         CASE p_effect WHEN 'permit' THEN 'Permit' ELSE 'Forbid' END
             || ' viewing rows of ' || left(v_type_key, 80)
             || ' when every authored condition holds.',
         p_effect,
         'enforced', 'no_code_draft',
         '{}'::jsonb, '{}'::jsonb, '{}'::jsonb, '[]'::jsonb,
         1, p_schema_version,
         -- Derived from the row actually stored beside it. A supplied digest is a
         -- stored false attestation that nothing ever re-checks. `normalized_row`
         -- and not the policy text, because the policy text is no longer stored:
         -- a digest over a NULL is NULL and fails the enforced-row CHECK, and a
         -- digest over the empty string would attest nothing at all.
         -- `jsonb::text` is canonical (keys sorted, whitespace fixed), so this is
         -- stable for a given row.
         'sha256:' || encode(sha256(convert_to(p_normalized_row::text, 'UTF8')), 'hex'),
         'valid', p_normalized_row, NULL,
         CASE WHEN p_native_command IS NULL THEN p_created_by END,
         CASE WHEN p_native_command IS NULL THEN p_created_by END,event_time,event_time,
         CASE WHEN p_native_command IS NULL THEN 'LEGACY_USER' ELSE 'NATIVE_ACCOUNT' END,
         CASE WHEN p_native_command IS NOT NULL THEN p_created_by END,
         CASE WHEN p_native_command IS NOT NULL THEN p_created_by END,
         CASE WHEN p_native_command IS NOT NULL THEN p_created_by END,p_native_command,native_receipt)
    RETURNING id INTO v_policy_id;

    INSERT INTO public.ont_object_policies
        (org_id,object_type_id,cedar_policy_id,effect,created_by,created_at,attribution_protocol,
         created_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
    VALUES
        (p_org_id,p_object_type_id,v_policy_id,p_effect,CASE WHEN p_native_command IS NULL THEN p_created_by END,
         event_time,CASE WHEN p_native_command IS NULL THEN 'LEGACY_USER' ELSE 'NATIVE_ACCOUNT' END,
         CASE WHEN p_native_command IS NOT NULL THEN p_created_by END,
         CASE WHEN p_native_command IS NOT NULL THEN p_created_by END,p_native_command,native_receipt);

    RETURN v_policy_id;
END;
$$;
CREATE OR REPLACE FUNCTION ont_policy_api.attach_object_policy_rows(
 p_org_id uuid,p_created_by uuid,p_object_type_id uuid,p_effect text,p_normalized_row jsonb,p_schema_version text)
RETURNS uuid LANGUAGE sql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
 SELECT ont_policy_api.attach_object_policy_rows_core_v1($1,$2,$3,$4,$5,$6,NULL);
$body$;


-- source: catalog-children.sql
CREATE OR REPLACE FUNCTION ontology_api.insert_children(
    p_org_id UUID,
    p_object_type_id UUID,
    p_snapshot JSONB,
    p_allow_existing BOOLEAN
)
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_item JSONB;
    v_existing JSONB;
    v_key TEXT;
    v_created_at TIMESTAMPTZ := pg_catalog.transaction_timestamp();
    v_parent public.ont_object_types%ROWTYPE;
    v_binding RECORD;
BEGIN
    SELECT x.* INTO v_parent FROM public.ont_object_types x
     WHERE x.org_id=p_org_id AND x.id=p_object_type_id;
    IF v_parent.attribution_protocol='NATIVE_ACCOUNT' THEN
        SELECT * INTO STRICT v_binding FROM public.company_enrollment_binding_v1(
            v_parent.origin_account_id,v_parent.origin_command_id);
        IF v_binding.request_state<>'PENDING'
          OR v_binding.org_id IS DISTINCT FROM p_org_id
          OR v_parent.origin_receipt_id IS DISTINCT FROM v_binding.receipt_id
          OR v_parent.created_by_account_id IS DISTINCT FROM v_binding.account_id
          OR v_parent.schema_version<>1 OR v_parent.lifecycle_state<>'published'
          OR v_parent.created_at IS DISTINCT FROM v_binding.started_at
          OR v_parent.updated_at IS DISTINCT FROM v_binding.started_at THEN
            RAISE EXCEPTION 'identity_native.catalog_invalid_birth';
        END IF;
        v_created_at:=v_binding.started_at;
    END IF;
    IF pg_catalog.jsonb_typeof(p_snapshot) IS DISTINCT FROM 'object'
       OR pg_catalog.jsonb_typeof(p_snapshot->'properties') IS DISTINCT FROM 'array'
       OR pg_catalog.jsonb_typeof(p_snapshot->'links') IS DISTINCT FROM 'array'
       OR pg_catalog.jsonb_typeof(p_snapshot->'actions') IS DISTINCT FROM 'array'
       OR pg_catalog.jsonb_typeof(p_snapshot->'analytics') IS DISTINCT FROM 'array' THEN
        RAISE EXCEPTION USING ERRCODE = '22023', MESSAGE = 'ontology_write.invalid_snapshot_shape';
    END IF;

    -- A staged draft is append-only for child identities. Its submitted full
    -- snapshot must retain every existing child byte-semantically after the
    -- same trim/default canonicalization used for incoming rows.
    IF p_allow_existing AND EXISTS (
        SELECT 1 FROM public.ont_property_defs d
        WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'properties', '[]'::JSONB)) i
              WHERE pg_catalog.btrim(i->>'key') = d.key
                AND pg_catalog.jsonb_build_object(
                    'key', pg_catalog.btrim(i->>'key'),
                    'title', pg_catalog.btrim(i->>'title'),
                    'field_type', pg_catalog.btrim(i->>'field_type'),
                    'config', COALESCE(i->'config', '{}'::JSONB),
                    'backing_column', i->'backing_column',
                    'required', COALESCE((i->>'required')::BOOLEAN, FALSE),
                    'in_property_policy', COALESCE((i->>'in_property_policy')::BOOLEAN, FALSE)
                ) = pg_catalog.jsonb_build_object(
                    'key', d.key, 'title', d.title, 'field_type', d.type,
                    'config', d.config, 'backing_column', pg_catalog.to_jsonb(d.backing_column),
                    'required', d.required, 'in_property_policy', d.in_property_policy
                )
          )
    ) THEN
        RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.property_snapshot_conflict';
    END IF;

    IF p_allow_existing AND EXISTS (
        SELECT 1 FROM public.ont_link_types d
        WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'links', '[]'::JSONB)) i
              WHERE pg_catalog.btrim(i->>'stable_key') = d.stable_key
                AND pg_catalog.jsonb_build_object(
                    'stable_key', pg_catalog.btrim(i->>'stable_key'),
                    'title', pg_catalog.btrim(i->>'title'),
                    'reverse_title', CASE WHEN i->>'reverse_title' IS NULL THEN 'null'::JSONB ELSE pg_catalog.to_jsonb(pg_catalog.btrim(i->>'reverse_title')) END,
                    'to_object_type_id', i->'to_object_type_id',
                    'cardinality', i->'cardinality',
                    'traversable', COALESCE((i->>'traversable')::BOOLEAN, TRUE)
                ) = pg_catalog.jsonb_build_object(
                    'stable_key', d.stable_key, 'title', d.title,
                    'reverse_title', pg_catalog.to_jsonb(d.reverse_title),
                    'to_object_type_id', pg_catalog.to_jsonb(d.to_object_type_id),
                    'cardinality', d.cardinality, 'traversable', d.traversable
                )
          )
    ) THEN
        RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.link_snapshot_conflict';
    END IF;

    IF p_allow_existing AND EXISTS (
        SELECT 1 FROM public.ont_action_types d
        WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'actions', '[]'::JSONB)) i
              WHERE pg_catalog.btrim(i->>'stable_key') = d.stable_key
                AND pg_catalog.jsonb_build_object(
                    'stable_key', pg_catalog.btrim(i->>'stable_key'),
                    'title', pg_catalog.btrim(i->>'title'),
                    'params_schema', COALESCE(i->'params_schema', '{}'::JSONB),
                    'edits', COALESCE(i->'edits', '[]'::JSONB),
                    'submission_criteria', COALESCE(i->'submission_criteria', '[]'::JSONB),
                    'side_effects', COALESCE(i->'side_effects', '[]'::JSONB),
                    'dispatch', i->'dispatch',
                    'dispatch_target', i->'dispatch_target',
                    'control_points', COALESCE(i->'control_points', '[]'::JSONB)
                ) = pg_catalog.jsonb_build_object(
                    'stable_key', d.stable_key, 'title', d.title,
                    'params_schema', d.params_schema, 'edits', d.edits,
                    'submission_criteria', d.submission_criteria,
                    'side_effects', d.side_effects, 'dispatch', d.dispatch,
                    'dispatch_target', pg_catalog.to_jsonb(d.dispatch_target),
                    'control_points', d.control_points
                )
          )
    ) THEN
        RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.action_snapshot_conflict';
    END IF;

    IF p_allow_existing AND EXISTS (
        SELECT 1 FROM public.ont_analytics d
        WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id
          AND NOT EXISTS (
              SELECT 1 FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'analytics', '[]'::JSONB)) i
              WHERE pg_catalog.btrim(i->>'key') = d.key
                AND pg_catalog.jsonb_build_object(
                    'key', pg_catalog.btrim(i->>'key'),
                    'title', pg_catalog.btrim(i->>'title'),
                    'formula', COALESCE(i->'formula', '{}'::JSONB),
                    'result_type', COALESCE(i->'result_type', '{}'::JSONB)
                ) = pg_catalog.jsonb_build_object(
                    'key', d.key, 'title', d.title,
                    'formula', d.formula, 'result_type', d.result_type
                )
          )
    ) THEN
        RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.analytic_snapshot_conflict';
    END IF;

    FOR v_item IN SELECT value FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'properties', '[]'::JSONB)) LOOP
        v_key := pg_catalog.btrim(v_item->>'key');
        SELECT pg_catalog.jsonb_build_object(
                   'key', d.key, 'title', d.title, 'field_type', d.type,
                   'config', d.config, 'backing_column', pg_catalog.to_jsonb(d.backing_column),
                   'required', d.required, 'in_property_policy', d.in_property_policy)
          INTO v_existing
          FROM public.ont_property_defs d
         WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id AND d.key = v_key;
        IF FOUND THEN
            IF NOT p_allow_existing OR v_existing <> pg_catalog.jsonb_build_object(
                'key', v_key, 'title', pg_catalog.btrim(v_item->>'title'),
                'field_type', pg_catalog.btrim(v_item->>'field_type'),
                'config', COALESCE(v_item->'config', '{}'::JSONB),
                'backing_column', v_item->'backing_column',
                'required', COALESCE((v_item->>'required')::BOOLEAN, FALSE),
                'in_property_policy', COALESCE((v_item->>'in_property_policy')::BOOLEAN, FALSE)) THEN
                RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.property_key_conflict';
            END IF;
        ELSE
            INSERT INTO public.ont_property_defs
                (id, org_id, object_type_id, key, title, type, config, backing_column, required, in_property_policy, created_at)
            VALUES
                (public.gen_random_uuid(), p_org_id, p_object_type_id, v_key,
                 pg_catalog.btrim(v_item->>'title'), pg_catalog.btrim(v_item->>'field_type'),
                 COALESCE(v_item->'config', '{}'::JSONB), NULLIF(v_item->>'backing_column', ''),
                 COALESCE((v_item->>'required')::BOOLEAN, FALSE),
                 COALESCE((v_item->>'in_property_policy')::BOOLEAN, FALSE), v_created_at);
        END IF;
    END LOOP;

    FOR v_item IN SELECT value FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'links', '[]'::JSONB)) LOOP
        v_key := pg_catalog.btrim(v_item->>'stable_key');
        SELECT pg_catalog.jsonb_build_object(
                   'stable_key', d.stable_key, 'title', d.title,
                   'reverse_title', pg_catalog.to_jsonb(d.reverse_title),
                   'to_object_type_id', pg_catalog.to_jsonb(d.to_object_type_id),
                   'cardinality', d.cardinality, 'traversable', d.traversable)
          INTO v_existing
          FROM public.ont_link_types d
         WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id AND d.stable_key = v_key;
        IF FOUND THEN
            IF NOT p_allow_existing OR v_existing <> pg_catalog.jsonb_build_object(
                'stable_key', v_key, 'title', pg_catalog.btrim(v_item->>'title'),
                'reverse_title', CASE WHEN v_item->>'reverse_title' IS NULL THEN 'null'::JSONB ELSE pg_catalog.to_jsonb(pg_catalog.btrim(v_item->>'reverse_title')) END,
                'to_object_type_id', v_item->'to_object_type_id', 'cardinality', v_item->'cardinality',
                'traversable', COALESCE((v_item->>'traversable')::BOOLEAN, TRUE)) THEN
                RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.link_key_conflict';
            END IF;
        ELSE
            INSERT INTO public.ont_link_types
                (id, org_id, object_type_id, stable_key, title, reverse_title, to_object_type_id, cardinality, traversable, created_at)
            VALUES
                (public.gen_random_uuid(), p_org_id, p_object_type_id, v_key,
                 pg_catalog.btrim(v_item->>'title'), NULLIF(pg_catalog.btrim(v_item->>'reverse_title'), ''),
                 NULLIF(v_item->>'to_object_type_id', '')::UUID, v_item->>'cardinality',
                 COALESCE((v_item->>'traversable')::BOOLEAN, TRUE), v_created_at);
        END IF;
    END LOOP;

    FOR v_item IN SELECT value FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'actions', '[]'::JSONB)) LOOP
        v_key := pg_catalog.btrim(v_item->>'stable_key');
        SELECT pg_catalog.jsonb_build_object(
                   'stable_key', d.stable_key, 'title', d.title,
                   'params_schema', d.params_schema, 'edits', d.edits,
                   'submission_criteria', d.submission_criteria, 'side_effects', d.side_effects,
                   'dispatch', d.dispatch, 'dispatch_target', pg_catalog.to_jsonb(d.dispatch_target),
                   'control_points', d.control_points)
          INTO v_existing
          FROM public.ont_action_types d
         WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id AND d.stable_key = v_key;
        IF FOUND THEN
            IF NOT p_allow_existing OR v_existing <> pg_catalog.jsonb_build_object(
                'stable_key', v_key, 'title', pg_catalog.btrim(v_item->>'title'),
                'params_schema', COALESCE(v_item->'params_schema', '{}'::JSONB),
                'edits', COALESCE(v_item->'edits', '[]'::JSONB),
                'submission_criteria', COALESCE(v_item->'submission_criteria', '[]'::JSONB),
                'side_effects', COALESCE(v_item->'side_effects', '[]'::JSONB),
                'dispatch', v_item->'dispatch', 'dispatch_target', v_item->'dispatch_target',
                'control_points', COALESCE(v_item->'control_points', '[]'::JSONB)) THEN
                RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.action_key_conflict';
            END IF;
        ELSE
            INSERT INTO public.ont_action_types
                (id, org_id, object_type_id, stable_key, title, params_schema, edits,
                 submission_criteria, side_effects, dispatch, dispatch_target, control_points, created_at)
            VALUES
                (public.gen_random_uuid(), p_org_id, p_object_type_id, v_key,
                 pg_catalog.btrim(v_item->>'title'), COALESCE(v_item->'params_schema', '{}'::JSONB),
                 COALESCE(v_item->'edits', '[]'::JSONB), COALESCE(v_item->'submission_criteria', '[]'::JSONB),
                 COALESCE(v_item->'side_effects', '[]'::JSONB), v_item->>'dispatch',
                 NULLIF(v_item->>'dispatch_target', ''), COALESCE(v_item->'control_points', '[]'::JSONB), v_created_at);
        END IF;
    END LOOP;

    FOR v_item IN SELECT value FROM pg_catalog.jsonb_array_elements(COALESCE(p_snapshot->'analytics', '[]'::JSONB)) LOOP
        v_key := pg_catalog.btrim(v_item->>'key');
        SELECT pg_catalog.jsonb_build_object('key', d.key, 'title', d.title, 'formula', d.formula, 'result_type', d.result_type)
          INTO v_existing
          FROM public.ont_analytics d
         WHERE d.org_id = p_org_id AND d.object_type_id = p_object_type_id AND d.key = v_key;
        IF FOUND THEN
            IF NOT p_allow_existing OR v_existing <> pg_catalog.jsonb_build_object(
                'key', v_key, 'title', pg_catalog.btrim(v_item->>'title'),
                'formula', COALESCE(v_item->'formula', '{}'::JSONB),
                'result_type', COALESCE(v_item->'result_type', '{}'::JSONB)) THEN
                RAISE EXCEPTION USING ERRCODE = '23505', MESSAGE = 'ontology_write.analytic_key_conflict';
            END IF;
        ELSE
            INSERT INTO public.ont_analytics
                (id, org_id, object_type_id, key, title, formula, result_type, created_at)
            VALUES
                (public.gen_random_uuid(), p_org_id, p_object_type_id, v_key,
                 pg_catalog.btrim(v_item->>'title'), COALESCE(v_item->'formula', '{}'::JSONB),
                 COALESCE(v_item->'result_type', '{}'::JSONB), v_created_at);
        END IF;
    END LOOP;
END;
$$;


-- source: catalog-legacy-prepare.sql
CREATE OR REPLACE FUNCTION ontology_api.prepare_legacy_object_type_write()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
BEGIN
    -- The earlier native attribution guard proves the exact pending binding.
    -- Native core already wrote its key before the parent; the legacy fallback
    -- must never manufacture or mutate a key for that native birth.
    IF NEW.attribution_protocol='NATIVE_ACCOUNT' THEN
        IF TG_OP<>'INSERT' THEN
            RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_operation_unavailable';
        END IF;
        RETURN NEW;
    END IF;
    IF ontology_api.invoker_role() <> 'console_rt'::NAME THEN
        RETURN NEW;
    END IF;

    PERFORM pg_catalog.pg_advisory_xact_lock(
        pg_catalog.hashtextextended('ontology-bootstrap:' || NEW.org_id::TEXT, 0)
    );
    IF TG_OP = 'INSERT' THEN
        INSERT INTO public.ont_object_type_key_revisions (org_id, stable_key)
        VALUES (NEW.org_id, NEW.stable_key)
        ON CONFLICT (org_id, stable_key) DO NOTHING;
    ELSIF ROW(
        NEW.id, NEW.org_id, NEW.stable_key, NEW.title, NEW.title_property_key,
        NEW.backing_kind, NEW.backing_table, NEW.primary_key_property,
        NEW.schema_version, NEW.created_by, NEW.created_at
    ) IS DISTINCT FROM ROW(
        OLD.id, OLD.org_id, OLD.stable_key, OLD.title, OLD.title_property_key,
        OLD.backing_kind, OLD.backing_table, OLD.primary_key_property,
        OLD.schema_version, OLD.created_by, OLD.created_at
    ) OR NEW.lifecycle_state IS NOT DISTINCT FROM OLD.lifecycle_state
      OR NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at THEN
        RAISE EXCEPTION USING
            ERRCODE = '42501',
            MESSAGE = 'ontology_legacy.lifecycle_update_only';
    END IF;
    RETURN NEW;
END;
$$;

-- source: catalog-native.sql
CREATE FUNCTION ontology_api.install_native_company_catalog_v1(p_account uuid,p_command uuid,p_trace text,p_span text)
RETURNS TABLE(catalog_version text,manifest_digest bytea,action_refs jsonb,property_refs jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; actual record; prior_org text:=current_setting('app.current_org',true);
 manifest constant jsonb:=$manifest${"catalog_version":"native-company-identity-2026-09-19.1","object_types":[{"stable_key":"company_workspace","title":"회사 업무 공간","title_property_key":"name","backing_kind":"projected","backing_table":"organizations","primary_key_property":"id","properties":[{"key":"name","title":"회사 이름","field_type":"text","config":{"utf8_bytes_min":1,"utf8_bytes_max":256,"nonblank":true,"controls_allowed":false},"backing_column":"name","required":true,"in_property_policy":true},{"key":"slug","title":"업무 공간 식별자","field_type":"text","config":{"ascii_pattern":"^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$","bytes_max":63},"backing_column":"slug","required":true,"in_property_policy":true}],"links":[],"actions":[{"stable_key":"context_discover","title":"회사 찾기","params_schema":{"type":"object","additionalProperties":false,"properties":{},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"context.discover","control_points":["authority"]},{"stable_key":"identity_read","title":"회사 정보 보기","params_schema":{"type":"object","additionalProperties":false,"properties":{},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.identity.read","control_points":["authority"]}],"analytics":[]},{"stable_key":"company_policy_assignment","title":"회사 권한 부여","title_property_key":null,"backing_kind":"projected","backing_table":"user_role_assignments","primary_key_property":"id","properties":[{"key":"account_id","title":"계정","field_type":"reference","config":{},"backing_column":"account_id","required":true,"in_property_policy":true},{"key":"scope","title":"회사 범위","field_type":"json","config":{"schema":{"type":"object","additionalProperties":false,"properties":{"kind":{"const":"COMPANY"},"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}}},"required":["kind","org_id"]}},"backing_column":"scope","required":true,"in_property_policy":true},{"key":"actions","title":"작업 권한","field_type":"json","config":{"schema":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"action_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"registration_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"manifest_digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["org_id","action_type_id","object_type_id","registration_revision","manifest_digest"]},"uniqueItems":true,"maxItems":5}},"backing_column":"actions","required":true,"in_property_policy":true},{"key":"fields","title":"열람 항목","field_type":"json","config":{"schema":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"property_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"schema_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"required":["org_id","object_type_id","property_id","schema_revision"]},"uniqueItems":true,"maxItems":10}},"backing_column":"fields","required":true,"in_property_policy":true},{"key":"valid_from","title":"시작 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC"},"backing_column":"valid_from","required":true,"in_property_policy":true},{"key":"valid_until","title":"종료 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC","nullable":true},"backing_column":"valid_until","required":false,"in_property_policy":true},{"key":"state","title":"상태","field_type":"choice","config":{"choices":[{"id":"ACTIVE","name":"ACTIVE"},{"id":"REVOKED","name":"REVOKED"}]},"backing_column":"state","required":true,"in_property_policy":true},{"key":"revision","title":"버전","field_type":"text","config":{"schema":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"backing_column":"revision","required":true,"in_property_policy":true}],"links":[],"actions":[{"stable_key":"policy_read","title":"권한 보기","params_schema":{"type":"object","additionalProperties":false,"properties":{},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.policy.read","control_points":["authority"]},{"stable_key":"policy_assign","title":"권한 부여","params_schema":{"type":"object","additionalProperties":false,"properties":{"command_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"expected_company_epoch":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"reason":{"type":"string","minLength":1,"maxLength":512},"administrative_account_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"parent_assignment_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"parent_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"valid_from":{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},"valid_until":{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},"clauses":{"type":"array","minItems":1,"maxItems":2,"uniqueItems":true,"items":{"type":"object","additionalProperties":false,"properties":{"kind":{"const":"COMPANY_CAPABILITY_CLAUSE_V1"},"action":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"action_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"registration_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"manifest_digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["org_id","action_type_id","object_type_id","registration_revision","manifest_digest"]},"resource":{"type":"object","additionalProperties":false,"properties":{"kind":{"const":"COMPANY"},"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}}},"required":["kind","org_id"]},"fields":{"type":"array","maxItems":8,"uniqueItems":true,"items":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"property_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"schema_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"required":["org_id","object_type_id","property_id","schema_revision"]}},"valid_from":{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},"valid_until":{"anyOf":[{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},{"type":"null"}]},"delegable":{"type":"boolean"}},"required":["kind","action","resource","fields","valid_from","valid_until","delegable"]}}},"required":["command_id","expected_company_epoch","reason","administrative_account_id","parent_assignment_id","parent_revision","valid_from","valid_until","clauses"]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.policy.assign","control_points":["authority"]},{"stable_key":"policy_revoke","title":"권한 회수","params_schema":{"type":"object","additionalProperties":false,"properties":{"command_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"expected_company_epoch":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"reason":{"type":"string","minLength":1,"maxLength":512},"assignment_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"expected_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"required":["command_id","expected_company_epoch","reason","assignment_id","expected_revision"]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.policy.revoke","control_points":["authority"]}],"analytics":[]}]}$manifest$::jsonb;
BEGIN
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 IF b.request_state<>'PENDING' THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT * INTO STRICT actual FROM ontology_api.install_builtin_catalog_core_v1(
  b.org_id,b.catalog_version,manifest,b.account_id,p_trace,p_span,b.command_id);
 IF actual.installed IS DISTINCT FROM true OR actual.object_type_count IS DISTINCT FROM 2 THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 INSERT INTO public.native_company_catalog_installs(org_id,catalog_version,manifest_digest,
  origin_account_id,origin_command_id,origin_receipt_id,installed_at)
 VALUES(b.org_id,b.catalog_version,b.manifest_digest,b.account_id,b.command_id,b.receipt_id,b.started_at);
 INSERT INTO public.native_company_object_refs(org_id,catalog_version,manifest_digest,object_key,
  object_type_id,schema_revision,content_digest)
 SELECT b.org_id,b.catalog_version,b.manifest_digest,o.stable_key,o.id,o.schema_version,
  sha256(convert_to(jsonb_build_object('stable_key',o.stable_key,'title',o.title,
   'title_property_key',o.title_property_key,'backing_kind',o.backing_kind,'backing_table',o.backing_table,
   'primary_key_property',o.primary_key_property,'schema_version',o.schema_version,'lifecycle_state',o.lifecycle_state)::text,'UTF8'))
 FROM public.ont_object_types o WHERE o.org_id=b.org_id;
 INSERT INTO public.native_company_action_refs(org_id,catalog_version,action_key,object_type_id,
  action_type_id,registration_revision,manifest_digest,content_digest)
 SELECT b.org_id,b.catalog_version,a.dispatch_target,a.object_type_id,a.id,1,b.manifest_digest,
  sha256(convert_to(jsonb_build_object('stable_key',a.stable_key,'title',a.title,'params_schema',a.params_schema,
   'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,'dispatch',a.dispatch,
   'dispatch_target',a.dispatch_target,'control_points',a.control_points)::text,'UTF8'))
 FROM public.ont_action_types a WHERE a.org_id=b.org_id;
 INSERT INTO public.native_company_property_refs(org_id,catalog_version,property_key,object_type_id,
  property_id,schema_revision,manifest_digest,content_digest)
 SELECT b.org_id,b.catalog_version,CASE WHEN o.stable_key='company_workspace' THEN 'company.' ELSE 'assignment.' END||p.key,
  p.object_type_id,p.id,1,b.manifest_digest,
  sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,'config',p.config,
   'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8'))
 FROM public.ont_property_defs p JOIN public.ont_object_types o ON o.org_id=p.org_id AND o.id=p.object_type_id
 WHERE p.org_id=b.org_id;
 RETURN QUERY SELECT b.catalog_version,b.manifest_digest,
  (SELECT jsonb_agg(jsonb_build_object('org_id',a.org_id::text,'object_type_id',a.object_type_id::text,
    'action_type_id',a.action_type_id::text,'registration_revision',a.registration_revision::text,
    'manifest_digest',encode(a.manifest_digest,'hex')) ORDER BY a.org_id,a.object_type_id,a.action_type_id,a.registration_revision,a.manifest_digest)
   FROM public.native_company_action_refs a WHERE a.org_id=b.org_id AND a.catalog_version=b.catalog_version),
  (SELECT jsonb_agg(jsonb_build_object('org_id',p.org_id::text,'object_type_id',p.object_type_id::text,
    'property_id',p.property_id::text,'schema_revision',p.schema_revision::text)
    ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision)
   FROM public.native_company_property_refs p WHERE p.org_id=b.org_id AND p.catalog_version=b.catalog_version);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION ont_policy_api.install_native_company_policy_v1(p_account uuid,p_command uuid,p_trace text,p_span text)
RETURNS TABLE(policy_id uuid,object_type_id uuid,bundle_digest bytea)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; o record; normalized jsonb; inserted uuid;
 prior_org text:=current_setting('app.current_org',true); count_inserted integer:=0;
BEGIN
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 IF b.request_state<>'PENDING' THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 PERFORM set_config('app.current_org',b.org_id::text,true);
 IF EXISTS(SELECT 1 FROM public.ont_object_policies p WHERE p.org_id=b.org_id) THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 FOR o IN SELECT r.object_type_id,r.object_key FROM public.native_company_object_refs r
  WHERE r.org_id=b.org_id AND r.catalog_version=b.catalog_version ORDER BY r.object_key COLLATE "C"
 LOOP
  normalized:=jsonb_build_object('effect','forbid','action','view','resource_type',o.object_key,'conditions','[]'::jsonb);
  inserted:=ont_policy_api.attach_object_policy_rows_core_v1(b.org_id,b.account_id,o.object_type_id,
   'forbid',normalized,'2026-07-ontology-authoring-v1',b.command_id);
  PERFORM public.company_enrollment_ontology_audit_v1(b.account_id,b.command_id,
   'ontology.object_policy.attach',o.object_type_id,p_trace,p_span);
  count_inserted:=count_inserted+1;
  RETURN QUERY SELECT inserted,o.object_type_id,sha256(convert_to(normalized::text,'UTF8'));
 END LOOP;
 IF count_inserted<>2 THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: catalog-map-guards.sql
CREATE FUNCTION public.native_company_catalog_birth_row_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE installation public.native_company_catalog_installs%ROWTYPE; b record; value jsonb:=to_jsonb(NEW); actual_digest bytea; actual_key text; parent public.ont_object_types%ROWTYPE;
BEGIN
 IF current_user<>'console_ontology_writer' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_owner_required';
 END IF;
 IF TG_TABLE_NAME='native_company_catalog_installs' THEN
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.origin_account_id,NEW.origin_command_id);
  IF NEW.origin_receipt_id IS DISTINCT FROM b.receipt_id OR NEW.installed_at IS DISTINCT FROM b.started_at THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
 ELSE
  SELECT x.* INTO STRICT installation FROM public.native_company_catalog_installs x
   WHERE x.org_id=NEW.org_id AND x.catalog_version=NEW.catalog_version;
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(installation.origin_account_id,installation.origin_command_id);
  IF installation.origin_receipt_id IS DISTINCT FROM b.receipt_id OR installation.installed_at IS DISTINCT FROM b.started_at THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
 END IF;
 IF b.request_state IS DISTINCT FROM 'PENDING' OR NEW.org_id IS DISTINCT FROM b.org_id
  OR NEW.catalog_version IS DISTINCT FROM b.catalog_version OR NEW.manifest_digest IS DISTINCT FROM b.manifest_digest
  OR (value?'schema_revision' AND (value->>'schema_revision')::bigint IS DISTINCT FROM 1)
  OR (value?'registration_revision' AND (value->>'registration_revision')::bigint IS DISTINCT FROM 1) THEN
  RAISE EXCEPTION 'identity_native.invalid_birth';
 END IF;
 IF TG_TABLE_NAME<>'native_company_catalog_installs' THEN
  SELECT o.* INTO STRICT parent FROM public.ont_object_types o WHERE o.org_id=NEW.org_id AND o.id=NEW.object_type_id;
  IF parent.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR parent.schema_version IS DISTINCT FROM 1
   OR parent.lifecycle_state IS DISTINCT FROM 'published' OR parent.origin_account_id IS DISTINCT FROM b.account_id
   OR parent.origin_command_id IS DISTINCT FROM b.command_id OR parent.origin_receipt_id IS DISTINCT FROM b.receipt_id THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
  IF TG_TABLE_NAME='native_company_object_refs' THEN
   actual_key:=parent.stable_key;
   actual_digest:=sha256(convert_to(jsonb_build_object('stable_key',parent.stable_key,'title',parent.title,
    'title_property_key',parent.title_property_key,'backing_kind',parent.backing_kind,'backing_table',parent.backing_table,
    'primary_key_property',parent.primary_key_property,'schema_version',parent.schema_version,'lifecycle_state',parent.lifecycle_state)::text,'UTF8'));
  ELSIF TG_TABLE_NAME='native_company_action_refs' THEN
   SELECT a.dispatch_target,sha256(convert_to(jsonb_build_object('stable_key',a.stable_key,'title',a.title,
    'params_schema',a.params_schema,'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,
    'dispatch',a.dispatch,'dispatch_target',a.dispatch_target,'control_points',a.control_points)::text,'UTF8'))
   INTO STRICT actual_key,actual_digest FROM public.ont_action_types a
    WHERE a.org_id=NEW.org_id AND a.object_type_id=NEW.object_type_id AND a.id=NEW.action_type_id;
  ELSE
   SELECT CASE WHEN parent.stable_key='company_workspace' THEN 'company.' ELSE 'assignment.' END||p.key,
    sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,'config',p.config,
     'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8'))
   INTO STRICT actual_key,actual_digest FROM public.ont_property_defs p
    WHERE p.org_id=NEW.org_id AND p.object_type_id=NEW.object_type_id AND p.id=NEW.property_id;
  END IF;
  IF NEW.content_digest IS DISTINCT FROM actual_digest
   OR coalesce(value->>'object_key',value->>'action_key',value->>'property_key') IS DISTINCT FROM actual_key THEN
   RAISE EXCEPTION 'identity_native.invalid_birth';
  END IF;
 END IF;
 -- The complete fixed-manifest comparison and cardinality proof is deferred
 -- until all maps exist; no label is accepted in place of a physical identity.
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'identity_native.invalid_birth';
END
$body$;

CREATE FUNCTION public.native_company_catalog_immutable_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
END
$body$;

CREATE FUNCTION public.native_company_catalog_birth_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); installation public.native_company_catalog_installs%ROWTYPE;
BEGIN
 PERFORM set_config('app.current_org',NEW.org_id::text,true); SELECT x.* INTO STRICT installation FROM public.native_company_catalog_installs x
  WHERE x.org_id=NEW.org_id AND x.catalog_version=NEW.catalog_version;
 PERFORM public.company_enrollment_assert_closure_v1(installation.origin_account_id,installation.origin_command_id);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: catalog-attribution-guard-round2.sql
CREATE FUNCTION ontology_api.native_catalog_attribution_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE value jsonb; previous jsonb; organization public.organizations%ROWTYPE;
 parent public.ont_object_types%ROWTYPE; b record; previous_binding record; native boolean:=false; previous_native boolean:=false;
BEGIN
 IF TG_OP<>'INSERT' THEN previous:=to_jsonb(OLD); END IF;
 IF TG_OP='DELETE' THEN value:=previous; ELSE value:=to_jsonb(NEW); END IF;
 IF current_user='console_ontology_writer' THEN
  SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1((value->>'org_id')::uuid);
  native:=b.account_id IS NOT NULL;
  IF TG_OP<>'INSERT' THEN
   SELECT * INTO previous_binding FROM public.company_enrollment_catalog_binding_v1((previous->>'org_id')::uuid);
   previous_native:=previous_binding.account_id IS NOT NULL;
  END IF;
 ELSE
  -- Retained direct legacy writers already have scoped organizations SELECT.
  -- They cannot execute the private binding helper or produce native rows.
  SELECT o.* INTO STRICT organization FROM public.organizations o WHERE o.id=(value->>'org_id')::uuid;
  native:=organization.origin_account_id IS NOT NULL;
  IF TG_OP<>'INSERT' THEN
   SELECT EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=(previous->>'org_id')::uuid
    AND o.origin_account_id IS NOT NULL) INTO previous_native;
  END IF;
 END IF;
 IF TG_OP<>'INSERT' AND (
   previous->>'attribution_protocol'='NATIVE_ACCOUNT' OR previous_native
   OR (value?'attribution_protocol' AND (
    value->'attribution_protocol' IS DISTINCT FROM previous->'attribution_protocol'
    OR value->'origin_account_id' IS DISTINCT FROM previous->'origin_account_id'
    OR value->'origin_command_id' IS DISTINCT FROM previous->'origin_command_id'
    OR value->'origin_receipt_id' IS DISTINCT FROM previous->'origin_receipt_id'))) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_operation_unavailable';
 END IF;
 IF NOT native THEN
  IF value?'attribution_protocol' AND value->>'attribution_protocol' IS DISTINCT FROM 'LEGACY_USER' THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
  IF TG_OP='DELETE' THEN RETURN OLD; END IF;
  RETURN NEW;
 END IF;
 IF current_user<>'console_ontology_writer' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_owner_required';
 END IF;
 IF b.request_state IS DISTINCT FROM 'PENDING' OR b.org_id IS DISTINCT FROM (value->>'org_id')::uuid
  OR (value?'created_at' AND (value->>'created_at')::timestamptz IS DISTINCT FROM b.started_at)
  OR (value?'updated_at' AND (value->>'updated_at')::timestamptz IS DISTINCT FROM b.started_at) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
 END IF;
 IF TG_TABLE_NAME='ont_object_type_key_revisions' THEN
  IF NEW.stable_key NOT IN ('company_workspace','company_policy_assignment') OR NEW.revision<>1
   OR NEW.validator_id IS NULL OR NEW.validator_id='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
 ELSIF TG_TABLE_NAME IN ('ont_property_defs','ont_link_types','ont_action_types','ont_analytics') THEN
  SELECT x.* INTO STRICT parent FROM public.ont_object_types x WHERE x.org_id=NEW.org_id AND x.id=NEW.object_type_id;
  IF parent.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
   OR parent.origin_account_id IS DISTINCT FROM b.account_id OR parent.origin_command_id IS DISTINCT FROM b.command_id
   OR parent.origin_receipt_id IS DISTINCT FROM b.receipt_id OR parent.created_by_account_id IS DISTINCT FROM b.account_id
   OR parent.created_at IS DISTINCT FROM b.started_at OR parent.updated_at IS DISTINCT FROM b.started_at
   OR parent.schema_version IS DISTINCT FROM 1 OR parent.lifecycle_state IS DISTINCT FROM 'published'
   OR parent.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
 ELSE
  IF value->>'attribution_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT'
   OR (value->>'origin_account_id')::uuid IS DISTINCT FROM b.account_id
   OR (value->>'origin_command_id')::uuid IS DISTINCT FROM b.command_id
   OR (value->>'origin_receipt_id')::uuid IS DISTINCT FROM b.receipt_id
   OR (value?'created_by' AND value->'created_by' IS DISTINCT FROM 'null'::jsonb)
   OR (value?'updated_by' AND value->'updated_by' IS DISTINCT FROM 'null'::jsonb)
   OR (value?'installed_by' AND value->'installed_by' IS DISTINCT FROM 'null'::jsonb)
   OR (value?'created_by_account_id' AND (value->>'created_by_account_id')::uuid IS DISTINCT FROM b.account_id)
   OR (value?'updated_by_account_id' AND (value->>'updated_by_account_id')::uuid IS DISTINCT FROM b.account_id)
   OR (value?'installed_by_account_id' AND (value->>'installed_by_account_id')::uuid IS DISTINCT FROM b.account_id) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
  IF TG_TABLE_NAME='ont_object_types' THEN
   IF NEW.stable_key NOT IN ('company_workspace','company_policy_assignment') OR NEW.schema_version<>1
    OR NEW.lifecycle_state IS DISTINCT FROM 'published' THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
   END IF;
  ELSIF TG_TABLE_NAME='ont_builtin_catalog_installs' THEN
   IF NEW.catalog_version IS DISTINCT FROM b.catalog_version OR NEW.manifest_digest IS DISTINCT FROM b.manifest_digest
    OR NEW.installed_at IS DISTINCT FROM b.started_at
    OR (SELECT count(*) FROM public.ont_object_types o WHERE o.org_id=b.org_id)<>2 THEN
    RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
   END IF;
  ELSIF TG_TABLE_NAME NOT IN ('cedar_policy_catalog_entries','ont_object_policies') THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
  END IF;
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.catalog_invalid_birth';
END
$body$;


-- source: catalog-audit-bound-parser.sql
CREATE FUNCTION public.company_enrollment_ontology_audit_v1(p_account uuid,p_command uuid,
 p_action text,p_object uuid,p_trace text,p_span text) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; o record; snapshot jsonb; target text;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 IF b.request_state<>'PENDING' OR p_action IS NULL
  OR p_action NOT IN ('ontology.object_type.builtin_install','ontology.object_policy.attach')
  OR p_trace IS NULL OR p_trace !~ '^[0-9a-f]{32}$' OR p_trace=repeat('0',32)
  OR p_span IS NULL OR p_span !~ '^[0-9a-f]{16}$' OR p_span=repeat('0',16) THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 PERFORM set_config('app.current_org',b.org_id::text,true);
 IF NOT EXISTS(SELECT 1 FROM public.organizations x WHERE x.id=b.org_id
  AND x.origin_account_id=b.account_id AND x.origin_command_id=b.command_id AND x.origin_receipt_id=b.receipt_id) THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;

 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=p_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 IF p_action='ontology.object_type.builtin_install' THEN
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=p_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=p_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 ELSE
  target:='ont_object_policies';
  SELECT c.normalized_row INTO STRICT snapshot
   FROM public.ont_object_policies p JOIN public.cedar_policy_catalog_entries c ON c.org_id=p.org_id AND c.id=p.cedar_policy_id
   WHERE p.org_id=b.org_id AND p.object_type_id=p_object AND p.attribution_protocol='NATIVE_ACCOUNT'
    AND p.origin_account_id=b.account_id AND p.origin_command_id=b.command_id AND p.origin_receipt_id=b.receipt_id
    AND p.created_by_account_id=b.account_id AND p.created_at=b.started_at
    AND c.attribution_protocol='NATIVE_ACCOUNT' AND c.origin_account_id=b.account_id AND c.origin_command_id=b.command_id
    AND c.origin_receipt_id=b.receipt_id AND c.created_by_account_id=b.account_id AND c.updated_by_account_id=b.account_id
    AND c.created_at=b.started_at AND c.updated_at=b.started_at;
  IF snapshot IS DISTINCT FROM jsonb_build_object('effect','forbid','action','view','resource_type',o.stable_key,'conditions','[]'::jsonb) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 END IF;
 INSERT INTO public.audit_events(id,actor,action,target_type,target_id,before_snap,after_snap,trace_id,span_id,occurred_at,org_id)
 VALUES(gen_random_uuid(),b.account_id,p_action,target,p_object::text,NULL,
  snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
   'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)),
  p_trace,p_span,b.started_at,b.org_id);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.company_enrollment_ontology_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; native_target boolean;
BEGIN
 IF NEW.action NOT IN ('ontology.object_type.builtin_install','ontology.object_policy.attach') THEN RETURN NEW; END IF;
 SELECT EXISTS(SELECT 1 FROM public.ont_object_types o WHERE o.org_id=NEW.org_id
  AND o.id::text=NEW.target_id AND o.attribution_protocol='NATIVE_ACCOUNT') INTO native_target;
 IF NOT native_target AND NOT coalesce(NEW.after_snap?'enrollment',false) THEN RETURN NEW; END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
 END IF;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.actor,(NEW.after_snap->'enrollment'->>'command_id')::uuid);
 IF b.org_id IS DISTINCT FROM NEW.org_id OR b.request_state<>'PENDING' OR NOT native_target
  OR NEW.occurred_at IS DISTINCT FROM b.started_at OR NEW.before_snap IS NOT NULL
  OR NEW.after_snap->'enrollment' IS DISTINCT FROM jsonb_build_object('account_id',b.account_id::text,
   'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)
  OR NEW.target_type IS DISTINCT FROM (CASE WHEN NEW.action='ontology.object_type.builtin_install' THEN 'ont_object_types' ELSE 'ont_object_policies' END) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
 END IF;
 RETURN NEW;
END
$body$;


-- source: native-protected-audit-parser.sql
CREATE OR REPLACE FUNCTION ontology_api.protected_audit_writer_guard()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_invoker NAME := ontology_api.invoker_role();
    v_stable_key TEXT;
    b record; o record; snapshot jsonb; target text;
    native_object uuid; native_action text;
    prior_org text:=current_setting('app.current_org',true);
BEGIN
    IF NEW.action IN ('ontology.object_type.create','ontology.object_type.stage_revision',
        'ontology.object_type.transition','ontology.object_type.builtin_install','ontology.object_policy.attach') THEN
      SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
      IF b.account_id IS NOT NULL THEN
       BEGIN
        IF NEW.action NOT IN ('ontology.object_type.builtin_install','ontology.object_policy.attach')
          OR b.request_state IS DISTINCT FROM 'PENDING' OR b.org_id IS DISTINCT FROM NEW.org_id
          OR NEW.actor IS DISTINCT FROM b.account_id OR NEW.occurred_at IS DISTINCT FROM b.started_at
          OR NEW.before_snap IS NOT NULL OR NEW.branch_id IS NOT NULL
          OR NEW.trace_id IS NULL OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
          OR NEW.span_id IS NULL OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        native_object:=NEW.target_id::uuid; native_action:=NEW.action;
        PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=native_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 IF native_action='ontology.object_type.builtin_install' THEN
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 ELSE
  target:='ont_object_policies';
  SELECT c.normalized_row INTO STRICT snapshot
   FROM public.ont_object_policies p JOIN public.cedar_policy_catalog_entries c ON c.org_id=p.org_id AND c.id=p.cedar_policy_id
   WHERE p.org_id=b.org_id AND p.object_type_id=native_object AND p.attribution_protocol='NATIVE_ACCOUNT'
    AND p.origin_account_id=b.account_id AND p.origin_command_id=b.command_id AND p.origin_receipt_id=b.receipt_id
    AND p.created_by_account_id=b.account_id AND p.created_at=b.started_at
    AND c.attribution_protocol='NATIVE_ACCOUNT' AND c.origin_account_id=b.account_id AND c.origin_command_id=b.command_id
    AND c.origin_receipt_id=b.receipt_id AND c.created_by_account_id=b.account_id AND c.updated_by_account_id=b.account_id
    AND c.created_at=b.started_at AND c.updated_at=b.started_at;
  IF snapshot IS DISTINCT FROM jsonb_build_object('effect','forbid','action','view','resource_type',o.stable_key,'conditions','[]'::jsonb) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 END IF;
        IF NEW.target_type IS DISTINCT FROM target OR NEW.after_snap IS DISTINCT FROM
          snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
            'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)) THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
       EXCEPTION WHEN OTHERS THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
       END;
      ELSIF coalesce(NEW.after_snap?'enrollment',false) THEN
        RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
      END IF;
    END IF;
    IF NEW.action <> ALL (ARRAY[
        'ontology.object_type.create',
        'ontology.object_type.stage_revision',
        'ontology.object_type.transition',
        'ontology.object_type.builtin_install'
    ]::TEXT[]) THEN
        RETURN NEW;
    END IF;

    -- Direct command credentials cannot INSERT audit_events. When an approved
    -- command reaches this trigger it is nested inside the writer-owned
    -- SECURITY DEFINER routine, while the old compatibility path arrives as
    -- console_rt and must prove a matching parent mutation in this transaction.
    IF v_invoker = 'console_rt'::NAME THEN
        IF NEW.action = 'ontology.object_type.builtin_install'
           OR NEW.target_type <> 'ont_object_types'
           OR NOT EXISTS (
               SELECT 1
               FROM public.users u
               WHERE u.id = NEW.actor AND u.org_id = NEW.org_id AND u.is_active
           ) THEN
            RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_audit.command_required';
        END IF;

        SELECT o.stable_key INTO v_stable_key
        FROM public.ont_object_types o
        WHERE o.org_id = NEW.org_id
          AND o.id::TEXT = NEW.target_id
          AND o.updated_at = NEW.occurred_at
          AND o.xmin = pg_catalog.pg_current_xact_id()::xid
          AND (
              (NEW.action = 'ontology.object_type.create' AND o.schema_version = 1 AND o.created_at = NEW.occurred_at)
              OR (NEW.action = 'ontology.object_type.stage_revision' AND o.schema_version > 1 AND o.created_at = NEW.occurred_at)
              OR NEW.action = 'ontology.object_type.transition'
          );
        IF v_stable_key IS NULL THEN
            RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_audit.command_required';
        END IF;
        IF NEW.action IN ('ontology.object_type.stage_revision', 'ontology.object_type.transition') THEN
            UPDATE public.ont_object_type_key_revisions k
               SET revision = k.revision + 1, updated_at = NEW.occurred_at
             WHERE k.org_id = NEW.org_id AND k.stable_key = v_stable_key;
            IF NOT FOUND THEN
                RAISE EXCEPTION USING ERRCODE = '23503', MESSAGE = 'ontology_legacy.key_revision_missing';
            END IF;
        END IF;
    ELSIF v_invoker <> 'console_ontology_cmd'::NAME THEN
        RAISE EXCEPTION USING
            ERRCODE = '42501',
            MESSAGE = 'ontology_audit.command_required';
    END IF;
    RETURN NEW;
END;
$$;

-- source: native-deferred-audit-parser.sql
CREATE OR REPLACE FUNCTION ontology_api.require_current_transaction_audit()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_parent_id UUID;
    v_org_id UUID;
    v_stable_key TEXT;
    v_parent_is_current BOOLEAN;
    b record; o record; snapshot jsonb; target text;
    native_object uuid; native_action text;
    prior_org text:=current_setting('app.current_org',true);
BEGIN
    SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
    IF b.account_id IS NOT NULL THEN
      BEGIN
        IF b.request_state IS DISTINCT FROM 'COMMITTED' OR b.org_id IS DISTINCT FROM NEW.org_id THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        IF TG_TABLE_NAME='ont_object_types' THEN native_object:=NEW.id;
        ELSE native_object:=NEW.object_type_id; END IF;
        PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=native_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
        IF (SELECT count(*) FROM public.audit_events e WHERE e.org_id=b.org_id
          AND e.action='ontology.object_type.builtin_install' AND e.target_id=native_object::text
          AND e.target_type=target AND e.actor=b.account_id AND e.occurred_at=b.started_at
          AND e.before_snap IS NULL AND e.branch_id IS NULL
          AND e.after_snap=snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
            'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)))<>1 THEN
          RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='ontology_write.exactly_one_current_transaction_audit_required';
        END IF;
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
      EXCEPTION WHEN OTHERS THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
      END;
    END IF;
    IF TG_TABLE_NAME = 'ont_object_types' THEN
        v_parent_id := NEW.id;
        v_org_id := NEW.org_id;
        v_stable_key := NEW.stable_key;
        v_parent_is_current := TRUE;
    ELSE
        v_parent_id := NEW.object_type_id;
        v_org_id := NEW.org_id;
        SELECT o.stable_key,
               o.xmin = pg_catalog.pg_current_xact_id()::xid
          INTO v_stable_key, v_parent_is_current
          FROM public.ont_object_types o
         WHERE o.id = v_parent_id AND o.org_id = v_org_id;
    END IF;

    IF COALESCE(v_parent_is_current, FALSE) AND TG_TABLE_NAME <> 'ont_object_types' THEN
        RETURN NEW;
    END IF;
    IF (
        SELECT COUNT(*) = 1
        FROM public.audit_events e
        LEFT JOIN public.ont_object_types target
          ON target.org_id = e.org_id AND target.id::TEXT = e.target_id
        WHERE e.org_id = v_org_id
          AND e.action = ANY (ARRAY[
              'ontology.object_type.create',
              'ontology.object_type.stage_revision',
              'ontology.object_type.transition',
              'ontology.object_type.builtin_install'
          ]::TEXT[])
          AND e.xmin = pg_catalog.pg_current_xact_id()::xid
          AND (
              e.target_id = v_parent_id::TEXT
              OR (e.action = 'ontology.object_type.transition' AND target.stable_key = v_stable_key)
          )
    ) THEN
        RETURN NEW;
    END IF;
    RAISE EXCEPTION USING
        ERRCODE = '23514',
        MESSAGE = 'ontology_write.exactly_one_current_transaction_audit_required';
END;
$$;

-- source: enrollment-audit-guard.sql
CREATE FUNCTION public.company_enrollment_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record;
BEGIN
 IF NEW.action IS DISTINCT FROM 'company.enroll' THEN RETURN NEW; END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.audit_owner_required';
 END IF;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.actor,(NEW.after_snap->'enrollment'->>'command_id')::uuid);
 IF b.request_state IS DISTINCT FROM 'PENDING' OR NEW.org_id IS DISTINCT FROM b.org_id
  OR NEW.target_type IS DISTINCT FROM 'organizations' OR NEW.target_id IS DISTINCT FROM b.org_id::text
  OR NEW.occurred_at IS DISTINCT FROM b.started_at OR NEW.before_snap IS NOT NULL OR NEW.branch_id IS NOT NULL
  OR NEW.trace_id IS NULL OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
  OR NEW.span_id IS NULL OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16)
  OR NEW.after_snap IS DISTINCT FROM jsonb_build_object('org_id',b.org_id::text,'group_id',b.group_id::text,
    'administrative_account_id',b.administrative_account_id::text,
    'enrollment',jsonb_build_object('account_id',b.account_id::text,'command_id',b.command_id::text,
      'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.audit_owner_required';
 END IF;
 RETURN NEW;
END
$body$;


-- source: identity-owner.sql
CREATE FUNCTION public.identity_enroll_company_administration_v1(p_account uuid,p_command uuid)
RETURNS TABLE(root_assignment_id uuid,root_revision bigint)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; role uuid:=gen_random_uuid(); assignment uuid:=gen_random_uuid();
 action_ref record; property_ref record; clause jsonb; clauses jsonb:='[]'::jsonb;
 fields jsonb; clause_hash bytea; role_hash bytea; ordinal smallint; selected_action_key text;
 prior_org text:=current_setting('app.current_org',true); changed integer;
BEGIN
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 IF b.request_state<>'PENDING' THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 PERFORM set_config('app.current_org',b.org_id::text,true);
 FOR ordinal IN 1..7 LOOP
  selected_action_key := (ARRAY['context.discover','company.identity.read','company.policy.read',
   'company.policy.assign','company.policy.revoke','context.discover','company.identity.read'])[ordinal];
  SELECT a.* INTO STRICT action_ref FROM public.native_company_action_refs a
   WHERE a.org_id=b.org_id AND a.catalog_version=b.catalog_version AND a.action_key=selected_action_key;
  SELECT coalesce(jsonb_agg(jsonb_build_object('org_id',p.org_id::text,
    'object_type_id',p.object_type_id::text,'property_id',p.property_id::text,'schema_revision',p.schema_revision::text)
    ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision),'[]'::jsonb)
   INTO fields FROM public.native_company_property_refs p
   WHERE p.org_id=b.org_id AND p.catalog_version=b.catalog_version AND p.object_type_id=action_ref.object_type_id
    AND ordinal NOT IN (4,5);
  clause := jsonb_build_object('kind','COMPANY_CAPABILITY_CLAUSE_V1',
   'action',jsonb_build_object('org_id',b.org_id::text,'object_type_id',action_ref.object_type_id::text,
    'action_type_id',action_ref.action_type_id::text,'registration_revision',action_ref.registration_revision::text,
    'manifest_digest',encode(action_ref.manifest_digest,'hex')),
   'resource',jsonb_build_object('kind','COMPANY','org_id',b.org_id::text),
   'fields',fields,'valid_from',to_char(b.started_at AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
   'valid_until',NULL,'delegable',ordinal>5);
  clauses:=clauses||jsonb_build_array(clause);
 END LOOP;
 SELECT sha256(convert_to(jsonb_agg(jsonb_build_object('clause_index',n,
   'clause_digest',encode(sha256(convert_to(c::text,'UTF8')),'hex')) ORDER BY n)::text,'UTF8'))
  INTO role_hash FROM jsonb_array_elements(clauses) WITH ORDINALITY AS x(c,n);
 INSERT INTO public.company_actors(org_id,account_id,admission_receipt_id,entitlement_ref,created_at)
  VALUES(b.org_id,b.administrative_account_id,b.receipt_id,
   jsonb_build_object('kind','COMPANY_ENROLLMENT_V1','account_id',b.account_id::text,
    'command_id',b.command_id::text,'org_id',b.org_id::text,'receipt_id',b.receipt_id::text),b.started_at);
 INSERT INTO public.policy_roles(id,org_id,role_key,display_name,description,status,is_system,
  created_at,updated_at,subject_protocol,native_current_revision,created_by_account_id,updated_by_account_id,
  origin_account_id,origin_command_id,origin_receipt_id)
 VALUES(role,b.org_id,'native_company_administration','회사 초기 관리자',NULL,'ACTIVE',true,
  b.started_at,b.started_at,'NATIVE_ACCOUNT',1,b.account_id,b.account_id,b.account_id,b.command_id,b.receipt_id);
 INSERT INTO public.policy_role_revisions(org_id,role_id,revision,subject_protocol,state,valid_from,valid_until,
  catalog_version,manifest_digest,clause_digest,origin_account_id,origin_command_id,origin_receipt_id,
  actor_account_id,session_id,created_at)
 VALUES(b.org_id,role,1,'NATIVE_ACCOUNT','ACTIVE',b.started_at,NULL,b.catalog_version,b.manifest_digest,
  role_hash,b.account_id,b.command_id,b.receipt_id,b.account_id,b.session_id,b.started_at);
 FOR ordinal IN 1..7 LOOP
  clause:=clauses->(ordinal-1);
  clause_hash:=sha256(convert_to(clause::text,'UTF8'));
  INSERT INTO public.policy_capability_clauses(org_id,role_id,role_revision,clause_index,effect,
   action_object_type_id,action_type_id,registration_revision,manifest_digest,resource_org_id,
   valid_from,valid_until,delegable,clause_digest)
  VALUES(b.org_id,role,1,ordinal,'ALLOW',(clause->'action'->>'object_type_id')::uuid,
   (clause->'action'->>'action_type_id')::uuid,1,b.manifest_digest,b.org_id,b.started_at,NULL,ordinal>5,clause_hash);
  INSERT INTO public.policy_capability_clause_fields(org_id,role_id,role_revision,clause_index,
   object_type_id,property_id,schema_revision,manifest_digest,content_digest)
  SELECT b.org_id,role,1,ordinal,p.object_type_id,p.property_id,p.schema_revision,p.manifest_digest,p.content_digest
   FROM public.native_company_property_refs p
   WHERE p.org_id=b.org_id AND p.catalog_version=b.catalog_version
    AND p.object_type_id=(clause->'action'->>'object_type_id')::uuid AND ordinal NOT IN (4,5);
 END LOOP;
 INSERT INTO public.user_role_assignments(id,org_id,role_id,created_at,subject_protocol,account_id,
  native_current_revision,assigned_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
 VALUES(assignment,b.org_id,role,b.started_at,'NATIVE_ACCOUNT',b.administrative_account_id,
  1,b.account_id,b.account_id,b.command_id,b.receipt_id);
 INSERT INTO public.policy_assignment_revisions(org_id,assignment_id,revision,subject_protocol,account_id,
  role_id,role_revision,state,valid_from,valid_until,ceiling_digest,origin_account_id,origin_command_id,
  origin_receipt_id,actor_account_id,session_id,created_at)
 VALUES(b.org_id,assignment,1,'NATIVE_ACCOUNT',b.administrative_account_id,role,1,'ACTIVE',b.started_at,NULL,
  role_hash,b.account_id,b.command_id,b.receipt_id,b.account_id,b.session_id,b.started_at);
 INSERT INTO public.company_authority_heads(org_id,epoch,origin_account_id,origin_command_id,origin_receipt_id)
  VALUES(b.org_id,1,b.account_id,b.command_id,b.receipt_id);
 INSERT INTO public.account_context_candidates(account_id,source_key,context_kind,context_id,source_revision,incarnation,state)
 VALUES(b.administrative_account_id,jsonb_build_object('kind','COMPANY_POLICY','org_id',b.org_id::text,
  'assignment_id',assignment::text,'revision','1'),'COMPANY',b.org_id,1,NULL,'CURRENT');
 UPDATE public.account_security s SET context_generation=s.context_generation+1
  WHERE s.account_id=b.administrative_account_id AND s.security_state='ACTIVE'
   AND s.context_generation=(SELECT x.recipient_context_generation_before
    FROM public.company_enrollment_effect_bindings x WHERE x.account_id=p_account AND x.command_id=p_command);
 GET DIAGNOSTICS changed=ROW_COUNT;
 IF changed<>1 THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN QUERY SELECT assignment,1::bigint;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE;
END
$body$;

CREATE FUNCTION public.identity_native_root_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_value jsonb; b record;
BEGIN
 IF TG_OP<>'INSERT' AND OLD.subject_protocol='NATIVE_ACCOUNT' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 IF TG_OP='UPDATE' AND NEW.subject_protocol IS DISTINCT FROM OLD.subject_protocol THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 IF NEW.subject_protocol='LEGACY_USER' THEN RETURN NEW; END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 row_value:=to_jsonb(NEW);
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(NEW.origin_account_id,NEW.origin_command_id);
 IF b.request_state<>'PENDING' OR NEW.org_id IS DISTINCT FROM b.org_id
  OR NEW.origin_receipt_id IS DISTINCT FROM b.receipt_id OR NEW.native_current_revision<>1
  OR NEW.created_at IS DISTINCT FROM b.started_at
  OR (TG_TABLE_NAME='policy_roles' AND ((row_value->>'role_key') IS DISTINCT FROM 'native_company_administration'
    OR (row_value->>'display_name') IS DISTINCT FROM '회사 초기 관리자' OR (row_value->>'description') IS NOT NULL
    OR (row_value->>'status') IS DISTINCT FROM 'ACTIVE' OR (row_value->>'is_system')::boolean IS DISTINCT FROM true
    OR (row_value->>'created_by_account_id')::uuid IS DISTINCT FROM b.account_id OR (row_value->>'updated_by_account_id')::uuid IS DISTINCT FROM b.account_id
    OR (row_value->>'updated_at')::timestamptz IS DISTINCT FROM b.started_at))
  OR (TG_TABLE_NAME='user_role_assignments' AND ((row_value->>'account_id')::uuid IS DISTINCT FROM b.administrative_account_id
    OR (row_value->>'assigned_by_account_id')::uuid IS DISTINCT FROM b.account_id)) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.identity_native_legacy_child_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF (TG_OP<>'INSERT' AND EXISTS(SELECT 1 FROM public.policy_roles p WHERE p.id=OLD.role_id
     AND p.org_id=OLD.org_id AND p.subject_protocol<>'LEGACY_USER'))
  OR (TG_OP<>'DELETE' AND EXISTS(SELECT 1 FROM public.policy_roles p WHERE p.id=NEW.role_id
     AND p.org_id=NEW.org_id AND p.subject_protocol<>'LEGACY_USER')) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.identity_native_any_origin_v1() RETURNS boolean
LANGUAGE sql STABLE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
 -- Native Group origin is global and immutable. RLS-scoped role scans would
 -- miss another Company's native roots during a global TRUNCATE. Every valid
 -- native role/assignment is closed against this Group-owned birth.
 SELECT EXISTS(SELECT 1 FROM public.groups WHERE origin_account_id IS NOT NULL);
$body$;
CREATE FUNCTION public.identity_native_truncate_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF public.identity_native_any_origin_v1() THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
 END IF;
 RETURN NULL;
END
$body$;
CREATE FUNCTION public.identity_native_immutable_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
END
$body$;

CREATE FUNCTION public.identity_native_birth_row_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE value jsonb:=to_jsonb(NEW); parent record; b record; account uuid; command uuid; receipt uuid;
BEGIN
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 IF TG_TABLE_NAME IN ('policy_capability_clauses','policy_capability_clause_fields') THEN
  SELECT r.* INTO STRICT parent FROM public.policy_role_revisions r
   WHERE r.org_id=NEW.org_id AND r.role_id=NEW.role_id AND r.revision=NEW.role_revision;
  account:=parent.origin_account_id; command:=parent.origin_command_id; receipt:=parent.origin_receipt_id;
 ELSE
  account:=NEW.origin_account_id; command:=NEW.origin_command_id; receipt:=NEW.origin_receipt_id;
 END IF;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(account,command);
 IF b.request_state<>'PENDING' OR b.org_id IS DISTINCT FROM NEW.org_id OR b.receipt_id IS DISTINCT FROM receipt
  OR (value?'revision' AND (value->>'revision')::bigint IS DISTINCT FROM 1)
  OR (value?'role_revision' AND (value->>'role_revision')::bigint IS DISTINCT FROM 1)
  OR (value?'epoch' AND (value->>'epoch')::bigint IS DISTINCT FROM 1)
  OR (value?'subject_protocol' AND value->>'subject_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT')
  OR (value?'state' AND value->>'state' IS DISTINCT FROM 'ACTIVE')
  OR (value?'account_id' AND (value->>'account_id')::uuid IS DISTINCT FROM b.administrative_account_id)
  OR (value?'actor_account_id' AND (value->>'actor_account_id')::uuid IS DISTINCT FROM b.account_id)
  OR (value?'session_id' AND (value->>'session_id')::uuid IS DISTINCT FROM b.session_id)
  OR (value?'created_at' AND (value->>'created_at')::timestamptz IS DISTINCT FROM b.started_at)
  OR (value?'valid_from' AND (value->>'valid_from')::timestamptz IS DISTINCT FROM b.started_at)
  OR (value?'valid_until' AND value->'valid_until' IS DISTINCT FROM 'null'::jsonb) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
END
$body$;

CREATE FUNCTION public.identity_native_birth_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); value jsonb:=to_jsonb(NEW); account uuid; command uuid;
BEGIN
 IF value?'subject_protocol' AND value->>'subject_protocol'='LEGACY_USER' THEN RETURN NEW; END IF;
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 IF TG_TABLE_NAME IN ('policy_capability_clauses','policy_capability_clause_fields') THEN
  SELECT r.origin_account_id,r.origin_command_id INTO STRICT account,command
   FROM public.policy_role_revisions r
   WHERE r.org_id=NEW.org_id AND r.role_id=NEW.role_id AND r.revision=NEW.role_revision;
 ELSE
  account:=NEW.origin_account_id; command:=NEW.origin_command_id;
 END IF;
 PERFORM public.company_enrollment_assert_closure_v1(account,command);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: catalog-current.sql
CREATE FUNCTION ontology_api.lock_native_company_catalog_current_v1(p_org uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); found_count integer; installation record;
BEGIN
 PERFORM set_config('app.current_org',p_org::text,true);
 SELECT n.* INTO STRICT installation FROM public.native_company_catalog_installs n WHERE n.org_id=p_org;
 IF installation.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
  OR installation.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex') THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM k.org_id FROM public.ont_object_type_key_revisions k
  JOIN public.native_company_object_refs r ON r.org_id=k.org_id AND r.object_key=k.stable_key
  WHERE k.org_id=p_org ORDER BY k.org_id,k.stable_key COLLATE "C" FOR SHARE OF k;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>2 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM o.id FROM public.ont_object_types o JOIN public.native_company_object_refs r ON r.org_id=o.org_id AND r.object_type_id=o.id
  WHERE o.org_id=p_org ORDER BY o.org_id,o.id FOR SHARE OF o;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>2 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM p.id FROM public.ont_property_defs p JOIN public.native_company_property_refs r ON r.org_id=p.org_id AND r.property_id=p.id
  WHERE p.org_id=p_org ORDER BY p.org_id,p.object_type_id,p.id FOR SHARE OF p;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>10 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 PERFORM a.id FROM public.ont_action_types a JOIN public.native_company_action_refs r ON r.org_id=a.org_id AND r.action_type_id=a.id
  WHERE a.org_id=p_org ORDER BY a.org_id,a.object_type_id,a.id FOR SHARE OF a;
 GET DIAGNOSTICS found_count=ROW_COUNT;
 IF found_count<>5 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 IF (SELECT count(*) FROM public.native_company_object_refs r WHERE r.org_id=p_org)<>2
  OR (SELECT count(*) FROM public.native_company_action_refs r WHERE r.org_id=p_org)<>5
  OR (SELECT count(*) FROM public.native_company_property_refs r WHERE r.org_id=p_org)<>10
  OR EXISTS(SELECT 1 FROM public.native_company_object_refs r
   LEFT JOIN public.ont_object_types o ON o.org_id=r.org_id AND o.id=r.object_type_id
   LEFT JOIN public.ont_object_type_key_revisions k ON k.org_id=o.org_id AND k.stable_key=o.stable_key
   WHERE r.org_id=p_org AND (o.id IS NULL OR k.revision IS DISTINCT FROM 1
    OR o.schema_version IS DISTINCT FROM r.schema_revision OR o.lifecycle_state IS DISTINCT FROM 'published'
    OR o.stable_key IS DISTINCT FROM r.object_key OR o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
    OR o.origin_account_id IS DISTINCT FROM installation.origin_account_id
    OR o.origin_command_id IS DISTINCT FROM installation.origin_command_id
    OR o.origin_receipt_id IS DISTINCT FROM installation.origin_receipt_id
    OR r.content_digest IS DISTINCT FROM sha256(convert_to(jsonb_build_object('stable_key',o.stable_key,'title',o.title,
     'title_property_key',o.title_property_key,'backing_kind',o.backing_kind,'backing_table',o.backing_table,
     'primary_key_property',o.primary_key_property,'schema_version',o.schema_version,'lifecycle_state',o.lifecycle_state)::text,'UTF8'))
    OR EXISTS(SELECT 1 FROM public.ont_object_types other WHERE other.org_id=o.org_id AND other.stable_key=o.stable_key
     AND other.id<>o.id AND other.lifecycle_state='published')))
  OR EXISTS(SELECT 1 FROM public.native_company_action_refs r
   LEFT JOIN public.ont_action_types a ON a.org_id=r.org_id AND a.object_type_id=r.object_type_id AND a.id=r.action_type_id
   WHERE r.org_id=p_org AND (a.id IS NULL OR a.dispatch_target IS DISTINCT FROM r.action_key
    OR r.content_digest IS DISTINCT FROM sha256(convert_to(jsonb_build_object('stable_key',a.stable_key,'title',a.title,
     'params_schema',a.params_schema,'edits',a.edits,'submission_criteria',a.submission_criteria,'side_effects',a.side_effects,
     'dispatch',a.dispatch,'dispatch_target',a.dispatch_target,'control_points',a.control_points)::text,'UTF8'))))
  OR EXISTS(SELECT 1 FROM public.native_company_property_refs r
   LEFT JOIN public.ont_property_defs p ON p.org_id=r.org_id AND p.object_type_id=r.object_type_id AND p.id=r.property_id
   WHERE r.org_id=p_org AND (p.id IS NULL
    OR r.content_digest IS DISTINCT FROM sha256(convert_to(jsonb_build_object('key',p.key,'title',p.title,'type',p.type,
     'config',p.config,'backing_column',p.backing_column,'required',p.required,'in_property_policy',p.in_property_policy)::text,'UTF8')))) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: identity-current.sql
CREATE FUNCTION public.identity_company_projection_v1(p_account uuid,p_family uuid,p_company uuid)
RETURNS TABLE(company_epoch bigint,context_generation bigint,assignment_id uuid,assignment_revision bigint,
 role_id uuid,role_revision bigint,registered_clauses jsonb,company_name text,company_slug text)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); planned_group uuid; actual_group uuid;
 company uuid:=p_company; role_id_value uuid; birth_time timestamptz;
 control record; family record; organization record; assignment record; assignment_revision_row record;
 role record; role_revision_row record; clause_row record; head record; group_head record;
 clauses jsonb; clause jsonb; fields jsonb; role_hash bytea;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_account IS NULL OR p_family IS NULL OR p_company IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN (p_account,p_family,p_company)
  OR p_company='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT o.group_id INTO planned_group FROM public.organizations o WHERE o.id=p_company;
 IF planned_group IS NOT NULL THEN
  SELECT * INTO STRICT group_head FROM public.group_authority_lock_shared_v1(planned_group);
 END IF;
 SELECT o.group_id INTO actual_group FROM public.organizations o WHERE o.id=p_company;
 IF actual_group IS DISTINCT FROM planned_group THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='company_enrollment.lock_plan_changed';
 END IF;
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 PERFORM 1 FROM public.account_context_presence_v1(p_account);
 IF planned_group IS NULL THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT o.* INTO STRICT organization FROM public.organizations o WHERE o.id=p_company;
 IF organization.origin_account_id IS NULL THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF group_head.state<>'ACTIVE' OR organization.status<>'ACTIVE'
  OR NOT EXISTS(SELECT 1 FROM public.groups g WHERE g.id=planned_group AND g.status='ACTIVE') THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF NOT EXISTS(SELECT 1 FROM public.group_memberships m JOIN public.group_membership_revisions r
    ON r.group_id=m.group_id AND r.org_id=m.org_id AND r.membership_id=m.membership_id
     AND r.revision=m.current_revision AND r.incarnation=m.incarnation
   WHERE m.org_id=p_company AND m.group_id=planned_group AND r.state='ACTIVE' AND r.to_time IS NULL) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 SELECT h.* INTO STRICT head FROM public.company_authority_heads h WHERE h.org_id=p_company FOR SHARE;
 IF head.epoch<>1 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 IF NOT EXISTS(SELECT 1 FROM public.user_role_assignments a WHERE a.org_id=p_company
   AND a.subject_protocol='NATIVE_ACCOUNT' AND a.account_id=p_account) THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT a.* INTO STRICT assignment FROM public.user_role_assignments a WHERE a.org_id=p_company
  AND a.subject_protocol='NATIVE_ACCOUNT' AND a.account_id=p_account FOR SHARE;
 SELECT r.* INTO STRICT role FROM public.policy_roles r WHERE r.org_id=p_company AND r.id=assignment.role_id FOR SHARE;
 SELECT r.* INTO STRICT assignment_revision_row FROM public.policy_assignment_revisions r
  WHERE r.org_id=p_company AND r.assignment_id=assignment.id AND r.revision=assignment.native_current_revision;
 SELECT r.* INTO STRICT role_revision_row FROM public.policy_role_revisions r
  WHERE r.org_id=p_company AND r.role_id=role.id AND r.revision=role.native_current_revision;
 IF assignment.native_current_revision<>1 OR role.native_current_revision<>1
  OR role.subject_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT'
  OR assignment_revision_row.role_id IS DISTINCT FROM role.id OR assignment_revision_row.role_revision<>1
  OR assignment_revision_row.account_id IS DISTINCT FROM p_account
  OR assignment_revision_row.ceiling_digest IS DISTINCT FROM role_revision_row.clause_digest
  OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_receipts r JOIN public.company_enrollment_requests q
    ON q.account_id=r.account_id AND q.command_id=r.command_id AND q.committed_receipt_id=r.receipt_id
    WHERE r.org_id=p_company AND r.administrative_account_id=p_account AND r.root_assignment_id=assignment.id
     AND r.root_revision=1 AND q.state='COMMITTED' AND r.account_id=assignment.origin_account_id
     AND r.command_id=assignment.origin_command_id AND r.receipt_id=assignment.origin_receipt_id)
  OR NOT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_company AND a.account_id=p_account
    AND a.admission_receipt_id=assignment.origin_receipt_id) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 IF role.status<>'ACTIVE' OR role_revision_row.state<>'ACTIVE' OR assignment_revision_row.state<>'ACTIVE'
  OR role_revision_row.valid_from>clock_timestamp() OR assignment_revision_row.valid_from>clock_timestamp()
  OR (role_revision_row.valid_until IS NOT NULL AND role_revision_row.valid_until<=clock_timestamp())
  OR (assignment_revision_row.valid_until IS NOT NULL AND assignment_revision_row.valid_until<=clock_timestamp()) THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 IF role_revision_row.valid_until IS NOT NULL OR assignment_revision_row.valid_until IS NOT NULL
  OR role_revision_row.valid_from IS DISTINCT FROM assignment_revision_row.valid_from THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM ontology_api.lock_native_company_catalog_current_v1(p_company);
 role_id_value:=role.id; birth_time:=role_revision_row.valid_from;
 -- Reconstruct from authoritative field membership, not the cached digest.
 clauses:='[]'::jsonb;
 IF (SELECT count(*) FROM public.policy_capability_clauses c WHERE c.org_id=company AND c.role_id=role_id_value AND c.role_revision=1)<>7
  OR (SELECT count(*) FROM public.policy_capability_clause_fields f WHERE f.org_id=company AND f.role_id=role_id_value AND f.role_revision=1)<>16 THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 FOR clause_row IN SELECT c.*,a.action_key FROM public.policy_capability_clauses c
  JOIN public.native_company_action_refs a ON a.org_id=c.org_id AND a.object_type_id=c.action_object_type_id
   AND a.action_type_id=c.action_type_id AND a.registration_revision=c.registration_revision AND a.manifest_digest=c.manifest_digest
  WHERE c.org_id=company AND c.role_id=role_id_value AND c.role_revision=1 ORDER BY c.clause_index
 LOOP
  IF clause_row.clause_index IS DISTINCT FROM jsonb_array_length(clauses)+1
   OR clause_row.action_key IS DISTINCT FROM (ARRAY['context.discover','company.identity.read','company.policy.read',
    'company.policy.assign','company.policy.revoke','context.discover','company.identity.read'])[clause_row.clause_index]
   OR clause_row.effect IS DISTINCT FROM 'ALLOW' OR clause_row.resource_org_id IS DISTINCT FROM company
   OR clause_row.delegable IS DISTINCT FROM (clause_row.clause_index>5)
   OR clause_row.valid_from IS DISTINCT FROM birth_time OR clause_row.valid_until IS NOT NULL THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  SELECT coalesce(jsonb_agg(jsonb_build_object('org_id',f.org_id::text,'object_type_id',f.object_type_id::text,
    'property_id',f.property_id::text,'schema_revision',f.schema_revision::text)
    ORDER BY f.org_id,f.object_type_id,f.property_id,f.schema_revision),'[]'::jsonb) INTO fields
   FROM public.policy_capability_clause_fields f
   WHERE f.org_id=company AND f.role_id=role_id_value AND f.role_revision=1 AND f.clause_index=clause_row.clause_index;
  IF jsonb_array_length(fields) IS DISTINCT FROM (ARRAY[2,2,8,0,0,2,2])[clause_row.clause_index] THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  clause:=jsonb_build_object('kind','COMPANY_CAPABILITY_CLAUSE_V1',
   'action',jsonb_build_object('org_id',company::text,'object_type_id',clause_row.action_object_type_id::text,
    'action_type_id',clause_row.action_type_id::text,'registration_revision',clause_row.registration_revision::text,
    'manifest_digest',encode(clause_row.manifest_digest,'hex')),
   'resource',jsonb_build_object('kind','COMPANY','org_id',company::text),'fields',fields,
   'valid_from',to_char(clause_row.valid_from AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
   'valid_until',NULL,'delegable',clause_row.delegable);
  IF clause_row.clause_digest IS DISTINCT FROM sha256(convert_to(clause::text,'UTF8')) THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  clauses:=clauses||jsonb_build_array(clause);
 END LOOP;
 IF jsonb_array_length(clauses)<>7 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 SELECT sha256(convert_to(jsonb_agg(jsonb_build_object('clause_index',n,
   'clause_digest',encode(sha256(convert_to(c::text,'UTF8')),'hex')) ORDER BY n)::text,'UTF8'))
  INTO role_hash FROM jsonb_array_elements(clauses) WITH ORDINALITY AS x(c,n);

 IF role_hash IS DISTINCT FROM role_revision_row.clause_digest THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 RETURN QUERY SELECT head.epoch,control.context_generation,assignment.id,assignment.native_current_revision,
  role.id,role.native_current_revision,clauses,organization.name,organization.slug;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: account-context.sql
CREATE OR REPLACE FUNCTION public.account_context_presence_v1(p_account uuid)
RETURNS TABLE(context_generation bigint, has_candidates boolean)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE
    control record;
    origin record;
    candidate_count bigint;
    expected_generation bigint:=1;
    prior_org text:=current_setting('app.current_org',true);
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
    IF control.security_state IS DISTINCT FROM 'ACTIVE' OR control.context_generation NOT BETWEEN 1 AND 257 OR control.context_generation IS NULL THEN
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
    SELECT count(*) INTO candidate_count FROM public.account_context_candidates c WHERE c.account_id=p_account;
    IF candidate_count<>control.context_generation-1 THEN RAISE EXCEPTION 'account.navigation_unavailable'; END IF;
    FOR origin IN SELECT b.*,r.root_assignment_id,r.root_revision FROM public.company_enrollment_effect_bindings b
      JOIN public.company_enrollment_receipts r ON r.account_id=b.account_id AND r.command_id=b.command_id AND r.receipt_id=b.receipt_id
      JOIN public.company_enrollment_requests q ON q.account_id=r.account_id AND q.command_id=r.command_id
        AND q.committed_receipt_id=r.receipt_id AND q.state='COMMITTED'
      WHERE b.administrative_account_id=p_account AND r.administrative_account_id=p_account
      ORDER BY b.recipient_context_generation_before LIMIT 257
    LOOP
      IF expected_generation>=control.context_generation OR origin.recipient_context_generation_before<>expected_generation
        OR origin.root_revision<>1 THEN RAISE EXCEPTION 'account.navigation_unavailable'; END IF;
      IF NOT EXISTS(SELECT 1 FROM public.account_context_candidates c WHERE c.account_id=p_account
        AND c.source_key=jsonb_build_object('kind','COMPANY_POLICY','org_id',origin.org_id::text,
          'assignment_id',origin.root_assignment_id::text,'revision','1')
        AND c.context_kind='COMPANY' AND c.context_id=origin.org_id AND c.source_revision=1 AND c.incarnation IS NULL AND c.state='CURRENT') THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
      END IF;
      PERFORM set_config('app.current_org',origin.org_id::text,true);
      IF NOT EXISTS(SELECT 1 FROM public.user_role_assignments a WHERE a.org_id=origin.org_id AND a.id=origin.root_assignment_id
        AND a.account_id=p_account AND a.subject_protocol='NATIVE_ACCOUNT' AND a.native_current_revision=1
        AND a.origin_account_id=origin.account_id AND a.origin_command_id=origin.command_id AND a.origin_receipt_id=origin.receipt_id)
        OR NOT EXISTS(SELECT 1 FROM public.policy_assignment_revisions a WHERE a.org_id=origin.org_id
          AND a.assignment_id=origin.root_assignment_id AND a.revision=1 AND a.account_id=p_account
          AND a.origin_account_id=origin.account_id AND a.origin_command_id=origin.command_id AND a.origin_receipt_id=origin.receipt_id) THEN
        RAISE EXCEPTION 'account.navigation_unavailable';
      END IF;
      expected_generation:=expected_generation+1;
    END LOOP;
    IF expected_generation<>control.context_generation THEN RAISE EXCEPTION 'account.navigation_unavailable'; END IF;
    PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
    RETURN QUERY SELECT control.context_generation,EXISTS(
        SELECT 1 FROM public.account_context_candidates c WHERE c.account_id=p_account);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
    PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
    RAISE EXCEPTION 'account.navigation_unavailable';
WHEN OTHERS THEN
    PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: account-candidates.sql
CREATE FUNCTION public.account_company_context_candidates_v1(p_account uuid,p_family uuid)
RETURNS TABLE(context_generation bigint,candidate_org_ids uuid[])
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE control record; family record; presence record; candidates uuid[];
BEGIN
 SELECT * INTO STRICT control FROM public.account_security_lock_shared_v1(p_account);
 SELECT * INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_account,p_family);
 IF control.security_state IS DISTINCT FROM 'ACTIVE' OR family.protocol IS DISTINCT FROM 'ACCOUNT_V1'
  OR family.user_id IS DISTINCT FROM p_account OR family.org_id IS NOT NULL OR family.revoked_at IS NOT NULL
  OR family.account_security_generation IS DISTINCT FROM control.security_generation THEN
  RAISE EXCEPTION 'account.authentication_invalid';
 END IF;
 SELECT * INTO STRICT presence FROM public.account_context_presence_v1(p_account);
 SELECT coalesce(array_agg(c.context_id ORDER BY c.context_id),ARRAY[]::uuid[]) INTO candidates
  FROM (SELECT a.context_id FROM public.account_context_candidates a WHERE a.account_id=p_account
    ORDER BY a.context_id LIMIT 257) c;
 IF cardinality(candidates)>256 OR cardinality(candidates)<>presence.context_generation-1 THEN
  RAISE EXCEPTION 'account.navigation_unavailable';
 END IF;
 RETURN QUERY SELECT presence.context_generation,candidates;
END
$body$;


-- source: executor-fence-successor.sql
CREATE FUNCTION public.company_enrollment_execute_v1(p_account uuid,p_family uuid,p_command uuid,
 p_digest bytea,p_trace text,p_span text)
RETURNS TABLE(receipt_id uuid,org_id uuid,group_id uuid,administrative_account_id uuid,replayed boolean)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='1s'
AS $body$
DECLARE q public.company_enrollment_requests%ROWTYPE; r public.company_enrollment_receipts%ROWTYPE;
 d record; binding public.company_enrollment_effect_bindings%ROWTYPE; recipient record; catalog record; identity_result record;
BEGIN
 SELECT x.* INTO q FROM public.company_enrollment_requests x WHERE x.account_id=p_account AND x.command_id=p_command;
 PERFORM 1 FROM public.company_enrollment_session_material_v1(p_account,p_family,p_command,q.input_bytes);
 BEGIN
  PERFORM pg_advisory_xact_lock(hashtextextended(
  'console.company.enrollment.command/1:'||p_account::text||':'||p_command::text,0));
 EXCEPTION WHEN lock_not_available THEN
  RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='company_enrollment.outcome_unknown';
 END;
 SELECT x.* INTO STRICT q FROM public.company_enrollment_requests x WHERE x.account_id=p_account AND x.command_id=p_command;
 IF q.input_digest IS DISTINCT FROM p_digest THEN RAISE EXCEPTION 'company_enrollment.conflict'; END IF;
 IF q.state='COMMITTED' THEN
  SELECT x.* INTO STRICT r FROM public.company_enrollment_receipts x
   WHERE x.account_id=p_account AND x.command_id=p_command AND x.receipt_id=q.committed_receipt_id
    AND x.input_digest=p_digest;
  RETURN QUERY SELECT r.receipt_id,r.org_id,r.group_id,r.administrative_account_id,true;
  RETURN;
 END IF;
 IF q.state<>'PENDING' OR q.input_bytes IS NULL OR clock_timestamp()>=q.expires_at THEN
  RAISE EXCEPTION 'company_enrollment.conflict';
 END IF;
 SELECT * INTO STRICT d FROM public.company_enrollment_decode_input_v1(q.input_bytes);
 IF d.group_id IS NOT NULL THEN RAISE EXCEPTION 'company_enrollment.group_unavailable'; END IF;
 IF NOT public.account_company_setup_eligibility_v1(p_account) THEN RAISE EXCEPTION 'company_enrollment.forbidden'; END IF;
 IF NOT EXISTS(SELECT 1 FROM public.deployment_operator_head h WHERE h.singleton=1
   AND h.account_id=p_account AND h.receipt_id=q.designation_receipt_id) THEN
  RAISE EXCEPTION 'company_enrollment.forbidden';
 END IF;
 SELECT s.* INTO STRICT recipient FROM public.account_security s WHERE s.account_id=d.administrative_account_id;
 IF recipient.security_state IS DISTINCT FROM 'ACTIVE' OR recipient.context_generation NOT BETWEEN 1 AND 256 THEN
  RAISE EXCEPTION 'company_enrollment.forbidden';
 END IF;
 PERFORM 1 FROM public.account_login_consent_v1(p_account);
 PERFORM 1 FROM public.account_login_consent_v1(d.administrative_account_id);
 PERFORM 1 FROM public.account_context_presence_v1(d.administrative_account_id);
 IF p_trace IS NULL OR p_trace !~ '^[0-9a-f]{32}$' OR p_trace=repeat('0',32)
  OR p_span IS NULL OR p_span !~ '^[0-9a-f]{16}$' OR p_span=repeat('0',16) THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 INSERT INTO public.company_enrollment_effect_bindings(account_id,command_id,codec_version,effect_xid,effect_backend_pid,
  org_id,group_id,receipt_id,administrative_account_id,designation_receipt_id,input_digest,started_at,catalog_version,
  manifest_digest,session_id,recipient_context_generation_before)
 VALUES(p_account,p_command,1,pg_current_xact_id(),pg_backend_pid(),gen_random_uuid(),gen_random_uuid(),gen_random_uuid(),
  d.administrative_account_id,q.designation_receipt_id,q.input_digest,clock_timestamp(),
  'native-company-identity-2026-09-19.1',decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex'),
  p_family,recipient.context_generation) RETURNING * INTO binding;
 PERFORM 1 FROM public.company_enrollment_topology_v1(p_account,p_command);
 SELECT * INTO STRICT catalog FROM ontology_api.install_native_company_catalog_v1(p_account,p_command,p_trace,p_span);
 SELECT * INTO STRICT identity_result FROM public.identity_enroll_company_administration_v1(p_account,p_command);
 PERFORM 1 FROM ont_policy_api.install_native_company_policy_v1(p_account,p_command,p_trace,p_span);
 PERFORM public.company_enrollment_audit_v1(p_account,p_command,p_trace,p_span);
 INSERT INTO public.company_enrollment_receipts(receipt_id,account_id,command_id,codec_version,input_digest,designation_receipt_id,
  org_id,group_id,administrative_account_id,root_assignment_id,root_revision,catalog_version,manifest_digest,action_refs,property_refs,session_id,committed_at)
 VALUES(binding.receipt_id,p_account,p_command,1,binding.input_digest,binding.designation_receipt_id,
  binding.org_id,binding.group_id,binding.administrative_account_id,identity_result.root_assignment_id,identity_result.root_revision,
  binding.catalog_version,binding.manifest_digest,catalog.action_refs,catalog.property_refs,p_family,binding.started_at);
 UPDATE public.company_enrollment_requests x SET state='COMMITTED',committed_receipt_id=binding.receipt_id,terminal_at=binding.started_at
  WHERE x.account_id=p_account AND x.command_id=p_command AND x.state='PENDING';
 IF NOT FOUND THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 INSERT INTO public.company_enrollment_request_events(account_id,command_id,event_revision,from_state,to_state,
  occurred_at,actor_account_id,session_id,reason_code)
 VALUES(p_account,p_command,2,'PENDING','COMMITTED',binding.started_at,p_account,p_family,'COMMITTED');
 PERFORM public.company_enrollment_assert_closure_v1(p_account,p_command);
 RETURN QUERY SELECT binding.receipt_id,binding.org_id,binding.group_id,binding.administrative_account_id,false;
END
$body$;

CREATE FUNCTION public.company_enrollment_audit_v1(p_account uuid,p_command uuid,p_trace text,p_span text) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 IF b.request_state<>'PENDING' THEN RAISE EXCEPTION 'company_enrollment.binding_invalid'; END IF;
 PERFORM set_config('app.current_org',b.org_id::text,true);
 INSERT INTO public.audit_events(id,actor,action,target_type,target_id,before_snap,after_snap,trace_id,span_id,occurred_at,org_id)
 VALUES(gen_random_uuid(),b.account_id,'company.enroll','organizations',b.org_id::text,NULL,
  jsonb_build_object('org_id',b.org_id::text,'group_id',b.group_id::text,
   'administrative_account_id',b.administrative_account_id::text,
   'enrollment',jsonb_build_object('account_id',b.account_id::text,'command_id',b.command_id::text,
    'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)),p_trace,p_span,b.started_at,b.org_id);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: closure.sql
CREATE FUNCTION public.company_enrollment_assert_closure_v1(p_account uuid,p_command uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record; q public.company_enrollment_requests%ROWTYPE; r public.company_enrollment_receipts%ROWTYPE;
 role record; assignment record; role_revision_row record; assignment_revision_row record; membership record;
 prior_org text:=current_setting('app.current_org',true); company uuid; role_id_value uuid; birth_time timestamptz;
 clauses jsonb; clause jsonb; fields jsonb; role_hash bytea; clause_row record;
 action_array jsonb; property_array jsonb; event record; audit record; object_row record; expected_snapshot jsonb;
 trace text; span text; provenance jsonb;
 native_manifest constant jsonb:=$manifest${"catalog_version":"native-company-identity-2026-09-19.1","object_types":[{"stable_key":"company_workspace","title":"회사 업무 공간","title_property_key":"name","backing_kind":"projected","backing_table":"organizations","primary_key_property":"id","properties":[{"key":"name","title":"회사 이름","field_type":"text","config":{"utf8_bytes_min":1,"utf8_bytes_max":256,"nonblank":true,"controls_allowed":false},"backing_column":"name","required":true,"in_property_policy":true},{"key":"slug","title":"업무 공간 식별자","field_type":"text","config":{"ascii_pattern":"^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?$","bytes_max":63},"backing_column":"slug","required":true,"in_property_policy":true}],"links":[],"actions":[{"stable_key":"context_discover","title":"회사 찾기","params_schema":{"type":"object","additionalProperties":false,"properties":{},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"context.discover","control_points":["authority"]},{"stable_key":"identity_read","title":"회사 정보 보기","params_schema":{"type":"object","additionalProperties":false,"properties":{},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.identity.read","control_points":["authority"]}],"analytics":[]},{"stable_key":"company_policy_assignment","title":"회사 권한 부여","title_property_key":null,"backing_kind":"projected","backing_table":"user_role_assignments","primary_key_property":"id","properties":[{"key":"account_id","title":"계정","field_type":"reference","config":{},"backing_column":"account_id","required":true,"in_property_policy":true},{"key":"scope","title":"회사 범위","field_type":"json","config":{"schema":{"type":"object","additionalProperties":false,"properties":{"kind":{"const":"COMPANY"},"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}}},"required":["kind","org_id"]}},"backing_column":"scope","required":true,"in_property_policy":true},{"key":"actions","title":"작업 권한","field_type":"json","config":{"schema":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"action_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"registration_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"manifest_digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["org_id","action_type_id","object_type_id","registration_revision","manifest_digest"]},"uniqueItems":true,"maxItems":5}},"backing_column":"actions","required":true,"in_property_policy":true},{"key":"fields","title":"열람 항목","field_type":"json","config":{"schema":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"property_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"schema_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"required":["org_id","object_type_id","property_id","schema_revision"]},"uniqueItems":true,"maxItems":10}},"backing_column":"fields","required":true,"in_property_policy":true},{"key":"valid_from","title":"시작 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC"},"backing_column":"valid_from","required":true,"in_property_policy":true},{"key":"valid_until","title":"종료 시각","field_type":"timestamp","config":{"precision":"microsecond","timezone":"UTC","nullable":true},"backing_column":"valid_until","required":false,"in_property_policy":true},{"key":"state","title":"상태","field_type":"choice","config":{"choices":[{"id":"ACTIVE","name":"ACTIVE"},{"id":"REVOKED","name":"REVOKED"}]},"backing_column":"state","required":true,"in_property_policy":true},{"key":"revision","title":"버전","field_type":"text","config":{"schema":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"backing_column":"revision","required":true,"in_property_policy":true}],"links":[],"actions":[{"stable_key":"policy_read","title":"권한 보기","params_schema":{"type":"object","additionalProperties":false,"properties":{},"required":[]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.policy.read","control_points":["authority"]},{"stable_key":"policy_assign","title":"권한 부여","params_schema":{"type":"object","additionalProperties":false,"properties":{"command_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"expected_company_epoch":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"reason":{"type":"string","minLength":1,"maxLength":512},"administrative_account_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"parent_assignment_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"parent_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"valid_from":{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},"valid_until":{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},"clauses":{"type":"array","minItems":1,"maxItems":2,"uniqueItems":true,"items":{"type":"object","additionalProperties":false,"properties":{"kind":{"const":"COMPANY_CAPABILITY_CLAUSE_V1"},"action":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"action_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"registration_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"manifest_digest":{"type":"string","pattern":"^[0-9a-f]{64}$"}},"required":["org_id","action_type_id","object_type_id","registration_revision","manifest_digest"]},"resource":{"type":"object","additionalProperties":false,"properties":{"kind":{"const":"COMPANY"},"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}}},"required":["kind","org_id"]},"fields":{"type":"array","maxItems":8,"uniqueItems":true,"items":{"type":"object","additionalProperties":false,"properties":{"org_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"object_type_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"property_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"schema_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"required":["org_id","object_type_id","property_id","schema_revision"]}},"valid_from":{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},"valid_until":{"anyOf":[{"type":"string","format":"date-time","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\\.[0-9]{6}Z$"},{"type":"null"}]},"delegable":{"type":"boolean"}},"required":["kind","action","resource","fields","valid_from","valid_until","delegable"]}}},"required":["command_id","expected_company_epoch","reason","administrative_account_id","parent_assignment_id","parent_revision","valid_from","valid_until","clauses"]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.policy.assign","control_points":["authority"]},{"stable_key":"policy_revoke","title":"권한 회수","params_schema":{"type":"object","additionalProperties":false,"properties":{"command_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"expected_company_epoch":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"},"reason":{"type":"string","minLength":1,"maxLength":512},"assignment_id":{"type":"string","format":"uuid","pattern":"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$","not":{"const":"00000000-0000-0000-0000-000000000000"}},"expected_revision":{"type":"string","pattern":"^[1-9][0-9]{0,18}$","x-max-integer":"9223372036854775807"}},"required":["command_id","expected_company_epoch","reason","assignment_id","expected_revision"]},"edits":[],"submission_criteria":[],"side_effects":[],"dispatch":"projected_usecase","dispatch_target":"company.policy.revoke","control_points":["authority"]}],"analytics":[]}]}$manifest$::jsonb;
BEGIN
 -- Resolve current backend/xid and namespace before touching any RLS source.
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(p_account,p_command);
 PERFORM set_config('app.current_org',b.org_id::text,true);
 company:=b.org_id; birth_time:=b.started_at;
 SELECT x.* INTO STRICT q FROM public.company_enrollment_requests x WHERE x.account_id=p_account AND x.command_id=p_command;
 SELECT x.* INTO STRICT r FROM public.company_enrollment_receipts x WHERE x.account_id=p_account AND x.command_id=p_command;
 IF b.request_state<>'COMMITTED' OR q.committed_receipt_id IS DISTINCT FROM b.receipt_id
  OR q.terminal_at IS DISTINCT FROM b.started_at OR r.receipt_id IS DISTINCT FROM b.receipt_id
  OR r.codec_version<>1 OR r.catalog_version IS DISTINCT FROM b.catalog_version OR r.manifest_digest IS DISTINCT FROM b.manifest_digest
  OR r.input_digest IS DISTINCT FROM b.input_digest
  OR sha256(q.input_bytes) IS DISTINCT FROM b.input_digest OR r.designation_receipt_id IS DISTINCT FROM b.designation_receipt_id
  OR r.org_id IS DISTINCT FROM b.org_id OR r.group_id IS DISTINCT FROM b.group_id
  OR r.administrative_account_id IS DISTINCT FROM b.administrative_account_id OR r.root_revision<>1
  OR r.session_id IS DISTINCT FROM b.session_id OR r.committed_at IS DISTINCT FROM b.started_at
  OR (SELECT count(*) FROM public.company_enrollment_request_events e WHERE e.account_id=p_account AND e.command_id=p_command)<>2 THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 SELECT e.* INTO STRICT event FROM public.company_enrollment_request_events e WHERE e.account_id=p_account AND e.command_id=p_command AND e.event_revision=2;
 IF event.from_state IS DISTINCT FROM 'PENDING' OR event.to_state IS DISTINCT FROM 'COMMITTED'
  OR event.reason_code IS DISTINCT FROM 'COMMITTED' OR event.occurred_at IS DISTINCT FROM b.started_at
  OR event.actor_account_id IS DISTINCT FROM b.account_id OR event.session_id IS DISTINCT FROM b.session_id THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 IF (SELECT count(*) FROM public.organizations o WHERE o.origin_account_id=p_account AND o.origin_command_id=p_command)<>1
  OR (SELECT count(*) FROM public.groups g WHERE g.origin_account_id=p_account AND g.origin_command_id=p_command)<>1
  OR NOT EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=b.org_id AND o.group_id=b.group_id
   AND o.slug=b.company_slug AND o.name=b.company_name AND o.status='ACTIVE'
   AND o.created_at=b.started_at AND o.updated_at=b.started_at AND o.origin_account_id=b.account_id
   AND o.origin_command_id=b.command_id AND o.origin_receipt_id=b.receipt_id)
  OR NOT EXISTS(SELECT 1 FROM public.groups g WHERE g.id=b.group_id AND g.name=b.company_name AND g.status='ACTIVE'
   AND g.created_at=b.started_at AND g.updated_at=b.started_at AND g.origin_account_id=b.account_id
   AND g.origin_command_id=b.command_id AND g.origin_receipt_id=b.receipt_id)
  OR NOT EXISTS(SELECT 1 FROM public.group_authority_heads h WHERE h.group_id=b.group_id AND h.revision=1 AND h.state='ACTIVE')
  OR (SELECT count(*) FROM public.group_memberships m WHERE m.group_id=b.group_id OR m.org_id=b.org_id)<>1
  OR (SELECT count(*) FROM public.group_membership_revisions h WHERE h.group_id=b.group_id OR h.org_id=b.org_id)<>1 THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 SELECT h.* INTO STRICT membership FROM public.group_memberships m JOIN public.group_membership_revisions h
  ON h.group_id=m.group_id AND h.org_id=m.org_id AND h.membership_id=m.membership_id AND h.revision=m.current_revision AND h.incarnation=m.incarnation
  WHERE m.org_id=b.org_id AND m.group_id=b.group_id AND m.current_revision=1 AND m.created_at=b.started_at;
 IF membership.state<>'ACTIVE' OR membership.from_time IS DISTINCT FROM b.started_at OR membership.to_time IS NOT NULL
  OR membership.provenance_kind IS DISTINCT FROM 'COMPANY_ENROLLMENT_V1' OR membership.native_account_id IS DISTINCT FROM b.account_id
  OR membership.command_id IS DISTINCT FROM b.command_id OR membership.command_receipt IS DISTINCT FROM b.receipt_id
  OR membership.legacy_actor_user_id IS NOT NULL THEN RAISE EXCEPTION 'company_enrollment.birth_closure_invalid'; END IF;
 IF (SELECT count(*) FROM public.company_actors a WHERE a.org_id=b.org_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=b.org_id AND a.account_id=b.administrative_account_id
    AND a.admission_receipt_id=b.receipt_id AND a.created_at=b.started_at
    AND a.entitlement_ref=jsonb_build_object('kind','COMPANY_ENROLLMENT_V1','account_id',b.account_id::text,
     'command_id',b.command_id::text,'org_id',b.org_id::text,'receipt_id',b.receipt_id::text))
  OR EXISTS(SELECT 1 FROM public.users u WHERE u.org_id=b.org_id)
  OR EXISTS(SELECT 1 FROM public.group_role_grants g WHERE g.group_id=b.group_id)
  OR (SELECT count(*) FROM public.policy_roles x WHERE x.org_id=b.org_id)<>1
  OR (SELECT count(*) FROM public.user_role_assignments x WHERE x.org_id=b.org_id)<>1
  OR (SELECT count(*) FROM public.policy_role_revisions x WHERE x.org_id=b.org_id)<>1
  OR (SELECT count(*) FROM public.policy_assignment_revisions x WHERE x.org_id=b.org_id)<>1
  OR EXISTS(SELECT 1 FROM public.policy_role_permissions x WHERE x.org_id=b.org_id)
  OR EXISTS(SELECT 1 FROM public.policy_role_conditions x WHERE x.org_id=b.org_id) THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT assignment FROM public.user_role_assignments x WHERE x.org_id=b.org_id;
 SELECT x.* INTO STRICT role FROM public.policy_roles x WHERE x.org_id=b.org_id;
 SELECT x.* INTO STRICT role_revision_row FROM public.policy_role_revisions x WHERE x.org_id=b.org_id;
 SELECT x.* INTO STRICT assignment_revision_row FROM public.policy_assignment_revisions x WHERE x.org_id=b.org_id;
 IF assignment.id IS DISTINCT FROM r.root_assignment_id OR assignment.account_id IS DISTINCT FROM b.administrative_account_id
  OR assignment.role_id IS DISTINCT FROM role.id OR assignment.native_current_revision<>1 OR assignment.subject_protocol<>'NATIVE_ACCOUNT'
  OR role.native_current_revision<>1 OR role.subject_protocol<>'NATIVE_ACCOUNT' OR role.status<>'ACTIVE'
  OR role.role_key<>'native_company_administration' OR role.display_name<>'회사 초기 관리자' OR role.description IS NOT NULL OR NOT role.is_system
  OR role_revision_row.role_id IS DISTINCT FROM role.id OR role_revision_row.revision<>1 OR role_revision_row.state<>'ACTIVE'
  OR assignment_revision_row.assignment_id IS DISTINCT FROM assignment.id OR assignment_revision_row.revision<>1 OR assignment_revision_row.state<>'ACTIVE'
  OR assignment_revision_row.account_id IS DISTINCT FROM b.administrative_account_id OR assignment_revision_row.role_id IS DISTINCT FROM role.id
  OR assignment_revision_row.role_revision<>1 OR assignment_revision_row.ceiling_digest IS DISTINCT FROM role_revision_row.clause_digest
  OR role_revision_row.valid_from IS DISTINCT FROM b.started_at OR role_revision_row.valid_until IS NOT NULL
  OR assignment_revision_row.valid_from IS DISTINCT FROM b.started_at OR assignment_revision_row.valid_until IS NOT NULL THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 role_id_value:=role.id;
 -- Reconstruct from authoritative field membership, not the cached digest.
 clauses:='[]'::jsonb;
 IF (SELECT count(*) FROM public.policy_capability_clauses c WHERE c.org_id=company AND c.role_id=role_id_value AND c.role_revision=1)<>7
  OR (SELECT count(*) FROM public.policy_capability_clause_fields f WHERE f.org_id=company AND f.role_id=role_id_value AND f.role_revision=1)<>16 THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 FOR clause_row IN SELECT c.*,a.action_key FROM public.policy_capability_clauses c
  JOIN public.native_company_action_refs a ON a.org_id=c.org_id AND a.object_type_id=c.action_object_type_id
   AND a.action_type_id=c.action_type_id AND a.registration_revision=c.registration_revision AND a.manifest_digest=c.manifest_digest
  WHERE c.org_id=company AND c.role_id=role_id_value AND c.role_revision=1 ORDER BY c.clause_index
 LOOP
  IF clause_row.clause_index IS DISTINCT FROM jsonb_array_length(clauses)+1
   OR clause_row.action_key IS DISTINCT FROM (ARRAY['context.discover','company.identity.read','company.policy.read',
    'company.policy.assign','company.policy.revoke','context.discover','company.identity.read'])[clause_row.clause_index]
   OR clause_row.effect IS DISTINCT FROM 'ALLOW' OR clause_row.resource_org_id IS DISTINCT FROM company
   OR clause_row.delegable IS DISTINCT FROM (clause_row.clause_index>5)
   OR clause_row.valid_from IS DISTINCT FROM birth_time OR clause_row.valid_until IS NOT NULL THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  SELECT coalesce(jsonb_agg(jsonb_build_object('org_id',f.org_id::text,'object_type_id',f.object_type_id::text,
    'property_id',f.property_id::text,'schema_revision',f.schema_revision::text)
    ORDER BY f.org_id,f.object_type_id,f.property_id,f.schema_revision),'[]'::jsonb) INTO fields
   FROM public.policy_capability_clause_fields f
   WHERE f.org_id=company AND f.role_id=role_id_value AND f.role_revision=1 AND f.clause_index=clause_row.clause_index;
  IF jsonb_array_length(fields) IS DISTINCT FROM (ARRAY[2,2,8,0,0,2,2])[clause_row.clause_index] THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  clause:=jsonb_build_object('kind','COMPANY_CAPABILITY_CLAUSE_V1',
   'action',jsonb_build_object('org_id',company::text,'object_type_id',clause_row.action_object_type_id::text,
    'action_type_id',clause_row.action_type_id::text,'registration_revision',clause_row.registration_revision::text,
    'manifest_digest',encode(clause_row.manifest_digest,'hex')),
   'resource',jsonb_build_object('kind','COMPANY','org_id',company::text),'fields',fields,
   'valid_from',to_char(clause_row.valid_from AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"'),
   'valid_until',NULL,'delegable',clause_row.delegable);
  IF clause_row.clause_digest IS DISTINCT FROM sha256(convert_to(clause::text,'UTF8')) THEN
   RAISE EXCEPTION 'identity_native.material_unavailable';
  END IF;
  clauses:=clauses||jsonb_build_array(clause);
 END LOOP;
 IF jsonb_array_length(clauses)<>7 THEN RAISE EXCEPTION 'identity_native.material_unavailable'; END IF;
 SELECT sha256(convert_to(jsonb_agg(jsonb_build_object('clause_index',n,
   'clause_digest',encode(sha256(convert_to(c::text,'UTF8')),'hex')) ORDER BY n)::text,'UTF8'))
  INTO role_hash FROM jsonb_array_elements(clauses) WITH ORDINALITY AS x(c,n);

 IF role_hash IS DISTINCT FROM role_revision_row.clause_digest THEN RAISE EXCEPTION 'company_enrollment.birth_closure_invalid'; END IF;
 IF NOT EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=b.org_id AND h.epoch=1
   AND h.origin_account_id=b.account_id AND h.origin_command_id=b.command_id AND h.origin_receipt_id=b.receipt_id)
  OR (SELECT count(*) FROM public.account_context_candidates c WHERE c.context_kind='COMPANY' AND c.context_id=b.org_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.account_context_candidates c WHERE c.account_id=b.administrative_account_id
   AND c.context_kind='COMPANY' AND c.context_id=b.org_id AND c.source_revision=1 AND c.incarnation IS NULL AND c.state='CURRENT'
   AND c.source_key=jsonb_build_object('kind','COMPANY_POLICY','org_id',b.org_id::text,'assignment_id',assignment.id::text,'revision','1'))
  OR NOT EXISTS(SELECT 1 FROM public.account_security s JOIN public.company_enrollment_effect_bindings x
   ON x.administrative_account_id=s.account_id WHERE x.account_id=p_account AND x.command_id=p_command
    AND s.context_generation=x.recipient_context_generation_before+1 AND s.security_state='ACTIVE') THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 FOR provenance IN
 SELECT to_jsonb(x) FROM public.policy_roles x WHERE x.org_id=b.org_id
 UNION ALL SELECT to_jsonb(x) FROM public.user_role_assignments x WHERE x.org_id=b.org_id
 UNION ALL SELECT to_jsonb(x) FROM public.policy_role_revisions x WHERE x.org_id=b.org_id
 UNION ALL SELECT to_jsonb(x) FROM public.policy_assignment_revisions x WHERE x.org_id=b.org_id
 UNION ALL SELECT to_jsonb(x) FROM public.native_company_catalog_installs x WHERE x.org_id=b.org_id
 UNION ALL SELECT jsonb_build_object('created_by',x.created_by,'created_by_account_id',x.created_by_account_id,'created_at',x.created_at,'updated_at',x.updated_at,'origin_account_id',x.origin_account_id,'origin_command_id',x.origin_command_id,'origin_receipt_id',x.origin_receipt_id,'attribution_protocol',x.attribution_protocol) FROM public.ont_object_types x WHERE x.org_id=b.org_id
 UNION ALL SELECT jsonb_build_object('installed_by',x.installed_by,'installed_by_account_id',x.installed_by_account_id,'installed_at',x.installed_at,'origin_account_id',x.origin_account_id,'origin_command_id',x.origin_command_id,'origin_receipt_id',x.origin_receipt_id,'attribution_protocol',x.attribution_protocol) FROM public.ont_builtin_catalog_installs x WHERE x.org_id=b.org_id
 UNION ALL SELECT jsonb_build_object('created_by',x.created_by,'updated_by',x.updated_by,'created_by_account_id',x.created_by_account_id,'updated_by_account_id',x.updated_by_account_id,'created_at',x.created_at,'updated_at',x.updated_at,'origin_account_id',x.origin_account_id,'origin_command_id',x.origin_command_id,'origin_receipt_id',x.origin_receipt_id,'attribution_protocol',x.attribution_protocol) FROM public.cedar_policy_catalog_entries x WHERE x.org_id=b.org_id
 UNION ALL SELECT jsonb_build_object('created_by',x.created_by,'created_by_account_id',x.created_by_account_id,'created_at',x.created_at,'origin_account_id',x.origin_account_id,'origin_command_id',x.origin_command_id,'origin_receipt_id',x.origin_receipt_id,'attribution_protocol',x.attribution_protocol) FROM public.ont_object_policies x WHERE x.org_id=b.org_id
 LOOP
  IF (provenance->>'origin_account_id')::uuid IS DISTINCT FROM b.account_id
   OR (provenance->>'origin_command_id')::uuid IS DISTINCT FROM b.command_id
   OR (provenance->>'origin_receipt_id')::uuid IS DISTINCT FROM b.receipt_id
   OR (provenance?'subject_protocol' AND provenance->>'subject_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT')
   OR (provenance?'attribution_protocol' AND provenance->>'attribution_protocol' IS DISTINCT FROM 'NATIVE_ACCOUNT') THEN
   RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
  END IF;
  FOREACH trace IN ARRAY ARRAY['created_by','updated_by','assigned_by','installed_by','user_id'] LOOP
   IF provenance?trace AND provenance->trace IS DISTINCT FROM 'null'::jsonb THEN
    RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
   END IF;
  END LOOP;
  FOREACH trace IN ARRAY ARRAY['created_by_account_id','updated_by_account_id','assigned_by_account_id','installed_by_account_id','actor_account_id'] LOOP
   IF provenance?trace AND (provenance->>trace)::uuid IS DISTINCT FROM b.account_id THEN
    RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
   END IF;
  END LOOP;
  FOREACH trace IN ARRAY ARRAY['created_at','updated_at','installed_at'] LOOP
   IF provenance?trace AND (provenance->>trace)::timestamptz IS DISTINCT FROM b.started_at THEN
    RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
   END IF;
  END LOOP;
  IF provenance?'session_id' AND (provenance->>'session_id')::uuid IS DISTINCT FROM b.session_id THEN
   RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
  END IF;
 END LOOP;
 trace:=NULL;
 IF EXISTS(SELECT 1 FROM public.ont_property_defs x WHERE x.org_id=b.org_id AND x.created_at IS DISTINCT FROM b.started_at)
  OR EXISTS(SELECT 1 FROM public.ont_action_types x WHERE x.org_id=b.org_id AND x.created_at IS DISTINCT FROM b.started_at)
  OR EXISTS(SELECT 1 FROM public.ont_object_type_key_revisions x WHERE x.org_id=b.org_id
   AND (x.revision<>1 OR x.created_at IS DISTINCT FROM b.started_at OR x.updated_at IS DISTINCT FROM b.started_at)) THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 PERFORM 1 FROM public.account_context_presence_v1(b.administrative_account_id);
 PERFORM ontology_api.lock_native_company_catalog_current_v1(b.org_id);
 IF (SELECT count(*) FROM public.ont_object_types x WHERE x.org_id=b.org_id)<>2
  OR (SELECT count(*) FROM public.ont_action_types x WHERE x.org_id=b.org_id)<>5
  OR (SELECT count(*) FROM public.ont_property_defs x WHERE x.org_id=b.org_id)<>10
  OR EXISTS(SELECT 1 FROM public.ont_link_types x WHERE x.org_id=b.org_id)
  OR EXISTS(SELECT 1 FROM public.ont_analytics x WHERE x.org_id=b.org_id)
  OR (SELECT count(*) FROM public.ont_builtin_catalog_installs x WHERE x.org_id=b.org_id)<>1
  OR (SELECT count(*) FROM public.native_company_catalog_installs x WHERE x.org_id=b.org_id)<>1
  OR (SELECT count(*) FROM public.ont_object_policies x WHERE x.org_id=b.org_id)<>2
  OR (SELECT count(*) FROM public.cedar_policy_catalog_entries x WHERE x.org_id=b.org_id)<>2 THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 -- Check actual canonical registry contents against the immutable allowlisted
 -- manifest as well as maps. A matching map digest alone cannot attest birth.
 FOR expected_snapshot IN SELECT value FROM jsonb_array_elements(native_manifest->'object_types') LOOP
  SELECT x.id,x.org_id,x.stable_key,x.title,x.title_property_key,x.backing_kind,x.backing_table,x.primary_key_property,x.schema_version,x.lifecycle_state,x.created_by,x.created_at,x.updated_at,x.attribution_protocol,x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id INTO STRICT object_row FROM public.ont_object_types x WHERE x.org_id=b.org_id AND x.stable_key=expected_snapshot->>'stable_key';
  IF jsonb_build_object('stable_key',object_row.stable_key,'title',object_row.title,'title_property_key',object_row.title_property_key,
   'backing_kind',object_row.backing_kind,'backing_table',object_row.backing_table,'primary_key_property',object_row.primary_key_property)
   IS DISTINCT FROM expected_snapshot-ARRAY['properties','links','actions','analytics'] THEN
   RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
  END IF;
  FOR provenance IN SELECT value FROM jsonb_array_elements(expected_snapshot->'properties') LOOP
   IF NOT EXISTS(SELECT 1 FROM public.ont_property_defs x WHERE x.org_id=b.org_id AND x.object_type_id=object_row.id
    AND jsonb_build_object('key',x.key,'title',x.title,'field_type',x.type,'config',x.config,'backing_column',x.backing_column,
     'required',x.required,'in_property_policy',x.in_property_policy)=provenance) THEN
    RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
   END IF;
  END LOOP;
  FOR provenance IN SELECT value FROM jsonb_array_elements(expected_snapshot->'actions') LOOP
   IF NOT EXISTS(SELECT 1 FROM public.ont_action_types x WHERE x.org_id=b.org_id AND x.object_type_id=object_row.id
    AND jsonb_build_object('stable_key',x.stable_key,'title',x.title,'params_schema',x.params_schema,'edits',x.edits,
     'submission_criteria',x.submission_criteria,'side_effects',x.side_effects,'dispatch',x.dispatch,
     'dispatch_target',x.dispatch_target,'control_points',x.control_points)=provenance) THEN
    RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
   END IF;
  END LOOP;
 END LOOP;
 SELECT jsonb_agg(jsonb_build_object('org_id',a.org_id::text,'object_type_id',a.object_type_id::text,
  'action_type_id',a.action_type_id::text,'registration_revision',a.registration_revision::text,
  'manifest_digest',encode(a.manifest_digest,'hex')) ORDER BY a.org_id,a.object_type_id,a.action_type_id,a.registration_revision,a.manifest_digest)
  INTO action_array FROM public.native_company_action_refs a WHERE a.org_id=b.org_id;
 SELECT jsonb_agg(jsonb_build_object('org_id',p.org_id::text,'object_type_id',p.object_type_id::text,
  'property_id',p.property_id::text,'schema_revision',p.schema_revision::text) ORDER BY p.org_id,p.object_type_id,p.property_id,p.schema_revision)
  INTO property_array FROM public.native_company_property_refs p WHERE p.org_id=b.org_id;
 IF action_array IS DISTINCT FROM r.action_refs OR property_array IS DISTINCT FROM r.property_refs THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 IF (SELECT count(*) FROM public.audit_events a WHERE a.org_id=b.org_id)<>5
  OR (SELECT count(*) FROM public.audit_events a WHERE a.org_id=b.org_id AND a.action='company.enroll')<>1
  OR (SELECT count(*) FROM public.audit_events a WHERE a.org_id=b.org_id AND a.action='ontology.object_type.builtin_install')<>2
  OR (SELECT count(*) FROM public.audit_events a WHERE a.org_id=b.org_id AND a.action='ontology.object_policy.attach')<>2 THEN
  RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
 END IF;
 FOR audit IN SELECT a.actor,a.action,a.target_type,a.target_id,a.branch_id,a.before_snap,a.after_snap,a.trace_id,a.span_id,a.occurred_at FROM public.audit_events a WHERE a.org_id=b.org_id LOOP
  IF audit.actor IS DISTINCT FROM b.account_id OR audit.occurred_at IS DISTINCT FROM b.started_at OR audit.before_snap IS NOT NULL OR audit.branch_id IS NOT NULL
   OR audit.after_snap->'enrollment' IS DISTINCT FROM jsonb_build_object('account_id',b.account_id::text,
    'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)
   OR audit.trace_id IS NULL OR audit.trace_id !~ '^[0-9a-f]{32}$' OR audit.trace_id=repeat('0',32)
   OR audit.span_id IS NULL OR audit.span_id !~ '^[0-9a-f]{16}$' OR audit.span_id=repeat('0',16)
   OR (trace IS NOT NULL AND audit.trace_id IS DISTINCT FROM trace) OR (span IS NOT NULL AND audit.span_id IS DISTINCT FROM span) THEN
   RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
  END IF;
  trace:=audit.trace_id; span:=audit.span_id;
  IF audit.action='company.enroll' THEN
   IF audit.target_type<>'organizations' OR audit.target_id<>b.org_id::text THEN RAISE EXCEPTION 'company_enrollment.birth_closure_invalid'; END IF;
   expected_snapshot:=jsonb_build_object('org_id',b.org_id::text,'group_id',b.group_id::text,'administrative_account_id',b.administrative_account_id::text);
  ELSE
   SELECT o.id,o.org_id,o.stable_key,o.title,o.title_property_key,o.backing_kind,o.backing_table,o.primary_key_property,o.schema_version,o.lifecycle_state,o.created_by,o.created_at,o.updated_at,o.attribution_protocol,o.created_by_account_id,o.origin_account_id,o.origin_command_id,o.origin_receipt_id INTO STRICT object_row FROM public.ont_object_types o WHERE o.org_id=b.org_id AND o.id::text=audit.target_id;
   IF (SELECT count(*) FROM public.audit_events a WHERE a.org_id=b.org_id AND a.action=audit.action AND a.target_id=audit.target_id)<>1 THEN
    RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
   END IF;
   IF audit.action='ontology.object_type.builtin_install' THEN
    IF audit.target_type<>'ont_object_types' THEN RAISE EXCEPTION 'company_enrollment.birth_closure_invalid'; END IF;
    expected_snapshot:=jsonb_build_object('stable_key',object_row.stable_key,'schema_version',1,'lifecycle_state','published',
     'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
   ELSE
    IF audit.target_type<>'ont_object_policies' THEN RAISE EXCEPTION 'company_enrollment.birth_closure_invalid'; END IF;
    expected_snapshot:=jsonb_build_object('effect','forbid','action','view','resource_type',object_row.stable_key,'conditions','[]'::jsonb);
    IF NOT EXISTS(SELECT 1 FROM public.ont_object_policies p JOIN public.cedar_policy_catalog_entries c ON c.org_id=p.org_id AND c.id=p.cedar_policy_id
     WHERE p.org_id=b.org_id AND p.object_type_id=object_row.id AND p.effect='forbid' AND c.effect='forbid'
      AND c.normalized_row=expected_snapshot AND c.generated_policy_text IS NULL AND c.status='enforced'
      AND c.schema_version='2026-07-ontology-authoring-v1' AND c.bundle_digest='sha256:'||encode(sha256(convert_to(expected_snapshot::text,'UTF8')),'hex')) THEN
     RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
    END IF;
   END IF;
  END IF;
  IF audit.after_snap-'enrollment' IS DISTINCT FROM expected_snapshot THEN RAISE EXCEPTION 'company_enrollment.birth_closure_invalid'; END IF;
 END LOOP;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'company_enrollment.birth_closure_invalid';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;


-- source: intake-guards.sql
CREATE OR REPLACE FUNCTION public.company_enrollment_request_guard_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE candidate record; binding record;
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
        IF NEW.state='COMMITTED' THEN
            SELECT * INTO STRICT binding FROM public.company_enrollment_binding_v1(NEW.account_id,NEW.command_id);
            IF OLD.state IS DISTINCT FROM 'PENDING' OR NEW.input_bytes IS DISTINCT FROM OLD.input_bytes
                OR NEW.committed_receipt_id IS DISTINCT FROM binding.receipt_id
                OR NEW.terminal_at IS DISTINCT FROM binding.started_at
                OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_receipts r
                    WHERE r.account_id=NEW.account_id AND r.command_id=NEW.command_id
                        AND r.receipt_id=binding.receipt_id AND r.committed_at=binding.started_at) THEN
                RAISE EXCEPTION 'company_enrollment.request_transition_invalid';
            END IF;
            RETURN NEW;
        END IF;
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

CREATE OR REPLACE FUNCTION public.company_enrollment_event_guard_v1()
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
            AND NEW.from_state='PENDING' AND NEW.to_state IN ('COMMITTED','CANCELLED','EXPIRED')
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

CREATE OR REPLACE FUNCTION public.company_enrollment_receipt_intake_guard_v1()
RETURNS trigger LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
    IF TG_TABLE_SCHEMA<>'public' OR TG_TABLE_NAME<>'company_enrollment_receipts'
        OR TG_LEVEL<>'STATEMENT' OR TG_WHEN<>'BEFORE' OR TG_OP<>'INSERT' THEN
        RAISE EXCEPTION 'company_enrollment.guard_context_invalid';
    END IF;
    IF current_user<>'console_account_owner' THEN
        RAISE EXCEPTION 'company_enrollment.writer_required';
    END IF;
    -- The immutable receipt FK and deferred complete physical closure validate
    -- every inserted row; zero-row statements confer no authority.
    RETURN NULL;
END
$body$;

CREATE OR REPLACE FUNCTION public.company_enrollment_intake_closure_v1()
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
    IF request.state='COMMITTED' THEN
        PERFORM public.company_enrollment_assert_closure_v1(request.account_id,request.command_id);
        RETURN NEW;
    END IF;
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


-- source: intake-owners-fence-successor.sql
CREATE OR REPLACE FUNCTION public.company_enrollment_prepare_v1(
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
    committed public.company_enrollment_receipts%ROWTYPE;
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
    BEGIN
     PERFORM pg_advisory_xact_lock(hashtextextended(
        'console.company.enrollment.command/1:'||p_account::text||':'||p_command::text,0));
    EXCEPTION WHEN lock_not_available THEN
     RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='company_enrollment.outcome_unknown';
    END;
    SELECT r.* INTO request FROM public.company_enrollment_requests r
        WHERE r.account_id=p_account AND r.command_id=p_command;
    IF request.account_id IS NOT NULL THEN
        IF request.input_digest IS DISTINCT FROM candidate.input_digest THEN
            RAISE EXCEPTION 'company_enrollment.conflict';
        END IF;
    observed_at := clock_timestamp();
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
    IF request.state='COMMITTED' THEN
        SELECT r.* INTO STRICT committed FROM public.company_enrollment_receipts r
            WHERE r.account_id=request.account_id AND r.command_id=request.command_id
                AND r.receipt_id=request.committed_receipt_id AND r.codec_version=request.codec_version
                AND r.input_digest=request.input_digest AND r.designation_receipt_id=request.designation_receipt_id
                AND r.committed_at=request.terminal_at;
        RETURN QUERY SELECT request.state,committed.receipt_id;
        RETURN;
    END IF;
        RETURN QUERY SELECT request.state,NULL::uuid;
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

CREATE OR REPLACE FUNCTION public.company_enrollment_status_v1(p_account uuid,p_family uuid,p_command uuid)
RETURNS TABLE(state text,codec_version smallint,input_bytes bytea,receipt_id uuid,org_id uuid,group_id uuid,administrative_account_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='1s'
AS $body$
DECLARE
    request public.company_enrollment_requests%ROWTYPE;
    committed public.company_enrollment_receipts%ROWTYPE;
    observed_at timestamptz;
BEGIN
    PERFORM 1 FROM public.company_enrollment_session_material_v1(p_account,p_family,p_command,NULL);
    BEGIN
     PERFORM pg_advisory_xact_lock(hashtextextended(
        'console.company.enrollment.command/1:'||p_account::text||':'||p_command::text,0));
    EXCEPTION WHEN lock_not_available THEN
     RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='company_enrollment.outcome_unknown';
    END;
    SELECT r.* INTO request FROM public.company_enrollment_requests r
        WHERE r.account_id=p_account AND r.command_id=p_command;
    IF NOT FOUND THEN RETURN; END IF;
    observed_at := clock_timestamp();
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
    IF request.state='COMMITTED' THEN
        SELECT r.* INTO STRICT committed FROM public.company_enrollment_receipts r
            WHERE r.account_id=request.account_id AND r.command_id=request.command_id
                AND r.receipt_id=request.committed_receipt_id AND r.codec_version=request.codec_version
                AND r.input_digest=request.input_digest AND r.designation_receipt_id=request.designation_receipt_id
                AND r.committed_at=request.terminal_at;
        RETURN QUERY SELECT request.state,request.codec_version,NULL::bytea,committed.receipt_id,committed.org_id,committed.group_id,committed.administrative_account_id;
        RETURN;
    END IF;
    RETURN QUERY SELECT request.state,request.codec_version,CASE WHEN request.state='PENDING' THEN request.input_bytes ELSE NULL::bytea END,NULL::uuid,NULL::uuid,NULL::uuid,NULL::uuid;
END
$body$;

CREATE OR REPLACE FUNCTION public.company_enrollment_cancel_v1(p_account uuid,p_family uuid,p_command uuid)
RETURNS TABLE(state text,receipt_id uuid,org_id uuid,group_id uuid,administrative_account_id uuid)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on SET lock_timeout='1s'
AS $body$
DECLARE
    request public.company_enrollment_requests%ROWTYPE;
    committed public.company_enrollment_receipts%ROWTYPE;
    observed_at timestamptz;
BEGIN
    PERFORM 1 FROM public.company_enrollment_session_material_v1(p_account,p_family,p_command,NULL);
    BEGIN
     PERFORM pg_advisory_xact_lock(hashtextextended(
        'console.company.enrollment.command/1:'||p_account::text||':'||p_command::text,0));
    EXCEPTION WHEN lock_not_available THEN
     RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='company_enrollment.outcome_unknown';
    END;
    SELECT r.* INTO request FROM public.company_enrollment_requests r
        WHERE r.account_id=p_account AND r.command_id=p_command;
    IF NOT FOUND THEN RETURN; END IF;
    observed_at := clock_timestamp();
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
    IF request.state='COMMITTED' THEN
        SELECT r.* INTO STRICT committed FROM public.company_enrollment_receipts r
            WHERE r.account_id=request.account_id AND r.command_id=request.command_id
                AND r.receipt_id=request.committed_receipt_id AND r.codec_version=request.codec_version
                AND r.input_digest=request.input_digest AND r.designation_receipt_id=request.designation_receipt_id
                AND r.committed_at=request.terminal_at;
        RETURN QUERY SELECT request.state,committed.receipt_id,committed.org_id,committed.group_id,committed.administrative_account_id;
        RETURN;
    END IF;
    IF request.state='PENDING' THEN
        UPDATE public.company_enrollment_requests r SET state='CANCELLED',terminal_at=observed_at
            WHERE r.account_id=p_account AND r.command_id=p_command;
        INSERT INTO public.company_enrollment_request_events
            (account_id,command_id,event_revision,from_state,to_state,occurred_at,actor_account_id,session_id,reason_code)
            VALUES(p_account,p_command,2,'PENDING','CANCELLED',observed_at,p_account,p_family,'CANCELLED');
        request.state := 'CANCELLED';
    END IF;
    RETURN QUERY SELECT request.state,NULL::uuid,NULL::uuid,NULL::uuid,NULL::uuid;
END
$body$;


-- source: validate-backfill-constraints
-- Drain only the populated structural backfill checks before table metadata DDL.
-- Restore initial deferral so subsequent birth/history writes remain receipt-last.
SET CONSTRAINTS public.group_history_native_receipt_fk,
 public.group_history_legacy_receipt_fk,public.group_history_force_receipt_fk,public.group_memberships_current_history_fk IMMEDIATE;
SET CONSTRAINTS public.group_history_native_receipt_fk,
 public.group_history_legacy_receipt_fk,public.group_history_force_receipt_fk,public.group_memberships_current_history_fk DEFERRED;

-- source: catalog-install-metadata.sql
ALTER TABLE public.policy_role_revisions OWNER TO console_account_owner;
REVOKE ALL ON public.policy_role_revisions FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.policy_role_revisions'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.policy_role_revisions FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.policy_role_revisions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.policy_role_revisions FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.policy_role_revisions USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.policy_role_revisions TO console_account_owner;
ALTER TABLE public.policy_capability_clauses OWNER TO console_account_owner;
REVOKE ALL ON public.policy_capability_clauses FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.policy_capability_clauses'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.policy_capability_clauses FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.policy_capability_clauses ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.policy_capability_clauses FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.policy_capability_clauses USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.policy_capability_clauses TO console_account_owner;
ALTER TABLE public.policy_capability_clause_fields OWNER TO console_account_owner;
REVOKE ALL ON public.policy_capability_clause_fields FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.policy_capability_clause_fields'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.policy_capability_clause_fields FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.policy_capability_clause_fields ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.policy_capability_clause_fields FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.policy_capability_clause_fields USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.policy_capability_clause_fields TO console_account_owner;
ALTER TABLE public.policy_assignment_revisions OWNER TO console_account_owner;
REVOKE ALL ON public.policy_assignment_revisions FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.policy_assignment_revisions'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.policy_assignment_revisions FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.policy_assignment_revisions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.policy_assignment_revisions FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.policy_assignment_revisions USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.policy_assignment_revisions TO console_account_owner;
ALTER TABLE public.company_authority_heads OWNER TO console_account_owner;
REVOKE ALL ON public.company_authority_heads FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.company_authority_heads'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.company_authority_heads FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.company_authority_heads ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.company_authority_heads FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.company_authority_heads USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.company_authority_heads TO console_account_owner;
ALTER TABLE public.native_company_catalog_installs OWNER TO console_ontology_writer;
REVOKE ALL ON public.native_company_catalog_installs FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.native_company_catalog_installs'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.native_company_catalog_installs FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.native_company_catalog_installs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_catalog_installs FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.native_company_catalog_installs USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.native_company_catalog_installs TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.native_company_catalog_installs TO console_account_owner;
ALTER TABLE public.native_company_object_refs OWNER TO console_ontology_writer;
REVOKE ALL ON public.native_company_object_refs FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.native_company_object_refs'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.native_company_object_refs FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.native_company_object_refs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_object_refs FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.native_company_object_refs USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.native_company_object_refs TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.native_company_object_refs TO console_account_owner;
ALTER TABLE public.native_company_action_refs OWNER TO console_ontology_writer;
REVOKE ALL ON public.native_company_action_refs FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.native_company_action_refs'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.native_company_action_refs FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.native_company_action_refs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_action_refs FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.native_company_action_refs USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.native_company_action_refs TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.native_company_action_refs TO console_account_owner;
ALTER TABLE public.native_company_property_refs OWNER TO console_ontology_writer;
REVOKE ALL ON public.native_company_property_refs FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.native_company_property_refs'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.native_company_property_refs FROM %I',grantee_name);
 END LOOP;
END
$acl$;
ALTER TABLE public.native_company_property_refs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_property_refs FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.native_company_property_refs USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid) WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
GRANT INSERT,REFERENCES,SELECT ON public.native_company_property_refs TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.native_company_property_refs TO console_account_owner;
ALTER TABLE public.company_enrollment_effect_bindings OWNER TO console_account_owner;
REVOKE ALL ON public.company_enrollment_effect_bindings FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.company_enrollment_effect_bindings'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.company_enrollment_effect_bindings FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT INSERT,REFERENCES,SELECT ON public.company_enrollment_effect_bindings TO console_account_owner;
ALTER TABLE public.group_authority_heads OWNER TO console_app;
REVOKE ALL ON public.group_authority_heads FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.group_authority_heads'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.group_authority_heads FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT DELETE,INSERT,REFERENCES,SELECT ON public.group_authority_heads TO console_app;
GRANT SELECT ON public.group_authority_heads TO console_account_owner;
ALTER TABLE public.group_membership_revisions OWNER TO console_app;
REVOKE ALL ON public.group_membership_revisions FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c CROSS JOIN LATERAL aclexplode(c.relacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE c.oid='public.group_membership_revisions'::regclass LOOP
  EXECUTE format('REVOKE ALL ON public.group_membership_revisions FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT INSERT,REFERENCES,SELECT ON public.group_membership_revisions TO console_app;
GRANT SELECT ON public.group_membership_revisions TO console_account_owner;
GRANT REFERENCES,SELECT ON public.policy_roles TO console_account_owner;
GRANT REFERENCES,SELECT ON public.user_role_assignments TO console_account_owner;
GRANT REFERENCES ON public.company_enrollment_receipts TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.ont_builtin_catalog_installs TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.ont_object_types TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.ont_action_types TO console_ontology_writer;
GRANT REFERENCES,SELECT ON public.ont_property_defs TO console_ontology_writer;
GRANT UPDATE("epoch") ON public.company_authority_heads TO console_account_owner;
GRANT INSERT("id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("org_id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("role_key") ON public.policy_roles TO console_account_owner;
GRANT INSERT("display_name") ON public.policy_roles TO console_account_owner;
GRANT INSERT("description") ON public.policy_roles TO console_account_owner;
GRANT INSERT("status") ON public.policy_roles TO console_account_owner;
GRANT INSERT("is_system") ON public.policy_roles TO console_account_owner;
GRANT INSERT("created_at") ON public.policy_roles TO console_account_owner;
GRANT INSERT("updated_at") ON public.policy_roles TO console_account_owner;
GRANT INSERT("subject_protocol") ON public.policy_roles TO console_account_owner;
GRANT INSERT("native_current_revision") ON public.policy_roles TO console_account_owner;
GRANT UPDATE("native_current_revision") ON public.policy_roles TO console_account_owner;
GRANT INSERT("created_by_account_id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("updated_by_account_id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("origin_account_id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("origin_command_id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("origin_receipt_id") ON public.policy_roles TO console_account_owner;
GRANT INSERT("id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("org_id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("role_id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("created_at") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("subject_protocol") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("account_id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("native_current_revision") ON public.user_role_assignments TO console_account_owner;
GRANT UPDATE("native_current_revision") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("assigned_by_account_id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("origin_account_id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("origin_command_id") ON public.user_role_assignments TO console_account_owner;
GRANT INSERT("origin_receipt_id") ON public.user_role_assignments TO console_account_owner;
GRANT UPDATE("revision","state") ON public.group_authority_heads TO console_app;
GRANT REFERENCES("account_id","command_id","receipt_id") ON public.company_enrollment_receipts TO console_app;
GRANT SELECT("id","group_id","slug","name","status","created_at","updated_at","origin_account_id","origin_command_id","origin_receipt_id") ON public.organizations TO console_account_owner;
GRANT SELECT("id","slug","name","status","created_at","updated_at","origin_account_id","origin_command_id","origin_receipt_id") ON public.groups TO console_account_owner;
GRANT SELECT("group_id","org_id","created_at","membership_id","current_revision","incarnation") ON public.group_memberships TO console_account_owner;
CREATE TRIGGER policy_roles_native_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.policy_roles FOR EACH ROW EXECUTE FUNCTION public.identity_native_root_guard_v1();
ALTER TABLE public.policy_roles ENABLE ALWAYS TRIGGER policy_roles_native_guard_v1;
CREATE CONSTRAINT TRIGGER policy_roles_native_birth_closure_v1 AFTER INSERT ON public.policy_roles DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.policy_roles ENABLE ALWAYS TRIGGER policy_roles_native_birth_closure_v1;
CREATE TRIGGER user_role_assignments_native_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.user_role_assignments FOR EACH ROW EXECUTE FUNCTION public.identity_native_root_guard_v1();
ALTER TABLE public.user_role_assignments ENABLE ALWAYS TRIGGER user_role_assignments_native_guard_v1;
CREATE CONSTRAINT TRIGGER user_role_assignments_native_birth_closure_v1 AFTER INSERT ON public.user_role_assignments DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.user_role_assignments ENABLE ALWAYS TRIGGER user_role_assignments_native_birth_closure_v1;
CREATE TRIGGER policy_role_permissions_native_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.policy_role_permissions FOR EACH ROW EXECUTE FUNCTION public.identity_native_legacy_child_guard_v1();
ALTER TABLE public.policy_role_permissions ENABLE ALWAYS TRIGGER policy_role_permissions_native_guard_v1;
CREATE TRIGGER policy_role_conditions_native_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.policy_role_conditions FOR EACH ROW EXECUTE FUNCTION public.identity_native_legacy_child_guard_v1();
ALTER TABLE public.policy_role_conditions ENABLE ALWAYS TRIGGER policy_role_conditions_native_guard_v1;
CREATE TRIGGER policy_roles_native_truncate_v1 BEFORE TRUNCATE ON public.policy_roles FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_truncate_guard_v1();
ALTER TABLE public.policy_roles ENABLE ALWAYS TRIGGER policy_roles_native_truncate_v1;
CREATE TRIGGER user_role_assignments_native_truncate_v1 BEFORE TRUNCATE ON public.user_role_assignments FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_truncate_guard_v1();
ALTER TABLE public.user_role_assignments ENABLE ALWAYS TRIGGER user_role_assignments_native_truncate_v1;
CREATE TRIGGER policy_role_permissions_native_truncate_v1 BEFORE TRUNCATE ON public.policy_role_permissions FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_truncate_guard_v1();
ALTER TABLE public.policy_role_permissions ENABLE ALWAYS TRIGGER policy_role_permissions_native_truncate_v1;
CREATE TRIGGER policy_role_conditions_native_truncate_v1 BEFORE TRUNCATE ON public.policy_role_conditions FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_truncate_guard_v1();
ALTER TABLE public.policy_role_conditions ENABLE ALWAYS TRIGGER policy_role_conditions_native_truncate_v1;
CREATE TRIGGER policy_role_revisions_birth_row_v1 BEFORE INSERT ON public.policy_role_revisions FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_row_guard_v1();
ALTER TABLE public.policy_role_revisions ENABLE ALWAYS TRIGGER policy_role_revisions_birth_row_v1;
CREATE TRIGGER policy_role_revisions_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.policy_role_revisions FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
ALTER TABLE public.policy_role_revisions ENABLE ALWAYS TRIGGER policy_role_revisions_immutable_v1;
CREATE CONSTRAINT TRIGGER policy_role_revisions_birth_closure_v1 AFTER INSERT ON public.policy_role_revisions DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.policy_role_revisions ENABLE ALWAYS TRIGGER policy_role_revisions_birth_closure_v1;
CREATE TRIGGER policy_capability_clauses_birth_row_v1 BEFORE INSERT ON public.policy_capability_clauses FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_row_guard_v1();
ALTER TABLE public.policy_capability_clauses ENABLE ALWAYS TRIGGER policy_capability_clauses_birth_row_v1;
CREATE TRIGGER policy_capability_clauses_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.policy_capability_clauses FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
ALTER TABLE public.policy_capability_clauses ENABLE ALWAYS TRIGGER policy_capability_clauses_immutable_v1;
CREATE CONSTRAINT TRIGGER policy_capability_clauses_birth_closure_v1 AFTER INSERT ON public.policy_capability_clauses DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.policy_capability_clauses ENABLE ALWAYS TRIGGER policy_capability_clauses_birth_closure_v1;
CREATE TRIGGER policy_capability_clause_fields_birth_row_v1 BEFORE INSERT ON public.policy_capability_clause_fields FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_row_guard_v1();
ALTER TABLE public.policy_capability_clause_fields ENABLE ALWAYS TRIGGER policy_capability_clause_fields_birth_row_v1;
CREATE TRIGGER policy_capability_clause_fields_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.policy_capability_clause_fields FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
ALTER TABLE public.policy_capability_clause_fields ENABLE ALWAYS TRIGGER policy_capability_clause_fields_immutable_v1;
CREATE CONSTRAINT TRIGGER policy_capability_clause_fields_birth_closure_v1 AFTER INSERT ON public.policy_capability_clause_fields DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.policy_capability_clause_fields ENABLE ALWAYS TRIGGER policy_capability_clause_fields_birth_closure_v1;
CREATE TRIGGER policy_assignment_revisions_birth_row_v1 BEFORE INSERT ON public.policy_assignment_revisions FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_row_guard_v1();
ALTER TABLE public.policy_assignment_revisions ENABLE ALWAYS TRIGGER policy_assignment_revisions_birth_row_v1;
CREATE TRIGGER policy_assignment_revisions_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.policy_assignment_revisions FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
ALTER TABLE public.policy_assignment_revisions ENABLE ALWAYS TRIGGER policy_assignment_revisions_immutable_v1;
CREATE CONSTRAINT TRIGGER policy_assignment_revisions_birth_closure_v1 AFTER INSERT ON public.policy_assignment_revisions DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.policy_assignment_revisions ENABLE ALWAYS TRIGGER policy_assignment_revisions_birth_closure_v1;
CREATE TRIGGER company_authority_heads_birth_row_v1 BEFORE INSERT ON public.company_authority_heads FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_row_guard_v1();
ALTER TABLE public.company_authority_heads ENABLE ALWAYS TRIGGER company_authority_heads_birth_row_v1;
CREATE TRIGGER company_authority_heads_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.company_authority_heads FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
ALTER TABLE public.company_authority_heads ENABLE ALWAYS TRIGGER company_authority_heads_immutable_v1;
CREATE CONSTRAINT TRIGGER company_authority_heads_birth_closure_v1 AFTER INSERT ON public.company_authority_heads DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_native_birth_closure_v1();
ALTER TABLE public.company_authority_heads ENABLE ALWAYS TRIGGER company_authority_heads_birth_closure_v1;
CREATE TRIGGER native_company_catalog_installs_birth_row_v1 BEFORE INSERT ON public.native_company_catalog_installs FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_row_guard_v1();
ALTER TABLE public.native_company_catalog_installs ENABLE ALWAYS TRIGGER native_company_catalog_installs_birth_row_v1;
CREATE TRIGGER native_company_catalog_installs_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_company_catalog_installs FOR EACH STATEMENT EXECUTE FUNCTION public.native_company_catalog_immutable_v1();
ALTER TABLE public.native_company_catalog_installs ENABLE ALWAYS TRIGGER native_company_catalog_installs_immutable_v1;
CREATE CONSTRAINT TRIGGER native_company_catalog_installs_birth_closure_v1 AFTER INSERT ON public.native_company_catalog_installs DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_closure_v1();
ALTER TABLE public.native_company_catalog_installs ENABLE ALWAYS TRIGGER native_company_catalog_installs_birth_closure_v1;
CREATE TRIGGER native_company_object_refs_birth_row_v1 BEFORE INSERT ON public.native_company_object_refs FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_row_guard_v1();
ALTER TABLE public.native_company_object_refs ENABLE ALWAYS TRIGGER native_company_object_refs_birth_row_v1;
CREATE TRIGGER native_company_object_refs_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_company_object_refs FOR EACH STATEMENT EXECUTE FUNCTION public.native_company_catalog_immutable_v1();
ALTER TABLE public.native_company_object_refs ENABLE ALWAYS TRIGGER native_company_object_refs_immutable_v1;
CREATE CONSTRAINT TRIGGER native_company_object_refs_birth_closure_v1 AFTER INSERT ON public.native_company_object_refs DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_closure_v1();
ALTER TABLE public.native_company_object_refs ENABLE ALWAYS TRIGGER native_company_object_refs_birth_closure_v1;
CREATE TRIGGER native_company_action_refs_birth_row_v1 BEFORE INSERT ON public.native_company_action_refs FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_row_guard_v1();
ALTER TABLE public.native_company_action_refs ENABLE ALWAYS TRIGGER native_company_action_refs_birth_row_v1;
CREATE TRIGGER native_company_action_refs_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_company_action_refs FOR EACH STATEMENT EXECUTE FUNCTION public.native_company_catalog_immutable_v1();
ALTER TABLE public.native_company_action_refs ENABLE ALWAYS TRIGGER native_company_action_refs_immutable_v1;
CREATE CONSTRAINT TRIGGER native_company_action_refs_birth_closure_v1 AFTER INSERT ON public.native_company_action_refs DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_closure_v1();
ALTER TABLE public.native_company_action_refs ENABLE ALWAYS TRIGGER native_company_action_refs_birth_closure_v1;
CREATE TRIGGER native_company_property_refs_birth_row_v1 BEFORE INSERT ON public.native_company_property_refs FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_row_guard_v1();
ALTER TABLE public.native_company_property_refs ENABLE ALWAYS TRIGGER native_company_property_refs_birth_row_v1;
CREATE TRIGGER native_company_property_refs_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.native_company_property_refs FOR EACH STATEMENT EXECUTE FUNCTION public.native_company_catalog_immutable_v1();
ALTER TABLE public.native_company_property_refs ENABLE ALWAYS TRIGGER native_company_property_refs_immutable_v1;
CREATE CONSTRAINT TRIGGER native_company_property_refs_birth_closure_v1 AFTER INSERT ON public.native_company_property_refs DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_company_catalog_birth_closure_v1();
ALTER TABLE public.native_company_property_refs ENABLE ALWAYS TRIGGER native_company_property_refs_birth_closure_v1;
CREATE TRIGGER company_effect_binding_insert_v1 BEFORE INSERT ON public.company_enrollment_effect_bindings FOR EACH ROW EXECUTE FUNCTION public.company_effect_binding_guard_v1();
ALTER TABLE public.company_enrollment_effect_bindings ENABLE ALWAYS TRIGGER company_effect_binding_insert_v1;
CREATE TRIGGER company_enrollment_effect_bindings_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.company_enrollment_effect_bindings FOR EACH STATEMENT EXECUTE FUNCTION public.company_topology_history_immutable_v1();
ALTER TABLE public.company_enrollment_effect_bindings ENABLE ALWAYS TRIGGER company_enrollment_effect_bindings_immutable_v1;
CREATE TRIGGER group_membership_revisions_immutable_v1 BEFORE UPDATE OR DELETE OR TRUNCATE ON public.group_membership_revisions FOR EACH STATEMENT EXECUTE FUNCTION public.company_topology_history_immutable_v1();
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER group_membership_revisions_immutable_v1;
CREATE TRIGGER group_history_insert_v1 BEFORE INSERT ON public.group_membership_revisions FOR EACH ROW EXECUTE FUNCTION public.company_topology_write_guard_v1();
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER group_history_insert_v1;
CREATE TRIGGER organizations_topology_write_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.organizations FOR EACH ROW EXECUTE FUNCTION public.company_topology_write_guard_v1();
ALTER TABLE public.organizations ENABLE ALWAYS TRIGGER organizations_topology_write_v1;
CREATE TRIGGER organizations_topology_truncate_v1 BEFORE TRUNCATE ON public.organizations FOR EACH STATEMENT EXECUTE FUNCTION public.company_topology_truncate_guard_v1();
ALTER TABLE public.organizations ENABLE ALWAYS TRIGGER organizations_topology_truncate_v1;
CREATE TRIGGER groups_topology_write_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.groups FOR EACH ROW EXECUTE FUNCTION public.company_topology_write_guard_v1();
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER groups_topology_write_v1;
CREATE TRIGGER groups_topology_truncate_v1 BEFORE TRUNCATE ON public.groups FOR EACH STATEMENT EXECUTE FUNCTION public.company_topology_truncate_guard_v1();
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER groups_topology_truncate_v1;
CREATE TRIGGER group_memberships_topology_write_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_memberships FOR EACH ROW EXECUTE FUNCTION public.company_topology_write_guard_v1();
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER group_memberships_topology_write_v1;
CREATE TRIGGER group_memberships_topology_truncate_v1 BEFORE TRUNCATE ON public.group_memberships FOR EACH STATEMENT EXECUTE FUNCTION public.company_topology_truncate_guard_v1();
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER group_memberships_topology_truncate_v1;
CREATE TRIGGER group_authority_heads_topology_write_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_authority_heads FOR EACH ROW EXECUTE FUNCTION public.company_topology_write_guard_v1();
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER group_authority_heads_topology_write_v1;
CREATE TRIGGER group_authority_heads_topology_truncate_v1 BEFORE TRUNCATE ON public.group_authority_heads FOR EACH STATEMENT EXECUTE FUNCTION public.company_topology_truncate_guard_v1();
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER group_authority_heads_topology_truncate_v1;
CREATE CONSTRAINT TRIGGER company_enrollment_effect_bindings_native_birth_closure_v1 AFTER INSERT ON public.company_enrollment_effect_bindings DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.company_native_topology_birth_closure_v1();
ALTER TABLE public.company_enrollment_effect_bindings ENABLE ALWAYS TRIGGER company_enrollment_effect_bindings_native_birth_closure_v1;
CREATE CONSTRAINT TRIGGER organizations_native_birth_closure_v1 AFTER INSERT ON public.organizations DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.company_native_topology_birth_closure_v1();
ALTER TABLE public.organizations ENABLE ALWAYS TRIGGER organizations_native_birth_closure_v1;
CREATE CONSTRAINT TRIGGER groups_native_birth_closure_v1 AFTER INSERT ON public.groups DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.company_native_topology_birth_closure_v1();
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER groups_native_birth_closure_v1;
CREATE CONSTRAINT TRIGGER group_memberships_native_birth_closure_v1 AFTER INSERT ON public.group_memberships DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.company_native_topology_birth_closure_v1();
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER group_memberships_native_birth_closure_v1;
CREATE CONSTRAINT TRIGGER group_authority_heads_native_birth_closure_v1 AFTER INSERT ON public.group_authority_heads DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.company_native_topology_birth_closure_v1();
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER group_authority_heads_native_birth_closure_v1;
CREATE CONSTRAINT TRIGGER group_membership_revisions_native_birth_closure_v1 AFTER INSERT ON public.group_membership_revisions DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.company_native_topology_birth_closure_v1();
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER group_membership_revisions_native_birth_closure_v1;
ALTER FUNCTION public.identity_native_root_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_root_guard_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_root_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_root_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_root_guard_v1() TO console_account_owner;
ALTER FUNCTION public.identity_native_legacy_child_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_legacy_child_guard_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_legacy_child_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_legacy_child_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_legacy_child_guard_v1() TO console_account_owner;
ALTER FUNCTION public.identity_native_truncate_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_truncate_guard_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_truncate_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_truncate_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_truncate_guard_v1() TO console_account_owner;
ALTER FUNCTION public.identity_native_birth_row_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_birth_row_guard_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_birth_row_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_birth_row_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_birth_row_guard_v1() TO console_account_owner;
ALTER FUNCTION public.identity_native_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_immutable_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_immutable_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_immutable_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_immutable_v1() TO console_account_owner;
ALTER FUNCTION public.identity_native_birth_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_native_birth_closure_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_birth_closure_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_birth_closure_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_birth_closure_v1() TO console_account_owner;
ALTER FUNCTION public.native_company_catalog_birth_row_guard_v1() OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION public.native_company_catalog_birth_row_guard_v1() FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_catalog_birth_row_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_catalog_birth_row_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.native_company_catalog_birth_row_guard_v1() TO console_ontology_writer;
ALTER FUNCTION public.native_company_catalog_immutable_v1() OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION public.native_company_catalog_immutable_v1() FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_catalog_immutable_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_catalog_immutable_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.native_company_catalog_immutable_v1() TO console_ontology_writer;
ALTER FUNCTION public.native_company_catalog_birth_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_catalog_birth_closure_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.native_company_catalog_birth_closure_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.native_company_catalog_birth_closure_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.native_company_catalog_birth_closure_v1() TO console_account_owner;
ALTER FUNCTION public.company_effect_binding_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_effect_binding_guard_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_effect_binding_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_effect_binding_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_effect_binding_guard_v1() TO console_account_owner;
ALTER FUNCTION public.company_topology_history_immutable_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_topology_history_immutable_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_topology_history_immutable_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_topology_history_immutable_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_topology_history_immutable_v1() TO console_account_owner;
ALTER FUNCTION public.company_topology_write_guard_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.company_topology_write_guard_v1() FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_topology_write_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_topology_write_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_topology_write_guard_v1() TO console_app;
ALTER FUNCTION public.company_topology_truncate_guard_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.company_topology_truncate_guard_v1() FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_topology_truncate_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_topology_truncate_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_topology_truncate_guard_v1() TO console_app;
ALTER FUNCTION public.company_native_topology_birth_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_native_topology_birth_closure_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_native_topology_birth_closure_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_native_topology_birth_closure_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_native_topology_birth_closure_v1() TO console_account_owner;
ALTER FUNCTION public.company_enrollment_binding_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_binding_v1(uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_binding_v1(uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_binding_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_binding_v1(uuid,uuid) TO console_account_owner,console_app,console_ontology_writer;
ALTER FUNCTION public.company_enrollment_topology_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.company_enrollment_topology_v1(uuid,uuid) FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_topology_v1(uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_topology_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_topology_v1(uuid,uuid) TO console_account_owner,console_app;
ALTER FUNCTION ontology_api.install_native_company_catalog_v1(uuid,uuid,text,text) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.install_native_company_catalog_v1(uuid,uuid,text,text) FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.install_native_company_catalog_v1(uuid,uuid,text,text)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.install_native_company_catalog_v1(uuid,uuid,text,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION ontology_api.install_native_company_catalog_v1(uuid,uuid,text,text) TO console_account_owner,console_ontology_writer;
ALTER FUNCTION ont_policy_api.install_native_company_policy_v1(uuid,uuid,text,text) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ont_policy_api.install_native_company_policy_v1(uuid,uuid,text,text) FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ont_policy_api.install_native_company_policy_v1(uuid,uuid,text,text)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ont_policy_api.install_native_company_policy_v1(uuid,uuid,text,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION ont_policy_api.install_native_company_policy_v1(uuid,uuid,text,text) TO console_account_owner,console_ontology_writer;
ALTER FUNCTION public.identity_enroll_company_administration_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_enroll_company_administration_v1(uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_enroll_company_administration_v1(uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_enroll_company_administration_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_enroll_company_administration_v1(uuid,uuid) TO console_account_owner;
ALTER FUNCTION public.company_enrollment_audit_v1(uuid,uuid,text,text) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_audit_v1(uuid,uuid,text,text) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_audit_v1(uuid,uuid,text,text)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_audit_v1(uuid,uuid,text,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_audit_v1(uuid,uuid,text,text) TO console_account_owner;
ALTER FUNCTION public.company_enrollment_assert_closure_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_assert_closure_v1(uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_assert_closure_v1(uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_assert_closure_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_assert_closure_v1(uuid,uuid) TO console_account_owner;
ALTER FUNCTION public.group_authority_lock_shared_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.group_authority_lock_shared_v1(uuid) FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.group_authority_lock_shared_v1(uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.group_authority_lock_shared_v1(uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.group_authority_lock_shared_v1(uuid) TO console_account_owner,console_app;
ALTER FUNCTION public.group_authority_lock_exclusive_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.group_authority_lock_exclusive_v1(uuid) FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.group_authority_lock_exclusive_v1(uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.group_authority_lock_exclusive_v1(uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.group_authority_lock_exclusive_v1(uuid) TO console_account_owner,console_app;
ALTER FUNCTION public.platform_create_organization_core_v1(uuid,uuid,text,text) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_create_organization_core_v1(uuid,uuid,text,text) FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.platform_create_organization_core_v1(uuid,uuid,text,text)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.platform_create_organization_core_v1(uuid,uuid,text,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.platform_create_organization_core_v1(uuid,uuid,text,text) TO console_app;
ALTER FUNCTION public.company_enrollment_execute_v1(uuid,uuid,uuid,bytea,text,text) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_execute_v1(uuid,uuid,uuid,bytea,text,text) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_execute_v1(uuid,uuid,uuid,bytea,text,text)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_execute_v1(uuid,uuid,uuid,bytea,text,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_execute_v1(uuid,uuid,uuid,bytea,text,text) TO console_account_owner,console_rt;
ALTER FUNCTION public.company_enrollment_prepare_v1(uuid,uuid,uuid,bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_prepare_v1(uuid,uuid,uuid,bytea) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_prepare_v1(uuid,uuid,uuid,bytea)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_prepare_v1(uuid,uuid,uuid,bytea) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_prepare_v1(uuid,uuid,uuid,bytea) TO console_account_owner,console_rt;
ALTER FUNCTION public.company_enrollment_status_v1(uuid,uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_status_v1(uuid,uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_status_v1(uuid,uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_status_v1(uuid,uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_status_v1(uuid,uuid,uuid) TO console_account_owner,console_rt;
ALTER FUNCTION public.company_enrollment_cancel_v1(uuid,uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_cancel_v1(uuid,uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_cancel_v1(uuid,uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_cancel_v1(uuid,uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_cancel_v1(uuid,uuid,uuid) TO console_account_owner,console_rt;
ALTER FUNCTION public.company_enrollment_session_material_v1(uuid,uuid,uuid,bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_session_material_v1(uuid,uuid,uuid,bytea) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_session_material_v1(uuid,uuid,uuid,bytea)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_session_material_v1(uuid,uuid,uuid,bytea) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_session_material_v1(uuid,uuid,uuid,bytea) TO console_account_owner,console_rt;
ALTER FUNCTION ontology_api.install_builtin_catalog_core_v1(uuid,text,jsonb,uuid,text,text,uuid) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.install_builtin_catalog_core_v1(uuid,text,jsonb,uuid,text,text,uuid) FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.install_builtin_catalog_core_v1(uuid,text,jsonb,uuid,text,text,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.install_builtin_catalog_core_v1(uuid,text,jsonb,uuid,text,text,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION ontology_api.install_builtin_catalog_core_v1(uuid,text,jsonb,uuid,text,text,uuid) TO console_ontology_writer;
ALTER FUNCTION ont_policy_api.attach_object_policy_rows_core_v1(uuid,uuid,uuid,text,jsonb,text,uuid) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ont_policy_api.attach_object_policy_rows_core_v1(uuid,uuid,uuid,text,jsonb,text,uuid) FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ont_policy_api.attach_object_policy_rows_core_v1(uuid,uuid,uuid,text,jsonb,text,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ont_policy_api.attach_object_policy_rows_core_v1(uuid,uuid,uuid,text,jsonb,text,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION ont_policy_api.attach_object_policy_rows_core_v1(uuid,uuid,uuid,text,jsonb,text,uuid) TO console_ontology_writer;
ALTER FUNCTION public.company_enrollment_ontology_audit_v1(uuid,uuid,text,uuid,text,text) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_ontology_audit_v1(uuid,uuid,text,uuid,text,text) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_ontology_audit_v1(uuid,uuid,text,uuid,text,text)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_ontology_audit_v1(uuid,uuid,text,uuid,text,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_ontology_audit_v1(uuid,uuid,text,uuid,text,text) TO console_account_owner,console_ontology_writer;
ALTER FUNCTION ontology_api.native_catalog_attribution_guard_v1() OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.native_catalog_attribution_guard_v1() FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.native_catalog_attribution_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.native_catalog_attribution_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION ontology_api.native_catalog_attribution_guard_v1() TO console_ontology_writer;
ALTER FUNCTION public.company_enrollment_ontology_audit_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_ontology_audit_guard_v1() FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_enrollment_ontology_audit_guard_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_enrollment_ontology_audit_guard_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_enrollment_ontology_audit_guard_v1() TO console_account_owner;
ALTER FUNCTION public.identity_native_any_origin_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.identity_native_any_origin_v1() FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_native_any_origin_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_native_any_origin_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_native_any_origin_v1() TO console_app,console_account_owner;
ALTER FUNCTION public.identity_company_projection_v1(uuid,uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_company_projection_v1(uuid,uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.identity_company_projection_v1(uuid,uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_company_projection_v1(uuid,uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_projection_v1(uuid,uuid,uuid) TO console_account_owner,console_rt;
ALTER FUNCTION public.account_company_context_candidates_v1(uuid,uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_company_context_candidates_v1(uuid,uuid) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.account_company_context_candidates_v1(uuid,uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.account_company_context_candidates_v1(uuid,uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.account_company_context_candidates_v1(uuid,uuid) TO console_account_owner,console_rt;
ALTER FUNCTION ontology_api.lock_native_company_catalog_current_v1(uuid) OWNER TO console_ontology_writer;
REVOKE ALL ON FUNCTION ontology_api.lock_native_company_catalog_current_v1(uuid) FROM PUBLIC,console_ontology_writer;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='ontology_api.lock_native_company_catalog_current_v1(uuid)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION ontology_api.lock_native_company_catalog_current_v1(uuid) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION ontology_api.lock_native_company_catalog_current_v1(uuid) TO console_account_owner,console_ontology_writer;
ALTER FUNCTION public.company_actor_entitlement_shape_v2(jsonb) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_actor_entitlement_shape_v2(jsonb) FROM PUBLIC,console_account_owner;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.company_actor_entitlement_shape_v2(jsonb)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.company_actor_entitlement_shape_v2(jsonb) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.company_actor_entitlement_shape_v2(jsonb) TO console_account_owner;
ALTER FUNCTION public.platform_legacy_topology_decode_input_v1(bytea) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_topology_decode_input_v1(bytea) FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.platform_legacy_topology_decode_input_v1(bytea)'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.platform_legacy_topology_decode_input_v1(bytea) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.platform_legacy_topology_decode_input_v1(bytea) TO console_app;
ALTER FUNCTION public.platform_legacy_topology_receipts_immutable_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_topology_receipts_immutable_v1() FROM PUBLIC,console_app;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p CROSS JOIN LATERAL aclexplode(p.proacl) a
 JOIN pg_roles r ON r.oid=a.grantee WHERE p.oid='public.platform_legacy_topology_receipts_immutable_v1()'::regprocedure LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.platform_legacy_topology_receipts_immutable_v1() FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.platform_legacy_topology_receipts_immutable_v1() TO console_app;


-- source: cross-owner-acl.sql
GRANT USAGE ON SCHEMA ontology_api,ont_policy_api TO console_account_owner;
GRANT SELECT(org_id) ON public.users,public.policy_role_permissions,public.policy_role_conditions TO console_account_owner;
GRANT SELECT(group_id) ON public.group_role_grants TO console_account_owner;
GRANT SELECT(org_id,stable_key,revision,validator_id,created_at,updated_at) ON public.ont_object_type_key_revisions TO console_account_owner;
GRANT SELECT(id,org_id,stable_key,title,title_property_key,backing_kind,backing_table,primary_key_property,
 schema_version,lifecycle_state,created_by,created_at,updated_at,attribution_protocol,created_by_account_id,
 origin_account_id,origin_command_id,origin_receipt_id) ON public.ont_object_types TO console_account_owner;
GRANT SELECT(org_id,catalog_version,manifest_digest,installed_by,installed_at,attribution_protocol,
 installed_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
 ON public.ont_builtin_catalog_installs TO console_account_owner;
GRANT SELECT(id,org_id,object_type_id,key,title,type,config,backing_column,required,in_property_policy,created_at)
 ON public.ont_property_defs TO console_account_owner;
GRANT SELECT(id,org_id,object_type_id,stable_key,title,params_schema,edits,submission_criteria,side_effects,
 dispatch,dispatch_target,control_points,created_at) ON public.ont_action_types TO console_account_owner;
GRANT SELECT(org_id) ON public.ont_link_types,public.ont_analytics TO console_account_owner;
GRANT SELECT(id,org_id,normalized_row,effect,bundle_digest,schema_version,generated_policy_text,status,created_by,updated_by,created_at,updated_at,
 attribution_protocol,created_by_account_id,updated_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
 ON public.cedar_policy_catalog_entries TO console_account_owner;
GRANT SELECT(id,org_id,object_type_id,cedar_policy_id,effect,created_by,created_at,attribution_protocol,
 created_by_account_id,origin_account_id,origin_command_id,origin_receipt_id)
 ON public.ont_object_policies TO console_account_owner;
GRANT UPDATE(id) ON public.ont_property_defs,public.ont_action_types TO console_ontology_writer;
GRANT SELECT(org_id,actor,action,target_type,target_id,branch_id,before_snap,after_snap,trace_id,span_id,occurred_at)
 ON public.audit_events TO console_account_owner;
GRANT INSERT(id,org_id,actor,action,target_type,target_id,before_snap,after_snap,trace_id,span_id,occurred_at)
 ON public.audit_events TO console_account_owner;


-- source: identity-existing-catalog-guards-round2.sql
-- Private complete-source proposal; not a standalone installer or activation.
-- Prerequisites: exact Account credential finalizer, enrollment binding/closure,
-- native assignment revisions and company_actor_entitlement_shape_v2.
CREATE FUNCTION public.identity_company_actor_birth_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b record;
BEGIN
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 IF TG_OP<>'INSERT' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
 END IF;
 IF NEW.entitlement_ref->>'kind' IS DISTINCT FROM 'COMPANY_ENROLLMENT_V1' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(
  (NEW.entitlement_ref->>'account_id')::uuid,(NEW.entitlement_ref->>'command_id')::uuid);
 IF b.request_state<>'PENDING'
  OR ROW(NEW.org_id,NEW.account_id,NEW.admission_receipt_id,NEW.created_at)
   IS DISTINCT FROM ROW(b.org_id,b.administrative_account_id,b.receipt_id,b.started_at)
  OR NEW.entitlement_ref IS DISTINCT FROM jsonb_build_object('kind','COMPANY_ENROLLMENT_V1',
    'account_id',b.account_id::text,'command_id',b.command_id::text,
    'org_id',b.org_id::text,'receipt_id',b.receipt_id::text) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.identity_company_candidate_birth_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE a public.user_role_assignments%ROWTYPE; b record;
BEGIN
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 IF TG_OP<>'INSERT' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.operation_unavailable';
 END IF;
 IF NEW.context_kind IS DISTINCT FROM 'COMPANY'
  OR NEW.source_key->>'kind' IS DISTINCT FROM 'COMPANY_POLICY' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 SELECT x.* INTO STRICT a FROM public.user_role_assignments x
  WHERE x.org_id=NEW.context_id AND x.id=(NEW.source_key->>'assignment_id')::uuid
   AND x.subject_protocol='NATIVE_ACCOUNT' AND x.native_current_revision=1;
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(a.origin_account_id,a.origin_command_id);
 IF b.request_state<>'PENDING' OR a.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR a.account_id IS DISTINCT FROM b.administrative_account_id
  OR ROW(NEW.account_id,NEW.context_id,NEW.source_revision,NEW.incarnation,NEW.state)
   IS DISTINCT FROM ROW(b.administrative_account_id,b.org_id,1::bigint,NULL::uuid,'CURRENT'::text)
  OR NEW.source_key IS DISTINCT FROM jsonb_build_object('kind','COMPANY_POLICY','org_id',b.org_id::text,
    'assignment_id',a.id::text,'revision','1') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_birth';
END
$body$;

CREATE FUNCTION public.identity_company_context_generation_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE frame public.company_enrollment_effect_bindings%ROWTYPE; b record;
BEGIN
 -- Preserve existing Auth transitions that do not change navigation generation.
 IF NEW.context_generation IS NOT DISTINCT FROM OLD.context_generation THEN RETURN NEW; END IF;
 IF current_user<>'console_account_owner' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='identity_native.owner_required';
 END IF;
 IF to_jsonb(NEW)-'context_generation' IS DISTINCT FROM to_jsonb(OLD)-'context_generation'
  OR OLD.security_state IS DISTINCT FROM 'ACTIVE'
  OR OLD.context_generation NOT BETWEEN 1 AND 9223372036854775806
  OR NEW.context_generation IS DISTINCT FROM OLD.context_generation+1 THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_context_generation';
 END IF;
 SELECT x.* INTO STRICT frame FROM public.company_enrollment_effect_bindings x
  WHERE x.administrative_account_id=OLD.account_id
   AND x.recipient_context_generation_before=OLD.context_generation
   AND x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(frame.account_id,frame.command_id);
 IF b.request_state<>'PENDING' OR NOT EXISTS(
  SELECT 1 FROM public.account_context_candidates c JOIN public.user_role_assignments a
   ON a.org_id=c.context_id AND a.id=(c.source_key->>'assignment_id')::uuid
  WHERE c.account_id=b.administrative_account_id AND c.context_kind='COMPANY' AND c.context_id=b.org_id
   AND c.source_revision=1 AND c.incarnation IS NULL AND c.state='CURRENT'
   AND c.source_key=jsonb_build_object('kind','COMPANY_POLICY','org_id',b.org_id::text,
     'assignment_id',a.id::text,'revision','1')
   AND a.subject_protocol='NATIVE_ACCOUNT' AND a.account_id=b.administrative_account_id
   AND a.native_current_revision=1 AND a.origin_account_id=b.account_id
   AND a.origin_command_id=b.command_id AND a.origin_receipt_id=b.receipt_id) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_context_generation';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='identity_native.invalid_context_generation';
END
$body$;

CREATE FUNCTION public.identity_company_existing_catalog_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE actor uuid; command uuid; value jsonb:=to_jsonb(NEW);
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 CASE TG_TABLE_NAME
 WHEN 'company_actors' THEN
  actor:=(value->'entitlement_ref'->>'account_id')::uuid;
  command:=(value->'entitlement_ref'->>'command_id')::uuid;
 WHEN 'account_context_candidates' THEN
  PERFORM set_config('app.current_org',value->>'context_id',true);
  SELECT a.origin_account_id,a.origin_command_id INTO STRICT actor,command
   FROM public.user_role_assignments a
   WHERE a.org_id=(value->>'context_id')::uuid
    AND a.id=(value->'source_key'->>'assignment_id')::uuid AND a.subject_protocol='NATIVE_ACCOUNT';
 WHEN 'account_security' THEN
  IF NEW.context_generation IS NOT DISTINCT FROM OLD.context_generation THEN RETURN NEW; END IF;
  SELECT x.account_id,x.command_id INTO STRICT actor,command
   FROM public.company_enrollment_effect_bindings x
   WHERE x.administrative_account_id=NEW.account_id
    AND x.recipient_context_generation_before=OLD.context_generation
    AND x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 ELSE RAISE EXCEPTION 'identity_native.invalid_closure';
 END CASE;
 PERFORM public.company_enrollment_assert_closure_v1(actor,command);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

ALTER FUNCTION public.identity_company_actor_birth_guard_v1() OWNER TO console_account_owner;
ALTER FUNCTION public.identity_company_candidate_birth_guard_v1() OWNER TO console_account_owner;
ALTER FUNCTION public.identity_company_context_generation_guard_v1() OWNER TO console_account_owner;
ALTER FUNCTION public.identity_company_existing_catalog_closure_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_company_actor_birth_guard_v1(),
 public.identity_company_candidate_birth_guard_v1(),public.identity_company_context_generation_guard_v1(),
 public.identity_company_existing_catalog_closure_v1() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.identity_company_actor_birth_guard_v1(),
 public.identity_company_candidate_birth_guard_v1(),public.identity_company_context_generation_guard_v1(),
 public.identity_company_existing_catalog_closure_v1() TO console_account_owner;

CREATE TRIGGER company_actor_birth_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.company_actors
 FOR EACH ROW EXECUTE FUNCTION public.identity_company_actor_birth_guard_v1();
CREATE TRIGGER company_actor_truncate_guard_v1 BEFORE TRUNCATE ON public.company_actors
 FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
CREATE TRIGGER company_candidate_birth_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.account_context_candidates
 FOR EACH ROW EXECUTE FUNCTION public.identity_company_candidate_birth_guard_v1();
CREATE TRIGGER company_candidate_truncate_guard_v1 BEFORE TRUNCATE ON public.account_context_candidates
 FOR EACH STATEMENT EXECUTE FUNCTION public.identity_native_immutable_v1();
CREATE TRIGGER company_context_generation_guard_v1 BEFORE UPDATE ON public.account_security
 FOR EACH ROW EXECUTE FUNCTION public.identity_company_context_generation_guard_v1();
CREATE CONSTRAINT TRIGGER company_actor_birth_closure_v1 AFTER INSERT ON public.company_actors
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_company_existing_catalog_closure_v1();
CREATE CONSTRAINT TRIGGER company_candidate_birth_closure_v1 AFTER INSERT ON public.account_context_candidates
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_company_existing_catalog_closure_v1();
CREATE CONSTRAINT TRIGGER company_context_generation_closure_v1 AFTER UPDATE ON public.account_security
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.identity_company_existing_catalog_closure_v1();
ALTER TABLE public.company_actors ENABLE ALWAYS TRIGGER company_actor_birth_guard_v1;
ALTER TABLE public.company_actors ENABLE ALWAYS TRIGGER company_actor_truncate_guard_v1;
ALTER TABLE public.company_actors ENABLE ALWAYS TRIGGER company_actor_birth_closure_v1;
ALTER TABLE public.account_context_candidates ENABLE ALWAYS TRIGGER company_candidate_birth_guard_v1;
ALTER TABLE public.account_context_candidates ENABLE ALWAYS TRIGGER company_candidate_truncate_guard_v1;
ALTER TABLE public.account_context_candidates ENABLE ALWAYS TRIGGER company_candidate_birth_closure_v1;
ALTER TABLE public.account_security ENABLE ALWAYS TRIGGER company_context_generation_guard_v1;
ALTER TABLE public.account_security ENABLE ALWAYS TRIGGER company_context_generation_closure_v1;

-- Exact projection/INSERT columns; UPDATE(org_id) is solely the RI row-lock
-- prerequisite. Its DML forms above remain denied, including no-op UPDATE.
GRANT SELECT(org_id,account_id,admission_receipt_id,entitlement_ref,created_at),
 INSERT(org_id,account_id,admission_receipt_id,entitlement_ref,created_at),UPDATE(org_id)
 ON public.company_actors TO console_account_owner;
GRANT INSERT(account_id,source_key,context_kind,context_id,source_revision,incarnation,state)
 ON public.account_context_candidates TO console_account_owner;
GRANT UPDATE(context_generation) ON public.account_security TO console_account_owner;


-- source: native-ri-lock-privileges.sql
-- Private complete-assembly component. Apply only after the exact existing
-- ALWAYS immutable triggers and owner SELECT ACLs in catalog-install-metadata.
-- These owner-only single columns permit PostgreSQL RI SELECT FOR KEY SHARE;
-- ordinary UPDATE remains refused by the immutable statement triggers.
GRANT UPDATE(org_id) ON public.policy_role_revisions,
 public.policy_capability_clauses,public.policy_capability_clause_fields,
 public.policy_assignment_revisions TO console_account_owner;
GRANT UPDATE(org_id) ON public.native_company_catalog_installs,
 public.native_company_object_refs,public.native_company_action_refs,
 public.native_company_property_refs TO console_ontology_writer;

-- The deferred current-history FK also performs SELECT FOR KEY SHARE.
-- Its table owner retains no mutation capability through this column grant:
-- the ALWAYS immutable statement trigger rejects UPDATE/DELETE/TRUNCATE.
GRANT UPDATE(org_id) ON public.group_membership_revisions TO console_app;


-- source: /private/tmp/console-legacy-membership-frame-20260921-round3/SCHEMA.sql
-- Exact proposed shape only. Install only with complete owner/guards/closure and custody.
CREATE TABLE public.platform_legacy_membership_effect_bindings (
 actor_user_id uuid NOT NULL,
 command_id uuid NOT NULL,
 receipt_id uuid NOT NULL UNIQUE,
 effect_family_id uuid NOT NULL,
 effect_xid xid8 NOT NULL,
 effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 kind smallint NOT NULL CHECK(kind IN(1,4,5)),
 input_bytes bytea NOT NULL CHECK(octet_length(input_bytes) BETWEEN 62 AND 4096),
 input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32 AND input_digest=sha256(input_bytes)),
 org_id uuid NOT NULL UNIQUE,
 company_status text NOT NULL CHECK(company_status IN('ACTIVE','SUSPENDED','ARCHIVED')),
 CHECK(kind<>1 OR company_status='ACTIVE'),
 old_group_id uuid,
 old_membership_id uuid,
 old_membership_revision bigint CHECK(old_membership_revision>0),
 old_membership_incarnation uuid,
 group_id uuid NOT NULL,
 mint_suffix smallint CHECK(mint_suffix BETWEEN 1 AND 9),
 group_is_new boolean NOT NULL,
 membership_id uuid NOT NULL,
 membership_incarnation uuid NOT NULL,
 occurred_at timestamptz NOT NULL CHECK(isfinite(occurred_at)),
 heads_before jsonb NOT NULL CHECK(
  CASE WHEN jsonb_typeof(heads_before)='array' THEN jsonb_array_length(heads_before)<=2 ELSE false END),
 PRIMARY KEY(actor_user_id,command_id),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (actor_user_id,command_id,receipt_id,effect_family_id,org_id,group_id,membership_id,membership_incarnation)),
 CHECK(old_group_id IS NULL OR old_group_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK((kind=1)=(old_group_id IS NULL)),
 CHECK((kind=1 AND old_membership_id IS NULL AND old_membership_revision IS NULL AND old_membership_incarnation IS NULL)
  OR (kind IN(4,5) AND old_membership_id IS NOT NULL AND old_membership_revision IS NOT NULL AND old_membership_incarnation IS NOT NULL)),
 CHECK(old_membership_id IS NULL OR old_membership_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK(old_membership_incarnation IS NULL OR old_membership_incarnation<>'00000000-0000-0000-0000-000000000000'::uuid),
 CHECK((kind IN(1,5))=(mint_suffix IS NOT NULL)),
 CHECK(kind<>4 OR NOT group_is_new)
);
ALTER TABLE public.platform_legacy_membership_effect_bindings OWNER TO console_app;
ALTER TABLE public.platform_legacy_membership_effect_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.platform_legacy_membership_effect_bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY platform_legacy_membership_binding_owner
 ON public.platform_legacy_membership_effect_bindings TO console_app
 USING(current_user='console_app') WITH CHECK(current_user='console_app');
REVOKE ALL ON public.platform_legacy_membership_effect_bindings FROM PUBLIC,
 console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,console_ontology_cmd,
 console_platform_force_cmd,console_account_owner,console_terms_owner,console_credential_owner,
 console_ontology_writer;
-- No other table grants, defaults, indexes or live-row FKs. The deferred retained-NEW
-- closure, not a mutable receipt flag or historical row xmin, proves completion.


-- source: /private/tmp/console-legacy-catalog-binding-successor-20260921/SCHEMA.sql
-- Proposed exact physical shape, NOT an executable standalone migration.
-- The receipt table is still unimplemented; these columns belong in its first
-- complete CREATE statement, never by rewriting historical installed receipts.
-- effect_xid xid8 NOT NULL,
-- effect_backend_pid integer NOT NULL CHECK (effect_backend_pid > 0)

CREATE TABLE public.platform_legacy_catalog_effect_bindings (
 effect_xid xid8 NOT NULL,
 effect_backend_pid integer NOT NULL,
 actor_user_id uuid NOT NULL,
 command_id uuid NOT NULL,
 input_digest bytea NOT NULL,
 effect_family_id uuid NOT NULL,
 receipt_id uuid NOT NULL,
 org_id uuid NOT NULL,
 admin_user_id uuid NOT NULL,
 catalog_version text NOT NULL,
 manifest_digest bytea NOT NULL,
 trace_id text NOT NULL,
 span_id text NOT NULL,
 catalog_at timestamptz NOT NULL,
 CONSTRAINT platform_legacy_catalog_binding_pk PRIMARY KEY(actor_user_id,command_id),
 CONSTRAINT platform_legacy_catalog_binding_org_key UNIQUE(org_id),
 CONSTRAINT platform_legacy_catalog_binding_receipt_key UNIQUE(receipt_id),
 CONSTRAINT platform_legacy_catalog_binding_backend_check CHECK(effect_backend_pid>0),
 CONSTRAINT platform_legacy_catalog_binding_ids_check CHECK(
  actor_user_id<>'00000000-0000-0000-0000-000000000000'::uuid AND
  command_id<>'00000000-0000-0000-0000-000000000000'::uuid AND
  effect_family_id<>'00000000-0000-0000-0000-000000000000'::uuid AND
  receipt_id<>'00000000-0000-0000-0000-000000000000'::uuid AND
  org_id<>'00000000-0000-0000-0000-000000000000'::uuid AND
  admin_user_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 CONSTRAINT platform_legacy_catalog_binding_input_check CHECK(octet_length(input_digest)=32),
 CONSTRAINT platform_legacy_catalog_binding_catalog_check CHECK(
  catalog_version='2026-07-19.1' AND
  manifest_digest=decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex')),
 CONSTRAINT platform_legacy_catalog_binding_trace_check CHECK(
  trace_id~'^[0-9a-f]{32}$' AND trace_id<>repeat('0',32) AND
  span_id~'^[0-9a-f]{16}$' AND span_id<>repeat('0',16)),
 CONSTRAINT platform_legacy_catalog_binding_time_check CHECK(isfinite(catalog_at))
);
ALTER TABLE public.platform_legacy_catalog_effect_bindings OWNER TO console_app;
ALTER TABLE public.platform_legacy_catalog_effect_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.platform_legacy_catalog_effect_bindings FORCE ROW LEVEL SECURITY;
CREATE POLICY platform_legacy_catalog_binding_owner
 ON public.platform_legacy_catalog_effect_bindings TO console_app
 USING(current_user='console_app') WITH CHECK(current_user='console_app');
REVOKE ALL ON public.platform_legacy_catalog_effect_bindings FROM PUBLIC;
-- Exact final ACL: owner implicit rights only; no nonowner grants, grant options,
-- defaults, live-identity FKs or additional indexes. This is transient private
-- coordination state, not durable command history. Serving roles get no access.
-- All four routines revoke PUBLIC/default EXECUTE. Only live_audit and
-- receipt_audit grant EXECUTE to existing console_ontology_writer.
-- BEFORE INSERT OR UPDATE OR DELETE OR TRUNCATE requires separate row DML and
-- statement TRUNCATE trigger registrations, both same write_guard function.
-- Names: trg_platform_legacy_catalog_binding_write (row I/U/D),
-- trg_platform_legacy_catalog_binding_truncate (statement TRUNCATE),
-- trg_platform_legacy_catalog_binding_closed (row AFTER INSERT,
-- CONSTRAINT DEFERRABLE INITIALLY DEFERRED).


-- source: /private/tmp/console-legacy-topology-source-bridge-20260921/account-roots-lock.sql
-- Private source candidate. Install only with the complete reviewed bridge/custody.
CREATE FUNCTION public.account_legacy_topology_roots_lock_v1(p_subjects uuid[])
RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR p_subjects IS NULL OR cardinality(p_subjects)=0
  OR array_ndims(p_subjects) IS DISTINCT FROM 1
  OR array_lower(p_subjects,1) IS DISTINCT FROM 1
  OR array_position(p_subjects,NULL) IS NOT NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid=ANY(p_subjects)
  OR p_subjects IS DISTINCT FROM (
   SELECT array_agg(DISTINCT x.id ORDER BY x.id) FROM unnest(p_subjects) x(id)) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_root_plan';
 END IF;
 -- User locks precede every Account lock. Missing roots are not manufactured
 -- here; the unchanged current-source owner still classifies source custody.
 PERFORM a.id FROM public.accounts a WHERE a.id=ANY(p_subjects)
  ORDER BY a.id FOR UPDATE OF a;
END
$body$;
ALTER FUNCTION public.account_legacy_topology_roots_lock_v1(uuid[]) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_legacy_topology_roots_lock_v1(uuid[]) FROM PUBLIC,
 console_app,console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,
 console_ontology_cmd,console_platform_force_cmd,console_account_owner,
 console_terms_owner,console_credential_owner,console_ontology_writer;
GRANT EXECUTE ON FUNCTION public.account_legacy_topology_roots_lock_v1(uuid[])
 TO console_account_owner,console_app;


-- source: legacy-command-frame-force-successor.sql
-- Private proposed finite witness successor; not an installation resource.
-- Same owner and exact original bytes as the canonical nine-kind receipt.
CREATE TABLE public.platform_legacy_topology_effect_bindings (
 actor_user_id uuid NOT NULL,command_id uuid NOT NULL,receipt_id uuid NOT NULL UNIQUE,
 effect_family_id uuid NOT NULL,codec_version smallint NOT NULL CHECK(codec_version=1),
 kind smallint NOT NULL CHECK(kind BETWEEN 1 AND 9),primary_target uuid,
 input_bytes bytea NOT NULL CHECK(octet_length(input_bytes) BETWEEN 62 AND 4096),
 input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32 AND input_digest=sha256(input_bytes)),
 effect_xid xid8 NOT NULL,effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 trace_id char(32) NOT NULL CHECK(trace_id~'^[0-9a-f]{32}$' AND trace_id<>repeat('0',32)),
 span_id char(16) NOT NULL CHECK(span_id~'^[0-9a-f]{16}$' AND span_id<>repeat('0',16)),
 occurred_at timestamptz NOT NULL CHECK(isfinite(occurred_at)),
 plan_snapshot jsonb NOT NULL CHECK(jsonb_typeof(plan_snapshot)='object'),
 PRIMARY KEY(actor_user_id,command_id),UNIQUE(effect_xid,effect_backend_pid),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN(actor_user_id,command_id,receipt_id,effect_family_id)),
 CHECK((kind IN(1,2))=(primary_target IS NULL)),
 CHECK(primary_target IS NULL OR primary_target<>'00000000-0000-0000-0000-000000000000'::uuid)
);
ALTER TABLE public.platform_legacy_topology_effect_bindings OWNER TO console_app;
REVOKE ALL ON public.platform_legacy_topology_effect_bindings FROM PUBLIC,console_rt,console_auth_rt,
 console_auth_startup,console_leave_cmd,console_ontology_cmd,console_platform_force_cmd,
 console_account_owner,console_terms_owner,console_credential_owner,console_ontology_writer;

CREATE FUNCTION public.platform_legacy_command_frame_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d record;r public.platform_legacy_topology_receipts%ROWTYPE;
BEGIN
 IF current_user<>'console_app' OR TG_OP NOT IN('INSERT','DELETE') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_invalid';
 END IF;
 IF TG_OP='INSERT' THEN
  IF EXISTS(SELECT 1 FROM public.platform_force_removal_effect_bindings x
    WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid())
   OR EXISTS(SELECT 1 FROM public.platform_force_removal_receipts x
    WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid()) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_invalid';
  END IF;
  SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(NEW.input_bytes);
  IF ROW(d.actor_user_id,d.command_id,d.kind,d.primary_target,d.input_digest)
    IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.kind,NEW.primary_target,NEW.input_digest)
   OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
   OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts x
    WHERE (x.actor_user_id=NEW.actor_user_id AND x.command_id=NEW.command_id)
     OR (x.effect_xid=NEW.effect_xid AND x.effect_backend_pid=NEW.effect_backend_pid)) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_invalid';
  END IF;
  IF NEW.plan_snapshot IS DISTINCT FROM public.platform_legacy_topology_plan_v1(
    NEW.actor_user_id,NEW.effect_family_id,NEW.input_bytes,
    (NEW.plan_snapshot->>'prospective_company')::uuid,(NEW.plan_snapshot->>'prospective_group')::uuid) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_invalid';
  END IF;
  RETURN NEW;
 END IF;
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.actor_user_id=OLD.actor_user_id AND x.command_id=OLD.command_id;
 IF OLD.effect_xid IS DISTINCT FROM pg_current_xact_id() OR OLD.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR ROW(r.receipt_id,r.effect_family_id,r.codec_version,r.kind,r.primary_target,r.input_bytes,r.input_digest,
     r.effect_xid,r.effect_backend_pid,r.trace_id,r.span_id,r.occurred_at)
   IS DISTINCT FROM ROW(OLD.receipt_id,OLD.effect_family_id,OLD.codec_version,OLD.kind,OLD.primary_target,OLD.input_bytes,OLD.input_digest,
     OLD.effect_xid,OLD.effect_backend_pid,OLD.trace_id,OLD.span_id,OLD.occurred_at) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_invalid';
 END IF;
 RETURN OLD;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_invalid';
END
$body$;
ALTER FUNCTION public.platform_legacy_command_frame_guard_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_command_frame_guard_v1() FROM PUBLIC;
CREATE TRIGGER trg_platform_legacy_command_frame_guard BEFORE INSERT OR UPDATE OR DELETE
 ON public.platform_legacy_topology_effect_bindings FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_command_frame_guard_v1();
CREATE TRIGGER trg_platform_legacy_command_frame_truncate BEFORE TRUNCATE
 ON public.platform_legacy_topology_effect_bindings FOR EACH STATEMENT EXECUTE FUNCTION public.platform_legacy_command_frame_guard_v1();
ALTER TABLE public.platform_legacy_topology_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_command_frame_guard;
ALTER TABLE public.platform_legacy_topology_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_command_frame_truncate;

CREATE FUNCTION public.platform_legacy_command_frame_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_legacy_topology_receipts%ROWTYPE;d record; item jsonb; actual jsonb;
 groups_expected uuid[]; groups_actual uuid[]; target uuid; expected_heads jsonb;
BEGIN
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(NEW.input_bytes);
 IF ROW(d.actor_user_id,d.command_id,d.kind,d.primary_target,d.input_digest)
   IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.kind,NEW.primary_target,NEW.input_digest)
  OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_effect_bindings b
   WHERE b.effect_xid=NEW.effect_xid AND b.effect_backend_pid=NEW.effect_backend_pid)
  OR ROW(r.actor_user_id,r.command_id,r.receipt_id,r.effect_family_id,r.codec_version,r.kind,r.primary_target,
     r.input_bytes,r.input_digest,r.effect_xid,r.effect_backend_pid,r.trace_id,r.span_id,r.occurred_at)
   IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.receipt_id,NEW.effect_family_id,NEW.codec_version,NEW.kind,NEW.primary_target,
     NEW.input_bytes,NEW.input_digest,NEW.effect_xid,NEW.effect_backend_pid,NEW.trace_id,NEW.span_id,NEW.occurred_at) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
 END IF;
 IF r.outcome='APPLIED' AND r.kind IN(1,6) THEN
  IF NOT EXISTS(SELECT 1 FROM public.platform_legacy_user_birth_witnesses w
    WHERE w.actor_user_id=r.actor_user_id AND w.command_id=r.command_id AND w.receipt_id=r.receipt_id
     AND w.effect_xid=r.effect_xid AND w.effect_backend_pid=r.effect_backend_pid
     AND w.user_id=r.result_user_id AND w.org_id=r.result_org_id) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
  END IF;
 ELSIF EXISTS(SELECT 1 FROM public.platform_legacy_user_birth_witnesses w
   WHERE w.actor_user_id=r.actor_user_id AND w.command_id=r.command_id) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
 END IF;
 -- NEW retains the exact trusted-prefix pre-effect snapshot even after frame
 -- deletion. This closes missing receipt+head/grant/User effects together.
 IF d.kind IN(7,8) THEN
  FOR item IN SELECT value FROM jsonb_array_elements(NEW.plan_snapshot->'grants') LOOP
   SELECT to_jsonb(g) INTO actual FROM public.group_role_grants g WHERE g.id=(item->>'id')::uuid;
   IF (r.outcome='APPLIED' AND FOUND) OR (r.outcome='REJECTED' AND actual IS DISTINCT FROM item) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
   END IF;
  END LOOP;
 END IF;
 IF d.kind=8 THEN
  target:=(d.payload->>'org_id')::uuid;
  FOR item IN SELECT value FROM jsonb_array_elements(NEW.plan_snapshot->'users')
   WHERE value->>'org_id'=target::text LOOP
   SELECT to_jsonb(u) INTO actual FROM public.users u WHERE u.id=(item->>'id')::uuid;
   IF (r.outcome='APPLIED' AND FOUND) OR (r.outcome='REJECTED' AND actual IS DISTINCT FROM item->'row') THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
   END IF;
  END LOOP;
  IF r.outcome='APPLIED' THEN
   SELECT array_agg((value->>'id')::uuid ORDER BY (value->>'id')::uuid) INTO groups_expected
    FROM jsonb_array_elements(NEW.plan_snapshot->'groups');
   SELECT array_agg((value->>'group_id')::uuid ORDER BY (value->>'group_id')::uuid) INTO groups_actual
    FROM jsonb_array_elements(r.heads_after);
   IF groups_actual IS DISTINCT FROM groups_expected
    OR EXISTS(SELECT 1 FROM public.users u WHERE u.org_id=target)
    OR NEW.plan_snapshot->'grantor_blocked' IS DISTINCT FROM 'false'::jsonb THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
   END IF;
   SELECT jsonb_agg(jsonb_build_object('group_id',v->>'id','incarnation',v->'head'->>'incarnation',
     'revision',v->'head'->>'revision') ORDER BY (v->>'id')::uuid)
    INTO expected_heads FROM jsonb_array_elements(NEW.plan_snapshot->'groups') t(v);
   IF r.heads_before IS DISTINCT FROM expected_heads THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
   END IF;
  END IF;
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.command_frame_closure_invalid';
END
$body$;
ALTER FUNCTION public.platform_legacy_command_frame_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_command_frame_closed_v1() FROM PUBLIC;
CREATE CONSTRAINT TRIGGER trg_platform_legacy_command_frame_closed AFTER INSERT
 ON public.platform_legacy_topology_effect_bindings DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_command_frame_closed_v1();

ALTER TABLE public.platform_legacy_topology_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_command_frame_closed;


-- source: legacy-user-birth-witness-force-successor.sql
-- Private finite physical witness; belongs to existing mandatory ordinary frame.
-- No historic receipt/command bytes change. Complete force/source/ACL closure
-- and independent review are prerequisites to assembling or installing this.
CREATE TABLE public.platform_legacy_user_birth_witnesses (
 actor_user_id uuid NOT NULL,command_id uuid NOT NULL,receipt_id uuid NOT NULL,
 effect_xid xid8 NOT NULL,effect_backend_pid integer NOT NULL,
 user_id uuid NOT NULL,org_id uuid NOT NULL,row_digest bytea NOT NULL CHECK(octet_length(row_digest)=32),
 PRIMARY KEY(actor_user_id,command_id),UNIQUE(effect_xid,effect_backend_pid),UNIQUE(user_id),
 FOREIGN KEY(actor_user_id,command_id) REFERENCES public.platform_legacy_topology_receipts(actor_user_id,command_id)
  DEFERRABLE INITIALLY DEFERRED
);
ALTER TABLE public.platform_legacy_user_birth_witnesses OWNER TO console_app;
REVOKE ALL ON public.platform_legacy_user_birth_witnesses FROM PUBLIC,console_rt,console_auth_rt,
 console_auth_startup,console_leave_cmd,console_ontology_cmd,console_platform_force_cmd,
 console_account_owner,console_terms_owner,console_credential_owner,console_ontology_writer;

CREATE FUNCTION public.platform_legacy_user_birth_capture_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE frame public.platform_legacy_topology_effect_bindings%ROWTYPE; d record;
 expected_org uuid; expected_name text; expected_phone text; expected_roles jsonb;
BEGIN
 -- Force never creates a User; its finite guard rejects this effect and tail writes.
 PERFORM public.platform_force_effect_admit_v1(TG_TABLE_NAME,TG_OP,NULL,to_jsonb(NEW));
 SELECT x.* INTO frame FROM public.platform_legacy_topology_effect_bindings x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 IF NOT FOUND THEN
  IF EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts r
    WHERE r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid()) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_invalid';
  END IF;
  RETURN NULL;
 END IF;
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(frame.input_bytes);
 IF d.kind NOT IN(1,6) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_invalid';
 END IF;
 IF d.kind=1 THEN
  expected_org:=(frame.plan_snapshot->>'prospective_company')::uuid;
  expected_name:='Tenant Admin';expected_phone:=NULL;expected_roles:='["SUPER_ADMIN"]'::jsonb;
 ELSE
  expected_org:=(d.payload->>'org_id')::uuid;expected_name:=d.payload->>'display_name';
  expected_phone:=d.payload->>'phone';expected_roles:=d.payload->'tenant_roles';
 END IF;
 IF NEW.id='00000000-0000-0000-0000-000000000000'::uuid
  OR to_jsonb(NEW) IS DISTINCT FROM jsonb_build_object('id',NEW.id,'org_id',expected_org,
   'display_name',expected_name,'phone',expected_phone,'roles',expected_roles,'team',NULL,
   'is_active',true,'is_org_lead',false,'employee_id',NULL,'created_at',transaction_timestamp()) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_invalid';
 END IF;
 INSERT INTO public.platform_legacy_user_birth_witnesses(actor_user_id,command_id,receipt_id,
  effect_xid,effect_backend_pid,user_id,org_id,row_digest)
 VALUES(frame.actor_user_id,frame.command_id,frame.receipt_id,frame.effect_xid,frame.effect_backend_pid,
  NEW.id,NEW.org_id,sha256(convert_to(to_jsonb(NEW)::text,'UTF8')));
 RETURN NULL;
END
$body$;

CREATE FUNCTION public.platform_legacy_user_birth_witness_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE frame public.platform_legacy_topology_effect_bindings%ROWTYPE; actual_digest bytea;
BEGIN
 IF TG_OP<>'INSERT' OR current_user<>'console_app' OR pg_trigger_depth()<2 THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_invalid';
 END IF;
 SELECT x.* INTO STRICT frame FROM public.platform_legacy_topology_effect_bindings x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 SELECT sha256(convert_to(to_jsonb(u)::text,'UTF8')) INTO STRICT actual_digest
  FROM public.users u WHERE u.id=NEW.user_id AND u.org_id=NEW.org_id;
 IF frame.kind NOT IN(1,6) OR NEW.row_digest IS DISTINCT FROM actual_digest
  OR ROW(NEW.actor_user_id,NEW.command_id,NEW.receipt_id,NEW.effect_xid,NEW.effect_backend_pid)
   IS DISTINCT FROM ROW(frame.actor_user_id,frame.command_id,frame.receipt_id,frame.effect_xid,frame.effect_backend_pid) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_invalid';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_invalid';
END
$body$;

CREATE FUNCTION public.platform_legacy_user_birth_witness_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_legacy_topology_receipts%ROWTYPE; actual_digest bytea;
BEGIN
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.actor_user_id=NEW.actor_user_id AND x.command_id=NEW.command_id;
 SELECT sha256(convert_to(to_jsonb(u)::text,'UTF8')) INTO STRICT actual_digest
  FROM public.users u WHERE u.id=NEW.user_id AND u.org_id=NEW.org_id;
 IF r.kind NOT IN(1,6) OR r.outcome IS DISTINCT FROM 'APPLIED' OR NEW.row_digest IS DISTINCT FROM actual_digest
  OR ROW(r.receipt_id,r.effect_xid,r.effect_backend_pid,r.result_user_id,r.result_org_id)
   IS DISTINCT FROM ROW(NEW.receipt_id,NEW.effect_xid,NEW.effect_backend_pid,NEW.user_id,NEW.org_id)
  OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid() THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_closure_invalid';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_birth_closure_invalid';
END
$body$;
ALTER FUNCTION public.platform_legacy_user_birth_capture_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_legacy_user_birth_witness_guard_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_legacy_user_birth_witness_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_user_birth_capture_v1(),
 public.platform_legacy_user_birth_witness_guard_v1(),public.platform_legacy_user_birth_witness_closed_v1() FROM PUBLIC;
CREATE TRIGGER trg_platform_legacy_user_birth_capture AFTER INSERT ON public.users
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_user_birth_capture_v1();
CREATE TRIGGER trg_platform_legacy_user_birth_witness_guard BEFORE INSERT OR UPDATE OR DELETE
 ON public.platform_legacy_user_birth_witnesses FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_user_birth_witness_guard_v1();
CREATE TRIGGER trg_platform_legacy_user_birth_witness_truncate BEFORE TRUNCATE
 ON public.platform_legacy_user_birth_witnesses FOR EACH STATEMENT EXECUTE FUNCTION public.platform_legacy_user_birth_witness_guard_v1();
CREATE CONSTRAINT TRIGGER trg_platform_legacy_user_birth_witness_closed AFTER INSERT
 ON public.platform_legacy_user_birth_witnesses DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_user_birth_witness_closed_v1();
ALTER TABLE public.users ENABLE ALWAYS TRIGGER trg_platform_legacy_user_birth_capture;
ALTER TABLE public.platform_legacy_user_birth_witnesses ENABLE ALWAYS TRIGGER trg_platform_legacy_user_birth_witness_guard;
ALTER TABLE public.platform_legacy_user_birth_witnesses ENABLE ALWAYS TRIGGER trg_platform_legacy_user_birth_witness_truncate;
ALTER TABLE public.platform_legacy_user_birth_witnesses ENABLE ALWAYS TRIGGER trg_platform_legacy_user_birth_witness_closed;


-- source: legacy-membership-binding.sql
-- Private candidate implementing the approved round3 membership frame contract.
-- Compose with its exact schema, mandatory closure and complete coordinator only.
CREATE FUNCTION public.platform_legacy_membership_binding_write_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b public.platform_legacy_membership_effect_bindings%ROWTYPE;
 decoded record; r public.platform_legacy_topology_receipts%ROWTYPE;
 o public.organizations%ROWTYPE; m public.group_memberships%ROWTYPE;
 h public.group_membership_revisions%ROWTYPE;
 expected_before jsonb; expected_count integer; item jsonb;
 suffix integer; candidate_slug text; candidate_group public.groups%ROWTYPE; selected boolean:=false;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF current_user IS DISTINCT FROM 'console_app' OR TG_OP IN ('UPDATE','TRUNCATE') THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 IF TG_OP='INSERT' THEN b:=NEW; ELSE b:=OLD; END IF;
 IF b.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR b.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(b.input_bytes);
 IF ROW(decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest)
   IS DISTINCT FROM ROW(b.actor_user_id,b.command_id,b.kind,b.input_digest)
  OR b.kind NOT IN (1,4,5)
  OR (b.kind IN (4,5) AND decoded.payload->>'org_id' IS DISTINCT FROM b.org_id::text)
  OR (b.kind=4 AND decoded.payload->>'group_id' IS DISTINCT FROM b.group_id::text)
  OR (b.kind=5 AND decoded.payload->>'group_id' IS DISTINCT FROM b.old_group_id::text) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 IF TG_OP='DELETE' THEN
  SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
   WHERE x.actor_user_id=b.actor_user_id AND x.command_id=b.command_id;
  IF ROW(r.receipt_id,r.codec_version,r.kind,r.input_bytes,r.input_digest,
     r.primary_target,r.outcome,r.result_code,r.result_org_id,r.result_group_id,
     r.effect_family_id,r.effect_xid,r.effect_backend_pid,r.occurred_at)
    IS DISTINCT FROM ROW(b.receipt_id,1::smallint,b.kind,b.input_bytes,b.input_digest,
     decoded.primary_target,'APPLIED'::text,'applied'::text,b.org_id,b.group_id,
     b.effect_family_id,b.effect_xid,b.effect_backend_pid,b.occurred_at) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
  RETURN OLD;
 END IF;
 IF EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts x
   WHERE (x.actor_user_id=b.actor_user_id AND x.command_id=b.command_id) OR x.receipt_id=b.receipt_id)
  OR EXISTS(SELECT 1 FROM public.groups x WHERE x.id IN (b.group_id,b.old_group_id)
   AND x.origin_account_id IS NOT NULL) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 -- Only the decoded private current-transaction binding now selects Company RLS.
 PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.* INTO o FROM public.organizations x WHERE x.id=b.org_id;
 SELECT x.* INTO m FROM public.group_memberships x WHERE x.org_id=b.org_id;
 IF b.kind=1 THEN
  IF o.id IS NOT NULL OR m.org_id IS NOT NULL OR b.company_status IS DISTINCT FROM 'ACTIVE'
   OR b.old_group_id IS NOT NULL OR b.old_membership_id IS NOT NULL
   OR b.old_membership_revision IS NOT NULL OR b.old_membership_incarnation IS NOT NULL THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
 ELSE
  IF ROW(o.id,o.group_id,o.status,o.origin_account_id,o.origin_command_id,o.origin_receipt_id)
    IS DISTINCT FROM ROW(b.org_id,b.old_group_id,b.company_status,NULL::uuid,NULL::uuid,NULL::uuid)
   OR ROW(m.org_id,m.group_id,m.membership_id,m.current_revision,m.incarnation)
    IS DISTINCT FROM ROW(b.org_id,b.old_group_id,b.old_membership_id,b.old_membership_revision,b.old_membership_incarnation) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
  SELECT x.* INTO h FROM public.group_membership_revisions x
   WHERE x.membership_id=b.old_membership_id AND x.revision=b.old_membership_revision;
  IF ROW(h.group_id,h.org_id,h.membership_id,h.revision,h.incarnation,h.state,h.to_time)
    IS DISTINCT FROM ROW(b.old_group_id,b.org_id,b.old_membership_id,b.old_membership_revision,
      b.old_membership_incarnation,'ACTIVE'::text,NULL::timestamptz)
   OR b.occurred_at<h.from_time THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
 END IF;
 IF b.kind=4 AND b.old_group_id=b.group_id THEN
  IF ROW(b.membership_id,b.membership_incarnation)
   IS DISTINCT FROM ROW(b.old_membership_id,b.old_membership_incarnation) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
 ELSIF EXISTS(SELECT 1 FROM public.group_membership_revisions x WHERE x.membership_id=b.membership_id)
  OR b.membership_id=b.old_membership_id OR b.membership_incarnation=b.old_membership_incarnation THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 IF b.group_is_new AND (b.kind=4 OR EXISTS(SELECT 1 FROM public.groups x WHERE x.id=b.group_id)
   OR EXISTS(SELECT 1 FROM public.group_authority_heads x WHERE x.group_id=b.group_id)) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 IF b.kind IN (1,5) THEN
  FOR suffix IN 1..9 LOOP
   candidate_slug:=(CASE WHEN suffix=1 THEN 'go-' ELSE 'g'||suffix::text||'-' END)
    ||replace(b.org_id::text,'-','');
   SELECT x.* INTO candidate_group FROM public.groups x WHERE x.slug=candidate_slug;
   IF candidate_group.id IS NULL THEN
    IF b.mint_suffix IS DISTINCT FROM suffix OR b.group_is_new IS DISTINCT FROM true THEN
     RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
    END IF;
    selected:=true; EXIT;
   ELSIF NOT EXISTS(SELECT 1 FROM public.group_memberships x
     WHERE x.group_id=candidate_group.id AND x.org_id<>b.org_id) THEN
    IF b.mint_suffix IS DISTINCT FROM suffix OR b.group_is_new IS DISTINCT FROM false
     OR b.group_id IS DISTINCT FROM candidate_group.id OR candidate_group.origin_account_id IS NOT NULL THEN
     RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
    END IF;
    selected:=true; EXIT;
   END IF;
  END LOOP;
  IF NOT selected THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
 END IF;
 -- Building the exact closed array from real rows validates types, spelling,
 -- key census, order, uniqueness, status and revision without unchecked casts.
 WITH wanted AS (
  SELECT DISTINCT id FROM unnest(ARRAY[b.old_group_id,b.group_id]) ids(id)
  WHERE id IS NOT NULL AND NOT (b.group_is_new AND id=b.group_id)
 ) SELECT count(*) INTO expected_count FROM wanted;
 WITH wanted AS (
  SELECT DISTINCT id FROM unnest(ARRAY[b.old_group_id,b.group_id]) ids(id)
  WHERE id IS NOT NULL AND NOT (b.group_is_new AND id=b.group_id)
 ) SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',g.id::text,
    'incarnation',a.incarnation::text,'revision',a.revision::text,'state',a.state,
    'group_status',g.status) ORDER BY g.id),'[]'::jsonb)
  INTO expected_before FROM wanted w JOIN public.groups g ON g.id=w.id
   JOIN public.group_authority_heads a ON a.group_id=g.id;
 IF expected_before IS DISTINCT FROM b.heads_before
  OR jsonb_array_length(expected_before)<>expected_count THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
 END IF;
 FOR item IN SELECT value FROM jsonb_array_elements(decoded.expected_groups) LOOP
  IF NOT EXISTS(SELECT 1 FROM jsonb_array_elements(expected_before) actual
   WHERE actual.value-ARRAY['state','group_status']::text[]=item) THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_membership.binding_invalid';
  END IF;
 END LOOP;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.platform_legacy_membership_binding_v1(p_org uuid,p_group uuid)
RETURNS SETOF public.platform_legacy_membership_effect_bindings
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b public.platform_legacy_membership_effect_bindings%ROWTYPE; decoded record;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 -- Company-only lookup first: a stale or mismatched row is never ignored.
 SELECT x.* INTO STRICT b FROM public.platform_legacy_membership_effect_bindings x WHERE x.org_id=p_org;
 IF b.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR b.effect_backend_pid IS DISTINCT FROM pg_backend_pid() OR b.group_id IS DISTINCT FROM p_group
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts r
   WHERE (r.actor_user_id=b.actor_user_id AND r.command_id=b.command_id) OR r.receipt_id=b.receipt_id)
  OR EXISTS(SELECT 1 FROM public.groups g WHERE g.id IN (b.group_id,b.old_group_id) AND g.origin_account_id IS NOT NULL) THEN
  RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
 END IF;
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(b.input_bytes);
 IF ROW(decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest)
    IS DISTINCT FROM ROW(b.actor_user_id,b.command_id,b.kind,b.input_digest)
  OR b.kind NOT IN (1,4,5)
  OR (b.kind IN (4,5) AND decoded.payload->>'org_id' IS DISTINCT FROM b.org_id::text)
  OR (b.kind=4 AND decoded.payload->>'group_id' IS DISTINCT FROM b.group_id::text)
  OR (b.kind=5 AND decoded.payload->>'group_id' IS DISTINCT FROM b.old_group_id::text) THEN
  RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
 END IF;
 -- Validate the private retained identity before arming its Company. The
 -- caller's prior Company scope must not hide a native-origin target here.
 PERFORM set_config('app.current_org',b.org_id::text,true);
 IF EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=b.org_id
   AND (o.origin_account_id IS NOT NULL OR o.origin_command_id IS NOT NULL
    OR o.origin_receipt_id IS NOT NULL OR o.status IS DISTINCT FROM b.company_status))
  OR (b.kind<>1 AND NOT EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=b.org_id)) THEN
  RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEXT b;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

ALTER FUNCTION public.platform_legacy_membership_binding_write_guard_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_legacy_membership_binding_v1(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_membership_binding_write_guard_v1(),
 public.platform_legacy_membership_binding_v1(uuid,uuid) FROM PUBLIC,
 console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,console_ontology_cmd,
 console_platform_force_cmd,console_account_owner,console_terms_owner,console_credential_owner,
 console_ontology_writer;
CREATE TRIGGER trg_platform_legacy_membership_binding_write
 BEFORE INSERT OR UPDATE OR DELETE ON public.platform_legacy_membership_effect_bindings
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_membership_binding_write_guard_v1();
CREATE TRIGGER trg_platform_legacy_membership_binding_truncate
 BEFORE TRUNCATE ON public.platform_legacy_membership_effect_bindings
 FOR EACH STATEMENT EXECUTE FUNCTION public.platform_legacy_membership_binding_write_guard_v1();
ALTER TABLE public.platform_legacy_membership_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_membership_binding_write;
ALTER TABLE public.platform_legacy_membership_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_membership_binding_truncate;


-- source: legacy-membership-core.sql
-- Private source. Same canonical membership writer for native birth and legacy
-- create/move/remint. Activate only with the complete frame/coordinator/ACLs.
CREATE OR REPLACE FUNCTION public.platform_attach_membership(p_group_id uuid,p_org_id uuid)
RETURNS void LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE organization_row public.organizations%ROWTYPE; b record;
 prior public.group_memberships%ROWTYPE; old_history public.group_membership_revisions%ROWTYPE;
 member uuid; incarnation_value uuid; event_time timestamptz;
BEGIN
 SELECT x.* INTO STRICT organization_row FROM public.organizations x WHERE x.id=p_org_id;
 SELECT x.* INTO prior FROM public.group_memberships x WHERE x.org_id=p_org_id;
 IF organization_row.origin_account_id IS NOT NULL THEN
  -- Preserve the existing native participant and its own binding authority.
  IF prior.group_id=p_group_id THEN RETURN; END IF;
  member:=gen_random_uuid(); incarnation_value:=gen_random_uuid(); event_time:=clock_timestamp();
  SELECT * INTO STRICT b FROM public.company_enrollment_binding_v1(organization_row.origin_account_id,organization_row.origin_command_id);
  IF b.request_state<>'PENDING' OR b.group_id IS DISTINCT FROM p_group_id OR b.org_id IS DISTINCT FROM p_org_id OR prior.org_id IS NOT NULL THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
  event_time:=b.started_at;
  INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
   state,provenance_kind,native_account_id,legacy_actor_user_id,command_id,command_receipt)
  VALUES(p_group_id,p_org_id,member,1,incarnation_value,event_time,NULL,'ACTIVE','COMPANY_ENROLLMENT_V1',b.account_id,NULL,b.command_id,b.receipt_id);
 ELSE
  SELECT * INTO STRICT b FROM public.platform_legacy_membership_binding_v1(p_org_id,p_group_id);
  IF b.kind=4 AND b.old_group_id=b.group_id THEN
   IF ROW(prior.group_id,prior.org_id,prior.membership_id,prior.current_revision,prior.incarnation)
    IS DISTINCT FROM ROW(b.old_group_id,b.org_id,b.old_membership_id,b.old_membership_revision,b.old_membership_incarnation) THEN
    RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
   END IF;
   RETURN;
  END IF;
  IF (b.kind IN (1,5) AND prior.org_id IS NOT NULL)
   OR (b.kind=4 AND ROW(prior.group_id,prior.org_id,prior.membership_id,prior.current_revision,prior.incarnation)
    IS DISTINCT FROM ROW(b.old_group_id,b.org_id,b.old_membership_id,b.old_membership_revision,b.old_membership_incarnation)) THEN
   RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
  END IF;
  IF b.kind IN (4,5) THEN
   SELECT x.* INTO STRICT old_history FROM public.group_membership_revisions x
    WHERE x.membership_id=b.old_membership_id AND x.revision=b.old_membership_revision;
   IF ROW(old_history.group_id,old_history.org_id,old_history.incarnation,old_history.state,old_history.to_time)
    IS DISTINCT FROM ROW(b.old_group_id,b.org_id,b.old_membership_incarnation,'ACTIVE'::text,NULL::timestamptz) THEN
    RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
   END IF;
   INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
    state,provenance_kind,native_account_id,legacy_actor_user_id,command_id,command_receipt)
   VALUES(b.old_group_id,p_org_id,b.old_membership_id,b.old_membership_revision+1,b.old_membership_incarnation,
    old_history.from_time,b.occurred_at,'REMOVED','LEGACY_TOPOLOGY_V1',NULL,b.actor_user_id,b.command_id,b.receipt_id);
  END IF;
  member:=b.membership_id; incarnation_value:=b.membership_incarnation; event_time:=b.occurred_at;
  INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
   state,provenance_kind,native_account_id,legacy_actor_user_id,command_id,command_receipt)
  VALUES(p_group_id,p_org_id,member,1,incarnation_value,event_time,NULL,'ACTIVE','LEGACY_TOPOLOGY_V1',NULL,
   b.actor_user_id,b.command_id,b.receipt_id);
 END IF;
 INSERT INTO public.group_memberships(group_id,org_id,created_at,membership_id,current_revision,incarnation)
 VALUES(p_group_id,p_org_id,event_time,member,1,incarnation_value)
 ON CONFLICT(org_id) DO UPDATE SET group_id=EXCLUDED.group_id,created_at=EXCLUDED.created_at,
  membership_id=EXCLUDED.membership_id,current_revision=EXCLUDED.current_revision,incarnation=EXCLUDED.incarnation;
END
$body$;

CREATE OR REPLACE FUNCTION public.platform_mint_group_row(p_org_id uuid,p_name text,p_status text)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE b public.platform_legacy_membership_effect_bindings%ROWTYPE; selected_group uuid;
 candidate public.groups%ROWTYPE; suffix integer; candidate_slug text; inserted uuid;
BEGIN
 -- Automatic BEFORE Company INSERT has no per-command trigger arguments.
 -- Resolve the actual Company row's unique private frame, then apply its exact
 -- full-xid/PID/decoded-command lookup. Never choose a permissive fallback.
 SELECT x.group_id INTO STRICT selected_group
  FROM public.platform_legacy_membership_effect_bindings x WHERE x.org_id=p_org_id;
 SELECT * INTO STRICT b FROM public.platform_legacy_membership_binding_v1(p_org_id,selected_group);
 IF b.kind NOT IN (1,5) OR p_status IS DISTINCT FROM b.company_status
  OR p_org_id='00000000-0000-0000-0000-00000000face'::uuid THEN
  RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
 END IF;
 IF b.kind=1 THEN
  IF p_name IS DISTINCT FROM (SELECT d.payload->>'name'
    FROM public.platform_legacy_topology_decode_input_v1(b.input_bytes) d) THEN
   RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
  END IF;
 ELSIF NOT EXISTS(SELECT 1 FROM public.organizations o
   WHERE o.id=p_org_id AND o.name=p_name AND o.status=p_status
    AND o.origin_account_id IS NULL AND o.origin_command_id IS NULL AND o.origin_receipt_id IS NULL) THEN
  RAISE EXCEPTION 'platform_legacy_membership.binding_invalid';
 END IF;
 FOR suffix IN 1..9 LOOP
  candidate_slug:=(CASE WHEN suffix=1 THEN 'go-' ELSE 'g'||suffix::text||'-' END)
   ||replace(p_org_id::text,'-','');
  SELECT x.* INTO candidate FROM public.groups x WHERE x.slug=candidate_slug;
  IF candidate.id IS NOT NULL THEN
   IF EXISTS(SELECT 1 FROM public.group_memberships m
     WHERE m.group_id=candidate.id AND m.org_id<>p_org_id) THEN CONTINUE; END IF;
   IF suffix IS DISTINCT FROM b.mint_suffix OR b.group_is_new
    OR candidate.id IS DISTINCT FROM b.group_id OR candidate.origin_account_id IS NOT NULL THEN
    RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
   END IF;
   RETURN candidate.id;
  END IF;
  IF suffix IS DISTINCT FROM b.mint_suffix OR NOT b.group_is_new THEN
   RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
  END IF;
  INSERT INTO public.groups(id,slug,name,status) VALUES(b.group_id,candidate_slug,p_name,p_status)
   ON CONFLICT(slug) DO NOTHING RETURNING id INTO inserted;
  IF inserted IS DISTINCT FROM b.group_id THEN
   -- The historical owner could adopt a concurrent winner here. That winner
   -- was not in this attempt's earlier Group lockset: roll back and replan.
   RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
  END IF;
  RETURN inserted;
 END LOOP;
 RAISE EXCEPTION USING ERRCODE='23505',MESSAGE='platform_topology.group_slug_exhausted';
END
$body$;

CREATE OR REPLACE FUNCTION public.platform_assign_org_to_group(p_group_id uuid,p_org_id uuid)
RETURNS uuid LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE group_status text; found_org uuid;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT g.status INTO group_status FROM public.groups g WHERE g.id=p_group_id;
 IF group_status IS NULL THEN RETURN NULL; END IF;
 IF group_status<>'ACTIVE' THEN
  RAISE EXCEPTION 'group % is not active',p_group_id USING ERRCODE='23514';
 END IF;
 PERFORM set_config('app.current_org',p_org_id::text,true);
 SELECT o.id INTO found_org FROM public.organizations o
  WHERE o.id=p_org_id AND o.id<>'00000000-0000-0000-0000-00000000face'::uuid;
 IF found_org IS NOT NULL THEN
  -- The old direct UPSERT now shares the one membership/history participant.
  PERFORM public.platform_attach_membership(p_group_id,p_org_id);
  UPDATE public.organizations SET group_id=p_group_id,updated_at=now() WHERE id=p_org_id;
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN found_org;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

ALTER FUNCTION public.platform_attach_membership(uuid,uuid) OWNER TO console_app;
ALTER FUNCTION public.platform_mint_group_row(uuid,text,text) OWNER TO console_app;
ALTER FUNCTION public.platform_assign_org_to_group(uuid,uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_attach_membership(uuid,uuid),
 public.platform_mint_group_row(uuid,text,text),public.platform_assign_org_to_group(uuid,uuid)
 FROM PUBLIC,console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,
 console_ontology_cmd,console_platform_force_cmd,console_account_owner,
 console_terms_owner,console_credential_owner,console_ontology_writer;


-- source: legacy-membership-closure.sql
-- Private candidate; retained NEW proves closure after the live frame is gone.
CREATE FUNCTION public.platform_legacy_membership_binding_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_legacy_topology_receipts%ROWTYPE; decoded record;
 old_history public.group_membership_revisions%ROWTYPE;
 removed_history public.group_membership_revisions%ROWTYPE;
 active_history public.group_membership_revisions%ROWTYPE;
 m public.group_memberships%ROWTYPE; o public.organizations%ROWTYPE;
 g public.groups%ROWTYPE; a public.group_authority_heads%ROWTYPE;
 item jsonb; projected_before jsonb; actual_after jsonb;
 no_change boolean:=NEW.kind=4 AND NEW.old_group_id=NEW.group_id;
 expected_history_count integer; final_revision bigint;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF NEW.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR EXISTS(SELECT 1 FROM public.platform_legacy_membership_effect_bindings b
   WHERE b.org_id=NEW.org_id OR (b.actor_user_id=NEW.actor_user_id AND b.command_id=NEW.command_id)) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.actor_user_id=NEW.actor_user_id AND x.command_id=NEW.command_id;
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(NEW.input_bytes);
 IF ROW(decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest)
    IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.kind,NEW.input_digest)
  OR ROW(r.receipt_id,r.codec_version,r.kind,r.input_bytes,r.input_digest,r.primary_target,
     r.outcome,r.result_code,r.result_org_id,r.result_group_id,
     r.effect_family_id,r.effect_xid,r.effect_backend_pid,r.occurred_at)
    IS DISTINCT FROM ROW(NEW.receipt_id,1::smallint,NEW.kind,NEW.input_bytes,NEW.input_digest,decoded.primary_target,
     'APPLIED'::text,'applied'::text,NEW.org_id,NEW.group_id,
     NEW.effect_family_id,NEW.effect_xid,NEW.effect_backend_pid,NEW.occurred_at) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 -- No client label or current mutable GUC establishes this disclosure scope.
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 SELECT x.* INTO STRICT o FROM public.organizations x WHERE x.id=NEW.org_id;
 SELECT x.* INTO STRICT m FROM public.group_memberships x WHERE x.org_id=NEW.org_id;
 IF ROW(o.group_id,o.status,o.origin_account_id,o.origin_command_id,o.origin_receipt_id)
    IS DISTINCT FROM ROW(NEW.group_id,NEW.company_status,NULL::uuid,NULL::uuid,NULL::uuid)
  OR (NEW.kind=1 AND (o.name IS DISTINCT FROM decoded.payload->>'name'
   OR o.slug IS DISTINCT FROM decoded.payload->>'slug')) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 final_revision:=CASE WHEN no_change THEN NEW.old_membership_revision ELSE 1 END;
 IF ROW(m.group_id,m.membership_id,m.current_revision,m.incarnation)
   IS DISTINCT FROM ROW(NEW.group_id,NEW.membership_id,final_revision,NEW.membership_incarnation)
  OR (NOT no_change AND m.created_at IS DISTINCT FROM NEW.occurred_at) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 SELECT x.* INTO STRICT active_history FROM public.group_membership_revisions x
  WHERE x.membership_id=NEW.membership_id AND x.revision=final_revision;
 IF ROW(active_history.group_id,active_history.org_id,active_history.membership_id,
     active_history.revision,active_history.incarnation,active_history.state,active_history.to_time)
    IS DISTINCT FROM ROW(NEW.group_id,NEW.org_id,NEW.membership_id,
     final_revision,NEW.membership_incarnation,'ACTIVE'::text,NULL::timestamptz) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 IF NEW.kind IN (4,5) THEN
  SELECT x.* INTO STRICT old_history FROM public.group_membership_revisions x
   WHERE x.membership_id=NEW.old_membership_id AND x.revision=NEW.old_membership_revision;
  IF ROW(old_history.group_id,old_history.org_id,old_history.membership_id,old_history.revision,
      old_history.incarnation,old_history.state,old_history.to_time)
     IS DISTINCT FROM ROW(NEW.old_group_id,NEW.org_id,NEW.old_membership_id,NEW.old_membership_revision,
      NEW.old_membership_incarnation,'ACTIVE'::text,NULL::timestamptz) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
  END IF;
 END IF;
 IF no_change THEN
  expected_history_count:=0;
  IF active_history IS DISTINCT FROM old_history OR m.created_at IS DISTINCT FROM old_history.from_time THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
  END IF;
 ELSE
  expected_history_count:=1;
  IF ROW(active_history.from_time,active_history.provenance_kind,active_history.native_account_id,
      active_history.legacy_actor_user_id,active_history.command_id,active_history.command_receipt)
     IS DISTINCT FROM ROW(NEW.occurred_at,'LEGACY_TOPOLOGY_V1'::text,NULL::uuid,
      NEW.actor_user_id,NEW.command_id,NEW.receipt_id) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
  END IF;
  IF NEW.kind IN (4,5) THEN
   expected_history_count:=2;
   SELECT x.* INTO STRICT removed_history FROM public.group_membership_revisions x
    WHERE x.membership_id=NEW.old_membership_id AND x.revision=NEW.old_membership_revision+1;
   IF ROW(removed_history.group_id,removed_history.org_id,removed_history.membership_id,
       removed_history.revision,removed_history.incarnation,removed_history.from_time,
       removed_history.to_time,removed_history.state,removed_history.provenance_kind,
       removed_history.native_account_id,removed_history.legacy_actor_user_id,
       removed_history.command_id,removed_history.command_receipt)
      IS DISTINCT FROM ROW(NEW.old_group_id,NEW.org_id,NEW.old_membership_id,
       NEW.old_membership_revision+1,NEW.old_membership_incarnation,old_history.from_time,
       NEW.occurred_at,'REMOVED'::text,'LEGACY_TOPOLOGY_V1'::text,
       NULL::uuid,NEW.actor_user_id,NEW.command_id,NEW.receipt_id) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
   END IF;
  END IF;
 END IF;
 IF (SELECT count(*) FROM public.group_membership_revisions x
   WHERE x.legacy_actor_user_id=NEW.actor_user_id AND x.command_id=NEW.command_id)<>expected_history_count THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 SELECT coalesce(jsonb_agg(value-ARRAY['state','group_status']::text[] ORDER BY value->>'group_id'),'[]'::jsonb)
  INTO projected_before FROM jsonb_array_elements(NEW.heads_before);
 IF r.heads_before IS DISTINCT FROM projected_before THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 FOR item IN SELECT value FROM jsonb_array_elements(NEW.heads_before) LOOP
  SELECT x.* INTO STRICT a FROM public.group_authority_heads x WHERE x.group_id::text=item->>'group_id';
  SELECT x.* INTO STRICT g FROM public.groups x WHERE x.id=a.group_id;
  IF jsonb_build_object('group_id',a.group_id::text,'incarnation',a.incarnation::text,
      'revision',(a.revision-CASE WHEN no_change THEN 0 ELSE 1 END)::text,
      'state',a.state,'group_status',g.status) IS DISTINCT FROM item
   OR g.origin_account_id IS NOT NULL OR g.origin_command_id IS NOT NULL OR g.origin_receipt_id IS NOT NULL THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
  END IF;
 END LOOP;
 SELECT x.* INTO STRICT g FROM public.groups x WHERE x.id=NEW.group_id;
 IF g.origin_account_id IS NOT NULL OR g.origin_command_id IS NOT NULL OR g.origin_receipt_id IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 IF NEW.kind IN (1,5) AND g.slug IS DISTINCT FROM
   (CASE WHEN NEW.mint_suffix=1 THEN 'go-' ELSE 'g'||NEW.mint_suffix::text||'-' END)
    ||replace(NEW.org_id::text,'-','') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 IF NEW.group_is_new THEN
  SELECT x.* INTO STRICT a FROM public.group_authority_heads x WHERE x.group_id=NEW.group_id;
  IF a.revision IS DISTINCT FROM 1 OR a.state IS DISTINCT FROM 'ACTIVE'
   OR a.incarnation IS NULL OR a.incarnation='00000000-0000-0000-0000-000000000000'::uuid
   OR g.status IS DISTINCT FROM NEW.company_status OR g.name IS DISTINCT FROM o.name THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
  END IF;
 END IF;
 SELECT jsonb_agg(jsonb_build_object('group_id',x.group_id::text,
   'incarnation',x.incarnation::text,'revision',x.revision::text) ORDER BY x.group_id)
  INTO actual_after FROM public.group_authority_heads x
  WHERE x.group_id IN (NEW.old_group_id,NEW.group_id);
 IF r.heads_after IS DISTINCT FROM actual_after THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
 END IF;
 -- The canonical receipt's independent deferred closure verifies the actual
 -- audit/credential/catalog participants for every legacy command kind.
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_membership.closure_invalid';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_membership_binding_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_membership_binding_closed_v1() FROM PUBLIC,
 console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,console_ontology_cmd,
 console_platform_force_cmd,console_account_owner,console_terms_owner,console_credential_owner,
 console_ontology_writer;
CREATE CONSTRAINT TRIGGER trg_platform_legacy_membership_binding_closed
 AFTER INSERT ON public.platform_legacy_membership_effect_bindings
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
 EXECUTE FUNCTION public.platform_legacy_membership_binding_closed_v1();


-- source: legacy-catalog-binding-guard.sql
CREATE FUNCTION public.platform_legacy_catalog_binding_write_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_value public.platform_legacy_catalog_effect_bindings%ROWTYPE;
BEGIN
 IF current_user IS DISTINCT FROM 'console_app' OR TG_OP IN ('UPDATE','TRUNCATE') THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_catalog.binding_invalid';
 END IF;
 IF TG_OP='INSERT' THEN row_value:=NEW; ELSE row_value:=OLD; END IF;
 IF row_value.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR row_value.effect_backend_pid IS DISTINCT FROM pg_backend_pid() THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_catalog.binding_invalid';
 END IF;
 IF TG_OP='INSERT' AND (
  row_value.catalog_version IS DISTINCT FROM '2026-07-19.1'
  OR row_value.manifest_digest IS DISTINCT FROM decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex')
  OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts r
   WHERE r.actor_user_id=NEW.actor_user_id AND r.command_id=NEW.command_id)) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_legacy_catalog.binding_invalid';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 RETURN NEW;
END
$body$;
ALTER FUNCTION public.platform_legacy_catalog_binding_write_guard_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_catalog_binding_write_guard_v1() FROM PUBLIC;

CREATE TRIGGER trg_platform_legacy_catalog_binding_write
 BEFORE INSERT OR UPDATE OR DELETE ON public.platform_legacy_catalog_effect_bindings
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_catalog_binding_write_guard_v1();
CREATE TRIGGER trg_platform_legacy_catalog_binding_truncate
 BEFORE TRUNCATE ON public.platform_legacy_catalog_effect_bindings
 FOR EACH STATEMENT EXECUTE FUNCTION public.platform_legacy_catalog_binding_write_guard_v1();
ALTER TABLE public.platform_legacy_catalog_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_catalog_binding_write;
ALTER TABLE public.platform_legacy_catalog_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_catalog_binding_truncate;


-- source: legacy-catalog-audit.sql
CREATE FUNCTION public.platform_legacy_catalog_live_audit_v1(p_event public.audit_events) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE proof record; decoded record; valid boolean;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF p_event.action IS DISTINCT FROM 'ontology.object_type.builtin_install'
  OR p_event.target_type IS DISTINCT FROM 'ont_object_types' OR p_event.before_snap IS NOT NULL
  OR p_event.branch_id IS NOT NULL OR p_event.org_id IS NULL OR p_event.actor IS NULL
  OR p_event.target_id IS NULL OR p_event.occurred_at IS NULL OR NOT isfinite(p_event.occurred_at)
  OR p_event.trace_id IS NULL OR p_event.span_id IS NULL THEN RETURN false; END IF;
 SELECT b.* INTO proof FROM public.platform_legacy_catalog_effect_bindings b
  WHERE b.org_id=p_event.org_id AND b.admin_user_id=p_event.actor
   AND b.effect_xid=pg_current_xact_id() AND b.effect_backend_pid=pg_backend_pid()
   AND b.trace_id=p_event.trace_id AND b.span_id=p_event.span_id AND b.catalog_at=p_event.occurred_at
   AND b.catalog_version='2026-07-19.1'
   AND b.manifest_digest=decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex')
   AND NOT EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts r
    WHERE r.actor_user_id=b.actor_user_id AND r.command_id=b.command_id);
 IF NOT FOUND THEN RETURN false; END IF;
 -- Scope comes only from the validated private binding/current receipt.
 PERFORM set_config('app.current_org',proof.org_id::text,true);
 SELECT EXISTS(
  SELECT 1 FROM public.ont_object_types o
  JOIN public.organizations company ON company.id=o.org_id
  JOIN public.users admin ON admin.id=proof.admin_user_id AND admin.org_id=company.id
  WHERE o.org_id=proof.org_id AND o.id::text=p_event.target_id
   AND company.origin_account_id IS NULL AND company.origin_command_id IS NULL AND company.origin_receipt_id IS NULL
   AND company.status='ACTIVE' AND admin.is_active
   AND o.created_by=proof.admin_user_id AND o.attribution_protocol='LEGACY_USER'
   AND o.created_by_account_id IS NULL AND o.origin_account_id IS NULL
   AND o.origin_command_id IS NULL AND o.origin_receipt_id IS NULL
   AND o.created_at=proof.catalog_at AND o.updated_at=proof.catalog_at
   AND o.schema_version=1 AND o.lifecycle_state='published'
   AND o.stable_key IN ('support_slo_setting','console_view','sla_setting','handover_policy','shift_timetable','labor_refusal','regulation_param','site_coverage','profitability_analytic','contract','position','posting','customer','site','equipment','employee','work_order','approval','support_ticket','evidence','compliance_obligation','compliance_regulation','compliance_framework','leave_request','workflow_definition','messenger_thread','mail')
   AND p_event.after_snap=jsonb_build_object('stable_key',o.stable_key,'schema_version',1,
    'lifecycle_state','published','catalog_version',proof.catalog_version,
    'manifest_digest',encode(proof.manifest_digest,'hex'))
   AND EXISTS(SELECT 1 FROM public.ont_builtin_catalog_allowlist a
    WHERE a.catalog_version=proof.catalog_version AND a.manifest_digest=proof.manifest_digest)
 ) INTO valid;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN valid;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_catalog_live_audit_v1(public.audit_events) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_catalog_live_audit_v1(public.audit_events) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.platform_legacy_catalog_live_audit_v1(public.audit_events) TO console_ontology_writer;

CREATE FUNCTION public.platform_legacy_catalog_receipt_audit_v1(p_event public.audit_events) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE proof record; decoded record; valid boolean;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF p_event.action IS DISTINCT FROM 'ontology.object_type.builtin_install'
  OR p_event.target_type IS DISTINCT FROM 'ont_object_types' OR p_event.before_snap IS NOT NULL
  OR p_event.branch_id IS NOT NULL OR p_event.org_id IS NULL OR p_event.actor IS NULL
  OR p_event.target_id IS NULL OR p_event.occurred_at IS NULL OR NOT isfinite(p_event.occurred_at)
  OR p_event.trace_id IS NULL OR p_event.span_id IS NULL THEN RETURN false; END IF;
 BEGIN
 SELECT r.actor_user_id,r.command_id,r.receipt_id,r.input_bytes,r.input_digest,r.primary_target,
   r.result_org_id AS org_id,r.result_user_id AS admin_user_id,r.occurred_at AS catalog_at,
   r.builtin_catalog_version AS catalog_version,r.builtin_catalog_digest AS manifest_digest
  INTO STRICT proof FROM public.platform_legacy_topology_receipts r
  WHERE r.result_org_id=p_event.org_id AND r.result_user_id=p_event.actor
   AND r.kind=1 AND r.codec_version=1 AND r.outcome='APPLIED' AND r.result_code='applied'
   AND r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid()
   AND r.trace_id=p_event.trace_id AND r.span_id=p_event.span_id AND r.occurred_at=p_event.occurred_at
   AND r.builtin_catalog_version='2026-07-19.1'
   AND r.builtin_catalog_digest=decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex');
 EXCEPTION WHEN NO_DATA_FOUND THEN RETURN false; END;
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(proof.input_bytes);
 IF decoded.kind IS DISTINCT FROM 1 OR decoded.actor_user_id IS DISTINCT FROM proof.actor_user_id
  OR decoded.command_id IS DISTINCT FROM proof.command_id OR decoded.primary_target IS NOT NULL
  OR proof.primary_target IS NOT NULL OR decoded.input_digest IS DISTINCT FROM proof.input_digest THEN RETURN false; END IF;
 -- Scope comes only from the validated private binding/current receipt.
 PERFORM set_config('app.current_org',proof.org_id::text,true);
 SELECT EXISTS(
  SELECT 1 FROM public.ont_object_types o
  JOIN public.organizations company ON company.id=o.org_id
  JOIN public.users admin ON admin.id=proof.admin_user_id AND admin.org_id=company.id
  WHERE o.org_id=proof.org_id AND o.id::text=p_event.target_id
   AND company.origin_account_id IS NULL AND company.origin_command_id IS NULL AND company.origin_receipt_id IS NULL
   AND company.status='ACTIVE' AND admin.is_active
   AND o.created_by=proof.admin_user_id AND o.attribution_protocol='LEGACY_USER'
   AND o.created_by_account_id IS NULL AND o.origin_account_id IS NULL
   AND o.origin_command_id IS NULL AND o.origin_receipt_id IS NULL
   AND o.created_at=proof.catalog_at AND o.updated_at=proof.catalog_at
   AND o.schema_version=1 AND o.lifecycle_state='published'
   AND o.stable_key IN ('support_slo_setting','console_view','sla_setting','handover_policy','shift_timetable','labor_refusal','regulation_param','site_coverage','profitability_analytic','contract','position','posting','customer','site','equipment','employee','work_order','approval','support_ticket','evidence','compliance_obligation','compliance_regulation','compliance_framework','leave_request','workflow_definition','messenger_thread','mail')
   AND p_event.after_snap=jsonb_build_object('stable_key',o.stable_key,'schema_version',1,
    'lifecycle_state','published','catalog_version',proof.catalog_version,
    'manifest_digest',encode(proof.manifest_digest,'hex'))
   AND EXISTS(SELECT 1 FROM public.ont_builtin_catalog_allowlist a
    WHERE a.catalog_version=proof.catalog_version AND a.manifest_digest=proof.manifest_digest)
 ) INTO valid;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN valid;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_catalog_receipt_audit_v1(public.audit_events) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_catalog_receipt_audit_v1(public.audit_events) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.platform_legacy_catalog_receipt_audit_v1(public.audit_events) TO console_ontology_writer;


-- source: legacy-catalog-closure-bound.sql
-- Private source; __LEGACY_MANIFEST__ is the separately captured fixed canonical
-- builder output, substituted by the complete reviewed resource generator.
CREATE FUNCTION public.platform_legacy_catalog_binding_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_legacy_topology_receipts%ROWTYPE; decoded record;
 manifest constant jsonb:=$manifest${
  "catalog_version": "2026-07-19.1",
  "object_types": [
    {
      "stable_key": "support_slo_setting",
      "title": "SLO 설정",
      "title_property_key": "ticket_type",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "ticket_type",
          "title": "티켓 유형",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "incident",
                "name": "장애"
              },
              {
                "id": "request",
                "name": "요청"
              },
              {
                "id": "change",
                "name": "변경"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "threshold_minutes",
          "title": "임계(분)",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "window",
          "title": "적용 시간",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "business_hours",
                "name": "업무시간"
              },
              {
                "id": "calendar",
                "name": "24x7"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "escalation_target",
          "title": "에스컬레이션 대상",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "ticket_type": {
              "required": true
            },
            "threshold_minutes": {
              "required": true
            },
            "window": {
              "required": true
            },
            "escalation_target": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "ticket_type",
              "param": "ticket_type"
            },
            {
              "property": "threshold_minutes",
              "param": "threshold_minutes"
            },
            {
              "property": "window",
              "param": "window"
            },
            {
              "property": "escalation_target",
              "param": "escalation_target"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "console_view",
      "title": "콘솔 뷰",
      "title_property_key": "screen_key",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "screen_key",
          "title": "화면",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "config",
          "title": "레이아웃",
          "field_type": "json",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "scope",
          "title": "범위",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "personal",
                "name": "개인"
              },
              {
                "id": "team",
                "name": "팀"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "screen_key": {
              "required": true
            },
            "config": {
              "required": true
            },
            "scope": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "screen_key",
              "param": "screen_key"
            },
            {
              "property": "config",
              "param": "config"
            },
            {
              "property": "scope",
              "param": "scope"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "sla_setting",
      "title": "SLA 설정",
      "title_property_key": "contract_ref",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "contract_ref",
          "title": "계약/현장 참조",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "tier",
          "title": "등급",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "standard",
                "name": "표준"
              },
              {
                "id": "premium",
                "name": "프리미엄"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "response_minutes",
          "title": "응답(분)",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "resolution_minutes",
          "title": "해결(분)",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "penalty_clause",
          "title": "위약조항",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "contract_ref": {
              "required": true
            },
            "tier": {
              "required": true
            },
            "response_minutes": {
              "required": true
            },
            "resolution_minutes": {
              "required": true
            },
            "penalty_clause": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "contract_ref",
              "param": "contract_ref"
            },
            {
              "property": "tier",
              "param": "tier"
            },
            {
              "property": "response_minutes",
              "param": "response_minutes"
            },
            {
              "property": "resolution_minutes",
              "param": "resolution_minutes"
            },
            {
              "property": "penalty_clause",
              "param": "penalty_clause"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "handover_policy",
      "title": "인수인계 정책",
      "title_property_key": "policy_name",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "policy_name",
          "title": "정책명",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "auto_act",
          "title": "자동조치",
          "field_type": "boolean",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "escalate",
          "title": "에스컬레이션",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "none",
                "name": "없음"
              },
              {
                "id": "supervisor",
                "name": "감독자"
              },
              {
                "id": "duty_manager",
                "name": "당직관리자"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "fit_floor",
          "title": "최소인원 기준",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "dept_heads",
          "title": "부서장",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "policy_name": {
              "required": true
            },
            "auto_act": {
              "required": true
            },
            "escalate": {
              "required": true
            },
            "fit_floor": {
              "required": true
            },
            "dept_heads": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "policy_name",
              "param": "policy_name"
            },
            {
              "property": "auto_act",
              "param": "auto_act"
            },
            {
              "property": "escalate",
              "param": "escalate"
            },
            {
              "property": "fit_floor",
              "param": "fit_floor"
            },
            {
              "property": "dept_heads",
              "param": "dept_heads"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "shift_timetable",
      "title": "교대 시간표",
      "title_property_key": "shift_name",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "shift_name",
          "title": "교대명",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "start_time",
          "title": "시작시각",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "end_time",
          "title": "종료시각",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "days_of_week",
          "title": "적용요일",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "shift_name": {
              "required": true
            },
            "start_time": {
              "required": true
            },
            "end_time": {
              "required": true
            },
            "days_of_week": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "shift_name",
              "param": "shift_name"
            },
            {
              "property": "start_time",
              "param": "start_time"
            },
            {
              "property": "end_time",
              "param": "end_time"
            },
            {
              "property": "days_of_week",
              "param": "days_of_week"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "labor_refusal",
      "title": "노무수령거부",
      "title_property_key": "employee_ref",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "employee_ref",
          "title": "대상 근로자",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "refusal_date",
          "title": "거부일자",
          "field_type": "date",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "reason",
          "title": "사유",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "pending",
                "name": "대기"
              },
              {
                "id": "confirmed",
                "name": "확정"
              },
              {
                "id": "withdrawn",
                "name": "철회"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "employee_ref": {
              "required": true
            },
            "refusal_date": {
              "required": true
            },
            "reason": {
              "required": true
            },
            "status": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "employee_ref",
              "param": "employee_ref"
            },
            {
              "property": "refusal_date",
              "param": "refusal_date"
            },
            {
              "property": "reason",
              "param": "reason"
            },
            {
              "property": "status",
              "param": "status"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "regulation_param",
      "title": "규제 파라미터",
      "title_property_key": "param_key",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "param_key",
          "title": "파라미터",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "min_wage",
                "name": "최저임금"
              },
              {
                "id": "max_weekly_hours",
                "name": "주52시간"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "value",
          "title": "값",
          "field_type": "decimal",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "effective_date",
          "title": "시행일",
          "field_type": "date",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "impact_scope",
          "title": "영향범위",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "impact_note",
          "title": "영향메모",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "param_key": {
              "required": true
            },
            "value": {
              "required": true
            },
            "effective_date": {
              "required": true
            },
            "impact_scope": {
              "required": true
            },
            "impact_note": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "param_key",
              "param": "param_key"
            },
            {
              "property": "value",
              "param": "value"
            },
            {
              "property": "effective_date",
              "param": "effective_date"
            },
            {
              "property": "impact_scope",
              "param": "impact_scope"
            },
            {
              "property": "impact_note",
              "param": "impact_note"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "site_coverage",
      "title": "현장 커버리지",
      "title_property_key": "site_ref",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "site_ref",
          "title": "현장",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "required_headcount",
          "title": "필요인원",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "assigned_headcount",
          "title": "배치인원",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "coverage_date",
          "title": "기준일",
          "field_type": "date",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "site_ref": {
              "required": true
            },
            "required_headcount": {
              "required": true
            },
            "assigned_headcount": {
              "required": true
            },
            "coverage_date": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "site_ref",
              "param": "site_ref"
            },
            {
              "property": "required_headcount",
              "param": "required_headcount"
            },
            {
              "property": "assigned_headcount",
              "param": "assigned_headcount"
            },
            {
              "property": "coverage_date",
              "param": "coverage_date"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "profitability_analytic",
      "title": "수익성 분석",
      "title_property_key": "contract_ref",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "contract_ref",
          "title": "계약",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "revenue",
          "title": "매출",
          "field_type": "decimal",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "cost",
          "title": "원가",
          "field_type": "decimal",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "margin_pct",
          "title": "마진율",
          "field_type": "decimal",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "formula",
          "title": "산식",
          "field_type": "text",
          "config": {
            "expression": "(revenue - cost) / revenue * 100"
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "contract_ref": {
              "required": true
            },
            "revenue": {
              "required": true
            },
            "cost": {
              "required": true
            },
            "margin_pct": {
              "required": true
            },
            "formula": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "contract_ref",
              "param": "contract_ref"
            },
            {
              "property": "revenue",
              "param": "revenue"
            },
            {
              "property": "cost",
              "param": "cost"
            },
            {
              "property": "margin_pct",
              "param": "margin_pct"
            },
            {
              "property": "formula",
              "param": "formula"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "contract",
      "title": "계약",
      "title_property_key": "client",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "client",
          "title": "거래처",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "monthly_fee",
          "title": "월 계약금",
          "field_type": "money",
          "config": {
            "currency": "KRW"
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "period",
          "title": "기간",
          "field_type": "daterange",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "lifecycle",
          "config": {
            "states": [
              {
                "id": "draft",
                "name": "초안"
              },
              {
                "id": "active",
                "name": "활성"
              },
              {
                "id": "expired",
                "name": "만료"
              },
              {
                "id": "terminated",
                "name": "해지"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "margin",
          "title": "마진",
          "field_type": "percent",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "positions",
          "title": "직무",
          "reverse_title": "계약",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "position"
        }
      ],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "client": {
              "required": true
            },
            "monthly_fee": {
              "required": true
            },
            "period": {
              "required": true
            },
            "status": {
              "required": true
            },
            "margin": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "client",
              "param": "client"
            },
            {
              "property": "monthly_fee",
              "param": "monthly_fee"
            },
            {
              "property": "period",
              "param": "period"
            },
            {
              "property": "status",
              "param": "status"
            },
            {
              "property": "margin",
              "param": "margin"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "position",
      "title": "직무",
      "title_property_key": "job_title",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "worksite",
          "title": "현장",
          "field_type": "reference",
          "config": {
            "target": "worksite"
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "job_function",
          "title": "직무",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "job_title",
          "title": "직책",
          "field_type": "text",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "headcount",
          "title": "정원(TO)",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "postings",
          "title": "공고",
          "reverse_title": "직무",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "posting"
        }
      ],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "worksite": {
              "required": true
            },
            "job_function": {
              "required": true
            },
            "job_title": {
              "required": true
            },
            "headcount": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "worksite",
              "param": "worksite"
            },
            {
              "property": "job_function",
              "param": "job_function"
            },
            {
              "property": "job_title",
              "param": "job_title"
            },
            {
              "property": "headcount",
              "param": "headcount"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "posting",
      "title": "채용 공고",
      "title_property_key": "scope",
      "backing_kind": "instance",
      "backing_table": null,
      "primary_key_property": null,
      "properties": [
        {
          "key": "scope",
          "title": "공개 범위",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "internal",
                "name": "내부"
              },
              {
                "id": "external",
                "name": "외부"
              }
            ]
          },
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "fill_count",
          "title": "충원",
          "field_type": "integer",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        },
        {
          "key": "deadline",
          "title": "마감",
          "field_type": "date",
          "config": {},
          "backing_column": null,
          "required": true,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "employee",
          "title": "충원 직원",
          "reverse_title": "공고",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": null
        }
      ],
      "actions": [
        {
          "stable_key": "create",
          "title": "저장",
          "params_schema": {
            "scope": {
              "required": true
            },
            "fill_count": {
              "required": true
            },
            "deadline": {
              "required": true
            }
          },
          "edits": [
            {
              "property": "scope",
              "param": "scope"
            },
            {
              "property": "fill_count",
              "param": "fill_count"
            },
            {
              "property": "deadline",
              "param": "deadline"
            }
          ],
          "submission_criteria": [],
          "side_effects": [],
          "dispatch": "instance_revision",
          "dispatch_target": null,
          "control_points": [
            "authority"
          ]
        }
      ],
      "analytics": []
    },
    {
      "stable_key": "customer",
      "title": "고객",
      "title_property_key": "name",
      "backing_kind": "projected",
      "backing_table": "registry_customers",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "branch_id",
          "title": "지사",
          "field_type": "reference",
          "config": {},
          "backing_column": "branch_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "name",
          "title": "고객명",
          "field_type": "text",
          "config": {},
          "backing_column": "name",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "site",
      "title": "현장",
      "title_property_key": "name",
      "backing_kind": "projected",
      "backing_table": "registry_sites",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "branch_id",
          "title": "지사",
          "field_type": "reference",
          "config": {},
          "backing_column": "branch_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "customer_id",
          "title": "고객",
          "field_type": "reference",
          "config": {},
          "backing_column": "customer_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "name",
          "title": "현장명",
          "field_type": "text",
          "config": {},
          "backing_column": "name",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "customer",
          "title": "고객",
          "reverse_title": "현장",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "customer"
        }
      ],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "equipment",
      "title": "장비",
      "title_property_key": "equipment_no",
      "backing_kind": "projected",
      "backing_table": "registry_equipment",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "equipment_no",
          "title": "장비번호",
          "field_type": "text",
          "config": {},
          "backing_column": "equipment_no",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "branch_id",
          "title": "지사",
          "field_type": "reference",
          "config": {},
          "backing_column": "branch_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "customer_id",
          "title": "고객",
          "field_type": "reference",
          "config": {},
          "backing_column": "customer_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "site_id",
          "title": "현장",
          "field_type": "reference",
          "config": {},
          "backing_column": "site_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "임대",
                "name": "임대"
              },
              {
                "id": "예비",
                "name": "예비"
              },
              {
                "id": "폐기",
                "name": "폐기"
              },
              {
                "id": "대체",
                "name": "대체"
              },
              {
                "id": "매각",
                "name": "매각"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "manufacturer_code",
          "title": "제조사코드",
          "field_type": "text",
          "config": {},
          "backing_column": "manufacturer_code",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "kind_code",
          "title": "종류코드",
          "field_type": "text",
          "config": {},
          "backing_column": "kind_code",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "specification",
          "title": "규격",
          "field_type": "text",
          "config": {},
          "backing_column": "specification",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "customer",
          "title": "고객",
          "reverse_title": "장비",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "customer"
        },
        {
          "stable_key": "site",
          "title": "현장",
          "reverse_title": "장비",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "site"
        }
      ],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "employee",
      "title": "직원",
      "title_property_key": "name",
      "backing_kind": "projected",
      "backing_table": "employees",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "company",
          "title": "회사",
          "field_type": "text",
          "config": {},
          "backing_column": "company",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "name",
          "title": "이름",
          "field_type": "text",
          "config": {},
          "backing_column": "name",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "source_key",
          "title": "원본 키",
          "field_type": "text",
          "config": {},
          "backing_column": "source_key",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "work_order",
      "title": "작업지시",
      "title_property_key": "request_no",
      "backing_kind": "projected",
      "backing_table": "work_orders",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "request_no",
          "title": "접수번호",
          "field_type": "text",
          "config": {},
          "backing_column": "request_no",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "branch_id",
          "title": "지사",
          "field_type": "reference",
          "config": {},
          "backing_column": "branch_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "equipment_id",
          "title": "장비",
          "field_type": "reference",
          "config": {},
          "backing_column": "equipment_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "customer_id",
          "title": "고객",
          "field_type": "reference",
          "config": {},
          "backing_column": "customer_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "site_id",
          "title": "현장",
          "field_type": "reference",
          "config": {},
          "backing_column": "site_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "RECEIVED",
                "name": "RECEIVED"
              },
              {
                "id": "UNASSIGNED",
                "name": "UNASSIGNED"
              },
              {
                "id": "ASSIGNED",
                "name": "ASSIGNED"
              },
              {
                "id": "IN_PROGRESS",
                "name": "IN_PROGRESS"
              },
              {
                "id": "REPORT_SUBMITTED",
                "name": "REPORT_SUBMITTED"
              },
              {
                "id": "ADMIN_REVIEW",
                "name": "ADMIN_REVIEW"
              },
              {
                "id": "FINAL_COMPLETED",
                "name": "FINAL_COMPLETED"
              },
              {
                "id": "REJECTED",
                "name": "REJECTED"
              },
              {
                "id": "ON_HOLD",
                "name": "ON_HOLD"
              },
              {
                "id": "DELAYED",
                "name": "DELAYED"
              },
              {
                "id": "TEMPORARY_ACTION",
                "name": "TEMPORARY_ACTION"
              },
              {
                "id": "PART_WAITING",
                "name": "PART_WAITING"
              },
              {
                "id": "EQUIPMENT_IN_USE",
                "name": "EQUIPMENT_IN_USE"
              },
              {
                "id": "REVISIT_REQUIRED",
                "name": "REVISIT_REQUIRED"
              },
              {
                "id": "ARCHIVED",
                "name": "ARCHIVED"
              },
              {
                "id": "CANCELLED",
                "name": "CANCELLED"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "priority",
          "title": "우선순위",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "P1",
                "name": "P1"
              },
              {
                "id": "P2",
                "name": "P2"
              },
              {
                "id": "P3",
                "name": "P3"
              },
              {
                "id": "OUTSOURCE",
                "name": "OUTSOURCE"
              },
              {
                "id": "UNSET",
                "name": "UNSET"
              }
            ]
          },
          "backing_column": "priority",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "symptom",
          "title": "증상",
          "field_type": "text",
          "config": {},
          "backing_column": "symptom",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "target_due_at",
          "title": "목표완료일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "target_due_at",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "접수일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "equipment",
          "title": "장비",
          "reverse_title": "작업지시",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "equipment"
        },
        {
          "stable_key": "customer",
          "title": "고객",
          "reverse_title": "작업지시",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "customer"
        },
        {
          "stable_key": "site",
          "title": "현장",
          "reverse_title": "작업지시",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "site"
        }
      ],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "approval",
      "title": "결재",
      "title_property_key": "kind",
      "backing_kind": "projected",
      "backing_table": "gov_approval_requests",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "request_ref",
          "title": "대상",
          "field_type": "reference",
          "config": {},
          "backing_column": "request_ref",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "kind",
          "title": "종류",
          "field_type": "text",
          "config": {},
          "backing_column": "kind",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "requested_by",
          "title": "기안자",
          "field_type": "reference",
          "config": {},
          "backing_column": "requested_by",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "기안일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "support_ticket",
      "title": "지원 티켓",
      "title_property_key": "title",
      "backing_kind": "projected",
      "backing_table": "support_tickets",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "origin",
          "title": "채널",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "INTERNAL",
                "name": "INTERNAL"
              },
              {
                "id": "CUSTOMER",
                "name": "CUSTOMER"
              }
            ]
          },
          "backing_column": "origin",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "category",
          "title": "분류",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "SYSTEM_BUG",
                "name": "SYSTEM_BUG"
              },
              {
                "id": "ACCESS_REQUEST",
                "name": "ACCESS_REQUEST"
              },
              {
                "id": "OPERATIONAL",
                "name": "OPERATIONAL"
              },
              {
                "id": "EQUIPMENT_INQUIRY",
                "name": "EQUIPMENT_INQUIRY"
              },
              {
                "id": "COMPLAINT",
                "name": "COMPLAINT"
              },
              {
                "id": "OTHER",
                "name": "OTHER"
              }
            ]
          },
          "backing_column": "category",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "priority",
          "title": "우선순위",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "LOW",
                "name": "LOW"
              },
              {
                "id": "MEDIUM",
                "name": "MEDIUM"
              },
              {
                "id": "HIGH",
                "name": "HIGH"
              },
              {
                "id": "URGENT",
                "name": "URGENT"
              }
            ]
          },
          "backing_column": "priority",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "OPEN",
                "name": "OPEN"
              },
              {
                "id": "IN_PROGRESS",
                "name": "IN_PROGRESS"
              },
              {
                "id": "ON_HOLD",
                "name": "ON_HOLD"
              },
              {
                "id": "RESOLVED",
                "name": "RESOLVED"
              },
              {
                "id": "CLOSED",
                "name": "CLOSED"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "title",
          "title": "제목",
          "field_type": "text",
          "config": {},
          "backing_column": "title",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "assignee_user_id",
          "title": "담당자",
          "field_type": "reference",
          "config": {},
          "backing_column": "assignee_user_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "due_at",
          "title": "SLA 기한",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "due_at",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "접수일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "evidence",
      "title": "증거",
      "title_property_key": "code",
      "backing_kind": "projected",
      "backing_table": "docs_evidence_objects",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "code",
          "title": "코드",
          "field_type": "text",
          "config": {},
          "backing_column": "code",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "source_type",
          "title": "출처유형",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "record_archive",
                "name": "record_archive"
              },
              {
                "id": "inbox_doc",
                "name": "inbox_doc"
              },
              {
                "id": "mail_attachment",
                "name": "mail_attachment"
              },
              {
                "id": "ingest_job",
                "name": "ingest_job"
              },
              {
                "id": "work_order_evidence_media",
                "name": "work_order_evidence_media"
              },
              {
                "id": "external_document",
                "name": "external_document"
              }
            ]
          },
          "backing_column": "source_type",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "classification",
          "title": "분류등급",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "GENERAL",
                "name": "GENERAL"
              },
              {
                "id": "INTERNAL",
                "name": "INTERNAL"
              },
              {
                "id": "SENSITIVE",
                "name": "SENSITIVE"
              },
              {
                "id": "CONFIDENTIAL",
                "name": "CONFIDENTIAL"
              },
              {
                "id": "SECRET",
                "name": "SECRET"
              }
            ]
          },
          "backing_column": "classification",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "current_custody_stage",
          "title": "보관단계",
          "field_type": "text",
          "config": {},
          "backing_column": "current_custody_stage",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "legal_hold_state",
          "title": "법적보류",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "CLEAR",
                "name": "CLEAR"
              },
              {
                "id": "ACTIVE",
                "name": "ACTIVE"
              }
            ]
          },
          "backing_column": "legal_hold_state",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "compliance_obligation",
      "title": "준수 의무",
      "title_property_key": "code",
      "backing_kind": "projected",
      "backing_table": "compliance_obligations",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "code",
          "title": "코드",
          "field_type": "text",
          "config": {},
          "backing_column": "code",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "obligation_type",
          "title": "유형",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "LEGAL",
                "name": "LEGAL"
              },
              {
                "id": "REGULATORY",
                "name": "REGULATORY"
              },
              {
                "id": "CONTRACTUAL",
                "name": "CONTRACTUAL"
              },
              {
                "id": "INTERNAL_POLICY",
                "name": "INTERNAL_POLICY"
              },
              {
                "id": "CONTROL_REQUIREMENT",
                "name": "CONTROL_REQUIREMENT"
              }
            ]
          },
          "backing_column": "obligation_type",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "scope_type",
          "title": "범위",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "ORG",
                "name": "ORG"
              },
              {
                "id": "BRANCH",
                "name": "BRANCH"
              },
              {
                "id": "SITE",
                "name": "SITE"
              },
              {
                "id": "TEAM",
                "name": "TEAM"
              },
              {
                "id": "ROLE",
                "name": "ROLE"
              }
            ]
          },
          "backing_column": "scope_type",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "site_id",
          "title": "현장",
          "field_type": "reference",
          "config": {},
          "backing_column": "site_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "severity",
          "title": "심각도",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "INFO",
                "name": "INFO"
              },
              {
                "id": "LOW",
                "name": "LOW"
              },
              {
                "id": "MEDIUM",
                "name": "MEDIUM"
              },
              {
                "id": "HIGH",
                "name": "HIGH"
              },
              {
                "id": "CRITICAL",
                "name": "CRITICAL"
              }
            ]
          },
          "backing_column": "severity",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "DRAFT",
                "name": "DRAFT"
              },
              {
                "id": "ACTIVE",
                "name": "ACTIVE"
              },
              {
                "id": "WAIVED",
                "name": "WAIVED"
              },
              {
                "id": "SUPERSEDED",
                "name": "SUPERSEDED"
              },
              {
                "id": "ARCHIVED",
                "name": "ARCHIVED"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "site",
          "title": "현장",
          "reverse_title": "준수 의무",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "site"
        }
      ],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "compliance_regulation",
      "title": "규정",
      "title_property_key": "code",
      "backing_kind": "projected",
      "backing_table": "compliance_regulation_impacts",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "code",
          "title": "코드",
          "field_type": "text",
          "config": {},
          "backing_column": "code",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "jurisdiction",
          "title": "관할",
          "field_type": "text",
          "config": {},
          "backing_column": "jurisdiction",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "regulator",
          "title": "규제기관",
          "field_type": "text",
          "config": {},
          "backing_column": "regulator",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "risk_level",
          "title": "위험도",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "INFO",
                "name": "INFO"
              },
              {
                "id": "LOW",
                "name": "LOW"
              },
              {
                "id": "MEDIUM",
                "name": "MEDIUM"
              },
              {
                "id": "HIGH",
                "name": "HIGH"
              },
              {
                "id": "CRITICAL",
                "name": "CRITICAL"
              }
            ]
          },
          "backing_column": "risk_level",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "DRAFT",
                "name": "DRAFT"
              },
              {
                "id": "ACTIVE",
                "name": "ACTIVE"
              },
              {
                "id": "SUPERSEDED",
                "name": "SUPERSEDED"
              },
              {
                "id": "ARCHIVED",
                "name": "ARCHIVED"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "compliance_framework",
      "title": "표준 프레임워크",
      "title_property_key": "code",
      "backing_kind": "projected",
      "backing_table": "compliance_frameworks",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "code",
          "title": "코드",
          "field_type": "text",
          "config": {},
          "backing_column": "code",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "name",
          "title": "명칭",
          "field_type": "text",
          "config": {},
          "backing_column": "name",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "framework_kind",
          "title": "유형",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "LEGAL_BASELINE",
                "name": "LEGAL_BASELINE"
              },
              {
                "id": "INTERNAL_CONTROL",
                "name": "INTERNAL_CONTROL"
              },
              {
                "id": "CUSTOMER_CONTROL",
                "name": "CUSTOMER_CONTROL"
              },
              {
                "id": "SECURITY_STANDARD",
                "name": "SECURITY_STANDARD"
              },
              {
                "id": "SAFETY_STANDARD",
                "name": "SAFETY_STANDARD"
              },
              {
                "id": "AUDIT_PROGRAM",
                "name": "AUDIT_PROGRAM"
              }
            ]
          },
          "backing_column": "framework_kind",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "DRAFT",
                "name": "DRAFT"
              },
              {
                "id": "ACTIVE",
                "name": "ACTIVE"
              },
              {
                "id": "RETIRED",
                "name": "RETIRED"
              },
              {
                "id": "ARCHIVED",
                "name": "ARCHIVED"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "leave_request",
      "title": "휴가 신청",
      "title_property_key": "reason",
      "backing_kind": "projected",
      "backing_table": "leave_requests",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "subject_employee_id",
          "title": "대상 직원",
          "field_type": "reference",
          "config": {},
          "backing_column": "subject_employee_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "leave_type",
          "title": "유형",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "annual",
                "name": "annual"
              },
              {
                "id": "half_day",
                "name": "half_day"
              }
            ]
          },
          "backing_column": "leave_type",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "pending",
                "name": "pending"
              },
              {
                "id": "approved",
                "name": "approved"
              },
              {
                "id": "returned",
                "name": "returned"
              },
              {
                "id": "rejected",
                "name": "rejected"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "start_date",
          "title": "시작일",
          "field_type": "date",
          "config": {},
          "backing_column": "start_date",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "end_date",
          "title": "종료일",
          "field_type": "date",
          "config": {},
          "backing_column": "end_date",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "reason",
          "title": "사유",
          "field_type": "text",
          "config": {},
          "backing_column": "reason",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "신청일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "employee",
          "title": "직원",
          "reverse_title": "휴가 신청",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "employee"
        }
      ],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "workflow_definition",
      "title": "워크플로우 정의",
      "title_property_key": "display_name",
      "backing_kind": "projected",
      "backing_table": "workflow_definitions",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "workflow_key",
          "title": "키",
          "field_type": "text",
          "config": {},
          "backing_column": "workflow_key",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "display_name",
          "title": "이름",
          "field_type": "text",
          "config": {},
          "backing_column": "display_name",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "object_type",
          "title": "대상 타입",
          "field_type": "text",
          "config": {},
          "backing_column": "object_type",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "status",
          "title": "상태",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "DRAFT",
                "name": "DRAFT"
              },
              {
                "id": "ACTIVE",
                "name": "ACTIVE"
              },
              {
                "id": "PAUSED",
                "name": "PAUSED"
              },
              {
                "id": "RETIRED",
                "name": "RETIRED"
              }
            ]
          },
          "backing_column": "status",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "등록일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "messenger_thread",
      "title": "메신저 스레드",
      "title_property_key": "title",
      "backing_kind": "projected",
      "backing_table": "messenger_threads",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "kind",
          "title": "종류",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "work_order",
                "name": "work_order"
              },
              {
                "id": "team",
                "name": "team"
              },
              {
                "id": "dm",
                "name": "dm"
              },
              {
                "id": "group",
                "name": "group"
              }
            ]
          },
          "backing_column": "kind",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "branch_id",
          "title": "지사",
          "field_type": "reference",
          "config": {},
          "backing_column": "branch_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "work_order_id",
          "title": "작업지시",
          "field_type": "reference",
          "config": {},
          "backing_column": "work_order_id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "title",
          "title": "제목",
          "field_type": "text",
          "config": {},
          "backing_column": "title",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "생성일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [
        {
          "stable_key": "work_order",
          "title": "작업지시",
          "reverse_title": "메신저 스레드",
          "traversable": true,
          "cardinality": "one_many",
          "to_stable_key": "work_order"
        }
      ],
      "actions": [],
      "analytics": []
    },
    {
      "stable_key": "mail",
      "title": "메일",
      "title_property_key": "subject",
      "backing_kind": "projected",
      "backing_table": "email_messages",
      "primary_key_property": "id",
      "properties": [
        {
          "key": "id",
          "title": "ID",
          "field_type": "reference",
          "config": {},
          "backing_column": "id",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "direction",
          "title": "방향",
          "field_type": "choice",
          "config": {
            "choices": [
              {
                "id": "IN",
                "name": "IN"
              },
              {
                "id": "OUT",
                "name": "OUT"
              }
            ]
          },
          "backing_column": "direction",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "from_address",
          "title": "발신자",
          "field_type": "text",
          "config": {},
          "backing_column": "from_address",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "subject",
          "title": "제목",
          "field_type": "text",
          "config": {},
          "backing_column": "subject",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "seen",
          "title": "읽음",
          "field_type": "boolean",
          "config": {},
          "backing_column": "seen",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "flagged",
          "title": "중요",
          "field_type": "boolean",
          "config": {},
          "backing_column": "flagged",
          "required": false,
          "in_property_policy": false
        },
        {
          "key": "created_at",
          "title": "수신일",
          "field_type": "timestamp",
          "config": {},
          "backing_column": "created_at",
          "required": false,
          "in_property_policy": false
        }
      ],
      "links": [],
      "actions": [],
      "analytics": []
    }
  ]
}$manifest$::jsonb;
 shape jsonb; expected jsonb; actual jsonb; child jsonb; child_shape jsonb;
 object_id uuid; target_id uuid; field_name text; key_name text; field_count bigint;
 installed_keys jsonb; event public.audit_events%ROWTYPE;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF NEW.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR NEW.catalog_version IS DISTINCT FROM '2026-07-19.1'
  OR NEW.manifest_digest IS DISTINCT FROM decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex')
  OR EXISTS(SELECT 1 FROM public.platform_legacy_catalog_effect_bindings b
   WHERE b.actor_user_id=NEW.actor_user_id AND b.command_id=NEW.command_id) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
 END IF;
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.actor_user_id=NEW.actor_user_id AND x.command_id=NEW.command_id;
 IF ROW(r.receipt_id,r.codec_version,r.kind,r.outcome,r.result_code,r.primary_target,
   r.result_org_id,r.result_user_id,r.input_digest,r.effect_family_id,r.effect_xid,r.effect_backend_pid,
   r.builtin_catalog_version,r.builtin_catalog_digest,r.trace_id::text,r.span_id::text,r.occurred_at)
  IS DISTINCT FROM ROW(NEW.receipt_id,1::smallint,1::smallint,'APPLIED'::text,'applied'::text,NULL::uuid,
   NEW.org_id,NEW.admin_user_id,NEW.input_digest,NEW.effect_family_id,NEW.effect_xid,NEW.effect_backend_pid,
   NEW.catalog_version,NEW.manifest_digest,NEW.trace_id,NEW.span_id,NEW.catalog_at) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
 END IF;
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(r.input_bytes);
 IF decoded.kind IS DISTINCT FROM 1 OR decoded.actor_user_id IS DISTINCT FROM r.actor_user_id
  OR decoded.command_id IS DISTINCT FROM r.command_id OR decoded.primary_target IS NOT NULL
  OR decoded.input_digest IS DISTINCT FROM r.input_digest OR decoded.expected_groups IS DISTINCT FROM '[]'::jsonb
  OR sha256(convert_to(manifest::text,'UTF8')) IS DISTINCT FROM NEW.manifest_digest
  OR manifest->>'catalog_version' IS DISTINCT FROM NEW.catalog_version
  OR jsonb_array_length(manifest->'object_types')<>27 THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
 END IF;
 -- Only the validated retained private binding/current receipt selects scope.
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 IF NOT EXISTS(SELECT 1 FROM public.organizations o
   JOIN public.users u ON u.org_id=o.id AND u.id=NEW.admin_user_id
   JOIN public.groups g ON g.id=o.group_id
   JOIN public.group_memberships m ON m.group_id=g.id AND m.org_id=o.id
   JOIN public.group_membership_revisions h ON h.group_id=m.group_id AND h.org_id=m.org_id
    AND h.membership_id=m.membership_id AND h.revision=m.current_revision AND h.incarnation=m.incarnation
   WHERE o.id=NEW.org_id AND o.group_id=r.result_group_id AND o.status='ACTIVE'
    AND o.slug=decoded.payload->>'slug' AND o.name=decoded.payload->>'name'
    AND o.origin_account_id IS NULL AND o.origin_command_id IS NULL AND o.origin_receipt_id IS NULL
    AND g.status='ACTIVE' AND g.origin_account_id IS NULL AND g.origin_command_id IS NULL AND g.origin_receipt_id IS NULL
    AND u.is_active AND u.roles=ARRAY['SUPER_ADMIN']::text[]
    AND h.provenance_kind='LEGACY_TOPOLOGY_V1' AND h.state='ACTIVE' AND h.to_time IS NULL
    AND h.native_account_id IS NULL AND h.legacy_actor_user_id=r.actor_user_id
    AND h.command_id=r.command_id AND h.command_receipt=r.receipt_id)
  OR NOT public.auth_legacy_bootstrap_issued_v1(NEW.org_id,NEW.admin_user_id,r.bootstrap_credential_id,r.bootstrap_expires_at)
  OR (SELECT count(*) FROM public.ont_object_types o WHERE o.org_id=NEW.org_id)<>27
  OR (SELECT count(*) FROM public.ont_object_type_key_revisions k WHERE k.org_id=NEW.org_id)<>27
  OR (SELECT count(*) FROM public.ont_builtin_catalog_installs i WHERE i.org_id=NEW.org_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.ont_builtin_catalog_installs i
   WHERE i.org_id=NEW.org_id AND i.catalog_version=NEW.catalog_version AND i.manifest_digest=NEW.manifest_digest
    AND i.installed_by=NEW.admin_user_id AND i.installed_at=NEW.catalog_at AND i.attribution_protocol='LEGACY_USER'
    AND i.installed_by_account_id IS NULL AND i.origin_account_id IS NULL
    AND i.origin_command_id IS NULL AND i.origin_receipt_id IS NULL)
  OR (SELECT count(*) FROM public.audit_events e WHERE e.org_id=NEW.org_id
   AND e.action='ontology.object_type.builtin_install')<>27
  OR (SELECT count(*) FROM public.audit_events e WHERE e.org_id=NEW.org_id
   AND e.action='ontology.builtin_catalog.install')<>1 THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
 END IF;
 installed_keys:='[]'::jsonb;
 FOR shape IN SELECT value FROM jsonb_array_elements(manifest->'object_types') LOOP
  SELECT o.id,to_jsonb(o) INTO STRICT object_id,actual FROM public.ont_object_types o
   WHERE o.org_id=NEW.org_id AND o.stable_key=shape->>'stable_key';
  expected:=shape-ARRAY['properties','links','actions','analytics']::text[];
  IF EXISTS(SELECT 1 FROM jsonb_each(expected) pair WHERE actual->pair.key IS DISTINCT FROM pair.value) OR actual->>'attribution_protocol' IS DISTINCT FROM 'LEGACY_USER'
   OR (actual->>'created_by')::uuid IS DISTINCT FROM NEW.admin_user_id
   OR (actual->>'created_at')::timestamptz IS DISTINCT FROM NEW.catalog_at
   OR (actual->>'updated_at')::timestamptz IS DISTINCT FROM NEW.catalog_at
   OR actual->>'lifecycle_state' IS DISTINCT FROM 'published' OR (actual->>'schema_version')::integer IS DISTINCT FROM 1
   OR actual->>'created_by_account_id' IS NOT NULL OR actual->>'origin_account_id' IS NOT NULL
   OR actual->>'origin_command_id' IS NOT NULL OR actual->>'origin_receipt_id' IS NOT NULL
   OR NOT EXISTS(SELECT 1 FROM public.ont_object_type_key_revisions k
    WHERE k.org_id=NEW.org_id AND k.stable_key=shape->>'stable_key' AND k.revision=1) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
  END IF;
  FOREACH field_name IN ARRAY ARRAY['properties','links','actions','analytics']::text[] LOOP
   -- Fixed four-table projection: no dynamic relation name or caller SQL.
   SELECT count(*) INTO field_count FROM (
    SELECT to_jsonb(p) AS value FROM public.ont_property_defs p WHERE field_name='properties' AND p.org_id=NEW.org_id AND p.object_type_id=object_id
    UNION ALL SELECT to_jsonb(l) FROM public.ont_link_types l WHERE field_name='links' AND l.org_id=NEW.org_id AND l.object_type_id=object_id
    UNION ALL SELECT to_jsonb(a) FROM public.ont_action_types a WHERE field_name='actions' AND a.org_id=NEW.org_id AND a.object_type_id=object_id
    UNION ALL SELECT to_jsonb(a) FROM public.ont_analytics a WHERE field_name='analytics' AND a.org_id=NEW.org_id AND a.object_type_id=object_id
   ) AS children;
   IF field_count<>jsonb_array_length(shape->field_name) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
   END IF;
   key_name:=CASE WHEN field_name IN ('properties','analytics') THEN 'key' ELSE 'stable_key' END;
   FOR child_shape IN SELECT value FROM jsonb_array_elements(shape->field_name) LOOP
    expected:=child_shape;
    IF field_name='properties' THEN expected:=(expected-'field_type')||jsonb_build_object('type',expected->'field_type'); END IF;
    IF field_name='links' THEN
     target_id:=NULL;
     IF child_shape->>'to_stable_key' IS NOT NULL THEN
      SELECT o.id INTO STRICT target_id FROM public.ont_object_types o
       WHERE o.org_id=NEW.org_id AND o.stable_key=child_shape->>'to_stable_key' AND o.lifecycle_state='published';
     END IF;
     expected:=(expected-'to_stable_key')||jsonb_build_object('to_object_type_id',target_id);
    END IF;
    SELECT value INTO STRICT child FROM (
     SELECT to_jsonb(p) AS value FROM public.ont_property_defs p WHERE field_name='properties' AND p.org_id=NEW.org_id AND p.object_type_id=object_id
     UNION ALL SELECT to_jsonb(l) FROM public.ont_link_types l WHERE field_name='links' AND l.org_id=NEW.org_id AND l.object_type_id=object_id
     UNION ALL SELECT to_jsonb(a) FROM public.ont_action_types a WHERE field_name='actions' AND a.org_id=NEW.org_id AND a.object_type_id=object_id
     UNION ALL SELECT to_jsonb(a) FROM public.ont_analytics a WHERE field_name='analytics' AND a.org_id=NEW.org_id AND a.object_type_id=object_id
    ) AS children WHERE value->>key_name=child_shape->>key_name;
    IF EXISTS(SELECT 1 FROM jsonb_each(expected) pair WHERE child->pair.key IS DISTINCT FROM pair.value) OR (child->>'created_at')::timestamptz IS DISTINCT FROM transaction_timestamp() THEN
     RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
    END IF;
   END LOOP;
  END LOOP;
  -- Exactly one audit on each affected type; another action cannot hide beside
  -- the required builtin event and falsely satisfy per-action counts.
  IF (SELECT count(*) FROM public.audit_events e WHERE e.org_id=NEW.org_id
    AND e.target_type='ont_object_types' AND e.target_id=object_id::text)<>1 THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
  END IF;
  SELECT e.* INTO STRICT event FROM public.audit_events e
   WHERE e.org_id=NEW.org_id AND e.action='ontology.object_type.builtin_install' AND e.target_id=object_id::text;
  IF NOT public.platform_legacy_catalog_receipt_audit_v1(event) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
  END IF;
  installed_keys:=installed_keys||jsonb_build_array(shape->>'stable_key');
 END LOOP;
 IF (SELECT count(*) FROM public.audit_events e WHERE e.org_id=NEW.org_id
   AND e.target_type='ont_builtin_catalog_installs' AND e.target_id=NEW.org_id::text)<>1 THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
 END IF;
 SELECT e.* INTO STRICT event FROM public.audit_events e
  WHERE e.org_id=NEW.org_id AND e.action='ontology.builtin_catalog.install';
 IF event.actor IS DISTINCT FROM NEW.admin_user_id OR event.target_type IS DISTINCT FROM 'ont_builtin_catalog_installs'
  OR event.target_id IS DISTINCT FROM NEW.org_id::text OR event.trace_id IS DISTINCT FROM NEW.trace_id
  OR event.span_id IS DISTINCT FROM NEW.span_id OR event.occurred_at IS DISTINCT FROM NEW.catalog_at
  OR event.branch_id IS NOT NULL OR event.before_snap IS NOT NULL
  OR event.after_snap IS DISTINCT FROM jsonb_build_object('catalog_version',NEW.catalog_version,
   'manifest_digest',encode(NEW.manifest_digest,'hex'),'installed_keys',installed_keys,'retained_keys','[]'::jsonb) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_legacy_catalog.closure_invalid';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_catalog_binding_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_catalog_binding_closed_v1() FROM PUBLIC;
CREATE CONSTRAINT TRIGGER trg_platform_legacy_catalog_binding_closed
 AFTER INSERT ON public.platform_legacy_catalog_effect_bindings DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_catalog_binding_closed_v1();


-- source: native-legacy-protected-audit-parser.sql
CREATE OR REPLACE FUNCTION ontology_api.protected_audit_writer_guard()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_invoker NAME := ontology_api.invoker_role();
    v_stable_key TEXT;
    b record; o record; snapshot jsonb; target text;
    native_object uuid; native_action text;
    prior_org text:=current_setting('app.current_org',true);
BEGIN
    IF NEW.action IN ('ontology.object_type.create','ontology.object_type.stage_revision',
        'ontology.object_type.transition','ontology.object_type.builtin_install','ontology.object_policy.attach') THEN
      SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
      IF b.account_id IS NOT NULL THEN
       BEGIN
        IF NEW.action NOT IN ('ontology.object_type.builtin_install','ontology.object_policy.attach')
          OR b.request_state IS DISTINCT FROM 'PENDING' OR b.org_id IS DISTINCT FROM NEW.org_id
          OR NEW.actor IS DISTINCT FROM b.account_id OR NEW.occurred_at IS DISTINCT FROM b.started_at
          OR NEW.before_snap IS NOT NULL OR NEW.branch_id IS NOT NULL
          OR NEW.trace_id IS NULL OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
          OR NEW.span_id IS NULL OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        native_object:=NEW.target_id::uuid; native_action:=NEW.action;
        PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=native_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
 IF native_action='ontology.object_type.builtin_install' THEN
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 ELSE
  target:='ont_object_policies';
  SELECT c.normalized_row INTO STRICT snapshot
   FROM public.ont_object_policies p JOIN public.cedar_policy_catalog_entries c ON c.org_id=p.org_id AND c.id=p.cedar_policy_id
   WHERE p.org_id=b.org_id AND p.object_type_id=native_object AND p.attribution_protocol='NATIVE_ACCOUNT'
    AND p.origin_account_id=b.account_id AND p.origin_command_id=b.command_id AND p.origin_receipt_id=b.receipt_id
    AND p.created_by_account_id=b.account_id AND p.created_at=b.started_at
    AND c.attribution_protocol='NATIVE_ACCOUNT' AND c.origin_account_id=b.account_id AND c.origin_command_id=b.command_id
    AND c.origin_receipt_id=b.receipt_id AND c.created_by_account_id=b.account_id AND c.updated_by_account_id=b.account_id
    AND c.created_at=b.started_at AND c.updated_at=b.started_at;
  IF snapshot IS DISTINCT FROM jsonb_build_object('effect','forbid','action','view','resource_type',o.stable_key,'conditions','[]'::jsonb) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
 END IF;
        IF NEW.target_type IS DISTINCT FROM target OR NEW.after_snap IS DISTINCT FROM
          snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
            'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)) THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
       EXCEPTION WHEN OTHERS THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
       END;
      ELSIF coalesce(NEW.after_snap?'enrollment',false) THEN
        RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='company_enrollment.ontology_audit_owner_required';
      END IF;
    END IF;
    IF NEW.action <> ALL (ARRAY[
        'ontology.object_type.create',
        'ontology.object_type.stage_revision',
        'ontology.object_type.transition',
        'ontology.object_type.builtin_install'
    ]::TEXT[]) THEN
        RETURN NEW;
    END IF;

    -- Direct command credentials cannot INSERT audit_events. When an approved
    -- command reaches this trigger it is nested inside the writer-owned
    -- SECURITY DEFINER routine, while the old compatibility path arrives as
    -- console_rt and must prove a matching parent mutation in this transaction.
    IF v_invoker = 'console_rt'::NAME THEN
        IF NEW.action = 'ontology.object_type.builtin_install'
           AND public.platform_legacy_catalog_live_audit_v1(NEW) THEN
            RETURN NEW;
        END IF;
        IF NEW.action = 'ontology.object_type.builtin_install'
           OR NEW.target_type <> 'ont_object_types'
           OR NOT EXISTS (
               SELECT 1
               FROM public.users u
               WHERE u.id = NEW.actor AND u.org_id = NEW.org_id AND u.is_active
           ) THEN
            RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_audit.command_required';
        END IF;

        SELECT parent_type.stable_key INTO v_stable_key
        FROM public.ont_object_types parent_type
        WHERE parent_type.org_id = NEW.org_id
          AND parent_type.id::TEXT = NEW.target_id
          AND parent_type.updated_at = NEW.occurred_at
          AND parent_type.xmin = pg_catalog.pg_current_xact_id()::xid
          AND (
              (NEW.action = 'ontology.object_type.create' AND parent_type.schema_version = 1 AND parent_type.created_at = NEW.occurred_at)
              OR (NEW.action = 'ontology.object_type.stage_revision' AND parent_type.schema_version > 1 AND parent_type.created_at = NEW.occurred_at)
              OR NEW.action = 'ontology.object_type.transition'
          );
        IF v_stable_key IS NULL THEN
            RAISE EXCEPTION USING ERRCODE = '42501', MESSAGE = 'ontology_audit.command_required';
        END IF;
        IF NEW.action IN ('ontology.object_type.stage_revision', 'ontology.object_type.transition') THEN
            UPDATE public.ont_object_type_key_revisions k
               SET revision = k.revision + 1, updated_at = NEW.occurred_at
             WHERE k.org_id = NEW.org_id AND k.stable_key = v_stable_key;
            IF NOT FOUND THEN
                RAISE EXCEPTION USING ERRCODE = '23503', MESSAGE = 'ontology_legacy.key_revision_missing';
            END IF;
        END IF;
    ELSIF v_invoker <> 'console_ontology_cmd'::NAME THEN
        RAISE EXCEPTION USING
            ERRCODE = '42501',
            MESSAGE = 'ontology_audit.command_required';
    END IF;
    RETURN NEW;
END;
$$;

-- source: native-legacy-deferred-audit-parser.sql
CREATE OR REPLACE FUNCTION ontology_api.require_current_transaction_audit()
RETURNS TRIGGER
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = pg_catalog
SET row_security = on
AS $$
DECLARE
    v_parent_id UUID;
    v_org_id UUID;
    v_stable_key TEXT;
    v_parent_is_current BOOLEAN;
    b record; o record; snapshot jsonb; target text;
    native_object uuid; native_action text;
    prior_org text:=current_setting('app.current_org',true);
BEGIN
    -- Scope comes from the persisted trigger row, never the caller's ambient Company.
    PERFORM set_config('app.current_org',NEW.org_id::text,true);
    SELECT * INTO b FROM public.company_enrollment_catalog_binding_v1(NEW.org_id);
    IF b.account_id IS NOT NULL THEN
        IF b.request_state IS DISTINCT FROM 'COMMITTED' OR b.org_id IS DISTINCT FROM NEW.org_id THEN
          RAISE EXCEPTION 'company_enrollment.binding_invalid';
        END IF;
        IF TG_TABLE_NAME='ont_object_types' THEN native_object:=NEW.id;
        ELSE native_object:=NEW.object_type_id; END IF;
        PERFORM set_config('app.current_org',b.org_id::text,true);
 SELECT x.id,x.stable_key,x.schema_version,x.lifecycle_state,x.attribution_protocol,x.created_by,
  x.created_by_account_id,x.origin_account_id,x.origin_command_id,x.origin_receipt_id,x.created_at,x.updated_at
  INTO STRICT o FROM public.ont_object_types x
  WHERE x.org_id=b.org_id AND x.id=native_object;
 IF o.attribution_protocol IS DISTINCT FROM 'NATIVE_ACCOUNT' OR o.created_by IS NOT NULL
  OR o.created_by_account_id IS DISTINCT FROM b.account_id OR o.origin_account_id IS DISTINCT FROM b.account_id
  OR o.origin_command_id IS DISTINCT FROM b.command_id OR o.origin_receipt_id IS DISTINCT FROM b.receipt_id
  OR o.created_at IS DISTINCT FROM b.started_at OR o.updated_at IS DISTINCT FROM b.started_at
  OR o.schema_version IS DISTINCT FROM 1 OR o.lifecycle_state IS DISTINCT FROM 'published'
  OR o.stable_key NOT IN ('company_workspace','company_policy_assignment') THEN
  RAISE EXCEPTION 'company_enrollment.binding_invalid';
 END IF;
  target:='ont_object_types';
  snapshot:=jsonb_build_object('stable_key',o.stable_key,'schema_version',o.schema_version,
   'lifecycle_state',o.lifecycle_state,'catalog_version',b.catalog_version,'manifest_digest',encode(b.manifest_digest,'hex'));
  IF (SELECT count(*) FROM public.ont_property_defs p WHERE p.org_id=b.org_id AND p.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 8 END)
   OR (SELECT count(*) FROM public.ont_action_types a WHERE a.org_id=b.org_id AND a.object_type_id=native_object)
    <>(CASE WHEN o.stable_key='company_workspace' THEN 2 ELSE 3 END) THEN
   RAISE EXCEPTION 'company_enrollment.binding_invalid';
  END IF;
        IF (SELECT count(*) FROM public.audit_events e WHERE e.org_id=b.org_id
          AND e.action='ontology.object_type.builtin_install' AND e.target_id=native_object::text
          AND e.target_type=target AND e.actor=b.account_id AND e.occurred_at=b.started_at
          AND e.before_snap IS NULL AND e.branch_id IS NULL
          AND e.after_snap=snapshot||jsonb_build_object('enrollment',jsonb_build_object('account_id',b.account_id::text,
            'command_id',b.command_id::text,'receipt_id',b.receipt_id::text,'session_id',b.session_id::text)))<>1 THEN
          RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='ontology_write.exactly_one_current_transaction_audit_required';
        END IF;
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
    END IF;
    IF TG_TABLE_NAME = 'ont_object_types' THEN
        v_parent_id := NEW.id;
        v_org_id := NEW.org_id;
        v_stable_key := NEW.stable_key;
        v_parent_is_current := TRUE;
    ELSE
        v_parent_id := NEW.object_type_id;
        v_org_id := NEW.org_id;
        SELECT parent_type.stable_key,
               parent_type.xmin = pg_catalog.pg_current_xact_id()::xid
          INTO v_stable_key, v_parent_is_current
          FROM public.ont_object_types parent_type
         WHERE parent_type.id = v_parent_id AND parent_type.org_id = v_org_id;
    END IF;

    IF COALESCE(v_parent_is_current, FALSE) AND TG_TABLE_NAME <> 'ont_object_types' THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
    END IF;
    IF (
        SELECT COUNT(*) = 1
        FROM public.audit_events e
        LEFT JOIN public.ont_object_types target
          ON target.org_id = e.org_id AND target.id::TEXT = e.target_id
        WHERE e.org_id = v_org_id
          AND e.action = ANY (ARRAY[
              'ontology.object_type.create',
              'ontology.object_type.stage_revision',
              'ontology.object_type.transition',
              'ontology.object_type.builtin_install'
          ]::TEXT[])
          AND (e.xmin = pg_catalog.pg_current_xact_id()::xid
               OR (e.action = 'ontology.object_type.builtin_install'
                   AND public.platform_legacy_catalog_receipt_audit_v1(e)))
          AND (
              e.target_id = v_parent_id::TEXT
              OR (e.action = 'ontology.object_type.transition' AND target.stable_key = v_stable_key)
          )
    ) THEN
        PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
        RETURN NEW;
    END IF;
    RAISE EXCEPTION USING
        ERRCODE = '23514',
        MESSAGE = 'ontology_write.exactly_one_current_transaction_audit_required';
EXCEPTION WHEN OTHERS THEN
    PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END;
$$;

-- source: legacy-bootstrap-issued.sql
-- Proposed private implementation; requires approved exact ACL/custody successor.
CREATE FUNCTION public.auth_legacy_bootstrap_issued_v1(p_company uuid,p_subject uuid,p_id uuid,p_expires_at timestamptz)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE CALLED ON NULL INPUT
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); valid boolean;
BEGIN
 IF p_company IS NULL OR p_subject IS NULL OR p_id IS NULL OR p_expires_at IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_company,p_subject,p_id)
  OR NOT isfinite(p_expires_at) THEN RETURN false; END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b
  WHERE b.id=p_id AND b.org_id=p_company AND b.user_id=p_subject AND b.expires_at=p_expires_at
   AND isfinite(b.issued_at) AND b.issued_at<b.expires_at AND octet_length(b.token_hash)=32
   AND b.consumed_at IS NULL AND b.revoked_at IS NULL AND b.revoked_reason IS NULL)
  AND NOT EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b
   WHERE b.org_id=p_company AND b.user_id=p_subject AND b.id<>p_id
    AND b.consumed_at IS NULL AND b.revoked_at IS NULL)
  AND NOT EXISTS(SELECT 1 FROM public.auth_webauthn_credentials k
   WHERE k.org_id=p_company AND k.user_id=p_subject)
 INTO valid;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN valid;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.auth_legacy_bootstrap_issued_v1(uuid,uuid,uuid,timestamptz) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_issued_v1(uuid,uuid,uuid,timestamptz) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_issued_v1(uuid,uuid,uuid,timestamptz) TO console_credential_owner,console_app;


-- source: legacy-bootstrap-issuer.sql
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
    IF NOT EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b
        WHERE b.id=p_id AND b.org_id=p_company AND b.user_id=p_subject
          AND b.token_hash=p_token_hash AND b.issued_at=p_issued_at AND b.expires_at=p_expires_at
          AND b.consumed_at IS NULL AND b.revoked_at IS NULL AND b.revoked_reason IS NULL) THEN
        RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='auth_legacy.bootstrap_issue_incomplete';
    END IF;
    RETURN 'issued';
END;$auth7_body$;
ALTER FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) FROM PUBLIC,console_app,console_rt,console_auth_rt,console_credential_owner;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamp with time zone,timestamp with time zone) TO console_credential_owner,console_rt;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_issue_v1(uuid,uuid,uuid,bytea,timestamptz,timestamptz) TO console_app;


-- source: legacy-kind8-grantor-index.sql
-- Private atomic-successor component. Historical migrations unchanged.
-- Supports retained-provenance refusal; no new mutation authority.
CREATE INDEX group_role_grants_grantor_idx ON public.group_role_grants(granted_by)
 WHERE granted_by IS NOT NULL;


-- source: legacy-removal-cohort.sql
-- Private shared query for the two existing removal intents. It captures facts,
-- never authorizes an operation. Ordinary and force frames remain distinct.
CREATE FUNCTION public.platform_company_removal_cohort_v1(p_org uuid) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE user_ids uuid[];users_value jsonb;grants_value jsonb;group_ids uuid[];blocked boolean;
BEGIN
 SELECT coalesce(array_agg(x.id ORDER BY x.id),ARRAY[]::uuid[]) INTO user_ids
  FROM (SELECT u.id FROM public.users u WHERE u.org_id=p_org ORDER BY u.id LIMIT 4097) x;
 IF cardinality(user_ids)>4096 THEN
  RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
 END IF;
 SELECT coalesce(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) INTO grants_value FROM (
  SELECT g.id,g.group_id,g.user_id,g.group_role,g.granted_by,g.created_at FROM public.group_role_grants g
   WHERE g.user_id=ANY(user_ids) ORDER BY g.id LIMIT 65537
 ) x;
 IF jsonb_array_length(grants_value)>65536 THEN
  RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
 END IF;
 SELECT coalesce(array_agg(DISTINCT id ORDER BY id),ARRAY[]::uuid[]) INTO group_ids FROM (
  SELECT m.group_id AS id FROM public.group_memberships m WHERE m.org_id=p_org
  UNION ALL SELECT (value->>'group_id')::uuid FROM jsonb_array_elements(grants_value)
 ) x;
 IF cardinality(group_ids)>128 THEN
  RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
 END IF;
 SELECT coalesce(jsonb_agg(to_jsonb(u) ORDER BY u.id),'[]'::jsonb) INTO users_value
  FROM public.users u WHERE u.id=ANY(user_ids);
 SELECT EXISTS(SELECT 1 FROM public.group_role_grants g
  WHERE g.granted_by=ANY(user_ids) AND NOT(g.user_id=ANY(user_ids))) INTO blocked;
 RETURN jsonb_build_object('user_ids',to_jsonb(user_ids),'users',users_value,
  'grants',grants_value,'group_ids',to_jsonb(group_ids),'grantor_blocked',blocked);
END
$body$;
ALTER FUNCTION public.platform_company_removal_cohort_v1(uuid) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_company_removal_cohort_v1(uuid) FROM PUBLIC;


-- source: legacy-topology-plan-source-successor.sql
-- Private source candidate. No public plan projection or authority from a hash.
CREATE FUNCTION public.platform_legacy_topology_plan_v1(
 p_actor uuid,p_family uuid,p_input bytea,p_prospective_company uuid,p_prospective_group uuid)
RETURNS jsonb LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE decoded record; receipt public.platform_legacy_topology_receipts%ROWTYPE;
 target_company uuid; target_user uuid; requested_group uuid; selected_group uuid;
 old_group uuid; companies uuid[]; users_to_lock uuid[]; groups_to_lock uuid[];
 company_rows jsonb; user_rows jsonb; group_rows jsonb; receipt_value jsonb;
 target_users uuid[]:=ARRAY[]::uuid[]; grant_rows jsonb:='[]'::jsonb; grantor_blocked boolean:=false;
 mint_value jsonb:='null'::jsonb; candidate public.groups%ROWTYPE;
 suffix integer; candidate_slug text; membership public.group_memberships%ROWTYPE;
 home constant uuid:='00000000-0000-0000-0000-00000000face';
BEGIN
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(p_input);
 IF decoded.actor_user_id IS DISTINCT FROM p_actor OR p_family IS NULL
  OR p_family='00000000-0000-0000-0000-000000000000'::uuid
  OR (decoded.kind=1) IS DISTINCT FROM (p_prospective_company IS NOT NULL)
  OR (decoded.kind IN (1,5)) IS DISTINCT FROM (p_prospective_group IS NOT NULL)
  OR p_prospective_company='00000000-0000-0000-0000-000000000000'::uuid
  OR p_prospective_group='00000000-0000-0000-0000-000000000000'::uuid THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_plan_input';
 END IF;
 SELECT x.* INTO receipt FROM public.platform_legacy_topology_receipts x
  WHERE x.actor_user_id=p_actor AND x.command_id=decoded.command_id;
 IF receipt.receipt_id IS NOT NULL THEN
  -- Replay needs current source and exact receipt reconciliation, not locks on
  -- an already deleted or subsequently moved historical result Company.
  receipt_value:=jsonb_build_object('receipt_id',receipt.receipt_id::text,
   'input_digest',encode(receipt.input_digest,'hex'),'input_bytes',encode(receipt.input_bytes,'hex'),
   'kind',receipt.kind,'outcome',receipt.outcome,'result_code',receipt.result_code,
   'result_org_id',receipt.result_org_id::text,'result_group_id',receipt.result_group_id::text,
   'result_user_id',receipt.result_user_id::text);
 ELSE
  receipt_value:='null'::jsonb;
  CASE decoded.kind
  WHEN 1 THEN target_company:=p_prospective_company;
  WHEN 3 THEN requested_group:=(decoded.payload->>'group_id')::uuid;
  WHEN 4,5,6 THEN
   target_company:=(decoded.payload->>'org_id')::uuid;
   requested_group:=(decoded.payload->>'group_id')::uuid;
  WHEN 7 THEN
   requested_group:=(decoded.payload->>'group_id')::uuid;
   target_user:=(decoded.payload->>'user_id')::uuid;
   SELECT x.org_id INTO target_company FROM public.users x WHERE x.id=target_user;
  WHEN 8,9 THEN target_company:=(decoded.payload->>'org_id')::uuid;
  ELSE NULL;
  END CASE;
  IF target_company IS NOT NULL THEN
   SELECT x.* INTO membership FROM public.group_memberships x WHERE x.org_id=target_company;
   old_group:=membership.group_id;
  END IF;
  IF decoded.kind=4 THEN selected_group:=requested_group; END IF;
  IF decoded.kind=1 OR (decoded.kind=5 AND old_group=requested_group) THEN
   FOR suffix IN 1..9 LOOP
    candidate_slug:=(CASE WHEN suffix=1 THEN 'go-' ELSE 'g'||suffix::text||'-' END)
     ||replace(target_company::text,'-','');
    SELECT x.* INTO candidate FROM public.groups x WHERE x.slug=candidate_slug;
    IF candidate.id IS NULL THEN
     selected_group:=p_prospective_group;
     mint_value:=jsonb_build_object('suffix',suffix,'group_id',selected_group::text,'is_new',true);
     EXIT;
    ELSIF NOT EXISTS(SELECT 1 FROM public.group_memberships m
      WHERE m.group_id=candidate.id AND m.org_id<>target_company) THEN
     selected_group:=candidate.id;
     mint_value:=jsonb_build_object('suffix',suffix,'group_id',selected_group::text,'is_new',false);
     EXIT;
    END IF;
   END LOOP;
   IF mint_value='null'::jsonb THEN mint_value:=jsonb_build_object('exhausted',true); END IF;
  END IF;
 END IF;
 SELECT array_agg(DISTINCT id ORDER BY id) INTO companies
  FROM unnest(ARRAY[home,target_company]) ids(id) WHERE id IS NOT NULL;
 -- Review/measurement hypotheses, not employee-count-derived release limits.
 -- Never pass a truncated cohort into a lock/effect/receipt path.
 IF receipt.receipt_id IS NULL AND decoded.kind=8 THEN
  SELECT coalesce(array_agg(x.id ORDER BY x.id),ARRAY[]::uuid[]) INTO target_users
   FROM (SELECT u.id FROM public.users u WHERE u.org_id=target_company ORDER BY u.id LIMIT 4097) x;
  IF cardinality(target_users)>4096 THEN
   RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
  END IF;
 END IF;
 SELECT array_agg(DISTINCT id ORDER BY id) INTO users_to_lock
  FROM unnest(ARRAY[p_actor,target_user]||target_users) ids(id) WHERE id IS NOT NULL;
 IF receipt.receipt_id IS NULL AND decoded.kind IN(7,8) THEN
  SELECT coalesce(jsonb_agg(to_jsonb(x) ORDER BY x.id),'[]'::jsonb) INTO grant_rows FROM (
   SELECT gr.id,gr.group_id,gr.user_id,gr.group_role,gr.granted_by,gr.created_at
    FROM public.group_role_grants gr
    WHERE (decoded.kind=8 AND gr.user_id=ANY(target_users))
     OR (decoded.kind=7 AND gr.group_id=requested_group AND gr.user_id=target_user
       AND gr.group_role=decoded.payload->>'group_role')
    ORDER BY gr.id LIMIT 65537
  ) x;
  IF jsonb_array_length(grant_rows)>65536 THEN
   RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
  END IF;
 END IF;
 SELECT coalesce(array_agg(DISTINCT id ORDER BY id),ARRAY[]::uuid[]) INTO groups_to_lock FROM (
  SELECT unnest(ARRAY[requested_group,old_group,selected_group]) AS id
  UNION ALL SELECT (value->>'group_id')::uuid FROM jsonb_array_elements(grant_rows)
 ) ids WHERE id IS NOT NULL;
 IF decoded.kind=8 AND cardinality(groups_to_lock)>128 THEN
  RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
 END IF;
 IF receipt.receipt_id IS NULL AND decoded.kind=8 THEN
  -- Surviving grants retain grantor provenance. Query uses the separately
  -- reviewed granted_by index and retained User locks on the final recheck.
  SELECT EXISTS(SELECT 1 FROM public.group_role_grants gr
    WHERE gr.granted_by=ANY(target_users) AND NOT(gr.user_id=ANY(target_users)))
   INTO grantor_blocked;
 END IF;
 SELECT jsonb_agg(jsonb_build_object('id',w.id::text,'exists',o.id IS NOT NULL,
   'group_id',o.group_id::text,'status',o.status,'slug',o.slug,'name',o.name,
   'origin_account_id',o.origin_account_id::text,'origin_command_id',o.origin_command_id::text,
   'origin_receipt_id',o.origin_receipt_id::text,'membership',
   CASE WHEN m.org_id IS NULL THEN 'null'::jsonb ELSE jsonb_build_object('group_id',m.group_id::text,
    'membership_id',m.membership_id::text,'revision',m.current_revision::text,'incarnation',m.incarnation::text) END)
   ORDER BY w.id) INTO company_rows FROM unnest(companies) w(id)
  LEFT JOIN public.organizations o ON o.id=w.id LEFT JOIN public.group_memberships m ON m.org_id=w.id;
 SELECT jsonb_agg(jsonb_build_object('id',w.id::text,'exists',u.id IS NOT NULL,
   'org_id',u.org_id::text,'active',u.is_active,'roles',to_jsonb(u.roles),
   'row',CASE WHEN u.id IS NULL THEN 'null'::jsonb ELSE to_jsonb(u) END) ORDER BY w.id)
  INTO user_rows FROM unnest(users_to_lock) w(id) LEFT JOIN public.users u ON u.id=w.id;
 SELECT coalesce(jsonb_agg(jsonb_build_object('id',w.id::text,'exists',g.id IS NOT NULL,
   'status',g.status,'slug',g.slug,'name',g.name,
   'origin_account_id',g.origin_account_id::text,'origin_command_id',g.origin_command_id::text,
   'origin_receipt_id',g.origin_receipt_id::text,'head',
   CASE WHEN h.group_id IS NULL THEN 'null'::jsonb ELSE jsonb_build_object('revision',h.revision::text,
    'incarnation',h.incarnation::text,'state',h.state) END) ORDER BY w.id),'[]'::jsonb)
  INTO group_rows FROM unnest(groups_to_lock) w(id) LEFT JOIN public.groups g ON g.id=w.id
   LEFT JOIN public.group_authority_heads h ON h.group_id=w.id;
 RETURN jsonb_build_object('codec',1,'actor',p_actor::text,'family',p_family::text,
  'input_digest',encode(decoded.input_digest,'hex'),'prospective_company',p_prospective_company::text,
  'prospective_group',p_prospective_group::text,'receipt',receipt_value,
  'groups',group_rows,'companies',company_rows,'users',user_rows,'mint',mint_value,
  'grants',grant_rows,'grantor_blocked',grantor_blocked);
END
$body$;

CREATE FUNCTION public.platform_legacy_topology_lock_plan_v1(
 p_actor uuid,p_family uuid,p_input bytea,p_prospective_company uuid,p_prospective_group uuid,p_expected_digest bytea)
RETURNS bytea LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE plan jsonb; digest bytea; ids uuid[]; material record;
 target_company uuid; decoded record;
 prior_org text:=current_setting('app.current_org',true);
 home constant uuid:='00000000-0000-0000-0000-00000000face';
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR (p_expected_digest IS NOT NULL AND octet_length(p_expected_digest)<>32) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_plan_input';
 END IF;
 plan:=public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,p_prospective_company,p_prospective_group);
 digest:=sha256(convert_to(plan::text,'UTF8'));
 IF p_expected_digest IS NOT NULL AND p_expected_digest IS DISTINCT FROM digest THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 SELECT array_agg((value->>'id')::uuid ORDER BY (value->>'id')::uuid) INTO ids
  FROM jsonb_array_elements(plan->'groups');
 PERFORM h.group_id FROM public.group_authority_heads h WHERE h.group_id=ANY(ids)
  ORDER BY h.group_id FOR UPDATE OF h;
 PERFORM g.id FROM public.groups g WHERE g.id=ANY(ids) ORDER BY g.id FOR UPDATE OF g;
 IF plan IS DISTINCT FROM public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,p_prospective_company,p_prospective_group) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(p_input);
 -- Company status/membership updates and purge require a retained UPDATE lock;
 -- source-only rows need SHARE, which conflicts with ordinary status updates.
 FOR target_company IN SELECT (value->>'id')::uuid FROM jsonb_array_elements(plan->'companies')
   ORDER BY (value->>'id')::uuid LOOP
  IF plan->'receipt'='null'::jsonb AND decoded.kind IN (4,5,8,9)
    AND target_company=(decoded.payload->>'org_id')::uuid THEN
   PERFORM o.id FROM public.organizations o WHERE o.id=target_company FOR UPDATE OF o;
  ELSE
   PERFORM o.id FROM public.organizations o WHERE o.id=target_company FOR SHARE OF o;
  END IF;
 END LOOP;
 IF plan IS DISTINCT FROM public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,p_prospective_company,p_prospective_group) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 SELECT array_agg((value->>'id')::uuid ORDER BY (value->>'id')::uuid) INTO ids
  FROM jsonb_array_elements(plan->'users');
 PERFORM u.id FROM public.users u WHERE u.id=ANY(ids) ORDER BY u.id FOR UPDATE OF u;
 IF plan IS DISTINCT FROM public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,p_prospective_company,p_prospective_group) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 PERFORM public.account_legacy_topology_roots_lock_v1(ids);
 IF plan IS DISTINCT FROM public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,p_prospective_company,p_prospective_group) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 PERFORM set_config('app.current_org',home::text,true);
 -- Only this source-query stage may classify Auth's finite refusal signals.
 BEGIN
  SELECT * INTO STRICT material FROM public.auth_legacy_platform_source_material_v1(p_actor,p_family);
 EXCEPTION WHEN OTHERS THEN
  IF (SQLSTATE='P0002' AND SQLERRM IN('auth_legacy.company_not_found',
    'account_company_deactivation.subject_not_found','auth_legacy.subject_not_found','auth_legacy_platform.family_not_found'))
   OR (SQLSTATE='28000' AND SQLERRM IN('auth_legacy_platform.source_fenced','auth_legacy_platform.source_inactive',
    'auth_legacy.subject_inactive','auth_legacy.subject_has_no_roles'))
   OR (SQLSTATE='P0001' AND SQLERRM='auth_legacy.fenced') THEN
   RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='platform_topology.source_denied';
  END IF;
  RAISE;
 END;
 -- Policy evaluation follows actual Auth capture; role denial is not authentication failure.
 IF ROW(material.family_id,material.family_user_id,material.family_org_id,material.family_protocol)
   IS DISTINCT FROM ROW(p_family,p_actor,home,'LEGACY_COMPANY'::text)
  OR material.family_revoked_at IS NOT NULL OR material.family_account_security_generation IS NOT NULL
  OR material.family_auth_time IS NOT NULL OR material.family_assurance IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='platform_topology.source_denied';
 END IF;
 IF plan IS DISTINCT FROM public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,p_prospective_company,p_prospective_group) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN digest;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_topology_plan_v1(uuid,uuid,bytea,uuid,uuid) OWNER TO console_app;
ALTER FUNCTION public.platform_legacy_topology_lock_plan_v1(uuid,uuid,bytea,uuid,uuid,bytea) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_topology_plan_v1(uuid,uuid,bytea,uuid,uuid),
 public.platform_legacy_topology_lock_plan_v1(uuid,uuid,bytea,uuid,uuid,bytea) FROM PUBLIC,
 console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,console_ontology_cmd,
 console_platform_force_cmd,console_account_owner,console_terms_owner,console_credential_owner,
 console_ontology_writer;
GRANT EXECUTE ON FUNCTION public.platform_legacy_topology_lock_plan_v1(uuid,uuid,bytea,uuid,uuid,bytea) TO console_rt;
GRANT EXECUTE ON FUNCTION public.auth_legacy_platform_source_material_v1(uuid,uuid) TO console_app;


-- source: legacy-force-plan-source-successor.sql
-- Private force plan/lock component. Reuses the exact complete removal cohort;
-- it does not decode force bytes as an ordinary kind8 command.
CREATE FUNCTION public.platform_force_remove_plan_v1(p_actor uuid,p_family uuid,p_input bytea) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d record;r public.platform_force_removal_receipts%ROWTYPE;cohort jsonb;
 company_value jsonb;membership_value jsonb;history_value jsonb;groups_value jsonb;users_value jsonb;source_company jsonb;
 groups_to_read uuid[];users_to_read uuid[];wiped jsonb;native_rows boolean:=false;
 home constant uuid:='00000000-0000-0000-0000-00000000face';
BEGIN
 SELECT * INTO STRICT d FROM public.platform_force_remove_decode_input_v1(p_input);
 IF d.actor_user_id IS DISTINCT FROM p_actor OR p_family IS NULL
  OR p_family='00000000-0000-0000-0000-000000000000'::uuid THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_force.invalid_command';
 END IF;
 SELECT x.* INTO r FROM public.platform_force_removal_receipts x
  WHERE x.actor_user_id=p_actor AND x.command_id=d.command_id;
 IF r.receipt_id IS NULL THEN
  cohort:=public.platform_company_removal_cohort_v1(d.org_id);
  SELECT array_agg(DISTINCT id ORDER BY id) INTO groups_to_read FROM (
   SELECT d.group_id AS id UNION ALL SELECT value::uuid FROM jsonb_array_elements_text(cohort->'group_ids')
  ) x;
  SELECT array_agg(DISTINCT id ORDER BY id) INTO users_to_read FROM (
   SELECT p_actor AS id UNION ALL SELECT value::uuid FROM jsonb_array_elements_text(cohort->'user_ids')
  ) x;
  SELECT to_jsonb(o) INTO company_value FROM public.organizations o WHERE o.id=d.org_id;
  SELECT to_jsonb(m) INTO membership_value FROM public.group_memberships m WHERE m.org_id=d.org_id;
  SELECT to_jsonb(h) INTO history_value FROM public.group_memberships m JOIN public.group_membership_revisions h
   ON h.membership_id=m.membership_id AND h.revision=m.current_revision AND h.incarnation=m.incarnation
   WHERE m.org_id=d.org_id;
  native_rows:=public.account_company_native_rows_present_v1(d.org_id);
  SELECT jsonb_build_object(
   'users',(SELECT count(*) FROM public.users WHERE org_id=d.org_id),
   'registry_customers',(SELECT count(*) FROM public.registry_customers WHERE org_id=d.org_id),
   'registry_sites',(SELECT count(*) FROM public.registry_sites WHERE org_id=d.org_id),
   'registry_equipment',(SELECT count(*) FROM public.registry_equipment WHERE org_id=d.org_id),
   'work_orders',(SELECT count(*) FROM public.work_orders WHERE org_id=d.org_id),
   'financial_rental_quotes',(SELECT count(*) FROM public.financial_rental_quotes WHERE org_id=d.org_id),
   'financial_purchase_requests',(SELECT count(*) FROM public.financial_purchase_requests WHERE org_id=d.org_id),
   'messenger_threads',(SELECT count(*) FROM public.messenger_threads WHERE org_id=d.org_id),
   'evidence_media',(SELECT count(*) FROM public.evidence_media WHERE org_id=d.org_id),
   'audit_events',(SELECT count(*) FROM public.audit_events WHERE org_id=d.org_id)) INTO wiped;
 ELSE
  groups_to_read:=ARRAY[]::uuid[];users_to_read:=ARRAY[p_actor];
  cohort:=jsonb_build_object('grants','[]'::jsonb,'grantor_blocked',false);
 END IF;
 IF cardinality(groups_to_read)>128 THEN
  RAISE EXCEPTION USING ERRCODE='54000',MESSAGE='platform_topology.cohort_capacity';
 END IF;
 SELECT to_jsonb(o) INTO source_company FROM public.organizations o WHERE o.id=home;
 SELECT coalesce(jsonb_agg(jsonb_build_object('id',w.id::text,'exists',g.id IS NOT NULL,
  'status',g.status,'slug',g.slug,'name',g.name,'origin_account_id',g.origin_account_id,
  'origin_command_id',g.origin_command_id,'origin_receipt_id',g.origin_receipt_id,'head',
  CASE WHEN h.group_id IS NULL THEN 'null'::jsonb ELSE jsonb_build_object('revision',h.revision::text,
    'incarnation',h.incarnation::text,'state',h.state) END) ORDER BY w.id),'[]'::jsonb)
  INTO groups_value FROM unnest(groups_to_read) w(id) LEFT JOIN public.groups g ON g.id=w.id
   LEFT JOIN public.group_authority_heads h ON h.group_id=w.id;
 SELECT jsonb_agg(jsonb_build_object('id',w.id::text,'exists',u.id IS NOT NULL,'org_id',u.org_id::text,
  'row',CASE WHEN u.id IS NULL THEN 'null'::jsonb ELSE to_jsonb(u) END) ORDER BY w.id)
  INTO users_value FROM unnest(users_to_read) w(id) LEFT JOIN public.users u ON u.id=w.id;
 RETURN jsonb_build_object('protocol','PLATFORM_FORCE_REMOVE_V1','actor',p_actor,'family',p_family,
  'input_digest',encode(d.input_digest,'hex'),'receipt',CASE WHEN r.receipt_id IS NULL THEN 'null'::jsonb ELSE to_jsonb(r) END,
  'company',company_value,'source_company',source_company,'membership',membership_value,'membership_history',history_value,
  'groups',groups_value,'users',users_value,'grants',cohort->'grants','grantor_blocked',cohort->'grantor_blocked',
  'has_native_rows',native_rows,'wiped',wiped);
END
$body$;

CREATE FUNCTION public.platform_force_remove_lock_plan_v1(p_actor uuid,p_family uuid,p_input bytea,p_expected bytea)
RETURNS bytea LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d record;plan jsonb;digest bytea;ids uuid[];company uuid;source record;
 home constant uuid:='00000000-0000-0000-0000-00000000face';
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR (p_expected IS NOT NULL AND octet_length(p_expected)<>32) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_force.invalid_plan';
 END IF;
 SELECT * INTO STRICT d FROM public.platform_force_remove_decode_input_v1(p_input);
 plan:=public.platform_force_remove_plan_v1(p_actor,p_family,p_input);digest:=sha256(convert_to(plan::text,'UTF8'));
 IF p_expected IS NOT NULL AND p_expected IS DISTINCT FROM digest THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_force.plan_changed';
 END IF;
 SELECT array_agg((v->>'id')::uuid ORDER BY (v->>'id')::uuid) INTO ids FROM jsonb_array_elements(plan->'groups') t(v);
 PERFORM h.group_id FROM public.group_authority_heads h WHERE h.group_id=ANY(ids) ORDER BY h.group_id FOR UPDATE OF h;
 PERFORM g.id FROM public.groups g WHERE g.id=ANY(ids) ORDER BY g.id FOR UPDATE OF g;
 IF plan IS DISTINCT FROM public.platform_force_remove_plan_v1(p_actor,p_family,p_input) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_force.plan_changed';
 END IF;
 FOR company IN SELECT DISTINCT id FROM unnest(ARRAY[home,CASE WHEN plan->'receipt'='null'::jsonb THEN d.org_id END]) x(id)
  WHERE id IS NOT NULL ORDER BY id LOOP
  IF company=d.org_id AND company<>home THEN
   PERFORM o.id FROM public.organizations o WHERE o.id=company FOR UPDATE OF o;
  ELSE PERFORM o.id FROM public.organizations o WHERE o.id=company FOR SHARE OF o; END IF;
 END LOOP;
 IF plan IS DISTINCT FROM public.platform_force_remove_plan_v1(p_actor,p_family,p_input) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_force.plan_changed';
 END IF;
 SELECT array_agg((v->>'id')::uuid ORDER BY (v->>'id')::uuid) INTO ids FROM jsonb_array_elements(plan->'users') t(v);
 PERFORM u.id FROM public.users u WHERE u.id=ANY(ids) ORDER BY u.id FOR UPDATE OF u;
 IF plan IS DISTINCT FROM public.platform_force_remove_plan_v1(p_actor,p_family,p_input) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_force.plan_changed';
 END IF;
 PERFORM public.account_legacy_topology_roots_lock_v1(ids);
 PERFORM set_config('app.current_org',home::text,true);
 -- Only this source-query stage may classify Auth's finite refusal signals.
 BEGIN
  SELECT * INTO STRICT source FROM public.auth_legacy_platform_source_material_v1(p_actor,p_family);
 EXCEPTION WHEN OTHERS THEN
  IF (SQLSTATE='P0002' AND SQLERRM IN('auth_legacy.company_not_found',
    'account_company_deactivation.subject_not_found','auth_legacy.subject_not_found','auth_legacy_platform.family_not_found'))
   OR (SQLSTATE='28000' AND SQLERRM IN('auth_legacy_platform.source_fenced','auth_legacy_platform.source_inactive',
    'auth_legacy.subject_inactive','auth_legacy.subject_has_no_roles'))
   OR (SQLSTATE='P0001' AND SQLERRM='auth_legacy.fenced') THEN
   RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='platform_force.source_denied';
  END IF;
  RAISE;
 END;
 -- Policy evaluation follows actual Auth capture; role denial is not authentication failure.
 IF ROW(source.family_id,source.family_user_id,source.family_org_id,source.family_protocol)
   IS DISTINCT FROM ROW(p_family,p_actor,home,'LEGACY_COMPANY'::text)
  OR source.family_revoked_at IS NOT NULL OR source.family_account_security_generation IS NOT NULL
  OR source.family_auth_time IS NOT NULL OR source.family_assurance IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='28000',MESSAGE='platform_force.source_denied';
 END IF;
 IF plan IS DISTINCT FROM public.platform_force_remove_plan_v1(p_actor,p_family,p_input) THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_force.plan_changed';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RETURN digest;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RAISE;
END
$body$;
ALTER FUNCTION public.platform_force_remove_plan_v1(uuid,uuid,bytea) OWNER TO console_app;
ALTER FUNCTION public.platform_force_remove_lock_plan_v1(uuid,uuid,bytea,bytea) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_force_remove_plan_v1(uuid,uuid,bytea),
 public.platform_force_remove_lock_plan_v1(uuid,uuid,bytea,bytea) FROM PUBLIC;


-- source: legacy-force-shared-guards.sql
-- Private finite force branches for existing shared physical boundaries.
CREATE FUNCTION public.account_company_native_rows_present_v1(p_org uuid) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE answer boolean;prior_org text:=current_setting('app.current_org',true);
BEGIN
 PERFORM set_config('app.current_org',p_org::text,true);
 SELECT EXISTS(SELECT 1 FROM public.company_actors a WHERE a.org_id=p_org)
  OR EXISTS(SELECT 1 FROM public.company_authority_heads h WHERE h.org_id=p_org)
  OR EXISTS(SELECT 1 FROM public.company_enrollment_effect_bindings b WHERE b.org_id=p_org) INTO answer;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RETURN answer;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RAISE;
END
$body$;
ALTER FUNCTION public.account_company_native_rows_present_v1(uuid) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.account_company_native_rows_present_v1(uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.account_company_native_rows_present_v1(uuid) TO console_app;

CREATE FUNCTION public.platform_force_effect_admit_v1(p_table text,p_op text,p_old jsonb,p_new jsonb) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE f public.platform_force_removal_effect_bindings%ROWTYPE;captured jsonb;expected jsonb;prior_history jsonb;
BEGIN
 SELECT x.* INTO f FROM public.platform_force_removal_effect_bindings x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 IF NOT FOUND THEN
  IF EXISTS(SELECT 1 FROM public.platform_force_removal_receipts r
    WHERE r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid()) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.tail_effect_denied';
  END IF;
  RETURN false;
 END IF;
 IF f.plan_snapshot->'has_native_rows' IS DISTINCT FROM 'false'::jsonb
  OR f.plan_snapshot->'grantor_blocked' IS DISTINCT FROM 'false'::jsonb
  OR f.plan_snapshot->'company'->>'status' IS DISTINCT FROM 'ARCHIVED'
  OR f.plan_snapshot->'company'->>'origin_account_id' IS NOT NULL
  OR EXISTS(SELECT 1 FROM jsonb_array_elements(f.plan_snapshot->'groups') t(v)
   WHERE v->>'origin_account_id' IS NOT NULL) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_denied';
 END IF;
 CASE p_table
 WHEN 'users' THEN
  SELECT v->'row' INTO STRICT captured FROM jsonb_array_elements(f.plan_snapshot->'users') t(v)
   WHERE v->>'id'=p_old->>'id' AND v->>'org_id'=f.org_id::text;
  expected:=captured||jsonb_build_object('employee_id',NULL);
  IF (p_op='UPDATE' AND (p_old IS DISTINCT FROM captured OR p_new IS DISTINCT FROM expected))
   OR (p_op='DELETE' AND p_old IS DISTINCT FROM expected) OR p_op NOT IN('UPDATE','DELETE') THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.user_effect_invalid';
  END IF;
 WHEN 'group_role_grants' THEN
  SELECT value INTO STRICT captured FROM jsonb_array_elements(f.plan_snapshot->'grants')
   WHERE value->>'id'=p_old->>'id';
  IF p_op<>'DELETE' OR p_old IS DISTINCT FROM captured THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.grant_effect_invalid';
  END IF;
 WHEN 'organizations' THEN
  IF p_op<>'DELETE' OR p_old IS DISTINCT FROM f.plan_snapshot->'company' THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.company_effect_invalid';
  END IF;
 WHEN 'group_memberships' THEN
  IF p_op<>'DELETE' OR p_old IS DISTINCT FROM f.plan_snapshot->'membership' THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.membership_effect_invalid';
  END IF;
 WHEN 'group_authority_heads' THEN
  SELECT v INTO STRICT captured FROM jsonb_array_elements(f.plan_snapshot->'groups') t(v)
   WHERE v->>'id'=p_old->>'group_id';
  expected:=jsonb_build_object('group_id',captured->>'id','revision',(captured->'head'->>'revision')::bigint,
   'incarnation',captured->'head'->>'incarnation','state',captured->'head'->>'state');
  IF p_op<>'UPDATE' OR p_old IS DISTINCT FROM expected
   OR p_new IS DISTINCT FROM expected||jsonb_build_object('revision',(captured->'head'->>'revision')::bigint+1) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.head_effect_invalid';
  END IF;
 WHEN 'group_membership_revisions' THEN
  prior_history:=f.plan_snapshot->'membership_history';
  expected:=prior_history||jsonb_build_object('revision',(prior_history->>'revision')::bigint+1,
   'to_time',f.occurred_at,'state','REMOVED','provenance_kind','LEGACY_FORCE_REMOVAL_V1',
   'native_account_id',NULL,'legacy_actor_user_id',NULL,'force_actor_user_id',f.actor_user_id,
   'command_id',f.command_id,'command_receipt',f.receipt_id);
  IF p_op<>'INSERT' OR p_new IS DISTINCT FROM expected THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.history_effect_invalid';
  END IF;
 ELSE RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.extra_effect_denied';
 END CASE;
 RETURN true;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_denied';
END
$body$;
ALTER FUNCTION public.platform_force_effect_admit_v1(text,text,jsonb,jsonb) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_force_effect_admit_v1(text,text,jsonb,jsonb) FROM PUBLIC;


-- source: legacy-force-closure-round3.sql
-- Private finite force closure. Historical command bytes and purge participants
-- are retained; this source cannot be activated without caller/ACL/custody cutover.
CREATE FUNCTION public.platform_force_frame_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d record;r public.platform_force_removal_receipts%ROWTYPE;
BEGIN
 IF current_user<>'console_app' OR TG_OP NOT IN('INSERT','DELETE') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.frame_invalid';
 END IF;
 IF TG_OP='INSERT' THEN
  SELECT * INTO STRICT d FROM public.platform_force_remove_decode_input_v1(NEW.input_bytes);
  IF ROW(d.actor_user_id,d.command_id,d.org_id,d.group_id,d.input_digest)
    IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.org_id,NEW.group_id,NEW.input_digest)
   OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
   OR EXISTS(SELECT 1 FROM public.platform_force_removal_receipts x
    WHERE (x.actor_user_id=NEW.actor_user_id AND x.command_id=NEW.command_id)
     OR (x.effect_xid=NEW.effect_xid AND x.effect_backend_pid=NEW.effect_backend_pid))
   OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts x
    WHERE x.effect_xid=NEW.effect_xid AND x.effect_backend_pid=NEW.effect_backend_pid)
   OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_effect_bindings x
    WHERE x.effect_xid=NEW.effect_xid AND x.effect_backend_pid=NEW.effect_backend_pid)
   OR NEW.plan_snapshot IS DISTINCT FROM public.platform_force_remove_plan_v1(NEW.actor_user_id,NEW.family_id,NEW.input_bytes) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.frame_invalid';
  END IF;
  RETURN NEW;
 END IF;
 SELECT x.* INTO STRICT r FROM public.platform_force_removal_receipts x
  WHERE x.actor_user_id=OLD.actor_user_id AND x.command_id=OLD.command_id;
 IF OLD.effect_xid IS DISTINCT FROM pg_current_xact_id() OR OLD.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR ROW(r.receipt_id,r.input_bytes,r.input_digest,r.org_id,r.group_id,r.family_id,r.effect_xid,r.effect_backend_pid,
    r.occurred_at,r.trace_id,r.span_id)
   IS DISTINCT FROM ROW(OLD.receipt_id,OLD.input_bytes,OLD.input_digest,OLD.org_id,OLD.group_id,OLD.family_id,OLD.effect_xid,
    OLD.effect_backend_pid,OLD.occurred_at,OLD.trace_id,OLD.span_id) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.frame_invalid';
 END IF;
 RETURN OLD;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.frame_invalid';
END
$body$;

CREATE FUNCTION public.platform_force_receipt_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY INVOKER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE f public.platform_force_removal_effect_bindings%ROWTYPE;
BEGIN
 IF current_user<>'console_app' OR TG_OP<>'INSERT' THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.receipt_invalid';
 END IF;
 SELECT x.* INTO STRICT f FROM public.platform_force_removal_effect_bindings x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 IF ROW(NEW.actor_user_id,NEW.command_id,NEW.receipt_id,NEW.input_bytes,NEW.input_digest,NEW.org_id,NEW.group_id,
   NEW.family_id,NEW.effect_xid,NEW.effect_backend_pid,NEW.occurred_at,NEW.trace_id,NEW.span_id)
  IS DISTINCT FROM ROW(f.actor_user_id,f.command_id,f.receipt_id,f.input_bytes,f.input_digest,f.org_id,f.group_id,
   f.family_id,f.effect_xid,f.effect_backend_pid,f.occurred_at,f.trace_id,f.span_id) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.receipt_invalid';
 END IF;
 RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.receipt_invalid';
END
$body$;

CREATE FUNCTION public.platform_force_frame_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_force_removal_receipts%ROWTYPE;d record;item jsonb;actual jsonb;expected jsonb;
 before_heads jsonb;after_heads jsonb;actual_heads jsonb;history jsonb;event public.audit_events%ROWTYPE;refusal text;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT x.* INTO STRICT r FROM public.platform_force_removal_receipts x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 SELECT * INTO STRICT d FROM public.platform_force_remove_decode_input_v1(NEW.input_bytes);
 IF ROW(r.actor_user_id,r.command_id,r.receipt_id,r.input_bytes,r.input_digest,r.org_id,r.group_id,
   r.family_id,r.effect_xid,r.effect_backend_pid,r.occurred_at,r.trace_id,r.span_id)
  IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.receipt_id,NEW.input_bytes,NEW.input_digest,NEW.org_id,NEW.group_id,
   NEW.family_id,NEW.effect_xid,NEW.effect_backend_pid,NEW.occurred_at,NEW.trace_id,NEW.span_id)
  OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR EXISTS(SELECT 1 FROM public.platform_force_removal_effect_bindings x
   WHERE x.effect_xid=NEW.effect_xid AND x.effect_backend_pid=NEW.effect_backend_pid) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
 END IF;
 -- Independently derive the exact result from the retained pre-effect facts,
 -- preserving the owner's documented refusal precedence.
 IF NEW.plan_snapshot->'company'='null'::jsonb
  OR NEW.org_id='00000000-0000-0000-0000-00000000face'::uuid THEN refusal:='not_found';
 ELSIF NEW.plan_snapshot->'company'->>'origin_account_id' IS NOT NULL
  OR NEW.plan_snapshot->'has_native_rows' IS DISTINCT FROM 'false'::jsonb
  OR EXISTS(SELECT 1 FROM jsonb_array_elements(NEW.plan_snapshot->'groups') t(v)
   WHERE v->>'origin_account_id' IS NOT NULL) THEN refusal:='native_origin_denied';
 ELSIF NEW.plan_snapshot->'company'->>'group_id' IS DISTINCT FROM d.group_id::text
  OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(NEW.plan_snapshot->'groups') t(v)
   WHERE v->>'id'=d.group_id::text AND v->'head'->>'incarnation'=d.expected_incarnation::text
    AND v->'head'->>'revision'=d.expected_revision::text) THEN refusal:='revision_conflict';
 ELSIF NEW.plan_snapshot->'company'->>'status' IS DISTINCT FROM 'ARCHIVED' THEN refusal:='blocked_active';
 ELSIF NEW.plan_snapshot->'grantor_blocked' IS DISTINCT FROM 'false'::jsonb THEN refusal:='blocked_has_data';
 END IF;
 IF r.result_code IS DISTINCT FROM coalesce(refusal,'removed')
  OR (r.outcome='REJECTED') IS DISTINCT FROM (refusal IS NOT NULL) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
 END IF;
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 SELECT to_jsonb(o) INTO actual FROM public.organizations o WHERE o.id=NEW.org_id;
 IF (r.outcome='APPLIED' AND actual IS NOT NULL)
  OR (r.outcome='REJECTED' AND coalesce(actual,'null'::jsonb) IS DISTINCT FROM NEW.plan_snapshot->'company') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
 END IF;
 SELECT to_jsonb(m) INTO actual FROM public.group_memberships m WHERE m.org_id=NEW.org_id;
 IF (r.outcome='APPLIED' AND actual IS NOT NULL)
  OR (r.outcome='REJECTED' AND coalesce(actual,'null'::jsonb) IS DISTINCT FROM NEW.plan_snapshot->'membership') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
 END IF;
 FOR item IN SELECT value FROM jsonb_array_elements(NEW.plan_snapshot->'users') LOOP
  SELECT to_jsonb(u) INTO actual FROM public.users u WHERE u.id=(item->>'id')::uuid;
  IF r.outcome='APPLIED' AND item->>'org_id'=NEW.org_id::text THEN
   IF actual IS NOT NULL THEN RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';END IF;
  ELSIF coalesce(actual,'null'::jsonb) IS DISTINCT FROM item->'row' THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
  END IF;
 END LOOP;
 FOR item IN SELECT value FROM jsonb_array_elements(NEW.plan_snapshot->'grants') LOOP
  SELECT to_jsonb(g) INTO actual FROM public.group_role_grants g WHERE g.id=(item->>'id')::uuid;
  IF (r.outcome='APPLIED' AND actual IS NOT NULL) OR (r.outcome='REJECTED' AND actual IS DISTINCT FROM item) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
  END IF;
 END LOOP;
 SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',v->>'id','incarnation',v->'head'->>'incarnation',
  'revision',v->'head'->>'revision') ORDER BY (v->>'id')::uuid),'[]'::jsonb) INTO before_heads
  FROM jsonb_array_elements(NEW.plan_snapshot->'groups') t(v) WHERE v->'head'<>'null'::jsonb;
 SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',v->>'id','incarnation',v->'head'->>'incarnation',
  'revision',((v->'head'->>'revision')::bigint+CASE WHEN r.outcome='APPLIED' THEN 1 ELSE 0 END)::text) ORDER BY (v->>'id')::uuid),'[]'::jsonb) INTO after_heads
  FROM jsonb_array_elements(NEW.plan_snapshot->'groups') t(v) WHERE v->'head'<>'null'::jsonb;
 SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',h.group_id::text,'incarnation',h.incarnation::text,
  'revision',h.revision::text) ORDER BY h.group_id),'[]'::jsonb) INTO actual_heads
  FROM public.group_authority_heads h WHERE h.group_id IN
   (SELECT (value->>'id')::uuid FROM jsonb_array_elements(NEW.plan_snapshot->'groups'));
 IF r.heads_before IS DISTINCT FROM before_heads OR actual_heads IS DISTINCT FROM
  (CASE WHEN r.outcome='APPLIED' THEN after_heads ELSE before_heads END)
  OR r.heads_after IS DISTINCT FROM (CASE WHEN r.outcome='APPLIED' THEN after_heads ELSE '[]'::jsonb END) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
 END IF;
 FOR item IN SELECT value FROM jsonb_array_elements(NEW.plan_snapshot->'groups') LOOP
  SELECT jsonb_build_object('id',g.id::text,'exists',true,'status',g.status,'slug',g.slug,'name',g.name,
   'origin_account_id',g.origin_account_id,'origin_command_id',g.origin_command_id,'origin_receipt_id',g.origin_receipt_id)
   INTO actual FROM public.groups g WHERE g.id=(item->>'id')::uuid;
  IF (item->>'exists')::boolean THEN
   IF actual IS DISTINCT FROM item-'head' THEN RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';END IF;
  ELSIF actual IS NOT NULL THEN RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';END IF;
 END LOOP;
 SELECT to_jsonb(h) INTO history FROM public.group_membership_revisions h
  WHERE h.force_actor_user_id=NEW.actor_user_id AND h.command_id=NEW.command_id;
 IF r.outcome='APPLIED' THEN
  IF NEW.org_id='00000000-0000-0000-0000-00000000face'::uuid
   OR NEW.plan_snapshot->'company'->>'status' IS DISTINCT FROM 'ARCHIVED'
   OR NEW.plan_snapshot->'company'->>'origin_account_id' IS NOT NULL
   OR NEW.plan_snapshot->'company'->>'group_id' IS DISTINCT FROM d.group_id::text
   OR NEW.plan_snapshot->'has_native_rows' IS DISTINCT FROM 'false'::jsonb
   OR NEW.plan_snapshot->'grantor_blocked' IS DISTINCT FROM 'false'::jsonb
   OR EXISTS(SELECT 1 FROM public.users u WHERE u.org_id=NEW.org_id)
   OR EXISTS(SELECT 1 FROM jsonb_array_elements(NEW.plan_snapshot->'groups') t(v)
    WHERE v->>'origin_account_id' IS NOT NULL OR v->'head'='null'::jsonb)
   OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(before_heads) t(v)
    WHERE v->>'group_id'=d.group_id::text AND v->>'incarnation'=d.expected_incarnation::text
     AND v->>'revision'=d.expected_revision::text) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
  END IF;
  expected:=NEW.plan_snapshot->'membership_history'||jsonb_build_object(
   'revision',(NEW.plan_snapshot->'membership_history'->>'revision')::bigint+1,'to_time',NEW.occurred_at,
   'state','REMOVED','provenance_kind','LEGACY_FORCE_REMOVAL_V1','native_account_id',NULL,
   'legacy_actor_user_id',NULL,'force_actor_user_id',NEW.actor_user_id,'command_id',NEW.command_id,'command_receipt',NEW.receipt_id);
  IF history IS DISTINCT FROM expected OR (SELECT count(*) FROM public.group_membership_revisions h
   WHERE h.force_actor_user_id=NEW.actor_user_id AND h.command_id=NEW.command_id)<>1 THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
  END IF;
  SELECT a.* INTO STRICT event FROM public.audit_events a WHERE a.actor=NEW.actor_user_id
   AND a.action='platform.tenant.force_remove' AND a.trace_id=NEW.trace_id AND a.span_id=NEW.span_id AND a.occurred_at=NEW.occurred_at;
  IF ROW(event.org_id,event.target_type,event.target_id,event.branch_id,event.after_snap)
    IS DISTINCT FROM ROW(NULL::uuid,'organizations'::text,NEW.org_id::text,NULL::uuid,NULL::jsonb)
   OR event.before_snap IS DISTINCT FROM jsonb_build_object('org_id',NEW.org_id,'slug',NEW.plan_snapshot->'company'->>'slug',
    'name',NEW.plan_snapshot->'company'->>'name','wiped',NEW.plan_snapshot->'wiped') THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
  END IF;
 ELSE
  IF history IS NOT NULL OR EXISTS(SELECT 1 FROM public.audit_events a WHERE a.actor=NEW.actor_user_id
   AND a.action='platform.tenant.force_remove' AND a.trace_id=NEW.trace_id AND a.span_id=NEW.span_id AND a.occurred_at=NEW.occurred_at) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
  END IF;
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.closure_invalid';
WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RAISE;
END
$body$;

-- Called by the existing physical closure functions after the live frame is
-- gone. The BEFORE branch proves the captured row; retained NEW above proves
-- the complete cohort. Here actual OLD/NEW must also match the force result.
CREATE FUNCTION public.platform_force_effect_closed_v1(p_table text,p_op text,p_old jsonb,p_new jsonb) RETURNS boolean
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_force_removal_receipts%ROWTYPE;actual jsonb;before_value jsonb;after_value jsonb;
BEGIN
 SELECT x.* INTO r FROM public.platform_force_removal_receipts x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 IF NOT FOUND THEN
  IF EXISTS(SELECT 1 FROM public.platform_force_removal_effect_bindings f
   WHERE f.effect_xid=pg_current_xact_id() AND f.effect_backend_pid=pg_backend_pid()) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.unclosed_effect';
  END IF;
  RETURN false;
 END IF;
 IF r.outcome<>'APPLIED' THEN RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_closure_invalid';END IF;
 CASE p_table
 WHEN 'organizations' THEN
  IF p_op<>'DELETE' OR p_old->>'id' IS DISTINCT FROM r.org_id::text OR p_old->>'group_id' IS DISTINCT FROM r.group_id::text
   OR p_old->>'status' IS DISTINCT FROM 'ARCHIVED' OR p_old->>'origin_account_id' IS NOT NULL
   OR EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=r.org_id) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_closure_invalid';
  END IF;
 WHEN 'group_role_grants' THEN
  IF p_op<>'DELETE' OR EXISTS(SELECT 1 FROM public.group_role_grants g WHERE g.id=(p_old->>'id')::uuid) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_closure_invalid';
  END IF;
 WHEN 'group_authority_heads' THEN
  before_value:=jsonb_build_object('group_id',p_old->>'group_id','incarnation',p_old->>'incarnation','revision',p_old->>'revision');
  after_value:=jsonb_build_object('group_id',p_new->>'group_id','incarnation',p_new->>'incarnation','revision',p_new->>'revision');
  SELECT to_jsonb(h) INTO actual FROM public.group_authority_heads h WHERE h.group_id=(p_new->>'group_id')::uuid;
  IF p_op<>'UPDATE' OR actual IS DISTINCT FROM p_new
   OR p_new IS DISTINCT FROM p_old||jsonb_build_object('revision',(p_old->>'revision')::bigint+1)
   OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(r.heads_before) t(v) WHERE v=before_value)
   OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(r.heads_after) t(v) WHERE v=after_value) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_closure_invalid';
  END IF;
 ELSE RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.effect_closure_invalid';
 END CASE;
 RETURN true;
END
$body$;

ALTER FUNCTION public.platform_force_frame_guard_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_force_receipt_guard_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_force_frame_closed_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_force_effect_closed_v1(text,text,jsonb,jsonb) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_force_frame_guard_v1(),public.platform_force_receipt_guard_v1(),
 public.platform_force_frame_closed_v1(),public.platform_force_effect_closed_v1(text,text,jsonb,jsonb) FROM PUBLIC;
CREATE TRIGGER trg_platform_force_frame_guard BEFORE INSERT OR UPDATE OR DELETE
 ON public.platform_force_removal_effect_bindings FOR EACH ROW EXECUTE FUNCTION public.platform_force_frame_guard_v1();
CREATE TRIGGER trg_platform_force_frame_truncate BEFORE TRUNCATE
 ON public.platform_force_removal_effect_bindings FOR EACH STATEMENT EXECUTE FUNCTION public.platform_force_frame_guard_v1();
CREATE TRIGGER trg_platform_force_receipt_guard BEFORE INSERT OR UPDATE OR DELETE
 ON public.platform_force_removal_receipts FOR EACH ROW EXECUTE FUNCTION public.platform_force_receipt_guard_v1();
CREATE TRIGGER trg_platform_force_receipt_truncate BEFORE TRUNCATE
 ON public.platform_force_removal_receipts FOR EACH STATEMENT EXECUTE FUNCTION public.platform_force_receipt_guard_v1();
CREATE CONSTRAINT TRIGGER trg_platform_force_frame_closed AFTER INSERT
 ON public.platform_force_removal_effect_bindings DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_force_frame_closed_v1();
ALTER TABLE public.platform_force_removal_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_force_frame_guard;
ALTER TABLE public.platform_force_removal_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_force_frame_truncate;
ALTER TABLE public.platform_force_removal_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_force_frame_closed;
ALTER TABLE public.platform_force_removal_receipts ENABLE ALWAYS TRIGGER trg_platform_force_receipt_guard;
ALTER TABLE public.platform_force_removal_receipts ENABLE ALWAYS TRIGGER trg_platform_force_receipt_truncate;


-- source: legacy-force-child-sweep-successor.sql
-- Private successor; historical0208 remains unchanged. Existing order retained.
CREATE OR REPLACE FUNCTION platform_force_remove_direct_org_children(p_id UUID)
RETURNS VOID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public, pg_temp
AS $$
DECLARE
    target RECORD;
BEGIN
    -- Native identity is retained, even when a legacy Company row is malformed.
    IF public.account_company_native_rows_present_v1(p_id) THEN
        RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_force.native_origin_denied';
    END IF;
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
              'company_actors', 'company_authority_heads', 'company_enrollment_effect_bindings'
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
$$;

-- Keep the migration identity as owner, for the reason 0196 records: production
-- migrations run as console_app and isolated SQLx databases run as their own table
-- owner, so reassigning this SECURITY DEFINER would separate it from the tables it
-- must delete under FORCE RLS.
REVOKE ALL ON FUNCTION platform_force_remove_direct_org_children(UUID) FROM PUBLIC, console_rt, console_platform_force_cmd;


-- source: legacy-grant-effects-force-successor.sql
-- Private finite nine-kind component, not an installation resource.
-- Force must supply its separately reviewed frame before this can be activated.
CREATE FUNCTION public.platform_legacy_grant_write_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE frame public.platform_legacy_topology_effect_bindings%ROWTYPE; d record; captured jsonb;
BEGIN
 IF TG_OP IN('INSERT','DELETE') AND public.platform_force_effect_admit_v1(TG_TABLE_NAME,TG_OP,
  CASE WHEN TG_OP='INSERT' THEN NULL ELSE to_jsonb(OLD) END,
  CASE WHEN TG_OP='DELETE' THEN NULL ELSE to_jsonb(NEW) END) THEN RETURN OLD;END IF;
 IF TG_OP NOT IN('INSERT','DELETE') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_effect_invalid';
 END IF;
 SELECT x.* INTO STRICT frame FROM public.platform_legacy_topology_effect_bindings x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(frame.input_bytes);
 IF TG_OP='INSERT' THEN
  IF d.kind<>6 OR NEW.group_id IS DISTINCT FROM (d.payload->>'group_id')::uuid
   OR NEW.group_role IS DISTINCT FROM d.payload->>'group_role'
   OR NEW.granted_by IS DISTINCT FROM frame.actor_user_id
   OR NEW.created_at IS DISTINCT FROM transaction_timestamp()
   OR NOT EXISTS(SELECT 1 FROM public.users u WHERE u.id=NEW.user_id
    AND u.org_id=(d.payload->>'org_id')::uuid AND u.is_active
    AND u.display_name=d.payload->>'display_name' AND u.phone IS NOT DISTINCT FROM d.payload->>'phone'
    AND to_jsonb(u.roles)=d.payload->'tenant_roles') THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_effect_invalid';
  END IF;
  RETURN NEW;
 END IF;
 IF d.kind NOT IN(7,8) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_effect_invalid';
 END IF;
 SELECT value INTO STRICT captured FROM jsonb_array_elements(frame.plan_snapshot->'grants')
  WHERE value->>'id'=OLD.id::text;
 IF captured IS DISTINCT FROM to_jsonb(OLD)
  OR (d.kind=7 AND ROW(OLD.group_id,OLD.user_id,OLD.group_role)
   IS DISTINCT FROM ROW((d.payload->>'group_id')::uuid,(d.payload->>'user_id')::uuid,d.payload->>'group_role'))
  OR (d.kind=8 AND NOT EXISTS(SELECT 1 FROM jsonb_array_elements(frame.plan_snapshot->'users') t(v)
    WHERE v->>'id'=OLD.user_id::text AND v->>'org_id'=d.payload->>'org_id')) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_effect_invalid';
 END IF;
 RETURN OLD;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_effect_invalid';
END
$body$;

CREATE FUNCTION public.platform_legacy_grant_effect_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_legacy_topology_receipts%ROWTYPE; d record; actual jsonb;
BEGIN
 IF public.platform_force_effect_closed_v1(TG_TABLE_NAME,TG_OP,
  CASE WHEN TG_OP='INSERT' THEN NULL ELSE to_jsonb(OLD) END,
  CASE WHEN TG_OP='DELETE' THEN NULL ELSE to_jsonb(NEW) END) THEN RETURN NULL;END IF;
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid()
   AND x.outcome='APPLIED';
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(r.input_bytes);
 IF TG_OP='INSERT' THEN
  SELECT to_jsonb(g) INTO STRICT actual FROM public.group_role_grants g WHERE g.id=NEW.id;
  IF r.kind<>6 OR actual IS DISTINCT FROM to_jsonb(NEW)
   OR ROW(NEW.group_id,NEW.user_id,NEW.group_role,NEW.granted_by)
    IS DISTINCT FROM ROW(r.result_group_id,r.result_user_id,d.payload->>'group_role',r.actor_user_id)
   OR (SELECT count(*) FROM public.group_role_grants g WHERE g.user_id=r.result_user_id)<>1 THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_closure_invalid';
  END IF;
 ELSE
  IF r.kind NOT IN(7,8) OR EXISTS(SELECT 1 FROM public.group_role_grants g WHERE g.id=OLD.id)
   OR (r.kind=7 AND ROW(OLD.group_id,OLD.user_id,OLD.group_role)
    IS DISTINCT FROM ROW(r.result_group_id,r.result_user_id,d.payload->>'group_role')) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_closure_invalid';
  END IF;
 END IF;
 RETURN NULL;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.grant_closure_invalid';
END
$body$;

-- No-frame ordinary User operations remain under their existing owners. During
-- a finite command, an actual User DELETE must exactly match its captured row.
CREATE FUNCTION public.platform_legacy_user_delete_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE frame public.platform_legacy_topology_effect_bindings%ROWTYPE; d record; captured jsonb;
BEGIN
 IF public.platform_force_effect_admit_v1(TG_TABLE_NAME,TG_OP,to_jsonb(OLD),NULL) THEN RETURN OLD;END IF;
 SELECT x.* INTO frame FROM public.platform_legacy_topology_effect_bindings x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid();
 IF NOT FOUND THEN
  IF EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts r
    WHERE r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid()) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_effect_invalid';
  END IF;
  RETURN OLD;
 END IF;
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(frame.input_bytes);
 IF d.kind<>8 OR OLD.org_id IS DISTINCT FROM (d.payload->>'org_id')::uuid THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_effect_invalid';
 END IF;
 SELECT value->'row' INTO STRICT captured FROM jsonb_array_elements(frame.plan_snapshot->'users')
  WHERE value->>'id'=OLD.id::text;
 IF captured IS DISTINCT FROM to_jsonb(OLD) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_effect_invalid';
 END IF;
 RETURN OLD;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_effect_invalid';
END
$body$;

ALTER FUNCTION public.platform_legacy_grant_write_guard_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_legacy_grant_effect_closed_v1() OWNER TO console_app;
ALTER FUNCTION public.platform_legacy_user_delete_guard_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_grant_write_guard_v1(),
 public.platform_legacy_grant_effect_closed_v1(),public.platform_legacy_user_delete_guard_v1() FROM PUBLIC;
CREATE TRIGGER trg_platform_legacy_grant_write_guard BEFORE INSERT OR UPDATE OR DELETE
 ON public.group_role_grants FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_grant_write_guard_v1();
CREATE TRIGGER trg_platform_legacy_grant_truncate_guard BEFORE TRUNCATE
 ON public.group_role_grants FOR EACH STATEMENT EXECUTE FUNCTION public.platform_legacy_grant_write_guard_v1();
CREATE CONSTRAINT TRIGGER trg_platform_legacy_grant_effect_closed AFTER INSERT OR DELETE
 ON public.group_role_grants DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_grant_effect_closed_v1();
CREATE TRIGGER trg_platform_legacy_user_delete_guard BEFORE DELETE ON public.users
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_user_delete_guard_v1();
ALTER TABLE public.group_role_grants ENABLE ALWAYS TRIGGER trg_platform_legacy_grant_write_guard;
ALTER TABLE public.group_role_grants ENABLE ALWAYS TRIGGER trg_platform_legacy_grant_truncate_guard;
ALTER TABLE public.group_role_grants ENABLE ALWAYS TRIGGER trg_platform_legacy_grant_effect_closed;
ALTER TABLE public.users ENABLE ALWAYS TRIGGER trg_platform_legacy_user_delete_guard;

-- No ordinary-nine participant updates a User. Keep unrelated ordinary owner
-- updates outside these transactions; a completed receipt fences tail writes.
CREATE FUNCTION public.platform_legacy_user_update_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF public.platform_force_effect_admit_v1(TG_TABLE_NAME,TG_OP,to_jsonb(OLD),to_jsonb(NEW)) THEN RETURN NEW;END IF;
 IF EXISTS(SELECT 1 FROM public.platform_legacy_topology_effect_bindings b
    WHERE b.effect_xid=pg_current_xact_id() AND b.effect_backend_pid=pg_backend_pid())
  OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts r
    WHERE r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid()) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.user_effect_invalid';
 END IF;
 RETURN NEW;
END
$body$;
ALTER FUNCTION public.platform_legacy_user_update_guard_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_user_update_guard_v1() FROM PUBLIC;
CREATE TRIGGER trg_platform_legacy_user_update_guard BEFORE UPDATE ON public.users
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_user_update_guard_v1();
ALTER TABLE public.users ENABLE ALWAYS TRIGGER trg_platform_legacy_user_update_guard;


-- source: legacy-head-closure-force-successor.sql
-- Deferred OLD/NEW is the independent before witness. No mutable completion
-- flag or extra generic command binding is introduced for the other kinds.
CREATE FUNCTION public.platform_legacy_head_effect_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE target uuid; receipt public.platform_legacy_topology_receipts%ROWTYPE;
 actual public.group_authority_heads%ROWTYPE; group_row public.groups%ROWTYPE;
 decoded record; before_value jsonb; after_value jsonb;
BEGIN
 IF public.platform_force_effect_closed_v1(TG_TABLE_NAME,TG_OP,
  CASE WHEN TG_OP='INSERT' THEN NULL ELSE to_jsonb(OLD) END,
  CASE WHEN TG_OP='DELETE' THEN NULL ELSE to_jsonb(NEW) END) THEN RETURN NULL;END IF;
 IF TG_OP='DELETE' THEN target:=OLD.group_id; ELSE target:=NEW.group_id; END IF;
 SELECT g.* INTO STRICT group_row FROM public.groups g WHERE g.id=target;
 IF group_row.origin_account_id IS NOT NULL THEN
  -- Native origin has its own mandatory birth/binding closure and no mutable
  -- legacy receipt may establish its provenance.
  RETURN NULL;
 END IF;
 IF TG_OP NOT IN ('INSERT','UPDATE') THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
 END IF;
 SELECT r.* INTO STRICT receipt FROM public.platform_legacy_topology_receipts r
  WHERE r.effect_xid=pg_current_xact_id() AND r.effect_backend_pid=pg_backend_pid()
   AND r.outcome='APPLIED' AND r.result_code='applied'
   AND EXISTS(SELECT 1 FROM jsonb_array_elements(r.heads_after) x
    WHERE x.value->>'group_id'=target::text);
 SELECT * INTO STRICT decoded FROM public.platform_legacy_topology_decode_input_v1(receipt.input_bytes);
 IF ROW(decoded.actor_user_id,decoded.command_id,decoded.kind,decoded.input_digest,decoded.primary_target)
   IS DISTINCT FROM ROW(receipt.actor_user_id,receipt.command_id,receipt.kind,receipt.input_digest,receipt.primary_target)
  OR group_row.origin_command_id IS NOT NULL OR group_row.origin_receipt_id IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
 END IF;
 SELECT a.* INTO STRICT actual FROM public.group_authority_heads a WHERE a.group_id=target;
 IF actual IS DISTINCT FROM NEW THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
 END IF;
 after_value:=jsonb_build_object('group_id',NEW.group_id::text,
  'incarnation',NEW.incarnation::text,'revision',NEW.revision::text);
 IF (SELECT count(*) FROM jsonb_array_elements(receipt.heads_after) x
    WHERE x.value->>'group_id'=target::text)<>1
  OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(receipt.heads_after) x WHERE x.value=after_value) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
 END IF;
 IF TG_OP='INSERT' THEN
  IF receipt.kind NOT IN (1,2,5) OR receipt.result_group_id IS DISTINCT FROM target
   OR NEW.revision IS DISTINCT FROM 1 OR NEW.state IS DISTINCT FROM 'ACTIVE'
   OR EXISTS(SELECT 1 FROM jsonb_array_elements(receipt.heads_before) x
    WHERE x.value->>'group_id'=target::text) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
  END IF;
 ELSE
  before_value:=jsonb_build_object('group_id',OLD.group_id::text,
   'incarnation',OLD.incarnation::text,'revision',OLD.revision::text);
  IF receipt.kind=2 OR NEW.group_id IS DISTINCT FROM OLD.group_id
   OR NEW.incarnation IS DISTINCT FROM OLD.incarnation OR NEW.state IS DISTINCT FROM OLD.state
   OR NEW.revision IS DISTINCT FROM OLD.revision+1
   OR (SELECT count(*) FROM jsonb_array_elements(receipt.heads_before) x
     WHERE x.value->>'group_id'=target::text)<>1
   OR NOT EXISTS(SELECT 1 FROM jsonb_array_elements(receipt.heads_before) x WHERE x.value=before_value) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
  END IF;
 END IF;
 RETURN NULL;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.head_closure_invalid';
END
$body$;
ALTER FUNCTION public.platform_legacy_head_effect_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_head_effect_closed_v1() FROM PUBLIC,
 console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,console_ontology_cmd,
 console_platform_force_cmd,console_account_owner,console_terms_owner,console_credential_owner,
 console_ontology_writer;
CREATE CONSTRAINT TRIGGER trg_platform_legacy_head_effect_closed
 AFTER INSERT OR UPDATE OR DELETE ON public.group_authority_heads
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW
 EXECUTE FUNCTION public.platform_legacy_head_effect_closed_v1();


-- source: legacy-entity-closure-force-successor.sql
-- Private candidate. Retained OLD/NEW prevents omitted/extra topology effects
-- from being justified solely by a coordinator's claimed receipt/head arrays.
-- Force has a separate successor obligation; do not activate without it.
CREATE FUNCTION public.platform_legacy_entity_effect_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE r public.platform_legacy_topology_receipts%ROWTYPE; d record;
 before_row jsonb; after_row jsonb; expected jsonb; current_row jsonb;
 target uuid; event public.audit_events%ROWTYPE;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 IF public.platform_force_effect_closed_v1(TG_TABLE_NAME,TG_OP,
  CASE WHEN TG_OP='INSERT' THEN NULL ELSE to_jsonb(OLD) END,
  CASE WHEN TG_OP='DELETE' THEN NULL ELSE to_jsonb(NEW) END) THEN RETURN NULL;END IF;
 IF TG_OP<>'INSERT' THEN before_row:=to_jsonb(OLD);END IF;
 IF TG_OP<>'DELETE' THEN after_row:=to_jsonb(NEW);END IF;
 -- Native provenance has its separately enforced binding, birth and receipt closure.
 IF coalesce(before_row,after_row)->>'origin_account_id' IS NOT NULL THEN RETURN NULL;END IF;
 target:=(coalesce(after_row,before_row)->>'id')::uuid;
 SELECT x.* INTO STRICT r FROM public.platform_legacy_topology_receipts x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid()
   AND x.outcome='APPLIED' AND x.result_code='applied';
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(r.input_bytes);
 IF ROW(d.actor_user_id,d.command_id,d.kind,d.primary_target,d.input_digest)
  IS DISTINCT FROM ROW(r.actor_user_id,r.command_id,r.kind,r.primary_target,r.input_digest)
  OR before_row->>'origin_command_id' IS NOT NULL OR after_row->>'origin_command_id' IS NOT NULL
  OR before_row->>'origin_receipt_id' IS NOT NULL OR after_row->>'origin_receipt_id' IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
 END IF;
 IF TG_TABLE_NAME='groups' THEN
  IF target IS DISTINCT FROM r.result_group_id OR TG_OP='DELETE' THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
  END IF;
  SELECT to_jsonb(g) INTO STRICT current_row FROM public.groups g WHERE g.id=target;
  IF current_row IS DISTINCT FROM after_row THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
  END IF;
  IF TG_OP='INSERT' THEN
   IF r.kind NOT IN(1,2,5) OR EXISTS(SELECT 1 FROM jsonb_array_elements(r.heads_before) t(v)
    WHERE v->>'group_id'=target::text) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
   END IF;
  ELSE
   IF r.kind<>3 OR target IS DISTINCT FROM (d.payload->>'group_id')::uuid THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
   END IF;
   expected:=(before_row-'updated_at')||jsonb_build_object(
    'slug',coalesce(d.payload->>'slug',before_row->>'slug'),
    'name',coalesce(d.payload->>'name',before_row->>'name'),
    'status',coalesce(d.payload->>'status',before_row->>'status'));
   IF after_row-'updated_at' IS DISTINCT FROM expected THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
   END IF;
  END IF;
 ELSIF TG_TABLE_NAME='organizations' THEN
  IF target IS DISTINCT FROM r.result_org_id THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
  END IF;
  PERFORM set_config('app.current_org',target::text,true);
  IF TG_OP='DELETE' THEN
   IF r.kind<>8 OR target IS DISTINCT FROM (d.payload->>'org_id')::uuid
    OR (before_row->>'group_id')::uuid IS DISTINCT FROM r.result_group_id
    OR EXISTS(SELECT 1 FROM public.organizations o WHERE o.id=target) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
   END IF;
   -- The old Company name/slug may not be reconstructed from a mutable receipt.
   SELECT a.* INTO STRICT event FROM public.audit_events a
    WHERE a.actor=r.actor_user_id AND a.action='platform.tenant.remove'
     AND a.trace_id=r.trace_id AND a.span_id=r.span_id AND a.occurred_at=r.occurred_at;
   IF event.target_type IS DISTINCT FROM 'organizations' OR event.target_id IS DISTINCT FROM target::text
    OR event.org_id IS NOT NULL OR event.before_snap IS NOT NULL
    OR event.after_snap IS DISTINCT FROM jsonb_build_object('org_id',target,'slug',before_row->>'slug') THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
   END IF;
  ELSE
   SELECT to_jsonb(o) INTO STRICT current_row FROM public.organizations o WHERE o.id=target;
   IF current_row IS DISTINCT FROM after_row THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
   END IF;
   IF TG_OP='INSERT' THEN
    IF r.kind<>1 OR after_row->>'slug' IS DISTINCT FROM d.payload->>'slug'
     OR after_row->>'name' IS DISTINCT FROM d.payload->>'name'
     OR after_row->>'status' IS DISTINCT FROM 'ACTIVE'
     OR (after_row->>'group_id')::uuid IS DISTINCT FROM r.result_group_id THEN
     RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
    END IF;
   ELSE
    IF r.kind IN(4,5) THEN
     expected:=(before_row-'updated_at')||jsonb_build_object('group_id',r.result_group_id);
    ELSIF r.kind=9 THEN
     expected:=(before_row-'updated_at')||jsonb_build_object('status',d.payload->>'status');
    ELSE RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
    END IF;
    IF target IS DISTINCT FROM (d.payload->>'org_id')::uuid
     OR after_row-'updated_at' IS DISTINCT FROM expected THEN
     RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
    END IF;
   END IF;
  END IF;
 ELSE RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RETURN NULL;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.entity_closure_invalid';
WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_entity_effect_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_entity_effect_closed_v1() FROM PUBLIC;
CREATE CONSTRAINT TRIGGER trg_platform_legacy_entity_effect_closed
 AFTER INSERT OR UPDATE OR DELETE ON public.groups DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_entity_effect_closed_v1();
CREATE CONSTRAINT TRIGGER trg_platform_legacy_entity_effect_closed
 AFTER INSERT OR UPDATE OR DELETE ON public.organizations DEFERRABLE INITIALLY DEFERRED
 FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_entity_effect_closed_v1();


-- source: legacy-bootstrap-receipt-matches-round1.sql
-- Prospective credential-owner predicate; no serving table grant or row disclosure.
-- Historical auth_legacy_bootstrap_issued_v1 and receipt bytes remain unchanged.
CREATE FUNCTION public.auth_legacy_bootstrap_receipt_matches_v1(
 p_company uuid,p_subject uuid,p_id uuid,p_issued_at timestamptz,p_expires_at timestamptz)
RETURNS boolean LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE CALLED ON NULL INPUT
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); valid boolean;
BEGIN
 IF p_company IS NULL OR p_subject IS NULL OR p_id IS NULL
  OR p_issued_at IS NULL OR p_expires_at IS NULL THEN RETURN false; END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT EXISTS(SELECT 1 FROM public.auth_bootstrap_credentials b
  WHERE b.id=p_id AND b.org_id=p_company AND b.user_id=p_subject
   AND b.issued_at=p_issued_at AND b.expires_at=p_expires_at
   AND octet_length(b.token_hash)=32 AND b.consumed_at IS NULL
   AND b.revoked_at IS NULL AND b.registration_ceremony_id IS NULL
   AND b.registration_started_at IS NULL)
 INTO valid;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN valid;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.auth_legacy_bootstrap_receipt_matches_v1(uuid,uuid,uuid,timestamptz,timestamptz)
 OWNER TO console_credential_owner;
REVOKE ALL ON FUNCTION public.auth_legacy_bootstrap_receipt_matches_v1(uuid,uuid,uuid,timestamptz,timestamptz)
 FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.auth_legacy_bootstrap_receipt_matches_v1(uuid,uuid,uuid,timestamptz,timestamptz)
 TO console_app;


-- source: legacy-receipt-closure-kind8-successor.sql
-- Private finite closure candidate. Requires independent source/runtime review.
-- No serving installation, custody activation or force-removal successor claim.
CREATE FUNCTION public.platform_legacy_receipt_closed_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d record; original public.platform_legacy_topology_receipts%ROWTYPE;
 o public.organizations%ROWTYPE; g public.groups%ROWTYPE; u public.users%ROWTYPE;
 m public.group_memberships%ROWTYPE; removed public.group_membership_revisions%ROWTYPE;
 audit public.audit_events%ROWTYPE;
 expected_groups uuid[]; before_groups uuid[]; after_groups uuid[]; requested_group uuid;
 actual_heads jsonb; expected_client_heads jsonb; before_head jsonb; after_head jsonb;
 history_count bigint; no_change boolean:=false; new_group boolean:=false;
 action_name text; target_type text; target_id text; audit_org uuid;
 expected_before jsonb; expected_after jsonb; prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT r.* INTO STRICT original FROM public.platform_legacy_topology_receipts r
  WHERE r.actor_user_id=NEW.actor_user_id AND r.command_id=NEW.command_id;
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(NEW.input_bytes);
 IF original IS DISTINCT FROM NEW OR NEW.effect_xid IS DISTINCT FROM pg_current_xact_id()
  OR NEW.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR ROW(d.actor_user_id,d.command_id,d.kind,d.primary_target,d.input_digest)
   IS DISTINCT FROM ROW(NEW.actor_user_id,NEW.command_id,NEW.kind,NEW.primary_target,NEW.input_digest)
  OR (SELECT count(*) FROM public.platform_legacy_topology_receipts r
   WHERE r.effect_xid=NEW.effect_xid AND r.effect_backend_pid=NEW.effect_backend_pid)<>1
  OR EXISTS(SELECT 1 FROM public.platform_legacy_membership_effect_bindings b
   WHERE b.actor_user_id=NEW.actor_user_id AND b.command_id=NEW.command_id)
  OR EXISTS(SELECT 1 FROM public.platform_legacy_catalog_effect_bindings b
   WHERE b.actor_user_id=NEW.actor_user_id AND b.command_id=NEW.command_id) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
 END IF;
 SELECT count(*) INTO history_count FROM public.group_membership_revisions h
  WHERE h.legacy_actor_user_id=NEW.actor_user_id AND h.command_id=NEW.command_id;
 IF NEW.outcome='REJECTED' THEN
  IF history_count<>0 OR EXISTS(SELECT 1 FROM public.audit_events a
   WHERE a.actor=NEW.actor_user_id AND a.trace_id=NEW.trace_id AND a.span_id=NEW.span_id
    AND a.occurred_at=NEW.occurred_at AND a.action IN('platform.tenant.create','platform.group.create',
     'platform.group.update','platform.group.assign_org','platform.group.remove_org',
     'platform.group.account.create','platform.group.account.revoke','platform.tenant.remove','platform.tenant.status')) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  RETURN NEW;
 END IF;
 requested_group:=(d.payload->>'group_id')::uuid;
 PERFORM set_config('app.current_org',coalesce(NEW.result_org_id::text,''),true);
 SELECT x.* INTO STRICT g FROM public.groups x WHERE x.id=NEW.result_group_id;
 IF g.origin_account_id IS NOT NULL OR g.origin_command_id IS NOT NULL OR g.origin_receipt_id IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
 END IF;
 IF NEW.kind IN(1,4,5,6,9) THEN
  SELECT x.* INTO STRICT o FROM public.organizations x WHERE x.id=NEW.result_org_id;
  SELECT x.* INTO STRICT m FROM public.group_memberships x WHERE x.org_id=o.id;
  IF o.group_id IS DISTINCT FROM g.id OR m.group_id IS DISTINCT FROM g.id
   OR o.origin_account_id IS NOT NULL OR o.origin_command_id IS NOT NULL OR o.origin_receipt_id IS NOT NULL
   OR NOT EXISTS(SELECT 1 FROM public.group_membership_revisions h
    WHERE h.org_id=m.org_id AND h.group_id=m.group_id AND h.membership_id=m.membership_id
     AND h.revision=m.current_revision AND h.incarnation=m.incarnation AND h.state='ACTIVE' AND h.to_time IS NULL) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
 END IF;
 IF NEW.kind IN(1,6) THEN
  SELECT x.* INTO STRICT u FROM public.users x WHERE x.id=NEW.result_user_id;
  IF u.org_id IS DISTINCT FROM o.id OR u.is_active IS DISTINCT FROM true THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  IF NOT public.auth_legacy_bootstrap_receipt_matches_v1(o.id,u.id,NEW.bootstrap_credential_id,
   NEW.occurred_at,NEW.bootstrap_expires_at) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
 END IF;
 action_name:=(ARRAY['platform.tenant.create','platform.group.create','platform.group.update',
  'platform.group.assign_org','platform.group.remove_org','platform.group.account.create',
  'platform.group.account.revoke','platform.tenant.remove','platform.tenant.status'])[NEW.kind];
 CASE NEW.kind
 WHEN 1 THEN
  IF o.slug IS DISTINCT FROM d.payload->>'slug' OR o.name IS DISTINCT FROM d.payload->>'name'
   OR o.status IS DISTINCT FROM 'ACTIVE' OR u.display_name IS DISTINCT FROM 'Tenant Admin'
   OR u.roles IS DISTINCT FROM ARRAY['SUPER_ADMIN']::text[] OR history_count<>1 THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='organizations';target_id:=o.id::text;audit_org:=o.id;
  expected_after:=jsonb_build_object('org_id',o.id,'slug',o.slug,'admin_user_id',u.id,'group_id',g.id);
  expected_groups:=ARRAY[g.id];
 WHEN 2 THEN
  IF g.slug IS DISTINCT FROM d.payload->>'slug' OR g.name IS DISTINCT FROM d.payload->>'name'
   OR g.status IS DISTINCT FROM 'ACTIVE' OR history_count<>0 THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='groups';target_id:=g.id::text;
  expected_after:=jsonb_build_object('group_id',g.id,'slug',g.slug,'name',g.name);expected_groups:=ARRAY[g.id];
 WHEN 3 THEN
  IF g.id IS DISTINCT FROM requested_group OR history_count<>0
   OR (d.payload->>'slug' IS NOT NULL AND g.slug IS DISTINCT FROM d.payload->>'slug')
   OR (d.payload->>'name' IS NOT NULL AND g.name IS DISTINCT FROM d.payload->>'name')
   OR (d.payload->>'status' IS NOT NULL AND g.status IS DISTINCT FROM d.payload->>'status') THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='groups';target_id:=g.id::text;expected_groups:=ARRAY[g.id];
  expected_after:=jsonb_build_object('group_id',g.id,'slug',d.payload->'slug','name',d.payload->'name','status',d.payload->'status');
 WHEN 4,5 THEN
  IF o.id IS DISTINCT FROM (d.payload->>'org_id')::uuid OR history_count NOT IN(0,2)
   OR (NEW.kind=4 AND g.id IS DISTINCT FROM requested_group) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  no_change:=history_count=0;
  IF NOT no_change THEN
   SELECT h.* INTO STRICT removed FROM public.group_membership_revisions h
    WHERE h.legacy_actor_user_id=NEW.actor_user_id AND h.command_id=NEW.command_id AND h.state='REMOVED';
   IF removed.command_receipt IS DISTINCT FROM NEW.receipt_id OR removed.org_id IS DISTINCT FROM o.id
    OR (NEW.kind=5 AND removed.group_id IS DISTINCT FROM requested_group) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
   END IF;
   expected_groups:=ARRAY[removed.group_id,g.id];
  ELSE
   IF NEW.kind=5 AND requested_group=g.id THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
   END IF;
   expected_groups:=ARRAY[requested_group,g.id];
  END IF;
  target_type:='group_memberships';target_id:=requested_group::text||':'||o.id::text;audit_org:=o.id;
  IF NEW.kind=4 THEN expected_after:=jsonb_build_object('group_id',g.id,'org_id',o.id);
  ELSE
   expected_before:=jsonb_build_object('group_id',requested_group,'org_id',o.id);
   IF NOT no_change AND removed.group_id IS DISTINCT FROM g.id THEN
    expected_after:=jsonb_build_object('group_id',g.id,'org_id',o.id);
   END IF;
  END IF;
 WHEN 6 THEN
  IF o.id IS DISTINCT FROM (d.payload->>'org_id')::uuid OR g.id IS DISTINCT FROM requested_group
   OR u.display_name IS DISTINCT FROM d.payload->>'display_name' OR u.phone IS DISTINCT FROM d.payload->>'phone'
   OR to_jsonb(u.roles) IS DISTINCT FROM d.payload->'tenant_roles' OR history_count<>0
   OR NOT EXISTS(SELECT 1 FROM public.group_role_grants r WHERE r.group_id=g.id AND r.user_id=u.id
    AND r.group_role=d.payload->>'group_role' AND r.granted_by=NEW.actor_user_id) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='group_role_grants';target_id:=g.id::text||':'||u.id::text||':'||(d.payload->>'group_role');audit_org:=o.id;
  expected_after:=jsonb_build_object('group_id',g.id,'org_id',o.id,'user_id',u.id,'tenant_roles',d.payload->'tenant_roles','group_role',d.payload->>'group_role');
  expected_groups:=ARRAY[g.id];
 WHEN 7 THEN
  SELECT x.* INTO STRICT u FROM public.users x WHERE x.id=NEW.result_user_id;
  SELECT x.* INTO STRICT o FROM public.organizations x WHERE x.id=u.org_id;
  SELECT x.* INTO STRICT m FROM public.group_memberships x WHERE x.org_id=o.id;
  IF o.group_id IS DISTINCT FROM m.group_id OR o.origin_account_id IS NOT NULL
   OR NOT EXISTS(SELECT 1 FROM public.group_membership_revisions h
    WHERE h.org_id=m.org_id AND h.group_id=m.group_id AND h.membership_id=m.membership_id
     AND h.revision=m.current_revision AND h.incarnation=m.incarnation AND h.state='ACTIVE' AND h.to_time IS NULL) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  IF u.id IS DISTINCT FROM (d.payload->>'user_id')::uuid OR g.id IS DISTINCT FROM requested_group OR history_count<>0
   OR EXISTS(SELECT 1 FROM public.group_role_grants r WHERE r.group_id=g.id AND r.user_id=u.id AND r.group_role=d.payload->>'group_role') THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='group_role_grants';target_id:=g.id::text||':'||u.id::text||':'||(d.payload->>'group_role');audit_org:=u.org_id;
  expected_before:=jsonb_build_object('group_id',g.id,'user_id',u.id,'group_role',d.payload->>'group_role');expected_groups:=ARRAY[g.id,m.group_id];
 WHEN 8 THEN
  IF NEW.result_org_id IS DISTINCT FROM (d.payload->>'org_id')::uuid OR history_count<>1
   OR EXISTS(SELECT 1 FROM public.organizations x WHERE x.id=NEW.result_org_id)
   OR EXISTS(SELECT 1 FROM public.group_memberships x WHERE x.org_id=NEW.result_org_id)
   OR EXISTS(SELECT 1 FROM public.users x WHERE x.org_id=NEW.result_org_id) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  SELECT h.* INTO STRICT removed FROM public.group_membership_revisions h
   WHERE h.legacy_actor_user_id=NEW.actor_user_id AND h.command_id=NEW.command_id;
  IF removed.org_id IS DISTINCT FROM NEW.result_org_id OR removed.group_id IS DISTINCT FROM g.id
   OR removed.state IS DISTINCT FROM 'REMOVED' OR removed.command_receipt IS DISTINCT FROM NEW.receipt_id
   OR removed.to_time IS DISTINCT FROM NEW.occurred_at THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='organizations';target_id:=NEW.result_org_id::text;
  -- The independent mandatory frame closure checks this complete Group set
  -- against its captured pre-effect grant cohort, not these claimed arrays.
  SELECT array_agg((value->>'group_id')::uuid ORDER BY (value->>'group_id')::uuid)
   INTO expected_groups FROM jsonb_array_elements(NEW.heads_before);
  -- The independently retained Organization DELETE witness checks the original slug.
 WHEN 9 THEN
  IF o.id IS DISTINCT FROM (d.payload->>'org_id')::uuid OR o.status IS DISTINCT FROM d.payload->>'status' OR history_count<>0 THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
  target_type:='organizations';target_id:=o.id::text;audit_org:=o.id;expected_groups:=ARRAY[g.id];
  expected_after:=jsonb_build_object('status',o.status);
 END CASE;
 SELECT array_agg(DISTINCT id ORDER BY id) INTO expected_groups FROM unnest(expected_groups) t(id);
 SELECT coalesce(array_agg((x->>'group_id')::uuid ORDER BY x->>'group_id'),ARRAY[]::uuid[])
  INTO after_groups FROM jsonb_array_elements(NEW.heads_after) t(x);
 SELECT coalesce(array_agg((x->>'group_id')::uuid ORDER BY x->>'group_id'),ARRAY[]::uuid[])
  INTO before_groups FROM jsonb_array_elements(NEW.heads_before) t(x);
 IF after_groups IS DISTINCT FROM expected_groups OR EXISTS(SELECT 1 FROM unnest(before_groups) t(id) WHERE NOT(id=ANY(expected_groups))) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
 END IF;
 SELECT jsonb_agg(jsonb_build_object('group_id',h.group_id::text,'incarnation',h.incarnation::text,'revision',h.revision::text) ORDER BY h.group_id)
  INTO actual_heads FROM public.group_authority_heads h WHERE h.group_id=ANY(expected_groups);
 IF actual_heads IS DISTINCT FROM NEW.heads_after THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
 END IF;
 FOR after_head IN SELECT value FROM jsonb_array_elements(NEW.heads_after) LOOP
  SELECT value INTO before_head FROM jsonb_array_elements(NEW.heads_before) WHERE value->>'group_id'=after_head->>'group_id';
  IF NOT FOUND THEN
   IF NEW.kind NOT IN(1,2,5) OR after_head->>'group_id'<>g.id::text OR after_head->>'revision'<>'1' OR no_change THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
   END IF;
   new_group:=true;
  ELSIF before_head IS DISTINCT FROM jsonb_build_object('group_id',after_head->>'group_id','incarnation',after_head->>'incarnation',
    'revision',((after_head->>'revision')::bigint-CASE WHEN no_change THEN 0 ELSE 1 END)::text) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
  END IF;
 END LOOP;
 IF NEW.kind=2 AND NOT new_group THEN RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid'; END IF;
 SELECT coalesce(jsonb_agg(value ORDER BY value->>'group_id'),'[]'::jsonb) INTO expected_client_heads
  FROM jsonb_array_elements(NEW.heads_before)
  WHERE NEW.kind NOT IN(1,2) AND (NEW.kind<>5 OR value->>'group_id'=requested_group::text)
   AND (NEW.kind<>8 OR value->>'group_id'=g.id::text);
 IF d.expected_groups IS DISTINCT FROM expected_client_heads THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
 END IF;
 PERFORM set_config('app.current_org',coalesce(audit_org::text,''),true);
 SELECT a.* INTO STRICT audit FROM public.audit_events a WHERE a.actor=NEW.actor_user_id
  AND a.trace_id=NEW.trace_id AND a.span_id=NEW.span_id AND a.occurred_at=NEW.occurred_at AND a.action=action_name;
 IF ROW(audit.target_type,audit.target_id,audit.org_id,audit.before_snap)
   IS DISTINCT FROM ROW(target_type,target_id,audit_org,expected_before)
  OR (NEW.kind<>8 AND audit.after_snap IS DISTINCT FROM expected_after)
  OR (NEW.kind=8 AND ((SELECT array_agg(key ORDER BY key) FROM jsonb_object_keys(audit.after_snap) t(key))
    IS DISTINCT FROM ARRAY['org_id','slug']::text[] OR audit.after_snap->>'org_id'<>NEW.result_org_id::text)) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RETURN NEW;
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.receipt_closure_invalid';
WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_receipt_closed_v1() OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_receipt_closed_v1() FROM PUBLIC;
CREATE CONSTRAINT TRIGGER trg_platform_legacy_receipt_closed AFTER INSERT ON public.platform_legacy_topology_receipts
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.platform_legacy_receipt_closed_v1();


-- source: legacy-coordinator-source-round2.sql
-- Private finite coordinator candidate; not a standalone installation resource.
-- Rust retains signed Auth, current Cedar and final clock checks around this call.
CREATE FUNCTION public.platform_legacy_topology_command_v1(
 p_actor uuid,p_family uuid,p_command uuid,p_primary uuid,p_kind text,p_input bytea,
 p_trace text,p_span text,p_material jsonb)
RETURNS TABLE(receipt_id uuid,outcome text,result_code text,result_org_id uuid,
 result_group_id uuid,result_user_id uuid,replayed boolean,
 bootstrap_credential_id uuid,bootstrap_expires_at timestamptz)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE policy_source record;d record; r public.platform_legacy_topology_receipts%ROWTYPE;
 plan jsonb; plan_material jsonb; boot jsonb; catalog jsonb; actual_digest bytea;
 prospective_org uuid; prospective_group uuid; command_lock bigint;
 target_org uuid; requested_group uuid; old_group uuid; target_user uuid;
 selected_group uuid; group_is_new boolean:=false; no_change boolean:=false;
 o public.organizations%ROWTYPE; m public.group_memberships%ROWTYPE;
 old_history public.group_membership_revisions%ROWTYPE;
 required_heads uuid[]; affected_heads uuid[]; client_heads jsonb;
 before_heads jsonb; frame_heads jsonb; after_heads jsonb; item jsonb;
 event_time timestamptz:=statement_timestamp(); result_id uuid; code text;
 refusal text; constraint_name text; catalog_result record;
 audit_action text; audit_target_type text; audit_target text; audit_org uuid;
 audit_before jsonb; audit_after jsonb; new_receipt uuid:=gen_random_uuid();
 issued_id uuid; issued_expiry timestamptz; new_user uuid;
 prior_org text:=current_setting('app.current_org',true);
 home constant uuid:='00000000-0000-0000-0000-00000000face';
 nil constant uuid:='00000000-0000-0000-0000-000000000000';
BEGIN
 SELECT * INTO STRICT d FROM public.platform_legacy_topology_decode_input_v1(p_input);
 IF ROW(d.actor_user_id,d.command_id,d.primary_target)
   IS DISTINCT FROM ROW(p_actor,p_command,p_primary)
  OR p_kind IS DISTINCT FROM (ARRAY['CREATE_COMPANY','CREATE_GROUP','UPDATE_GROUP','ASSIGN_COMPANY',
   'REMOVE_COMPANY_FROM_GROUP','CREATE_LEGACY_GROUP_ACCOUNT','REVOKE_LEGACY_GROUP_ROLE',
   'REMOVE_EMPTY_COMPANY','SET_COMPANY_STATUS'])[d.kind]
  OR p_family IS NULL OR p_family=nil
  OR p_trace IS NULL OR p_trace!~'^[0-9a-f]{32}$' OR p_trace=repeat('0',32)
  OR p_span IS NULL OR p_span!~'^[0-9a-f]{16}$' OR p_span=repeat('0',16)
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR jsonb_typeof(p_material) IS DISTINCT FROM 'object'
  OR (SELECT array_agg(key ORDER BY key) FROM jsonb_object_keys(p_material) k(key))
    IS DISTINCT FROM ARRAY['bootstrap','catalog','plan']::text[] THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_command';
 END IF;
 plan_material:=p_material->'plan';
 IF jsonb_typeof(plan_material) IS DISTINCT FROM 'object'
  OR (SELECT array_agg(key ORDER BY key) FROM jsonb_object_keys(plan_material) k(key))
   IS DISTINCT FROM ARRAY['digest_hex','prospective_company_id','prospective_group_id']::text[]
  OR jsonb_typeof(plan_material->'digest_hex') IS DISTINCT FROM 'string'
  OR plan_material->>'digest_hex'!~'^[0-9a-f]{64}$'
  OR jsonb_typeof(plan_material->'prospective_company_id') IS DISTINCT FROM (CASE WHEN d.kind=1 THEN 'string' ELSE 'null' END)
  OR jsonb_typeof(plan_material->'prospective_group_id') IS DISTINCT FROM (CASE WHEN d.kind IN(1,5) THEN 'string' ELSE 'null' END) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_plan_input';
 END IF;
 FOR item IN SELECT value FROM jsonb_each(plan_material) p WHERE key IN('prospective_company_id','prospective_group_id') LOOP
  IF item<>'null'::jsonb AND (item#>>'{}'!~'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
   OR item#>>'{}'=nil::text) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_plan_input';
  END IF;
 END LOOP;
 prospective_org:=(plan_material->>'prospective_company_id')::uuid;
 prospective_group:=(plan_material->>'prospective_group_id')::uuid;
 -- Reacquires only the exact retained Group-first plan, then current SQL source.
 actual_digest:=public.platform_legacy_topology_lock_plan_v1(p_actor,p_family,p_input,
  prospective_org,prospective_group,decode(plan_material->>'digest_hex','hex'));
 -- The Rust caller evaluates current Cedar before invoking this coordinator.
 -- Retain the finite SQL role prerequisite under those same acquired locks.
 SELECT * INTO STRICT policy_source FROM public.auth_legacy_platform_source_material_v1(p_actor,p_family);
 IF ('SUPER_ADMIN'=ANY(policy_source.roles)) IS DISTINCT FROM true THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_topology.policy_denied';
 END IF;
 command_lock:=('x'||substr(encode(sha256(convert_to('console.platform.topology.command/1','UTF8')
  ||uuid_send(p_actor)||uuid_send(p_command)),'hex'),1,16))::bit(64)::bigint;
 PERFORM pg_advisory_xact_lock(command_lock);
 -- This is a FRESH READ COMMITTED lookup after serialization, not the planner's
 -- earlier absence. A winner's receipt outranks stale target/head conditions.
 SELECT x.* INTO r FROM public.platform_legacy_topology_receipts x
  WHERE x.actor_user_id=p_actor AND x.command_id=p_command;
 IF r.receipt_id IS NOT NULL THEN
  IF ROW(r.codec_version,r.kind,r.primary_target,r.input_digest,r.input_bytes)
    IS DISTINCT FROM ROW(1::smallint,d.kind,p_primary,d.input_digest,p_input) THEN
   RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='platform_topology.command_input_conflict';
  END IF;
  RETURN QUERY SELECT r.receipt_id,r.outcome,r.result_code,r.result_org_id,r.result_group_id,
   r.result_user_id,true,r.bootstrap_credential_id,r.bootstrap_expires_at;
  RETURN;
 END IF;
 IF EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts x
  WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid())
  OR EXISTS(SELECT 1 FROM public.platform_force_removal_effect_bindings x
   WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid())
  OR EXISTS(SELECT 1 FROM public.platform_force_removal_receipts x
   WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid()) THEN
  RAISE EXCEPTION USING ERRCODE='25000',MESSAGE='platform_topology.one_command_per_transaction';
 END IF;
 plan:=public.platform_legacy_topology_plan_v1(p_actor,p_family,p_input,prospective_org,prospective_group);
 IF sha256(convert_to(plan::text,'UTF8')) IS DISTINCT FROM actual_digest THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 END IF;
 -- Trusted entry prefix: unconditional retained witness precedes all tentative
 -- effects. Participant fault injection begins AFTER this insertion; the SQL
 -- installation owner and this prefix are explicit trusted assumptions.
 INSERT INTO public.platform_legacy_topology_effect_bindings(actor_user_id,command_id,receipt_id,
  effect_family_id,codec_version,kind,primary_target,input_bytes,input_digest,effect_xid,
  effect_backend_pid,trace_id,span_id,occurred_at,plan_snapshot)
 VALUES(p_actor,p_command,new_receipt,p_family,1,d.kind,p_primary,p_input,d.input_digest,
  pg_current_xact_id(),pg_backend_pid(),p_trace,p_span,event_time,plan);
 target_org:=CASE WHEN d.kind=1 THEN prospective_org ELSE (d.payload->>'org_id')::uuid END;
 requested_group:=(d.payload->>'group_id')::uuid;
 target_user:=(d.payload->>'user_id')::uuid;
 IF d.kind=7 THEN SELECT x.org_id INTO target_org FROM public.users x WHERE x.id=target_user; END IF;
 IF target_org IS NOT NULL THEN
  PERFORM set_config('app.current_org',target_org::text,true);
  SELECT x.* INTO o FROM public.organizations x WHERE x.id=target_org;
  SELECT x.* INTO m FROM public.group_memberships x WHERE x.org_id=target_org;
  old_group:=m.group_id;
 END IF;
 -- Only named, canonical domain predicates produce durable rejection.
 IF d.kind=1 AND o.id IS NOT NULL THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_topology.plan_changed';
 ELSIF d.kind IN(4,5,6,7,8,9) AND (o.id IS NULL OR target_org=home) THEN refusal:='not_found';
 ELSIF d.kind IN(3,4,5,6,7) AND NOT EXISTS(SELECT 1 FROM public.groups g WHERE g.id=requested_group) THEN refusal:='not_found';
 END IF;
 IF refusal IS NULL AND d.kind IN(4,5,6,7,8,9) AND
  (m.org_id IS NULL OR o.group_id IS DISTINCT FROM m.group_id OR NOT EXISTS(
   SELECT 1 FROM public.group_membership_revisions h WHERE h.org_id=m.org_id AND h.group_id=m.group_id
    AND h.membership_id=m.membership_id AND h.revision=m.current_revision AND h.incarnation=m.incarnation
    AND h.state='ACTIVE' AND h.to_time IS NULL)) THEN
  RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_topology.membership_corrupt';
 END IF;
 IF refusal IS NULL AND (o.origin_account_id IS NOT NULL OR EXISTS(
  SELECT 1 FROM jsonb_array_elements(plan->'groups') p JOIN public.groups g ON g.id=(p.value->>'id')::uuid
  WHERE g.origin_account_id IS NOT NULL OR EXISTS(SELECT 1 FROM public.organizations native
   WHERE native.group_id=g.id AND native.origin_account_id IS NOT NULL))) THEN refusal:='native_origin_denied'; END IF;
 SELECT coalesce(array_agg(DISTINCT id ORDER BY id),ARRAY[]::uuid[]) INTO required_heads
 FROM unnest(CASE d.kind WHEN 1 THEN ARRAY[]::uuid[] WHEN 2 THEN ARRAY[]::uuid[]
  WHEN 3 THEN ARRAY[requested_group] WHEN 4 THEN ARRAY[old_group,requested_group]
  WHEN 5 THEN ARRAY[requested_group] WHEN 6 THEN ARRAY[requested_group]
  WHEN 7 THEN ARRAY[requested_group,old_group] ELSE ARRAY[old_group] END) x(id) WHERE id IS NOT NULL;
 SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',h.group_id::text,'incarnation',h.incarnation::text,
  'revision',h.revision::text) ORDER BY h.group_id),'[]'::jsonb) INTO client_heads
 FROM public.group_authority_heads h WHERE h.group_id=ANY(required_heads);
 IF refusal IS NULL AND (client_heads IS DISTINCT FROM d.expected_groups
  OR jsonb_array_length(client_heads)<>cardinality(required_heads)) THEN refusal:='revision_conflict'; END IF;
 IF refusal IS NULL AND d.kind IN(4,6) AND EXISTS(
  SELECT 1 FROM public.groups g WHERE g.id=requested_group AND g.status<>'ACTIVE') THEN refusal:='group_inactive'; END IF;
 IF refusal IS NULL AND d.kind=6 AND o.status<>'ACTIVE' THEN refusal:='company_inactive'; END IF;
 IF refusal IS NULL AND d.kind=6 AND old_group IS DISTINCT FROM requested_group THEN refusal:='invalid_membership'; END IF;
 IF refusal IS NULL AND d.kind=7 AND NOT EXISTS(SELECT 1 FROM public.group_role_grants g
  WHERE g.group_id=requested_group AND g.user_id=target_user AND g.group_role=d.payload->>'group_role') THEN refusal:='not_found'; END IF;
 IF refusal IS NULL AND d.kind=8 AND plan->'grantor_blocked'='true'::jsonb THEN refusal:='blocked_has_data'; END IF;
 -- Ordinary removal cannot discard a Company's governed catalog or its creator.
 -- The retained Company FOR UPDATE lock precedes this current scoped read.
 IF refusal IS NULL AND d.kind=8 AND (EXISTS(
  SELECT 1 FROM public.ont_object_types catalog_object WHERE catalog_object.org_id=target_org
 ) OR EXISTS(
  SELECT 1 FROM public.ont_builtin_catalog_installs catalog_install WHERE catalog_install.org_id=target_org
 )) THEN refusal:='blocked_has_data'; END IF;
 IF refusal IS NULL AND d.kind IN(1,5) AND plan->'mint'->'exhausted'='true'::jsonb THEN refusal:='slug_conflict'; END IF;
 IF d.kind=4 THEN selected_group:=requested_group; no_change:=old_group=requested_group;
 ELSIF d.kind=5 AND old_group IS DISTINCT FROM requested_group THEN selected_group:=old_group; no_change:=true;
 ELSIF d.kind IN(1,5) THEN
  selected_group:=(plan->'mint'->>'group_id')::uuid;
  group_is_new:=(plan->'mint'->>'is_new')::boolean;
 ELSE selected_group:=coalesce(requested_group,old_group); END IF;
 SELECT coalesce(array_agg(DISTINCT id ORDER BY id),ARRAY[]::uuid[]) INTO affected_heads
 FROM unnest(CASE WHEN d.kind IN(1,4,5) THEN ARRAY[old_group,selected_group,
  CASE WHEN d.kind=5 AND no_change THEN requested_group END]
  WHEN d.kind=7 THEN ARRAY[requested_group,old_group]
  WHEN d.kind=8 THEN ARRAY(SELECT (value->>'id')::uuid FROM jsonb_array_elements(plan->'groups'))
  ELSE ARRAY[selected_group] END) x(id)
 WHERE id IS NOT NULL;
 SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',h.group_id::text,'incarnation',h.incarnation::text,
  'revision',h.revision::text) ORDER BY h.group_id),'[]'::jsonb) INTO before_heads
 FROM public.group_authority_heads h WHERE h.group_id=ANY(affected_heads);
 IF refusal IS NULL THEN
  boot:=p_material->'bootstrap'; catalog:=p_material->'catalog';
  IF jsonb_typeof(boot) IS DISTINCT FROM (CASE WHEN d.kind IN(1,6) THEN 'object' ELSE 'null' END)
   OR jsonb_typeof(catalog) IS DISTINCT FROM (CASE WHEN d.kind=1 THEN 'object' ELSE 'null' END) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_effect_material';
  END IF;
  IF d.kind IN(1,6) THEN
   IF (SELECT array_agg(key ORDER BY key) FROM jsonb_object_keys(boot) k(key))
      IS DISTINCT FROM ARRAY['credential_id','expires_at','token_hash_hex']::text[]
    OR jsonb_typeof(boot->'credential_id') IS DISTINCT FROM 'string'
    OR boot->>'credential_id'!~'^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
    OR boot->>'credential_id'=nil::text OR jsonb_typeof(boot->'token_hash_hex') IS DISTINCT FROM 'string'
    OR boot->>'token_hash_hex'!~'^[0-9a-f]{64}$' OR jsonb_typeof(boot->'expires_at') IS DISTINCT FROM 'string'
    OR boot->>'expires_at'!~'^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{6}Z$' THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_effect_material';
   END IF;
   issued_id:=(boot->>'credential_id')::uuid; issued_expiry:=(boot->>'expires_at')::timestamptz;
   IF NOT isfinite(issued_expiry) OR issued_expiry<=clock_timestamp()
    OR to_char(issued_expiry AT TIME ZONE 'UTC','YYYY-MM-DD"T"HH24:MI:SS.US"Z"') IS DISTINCT FROM boot->>'expires_at' THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_effect_material';
   END IF;
  END IF;
  IF d.kind=1 AND ((SELECT array_agg(key ORDER BY key) FROM jsonb_object_keys(catalog) k(key))
    IS DISTINCT FROM ARRAY['manifest','version']::text[] OR catalog->>'version' IS DISTINCT FROM '2026-07-19.1'
    OR jsonb_typeof(catalog->'manifest') IS DISTINCT FROM 'object'
    OR sha256(convert_to((catalog->'manifest')::text,'UTF8')) IS DISTINCT FROM
      decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex')) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_topology.invalid_effect_material';
  END IF;
  -- This exception block is the tentative-effect savepoint. A known refusal
  -- rolls every participant back before the only durable REJECTED insertion.
  BEGIN
   IF d.kind IN(1,4) OR (d.kind=5 AND NOT no_change) THEN
    SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',h.group_id::text,'incarnation',h.incarnation::text,
     'revision',h.revision::text,'state',h.state,'group_status',g.status) ORDER BY h.group_id),'[]'::jsonb)
     INTO frame_heads FROM public.group_authority_heads h JOIN public.groups g ON g.id=h.group_id
     WHERE h.group_id=ANY(affected_heads);
    INSERT INTO public.platform_legacy_membership_effect_bindings(actor_user_id,command_id,receipt_id,
     effect_family_id,effect_xid,effect_backend_pid,kind,input_bytes,input_digest,org_id,company_status,
     old_group_id,old_membership_id,old_membership_revision,old_membership_incarnation,group_id,mint_suffix,
     group_is_new,membership_id,membership_incarnation,occurred_at,heads_before)
    VALUES(p_actor,p_command,new_receipt,p_family,pg_current_xact_id(),pg_backend_pid(),d.kind,p_input,d.input_digest,
     target_org,CASE WHEN d.kind=1 THEN 'ACTIVE' ELSE o.status END,old_group,m.membership_id,m.current_revision,m.incarnation,
     selected_group,(plan->'mint'->>'suffix')::smallint,group_is_new,
     CASE WHEN no_change THEN m.membership_id ELSE gen_random_uuid() END,
     CASE WHEN no_change THEN m.incarnation ELSE gen_random_uuid() END,event_time,frame_heads);
   END IF;
   CASE d.kind
   WHEN 1 THEN
    result_id:=public.platform_create_organization_core_v1(target_org,NULL,d.payload->>'slug',d.payload->>'name');
    INSERT INTO public.users(display_name,roles,is_active,org_id)
     VALUES('Tenant Admin',ARRAY['SUPER_ADMIN']::text[],true,target_org) RETURNING id INTO new_user;
    audit_action:='platform.tenant.create'; audit_target_type:='organizations'; audit_target:=target_org::text; audit_org:=target_org;
    audit_after:=jsonb_build_object('org_id',target_org,'slug',d.payload->>'slug','admin_user_id',new_user,'group_id',selected_group);
   WHEN 2 THEN
    selected_group:=public.platform_create_group(d.payload->>'slug',d.payload->>'name');
    affected_heads:=ARRAY[selected_group]; group_is_new:=true;
    audit_action:='platform.group.create'; audit_target_type:='groups'; audit_target:=selected_group::text;
    audit_after:=jsonb_build_object('group_id',selected_group,'slug',d.payload->>'slug','name',d.payload->>'name');
   WHEN 3 THEN
    result_id:=public.platform_update_group(requested_group,d.payload->>'slug',d.payload->>'name',d.payload->>'status');
    audit_action:='platform.group.update'; audit_target_type:='groups'; audit_target:=requested_group::text;
    audit_after:=jsonb_build_object('group_id',requested_group,'slug',d.payload->'slug','name',d.payload->'name','status',d.payload->'status');
   WHEN 4 THEN
    result_id:=public.platform_assign_org_to_group(requested_group,target_org);
    audit_action:='platform.group.assign_org'; audit_target_type:='group_memberships';
    audit_target:=requested_group::text||':'||target_org::text; audit_org:=target_org;
    audit_after:=jsonb_build_object('group_id',requested_group,'org_id',target_org);
   WHEN 5 THEN
    result_id:=public.platform_remove_org_from_group(requested_group,target_org);
    audit_action:='platform.group.remove_org'; audit_target_type:='group_memberships';
    audit_target:=requested_group::text||':'||target_org::text; audit_org:=target_org;
    audit_before:=jsonb_build_object('group_id',requested_group,'org_id',target_org);
    IF selected_group IS DISTINCT FROM old_group THEN audit_after:=jsonb_build_object('group_id',selected_group,'org_id',target_org); END IF;
   WHEN 6 THEN
    new_user:=public.platform_create_group_account(requested_group,target_org,d.payload->>'display_name',d.payload->>'phone',
     ARRAY(SELECT jsonb_array_elements_text(d.payload->'tenant_roles')),d.payload->>'group_role',p_actor);
    result_id:=new_user;
    audit_action:='platform.group.account.create'; audit_target_type:='group_role_grants'; audit_org:=target_org;
    audit_target:=requested_group::text||':'||new_user::text||':'||(d.payload->>'group_role');
    audit_after:=jsonb_build_object('group_id',requested_group,'org_id',target_org,'user_id',new_user,
     'tenant_roles',d.payload->'tenant_roles','group_role',d.payload->>'group_role');
   WHEN 7 THEN
    result_id:=public.platform_revoke_group_role(requested_group,target_user,d.payload->>'group_role'); new_user:=target_user;
    audit_action:='platform.group.account.revoke'; audit_target_type:='group_role_grants'; audit_org:=target_org;
    audit_target:=requested_group::text||':'||target_user::text||':'||(d.payload->>'group_role');
    audit_before:=jsonb_build_object('group_id',requested_group,'user_id',target_user,'group_role',d.payload->>'group_role');
   WHEN 8 THEN
    SELECT h.* INTO STRICT old_history FROM public.group_membership_revisions h
     WHERE h.membership_id=m.membership_id AND h.revision=m.current_revision;
    code:=public.platform_remove_organization(target_org);
    IF code IN('not_found','blocked_has_data') THEN
     RAISE EXCEPTION USING ERRCODE='P1001',MESSAGE=code;
    ELSIF code IS DISTINCT FROM 'removed' THEN RAISE EXCEPTION 'platform_topology.invalid_remove_result'; END IF;
    INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
     state,provenance_kind,native_account_id,legacy_actor_user_id,command_id,command_receipt)
    VALUES(old_group,target_org,m.membership_id,m.current_revision+1,m.incarnation,old_history.from_time,event_time,
     'REMOVED','LEGACY_TOPOLOGY_V1',NULL,p_actor,p_command,new_receipt);
    audit_action:='platform.tenant.remove'; audit_target_type:='organizations'; audit_target:=target_org::text;
    audit_after:=jsonb_build_object('org_id',target_org,'slug',o.slug);
   WHEN 9 THEN
    result_id:=public.platform_set_organization_status(target_org,d.payload->>'status');
    audit_action:='platform.tenant.status'; audit_target_type:='organizations'; audit_target:=target_org::text; audit_org:=target_org;
    audit_after:=jsonb_build_object('status',d.payload->>'status');
   END CASE;
   IF d.kind IN(1,3,4,5,6,7,9) AND result_id IS NULL THEN
    RAISE EXCEPTION 'platform_topology.owner_effect_missing';
   END IF;
   IF d.kind IN(1,6) THEN
    PERFORM set_config('app.current_org',target_org::text,true);
    code:=public.auth_legacy_bootstrap_issue_v1(target_org,new_user,issued_id,decode(boot->>'token_hash_hex','hex'),event_time,issued_expiry);
    IF code IN('has_passkey','open_exists') THEN RAISE EXCEPTION USING ERRCODE='P1001',MESSAGE='credential_state_conflict';
    ELSIF code IS DISTINCT FROM 'issued' THEN RAISE EXCEPTION 'platform_topology.invalid_bootstrap_result'; END IF;
   END IF;
   IF d.kind=1 THEN
    INSERT INTO public.platform_legacy_catalog_effect_bindings(effect_xid,effect_backend_pid,actor_user_id,
     command_id,input_digest,effect_family_id,receipt_id,org_id,admin_user_id,catalog_version,manifest_digest,trace_id,span_id,catalog_at)
    VALUES(pg_current_xact_id(),pg_backend_pid(),p_actor,p_command,d.input_digest,p_family,new_receipt,target_org,new_user,
     catalog->>'version',sha256(convert_to((catalog->'manifest')::text,'UTF8')),p_trace,p_span,event_time);
    SELECT * INTO STRICT catalog_result FROM ontology_api.install_builtin_catalog(target_org,catalog->>'version',catalog->'manifest',new_user,p_trace,p_span);
    IF catalog_result.installed IS DISTINCT FROM true OR catalog_result.object_type_count IS DISTINCT FROM 27::bigint THEN
     RAISE EXCEPTION 'platform_topology.catalog_effect_missing';
    END IF;
   END IF;
   IF group_is_new THEN
    INSERT INTO public.group_authority_heads(group_id,revision,incarnation,state)
     VALUES(selected_group,1,gen_random_uuid(),'ACTIVE');
   END IF;
   IF NOT no_change THEN
    UPDATE public.group_authority_heads SET revision=revision+1
     WHERE group_id=ANY(affected_heads) AND NOT(group_is_new AND group_id=selected_group);
   END IF;
   SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',h.group_id::text,'incarnation',h.incarnation::text,
    'revision',h.revision::text) ORDER BY h.group_id),'[]'::jsonb) INTO after_heads
    FROM public.group_authority_heads h WHERE h.group_id=ANY(affected_heads);
   PERFORM set_config('app.current_org',coalesce(audit_org::text,''),true);
   INSERT INTO public.audit_events(id,actor,action,target_type,target_id,before_snap,after_snap,trace_id,span_id,occurred_at,org_id)
    VALUES(gen_random_uuid(),p_actor,audit_action,audit_target_type,audit_target,audit_before,audit_after,p_trace,p_span,event_time,audit_org);
  EXCEPTION WHEN SQLSTATE 'P1001' THEN
   GET STACKED DIAGNOSTICS refusal=MESSAGE_TEXT;
   IF refusal NOT IN('not_found','blocked_has_data','credential_state_conflict') THEN RAISE; END IF;
  WHEN unique_violation THEN
   GET STACKED DIAGNOSTICS constraint_name=CONSTRAINT_NAME;
   IF (d.kind=1 AND constraint_name='organizations_slug_key') OR (d.kind IN(2,3) AND constraint_name='groups_slug_key') THEN
    refusal:='slug_conflict';
   ELSE RAISE; END IF;
  END;
 END IF;
 INSERT INTO public.platform_legacy_topology_receipts(actor_user_id,command_id,receipt_id,codec_version,kind,input_bytes,
  input_digest,primary_target,outcome,result_code,result_org_id,result_group_id,result_user_id,effect_family_id,
  occurred_at,effect_xid,effect_backend_pid,trace_id,span_id,bootstrap_credential_id,bootstrap_expires_at,
  builtin_catalog_version,builtin_catalog_digest,heads_before,heads_after)
 VALUES(p_actor,p_command,new_receipt,1,d.kind,p_input,d.input_digest,p_primary,
  CASE WHEN refusal IS NULL THEN 'APPLIED' ELSE 'REJECTED' END,coalesce(refusal,'applied'),
  CASE WHEN refusal IS NULL AND d.kind IN(1,4,5,6,8,9) THEN target_org END,
  CASE WHEN refusal IS NULL THEN selected_group END,CASE WHEN refusal IS NULL AND d.kind IN(1,6,7) THEN new_user END,
  p_family,event_time,pg_current_xact_id(),pg_backend_pid(),p_trace,p_span,
  CASE WHEN refusal IS NULL THEN issued_id END,CASE WHEN refusal IS NULL THEN issued_expiry END,
  CASE WHEN refusal IS NULL AND d.kind=1 THEN '2026-07-19.1' END,
  CASE WHEN refusal IS NULL AND d.kind=1 THEN decode('e2b5fdff9a03d4d798344cac2496acab412ffc21e2be84c03e7345a328123247','hex') END,
  before_heads,CASE WHEN refusal IS NULL THEN after_heads ELSE '[]'::jsonb END)
 RETURNING * INTO r;
 IF refusal IS NULL THEN
  DELETE FROM public.platform_legacy_membership_effect_bindings b WHERE b.actor_user_id=p_actor AND b.command_id=p_command;
  DELETE FROM public.platform_legacy_catalog_effect_bindings b WHERE b.actor_user_id=p_actor AND b.command_id=p_command;
 END IF;
 DELETE FROM public.platform_legacy_topology_effect_bindings b
  WHERE b.actor_user_id=p_actor AND b.command_id=p_command;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN QUERY SELECT r.receipt_id,r.outcome,r.result_code,r.result_org_id,r.result_group_id,r.result_user_id,
  false,r.bootstrap_credential_id,r.bootstrap_expires_at;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;
ALTER FUNCTION public.platform_legacy_topology_command_v1(uuid,uuid,uuid,uuid,text,bytea,text,text,jsonb) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_legacy_topology_command_v1(uuid,uuid,uuid,uuid,text,bytea,text,text,jsonb)
 FROM PUBLIC,console_rt,console_auth_rt,console_auth_startup,console_leave_cmd,console_ontology_cmd,
 console_platform_force_cmd,console_account_owner,console_terms_owner,console_credential_owner,console_ontology_writer;
GRANT EXECUTE ON FUNCTION public.platform_legacy_topology_command_v1(uuid,uuid,uuid,uuid,text,bytea,text,text,jsonb) TO console_rt;


-- source: legacy-force-owner-source-successor.sql:command
CREATE FUNCTION public.platform_force_remove_command_v1(
 p_actor uuid,p_family uuid,p_input bytea,p_plan_digest bytea,p_trace text,p_span text)
RETURNS TABLE(receipt_id uuid,result_code text,replayed boolean)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE policy_source record;d record;r public.platform_force_removal_receipts%ROWTYPE;plan jsonb;
 o public.organizations%ROWTYPE;h public.group_membership_revisions%ROWTYPE;
 result_receipt uuid:=gen_random_uuid();event_time timestamptz;refusal text;outcome_code text;
 before_heads jsonb;after_heads jsonb:='[]'::jsonb;changed_groups uuid[];changed integer;
 prior_org text:=current_setting('app.current_org',true);
BEGIN
 SELECT * INTO STRICT d FROM public.platform_force_remove_decode_input_v1(p_input);
 IF p_actor IS DISTINCT FROM d.actor_user_id OR p_family IS NULL
  OR p_family='00000000-0000-0000-0000-000000000000'::uuid
  OR p_plan_digest IS NULL OR octet_length(p_plan_digest)<>32
  OR p_trace IS NULL OR p_trace!~'^[0-9a-f]{32}$' OR p_trace=repeat('0',32)
  OR p_span IS NULL OR p_span!~'^[0-9a-f]{16}$' OR p_span=repeat('0',16) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='platform_force.invalid_command';
 END IF;
 PERFORM public.platform_force_remove_lock_plan_v1(p_actor,p_family,p_input,p_plan_digest);
 -- The Rust caller evaluates current Cedar before invoking this coordinator.
 -- Retain the finite SQL role prerequisite under those same acquired locks.
 SELECT * INTO STRICT policy_source FROM public.auth_legacy_platform_source_material_v1(p_actor,p_family);
 IF ('SUPER_ADMIN'=ANY(policy_source.roles)) IS DISTINCT FROM true THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='platform_force.policy_denied';
 END IF;
 PERFORM pg_advisory_xact_lock(('x'||substr(encode(sha256(convert_to('console.platform.force_remove.command/1','UTF8')
  ||uuid_send(p_actor)||uuid_send(d.command_id)),'hex'),1,16))::bit(64)::bigint);
 SELECT x.* INTO r FROM public.platform_force_removal_receipts x
  WHERE x.actor_user_id=p_actor AND x.command_id=d.command_id;
 IF FOUND THEN
  IF r.input_bytes IS DISTINCT FROM p_input OR r.input_digest IS DISTINCT FROM d.input_digest THEN
   RAISE EXCEPTION USING ERRCODE='P0001',MESSAGE='platform_force.command_input_conflict';
  END IF;
  RETURN QUERY SELECT r.receipt_id,r.result_code,true;RETURN;
 END IF;
 IF EXISTS(SELECT 1 FROM public.platform_force_removal_receipts x
   WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid())
  OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_receipts x
   WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid())
  OR EXISTS(SELECT 1 FROM public.platform_legacy_topology_effect_bindings x
   WHERE x.effect_xid=pg_current_xact_id() AND x.effect_backend_pid=pg_backend_pid()) THEN
  RAISE EXCEPTION USING ERRCODE='25000',MESSAGE='platform_force.one_command_per_transaction';
 END IF;
 plan:=public.platform_force_remove_plan_v1(p_actor,p_family,p_input);
 IF sha256(convert_to(plan::text,'UTF8')) IS DISTINCT FROM p_plan_digest THEN
  RAISE EXCEPTION USING ERRCODE='40001',MESSAGE='platform_force.plan_changed';
 END IF;
 event_time:=clock_timestamp();
 -- Trusted mandatory frame prefix, same explicit assumption as ordinary owner.
 INSERT INTO public.platform_force_removal_effect_bindings(actor_user_id,command_id,receipt_id,input_bytes,
  input_digest,org_id,group_id,family_id,effect_xid,effect_backend_pid,occurred_at,trace_id,span_id,plan_snapshot)
 VALUES(p_actor,d.command_id,result_receipt,p_input,d.input_digest,d.org_id,d.group_id,p_family,
  pg_current_xact_id(),pg_backend_pid(),event_time,p_trace,p_span,plan);
 PERFORM set_config('app.current_org',d.org_id::text,true);
 SELECT x.* INTO o FROM public.organizations x WHERE x.id=d.org_id;
 SELECT array_agg((v->>'id')::uuid ORDER BY (v->>'id')::uuid) INTO changed_groups
  FROM jsonb_array_elements(plan->'groups') t(v);
 SELECT coalesce(jsonb_agg(jsonb_build_object('group_id',g.group_id::text,'incarnation',g.incarnation::text,
  'revision',g.revision::text) ORDER BY g.group_id),'[]'::jsonb) INTO before_heads
  FROM public.group_authority_heads g WHERE g.group_id=ANY(changed_groups);
 IF o.id IS NULL OR o.id='00000000-0000-0000-0000-00000000face'::uuid THEN refusal:='not_found';
 ELSIF o.origin_account_id IS NOT NULL OR EXISTS(SELECT 1 FROM public.groups g
    WHERE g.id=ANY(changed_groups) AND g.origin_account_id IS NOT NULL)
   OR plan->'has_native_rows' IS DISTINCT FROM 'false'::jsonb THEN refusal:='native_origin_denied';
 ELSIF o.group_id IS DISTINCT FROM d.group_id OR NOT EXISTS(SELECT 1 FROM public.group_authority_heads g
   WHERE g.group_id=d.group_id AND g.incarnation=d.expected_incarnation AND g.revision=d.expected_revision) THEN refusal:='revision_conflict';
 ELSIF o.status<>'ARCHIVED' THEN refusal:='blocked_active';
 ELSIF plan->'grantor_blocked' IS DISTINCT FROM 'false'::jsonb THEN refusal:='blocked_has_data';
 END IF;
 IF refusal IS NULL THEN
  SELECT x.* INTO STRICT h FROM public.group_memberships m JOIN public.group_membership_revisions x
   ON x.membership_id=m.membership_id AND x.revision=m.current_revision AND x.incarnation=m.incarnation
   WHERE m.org_id=d.org_id AND m.group_id=d.group_id;
  outcome_code:=public.platform_force_remove_organization(d.org_id);
  IF outcome_code IS DISTINCT FROM 'removed' THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.purge_result_invalid';
  END IF;
  INSERT INTO public.group_membership_revisions(group_id,org_id,membership_id,revision,incarnation,from_time,to_time,
   state,provenance_kind,native_account_id,legacy_actor_user_id,force_actor_user_id,command_id,command_receipt)
  VALUES(d.group_id,d.org_id,h.membership_id,h.revision+1,h.incarnation,h.from_time,event_time,
   'REMOVED','LEGACY_FORCE_REMOVAL_V1',NULL,NULL,p_actor,d.command_id,result_receipt);
  UPDATE public.group_authority_heads g SET revision=g.revision+1 WHERE g.group_id=ANY(changed_groups);
  GET DIAGNOSTICS changed=ROW_COUNT;
  IF changed IS DISTINCT FROM cardinality(changed_groups) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='platform_force.head_result_invalid';
  END IF;
  SELECT jsonb_agg(jsonb_build_object('group_id',g.group_id::text,'incarnation',g.incarnation::text,
   'revision',g.revision::text) ORDER BY g.group_id) INTO after_heads
   FROM public.group_authority_heads g WHERE g.group_id=ANY(changed_groups);
  INSERT INTO public.audit_events(org_id,actor,action,target_type,target_id,branch_id,before_snap,after_snap,
   trace_id,span_id,occurred_at)
  VALUES(NULL,p_actor,'platform.tenant.force_remove','organizations',d.org_id::text,NULL,
   jsonb_build_object('org_id',d.org_id,'slug',o.slug,'name',o.name,'wiped',plan->'wiped'),NULL,
   p_trace,p_span,event_time);
 END IF;
 INSERT INTO public.platform_force_removal_receipts(actor_user_id,command_id,receipt_id,input_bytes,input_digest,
  org_id,group_id,family_id,effect_xid,effect_backend_pid,outcome,result_code,occurred_at,trace_id,span_id,heads_before,heads_after)
 VALUES(p_actor,d.command_id,result_receipt,p_input,d.input_digest,d.org_id,d.group_id,p_family,
  pg_current_xact_id(),pg_backend_pid(),CASE WHEN refusal IS NULL THEN 'APPLIED' ELSE 'REJECTED' END,
  coalesce(refusal,'removed'),event_time,p_trace,p_span,before_heads,after_heads);
 DELETE FROM public.platform_force_removal_effect_bindings x WHERE x.actor_user_id=p_actor AND x.command_id=d.command_id;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RETURN QUERY SELECT result_receipt,coalesce(refusal,'removed'),false;
EXCEPTION WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);RAISE;
END
$body$;
ALTER FUNCTION public.platform_force_remove_command_v1(uuid,uuid,bytea,bytea,text,text) OWNER TO console_app;
REVOKE ALL ON FUNCTION public.platform_force_remove_command_v1(uuid,uuid,bytea,bytea,text,text) FROM PUBLIC;
-- Final exact runtime grant only in the atomic caller/ACL/custody cutover.


-- source: legacy-closure-always
ALTER TABLE public.platform_legacy_topology_receipts ENABLE ALWAYS TRIGGER trg_platform_legacy_receipt_closed;
ALTER TABLE public.platform_legacy_membership_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_membership_binding_closed;
ALTER TABLE public.platform_legacy_catalog_effect_bindings ENABLE ALWAYS TRIGGER trg_platform_legacy_catalog_binding_closed;
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER trg_platform_legacy_head_effect_closed;
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER trg_platform_legacy_entity_effect_closed;
ALTER TABLE public.organizations ENABLE ALWAYS TRIGGER trg_platform_legacy_entity_effect_closed;


-- source: additional-metadata
ALTER FUNCTION public.company_enrollment_audit_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.company_enrollment_audit_guard_v1() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.company_enrollment_audit_guard_v1() TO console_account_owner;
CREATE TRIGGER company_enrollment_audit_guard_v1 BEFORE INSERT ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.company_enrollment_audit_guard_v1();
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER company_enrollment_audit_guard_v1;
CREATE TRIGGER company_enrollment_ontology_audit_guard_v1 BEFORE INSERT ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.company_enrollment_ontology_audit_guard_v1();
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER company_enrollment_ontology_audit_guard_v1;


-- source: attribution-trigger-ont_object_type_key_revisions
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_object_type_key_revisions
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_object_type_key_revisions ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_object_types
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_object_types
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_object_types ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_builtin_catalog_installs
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_builtin_catalog_installs
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_builtin_catalog_installs ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-cedar_policy_catalog_entries
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.cedar_policy_catalog_entries
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.cedar_policy_catalog_entries ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_object_policies
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_object_policies
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_object_policies ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_property_defs
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_property_defs
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_property_defs ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_link_types
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_link_types
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_link_types ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_action_types
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_action_types
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_action_types ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;

-- source: attribution-trigger-ont_analytics
CREATE TRIGGER native_catalog_attribution_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_analytics
 FOR EACH ROW EXECUTE FUNCTION ontology_api.native_catalog_attribution_guard_v1();
ALTER TABLE public.ont_analytics ENABLE ALWAYS TRIGGER native_catalog_attribution_guard_v1;
