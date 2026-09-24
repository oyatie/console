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
