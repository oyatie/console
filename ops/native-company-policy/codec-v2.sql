-- Closed successor decoder. Stored discriminators select their exact grammar;
-- the original Payroll decoder and all acknowledged bytes remain unchanged.
CREATE FUNCTION public.native_company_policy_codec_v2(p_input bytea) RETURNS smallint
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF substring(p_input FROM 1 FOR 34)=convert_to('console.company.business-policy','UTF8')||decode('000001','hex') THEN
  RETURN 1;
 ELSIF substring(p_input FROM 1 FOR 32)=convert_to('console.company.people-policy','UTF8')||decode('000001','hex') THEN
  RETURN 2;
 END IF;
 RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
END
$body$;

CREATE FUNCTION public.native_company_policy_decode_v2(p_codec smallint,p_input bytea)
RETURNS TABLE(actor_account_id uuid,org_id uuid,command_id uuid,expected_company_epoch bigint,
 manifest_digest bytea,operation smallint,recipient_account_id uuid,expected_role_revision bigint,
 assignment_id uuid,expected_assignment_revision bigint,expires_at timestamptz,
 codec_version smallint,catalog_version text,action_key text,role_key text)
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE size integer:=octet_length(p_input); witness integer; offset_value integer;
 expiry_us bigint; action_byte integer;
BEGIN
 IF p_codec=1 THEN
  RETURN QUERY SELECT d.*,1::smallint,'native-payroll-collection-read-v1'::text,
   'payroll.collection.read'::text,'native_payroll_collection_read'::text
   FROM public.native_company_policy_decode_v1(p_input) d;
  RETURN;
 END IF;
 IF p_codec<>2 OR size NOT IN (121,147,154,179)
  OR substring(p_input FROM 1 FOR 32)<>convert_to('console.company.people-policy','UTF8')||decode('000001','hex') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 actor_account_id:=encode(substring(p_input FROM 33 FOR 16),'hex')::uuid;
 org_id:=encode(substring(p_input FROM 49 FOR 16),'hex')::uuid;
 command_id:=encode(substring(p_input FROM 65 FOR 16),'hex')::uuid;
 expected_company_epoch:=('x'||encode(substring(p_input FROM 81 FOR 8),'hex'))::bit(64)::bigint;
 manifest_digest:=substring(p_input FROM 89 FOR 32);
 operation:=get_byte(p_input,120)::smallint;
 codec_version:=2; catalog_version:='native-people-directory-v1';
 IF '00000000-0000-0000-0000-000000000000'::uuid IN (actor_account_id,org_id,command_id)
  OR org_id='00000000-0000-0000-0000-00000000face'::uuid OR expected_company_epoch<1
  OR manifest_digest<>decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex')
  OR operation NOT BETWEEN 1 AND 3 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 IF operation=1 THEN
  IF size<>121 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input'; END IF;
 ELSE
  IF (operation=2 AND size NOT IN (147,179)) OR (operation=3 AND size<>154) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
  action_byte:=get_byte(p_input,121);
  IF action_byte NOT IN (1,2) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
  action_key:=(ARRAY['people.directory.read','people.directory.create'])[action_byte];
  role_key:=(ARRAY['native_people_directory_read','native_people_directory_create'])[action_byte];
  IF operation=2 THEN
   recipient_account_id:=encode(substring(p_input FROM 123 FOR 16),'hex')::uuid;
   witness:=get_byte(p_input,138);
   IF recipient_account_id='00000000-0000-0000-0000-000000000000'::uuid
    OR NOT ((witness=0 AND size=147) OR (witness=1 AND size=179)) THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
   END IF;
   IF witness=1 THEN offset_value:=140; END IF;
   expiry_us:=('x'||encode(substring(p_input FROM size-7 FOR 8),'hex'))::bit(64)::bigint;
   IF expiry_us NOT BETWEEN -62135596800000000 AND 253402268340000000 OR expiry_us%60000000<>0 THEN
    RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
   END IF;
   expires_at:=timestamptz '1970-01-01 00:00:00+00'+((expiry_us/60000000)::text||' minutes')::interval;
  ELSE offset_value:=123;
  END IF;
 END IF;
 IF offset_value IS NOT NULL THEN
  expected_role_revision:=('x'||encode(substring(p_input FROM offset_value FOR 8),'hex'))::bit(64)::bigint;
  assignment_id:=encode(substring(p_input FROM offset_value+8 FOR 16),'hex')::uuid;
  expected_assignment_revision:=('x'||encode(substring(p_input FROM offset_value+24 FOR 8),'hex'))::bit(64)::bigint;
  IF expected_role_revision<>1 OR expected_assignment_revision<1
   OR assignment_id='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
 END IF;
 RETURN NEXT;
END
$body$;
