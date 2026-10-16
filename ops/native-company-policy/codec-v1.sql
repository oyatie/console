-- Exact frozen Company business bytes; integer arithmetic preserves every
-- microsecond, including historical pre-epoch inputs. No current-time check.
CREATE FUNCTION public.native_company_policy_decode_v1(p_input bytea)
RETURNS TABLE(actor_account_id uuid,org_id uuid,command_id uuid,expected_company_epoch bigint,
 manifest_digest bytea,operation smallint,recipient_account_id uuid,expected_role_revision bigint,
 assignment_id uuid,expected_assignment_revision bigint,expires_at timestamptz)
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE size integer:=octet_length(p_input); witness integer; offset_value integer;
 expiry_us bigint;
BEGIN
 IF size NOT IN (123,148,155,180)
  OR substring(p_input FROM 1 FOR 34)<>convert_to('console.company.business-policy','UTF8')||decode('000001','hex') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 actor_account_id:=encode(substring(p_input FROM 35 FOR 16),'hex')::uuid;
 org_id:=encode(substring(p_input FROM 51 FOR 16),'hex')::uuid;
 command_id:=encode(substring(p_input FROM 67 FOR 16),'hex')::uuid;
 expected_company_epoch:=('x'||encode(substring(p_input FROM 83 FOR 8),'hex'))::bit(64)::bigint;
 manifest_digest:=substring(p_input FROM 91 FOR 32);
 operation:=get_byte(p_input,122)::smallint;
 IF '00000000-0000-0000-0000-000000000000'::uuid IN (actor_account_id,org_id,command_id)
  OR org_id='00000000-0000-0000-0000-00000000face'::uuid OR expected_company_epoch<1
  OR manifest_digest<>decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex')
  OR operation NOT BETWEEN 1 AND 3 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
 END IF;
 IF operation=1 THEN
  IF size<>123 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input'; END IF;
 ELSIF operation=2 THEN
  IF size NOT IN (148,180) THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input'; END IF;
  recipient_account_id:=encode(substring(p_input FROM 124 FOR 16),'hex')::uuid;
  witness:=get_byte(p_input,139);
  IF recipient_account_id='00000000-0000-0000-0000-000000000000'::uuid
   OR NOT ((witness=0 AND size=148) OR (witness=1 AND size=180)) THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
  IF witness=1 THEN offset_value:=141; END IF;
  expiry_us:=('x'||encode(substring(p_input FROM size-7 FOR 8),'hex'))::bit(64)::bigint;
  IF expiry_us NOT BETWEEN -62135596800000000 AND 253402268340000000 OR expiry_us%60000000<>0 THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input';
  END IF;
  -- The validated timestamp is minute aligned; integer text avoids the float8
  -- interval multiplication and to_timestamp(double precision) conversion.
  expires_at:=timestamptz '1970-01-01 00:00:00+00'+((expiry_us/60000000)::text||' minutes')::interval;
 ELSE
  IF size<>155 THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_company_policy.invalid_input'; END IF;
  offset_value:=124;
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
ALTER FUNCTION public.native_company_policy_decode_v1(bytea) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_company_policy_decode_v1(bytea) FROM PUBLIC;
