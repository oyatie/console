-- Exact new Group command1 and layout2 byte grammar. Historical codecs remain
-- untouched. These pure helpers confer no source, session or resource authority.
CREATE FUNCTION public.native_group_process_uuid_v1(p_bytes bytea,p_at integer) RETURNS uuid
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE result uuid;
BEGIN
 IF p_at<1 OR p_at>octet_length(p_bytes)-15 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 result:=encode(substring(p_bytes FROM p_at FOR 16),'hex')::uuid;
 IF result='00000000-0000-0000-0000-000000000000'::uuid THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 RETURN result;
END
$body$;
CREATE FUNCTION public.native_group_process_i64_v1(p_bytes bytea,p_at integer) RETURNS bigint
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF p_at<1 OR p_at>octet_length(p_bytes)-7 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 RETURN ('x'||encode(substring(p_bytes FROM p_at FOR 8),'hex'))::bit(64)::bigint;
END
$body$;
CREATE FUNCTION public.native_group_process_text_v1(p_value text,p_max integer) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE bytes bytea:=convert_to(p_value,'UTF8'); index_value integer;point integer; nonblank boolean:=false;
BEGIN
 IF octet_length(bytes) NOT BETWEEN 1 AND p_max OR btrim(p_value,E' \t\n\r')=''
  OR p_max NOT IN (120,128,2048) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 -- Unicode control points U+0000..001F and U+007F..009F; only tab/newline
 -- are admitted. UTF8 conversion also rejects ill-formed input at the boundary.
 FOR index_value IN 1..char_length(p_value) LOOP
  point:=ascii(substring(p_value FROM index_value FOR 1));
  IF (point BETWEEN 0 AND 31 AND point NOT IN (9,10)) OR point BETWEEN 127 AND 159 THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
  END IF;
  IF NOT (point BETWEEN 9 AND 13 OR point IN (32,133,160,5760,8232,8233,8239,8287,12288)
   OR point BETWEEN 8192 AND 8202) THEN nonblank:=true; END IF;
 END LOOP;
 IF NOT nonblank THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
 RETURN int4send(octet_length(bytes))||bytes;
END
$body$;
CREATE FUNCTION public.native_group_process_micros_v1(p_value timestamptz) RETURNS bigint
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF NOT isfinite(p_value) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 RETURN (extract(epoch FROM p_value)*1000000)::bigint;
END
$body$;

CREATE FUNCTION public.native_group_process_decode_v1(p_input bytea) RETURNS jsonb
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE result jsonb; content jsonb:='{}'; size integer:=octet_length(p_input);
 cursor_value integer:=1; operation integer; group_value uuid; incarnation_value uuid;
 value bigint; length_value bigint; text_value text; content_sum integer:=42; field_name text;
 prefix bytea:=convert_to('CONSOLE.IDENTITY.GROUP','UTF8')||decode('000001','hex');
