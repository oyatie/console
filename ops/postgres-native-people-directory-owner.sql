-- Declared source for disposable capture only; not a custody finalizer.
-- source: ops/native-people-directory/expansion-v1.sql
-- First module of the atomic native People successor, never a standalone installer.
-- The outer owner must verify and lock the exact old custody plus closed230
-- before this expansion. All remaining modules and final capture share its transaction.
-- Historical canonical ALTER statements and column comments are retained verbatim.

ALTER TABLE public.person_revisions
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ALTER COLUMN actor_id DROP NOT NULL,
    ADD CONSTRAINT person_revisions_actor_staged_user_v1
        CHECK (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL);
ALTER TABLE public.employee_person_bindings
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ALTER COLUMN actor_id DROP NOT NULL,
    ADD CONSTRAINT employee_person_bindings_actor_staged_user_v1
        CHECK (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL);
ALTER TABLE public.ont_action_command_receipts
    ADD COLUMN actor_kind text NOT NULL DEFAULT 'USER',
    ADD COLUMN actor_account_id uuid,
    ALTER COLUMN actor_id DROP NOT NULL,
    ADD CONSTRAINT ont_action_receipts_actor_staged_user_v1
        CHECK (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL);
-- Preserve old actor_id FKs, shared Company+command PK, owner/target checks,
-- all old table ACLs/RLS/immutability, and0229 UPDATE(employee_id) row-lock grant.
-- Native company_actors FKs are deliberately NOT installed by this migration:
-- finalized Account custody removes migration-role REFERENCES privileges.

ALTER TABLE public.employees
    ADD COLUMN source_kind text NOT NULL DEFAULT 'LEGACY',
    ADD COLUMN native_command_id uuid,
    ALTER COLUMN source_filename DROP NOT NULL,
    ALTER COLUMN source_sheet DROP NOT NULL,
    ALTER COLUMN source_row DROP NOT NULL,
    ADD CONSTRAINT employees_provenance_staged_legacy_v1 CHECK(
        source_kind='LEGACY' AND native_command_id IS NULL
        AND source_filename IS NOT NULL AND source_sheet IS NOT NULL AND source_row IS NOT NULL
        AND btrim(source_filename)<>'' AND btrim(source_sheet)<>'' AND source_row>0),
    ADD CONSTRAINT employees_native_intake_v1
        FOREIGN KEY(org_id,native_command_id,id)
        REFERENCES public.native_people_inputs_v1(org_id,command_id,employee_id)
        ON UPDATE RESTRICT ON DELETE RESTRICT;
-- Historical inline nonblank/positive checks remain; native NULL acceptance is
-- still impossible. Existing employee-number/import behavior is unchanged while
-- dormant. Do not open provenance gate until successor guards are complete.

COMMENT ON COLUMN public.person_revisions.actor_kind IS 'pd:personal — canonical Person actor attribution protocol';
COMMENT ON COLUMN public.person_revisions.actor_account_id IS 'pd:personal — canonical Person Account actor identity';
COMMENT ON COLUMN public.employee_person_bindings.actor_kind IS 'pd:personal — employee Person binding actor attribution protocol';
COMMENT ON COLUMN public.employee_person_bindings.actor_account_id IS 'pd:personal — employee Person binding Account actor identity';
COMMENT ON COLUMN public.ont_action_command_receipts.actor_kind IS 'pd:personal — canonical command actor attribution protocol';
COMMENT ON COLUMN public.ont_action_command_receipts.actor_account_id IS 'pd:personal — canonical command Account actor identity';
COMMENT ON COLUMN public.employees.source_kind IS 'pd:personal — directory identity provenance category';
COMMENT ON COLUMN public.employees.native_command_id IS 'pd:personal — accepted native directory provenance command';

-- source: ops/native-people-directory/codec-v1.sql
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

-- source: ops/native-people-directory/current-source-v1.sql
-- Proposal: new People business source; no replacement of historical Payroll SQL.
-- This source retains the inherited Company birth-administrator admission ceiling.
-- A catalog install is not a grant. Caller must authenticate retained credentials,
-- evaluate actual Cedar, and recheck this source before disclosing or committing.
CREATE FUNCTION public.identity_company_people_projection_v1(
 p_account uuid,p_family uuid,p_company uuid,p_action text)
RETURNS TABLE(company_epoch bigint,current_policy_receipt_id uuid,
 assignment_id uuid,assignment_revision bigint,role_id uuid,role_revision bigint,
 registered_clauses jsonb,assignment_valid_from timestamptz,assignment_valid_until timestamptz,
 action_reference jsonb,named_properties jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); identity_row record;
 role_row public.policy_roles; a public.user_role_assignments; ar public.policy_assignment_revisions;
 r public.native_company_policy_receipts_v1; clauses jsonb; at_time timestamptz;
 selected_role text; action_ref public.native_company_action_refs; fields jsonb;
BEGIN
 IF p_action IS NULL OR p_action NOT IN ('people.directory.read','people.directory.create') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_action';
 END IF;
 selected_role:=CASE p_action WHEN 'people.directory.read' THEN 'native_people_directory_read'
  ELSE 'native_people_directory_create' END;
 -- Existing source owns Group-first topology, Account/family, Company and
 -- registered catalog locks; it does not itself authorize the business action.
 SELECT * INTO identity_row FROM public.identity_company_projection_v2(p_account,p_family,p_company);
 IF NOT FOUND THEN RETURN; END IF;
 PERFORM set_config('app.current_org',p_company::text,true);
 SELECT x.* INTO role_row FROM public.policy_roles x
  WHERE x.org_id=p_company AND x.role_key=selected_role FOR SHARE;
 IF NOT FOUND THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 -- Native-subject unique index binds (Company, Account, role); no grant is
 -- an ordinary denial, distinct from a missing revision of an existing grant.
 SELECT x.* INTO a FROM public.user_role_assignments x WHERE x.org_id=p_company
  AND x.role_id=role_row.id AND x.account_id=p_account AND x.subject_protocol='NATIVE_ACCOUNT' FOR SHARE;
 IF NOT FOUND THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT x.* INTO STRICT ar FROM public.policy_assignment_revisions x
  WHERE x.org_id=p_company AND x.assignment_id=a.id AND x.revision=a.native_current_revision;
 SELECT x.* INTO STRICT r FROM public.native_company_policy_receipts_v1 x
  WHERE x.org_id=p_company AND x.receipt_id=ar.policy_receipt_id;
 IF ar.valid_from IS NULL OR ar.valid_until IS NULL
  OR NOT isfinite(ar.valid_from) OR NOT isfinite(ar.valid_until)
  OR ar.valid_from>=ar.valid_until OR ar.valid_until-ar.valid_from>interval '30 days'
  OR r.codec_version<>2 OR r.catalog_version<>'native-people-directory-v1'
  OR r.outcome<>'COMMITTED' OR r.operation NOT IN (2,3)
  OR r.assignment_id IS DISTINCT FROM a.id OR r.assignment_revision_after IS DISTINCT FROM ar.revision
  OR a.native_current_revision<1 OR role_row.subject_protocol<>'NATIVE_ACCOUNT'
  OR ar.subject_protocol<>'NATIVE_ACCOUNT' OR ar.account_id IS DISTINCT FROM p_account
  OR ar.role_id IS DISTINCT FROM role_row.id OR ar.role_revision<>1
  OR (r.recipient_account_id,r.role_id,r.role_revision,r.assignment_state_after,r.assignment_valid_from,r.assignment_valid_until)
   IS DISTINCT FROM (p_account,role_row.id,1::bigint,ar.state,ar.valid_from,ar.valid_until) THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 PERFORM public.native_company_policy_assert_effects_v1(p_company,r.receipt_id);
 clauses:=public.native_company_policy_business_clauses_v1(p_company,role_row.id);
 SELECT x.* INTO STRICT action_ref FROM public.native_company_action_refs x
  WHERE x.org_id=p_company AND x.catalog_version='native-people-directory-v1' AND x.action_key=p_action;
 SELECT jsonb_agg(jsonb_build_object('key',p.property_key,'org_id',p.org_id::text,
   'object_type_id',p.object_type_id::text,'property_id',p.property_id::text,
   'schema_revision',p.schema_revision::text) ORDER BY p.property_key COLLATE "C") INTO fields
 FROM public.native_company_property_refs p
 WHERE p.org_id=p_company AND p.catalog_version='native-people-directory-v1'
  AND p.object_type_id=action_ref.object_type_id
  AND (p_action='people.directory.read' OR p.property_key IN ('person.legal_name','person.employee_number'));
 IF jsonb_array_length(fields) IS DISTINCT FROM
    (CASE p_action WHEN 'people.directory.read' THEN 6 ELSE 2 END)
  OR action_ref.registration_revision<>1
  OR action_ref.manifest_digest IS DISTINCT FROM
     decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex') THEN
  RAISE EXCEPTION 'identity_native.material_unavailable';
 END IF;
 at_time:=clock_timestamp();
 IF ar.state<>'ACTIVE' OR role_row.status<>'ACTIVE'
  OR ar.valid_from>at_time OR ar.valid_until<=at_time THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 RETURN QUERY SELECT identity_row.company_epoch,identity_row.current_policy_receipt_id,
  a.id,a.native_current_revision,role_row.id,1::bigint,clauses,ar.valid_from,ar.valid_until,
  jsonb_build_object('org_id',action_ref.org_id::text,'object_type_id',action_ref.object_type_id::text,
   'action_type_id',action_ref.action_type_id::text,'registration_revision',action_ref.registration_revision::text,
   'manifest_digest',encode(action_ref.manifest_digest,'hex')),fields;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
 RAISE EXCEPTION 'identity_native.material_unavailable';
WHEN OTHERS THEN
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- Successor custody installation (same atomic installer transaction).
ALTER FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) FROM PUBLIC;
DO $acl$
DECLARE grantee_name text;
BEGIN
 FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_catalog.pg_proc p
  CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a
  JOIN pg_catalog.pg_roles r ON r.oid=a.grantee
  WHERE p.oid='public.identity_company_people_projection_v1(uuid,uuid,uuid,text)'::regprocedure
   AND r.rolname<>'console_account_owner'
 LOOP
  EXECUTE format('REVOKE ALL ON FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) FROM %I',grantee_name);
 END LOOP;
