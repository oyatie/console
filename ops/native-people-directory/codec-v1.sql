-- Concrete codec/storage candidate; install only inside the reviewed successor.
-- Matches approved application command v1; historical command codecs unchanged.
CREATE FUNCTION public.native_people_text_valid_v1(p_text text,p_scalars integer)
RETURNS boolean LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT CASE WHEN p_scalars NOT IN (64,200) OR octet_length(p_text)>p_scalars*4
  OR char_length(p_text) NOT BETWEEN 1 AND p_scalars THEN false ELSE
  NOT EXISTS(SELECT 1 FROM generate_series(1,char_length(p_text)) n
   WHERE ascii(substring(p_text FROM n FOR 1)) BETWEEN 0 AND 31
      OR ascii(substring(p_text FROM n FOR 1)) BETWEEN 127 AND 159)
  AND convert_to(p_text,'UTF8')=convert_to(btrim(p_text,U&'\0020\00a0\1680\2000\2001\2002\2003\2004\2005\2006\2007\2008\2009\200a\2028\2029\202f\205f\3000'),'UTF8') END
$body$;

CREATE FUNCTION public.native_people_encode_v1(i public.native_people_inputs_v1)
RETURNS bytea LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT decode('636f6e736f6c652e70656f706c652e6469726563746f72792d7265676973746572000001','hex')
  ||uuid_send(i.actor_account_id)||uuid_send(i.org_id)||uuid_send(i.command_id)||uuid_send(i.employee_id)
  ||int8send(i.expected_company_epoch)||uuid_send(i.expected_object_type_id)||uuid_send(i.expected_action_type_id)
  ||int8send(i.expected_action_revision)||int8send(i.expected_schema_revision)
  ||uuid_send(i.legal_name_property_id)||uuid_send(i.employee_number_property_id)
  ||decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')
  ||int2send(octet_length(i.legal_name)::smallint)||convert_to(i.legal_name,'UTF8')
  ||int2send(octet_length(i.employee_number)::smallint)||convert_to(i.employee_number,'UTF8')
$body$;

CREATE FUNCTION public.native_people_decode_v1(p_codec smallint,p_bytes bytea)
RETURNS TABLE(actor_account_id uuid,org_id uuid,command_id uuid,employee_id uuid,
 expected_company_epoch bigint,expected_object_type_id uuid,expected_action_type_id uuid,
 expected_action_revision bigint,expected_schema_revision bigint,legal_name_property_id uuid,
 employee_number_property_id uuid,manifest_digest bytea,legal_name text,employee_number text)
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
DECLARE name_bytes integer; number_bytes integer; number_length_offset integer;
BEGIN
 IF p_codec IS DISTINCT FROM 1 OR p_bytes IS NULL OR octet_length(p_bytes) NOT BETWEEN 226 AND 1280
  OR substring(p_bytes FROM 1 FOR 36) IS DISTINCT FROM
   decode('636f6e736f6c652e70656f706c652e6469726563746f72792d7265676973746572000001','hex')
  OR substring(p_bytes FROM 189 FOR 32) IS DISTINCT FROM
   decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex') THEN
  RAISE EXCEPTION 'people.directory.invalid_codec' USING ERRCODE='22023';
 END IF;
 actor_account_id:=encode(substring(p_bytes FROM 37 FOR 16),'hex')::uuid;
 org_id:=encode(substring(p_bytes FROM 53 FOR 16),'hex')::uuid;
 command_id:=encode(substring(p_bytes FROM 69 FOR 16),'hex')::uuid;
 employee_id:=encode(substring(p_bytes FROM 85 FOR 16),'hex')::uuid;
 expected_company_epoch:=('x'||encode(substring(p_bytes FROM 101 FOR 8),'hex'))::bit(64)::bigint;
 expected_object_type_id:=encode(substring(p_bytes FROM 109 FOR 16),'hex')::uuid;
 expected_action_type_id:=encode(substring(p_bytes FROM 125 FOR 16),'hex')::uuid;
 expected_action_revision:=('x'||encode(substring(p_bytes FROM 141 FOR 8),'hex'))::bit(64)::bigint;
 expected_schema_revision:=('x'||encode(substring(p_bytes FROM 149 FOR 8),'hex'))::bit(64)::bigint;
 legal_name_property_id:=encode(substring(p_bytes FROM 157 FOR 16),'hex')::uuid;
 employee_number_property_id:=encode(substring(p_bytes FROM 173 FOR 16),'hex')::uuid;
 manifest_digest:=substring(p_bytes FROM 189 FOR 32);
 IF '00000000-0000-0000-0000-000000000000'::uuid IN
  (actor_account_id,org_id,command_id,employee_id,expected_object_type_id,expected_action_type_id,legal_name_property_id,employee_number_property_id)
  OR org_id='00000000-0000-0000-0000-00000000face'::uuid
  OR expected_company_epoch<1 OR expected_action_revision<1 OR expected_schema_revision<1
  OR legal_name_property_id=employee_number_property_id THEN
  RAISE EXCEPTION 'people.directory.invalid_codec' USING ERRCODE='22023';
 END IF;
 name_bytes:=get_byte(p_bytes,220)*256+get_byte(p_bytes,221);
 number_length_offset:=222+name_bytes;
 IF name_bytes NOT BETWEEN 1 AND 800 OR number_length_offset+2>octet_length(p_bytes) THEN
  RAISE EXCEPTION 'people.directory.invalid_codec' USING ERRCODE='22023';
 END IF;
 number_bytes:=get_byte(p_bytes,number_length_offset)*256+get_byte(p_bytes,number_length_offset+1);
 IF number_bytes NOT BETWEEN 1 AND 256 OR number_length_offset+2+number_bytes<>octet_length(p_bytes) THEN
  RAISE EXCEPTION 'people.directory.invalid_codec' USING ERRCODE='22023';
 END IF;
 legal_name:=convert_from(substring(p_bytes FROM 223 FOR name_bytes),'UTF8');
 employee_number:=convert_from(substring(p_bytes FROM number_length_offset+3 FOR number_bytes),'UTF8');
 IF NOT public.native_people_text_valid_v1(legal_name,200)
  OR NOT public.native_people_text_valid_v1(employee_number,64) THEN
  RAISE EXCEPTION 'people.directory.invalid_codec' USING ERRCODE='22023';
 END IF;
 RETURN NEXT;
EXCEPTION WHEN character_not_in_repertoire OR untranslatable_character THEN
 RAISE EXCEPTION 'people.directory.invalid_codec' USING ERRCODE='22023';
END
$body$;

CREATE FUNCTION public.native_people_effect_digest_v1(i public.native_people_inputs_v1)
RETURNS bytea LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT sha256(convert_to('console.people.directory-effect','UTF8')||decode('000001','hex')||i.input_bytes)
$body$;

CREATE FUNCTION public.native_people_result_v1(i public.native_people_inputs_v1)
RETURNS jsonb LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT jsonb_build_object('person_id',i.employee_id::text,'version',1,'target','people.create_person')
$body$;