BEGIN
 IF size NOT BETWEEN 188 AND 16521 OR substring(p_input FROM 1 FOR octet_length(prefix))<>prefix THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 cursor_value:=octet_length(prefix)+1;
 operation:=get_byte(p_input,cursor_value-1)*256+get_byte(p_input,cursor_value);cursor_value:=cursor_value+2;
 IF operation NOT IN (1,6) OR (operation=6 AND size NOT BETWEEN 208 AND 2255) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 result:=jsonb_build_object('operation',operation,
  'actor_account_id',public.native_group_process_uuid_v1(p_input,cursor_value),
  'command_id',public.native_group_process_uuid_v1(p_input,cursor_value+16));cursor_value:=cursor_value+32;
 group_value:=public.native_group_process_uuid_v1(p_input,cursor_value);
 incarnation_value:=public.native_group_process_uuid_v1(p_input,cursor_value+16);cursor_value:=cursor_value+32;
 value:=public.native_group_process_i64_v1(p_input,cursor_value);cursor_value:=cursor_value+8;
 IF value<1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
 result:=result||jsonb_build_object('group_id',group_value,'group_incarnation',incarnation_value,'expected_group_revision',value);
 value:=public.native_group_process_i64_v1(p_input,cursor_value);cursor_value:=cursor_value+8;
 IF value<0 OR (operation=6 AND value=0) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 result:=result||jsonb_build_object('expected_policy_revision',value,'process_id',public.native_group_process_uuid_v1(p_input,cursor_value));
 cursor_value:=cursor_value+16;
 IF operation=1 THEN
  value:=public.native_group_process_i64_v1(p_input,cursor_value);cursor_value:=cursor_value+8;
  IF value<0 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
  result:=result||jsonb_build_object('expected_prior_head_revision',value);
  value:=public.native_group_process_i64_v1(p_input,cursor_value);cursor_value:=cursor_value+8;
  -- HTML input grammar is years 0001..9999, KST seconds exactly; avoid
  -- floating-point to_timestamp and retain the original UTC microseconds.
  IF value NOT BETWEEN -62135629200000000 AND 253402268399000000 OR value%1000000<>0 THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
  END IF;
  result:=result||jsonb_build_object('expiry_us',value);
  IF substring(p_input FROM cursor_value FOR 2)<>decode('0001','hex') THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
  END IF;
  cursor_value:=cursor_value+2;
  FOREACH field_name IN ARRAY ARRAY['title','intended_claimant_matching_procedure','account_possession_procedure',
   'physical_human_evidence_procedure','duplicate_contradictory_claim_procedure','qualification_criteria_instruction',
   'escalation_adjudication_procedure','evidence_minimization_retention_description','recipient_responsibility'] LOOP
   IF cursor_value>size-3 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
   length_value:=('x'||encode(substring(p_input FROM cursor_value FOR 4),'hex'))::bit(32)::bigint;cursor_value:=cursor_value+4;
   IF length_value NOT BETWEEN 1 AND (CASE WHEN field_name='title' THEN 120 ELSE 2048 END)
    OR length_value>size-cursor_value+1 THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
   END IF;
   text_value:=convert_from(substring(p_input FROM cursor_value FOR length_value::integer),'UTF8');
   PERFORM public.native_group_process_text_v1(text_value,CASE WHEN field_name='title' THEN 120 ELSE 2048 END);
   cursor_value:=cursor_value+length_value::integer;content_sum:=content_sum+length_value::integer;
   content:=content||jsonb_build_object(field_name,text_value);
   IF field_name='title' THEN
    IF cursor_value>size-1 OR substring(p_input FROM cursor_value FOR 2)<>decode('0001','hex') THEN
     RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
    END IF;
    cursor_value:=cursor_value+2;
   END IF;
  END LOOP;
  IF content_sum>16384 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
  result:=result||jsonb_build_object('operator_responsibility',1,
   'content',content||jsonb_build_object('method','ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1'));
 ELSE
  value:=public.native_group_process_i64_v1(p_input,cursor_value);cursor_value:=cursor_value+8;
  IF value<1 OR cursor_value>size-31 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
  result:=result||jsonb_build_object('content_version',value,'content_digest',chr(92)||'x'||encode(substring(p_input FROM cursor_value FOR 32),'hex'));
  cursor_value:=cursor_value+32;value:=public.native_group_process_i64_v1(p_input,cursor_value);cursor_value:=cursor_value+8;
  IF value<1 OR cursor_value>size-31 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
  result:=result||jsonb_build_object('expected_head_revision',value,'expected_head_digest',chr(92)||'x'||encode(substring(p_input FROM cursor_value FOR 32),'hex'));
  cursor_value:=cursor_value+32;
  IF cursor_value>size-3 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
  length_value:=('x'||encode(substring(p_input FROM cursor_value FOR 4),'hex'))::bit(32)::bigint;cursor_value:=cursor_value+4;
  IF length_value NOT BETWEEN 1 AND 2048 OR length_value>size-cursor_value+1 THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
  END IF;
  text_value:=convert_from(substring(p_input FROM cursor_value FOR length_value::integer),'UTF8');
  PERFORM public.native_group_process_text_v1(text_value,2048);cursor_value:=cursor_value+length_value::integer;
  result:=result||jsonb_build_object('reason',text_value);
 END IF;
 IF cursor_value<>size+1 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input'; END IF;
 RETURN result;
END
$body$;