END
$acl$;
GRANT EXECUTE ON FUNCTION public.identity_company_people_projection_v1(uuid,uuid,uuid,text) TO console_rt;

-- source: ops/native-people-directory/commands-v1.sql
-- UNEXECUTED SOURCE PROPOSAL. Protected intake and terminal allocation only.
-- Canonical employee/Person DML and common audit INSERT remain existing Rust owners.
-- Same retained adapter transaction authenticates original credentials, validates
-- CSRF and evaluates actual Cedar both before entry and after deferred closure.
CREATE FUNCTION public.native_people_expectations_match_v1(i public.native_people_inputs_v1,m jsonb)
RETURNS boolean LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT (i.expected_company_epoch=(m->>'company_epoch')::bigint
  AND i.expected_object_type_id=(m#>>'{action_reference,object_type_id}')::uuid
  AND i.expected_action_type_id=(m#>>'{action_reference,action_type_id}')::uuid
  AND i.expected_action_revision=(m#>>'{action_reference,registration_revision}')::bigint
  AND i.manifest_digest=decode(m#>>'{action_reference,manifest_digest}','hex')
  AND (SELECT count(*) FROM jsonb_array_elements(m->'named_properties'))=2
  AND EXISTS(SELECT 1 FROM jsonb_array_elements(m->'named_properties') p
   WHERE p->>'key'='person.legal_name' AND (p->>'org_id')::uuid=i.org_id
    AND (p->>'object_type_id')::uuid=i.expected_object_type_id
    AND (p->>'property_id')::uuid=i.legal_name_property_id
    AND (p->>'schema_revision')::bigint=i.expected_schema_revision)
  AND EXISTS(SELECT 1 FROM jsonb_array_elements(m->'named_properties') p
   WHERE p->>'key'='person.employee_number' AND (p->>'org_id')::uuid=i.org_id
    AND (p->>'object_type_id')::uuid=i.expected_object_type_id
    AND (p->>'property_id')::uuid=i.employee_number_property_id
    AND (p->>'schema_revision')::bigint=i.expected_schema_revision)) IS TRUE
$body$;

CREATE FUNCTION public.native_people_current_v1(p_actor uuid,p_family uuid,p_org uuid)
RETURNS jsonb LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE m jsonb;
BEGIN
 SELECT to_jsonb(s) INTO STRICT m FROM public.identity_company_people_projection_v1(
  p_actor,p_family,p_org,'people.directory.create') s;
 RETURN m;
EXCEPTION WHEN no_data_found THEN RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='people.directory.not_visible';
END
$body$;

CREATE FUNCTION public.native_people_prepare_v1(
 p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_expected_company_epoch bigint,
 p_object uuid,p_action uuid,p_action_revision bigint,p_schema_revision bigint,
 p_name_property uuid,p_number_property uuid,p_name text,p_number text)
RETURNS TABLE(inserted boolean,accepted_input public.native_people_inputs_v1,terminal public.native_people_terminals_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); m jsonb; at_time timestamptz;
BEGIN
 m:=public.native_people_current_v1(p_actor,p_family,p_org);
 PERFORM set_config('app.current_org',p_org::text,true);
 -- Company admission is bounded independently; unrelated Company work proceeds.
 PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.admission/1:'||p_org::text,0));
 PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.command/1:'||p_org::text||':'||p_command::text,0));
 SELECT i.* INTO accepted_input FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org AND i.command_id=p_command;
 IF FOUND THEN
  IF (accepted_input.actor_account_id,accepted_input.expected_company_epoch,accepted_input.expected_object_type_id,
    accepted_input.expected_action_type_id,accepted_input.expected_action_revision,accepted_input.expected_schema_revision,
    accepted_input.legal_name_property_id,accepted_input.employee_number_property_id,accepted_input.legal_name COLLATE "C",accepted_input.employee_number COLLATE "C")
   IS DISTINCT FROM (p_actor,p_expected_company_epoch,p_object,p_action,p_action_revision,p_schema_revision,p_name_property,p_number_property,p_name COLLATE "C",p_number COLLATE "C") THEN
   RAISE EXCEPTION 'people.directory.conflict';
  END IF;
  SELECT t.* INTO terminal FROM public.native_people_terminals_v1 t WHERE t.org_id=p_org AND t.command_id=p_command;
  inserted:=false;
 ELSE
  -- Never trim or normalize persisted inputs. Adapter validates user text first.
  IF public.native_people_text_valid_v1(p_name,200) IS DISTINCT FROM true
   OR public.native_people_text_valid_v1(p_number,64) IS DISTINCT FROM true THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_input';
  END IF;
  accepted_input.org_id:=p_org; accepted_input.command_id:=p_command; accepted_input.actor_account_id:=p_actor;
  accepted_input.employee_id:=gen_random_uuid(); accepted_input.legal_name:=p_name; accepted_input.employee_number:=p_number;
  accepted_input.expected_company_epoch:=p_expected_company_epoch; accepted_input.expected_object_type_id:=p_object;
  accepted_input.expected_action_type_id:=p_action; accepted_input.expected_action_revision:=p_action_revision;
  accepted_input.expected_schema_revision:=p_schema_revision; accepted_input.legal_name_property_id:=p_name_property;
  accepted_input.employee_number_property_id:=p_number_property;
  accepted_input.manifest_digest:=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex');
  accepted_input.codec_version:=1; accepted_input.input_bytes:=public.native_people_encode_v1(accepted_input);
  PERFORM 1 FROM public.native_people_decode_v1(1::smallint,accepted_input.input_bytes);
  m:=public.native_people_current_v1(p_actor,p_family,p_org);
  IF NOT public.native_people_expectations_match_v1(accepted_input,m) THEN RAISE EXCEPTION 'people.directory.revision_conflict'; END IF;
  IF EXISTS(SELECT 1 FROM public.ont_action_command_receipts r WHERE r.org_id=p_org AND r.command_id=p_command) THEN
   RAISE EXCEPTION 'people.directory.command_conflict';
  END IF;
  at_time:=clock_timestamp();
  IF (SELECT count(*) FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org
    AND i.execution_not_after>at_time AND NOT EXISTS(SELECT 1 FROM public.native_people_terminals_v1 t WHERE t.org_id=i.org_id AND t.command_id=i.command_id))>=256
   OR (SELECT count(*) FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org AND i.actor_account_id=p_actor
    AND i.execution_not_after>at_time AND NOT EXISTS(SELECT 1 FROM public.native_people_terminals_v1 t WHERE t.org_id=i.org_id AND t.command_id=i.command_id))>=16 THEN
   RAISE EXCEPTION 'people.directory.capacity';
  END IF;
  accepted_input.input_digest:=sha256(accepted_input.input_bytes); accepted_input.intake_receipt_id:=gen_random_uuid();
  accepted_input.accepted_at:=at_time; accepted_input.execution_not_after:=at_time+interval '168 hours';
  accepted_input.accepting_session_id:=p_family; accepted_input.acceptance_xid:=pg_current_xact_id();
  accepted_input.acceptance_backend_pid:=pg_backend_pid(); accepted_input.accepting_assignment_id:=(m->>'assignment_id')::uuid;
  accepted_input.accepting_assignment_revision:=(m->>'assignment_revision')::bigint;
  INSERT INTO public.native_people_inputs_v1 SELECT accepted_input.*;
  inserted:=true;
 END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_people_terminal_open_v1(p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_transition text)
RETURNS TABLE(inserted boolean,accepted_input public.native_people_inputs_v1,terminal public.native_people_terminals_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); m jsonb; at_time timestamptz; failure text;
BEGIN
 IF p_transition IS NULL OR p_transition NOT IN ('EXECUTE','CANCEL','STATUS') THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_transition';
 END IF;
 m:=public.native_people_current_v1(p_actor,p_family,p_org);
 PERFORM set_config('app.current_org',p_org::text,true);
 PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.command/1:'||p_org::text||':'||p_command::text,0));
 SELECT i.* INTO accepted_input FROM public.native_people_inputs_v1 i WHERE i.org_id=p_org AND i.command_id=p_command;
 IF NOT FOUND OR accepted_input.actor_account_id IS DISTINCT FROM p_actor THEN
  PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN;
 END IF;
 SELECT t.* INTO terminal FROM public.native_people_terminals_v1 t WHERE t.org_id=p_org AND t.command_id=p_command;
 IF FOUND THEN
  inserted:=false; PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT; RETURN;
 END IF;
 IF accepted_input.acceptance_xid=pg_current_xact_id() THEN RAISE EXCEPTION 'people.directory.separate_commit_required'; END IF;
 IF accepted_input.input_bytes IS DISTINCT FROM public.native_people_encode_v1(accepted_input)
  OR accepted_input.input_digest IS DISTINCT FROM sha256(accepted_input.input_bytes) THEN RAISE EXCEPTION 'people.directory.custody_invalid'; END IF;
 PERFORM 1 FROM public.native_people_decode_v1(accepted_input.codec_version,accepted_input.input_bytes);
 IF p_transition='EXECUTE' THEN
  -- Same historical key as legacy number writers; held through canonical INSERT.
  PERFORM pg_advisory_xact_lock(hashtext(p_org::text||':'||accepted_input.employee_number));
 END IF;
 m:=public.native_people_current_v1(p_actor,p_family,p_org); at_time:=clock_timestamp();
 IF at_time>=accepted_input.execution_not_after THEN failure:='intake_expired';
 ELSIF p_transition='STATUS' THEN
  inserted:=false; PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT; RETURN;
 ELSIF p_transition='CANCEL' THEN failure:='cancelled';
 ELSIF NOT public.native_people_expectations_match_v1(accepted_input,m) THEN failure:='revision_conflict';
 ELSIF EXISTS(SELECT 1 FROM public.ont_action_command_receipts r WHERE r.org_id=p_org AND r.command_id=p_command) THEN failure:='command_conflict';
 ELSIF EXISTS(SELECT 1 FROM public.employees e WHERE e.org_id=p_org AND e.employee_number COLLATE "C"=accepted_input.employee_number COLLATE "C") THEN failure:='employee_number_conflict';
 END IF;
 terminal.org_id:=p_org; terminal.command_id:=p_command; terminal.actor_account_id:=p_actor;
 terminal.intake_receipt_id:=accepted_input.intake_receipt_id; terminal.input_digest:=accepted_input.input_digest;
 terminal.outcome:=CASE failure WHEN 'intake_expired' THEN 'EXPIRED' WHEN 'cancelled' THEN 'CANCELLED'
  ELSE CASE WHEN failure IS NULL THEN 'COMMITTED' ELSE 'REJECTED' END END;
 terminal.result_code:=coalesce(failure,'registered'); terminal.transition_kind:=p_transition;
 terminal.execution_session_id:=p_family; terminal.effect_xid:=pg_current_xact_id(); terminal.effect_backend_pid:=pg_backend_pid();
 terminal.source_company_epoch:=(m->>'company_epoch')::bigint; terminal.source_policy_receipt_id:=(m->>'current_policy_receipt_id')::uuid;
 terminal.source_assignment_id:=(m->>'assignment_id')::uuid; terminal.source_assignment_revision:=(m->>'assignment_revision')::bigint;
 terminal.source_valid_from:=(m->>'assignment_valid_from')::timestamptz; terminal.source_valid_until:=(m->>'assignment_valid_until')::timestamptz;
 terminal.terminal_at:=at_time;
 IF failure IS NULL THEN terminal.employee_id:=accepted_input.employee_id; terminal.person_id:=accepted_input.employee_id; terminal.canonical_command_id:=p_command; END IF;
 -- Provisional only. Deferred closure requires all canonical rows/audit before COMMIT.
 INSERT INTO public.native_people_terminals_v1 SELECT terminal.*;
 inserted:=true; PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_people_preflight_v1(
 p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_expected_company_epoch bigint,
 p_object uuid,p_action uuid,p_action_revision bigint,p_schema_revision bigint,
 p_name_property uuid,p_number_property uuid,p_name text,p_number text)
RETURNS void LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); m jsonb; i public.native_people_inputs_v1;
BEGIN
 m:=public.native_people_current_v1(p_actor,p_family,p_org);
 IF p_command IS NULL OR p_command='00000000-0000-0000-0000-000000000000'::uuid
  OR public.native_people_text_valid_v1(p_name,200) IS DISTINCT FROM true
  OR public.native_people_text_valid_v1(p_number,64) IS DISTINCT FROM true THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_input';
 END IF;
 i.org_id:=p_org; i.expected_company_epoch:=p_expected_company_epoch; i.expected_object_type_id:=p_object;
 i.expected_action_type_id:=p_action; i.expected_action_revision:=p_action_revision;
 i.expected_schema_revision:=p_schema_revision; i.legal_name_property_id:=p_name_property;
 i.employee_number_property_id:=p_number_property;
 i.manifest_digest:=decode('591e8fe626a11ce724c81330f0358f6f4fb3df4fa1532fa329ee28d7c5b38e5e','hex');
 IF NOT public.native_people_expectations_match_v1(i,m) THEN RAISE EXCEPTION 'people.directory.revision_conflict'; END IF;
 PERFORM set_config('app.current_org',p_org::text,true);
 IF EXISTS(SELECT 1 FROM public.ont_action_command_receipts r WHERE r.org_id=p_org AND r.command_id=p_command)
  OR EXISTS(SELECT 1 FROM public.native_people_inputs_v1 a WHERE a.org_id=p_org AND a.command_id=p_command) THEN
  RAISE EXCEPTION 'people.directory.command_conflict';
 END IF;
 IF EXISTS(SELECT 1 FROM public.employees e WHERE e.org_id=p_org AND e.employee_number COLLATE "C"=p_number COLLATE "C") THEN
  RAISE EXCEPTION 'people.directory.employee_number_conflict';
 END IF;
 -- Feasibility only; execute rechecks under shared number/command serialization.
 -- No identity allocation, intake/terminal, audit, or Auth proof side effect.
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true);
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: ops/native-people-directory/guards-v1.sql
-- UNEXECUTED activation source; canonical effects remain Rust-owned.
CREATE FUNCTION public.native_people_frame_v1(p_org uuid,p_command uuid)
RETURNS public.native_people_terminals_v1
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE t public.native_people_terminals_v1; i public.native_people_inputs_v1; m jsonb;
BEGIN
 SELECT x.* INTO STRICT t FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
 SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
 IF t.outcome<>'COMMITTED' OR t.effect_xid<>pg_current_xact_id() OR t.effect_backend_pid<>pg_backend_pid()
  OR t.transition_kind<>'EXECUTE' OR t.terminal_at<i.accepted_at OR t.terminal_at>=i.execution_not_after
  OR clock_timestamp()>=i.execution_not_after OR i.input_digest<>sha256(i.input_bytes)
  OR i.input_bytes<>public.native_people_encode_v1(i) THEN RAISE EXCEPTION 'people.directory.effect_frame_invalid'; END IF;
 m:=public.native_people_current_v1(t.actor_account_id,t.execution_session_id,p_org);
 IF NOT public.native_people_expectations_match_v1(i,m)
  OR (t.source_company_epoch,t.source_policy_receipt_id,t.source_assignment_id,t.source_assignment_revision,t.source_valid_from,t.source_valid_until)
   IS DISTINCT FROM ((m->>'company_epoch')::bigint,(m->>'current_policy_receipt_id')::uuid,(m->>'assignment_id')::uuid,
    (m->>'assignment_revision')::bigint,(m->>'assignment_valid_from')::timestamptz,(m->>'assignment_valid_until')::timestamptz) THEN
  RAISE EXCEPTION 'people.directory.effect_frame_invalid';
 END IF;
 RETURN t;
END
$body$;

CREATE FUNCTION public.native_people_employee_shape_v1(e public.employees,i public.native_people_inputs_v1,t public.native_people_terminals_v1)
RETURNS boolean LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT (e.id=i.employee_id AND e.org_id=i.org_id
  AND convert_to(e.name,'UTF8')=convert_to(i.legal_name,'UTF8')
  AND convert_to(e.employee_number,'UTF8')=convert_to(i.employee_number,'UTF8')
  AND e.source_kind='NATIVE_DIRECTORY' AND e.native_command_id=i.command_id
  AND e.source_key='native-directory:'||i.command_id::text
  AND e.raw_row='{}'::jsonb AND e.source_metadata='{}'::jsonb
  AND e.created_at=t.terminal_at AND e.updated_at=t.terminal_at
  AND e.employment_status='UNKNOWN' AND e.identity_resolution_strategy='employee_number'
  AND e.identity_resolution_confidence='low' AND e.identity_review_required AND NOT e.identity_name_only_merge
  AND e.source_filename IS NULL AND e.source_sheet IS NULL AND e.source_row IS NULL
  AND e.home_branch_id IS NULL AND e.hire_date IS NULL AND e.exit_date IS NULL
  AND e.org_unit IS NULL AND e.job IS NULL AND e.position IS NULL
  AND e.worksite_name IS NULL AND e.worksite_address IS NULL
  AND e.leave_accrued IS NULL AND e.leave_used IS NULL AND e.leave_remaining IS NULL) IS TRUE
$body$;

CREATE FUNCTION public.native_people_employee_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE i public.native_people_inputs_v1; t public.native_people_terminals_v1; company_name text;
BEGIN
 IF TG_OP<>'INSERT' THEN
  IF OLD.source_kind='NATIVE_DIRECTORY' OR (TG_OP='UPDATE' AND NEW.source_kind IS DISTINCT FROM OLD.source_kind) THEN
   -- Directory provenance and identity are immutable. A later, separately admitted
   -- employment/name-correction owner must version its own transitions explicitly.
   RAISE EXCEPTION 'people.directory.native_provenance_immutable';
  END IF;
  IF TG_OP='DELETE' THEN RETURN OLD; END IF;
  IF (NEW.org_id,NEW.id) IS DISTINCT FROM (OLD.org_id,OLD.id) THEN
   SELECT x.* INTO i FROM public.native_people_inputs_v1 x WHERE x.org_id=NEW.org_id AND x.employee_id=NEW.id;
   IF i.command_id IS NOT NULL THEN
    PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.command/1:'||i.org_id::text||':'||i.command_id::text,0));
    RAISE EXCEPTION 'people.directory.reserved_identity';
   END IF;
  END IF;
  RETURN NEW;
 END IF;
 IF NEW.source_kind='LEGACY' THEN
  IF NEW.native_command_id IS NOT NULL OR NEW.source_filename IS NULL OR NEW.source_sheet IS NULL OR NEW.source_row IS NULL THEN
   RAISE EXCEPTION 'people.directory.invalid_legacy_provenance';
  END IF;
  IF EXISTS(SELECT 1 FROM public.native_people_inputs_v1 x WHERE x.org_id=NEW.org_id AND x.employee_id=NEW.id) THEN
   RAISE EXCEPTION 'people.directory.reserved_identity';
  END IF;
  RETURN NEW;
 END IF;
 IF NEW.source_kind IS DISTINCT FROM 'NATIVE_DIRECTORY' THEN RAISE EXCEPTION 'people.directory.invalid_provenance'; END IF;
 SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=NEW.org_id AND x.command_id=NEW.native_command_id;
 t:=public.native_people_frame_v1(NEW.org_id,NEW.native_command_id);
 SELECT o.name INTO STRICT company_name FROM public.organizations o WHERE o.id=NEW.org_id;
 IF NOT public.native_people_employee_shape_v1(NEW,i,t) OR convert_to(NEW.company,'UTF8') IS DISTINCT FROM convert_to(company_name,'UTF8') THEN
  RAISE EXCEPTION 'people.directory.invalid_native_employee';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_canonical_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE row_json jsonb; i public.native_people_inputs_v1; t public.native_people_terminals_v1;
 org uuid; command uuid; person uuid; digest bytea; expected jsonb;
BEGIN
 row_json:=CASE WHEN TG_OP='DELETE' THEN to_jsonb(OLD) ELSE to_jsonb(NEW) END;
 org:=(row_json->>'org_id')::uuid;
 IF TG_TABLE_NAME='ont_action_command_receipts' THEN
  command:=(row_json->>'command_id')::uuid;
  -- Every shared-receipt writer respects accepted native command reservations.
  PERFORM pg_advisory_xact_lock(hashtextextended('console.people.directory.command/1:'||org::text||':'||command::text,0));
  SELECT x.* INTO i FROM public.native_people_inputs_v1 x WHERE x.org_id=org AND x.command_id=command;
 ELSE
  person:=(row_json->>CASE WHEN TG_TABLE_NAME='persons' THEN 'id' ELSE 'person_id' END)::uuid;
  SELECT x.* INTO i FROM public.native_people_inputs_v1 x WHERE x.org_id=org AND x.employee_id=person;
  IF TG_TABLE_NAME='employee_person_bindings' AND i.command_id IS NULL THEN
   SELECT x.* INTO i FROM public.native_people_inputs_v1 x WHERE x.org_id=org AND x.employee_id=(row_json->>'employee_id')::uuid;
  END IF;
 END IF;
 IF i.command_id IS NULL THEN
  IF TG_TABLE_NAME<>'persons' AND row_json->>'actor_kind' IS DISTINCT FROM 'USER' THEN
   RAISE EXCEPTION 'people.directory.effect_frame_required';
  END IF;
  IF TG_OP='DELETE' THEN RETURN OLD; ELSE RETURN NEW; END IF;
 END IF;
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'people.directory.native_canonical_immutable'; END IF;
 t:=public.native_people_frame_v1(org,i.command_id); digest:=public.native_people_effect_digest_v1(i);
 IF (row_json->>'created_at')::timestamptz IS DISTINCT FROM t.terminal_at THEN RAISE EXCEPTION 'people.directory.canonical_shape_invalid'; END IF;
 IF TG_TABLE_NAME='persons' THEN RETURN NEW; END IF;
 IF row_json->>'actor_kind' IS DISTINCT FROM 'ACCOUNT' OR row_json->'actor_id' IS DISTINCT FROM 'null'::jsonb
  OR (row_json->>'actor_account_id')::uuid IS DISTINCT FROM i.actor_account_id
  OR NEW.payload_digest IS DISTINCT FROM digest THEN
  RAISE EXCEPTION 'people.directory.canonical_actor_invalid';
 END IF;
 IF TG_TABLE_NAME='employee_person_bindings' THEN
  IF (row_json->>'employee_id')::uuid IS DISTINCT FROM i.employee_id OR person IS DISTINCT FROM i.employee_id THEN RAISE EXCEPTION 'people.directory.canonical_shape_invalid'; END IF;
 ELSE
  IF (row_json->>'command_id')::uuid IS DISTINCT FROM i.command_id
   OR row_json->'receipt' IS DISTINCT FROM public.native_people_result_v1(i) THEN RAISE EXCEPTION 'people.directory.canonical_shape_invalid'; END IF;
  IF TG_TABLE_NAME='person_revisions' THEN
   IF (row_json->>'version')::bigint IS DISTINCT FROM 1 OR row_json->'attributes' IS DISTINCT FROM jsonb_build_object('legal_name',i.legal_name) THEN
    RAISE EXCEPTION 'people.directory.canonical_shape_invalid';
   END IF;
  ELSIF TG_TABLE_NAME='ont_action_command_receipts' THEN
   IF row_json->>'owner' IS DISTINCT FROM 'person' OR row_json->>'target' IS DISTINCT FROM 'people.create_person'
    OR row_json->>'action_key' IS DISTINCT FROM 'directory_create'
    OR (row_json->>'object_type_id')::uuid IS DISTINCT FROM i.expected_object_type_id THEN
    RAISE EXCEPTION 'people.directory.canonical_shape_invalid';
   END IF;
  ELSE RAISE EXCEPTION 'people.directory.invalid_guard_target'; END IF;
 END IF;
 RETURN NEW;
END
$body$;

-- All insert/number-change paths use the same historical key, exact bytes and a
-- fresh VOLATILE READ COMMITTED read after waiting. Historical duplicates remain
-- visible; legacy import duplicates remain permitted unless a native row reserves
-- that number. Native/console writes reject every competing row. No UNIQUE backfill.
CREATE OR REPLACE FUNCTION public.console_employee_number_unique() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE old_key bigint; new_key bigint;
BEGIN
 IF TG_OP='UPDATE' AND NEW.org_id=OLD.org_id AND convert_to(NEW.employee_number,'UTF8') IS NOT DISTINCT FROM convert_to(OLD.employee_number,'UTF8') THEN RETURN NEW; END IF;
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'people.directory.number_isolation_unsupported';
 END IF;
 IF TG_OP='UPDATE' AND OLD.employee_number IS NOT NULL THEN old_key:=hashtext(OLD.org_id::text||':'||OLD.employee_number); END IF;
 IF NEW.employee_number IS NOT NULL THEN new_key:=hashtext(NEW.org_id::text||':'||NEW.employee_number); END IF;
 IF old_key IS NOT NULL AND new_key IS NOT NULL THEN
  PERFORM pg_advisory_xact_lock(least(old_key,new_key));
  IF old_key<>new_key THEN PERFORM pg_advisory_xact_lock(greatest(old_key,new_key)); END IF;
 ELSIF old_key IS NOT NULL THEN PERFORM pg_advisory_xact_lock(old_key);
 ELSIF new_key IS NOT NULL THEN PERFORM pg_advisory_xact_lock(new_key); END IF;
 IF NEW.employee_number IS NOT NULL AND EXISTS(SELECT 1 FROM public.employees e
  WHERE e.org_id=NEW.org_id AND e.employee_number COLLATE "C"=NEW.employee_number COLLATE "C" AND e.id<>NEW.id
   AND (NEW.source_kind='NATIVE_DIRECTORY' OR NEW.source_filename='console' OR e.source_kind='NATIVE_DIRECTORY')) THEN
  RAISE EXCEPTION USING ERRCODE='23505',MESSAGE='employee number already exists in this organization';
 END IF;
 RETURN NEW;
END
$body$;

-- Applies independently of deferred closure, including effects attempted AFTER
-- SET CONSTRAINTS ALL IMMEDIATE. Native employment admission is a future owning
-- transition; legacy/import writers cannot fabricate it from directory identity.
CREATE FUNCTION public.native_people_non_directory_effect_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE selected_org uuid; selected_employee uuid;
BEGIN
 IF TG_OP<>'INSERT' THEN
  IF EXISTS(SELECT 1 FROM public.native_people_inputs_v1 i WHERE i.org_id=OLD.org_id AND i.employee_id=OLD.employee_id) THEN
   RAISE EXCEPTION 'people.directory.non_directory_owner_required';
  END IF;
 END IF;
 IF TG_OP<>'DELETE' THEN
  IF EXISTS(SELECT 1 FROM public.native_people_inputs_v1 i WHERE i.org_id=NEW.org_id AND i.employee_id=NEW.employee_id) THEN
   RAISE EXCEPTION 'people.directory.non_directory_owner_required';
  END IF;
  RETURN NEW;
 END IF;
 RETURN OLD;
END
$body$;

-- source: ops/native-people-directory/closure-v1.sql
-- UNEXECUTED exact protected audit frames and deferred new-command closure.
CREATE FUNCTION public.native_people_accept_snapshot_v1(i public.native_people_inputs_v1)
RETURNS jsonb LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT jsonb_build_object('protocol','NATIVE_PEOPLE_DIRECTORY_V1','command_id',i.command_id::text,
  'intake_receipt_id',i.intake_receipt_id::text,'input_digest',encode(i.input_digest,'hex'),'session_id',i.accepting_session_id::text)
$body$;
CREATE FUNCTION public.native_people_terminal_snapshot_v1(t public.native_people_terminals_v1)
RETURNS jsonb LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp AS $body$
 SELECT jsonb_build_object('protocol','NATIVE_PEOPLE_DIRECTORY_V1','command_id',t.command_id::text,
  'intake_receipt_id',t.intake_receipt_id::text,'input_digest',encode(t.input_digest,'hex'),'session_id',t.execution_session_id::text,
  'outcome',t.outcome,'result_code',t.result_code,'employee_id',t.employee_id::text,
  'person_id',t.person_id::text,'canonical_command_id',t.canonical_command_id::text)
$body$;

CREATE FUNCTION public.native_people_audit_material_v1(p_actor uuid,p_family uuid,p_org uuid,p_command uuid,p_action text)
RETURNS TABLE(target_type text,target_id text,occurred_at timestamptz,payload jsonb)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true); i public.native_people_inputs_v1; t public.native_people_terminals_v1;
BEGIN
 PERFORM public.native_people_current_v1(p_actor,p_family,p_org);
 PERFORM set_config('app.current_org',p_org::text,true);
 IF p_action='people.directory.prepare' THEN
  SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
  IF (i.actor_account_id,i.accepting_session_id,i.acceptance_xid,i.acceptance_backend_pid)
   IS DISTINCT FROM (p_actor,p_family,pg_current_xact_id(),pg_backend_pid()) THEN RAISE EXCEPTION 'people.directory.audit_frame_invalid'; END IF;
  target_type:='native_people_inputs_v1'; target_id:=i.intake_receipt_id::text; occurred_at:=i.accepted_at;
  payload:=public.native_people_accept_snapshot_v1(i);
 ELSIF p_action='people.directory.register' THEN
  SELECT x.* INTO STRICT t FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
  IF (t.actor_account_id,t.execution_session_id,t.effect_xid,t.effect_backend_pid)
   IS DISTINCT FROM (p_actor,p_family,pg_current_xact_id(),pg_backend_pid()) THEN RAISE EXCEPTION 'people.directory.audit_frame_invalid'; END IF;
  target_type:='native_people_terminals_v1'; target_id:=t.command_id::text; occurred_at:=t.terminal_at;
  payload:=public.native_people_terminal_snapshot_v1(t);
 ELSE RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='people.directory.invalid_audit_action'; END IF;
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NEXT;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

CREATE FUNCTION public.native_people_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE frame record;
BEGIN
 IF NEW.action NOT IN ('people.directory.prepare','people.directory.register') THEN RETURN NEW; END IF;
 SELECT * INTO STRICT frame FROM public.native_people_audit_material_v1(NEW.actor,
  (NEW.after_snap->>'session_id')::uuid,NEW.org_id,(NEW.after_snap->>'command_id')::uuid,NEW.action);
 IF (NEW.target_type,NEW.target_id,NEW.occurred_at,NEW.after_snap) IS DISTINCT FROM
  (frame.target_type,frame.target_id,frame.occurred_at,frame.payload)
  OR NEW.before_snap IS NOT NULL OR NEW.branch_id IS NOT NULL
  OR NEW.trace_id IS NULL OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
  OR NEW.span_id IS NULL OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
  RAISE EXCEPTION 'people.directory.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_input_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE m jsonb;
BEGIN
 IF TG_OP<>'INSERT' OR TG_TABLE_NAME<>'native_people_inputs_v1'
  OR NEW.acceptance_xid<>pg_current_xact_id() OR NEW.acceptance_backend_pid<>pg_backend_pid()
  OR NEW.accepted_at>clock_timestamp() OR clock_timestamp()>=NEW.execution_not_after
  OR NEW.input_bytes IS DISTINCT FROM public.native_people_encode_v1(NEW)
  OR NEW.input_digest IS DISTINCT FROM sha256(NEW.input_bytes) THEN RAISE EXCEPTION 'people.directory.input_frame_invalid'; END IF;
 PERFORM 1 FROM public.native_people_decode_v1(NEW.codec_version,NEW.input_bytes);
 m:=public.native_people_current_v1(NEW.actor_account_id,NEW.accepting_session_id,NEW.org_id);
 IF NOT public.native_people_expectations_match_v1(NEW,m)
  OR (NEW.accepting_assignment_id,NEW.accepting_assignment_revision)
   IS DISTINCT FROM ((m->>'assignment_id')::uuid,(m->>'assignment_revision')::bigint) THEN
  RAISE EXCEPTION 'people.directory.input_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_terminal_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE i public.native_people_inputs_v1; m jsonb;
BEGIN
 IF TG_OP<>'INSERT' OR TG_TABLE_NAME<>'native_people_terminals_v1'
  OR NEW.effect_xid<>pg_current_xact_id() OR NEW.effect_backend_pid<>pg_backend_pid() THEN RAISE EXCEPTION 'people.directory.terminal_frame_invalid'; END IF;
 SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=NEW.org_id AND x.command_id=NEW.command_id;
 IF i.acceptance_xid=pg_current_xact_id() OR NEW.terminal_at<i.accepted_at OR NEW.terminal_at>clock_timestamp()
  OR (NEW.outcome='EXPIRED' AND NEW.terminal_at<i.execution_not_after)
  OR (NEW.outcome<>'EXPIRED' AND (NEW.terminal_at>=i.execution_not_after OR clock_timestamp()>=i.execution_not_after)) THEN
  RAISE EXCEPTION 'people.directory.terminal_frame_invalid';
 END IF;
 m:=public.native_people_current_v1(NEW.actor_account_id,NEW.execution_session_id,NEW.org_id);
 IF (NEW.source_company_epoch,NEW.source_policy_receipt_id,NEW.source_assignment_id,NEW.source_assignment_revision,NEW.source_valid_from,NEW.source_valid_until)
  IS DISTINCT FROM ((m->>'company_epoch')::bigint,(m->>'current_policy_receipt_id')::uuid,(m->>'assignment_id')::uuid,
    (m->>'assignment_revision')::bigint,(m->>'assignment_valid_from')::timestamptz,(m->>'assignment_valid_until')::timestamptz)
  OR (NEW.outcome='COMMITTED' AND NOT public.native_people_expectations_match_v1(i,m)) THEN
  RAISE EXCEPTION 'people.directory.terminal_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_people_assert_closed_v1(p_org uuid,p_command uuid,p_stage text) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE i public.native_people_inputs_v1; t public.native_people_terminals_v1; m jsonb;
 e public.employees; digest bytea; result jsonb; frame record; actor_value uuid; family_value uuid; action_value text;
BEGIN
 SELECT x.* INTO STRICT i FROM public.native_people_inputs_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
 IF p_stage='PREPARE' THEN
  IF i.acceptance_xid<>pg_current_xact_id() OR i.acceptance_backend_pid<>pg_backend_pid() THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
  actor_value:=i.actor_account_id; family_value:=i.accepting_session_id; action_value:='people.directory.prepare';
  IF clock_timestamp()>=i.execution_not_after OR EXISTS(SELECT 1 FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command) THEN
   RAISE EXCEPTION 'people.directory.closure_invalid';
  END IF;
  m:=public.native_people_current_v1(actor_value,family_value,p_org);
  IF NOT public.native_people_expectations_match_v1(i,m) THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
 ELSIF p_stage='TERMINAL' THEN
  SELECT x.* INTO STRICT t FROM public.native_people_terminals_v1 x WHERE x.org_id=p_org AND x.command_id=p_command;
  IF t.effect_xid<>pg_current_xact_id() OR t.effect_backend_pid<>pg_backend_pid() THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
  actor_value:=t.actor_account_id; family_value:=t.execution_session_id; action_value:='people.directory.register';
  m:=public.native_people_current_v1(actor_value,family_value,p_org);
  IF (t.source_company_epoch,t.source_policy_receipt_id,t.source_assignment_id,t.source_assignment_revision,t.source_valid_from,t.source_valid_until)
   IS DISTINCT FROM ((m->>'company_epoch')::bigint,(m->>'current_policy_receipt_id')::uuid,(m->>'assignment_id')::uuid,
    (m->>'assignment_revision')::bigint,(m->>'assignment_valid_from')::timestamptz,(m->>'assignment_valid_until')::timestamptz)
   OR (t.outcome<>'EXPIRED' AND clock_timestamp()>=i.execution_not_after) THEN RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
 ELSE RAISE EXCEPTION 'people.directory.closure_invalid'; END IF;
 SELECT * INTO STRICT frame FROM public.native_people_audit_material_v1(actor_value,family_value,p_org,p_command,action_value);
 IF (SELECT count(*) FROM public.audit_events a WHERE a.org_id=p_org AND a.action=action_value AND a.target_id=frame.target_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.audit_events a WHERE a.org_id=p_org AND a.actor=actor_value AND a.action=action_value
   AND a.target_type=frame.target_type AND a.target_id=frame.target_id AND a.occurred_at=frame.occurred_at
   AND a.before_snap IS NULL AND a.branch_id IS NULL AND a.after_snap=frame.payload AND a.xmin=pg_current_xact_id()::xid) THEN
  RAISE EXCEPTION 'people.directory.audit_closure_invalid';
 END IF;
 -- Prohibit registration from fabricating Employment/profile/lifecycle/leave facts.
 IF EXISTS(SELECT 1 FROM public.employee_employment_profiles x WHERE x.org_id=p_org AND x.employee_id=i.employee_id)
  OR EXISTS(SELECT 1 FROM public.employee_lifecycle_events x WHERE x.org_id=p_org AND x.employee_id=i.employee_id)
  OR EXISTS(SELECT 1 FROM public.employment_source_bindings x WHERE x.org_id=p_org AND x.employee_id=i.employee_id)
  OR EXISTS(SELECT 1 FROM public.employment_revisions x WHERE x.org_id=p_org AND x.command_id=p_command)
  OR EXISTS(SELECT 1 FROM public.leave_balance_import_receipts x WHERE x.org_id=p_org AND x.employee_id=i.employee_id) THEN
  RAISE EXCEPTION 'people.directory.prohibited_effect';
 END IF;
 IF p_stage='PREPARE' OR t.outcome<>'COMMITTED' THEN
  IF EXISTS(SELECT 1 FROM public.employees x WHERE x.org_id=p_org AND (x.id=i.employee_id OR x.native_command_id=p_command))
   OR EXISTS(SELECT 1 FROM public.persons x WHERE x.org_id=p_org AND x.id=i.employee_id)
   OR EXISTS(SELECT 1 FROM public.person_revisions x WHERE x.org_id=p_org AND (x.person_id=i.employee_id OR (x.command_id=p_command AND x.actor_kind='ACCOUNT')))
   OR EXISTS(SELECT 1 FROM public.employee_person_bindings x WHERE x.org_id=p_org AND (x.employee_id=i.employee_id OR x.person_id=i.employee_id))
   OR EXISTS(SELECT 1 FROM public.ont_action_command_receipts x WHERE x.org_id=p_org AND x.command_id=p_command AND x.actor_kind='ACCOUNT') THEN
   RAISE EXCEPTION 'people.directory.unexpected_effect';
  END IF;
  RETURN;
 END IF;
 PERFORM public.native_people_frame_v1(p_org,p_command); digest:=public.native_people_effect_digest_v1(i); result:=public.native_people_result_v1(i);
 SELECT (jsonb_populate_record(NULL::public.employees,to_jsonb(projected))).* INTO STRICT e
 FROM (SELECT x.id,x.org_id,x.company,x.name,x.employee_number,x.source_kind,x.native_command_id,x.source_key,x.raw_row,x.source_metadata,x.created_at,x.updated_at,x.employment_status,x.identity_resolution_strategy,x.identity_resolution_confidence,x.identity_review_required,x.identity_name_only_merge,x.source_filename,x.source_sheet,x.source_row,x.home_branch_id,x.hire_date,x.exit_date,x.org_unit,x.job,x.position,x.worksite_name,x.worksite_address,x.leave_accrued,x.leave_used,x.leave_remaining
 FROM public.employees x WHERE x.org_id=p_org AND x.id=i.employee_id AND x.xmin=pg_current_xact_id()::xid) projected;
 IF NOT public.native_people_employee_shape_v1(e,i,t)
  OR (SELECT count(*) FROM public.employees x WHERE x.org_id=p_org AND x.native_command_id=p_command)<>1
  OR NOT EXISTS(SELECT 1 FROM public.persons x WHERE x.org_id=p_org AND x.id=i.employee_id AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid)
  OR (SELECT count(*) FROM public.person_revisions x WHERE x.org_id=p_org AND x.person_id=i.employee_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.person_revisions x WHERE x.org_id=p_org AND x.person_id=i.employee_id AND x.version=1
   AND x.command_id=p_command AND x.actor_kind='ACCOUNT' AND x.actor_id IS NULL AND x.actor_account_id=i.actor_account_id
   AND x.payload_digest=digest AND x.attributes=jsonb_build_object('legal_name',i.legal_name) AND x.receipt=result
   AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid)
  OR (SELECT count(*) FROM public.employee_person_bindings x WHERE x.org_id=p_org AND x.person_id=i.employee_id)<>1
  OR NOT EXISTS(SELECT 1 FROM public.employee_person_bindings x WHERE x.org_id=p_org AND x.employee_id=i.employee_id AND x.person_id=i.employee_id
   AND x.actor_kind='ACCOUNT' AND x.actor_id IS NULL AND x.actor_account_id=i.actor_account_id AND x.payload_digest=digest
   AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid)
  OR NOT EXISTS(SELECT 1 FROM public.ont_action_command_receipts x WHERE x.org_id=p_org AND x.command_id=p_command
   AND x.actor_kind='ACCOUNT' AND x.actor_id IS NULL AND x.actor_account_id=i.actor_account_id AND x.payload_digest=digest
   AND x.owner='person' AND x.target='people.create_person' AND x.action_key='directory_create' AND x.object_type_id=i.expected_object_type_id
   AND x.receipt=result AND x.created_at=t.terminal_at AND x.xmin=pg_current_xact_id()::xid) THEN
  RAISE EXCEPTION 'people.directory.effect_closure_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_people_deferred_closure_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on AS $body$
DECLARE prior_org text:=current_setting('app.current_org',true);
BEGIN
 PERFORM set_config('app.current_org',NEW.org_id::text,true);
 PERFORM public.native_people_assert_closed_v1(NEW.org_id,NEW.command_id,
  CASE TG_TABLE_NAME WHEN 'native_people_inputs_v1' THEN 'PREPARE' WHEN 'native_people_terminals_v1' THEN 'TERMINAL' END);
 PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RETURN NULL;
EXCEPTION WHEN OTHERS THEN PERFORM set_config('app.current_org',coalesce(prior_org,''),true); RAISE;
END
$body$;

-- source: ops/native-people-directory/legacy-import-v1.sql
-- Exact0166 owner body plus native-target refusals before both replay shortcuts.
-- Preserve console_leave_definer ownership, ACL, search_path and original byte semantics.
CREATE OR REPLACE FUNCTION leave_api.apply_employee_import_batch(
    p_org_id UUID, p_run_id UUID, p_source_ref TEXT, p_rows JSONB,
    p_actor UUID, p_apply_audit JSONB, p_trace_id TEXT, p_span_id TEXT
) RETURNS TABLE(report JSONB, replayed BOOLEAN)
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = pg_catalog SET row_security = on AS $$
DECLARE
    v_run public.data_import_runs%ROWTYPE;
    v_row JSONB;
    v_employee public.employees%ROWTYPE;
    v_employee_id UUID;
    v_employee_updated_at TIMESTAMPTZ;
    v_balance_result RECORD;
    v_source_key TEXT;
    v_company TEXT;
    v_name TEXT;
    v_idempotency_key TEXT;
    v_identity_strategy TEXT;
    v_identity_confidence TEXT;
    v_identity_review_required BOOLEAN;
    v_outcome TEXT;
    v_outcomes JSONB := '[]'::JSONB;
    v_report JSONB;
    v_input_rows INTEGER;
BEGIN
    PERFORM leave_api.assert_context(p_org_id,p_actor,p_trace_id,p_span_id);
    PERFORM leave_api.assert_employee_importer(p_org_id,p_actor);
    IF char_length(pg_catalog.btrim(p_source_ref)) NOT BETWEEN 1 AND 256
       OR jsonb_typeof(p_rows) IS DISTINCT FROM 'array' THEN
        RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.invalid_payload';
    END IF;
    v_input_rows := jsonb_array_length(p_rows);

    -- APPLIED-run replay below skips per-row processing; current native targets
    -- still refuse rather than returning a success-shaped import summary.
    IF EXISTS (
        SELECT 1 FROM jsonb_array_elements(p_rows) item
        JOIN public.employees e ON e.org_id=p_org_id
          AND e.source_key=pg_catalog.btrim(item->>'source_key')
        WHERE e.source_kind='NATIVE_DIRECTORY'
    ) THEN
        RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='employee_import_batch.native_directory_target';
    END IF;

    IF p_run_id IS NOT NULL THEN
        SELECT * INTO v_run
          FROM public.data_import_runs r
         WHERE r.org_id=p_org_id AND r.id=p_run_id
         FOR UPDATE;
        IF NOT FOUND OR v_run.entity_type <> 'employee_hr' THEN
            RAISE EXCEPTION USING ERRCODE='P0001', MESSAGE='employee_import_batch.run_not_found';
        END IF;
        IF v_run.status = 'APPLIED' THEN
            SELECT v_run.apply_summary || pg_catalog.jsonb_build_object(
                'inserted',0,
                'updated',0,
                'skipped',coalesce((v_run.apply_summary->>'input_rows')::INTEGER,0),
                'companies',coalesce((
                    SELECT jsonb_agg(item || pg_catalog.jsonb_build_object(
                        'inserted',0,
                        'updated',0,
                        'skipped',coalesce((item->>'input_rows')::INTEGER,0)
                    ) ORDER BY item->>'company')
                    FROM jsonb_array_elements(
                        coalesce(v_run.apply_summary->'companies','[]'::JSONB)
                    ) item
                ),'[]'::JSONB)
            ) INTO v_report;
            RETURN QUERY SELECT v_report, true;
            RETURN;
        END IF;
        IF v_run.status <> 'DRY_RUN' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.run_not_dry_run';
        END IF;
        IF p_source_ref <> 'run:' || p_run_id::TEXT
           OR v_input_rows <> v_run.candidate_rows
           OR (SELECT count(DISTINCT item->>'source_key')
                 FROM jsonb_array_elements(p_rows) item) <> v_run.candidate_rows
           OR EXISTS (
                SELECT 1
                  FROM jsonb_array_elements(p_rows) item
                 WHERE NOT EXISTS (
                    SELECT 1 FROM public.data_import_rows ir
                     WHERE ir.org_id=p_org_id AND ir.run_id=p_run_id
                       AND ir.row_status='CANDIDATE'
                       AND ir.source_key=item->>'source_key'
                 )
           )
           OR EXISTS (
                SELECT 1
                  FROM jsonb_array_elements(p_rows) item
                  JOIN public.data_import_rows ir
                    ON ir.org_id=p_org_id
                   AND ir.run_id=p_run_id
                   AND ir.row_status='CANDIDATE'
                   AND ir.source_key=item->>'source_key'
                 CROSS JOIN LATERAL (
                    SELECT CASE
                        WHEN ir.canonical_row#>>'{source_metadata,identity_resolution,strategy}'
                             IN ('employee_number','legal_identifier_hash',
                                 'birth_hire_fingerprint','source_row_fingerprint')
                        THEN ir.canonical_row#>>'{source_metadata,identity_resolution,strategy}'
                        ELSE 'source_row_fingerprint'
                    END AS strategy
                 ) identity
                 WHERE item IS DISTINCT FROM (
                    ir.canonical_row || pg_catalog.jsonb_build_object(
                        'raw_row',ir.raw_row,
                        'identity',pg_catalog.jsonb_build_object(
                            'strategy',identity.strategy,
                            'confidence',CASE identity.strategy
                                WHEN 'employee_number' THEN 'high'
                                WHEN 'legal_identifier_hash' THEN 'high'
                                WHEN 'birth_hire_fingerprint' THEN 'medium'
                                ELSE 'low'
                            END,
                            'review_required',NOT (
                                coalesce((ir.canonical_row#>>
                                    '{source_metadata,identity_resolution,manual_review_required}')
                                    ::BOOLEAN,true) = false
                                AND identity.strategy IN (
                                    'employee_number','legal_identifier_hash'
                                )
                            )
                        )
                    )
                 )
           ) THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.run_payload_mismatch';
        END IF;
    END IF;

    FOR v_row IN SELECT value FROM jsonb_array_elements(p_rows)
    LOOP
        v_company := pg_catalog.btrim(v_row->>'company');
        v_name := pg_catalog.btrim(v_row->>'name');
        v_source_key := pg_catalog.btrim(v_row->>'source_key');
        IF coalesce(v_company,'') = '' OR coalesce(v_name,'') = ''
           OR coalesce(v_source_key,'') = ''
           OR coalesce(pg_catalog.btrim(v_row->>'source_filename'),'') = ''
           OR coalesce(pg_catalog.btrim(v_row->>'source_sheet'),'') = ''
           OR coalesce((v_row->>'source_row')::INTEGER,0) <= 0
           OR jsonb_typeof(v_row->'raw_row') IS DISTINCT FROM 'object'
           OR jsonb_typeof(v_row->'source_metadata') IS DISTINCT FROM 'object'
           OR jsonb_typeof(v_row->'canonical') IS DISTINCT FROM 'object' THEN
            RAISE EXCEPTION USING ERRCODE='22023', MESSAGE='employee_import_batch.invalid_row';
        END IF;

        v_idempotency_key := 'employee-import:' || pg_catalog.encode(
            public.digest(p_source_ref || ':' || v_source_key,'sha256'),'hex'
        );
        SELECT * INTO v_employee
          FROM public.employees e
         WHERE e.org_id=p_org_id AND e.source_key=v_source_key
         FOR UPDATE;

        -- Native directory has no import or leave provenance. Refuse BEFORE the
        -- existing receipt replay branch, including a no-employee-DML replay.
        IF FOUND AND v_employee.source_kind='NATIVE_DIRECTORY' THEN
            RAISE EXCEPTION USING ERRCODE='42501', MESSAGE='employee_import_batch.native_directory_target';
        END IF;

        -- A prior receipt means this exact source row already committed. Call
        -- the protected command to verify its payload binding, but deliberately
        -- do not rewrite ordinary roster fields or updated_at on replay.
        IF FOUND AND EXISTS (
            SELECT 1 FROM public.leave_balance_import_receipts r
             WHERE r.org_id=p_org_id AND r.idempotency_key=v_idempotency_key
        ) THEN
            SELECT * INTO v_balance_result
              FROM leave_api.import_employee_leave_balance(
                p_org_id,v_employee.id,v_employee.updated_at,
                v_row#>>'{canonical,leave_accrued}',
                v_row#>>'{canonical,leave_used}',
                v_row#>>'{canonical,leave_remaining}',
                'employee_import',p_source_ref,v_idempotency_key,
                p_actor,p_trace_id,p_span_id
              );
            v_outcome := 'skipped';
        ELSE
            v_identity_strategy := coalesce(
                v_row#>>'{identity,strategy}','source_row_fingerprint'
            );
            v_identity_confidence := coalesce(v_row#>>'{identity,confidence}','low');
            v_identity_review_required := coalesce(
                (v_row#>>'{identity,review_required}')::BOOLEAN,true
            );
            INSERT INTO public.employees (
                org_id,company,name,source_filename,source_sheet,source_row,
                source_key,raw_row,source_metadata,employee_number,org_unit,job,
                position,worksite_name,worksite_address,hire_date,exit_date,
                employment_status,identity_resolution_strategy,
                identity_resolution_confidence,identity_review_required,
                identity_name_only_merge
            ) VALUES (
                p_org_id,v_company,v_name,v_row->>'source_filename',
                v_row->>'source_sheet',(v_row->>'source_row')::INTEGER,
                v_source_key,v_row->'raw_row',v_row->'source_metadata',
                v_row#>>'{canonical,employee_number}',
                v_row#>>'{canonical,org_unit}',v_row#>>'{canonical,job}',
                v_row#>>'{canonical,position}',v_row#>>'{canonical,worksite_name}',
                v_row#>>'{canonical,worksite_address}',v_row#>>'{canonical,hire_date}',
                v_row#>>'{canonical,exit_date}',
                coalesce(v_row#>>'{canonical,employment_status}','ACTIVE'),
                v_identity_strategy,v_identity_confidence,
                v_identity_review_required,false
            )
            ON CONFLICT (org_id,source_key) DO UPDATE SET
                company=EXCLUDED.company,name=EXCLUDED.name,
                source_filename=EXCLUDED.source_filename,
                source_sheet=EXCLUDED.source_sheet,source_row=EXCLUDED.source_row,
                raw_row=EXCLUDED.raw_row,source_metadata=EXCLUDED.source_metadata,
                employee_number=EXCLUDED.employee_number,org_unit=EXCLUDED.org_unit,
                job=EXCLUDED.job,position=EXCLUDED.position,
                worksite_name=EXCLUDED.worksite_name,
                worksite_address=EXCLUDED.worksite_address,
                hire_date=EXCLUDED.hire_date,exit_date=EXCLUDED.exit_date,
                employment_status=EXCLUDED.employment_status,
                identity_resolution_strategy=EXCLUDED.identity_resolution_strategy,
                identity_resolution_confidence=EXCLUDED.identity_resolution_confidence,
                identity_review_required=EXCLUDED.identity_review_required,
                identity_name_only_merge=false,updated_at=pg_catalog.clock_timestamp()
            RETURNING id,updated_at,
                CASE WHEN xmax=0 THEN 'inserted' ELSE 'updated' END
              INTO v_employee_id,v_employee_updated_at,v_outcome;

            SELECT * INTO v_balance_result
              FROM leave_api.import_employee_leave_balance(
                p_org_id,v_employee_id,v_employee_updated_at,
                v_row#>>'{canonical,leave_accrued}',
                v_row#>>'{canonical,leave_used}',
                v_row#>>'{canonical,leave_remaining}',
                'employee_import',p_source_ref,v_idempotency_key,
                p_actor,p_trace_id,p_span_id
              );
        END IF;
        v_outcomes := v_outcomes || pg_catalog.jsonb_build_array(
            pg_catalog.jsonb_build_object('company',v_company,'outcome',v_outcome)
        );
    END LOOP;

    SELECT pg_catalog.jsonb_build_object(
        'input_rows',v_input_rows,
        'inserted',(SELECT count(*) FROM jsonb_array_elements(v_outcomes) item
                    WHERE item->>'outcome'='inserted'),
        'updated',(SELECT count(*) FROM jsonb_array_elements(v_outcomes) item
                   WHERE item->>'outcome'='updated'),
        'skipped',(SELECT count(*) FROM jsonb_array_elements(v_outcomes) item
                   WHERE item->>'outcome'='skipped'),
        'companies',coalesce((
            SELECT jsonb_agg(pg_catalog.jsonb_build_object(
                'company',company,'input_rows',input_rows,
                'inserted',inserted,'updated',updated,'skipped',skipped
            ) ORDER BY company)
            FROM (
                SELECT item->>'company' AS company,count(*) AS input_rows,
                    count(*) FILTER (WHERE item->>'outcome'='inserted') AS inserted,
                    count(*) FILTER (WHERE item->>'outcome'='updated') AS updated,
                    count(*) FILTER (WHERE item->>'outcome'='skipped') AS skipped
                FROM jsonb_array_elements(v_outcomes) item
                GROUP BY item->>'company'
            ) companies
        ),'[]'::JSONB)
    ) INTO v_report;

    IF p_run_id IS NOT NULL THEN
        UPDATE public.data_import_runs r
           SET status='APPLIED',apply_summary=v_report,applied_by=p_actor,
               applied_at=pg_catalog.clock_timestamp(),updated_at=pg_catalog.clock_timestamp()
         WHERE r.org_id=p_org_id AND r.id=p_run_id;
    END IF;
    -- A successful first application always has intrinsic command-owned
    -- evidence, including legacy direct imports and roster-only/null-balance
    -- rows. Pure receipt/APPLIED replay is side-effect-free and reuses the
    -- original evidence rather than appending a misleading duplicate.
    IF p_run_id IS NOT NULL OR EXISTS (
        SELECT 1 FROM jsonb_array_elements(v_outcomes) item
        WHERE item->>'outcome' <> 'skipped'
    ) THEN
        INSERT INTO public.audit_events
            (actor,action,target_type,target_id,before_snap,after_snap,
             trace_id,span_id,occurred_at,org_id)
        VALUES (
            p_actor,'data_import.apply',
            CASE WHEN p_run_id IS NULL THEN 'employee_import_batch' ELSE 'data_import_run' END,
            coalesce(p_run_id::TEXT,pg_catalog.btrim(p_source_ref)),NULL,
            coalesce(p_apply_audit,'{}'::JSONB) || pg_catalog.jsonb_build_object(
                'run_id',p_run_id,'entity_type','employee_hr',
                'source_ref',pg_catalog.btrim(p_source_ref),'report',v_report
            ),p_trace_id,p_span_id,pg_catalog.statement_timestamp(),p_org_id
        );
    END IF;
    RETURN QUERY SELECT v_report,false;
END;
$$;

-- source: ops/native-people-directory/activation-v1.sql
-- UNEXECUTED privileged successor source, not standalone operator authorization.
-- Outer reviewed custody owner MUST verify exact protected predecessor+staged230,
-- take existing deployment/admission locks, and apply the complete module set in
-- ONE transaction. No historical fingerprint/profile is changed or resealed.
-- Prerequisite order: migration230, codec-v3, reviewed current-source-v2,
-- commands-v2, guards-v2, closure-audit-v2, import-replay-guard-v2, then this file.
DO $precondition$
BEGIN
 IF current_setting('server_encoding')<>'UTF8'
  OR EXISTS(SELECT 1 FROM public.native_people_inputs_v1)
  OR EXISTS(SELECT 1 FROM public.native_people_terminals_v1) THEN
  RAISE EXCEPTION 'people.directory.activation_precondition_invalid';
 END IF;
END
$precondition$;

-- The dormant numbered migration retains CHECK(false) and no runtime rights.
-- Add every Company boundary before any owner grant or staged gate is opened.
ALTER TABLE public.native_people_inputs_v1 ADD CONSTRAINT native_people_inputs_org_v1
 FOREIGN KEY(org_id) REFERENCES public.organizations(id) ON UPDATE RESTRICT ON DELETE RESTRICT;

ALTER TABLE public.native_people_inputs_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_people_terminals_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_people_inputs_v1 ADD CONSTRAINT native_people_inputs_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.native_people_terminals_v1 ADD CONSTRAINT native_people_terminal_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.person_revisions ADD CONSTRAINT person_revisions_native_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.employee_person_bindings ADD CONSTRAINT employee_person_bindings_native_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.ont_action_command_receipts ADD CONSTRAINT ont_action_receipts_native_actor_v1
 FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id) ON UPDATE RESTRICT ON DELETE RESTRICT;

ALTER TABLE public.person_revisions ADD CONSTRAINT person_revisions_actor_protocol_v1 CHECK((
 (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
 OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL)) IS TRUE);
ALTER TABLE public.employee_person_bindings ADD CONSTRAINT employee_person_bindings_actor_protocol_v1 CHECK((
 (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
 OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL)) IS TRUE);
ALTER TABLE public.ont_action_command_receipts ADD CONSTRAINT ont_action_receipts_actor_protocol_v1 CHECK((
 (actor_kind='USER' AND actor_id IS NOT NULL AND actor_account_id IS NULL)
 OR (actor_kind='ACCOUNT' AND actor_id IS NULL AND actor_account_id IS NOT NULL
  AND owner='person' AND target='people.create_person' AND action_key='directory_create' AND object_type_id IS NOT NULL)) IS TRUE);
ALTER TABLE public.employees ADD CONSTRAINT employees_provenance_protocol_v1 CHECK((
 (source_kind='LEGACY' AND native_command_id IS NULL AND source_filename IS NOT NULL AND source_sheet IS NOT NULL AND source_row IS NOT NULL
  AND btrim(source_filename)<>'' AND btrim(source_sheet)<>'' AND source_row>0)
 OR (source_kind='NATIVE_DIRECTORY' AND native_command_id IS NOT NULL AND source_filename IS NULL AND source_sheet IS NULL AND source_row IS NULL
  AND source_key='native-directory:'||native_command_id::text)) IS TRUE);

CREATE POLICY native_people_owner_v1 ON public.native_people_inputs_v1 TO console_account_owner
 USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid)
 WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
CREATE POLICY native_people_owner_v1 ON public.native_people_terminals_v1 TO console_account_owner
 USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid)
 WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
-- FORCE RLS from numbered230 remains. No runtime SELECT/INSERT/UPDATE/DELETE on
-- protected tables; reads and transitions use checked functions only.
REVOKE ALL ON public.native_people_inputs_v1,public.native_people_terminals_v1 FROM PUBLIC,console_rt,console_app;
GRANT SELECT,INSERT ON public.native_people_inputs_v1,public.native_people_terminals_v1 TO console_account_owner;

-- Exact read-only columns used by validators; no new canonical DML grants.
GRANT SELECT(id,org_id,company,name,employee_number,source_kind,native_command_id,source_key,raw_row,source_metadata,created_at,updated_at,
 employment_status,identity_resolution_strategy,identity_resolution_confidence,identity_review_required,identity_name_only_merge,
 source_filename,source_sheet,source_row,home_branch_id,hire_date,exit_date,org_unit,job,position,worksite_name,worksite_address,
 leave_accrued,leave_used,leave_remaining,xmin) ON public.employees TO console_account_owner;
GRANT SELECT(org_id,id,created_at,xmin) ON public.persons TO console_account_owner;
GRANT SELECT(org_id,id,person_id,version,command_id,actor_kind,actor_id,actor_account_id,payload_digest,attributes,receipt,created_at,xmin)
 ON public.person_revisions TO console_account_owner;
GRANT SELECT(org_id,employee_id,person_id,actor_kind,actor_id,actor_account_id,payload_digest,created_at,xmin)
 ON public.employee_person_bindings TO console_account_owner;
GRANT SELECT(org_id,command_id,actor_kind,actor_id,actor_account_id,payload_digest,receipt,created_at,owner,target,action_key,object_type_id,xmin)
 ON public.ont_action_command_receipts TO console_account_owner;
GRANT SELECT(org_id,employee_id) ON public.employee_employment_profiles,public.employee_lifecycle_events,
 public.employment_source_bindings,public.leave_balance_import_receipts TO console_account_owner;
GRANT SELECT(org_id,command_id) ON public.employment_revisions TO console_account_owner;
-- organizations and audit_events read custody is already held by existing Account
-- policy owner; no relaxation to their writers or schemas is introduced here.

CREATE TRIGGER native_people_input_frame_v1 BEFORE INSERT ON public.native_people_inputs_v1
 FOR EACH ROW EXECUTE FUNCTION public.native_people_input_guard_v1();
CREATE TRIGGER native_people_terminal_frame_v1 BEFORE INSERT ON public.native_people_terminals_v1
 FOR EACH ROW EXECUTE FUNCTION public.native_people_terminal_guard_v1();
CREATE TRIGGER native_people_employee_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employees
 FOR EACH ROW EXECUTE FUNCTION public.native_people_employee_guard_v1();
CREATE TRIGGER native_people_person_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.persons
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
CREATE TRIGGER native_people_revision_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.person_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
CREATE TRIGGER native_people_binding_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employee_person_bindings
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
CREATE TRIGGER native_people_receipt_frame_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.ont_action_command_receipts
 FOR EACH ROW EXECUTE FUNCTION public.native_people_canonical_guard_v1();
DROP TRIGGER trg_console_employee_number_unique ON public.employees;
CREATE TRIGGER trg_console_employee_number_unique BEFORE INSERT OR UPDATE OF org_id,employee_number ON public.employees
 FOR EACH ROW EXECUTE FUNCTION public.console_employee_number_unique();
CREATE TRIGGER native_people_audit_frame_v1 BEFORE INSERT ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.native_people_audit_guard_v1();
CREATE TRIGGER native_people_employee_employment_profiles_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employee_employment_profiles
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.employee_employment_profiles ENABLE ALWAYS TRIGGER native_people_employee_employment_profiles_guard_v1;
CREATE TRIGGER native_people_employee_lifecycle_events_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employee_lifecycle_events
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.employee_lifecycle_events ENABLE ALWAYS TRIGGER native_people_employee_lifecycle_events_guard_v1;
CREATE TRIGGER native_people_employment_source_bindings_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.employment_source_bindings
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.employment_source_bindings ENABLE ALWAYS TRIGGER native_people_employment_source_bindings_guard_v1;
CREATE TRIGGER native_people_leave_balance_import_receipts_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.leave_balance_import_receipts
 FOR EACH ROW EXECUTE FUNCTION public.native_people_non_directory_effect_guard_v1();
ALTER TABLE public.leave_balance_import_receipts ENABLE ALWAYS TRIGGER native_people_leave_balance_import_receipts_guard_v1;
CREATE CONSTRAINT TRIGGER native_people_input_closure_v1 AFTER INSERT ON public.native_people_inputs_v1
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_people_deferred_closure_v1();
CREATE CONSTRAINT TRIGGER native_people_terminal_closure_v1 AFTER INSERT ON public.native_people_terminals_v1
 DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_people_deferred_closure_v1();

ALTER TABLE public.native_people_inputs_v1 ENABLE ALWAYS TRIGGER native_people_input_frame_v1;
ALTER TABLE public.native_people_inputs_v1 ENABLE ALWAYS TRIGGER native_people_input_closure_v1;
ALTER TABLE public.native_people_terminals_v1 ENABLE ALWAYS TRIGGER native_people_terminal_frame_v1;
ALTER TABLE public.native_people_terminals_v1 ENABLE ALWAYS TRIGGER native_people_terminal_closure_v1;
ALTER TABLE public.employees ENABLE ALWAYS TRIGGER native_people_employee_frame_v1;
ALTER TABLE public.employees ENABLE ALWAYS TRIGGER trg_console_employee_number_unique;
ALTER TABLE public.persons ENABLE ALWAYS TRIGGER native_people_person_frame_v1;
ALTER TABLE public.person_revisions ENABLE ALWAYS TRIGGER native_people_revision_frame_v1;
ALTER TABLE public.employee_person_bindings ENABLE ALWAYS TRIGGER native_people_binding_frame_v1;
ALTER TABLE public.ont_action_command_receipts ENABLE ALWAYS TRIGGER native_people_receipt_frame_v1;
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_people_audit_frame_v1;

DO $function_acl$
DECLARE signature text; grantee_name text;
BEGIN
 FOREACH signature IN ARRAY ARRAY[
  'public.native_people_history_immutable_v1()',
  'public.native_people_text_valid_v1(text,integer)',
  'public.native_people_encode_v1(public.native_people_inputs_v1)',
  'public.native_people_decode_v1(smallint,bytea)',
  'public.native_people_effect_digest_v1(public.native_people_inputs_v1)',
  'public.native_people_result_v1(public.native_people_inputs_v1)',
  'public.native_people_expectations_match_v1(public.native_people_inputs_v1,jsonb)',
  'public.native_people_current_v1(uuid,uuid,uuid)',
  'public.native_people_prepare_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text)',
  'public.native_people_preflight_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text)',
  'public.native_people_terminal_open_v1(uuid,uuid,uuid,uuid,text)',
  'public.native_people_frame_v1(uuid,uuid)',
  'public.native_people_employee_shape_v1(public.employees,public.native_people_inputs_v1,public.native_people_terminals_v1)',
  'public.native_people_employee_guard_v1()',
  'public.native_people_canonical_guard_v1()',
  'public.native_people_non_directory_effect_guard_v1()',
  'public.console_employee_number_unique()',
  'public.native_people_accept_snapshot_v1(public.native_people_inputs_v1)',
  'public.native_people_terminal_snapshot_v1(public.native_people_terminals_v1)',
  'public.native_people_audit_material_v1(uuid,uuid,uuid,uuid,text)',
  'public.native_people_audit_guard_v1()',
  'public.native_people_input_guard_v1()',
  'public.native_people_terminal_guard_v1()',
  'public.native_people_assert_closed_v1(uuid,uuid,text)',
  'public.native_people_deferred_closure_v1()'
 ] LOOP
  EXECUTE format('ALTER FUNCTION %s OWNER TO console_account_owner',signature);
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',signature);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_catalog.pg_proc p
   CROSS JOIN LATERAL pg_catalog.aclexplode(p.proacl) a JOIN pg_catalog.pg_roles r ON r.oid=a.grantee
   WHERE p.oid=signature::regprocedure AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',signature,grantee_name); END LOOP;
 END LOOP;
END
$function_acl$;
GRANT EXECUTE ON FUNCTION
 public.native_people_prepare_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text),
 public.native_people_preflight_v1(uuid,uuid,uuid,uuid,bigint,uuid,uuid,bigint,bigint,uuid,uuid,text,text),
 public.native_people_terminal_open_v1(uuid,uuid,uuid,uuid,text),
 public.native_people_audit_material_v1(uuid,uuid,uuid,uuid,text) TO console_rt;
-- Current source already has its separately reviewed exact owner/EXECUTE ACL.
-- Existing leave import routine retains its prior console_leave_definer owner/ACL;
-- CREATE OR REPLACE changed only its pre-replay native-target refusal.

-- Open staged gates only after complete guards/closure/RLS/ACL are installed.
ALTER TABLE public.person_revisions DROP CONSTRAINT person_revisions_actor_staged_user_v1;
ALTER TABLE public.employee_person_bindings DROP CONSTRAINT employee_person_bindings_actor_staged_user_v1;
ALTER TABLE public.ont_action_command_receipts DROP CONSTRAINT ont_action_receipts_actor_staged_user_v1;
ALTER TABLE public.employees DROP CONSTRAINT employees_provenance_staged_legacy_v1;
ALTER TABLE public.native_people_inputs_v1 DROP CONSTRAINT native_people_inputs_staged_closed_v1;
ALTER TABLE public.native_people_terminals_v1 DROP CONSTRAINT native_people_terminals_staged_closed_v1;
-- Outer successor captures exact installed shape/functions/ACL/ALWAYS triggers,
-- binds resulting hashes only to the NEW profile, and commits atomically.
