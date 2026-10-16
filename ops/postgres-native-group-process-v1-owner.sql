-- Generated UNINSTALLED native Group source; not a custody finalizer.
-- No installed profile or serving readiness is asserted by this artifact.
-- source: ops/native-group-process/schema-v1.sql
-- Additive Group owner source. Install only as the complete reviewed custody
-- successor; this file alone is neither an installer nor serving authority.
-- Group is not a Company RLS cell. No app.current_org is used by this family.
CREATE TABLE public.native_group_process_inputs_v1 (
 actor_account_id uuid NOT NULL REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 command_id uuid NOT NULL,
 group_id uuid NOT NULL,
 group_incarnation uuid NOT NULL,
 operation smallint NOT NULL CHECK(operation IN (1,6)),
 codec_version smallint NOT NULL CHECK(codec_version=1),
 input_bytes bytea NOT NULL,
 input_digest bytea NOT NULL CHECK(input_digest=sha256(input_bytes)),
 intake_receipt_id uuid NOT NULL UNIQUE,
 accepted_at timestamptz NOT NULL CHECK(isfinite(accepted_at)),
 accepted_session_id uuid NOT NULL,
 account_security_generation bigint NOT NULL CHECK(account_security_generation>0),
 designation_receipt_id uuid NOT NULL,
 designation_revision bigint NOT NULL CHECK(designation_revision>0),
 observed_group_revision bigint NOT NULL CHECK(observed_group_revision>0),
 accepted_policy_tag smallint NOT NULL CHECK(accepted_policy_tag IN (0,1)),
 accepted_policy_revision bigint,
 accepted_policy_head_digest bytea,
 schema_id text NOT NULL CHECK(schema_id='native-group-process-v1'),
 schema_digest bytea NOT NULL CHECK(octet_length(schema_digest)=32),
 policy_digest bytea NOT NULL CHECK(octet_length(policy_digest)=32),
 codec_contract_digest bytea NOT NULL CHECK(octet_length(codec_contract_digest)=32),
 registration_manifest_version bigint NOT NULL CHECK(registration_manifest_version=1),
 registration_manifest_digest bytea NOT NULL CHECK(octet_length(registration_manifest_digest)=32),
 cedar_sdk_version text NOT NULL CHECK(octet_length(cedar_sdk_version) BETWEEN 1 AND 128 AND btrim(cedar_sdk_version)<>''),
 cedar_language_version text NOT NULL CHECK(octet_length(cedar_language_version) BETWEEN 1 AND 128 AND btrim(cedar_language_version)<>''),
 acceptance_xid xid8 NOT NULL,
 acceptance_backend_pid integer NOT NULL CHECK(acceptance_backend_pid>0),
 audit_id uuid NOT NULL UNIQUE REFERENCES public.audit_events(id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 PRIMARY KEY(actor_account_id,command_id),
 UNIQUE(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (actor_account_id,command_id,group_id,group_incarnation,intake_receipt_id,accepted_session_id,designation_receipt_id,audit_id)),
 CHECK((operation=1 AND octet_length(input_bytes) BETWEEN 188 AND 16521)
  OR (operation=6 AND octet_length(input_bytes) BETWEEN 208 AND 2255)),
 CHECK(((accepted_policy_tag=0 AND accepted_policy_revision IS NULL AND accepted_policy_head_digest IS NULL)
  OR (accepted_policy_tag=1 AND accepted_policy_revision>0 AND octet_length(accepted_policy_head_digest)=32)) IS TRUE)
);
CREATE INDEX native_group_process_inputs_v1_group_commands
 ON public.native_group_process_inputs_v1(group_id,group_incarnation,actor_account_id,command_id);

CREATE TABLE public.native_group_process_effects_v1 (
 actor_account_id uuid NOT NULL,
 command_id uuid NOT NULL,
 group_id uuid NOT NULL,
 group_incarnation uuid NOT NULL,
 operation smallint NOT NULL CHECK(operation IN (1,6)),
 input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32),
 intake_receipt_id uuid NOT NULL,
 effect_id uuid NOT NULL UNIQUE,
 result_receipt_id uuid NOT NULL UNIQUE,
 terminal_code text NOT NULL CHECK(terminal_code IN ('ADOPTED','REPLACED','SUSPENDED',
  'REJECTED_STALE_EXPECTATION','REJECTED_PROCESS_EXPIRED','REJECTED_ALREADY_SUSPENDED')),
 accepted_at timestamptz NOT NULL CHECK(isfinite(accepted_at)),
 executed_at timestamptz NOT NULL CHECK(isfinite(executed_at) AND executed_at>=accepted_at),
 execution_session_id uuid NOT NULL,
 account_security_generation bigint NOT NULL CHECK(account_security_generation>0),
 designation_receipt_id uuid NOT NULL,
 designation_revision bigint NOT NULL CHECK(designation_revision>0),
 observed_group_revision bigint NOT NULL CHECK(observed_group_revision>0),
 policy_before_tag smallint NOT NULL CHECK(policy_before_tag IN (0,1)),
 policy_before_revision bigint,
 policy_before_head_digest bytea,
 policy_after_tag smallint NOT NULL CHECK(policy_after_tag IN (0,1)),
 policy_after_revision bigint,
 policy_after_head_digest bytea,
 schema_id text NOT NULL CHECK(schema_id='native-group-process-v1'),
 schema_digest bytea NOT NULL CHECK(octet_length(schema_digest)=32),
 policy_digest bytea NOT NULL CHECK(octet_length(policy_digest)=32),
 codec_contract_digest bytea NOT NULL CHECK(octet_length(codec_contract_digest)=32),
 registration_manifest_version bigint NOT NULL CHECK(registration_manifest_version=1),
 registration_manifest_digest bytea NOT NULL CHECK(octet_length(registration_manifest_digest)=32),
 cedar_sdk_version text NOT NULL CHECK(octet_length(cedar_sdk_version) BETWEEN 1 AND 128 AND btrim(cedar_sdk_version)<>''),
 cedar_language_version text NOT NULL CHECK(octet_length(cedar_language_version) BETWEEN 1 AND 128 AND btrim(cedar_language_version)<>''),
 requested_process_id uuid NOT NULL,
 before_process_id uuid,
 before_head_revision bigint,
 before_content_version bigint,
 before_content_digest bytea,
 before_head_digest bytea,
 before_state text,
 before_expires_at timestamptz,
 after_process_id uuid,
 after_head_revision bigint,
 after_content_version bigint,
 after_content_digest bytea,
 after_head_digest bytea,
 after_state text,
 after_expires_at timestamptz,
 effect_xid xid8 NOT NULL,
 effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 effect_census jsonb NOT NULL CHECK(jsonb_typeof(effect_census)='object'),
 PRIMARY KEY(actor_account_id,command_id),
 UNIQUE(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id,effect_id,result_receipt_id),
 UNIQUE(group_id,group_incarnation,result_receipt_id),
 FOREIGN KEY(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id)
  REFERENCES public.native_group_process_inputs_v1(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (actor_account_id,command_id,group_id,group_incarnation,intake_receipt_id,effect_id,result_receipt_id,
   execution_session_id,designation_receipt_id,requested_process_id)),
 CHECK(((policy_before_tag=0 AND policy_before_revision IS NULL AND policy_before_head_digest IS NULL)
  OR (policy_before_tag=1 AND policy_before_revision>0 AND octet_length(policy_before_head_digest)=32)) IS TRUE),
 CHECK(((policy_after_tag=0 AND policy_after_revision IS NULL AND policy_after_head_digest IS NULL)
  OR (policy_after_tag=1 AND policy_after_revision>0 AND octet_length(policy_after_head_digest)=32)) IS TRUE),
 CHECK((num_nonnulls(before_process_id,before_head_revision,before_content_version,before_content_digest,before_head_digest,before_state,before_expires_at)=0
  OR (num_nonnulls(before_process_id,before_head_revision,before_content_version,before_content_digest,before_head_digest,before_state,before_expires_at)=7
   AND before_process_id<>'00000000-0000-0000-0000-000000000000'::uuid AND before_head_revision>0 AND before_content_version>0
   AND octet_length(before_content_digest)=32 AND octet_length(before_head_digest)=32
   AND before_state IN ('ACTIVE','SUSPENDED') AND isfinite(before_expires_at))) IS TRUE),
 CHECK((num_nonnulls(after_process_id,after_head_revision,after_content_version,after_content_digest,after_head_digest,after_state,after_expires_at)=0
  OR (num_nonnulls(after_process_id,after_head_revision,after_content_version,after_content_digest,after_head_digest,after_state,after_expires_at)=7
   AND after_process_id<>'00000000-0000-0000-0000-000000000000'::uuid AND after_head_revision>0 AND after_content_version>0
   AND octet_length(after_content_digest)=32 AND octet_length(after_head_digest)=32
   AND after_state IN ('ACTIVE','SUSPENDED') AND isfinite(after_expires_at))) IS TRUE),
 CHECK(((terminal_code='ADOPTED' AND operation=1 AND policy_before_tag=0 AND policy_after_tag=1
   AND policy_after_revision=1 AND before_process_id IS NULL AND after_process_id=requested_process_id
   AND after_head_revision=1 AND after_content_version=1 AND after_state='ACTIVE'
   AND effect_census='{"policy_heads":1,"versions":1,"heads":1,"head_revisions":1,"effects":1,"results":1,"audits":1}'::jsonb)
  OR (terminal_code='REPLACED' AND operation=1 AND policy_before_tag=1 AND policy_after_tag=1
   AND (policy_before_revision,policy_before_head_digest)=(policy_after_revision,policy_after_head_digest)
   AND before_process_id=requested_process_id AND after_process_id=requested_process_id
   AND after_head_revision::numeric=before_head_revision::numeric+1 AND after_content_version::numeric=before_content_version::numeric+1
   AND after_state='ACTIVE'
   AND effect_census='{"policy_heads":0,"versions":1,"heads":1,"head_revisions":1,"effects":1,"results":1,"audits":1}'::jsonb)
  OR (terminal_code='SUSPENDED' AND operation=6 AND policy_before_tag=1 AND policy_after_tag=1
   AND (policy_before_revision,policy_before_head_digest)=(policy_after_revision,policy_after_head_digest)
   AND before_process_id=requested_process_id AND after_process_id=requested_process_id AND before_state='ACTIVE' AND after_state='SUSPENDED'
   AND after_head_revision::numeric=before_head_revision::numeric+1
   AND (after_content_version,after_content_digest,after_expires_at)=(before_content_version,before_content_digest,before_expires_at)
   AND effect_census='{"policy_heads":0,"versions":0,"heads":1,"head_revisions":1,"effects":1,"results":1,"audits":1}'::jsonb)
  OR (terminal_code IN ('REJECTED_STALE_EXPECTATION','REJECTED_PROCESS_EXPIRED','REJECTED_ALREADY_SUSPENDED')
   AND (policy_before_tag,policy_before_revision,policy_before_head_digest) IS NOT DISTINCT FROM (policy_after_tag,policy_after_revision,policy_after_head_digest)
   AND (before_process_id,before_head_revision,before_content_version,before_content_digest,before_head_digest,before_state,before_expires_at)
    IS NOT DISTINCT FROM (after_process_id,after_head_revision,after_content_version,after_content_digest,after_head_digest,after_state,after_expires_at)
   AND effect_census='{"policy_heads":0,"versions":0,"heads":0,"head_revisions":0,"effects":1,"results":1,"audits":1}'::jsonb)) IS TRUE)
);

-- LIKE copies the closed CHECK grammar, never the effect's indexes or grants.
-- Separate reciprocal composite FKs and the deferred owner census bind both.
CREATE TABLE public.native_group_process_results_v1 (
 LIKE public.native_group_process_effects_v1 INCLUDING CONSTRAINTS,
 layout_version smallint NOT NULL CHECK(layout_version=2),
 result_bytes bytea NOT NULL CHECK(octet_length(result_bytes) BETWEEN 300 AND 2048),
 result_digest bytea NOT NULL CHECK(result_digest=sha256(result_bytes)),
 audit_id uuid NOT NULL UNIQUE REFERENCES public.audit_events(id) ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 PRIMARY KEY(actor_account_id,command_id),
 UNIQUE(result_receipt_id),
 UNIQUE(group_id,group_incarnation,result_receipt_id),
 UNIQUE(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id,effect_id,result_receipt_id),
 FOREIGN KEY(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id,effect_id,result_receipt_id)
  REFERENCES public.native_group_process_effects_v1(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id,effect_id,result_receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);
ALTER TABLE public.native_group_process_effects_v1
 ADD CONSTRAINT native_group_process_effects_v1_result_fk
 FOREIGN KEY(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id,effect_id,result_receipt_id)
 REFERENCES public.native_group_process_results_v1(actor_account_id,command_id,group_id,group_incarnation,operation,input_digest,intake_receipt_id,effect_id,result_receipt_id)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE public.native_group_identity_policy_heads_v1 (
 group_id uuid NOT NULL,
 group_incarnation uuid NOT NULL,
 revision bigint NOT NULL CHECK(revision=1),
 schema_id text NOT NULL CHECK(schema_id='native-group-process-v1'),
 schema_digest bytea NOT NULL CHECK(octet_length(schema_digest)=32),
 policy_digest bytea NOT NULL CHECK(octet_length(policy_digest)=32),
 codec_contract_digest bytea NOT NULL CHECK(octet_length(codec_contract_digest)=32),
 registration_manifest_version bigint NOT NULL CHECK(registration_manifest_version=1),
 registration_manifest_digest bytea NOT NULL CHECK(octet_length(registration_manifest_digest)=32),
 registered_actions jsonb NOT NULL CHECK(jsonb_typeof(registered_actions)='array' AND jsonb_array_length(registered_actions)=4),
 first_actor_account_id uuid NOT NULL,
 first_command_id uuid NOT NULL,
 first_input_digest bytea NOT NULL CHECK(octet_length(first_input_digest)=32),
 activation_receipt_id uuid NOT NULL,
 activated_at timestamptz NOT NULL CHECK(isfinite(activated_at)),
 head_digest bytea NOT NULL CHECK(octet_length(head_digest)=32),
 PRIMARY KEY(group_id,group_incarnation),
 UNIQUE(group_id,group_incarnation,revision,head_digest),
 FOREIGN KEY(group_id,group_incarnation,activation_receipt_id)
  REFERENCES public.native_group_process_results_v1(group_id,group_incarnation,result_receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (group_id,group_incarnation,first_actor_account_id,first_command_id,activation_receipt_id))
);
ALTER TABLE public.native_group_process_inputs_v1
 ADD CONSTRAINT native_group_process_inputs_v1_policy_fk FOREIGN KEY(group_id,group_incarnation,accepted_policy_revision,accepted_policy_head_digest)
 REFERENCES public.native_group_identity_policy_heads_v1(group_id,group_incarnation,revision,head_digest)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;
ALTER TABLE public.native_group_process_effects_v1
 ADD CONSTRAINT native_group_process_effects_v1_before_policy_fk FOREIGN KEY(group_id,group_incarnation,policy_before_revision,policy_before_head_digest)
 REFERENCES public.native_group_identity_policy_heads_v1(group_id,group_incarnation,revision,head_digest)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 ADD CONSTRAINT native_group_process_effects_v1_after_policy_fk FOREIGN KEY(group_id,group_incarnation,policy_after_revision,policy_after_head_digest)
 REFERENCES public.native_group_identity_policy_heads_v1(group_id,group_incarnation,revision,head_digest)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED;

CREATE TABLE public.native_group_process_versions_v1 (
 group_id uuid NOT NULL,
 group_incarnation uuid NOT NULL,
 process_id uuid NOT NULL,
 version bigint NOT NULL CHECK(version>0),
 schema_id text NOT NULL CHECK(schema_id='GROUP_VERIFIER_PROCESS_V1'),
 actor_account_id uuid NOT NULL,
 designation_receipt_id uuid NOT NULL,
 designation_revision bigint NOT NULL CHECK(designation_revision>0),
 policy_revision bigint NOT NULL CHECK(policy_revision=1),
 policy_head_digest bytea NOT NULL CHECK(octet_length(policy_head_digest)=32),
 adopt_command_id uuid NOT NULL,
 input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32),
 admitted_at timestamptz NOT NULL CHECK(isfinite(admitted_at)),
 expires_at timestamptz NOT NULL CHECK(isfinite(expires_at) AND expires_at>admitted_at AND extract(epoch FROM expires_at)-extract(epoch FROM admitted_at)<=31536000),
 operator_responsibility smallint NOT NULL CHECK(operator_responsibility=1),
 title text NOT NULL CHECK(octet_length(title) BETWEEN 1 AND 120 AND btrim(title)<>''),
 method text NOT NULL CHECK(method='ATTENDED_ACCOUNT_AND_DOCUMENTARY_REVIEW_V1'),
 intended_claimant_matching_procedure text NOT NULL CHECK(octet_length(intended_claimant_matching_procedure) BETWEEN 1 AND 2048 AND btrim(intended_claimant_matching_procedure)<>''),
 account_possession_procedure text NOT NULL CHECK(octet_length(account_possession_procedure) BETWEEN 1 AND 2048 AND btrim(account_possession_procedure)<>''),
 physical_human_evidence_procedure text NOT NULL CHECK(octet_length(physical_human_evidence_procedure) BETWEEN 1 AND 2048 AND btrim(physical_human_evidence_procedure)<>''),
 duplicate_contradictory_claim_procedure text NOT NULL CHECK(octet_length(duplicate_contradictory_claim_procedure) BETWEEN 1 AND 2048 AND btrim(duplicate_contradictory_claim_procedure)<>''),
 qualification_criteria_instruction text NOT NULL CHECK(octet_length(qualification_criteria_instruction) BETWEEN 1 AND 2048 AND btrim(qualification_criteria_instruction)<>''),
 escalation_adjudication_procedure text NOT NULL CHECK(octet_length(escalation_adjudication_procedure) BETWEEN 1 AND 2048 AND btrim(escalation_adjudication_procedure)<>''),
 evidence_minimization_retention_description text NOT NULL CHECK(octet_length(evidence_minimization_retention_description) BETWEEN 1 AND 2048 AND btrim(evidence_minimization_retention_description)<>''),
 recipient_responsibility text NOT NULL CHECK(octet_length(recipient_responsibility) BETWEEN 1 AND 2048 AND btrim(recipient_responsibility)<>''),
 content_digest bytea NOT NULL CHECK(octet_length(content_digest)=32),
 PRIMARY KEY(group_id,group_incarnation,process_id,version),
 UNIQUE(group_id,group_incarnation,process_id,version,content_digest,expires_at),
 FOREIGN KEY(group_id,group_incarnation,policy_revision,policy_head_digest)
  REFERENCES public.native_group_identity_policy_heads_v1(group_id,group_incarnation,revision,head_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(actor_account_id,adopt_command_id)
  REFERENCES public.native_group_process_inputs_v1(actor_account_id,command_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK(octet_length(title)+octet_length(method)+octet_length(intended_claimant_matching_procedure)
  +octet_length(account_possession_procedure)+octet_length(physical_human_evidence_procedure)
  +octet_length(duplicate_contradictory_claim_procedure)+octet_length(qualification_criteria_instruction)
  +octet_length(escalation_adjudication_procedure)+octet_length(evidence_minimization_retention_description)
  +octet_length(recipient_responsibility)<=16384),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (group_id,group_incarnation,process_id,actor_account_id,designation_receipt_id,adopt_command_id))
);

CREATE TABLE public.native_group_process_head_revisions_v1 (
 group_id uuid NOT NULL,
 group_incarnation uuid NOT NULL,
 process_id uuid NOT NULL,
 head_revision bigint NOT NULL CHECK(head_revision>0),
 content_version bigint NOT NULL CHECK(content_version>0),
 content_digest bytea NOT NULL CHECK(octet_length(content_digest)=32),
 state text NOT NULL CHECK(state IN ('ACTIVE','SUSPENDED')),
 expires_at timestamptz NOT NULL CHECK(isfinite(expires_at)),
 last_actor_account_id uuid NOT NULL,
 last_command_id uuid NOT NULL,
 last_input_digest bytea NOT NULL CHECK(octet_length(last_input_digest)=32),
 result_receipt_id uuid NOT NULL UNIQUE,
 updated_at timestamptz NOT NULL CHECK(isfinite(updated_at)),
 head_digest bytea NOT NULL CHECK(octet_length(head_digest)=32),
 before_head_digest bytea,
 PRIMARY KEY(group_id,group_incarnation,head_revision),
 UNIQUE(group_id,group_incarnation,process_id,head_revision,content_version,content_digest,state,expires_at,
  last_actor_account_id,last_command_id,last_input_digest,result_receipt_id,updated_at,head_digest),
 FOREIGN KEY(group_id,group_incarnation,process_id,content_version,content_digest,expires_at)
  REFERENCES public.native_group_process_versions_v1(group_id,group_incarnation,process_id,version,content_digest,expires_at)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(group_id,group_incarnation,result_receipt_id)
  REFERENCES public.native_group_process_results_v1(group_id,group_incarnation,result_receipt_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK((head_revision=1 AND before_head_digest IS NULL)
  OR (head_revision>1 AND octet_length(before_head_digest)=32) IS TRUE),
 CHECK('00000000-0000-0000-0000-000000000000'::uuid NOT IN
  (group_id,group_incarnation,process_id,last_actor_account_id,last_command_id,result_receipt_id))
);
CREATE TABLE public.native_group_process_heads_v1 (
 LIKE public.native_group_process_head_revisions_v1 INCLUDING CONSTRAINTS,
 PRIMARY KEY(group_id,group_incarnation),
 FOREIGN KEY(group_id,group_incarnation,process_id,head_revision,content_version,content_digest,state,expires_at,
  last_actor_account_id,last_command_id,last_input_digest,result_receipt_id,updated_at,head_digest)
 REFERENCES public.native_group_process_head_revisions_v1(group_id,group_incarnation,process_id,head_revision,content_version,content_digest,state,expires_at,
  last_actor_account_id,last_command_id,last_input_digest,result_receipt_id,updated_at,head_digest)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED
);

-- No FK to deletable live Group topology: immutable locators and receipts must
-- survive topology/designation loss. Current usability is checked by the owner.
ALTER TABLE public.native_group_process_inputs_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_group_process_effects_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_group_process_results_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_group_identity_policy_heads_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_group_process_versions_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_group_process_head_revisions_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_group_process_heads_v1 OWNER TO console_account_owner;
REVOKE ALL ON public.native_group_process_inputs_v1,public.native_group_process_effects_v1,
 public.native_group_process_results_v1,public.native_group_identity_policy_heads_v1,
 public.native_group_process_versions_v1,public.native_group_process_head_revisions_v1,
 public.native_group_process_heads_v1 FROM PUBLIC,console_rt,console_app;
ALTER TABLE public.native_group_process_inputs_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_inputs_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_effects_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_effects_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_results_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_results_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_identity_policy_heads_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_identity_policy_heads_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_versions_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_versions_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_head_revisions_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_head_revisions_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_heads_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_group_process_heads_v1 FORCE ROW LEVEL SECURITY;
CREATE POLICY native_group_owner_only ON public.native_group_process_inputs_v1 TO console_account_owner USING(true) WITH CHECK(true);
CREATE POLICY native_group_owner_only ON public.native_group_process_effects_v1 TO console_account_owner USING(true) WITH CHECK(true);
CREATE POLICY native_group_owner_only ON public.native_group_process_results_v1 TO console_account_owner USING(true) WITH CHECK(true);
CREATE POLICY native_group_owner_only ON public.native_group_identity_policy_heads_v1 TO console_account_owner USING(true) WITH CHECK(true);
CREATE POLICY native_group_owner_only ON public.native_group_process_versions_v1 TO console_account_owner USING(true) WITH CHECK(true);
CREATE POLICY native_group_owner_only ON public.native_group_process_head_revisions_v1 TO console_account_owner USING(true) WITH CHECK(true);
CREATE POLICY native_group_owner_only ON public.native_group_process_heads_v1 TO console_account_owner USING(true) WITH CHECK(true);

-- source: ops/native-group-process/codec-v1.sql
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

-- source: ops/native-group-process/result-codec-v2.sql
-- New Group layout1 digests and result layout2. The preallocated result receipt
-- enters heads; the result digest never enters its own inputs. No history reseal.
CREATE FUNCTION public.native_group_process_policy_ref_v1(p_tag smallint,p_revision bigint,p_digest bytea) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF p_tag=0 AND p_revision IS NULL AND p_digest IS NULL THEN RETURN decode('00','hex'); END IF;
 IF p_tag=1 AND p_revision>0 AND octet_length(p_digest)=32 THEN
  RETURN decode('01','hex')||int8send(p_revision)||p_digest;
 END IF;
 RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_policy_reference';
END
$body$;
CREATE FUNCTION public.native_group_process_head_ref_v1(p_process uuid,p_head bigint,p_version bigint,
 p_content_digest bytea,p_head_digest bytea,p_state text,p_expiry timestamptz) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
BEGIN
 IF num_nonnulls(p_process,p_head,p_version,p_content_digest,p_head_digest,p_state,p_expiry)=0 THEN
  RETURN decode('00','hex');
 END IF;
 IF (num_nonnulls(p_process,p_head,p_version,p_content_digest,p_head_digest,p_state,p_expiry)=7
  AND p_process<>'00000000-0000-0000-0000-000000000000'::uuid AND p_head>0 AND p_version>0
  AND octet_length(p_content_digest)=32 AND octet_length(p_head_digest)=32
  AND p_state IN ('ACTIVE','SUSPENDED') AND isfinite(p_expiry)) IS TRUE THEN
  RETURN decode('01','hex')||uuid_send(p_process)||int8send(p_head)||int8send(p_version)
   ||p_content_digest||p_head_digest||int2send(CASE p_state WHEN 'ACTIVE' THEN 1 ELSE 2 END::smallint)
   ||int8send(public.native_group_process_micros_v1(p_expiry));
 END IF;
 RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_head_reference';
END
$body$;

CREATE FUNCTION public.native_group_process_action_roster_v1() RETURNS jsonb
LANGUAGE sql IMMUTABLE SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$ SELECT $roster$[{"key":"identity.verifier.process.adopt/1","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","receipt_locator"]},{"key":"identity.verifier.process.suspend/1","revision":1,"fields":["process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","reason","receipt_locator"]},{"key":"identity.verifier.process.read/1","revision":1,"fields":["group_context","process_id","process_version","process_digest","process_head_revision","process_head_digest","state","expiry","title","method","intended_claimant_matching_procedure","account_possession_procedure","physical_human_evidence_procedure","duplicate_contradictory_claim_procedure","qualification_criteria_instruction","escalation_adjudication_procedure","evidence_minimization_retention_description","recipient_responsibility","history","allowed_actions"]},{"key":"identity.verifier.process.receipt.read-own/1","revision":1,"fields":["command_id","input_digest","intake_receipt_id","result_receipt_id","terminal_code","accepted_at","executed_at","process_id","before_head","after_head","original_content","original_reason","receipt_locator"]}]$roster$::jsonb $body$;

CREATE FUNCTION public.native_group_process_registration_bytes_v1(p_group uuid,p_incarnation uuid,
 p_schema text,p_schema_digest bytea,p_policy_digest bytea,p_codec_digest bytea,p_actions jsonb) RETURNS bytea
LANGUAGE plpgsql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE output bytea; item jsonb; field_value text; action_id uuid; seen uuid[]:=ARRAY[]::uuid[]; roster jsonb:=public.native_group_process_action_roster_v1(); action_index integer:=0;
BEGIN
 IF p_group='00000000-0000-0000-0000-000000000000'::uuid
  OR p_incarnation='00000000-0000-0000-0000-000000000000'::uuid
  OR p_schema<>'native-group-process-v1' OR octet_length(p_schema_digest)<>32
  OR octet_length(p_policy_digest)<>32 OR octet_length(p_codec_digest)<>32
  OR jsonb_typeof(p_actions)<>'array' OR jsonb_array_length(p_actions)<>4 THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration';
 END IF;
 output:=convert_to('CONSOLE.IDENTITY.GROUP.PROCESS.REGISTRATION','UTF8')||decode('000001','hex')
  ||uuid_send(p_group)||uuid_send(p_incarnation)||int8send(1::bigint)
  ||public.native_group_process_text_v1(p_schema,128)||p_schema_digest||p_policy_digest||p_codec_digest||int2send(4::smallint);
 FOR item IN SELECT value FROM jsonb_array_elements(p_actions) WITH ORDINALITY x(value,position) ORDER BY position LOOP
  IF jsonb_typeof(item) IS DISTINCT FROM 'object'
   OR item-'action_id' IS DISTINCT FROM roster->action_index
   OR jsonb_typeof(item->'action_id') IS DISTINCT FROM 'string'
   OR (item->>'action_id') COLLATE "C" !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration';
  END IF;
  action_index:=action_index+1;
  action_id:=(item->>'action_id')::uuid;
  IF action_id IS NULL OR action_id='00000000-0000-0000-0000-000000000000'::uuid
   OR action_id=ANY(seen) OR (item->>'revision')::bigint IS DISTINCT FROM 1
   OR jsonb_typeof(item->'fields') IS DISTINCT FROM 'array'
   OR jsonb_array_length(item->'fields') NOT BETWEEN 1 AND 64 THEN
   RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration';
  END IF;
  seen:=array_append(seen,action_id);
  output:=output||public.native_group_process_text_v1(item->>'key',128)||uuid_send(action_id)||int8send(1::bigint)
   ||int2send(jsonb_array_length(item->'fields')::smallint);
  FOR field_value IN SELECT value FROM jsonb_array_elements_text(item->'fields') WITH ORDINALITY x(value,position) ORDER BY position LOOP
   output:=output||public.native_group_process_text_v1(field_value,128);
  END LOOP;
 END LOOP;
 IF output IS NULL THEN RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_registration'; END IF;
 RETURN output;
END
$body$;

CREATE FUNCTION public.native_group_process_policy_head_bytes_v1(r public.native_group_identity_policy_heads_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.GROUP.POLICY.HEAD','UTF8')||decode('000001','hex')
  ||uuid_send(r.group_id)||uuid_send(r.group_incarnation)||int8send(r.revision)
  ||public.native_group_process_text_v1(r.schema_id,128)||r.schema_digest||r.policy_digest||r.codec_contract_digest
  ||r.registration_manifest_digest||uuid_send(r.first_actor_account_id)||uuid_send(r.first_command_id)
  ||r.first_input_digest||uuid_send(r.activation_receipt_id)||int8send(public.native_group_process_micros_v1(r.activated_at))
$body$;
CREATE FUNCTION public.native_group_process_version_bytes_v1(r public.native_group_process_versions_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.PROCESS.VERSION','UTF8')||decode('000001','hex')
  ||uuid_send(r.group_id)||uuid_send(r.group_incarnation)||uuid_send(r.process_id)||int8send(r.version)
  ||public.native_group_process_text_v1(r.schema_id,128)||uuid_send(r.actor_account_id)||uuid_send(r.designation_receipt_id)
  ||int8send(r.designation_revision)||int8send(r.policy_revision)||r.policy_head_digest||uuid_send(r.adopt_command_id)
  ||r.input_digest||int8send(public.native_group_process_micros_v1(r.admitted_at))||int8send(public.native_group_process_micros_v1(r.expires_at))
  ||int2send(r.operator_responsibility)||public.native_group_process_text_v1(r.title,120)||int2send(1::smallint)
  ||public.native_group_process_text_v1(r.intended_claimant_matching_procedure,2048)
  ||public.native_group_process_text_v1(r.account_possession_procedure,2048)
  ||public.native_group_process_text_v1(r.physical_human_evidence_procedure,2048)
  ||public.native_group_process_text_v1(r.duplicate_contradictory_claim_procedure,2048)
  ||public.native_group_process_text_v1(r.qualification_criteria_instruction,2048)
  ||public.native_group_process_text_v1(r.escalation_adjudication_procedure,2048)
  ||public.native_group_process_text_v1(r.evidence_minimization_retention_description,2048)
  ||public.native_group_process_text_v1(r.recipient_responsibility,2048)
$body$;
CREATE FUNCTION public.native_group_process_head_bytes_v1(r public.native_group_process_head_revisions_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.PROCESS.HEAD','UTF8')||decode('000001','hex')
  ||uuid_send(r.group_id)||uuid_send(r.group_incarnation)||uuid_send(r.process_id)||int8send(r.head_revision)
  ||int8send(r.content_version)||r.content_digest||int2send(CASE r.state WHEN 'ACTIVE' THEN 1 WHEN 'SUSPENDED' THEN 2 END::smallint)
  ||int8send(public.native_group_process_micros_v1(r.expires_at))||uuid_send(r.last_actor_account_id)||uuid_send(r.last_command_id)
  ||r.last_input_digest||uuid_send(r.result_receipt_id)||int8send(public.native_group_process_micros_v1(r.updated_at))
$body$;
CREATE FUNCTION public.native_group_process_result_bytes_v2(r public.native_group_process_results_v1) RETURNS bytea
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT convert_to('CONSOLE.IDENTITY.PROCESS.RESULT','UTF8')||decode('000002','hex')
  ||uuid_send(r.actor_account_id)||uuid_send(r.command_id)||uuid_send(r.group_id)||uuid_send(r.group_incarnation)
  ||int2send(r.operation)||r.input_digest||uuid_send(r.intake_receipt_id)||uuid_send(r.result_receipt_id)
  ||int2send(CASE r.terminal_code WHEN 'ADOPTED' THEN 1 WHEN 'REPLACED' THEN 2 WHEN 'SUSPENDED' THEN 3
   WHEN 'REJECTED_STALE_EXPECTATION' THEN 4 WHEN 'REJECTED_PROCESS_EXPIRED' THEN 5 WHEN 'REJECTED_ALREADY_SUSPENDED' THEN 6 END::smallint)
  ||int8send(public.native_group_process_micros_v1(r.accepted_at))||int8send(public.native_group_process_micros_v1(r.executed_at))
  ||uuid_send(r.execution_session_id)||int8send(r.account_security_generation)||uuid_send(r.designation_receipt_id)
  ||int8send(r.designation_revision)||int8send(r.observed_group_revision)
  ||public.native_group_process_policy_ref_v1(r.policy_before_tag,r.policy_before_revision,r.policy_before_head_digest)
  ||public.native_group_process_policy_ref_v1(r.policy_after_tag,r.policy_after_revision,r.policy_after_head_digest)
  ||public.native_group_process_text_v1(r.schema_id,128)||r.schema_digest||r.policy_digest||r.codec_contract_digest
  ||int8send(r.registration_manifest_version)||r.registration_manifest_digest
  ||public.native_group_process_text_v1(r.cedar_sdk_version,128)||public.native_group_process_text_v1(r.cedar_language_version,128)
  ||uuid_send(r.requested_process_id)
  ||public.native_group_process_head_ref_v1(r.before_process_id,r.before_head_revision,r.before_content_version,
   r.before_content_digest,r.before_head_digest,r.before_state,r.before_expires_at)
  ||public.native_group_process_head_ref_v1(r.after_process_id,r.after_head_revision,r.after_content_version,
   r.after_content_digest,r.after_head_digest,r.after_state,r.after_expires_at)
  ||xid8send(r.effect_xid)||int4send(r.effect_backend_pid)
$body$;

-- source: ops/native-group-process/source-v1.sql
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

-- source: ops/native-group-process/locks-v1.sql
-- Group-first guards also cover immutable locators after live topology is gone.
-- The finite topology fence is intentional: legacy writers may not change a
-- native Group while the Group process source is installed. A later canonical
-- topology writer must replace this fence with a separately reviewed protocol.
CREATE FUNCTION public.native_group_process_group_guard_v1(p_group uuid,p_write boolean) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE guard_key bigint;
BEGIN
 IF p_group IS NULL OR p_write IS NULL
  OR p_group='00000000-0000-0000-0000-000000000000'::uuid
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'native_group_process.guard_unavailable';
 END IF;
 guard_key:=hashtextextended('console.identity.group-process.group/1:'||p_group::text,0);
 IF p_write THEN PERFORM pg_advisory_xact_lock(guard_key);
 ELSE PERFORM pg_advisory_xact_lock_shared(guard_key); END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_command_guard_v1(p_actor uuid,p_command uuid,p_write boolean) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$
DECLARE guard_key bigint;
BEGIN
 IF p_actor IS NULL OR p_command IS NULL OR p_write IS NULL
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_command)
  OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
  RAISE EXCEPTION 'native_group_process.guard_unavailable';
 END IF;
 guard_key:=hashtextextended('console.identity.group-process.command/1:'||p_actor::text||':'||p_command::text,0);
 IF p_write THEN PERFORM pg_advisory_xact_lock(guard_key);
 ELSE PERFORM pg_advisory_xact_lock_shared(guard_key); END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_topology_fence_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE old_row jsonb; new_row jsonb; affected uuid;
BEGIN
 IF TG_OP='TRUNCATE' THEN
  IF EXISTS(SELECT 1 FROM public.groups g WHERE g.origin_account_id IS NOT NULL)
   OR EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_group_process.topology_fenced';
  END IF;
  RETURN NULL;
 END IF;
 IF TG_TABLE_NAME NOT IN ('groups','group_authority_heads','group_memberships','group_membership_revisions')
  OR TG_OP NOT IN ('INSERT','UPDATE','DELETE') THEN
  RAISE EXCEPTION 'native_group_process.guard_unavailable';
 END IF;
 IF TG_OP<>'INSERT' THEN old_row:=to_jsonb(OLD); END IF;
 IF TG_OP<>'DELETE' THEN new_row:=to_jsonb(NEW); END IF;
 FOR affected IN SELECT DISTINCT value FROM unnest(ARRAY[
   (old_row->>CASE TG_TABLE_NAME WHEN 'groups' THEN 'id' ELSE 'group_id' END)::uuid,
   (new_row->>CASE TG_TABLE_NAME WHEN 'groups' THEN 'id' ELSE 'group_id' END)::uuid]) value
  WHERE value IS NOT NULL ORDER BY value
 LOOP
  -- UPDATE and DELETE are denied before waiting for any new owner guard; this
  -- prevents a legacy row lock from inverting the Group-first owner order.
  IF TG_OP IN ('UPDATE','DELETE') AND
   (EXISTS(SELECT 1 FROM public.groups g WHERE g.id=affected AND g.origin_account_id IS NOT NULL)
    OR EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1 i WHERE i.group_id=affected)
    OR (TG_TABLE_NAME='groups' AND old_row->>'origin_account_id' IS NOT NULL)) THEN
   RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_group_process.topology_fenced';
  END IF;
  IF TG_OP='INSERT' AND TG_TABLE_NAME='groups' THEN
   -- Native birth holds Account already. Never wait here on a Group-first
   -- reader; abort contended B, preserving A. Successful birth retains the
   -- exact exclusive identity guard through publication and existing closure.
   IF affected='00000000-0000-0000-0000-000000000000'::uuid
    OR current_setting('transaction_isolation') IS DISTINCT FROM 'read committed' THEN
    RAISE EXCEPTION 'native_group_process.guard_unavailable';
   END IF;
   IF NOT pg_try_advisory_xact_lock(hashtextextended('console.identity.group-process.group/1:'||affected::text,0)) THEN
    RAISE EXCEPTION USING ERRCODE='55P03',MESSAGE='native_group_process.birth_guard_unavailable';
   END IF;
  ELSIF TG_OP='INSERT' AND
   EXISTS(SELECT 1 FROM public.groups g WHERE g.id=affected AND g.origin_account_id IS NOT NULL) THEN
   -- Only the original same-transaction Company birth may add its initial
   -- authority/membership rows. No legacy append may widen an existing Group.
   IF NOT EXISTS(SELECT 1 FROM public.groups g
    JOIN public.company_enrollment_effect_bindings b ON
     (b.account_id,b.command_id,b.receipt_id,b.group_id)=
     (g.origin_account_id,g.origin_command_id,g.origin_receipt_id,g.id)
    WHERE g.id=affected AND b.effect_xid=pg_current_xact_id()
     AND b.effect_backend_pid=pg_backend_pid()) THEN
    RAISE EXCEPTION USING ERRCODE='23514',MESSAGE='native_group_process.topology_fenced';
   END IF;
  END IF;
 END LOOP;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 RETURN NEW;
END
$body$;

CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.groups
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_authority_heads
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_memberships
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_fence_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.group_membership_revisions
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.groups
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.group_authority_heads
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.group_memberships
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
CREATE TRIGGER native_group_process_topology_truncate_v1 BEFORE TRUNCATE ON public.group_membership_revisions
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_topology_fence_v1();
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER native_group_process_topology_fence_v1;
ALTER TABLE public.groups ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
ALTER TABLE public.group_authority_heads ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
ALTER TABLE public.group_memberships ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;
ALTER TABLE public.group_membership_revisions ENABLE ALWAYS TRIGGER native_group_process_topology_truncate_v1;

-- source: ops/native-group-process/closure-v1.sql
-- Historical closure checks require retained Group/command guards but never
-- current designation or live topology. Own receipts survive their loss.
-- Install only with the complete independently reviewed owner successor.
CREATE FUNCTION public.native_group_process_accept_snapshot_v1(i public.native_group_process_inputs_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT jsonb_build_object('protocol','GROUP_PROCESS_V1','command_id',i.command_id::text,
  'group_id',i.group_id::text,'group_incarnation',i.group_incarnation::text,'operation',i.operation,
  'input_digest',encode(i.input_digest,'hex'),'intake_receipt_id',i.intake_receipt_id::text,
  'session_id',i.accepted_session_id::text)
$body$;

CREATE FUNCTION public.native_group_process_complete_snapshot_v1(r public.native_group_process_results_v1) RETURNS jsonb
LANGUAGE sql IMMUTABLE STRICT SECURITY INVOKER PARALLEL SAFE
SET search_path=pg_catalog,pg_temp
AS $body$
 SELECT jsonb_build_object('protocol','GROUP_PROCESS_V1','command_id',r.command_id::text,
  'group_id',r.group_id::text,'group_incarnation',r.group_incarnation::text,'operation',r.operation,
  'input_digest',encode(r.input_digest,'hex'),'intake_receipt_id',r.intake_receipt_id::text,
  'result_receipt_id',r.result_receipt_id::text,'session_id',r.execution_session_id::text,
  'terminal_code',r.terminal_code)
$body$;

CREATE FUNCTION public.native_group_process_assert_input_v1(p_actor uuid,p_command uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; decoded jsonb;
BEGIN
 SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 decoded:=public.native_group_process_decode_v1(i.input_bytes);
 IF i.codec_version<>1 OR i.input_digest IS DISTINCT FROM sha256(i.input_bytes)
  OR (decoded->>'actor_account_id',decoded->>'command_id',decoded->>'group_id',decoded->>'group_incarnation',
    (decoded->>'operation')::smallint)
   IS DISTINCT FROM (i.actor_account_id::text,i.command_id::text,i.group_id::text,i.group_incarnation::text,i.operation)
  OR NOT EXISTS(SELECT 1 FROM public.audit_events a WHERE a.id=i.audit_id
    AND a.org_id IS NULL AND a.actor=i.actor_account_id AND a.action='identity.group_process.accept'
    AND a.target_type='native_group_process_inputs_v1' AND a.target_id=i.intake_receipt_id::text
    AND a.occurred_at=i.accepted_at AND a.before_snap IS NULL AND a.branch_id IS NULL
    AND a.after_snap=public.native_group_process_accept_snapshot_v1(i))
  OR (SELECT count(*) FROM public.audit_events a WHERE a.action='identity.group_process.accept'
    AND a.target_id=i.intake_receipt_id::text)<>1 THEN
  RAISE EXCEPTION 'native_group_process.input_closure_invalid';
 END IF;
 IF i.operation=1 AND ((decoded->>'expiry_us')::numeric<=public.native_group_process_micros_v1(i.accepted_at)
  OR (decoded->>'expiry_us')::numeric-public.native_group_process_micros_v1(i.accepted_at)>31536000000000) THEN
  RAISE EXCEPTION 'native_group_process.input_closure_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_assert_result_v1(p_actor uuid,p_command uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; r public.native_group_process_results_v1;
 f public.native_group_process_effects_v1; policy public.native_group_identity_policy_heads_v1;
 activation public.native_group_process_results_v1; adoption public.native_group_process_results_v1;
 prior public.native_group_process_head_revisions_v1; successor public.native_group_process_head_revisions_v1;
 version_row public.native_group_process_versions_v1; decoded jsonb; content jsonb;
 stale boolean; expected_code text; committed boolean;
BEGIN
 PERFORM public.native_group_process_assert_input_v1(p_actor,p_command);
 SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 SELECT x.* INTO STRICT r FROM public.native_group_process_results_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 SELECT x.* INTO STRICT f FROM public.native_group_process_effects_v1 x
  WHERE x.actor_account_id=p_actor AND x.command_id=p_command;
 decoded:=public.native_group_process_decode_v1(i.input_bytes);
 IF to_jsonb(r)-ARRAY['layout_version','result_bytes','result_digest','audit_id'] IS DISTINCT FROM to_jsonb(f)
  OR r.layout_version<>2 OR r.result_bytes IS DISTINCT FROM public.native_group_process_result_bytes_v2(r)
  OR r.result_digest IS DISTINCT FROM sha256(r.result_bytes)
  OR (r.policy_before_tag=0)<>(r.before_process_id IS NULL)
  OR (r.policy_after_tag=0)<>(r.after_process_id IS NULL)
  OR (r.policy_before_tag=0 AND (r.operation<>1
    OR (decoded->>'expected_policy_revision')::bigint IS DISTINCT FROM 0
    OR (decoded->>'expected_prior_head_revision')::bigint IS DISTINCT FROM 0))
  OR (r.group_id,r.group_incarnation,r.operation,r.input_digest,r.intake_receipt_id,r.accepted_at)
   IS DISTINCT FROM (i.group_id,i.group_incarnation,i.operation,i.input_digest,i.intake_receipt_id,i.accepted_at)
  OR r.requested_process_id::text IS DISTINCT FROM decoded->>'process_id'
  OR r.effect_xid=i.acceptance_xid
  OR NOT EXISTS(SELECT 1 FROM public.audit_events a WHERE a.id=r.audit_id
    AND a.org_id IS NULL AND a.actor=r.actor_account_id AND a.action='identity.group_process.complete'
    AND a.target_type='native_group_process_results_v1' AND a.target_id=r.result_receipt_id::text
    AND a.occurred_at=r.executed_at AND a.before_snap IS NULL AND a.branch_id IS NULL
    AND a.after_snap=public.native_group_process_complete_snapshot_v1(r))
  OR (SELECT count(*) FROM public.audit_events a WHERE a.action='identity.group_process.complete'
    AND a.target_id=r.result_receipt_id::text)<>1 THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 -- Source identity is historical metadata. Current serving-source checks are
 -- separate, and never rewrite the original result or command bytes.
 IF (r.schema_id,r.schema_digest,r.policy_digest,r.codec_contract_digest,r.registration_manifest_version,
   r.registration_manifest_digest,r.cedar_sdk_version,r.cedar_language_version)
  IS DISTINCT FROM (i.schema_id,i.schema_digest,i.policy_digest,i.codec_contract_digest,i.registration_manifest_version,
   i.registration_manifest_digest,i.cedar_sdk_version,i.cedar_language_version) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 IF r.policy_after_tag=1 THEN
  SELECT x.* INTO STRICT policy FROM public.native_group_identity_policy_heads_v1 x
   WHERE x.group_id=r.group_id AND x.group_incarnation=r.group_incarnation;
  IF (policy.revision,policy.head_digest) IS DISTINCT FROM (r.policy_after_revision,r.policy_after_head_digest)
   OR policy.head_digest IS DISTINCT FROM sha256(public.native_group_process_policy_head_bytes_v1(policy))
   OR policy.registration_manifest_digest IS DISTINCT FROM sha256(public.native_group_process_registration_bytes_v1(
     policy.group_id,policy.group_incarnation,policy.schema_id,policy.schema_digest,policy.policy_digest,
     policy.codec_contract_digest,policy.registered_actions)) THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
  IF r.terminal_code<>'ADOPTED' THEN
   SELECT x.* INTO STRICT activation FROM public.native_group_process_results_v1 x
    WHERE x.actor_account_id=policy.first_actor_account_id AND x.command_id=policy.first_command_id;
   -- Check the closed terminal role before recursion: activation is always
   -- the first adoption, so corrupt self-links cannot form a recursion cycle.
   IF activation.terminal_code<>'ADOPTED' OR activation.operation<>1
    OR (activation.group_id,activation.group_incarnation,activation.result_receipt_id,activation.executed_at,
      activation.input_digest,activation.policy_after_revision,activation.policy_after_head_digest)
     IS DISTINCT FROM (policy.group_id,policy.group_incarnation,policy.activation_receipt_id,policy.activated_at,
      policy.first_input_digest,policy.revision,policy.head_digest)
    OR activation.executed_at>r.executed_at THEN
    RAISE EXCEPTION 'native_group_process.result_closure_invalid';
   END IF;
   PERFORM public.native_group_process_assert_result_v1(activation.actor_account_id,activation.command_id);
  END IF;
 END IF;
 IF r.before_process_id IS NOT NULL THEN
  SELECT x.* INTO STRICT prior FROM public.native_group_process_head_revisions_v1 x
   WHERE x.group_id=r.group_id AND x.group_incarnation=r.group_incarnation AND x.head_revision=r.before_head_revision;
  IF (prior.process_id,prior.content_version,prior.content_digest,prior.head_digest,prior.state,prior.expires_at)
   IS DISTINCT FROM (r.before_process_id,r.before_content_version,r.before_content_digest,r.before_head_digest,r.before_state,r.before_expires_at)
   OR prior.head_digest IS DISTINCT FROM sha256(public.native_group_process_head_bytes_v1(prior))
   OR prior.updated_at>r.executed_at THEN RAISE EXCEPTION 'native_group_process.result_closure_invalid'; END IF;
 END IF;
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
 IF r.terminal_code IS DISTINCT FROM expected_code THEN RAISE EXCEPTION 'native_group_process.result_closure_invalid'; END IF;
 committed:=r.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED');
 IF NOT committed THEN
  IF EXISTS(SELECT 1 FROM public.native_group_process_head_revisions_v1 h WHERE h.result_receipt_id=r.result_receipt_id)
   OR EXISTS(SELECT 1 FROM public.native_group_process_versions_v1 v WHERE v.actor_account_id=p_actor AND v.adopt_command_id=p_command)
   OR EXISTS(SELECT 1 FROM public.native_group_identity_policy_heads_v1 p WHERE p.activation_receipt_id=r.result_receipt_id) THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
  RETURN;
 END IF;
 SELECT x.* INTO STRICT successor FROM public.native_group_process_head_revisions_v1 x
  WHERE x.group_id=r.group_id AND x.group_incarnation=r.group_incarnation AND x.head_revision=r.after_head_revision;
 IF (successor.process_id,successor.content_version,successor.content_digest,successor.head_digest,successor.state,successor.expires_at,
   successor.last_actor_account_id,successor.last_command_id,successor.last_input_digest,successor.result_receipt_id,successor.updated_at,successor.before_head_digest)
  IS DISTINCT FROM (r.after_process_id,r.after_content_version,r.after_content_digest,r.after_head_digest,r.after_state,r.after_expires_at,
   r.actor_account_id,r.command_id,r.input_digest,r.result_receipt_id,r.executed_at,r.before_head_digest)
  OR successor.head_digest IS DISTINCT FROM sha256(public.native_group_process_head_bytes_v1(successor)) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT version_row FROM public.native_group_process_versions_v1 x WHERE
  (x.group_id,x.group_incarnation,x.process_id,x.version)=(r.group_id,r.group_incarnation,r.after_process_id,r.after_content_version);
 IF (version_row.content_digest,version_row.expires_at,version_row.policy_revision,version_row.policy_head_digest)
  IS DISTINCT FROM (r.after_content_digest,r.after_expires_at,r.policy_after_revision,r.policy_after_head_digest)
  OR version_row.content_digest IS DISTINCT FROM sha256(public.native_group_process_version_bytes_v1(version_row)) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
 IF r.operation=1 THEN
  content:=jsonb_build_object('title',version_row.title,'method',version_row.method,
   'intended_claimant_matching_procedure',version_row.intended_claimant_matching_procedure,
   'account_possession_procedure',version_row.account_possession_procedure,
   'physical_human_evidence_procedure',version_row.physical_human_evidence_procedure,
   'duplicate_contradictory_claim_procedure',version_row.duplicate_contradictory_claim_procedure,
   'qualification_criteria_instruction',version_row.qualification_criteria_instruction,
   'escalation_adjudication_procedure',version_row.escalation_adjudication_procedure,
   'evidence_minimization_retention_description',version_row.evidence_minimization_retention_description,
   'recipient_responsibility',version_row.recipient_responsibility);
  IF content IS DISTINCT FROM decoded->'content'
   OR public.native_group_process_micros_v1(version_row.expires_at) IS DISTINCT FROM (decoded->>'expiry_us')::bigint
   OR (version_row.actor_account_id,version_row.designation_receipt_id,version_row.designation_revision,
    version_row.adopt_command_id,version_row.input_digest,version_row.admitted_at,version_row.operator_responsibility)
    IS DISTINCT FROM (r.actor_account_id,r.designation_receipt_id,r.designation_revision,r.command_id,r.input_digest,i.accepted_at,1::smallint) THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
 ELSE
  SELECT x.* INTO STRICT adoption FROM public.native_group_process_results_v1 x
   WHERE x.actor_account_id=version_row.actor_account_id AND x.command_id=version_row.adopt_command_id;
  -- A suspension retains the original version and its original admission
  -- attribution. Resolve that adoption rather than attribute it to suspension.
  IF adoption.operation<>1 OR adoption.terminal_code NOT IN ('ADOPTED','REPLACED')
   OR (adoption.group_id,adoption.group_incarnation,adoption.after_process_id,adoption.after_content_version,
     adoption.after_content_digest,adoption.after_expires_at,adoption.policy_after_revision,adoption.policy_after_head_digest)
    IS DISTINCT FROM (version_row.group_id,version_row.group_incarnation,version_row.process_id,version_row.version,
     version_row.content_digest,version_row.expires_at,version_row.policy_revision,version_row.policy_head_digest)
   OR adoption.executed_at>r.executed_at THEN
   RAISE EXCEPTION 'native_group_process.result_closure_invalid';
  END IF;
  PERFORM public.native_group_process_assert_result_v1(adoption.actor_account_id,adoption.command_id);
 END IF;
 IF r.terminal_code='ADOPTED' AND (policy.first_actor_account_id,policy.first_command_id,policy.first_input_digest,
  policy.activation_receipt_id,policy.activated_at,policy.schema_id,policy.schema_digest,policy.policy_digest,policy.codec_contract_digest,
  policy.registration_manifest_version,policy.registration_manifest_digest)
  IS DISTINCT FROM (r.actor_account_id,r.command_id,r.input_digest,r.result_receipt_id,r.executed_at,r.schema_id,r.schema_digest,
   r.policy_digest,r.codec_contract_digest,r.registration_manifest_version,r.registration_manifest_digest) THEN
  RAISE EXCEPTION 'native_group_process.result_closure_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_assert_current_v1(p_group uuid,p_incarnation uuid) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE policy public.native_group_identity_policy_heads_v1; head public.native_group_process_heads_v1;
 historical public.native_group_process_head_revisions_v1; source jsonb; count_value bigint; latest bigint; version_count bigint;
BEGIN
 SELECT x.* INTO policy FROM public.native_group_identity_policy_heads_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation;
 SELECT x.* INTO head FROM public.native_group_process_heads_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation;
 SELECT count(*),max(h.head_revision) INTO count_value,latest FROM public.native_group_process_head_revisions_v1 h
  WHERE h.group_id=p_group AND h.group_incarnation=p_incarnation;
 SELECT count(*) INTO version_count FROM public.native_group_process_versions_v1 v
  WHERE v.group_id=p_group AND v.group_incarnation=p_incarnation;
 IF policy.group_id IS NULL THEN
  IF head.group_id IS NOT NULL OR count_value<>0 OR version_count<>0 THEN
   RAISE EXCEPTION 'native_group_process.current_closure_invalid';
  END IF;
  RETURN;
 END IF;
 source:=public.native_group_process_source_v1(p_group,p_incarnation);
 IF policy.revision<>1 OR head.group_id IS NULL OR count_value<>latest OR latest IS DISTINCT FROM head.head_revision
  OR version_count<>head.content_version
  OR (to_jsonb(policy)-ARRAY['group_id','group_incarnation','revision','first_actor_account_id','first_command_id',
    'first_input_digest','activation_receipt_id','activated_at','head_digest'])
   IS DISTINCT FROM (source-ARRAY['cedar_sdk_version','cedar_language_version'])
  OR policy.head_digest IS DISTINCT FROM sha256(public.native_group_process_policy_head_bytes_v1(policy)) THEN
  RAISE EXCEPTION 'native_group_process.current_closure_invalid';
 END IF;
 SELECT x.* INTO STRICT historical FROM public.native_group_process_head_revisions_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation AND x.head_revision=latest;
 IF head IS DISTINCT FROM historical THEN RAISE EXCEPTION 'native_group_process.current_closure_invalid'; END IF;
 FOR historical IN SELECT x.* FROM public.native_group_process_head_revisions_v1 x
  WHERE x.group_id=p_group AND x.group_incarnation=p_incarnation ORDER BY x.head_revision LOOP
  IF historical.process_id IS DISTINCT FROM head.process_id THEN
   RAISE EXCEPTION 'native_group_process.current_closure_invalid';
  END IF;
  PERFORM public.native_group_process_assert_result_v1(historical.last_actor_account_id,historical.last_command_id);
 END LOOP;
 IF EXISTS(SELECT 1 FROM public.native_group_process_versions_v1 v WHERE v.group_id=p_group AND v.group_incarnation=p_incarnation
  AND (v.process_id<>head.process_id OR NOT EXISTS(SELECT 1 FROM public.native_group_process_head_revisions_v1 h
   WHERE h.group_id=v.group_id AND h.group_incarnation=v.group_incarnation AND h.process_id=v.process_id
    AND h.content_version=v.version AND h.content_digest=v.content_digest AND h.state='ACTIVE')))
  OR EXISTS(SELECT 1 FROM public.native_group_process_effects_v1 f WHERE f.group_id=p_group AND f.group_incarnation=p_incarnation
   AND NOT EXISTS(SELECT 1 FROM public.native_group_process_results_v1 r WHERE r.actor_account_id=f.actor_account_id AND r.command_id=f.command_id)) THEN
  RAISE EXCEPTION 'native_group_process.current_closure_invalid';
 END IF;
END
$body$;

-- source: ops/native-group-process/context-v1.sql
-- Uninstalled current-identity helper shared by full projections and provisional
-- effect guards. It never asserts incomplete B rows, issues proofs or authorizes
-- a caller; Auth/Cedar and consuming finish remain the retained adapter boundary.
CREATE FUNCTION public.native_group_process_current_context_v1(
 p_account uuid,p_family uuid,p_group uuid,p_incarnation uuid,p_write boolean) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE account_material jsonb; topology public.group_authority_heads; group_row public.groups;
 designation public.deployment_operator_head; birth public.company_enrollment_receipts;
 provenance public.deployment_operator_receipts;
BEGIN
 PERFORM public.native_group_process_group_guard_v1(p_group,p_write);
 PERFORM 1 FROM public.group_authority_lock_shared_v1(p_group);
 account_material:=public.native_group_process_account_material_v1(p_account,p_family);
 IF NOT public.account_company_setup_eligibility_v1(p_account) THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
 END IF;
 SELECT h.* INTO STRICT designation FROM public.deployment_operator_head h
  WHERE h.singleton=1 AND h.account_id=p_account;
 SELECT g.* INTO STRICT group_row FROM public.groups g WHERE g.id=p_group;
 SELECT h.* INTO STRICT topology FROM public.group_authority_heads h WHERE h.group_id=p_group;
 IF topology.incarnation IS DISTINCT FROM p_incarnation OR topology.state IS DISTINCT FROM 'ACTIVE'
  OR group_row.status IS DISTINCT FROM 'ACTIVE' THEN
  RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
 END IF;
 SELECT r.* INTO STRICT birth FROM public.company_enrollment_receipts r
  JOIN public.company_enrollment_requests q ON q.account_id=r.account_id AND q.command_id=r.command_id
   AND q.committed_receipt_id=r.receipt_id AND q.state='COMMITTED'
  WHERE (r.group_id,r.account_id,r.command_id,r.receipt_id)=
   (p_group,group_row.origin_account_id,group_row.origin_command_id,group_row.origin_receipt_id);
 SELECT r.* INTO STRICT provenance FROM public.deployment_operator_receipts r WHERE r.receipt_id=birth.designation_receipt_id;
 IF (provenance.system_identifier,provenance.database_name,provenance.database_oid)
  IS DISTINCT FROM (designation.system_identifier,designation.database_name,designation.database_oid) THEN
  RAISE EXCEPTION 'native_group_process.source_unavailable';
 END IF;
 RETURN jsonb_build_object('account',account_material,'group',to_jsonb(group_row),
  'topology',to_jsonb(topology),'designation',to_jsonb(designation),
  'group_deployment',jsonb_build_object('system_identifier',provenance.system_identifier,
   'database_name',provenance.database_name,'database_oid',provenance.database_oid),
  'source',public.native_group_process_source_v1(p_group,p_incarnation));
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;

-- source: ops/native-group-process/material-v1.sql
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

-- source: ops/native-group-process/transition-v1.sql
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

-- source: ops/native-group-process/guards-v1.sql
-- Uninstalled Group owner perimeter. A has intake/audit only; B participants
-- attest an incomplete same-transaction frame, then deferred closure proves the
-- complete reciprocal result. No guard turns a selected UUID into Auth/Cedar.
CREATE FUNCTION public.native_group_process_validate_frame_v1(f public.native_group_process_effects_v1) RETURNS void
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; m jsonb; source jsonb; d jsonb;
BEGIN
 PERFORM public.native_group_process_group_guard_v1(f.group_id,true);
 m:=public.native_group_process_current_context_v1(f.actor_account_id,f.execution_session_id,f.group_id,f.group_incarnation,true);
 PERFORM public.native_group_process_command_guard_v1(f.actor_account_id,f.command_id,true);
 SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x
  WHERE x.actor_account_id=f.actor_account_id AND x.command_id=f.command_id;
 PERFORM public.native_group_process_assert_input_v1(f.actor_account_id,f.command_id);
 source:=m->'source'; d:=public.native_group_process_decode_v1(i.input_bytes);
 IF (f.group_id,f.group_incarnation,f.operation,f.input_digest,f.intake_receipt_id,f.accepted_at)
   IS DISTINCT FROM (i.group_id,i.group_incarnation,i.operation,i.input_digest,i.intake_receipt_id,i.accepted_at)
  OR f.effect_xid IS DISTINCT FROM pg_current_xact_id() OR f.effect_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR f.effect_xid=i.acceptance_xid OR f.executed_at<transaction_timestamp() OR f.executed_at>clock_timestamp()
  OR f.executed_at<i.accepted_at
  OR f.account_security_generation IS DISTINCT FROM (m#>>'{account,account_security_generation}')::bigint
  OR (f.designation_receipt_id,f.designation_revision,f.observed_group_revision)
   IS DISTINCT FROM ((m#>>'{designation,receipt_id}')::uuid,(m#>>'{designation,revision}')::bigint,(m#>>'{topology,revision}')::bigint)
  OR (f.schema_id,f.schema_digest,f.policy_digest,f.codec_contract_digest,f.registration_manifest_version,
    f.registration_manifest_digest,f.cedar_sdk_version,f.cedar_language_version)
   IS DISTINCT FROM (source->>'schema_id',(source->>'schema_digest')::bytea,(source->>'policy_digest')::bytea,
    (source->>'codec_contract_digest')::bytea,(source->>'registration_manifest_version')::bigint,
    (source->>'registration_manifest_digest')::bytea,source->>'cedar_sdk_version',source->>'cedar_language_version')
  OR (f.schema_id,f.schema_digest,f.policy_digest,f.codec_contract_digest,f.registration_manifest_version,
    f.registration_manifest_digest,f.cedar_sdk_version,f.cedar_language_version)
   IS DISTINCT FROM (i.schema_id,i.schema_digest,i.policy_digest,i.codec_contract_digest,i.registration_manifest_version,
    i.registration_manifest_digest,i.cedar_sdk_version,i.cedar_language_version)
  OR f.requested_process_id IS DISTINCT FROM (d->>'process_id')::uuid
  OR f.terminal_code IS DISTINCT FROM public.native_group_process_classify_v1(f,d)
  OR (f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') AND f.after_expires_at<=clock_timestamp()) THEN
  RAISE EXCEPTION 'native_group_process.effect_frame_invalid';
 END IF;
END
$body$;

CREATE FUNCTION public.native_group_process_input_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE m jsonb; source jsonb; d jsonb;
BEGIN
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_group_process.input_immutable'; END IF;
 m:=public.identity_native_group_process_material_v1(NEW.actor_account_id,NEW.accepted_session_id,
  NEW.group_id,NEW.group_incarnation,NEW.command_id,3::smallint,NEW.input_bytes);
 source:=m->'source'; d:=public.native_group_process_decode_v1(NEW.input_bytes);
 IF NEW.acceptance_xid IS DISTINCT FROM pg_current_xact_id() OR NEW.acceptance_backend_pid IS DISTINCT FROM pg_backend_pid()
  OR NEW.accepted_at<transaction_timestamp() OR NEW.accepted_at>clock_timestamp()
  OR NEW.codec_version<>1 OR NEW.input_digest IS DISTINCT FROM sha256(NEW.input_bytes)
  OR NEW.operation IS DISTINCT FROM (d->>'operation')::smallint
  OR NEW.account_security_generation IS DISTINCT FROM (m#>>'{account,account_security_generation}')::bigint
  OR (NEW.designation_receipt_id,NEW.designation_revision,NEW.observed_group_revision)
   IS DISTINCT FROM ((m#>>'{designation,receipt_id}')::uuid,(m#>>'{designation,revision}')::bigint,(m#>>'{topology,revision}')::bigint)
  OR (NEW.schema_id,NEW.schema_digest,NEW.policy_digest,NEW.codec_contract_digest,NEW.registration_manifest_version,
    NEW.registration_manifest_digest,NEW.cedar_sdk_version,NEW.cedar_language_version)
   IS DISTINCT FROM (source->>'schema_id',(source->>'schema_digest')::bytea,(source->>'policy_digest')::bytea,
    (source->>'codec_contract_digest')::bytea,(source->>'registration_manifest_version')::bigint,
    (source->>'registration_manifest_digest')::bytea,source->>'cedar_sdk_version',source->>'cedar_language_version')
  OR (NEW.accepted_policy_tag,NEW.accepted_policy_revision,NEW.accepted_policy_head_digest)
   IS DISTINCT FROM (CASE WHEN m->'policy'='null'::jsonb THEN 0::smallint ELSE 1::smallint END,
    (m#>>'{policy,revision}')::bigint,(m#>>'{policy,head_digest}')::bytea)
  OR (d->>'expected_group_revision')::bigint IS DISTINCT FROM NEW.observed_group_revision
  OR (d->>'expected_policy_revision')::bigint IS DISTINCT FROM coalesce(NEW.accepted_policy_revision,0)
  OR (NEW.operation=1 AND ((d->>'expected_prior_head_revision')::bigint IS DISTINCT FROM coalesce((m#>>'{head,head_revision}')::bigint,0)
    OR (m->'head'<>'null'::jsonb AND (d->>'process_id')::uuid IS DISTINCT FROM (m#>>'{head,process_id}')::uuid)
    OR (d->>'expiry_us')::numeric<=public.native_group_process_micros_v1(NEW.accepted_at)
    OR (d->>'expiry_us')::numeric-public.native_group_process_micros_v1(NEW.accepted_at)>31536000000000))
  OR (NEW.operation=6 AND (m->'head'='null'::jsonb
    OR (d->>'process_id',d->>'content_version',d->>'content_digest',d->>'expected_head_revision',d->>'expected_head_digest')
     IS DISTINCT FROM (m#>>'{head,process_id}',m#>>'{head,content_version}',m#>>'{head,content_digest}',m#>>'{head,head_revision}',m#>>'{head,head_digest}'))) THEN
  RAISE EXCEPTION 'native_group_process.input_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_effect_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE head public.native_group_process_heads_v1; policy public.native_group_identity_policy_heads_v1;
BEGIN
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_group_process.effect_immutable'; END IF;
 PERFORM public.native_group_process_validate_frame_v1(NEW);
 PERFORM public.native_group_process_assert_current_v1(NEW.group_id,NEW.group_incarnation);
 SELECT x.* INTO head FROM public.native_group_process_heads_v1 x WHERE x.group_id=NEW.group_id AND x.group_incarnation=NEW.group_incarnation;
 SELECT x.* INTO policy FROM public.native_group_identity_policy_heads_v1 x WHERE x.group_id=NEW.group_id AND x.group_incarnation=NEW.group_incarnation;
 IF (NEW.policy_before_tag,NEW.policy_before_revision,NEW.policy_before_head_digest)
   IS DISTINCT FROM (CASE WHEN policy.group_id IS NULL THEN 0::smallint ELSE 1::smallint END,policy.revision,policy.head_digest)
  OR (NEW.before_process_id,NEW.before_head_revision,NEW.before_content_version,NEW.before_content_digest,
    NEW.before_head_digest,NEW.before_state,NEW.before_expires_at)
   IS DISTINCT FROM (head.process_id,head.head_revision,head.content_version,head.content_digest,head.head_digest,head.state,head.expires_at) THEN
  RAISE EXCEPTION 'native_group_process.effect_preimage_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_participant_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_json jsonb; f public.native_group_process_effects_v1; actor uuid; command uuid; receipt uuid;
BEGIN
 IF TG_OP='TRUNCATE' THEN RAISE EXCEPTION 'native_group_process.truncate_denied'; END IF;
 IF TG_OP='DELETE' OR (TG_OP='UPDATE' AND TG_TABLE_NAME<>'native_group_process_heads_v1') THEN
  RAISE EXCEPTION 'native_group_process.participant_immutable';
 END IF;
 row_json:=to_jsonb(NEW);
 IF TG_TABLE_NAME='native_group_process_results_v1' THEN
  actor:=NEW.actor_account_id; command:=NEW.command_id;
 ELSIF TG_TABLE_NAME='native_group_identity_policy_heads_v1' THEN
  actor:=NEW.first_actor_account_id; command:=NEW.first_command_id;
 ELSIF TG_TABLE_NAME='native_group_process_versions_v1' THEN
  actor:=NEW.actor_account_id; command:=NEW.adopt_command_id;
 ELSIF TG_TABLE_NAME IN ('native_group_process_head_revisions_v1','native_group_process_heads_v1') THEN
  actor:=NEW.last_actor_account_id; command:=NEW.last_command_id;
 ELSE RAISE EXCEPTION 'native_group_process.guard_unavailable'; END IF;
 SELECT x.* INTO STRICT f FROM public.native_group_process_effects_v1 x WHERE x.actor_account_id=actor AND x.command_id=command;
 PERFORM public.native_group_process_validate_frame_v1(f);
 IF (row_json->>'group_id',row_json->>'group_incarnation') IS DISTINCT FROM (f.group_id::text,f.group_incarnation::text) THEN
  RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
 END IF;
 IF TG_TABLE_NAME='native_group_process_results_v1' THEN
  IF row_json-ARRAY['layout_version','result_bytes','result_digest','audit_id'] IS DISTINCT FROM to_jsonb(f)
   OR NEW.layout_version<>2 OR NEW.result_bytes IS DISTINCT FROM public.native_group_process_result_bytes_v2(NEW)
   OR NEW.result_digest IS DISTINCT FROM sha256(NEW.result_bytes) THEN RAISE EXCEPTION 'native_group_process.participant_frame_invalid'; END IF;
 ELSIF TG_TABLE_NAME='native_group_identity_policy_heads_v1' THEN
  IF f.terminal_code<>'ADOPTED' OR NEW.activation_receipt_id IS DISTINCT FROM f.result_receipt_id
   OR NEW.head_digest IS DISTINCT FROM f.policy_after_head_digest
   OR NEW.head_digest IS DISTINCT FROM sha256(public.native_group_process_policy_head_bytes_v1(NEW)) THEN
   RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
  END IF;
 ELSIF TG_TABLE_NAME='native_group_process_versions_v1' THEN
  IF f.terminal_code NOT IN ('ADOPTED','REPLACED')
   OR (NEW.process_id,NEW.version,NEW.content_digest,NEW.expires_at)
    IS DISTINCT FROM (f.after_process_id,f.after_content_version,f.after_content_digest,f.after_expires_at)
   OR NEW.content_digest IS DISTINCT FROM sha256(public.native_group_process_version_bytes_v1(NEW)) THEN
   RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
  END IF;
 ELSE
  IF f.terminal_code NOT IN ('ADOPTED','REPLACED','SUSPENDED')
   OR (NEW.process_id,NEW.head_revision,NEW.content_version,NEW.content_digest,NEW.head_digest,NEW.state,NEW.expires_at,
     NEW.last_input_digest,NEW.result_receipt_id,NEW.updated_at,NEW.before_head_digest)
    IS DISTINCT FROM (f.after_process_id,f.after_head_revision,f.after_content_version,f.after_content_digest,f.after_head_digest,
     f.after_state,f.after_expires_at,f.input_digest,f.result_receipt_id,f.executed_at,f.before_head_digest) THEN
   RAISE EXCEPTION 'native_group_process.participant_frame_invalid';
  END IF;
  IF TG_TABLE_NAME='native_group_process_heads_v1' THEN
   IF (TG_OP='INSERT' AND f.terminal_code<>'ADOPTED') OR (TG_OP='UPDATE' AND
     (f.terminal_code='ADOPTED' OR (OLD.group_id,OLD.group_incarnation,OLD.process_id,OLD.head_revision,OLD.content_version,
       OLD.content_digest,OLD.head_digest,OLD.state,OLD.expires_at)
      IS DISTINCT FROM (f.group_id,f.group_incarnation,f.before_process_id,f.before_head_revision,f.before_content_version,
       f.before_content_digest,f.before_head_digest,f.before_state,f.before_expires_at))) THEN
    RAISE EXCEPTION 'native_group_process.head_preimage_invalid';
   END IF;
  END IF;
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_deferred_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE row_json jsonb:=to_jsonb(NEW); actor uuid; command uuid;
BEGIN
 IF TG_TABLE_NAME='native_group_identity_policy_heads_v1' THEN actor:=NEW.first_actor_account_id; command:=NEW.first_command_id;
 ELSIF TG_TABLE_NAME='native_group_process_versions_v1' THEN actor:=NEW.actor_account_id; command:=NEW.adopt_command_id;
 ELSIF TG_TABLE_NAME IN ('native_group_process_head_revisions_v1','native_group_process_heads_v1') THEN actor:=NEW.last_actor_account_id; command:=NEW.last_command_id;
 ELSE actor:=NEW.actor_account_id; command:=NEW.command_id; END IF;
 PERFORM public.native_group_process_assert_input_v1(actor,command);
 IF TG_TABLE_NAME<>'native_group_process_inputs_v1' THEN
  PERFORM public.native_group_process_assert_result_v1(actor,command);
  PERFORM public.native_group_process_assert_current_v1((row_json->>'group_id')::uuid,(row_json->>'group_incarnation')::uuid);
 END IF;
 RETURN NULL;
END
$body$;

CREATE FUNCTION public.native_group_process_immutable_statement_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp
AS $body$ BEGIN RAISE EXCEPTION 'native_group_process.statement_denied'; END $body$;

-- Row admission protects every actual head change. The transition table also
-- rejects zero-row or multi-row UPDATE attempts, which row triggers cannot see.
CREATE FUNCTION public.native_group_process_head_statement_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF (SELECT count(*) FROM native_group_old_heads)<>1 OR (SELECT count(*) FROM native_group_new_heads)<>1 THEN
  RAISE EXCEPTION 'native_group_process.head_statement_invalid';
 END IF;
 RETURN NULL;
END
$body$;

DO $triggers$
DECLARE relation text; before_function text;
BEGIN
 FOREACH relation IN ARRAY ARRAY['native_group_process_inputs_v1','native_group_process_effects_v1','native_group_process_results_v1',
  'native_group_identity_policy_heads_v1','native_group_process_versions_v1','native_group_process_head_revisions_v1','native_group_process_heads_v1'] LOOP
  before_function:=CASE relation WHEN 'native_group_process_inputs_v1' THEN 'native_group_process_input_guard_v1'
   WHEN 'native_group_process_effects_v1' THEN 'native_group_process_effect_guard_v1' ELSE 'native_group_process_participant_guard_v1' END;
  EXECUTE format('CREATE TRIGGER native_group_process_row_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.%I FOR EACH ROW EXECUTE FUNCTION public.%I()',relation,before_function);
  EXECUTE format('CREATE TRIGGER native_group_process_truncate_guard_v1 BEFORE TRUNCATE ON public.%I FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_participant_guard_v1()',relation);
  IF relation='native_group_process_heads_v1' THEN
   EXECUTE format('CREATE TRIGGER native_group_process_immutable_statement_v1 BEFORE DELETE ON public.%I FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_immutable_statement_v1()',relation);
   EXECUTE format('CREATE TRIGGER native_group_process_head_statement_v1 AFTER UPDATE ON public.%I REFERENCING OLD TABLE AS native_group_old_heads NEW TABLE AS native_group_new_heads FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_head_statement_v1()',relation);
   EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_head_statement_v1',relation);
  ELSE
   EXECUTE format('CREATE TRIGGER native_group_process_immutable_statement_v1 BEFORE UPDATE OR DELETE ON public.%I FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_immutable_statement_v1()',relation);
  END IF;
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_immutable_statement_v1',relation);
  EXECUTE format('CREATE CONSTRAINT TRIGGER native_group_process_closure_v1 AFTER INSERT OR UPDATE ON public.%I DEFERRABLE INITIALLY DEFERRED FOR EACH ROW EXECUTE FUNCTION public.native_group_process_deferred_guard_v1()',relation);
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_row_guard_v1',relation);
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_truncate_guard_v1',relation);
  EXECUTE format('ALTER TABLE public.%I ENABLE ALWAYS TRIGGER native_group_process_closure_v1',relation);
 END LOOP;
END
$triggers$;

-- source: ops/native-group-process/commands-v1.sql
-- Additive, uninstalled Group writers. Explicit family selectors follow the
-- reviewed ABI amendment; current Auth/Cedar/proof remain adapter-owned. Owner
-- results are provisional until the shared audit and consuming finish commit.
CREATE FUNCTION public.identity_native_group_process_prepare_v1(
 p_actor uuid,p_family uuid,p_command uuid,p_input bytea)
RETURNS TABLE(inserted boolean,accepted_input public.native_group_process_inputs_v1,terminal public.native_group_process_results_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE d jsonb; m jsonb; source jsonb; at_time timestamptz;
BEGIN
 d:=public.native_group_process_decode_v1(p_input);
 IF (d->>'actor_account_id',d->>'command_id') IS DISTINCT FROM (p_actor::text,p_command::text) THEN
  RAISE EXCEPTION USING ERRCODE='22023',MESSAGE='native_group_process.invalid_input';
 END IF;
 m:=public.identity_native_group_process_material_v1(p_actor,p_family,(d->>'group_id')::uuid,
  (d->>'group_incarnation')::uuid,p_command,3::smallint,p_input);
 IF m->'original'<>'null'::jsonb THEN
  SELECT i.* INTO STRICT accepted_input FROM public.native_group_process_inputs_v1 i
   WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
  IF accepted_input.input_bytes IS DISTINCT FROM p_input THEN RAISE EXCEPTION 'native_group_process.conflict'; END IF;
  SELECT r.* INTO terminal FROM public.native_group_process_results_v1 r WHERE r.actor_account_id=p_actor AND r.command_id=p_command;
  inserted:=false; RETURN NEXT; RETURN;
 END IF;
 source:=m->'source'; at_time:=clock_timestamp();
 accepted_input:=jsonb_populate_record(NULL::public.native_group_process_inputs_v1,source-'registered_actions'||jsonb_build_object(
  'actor_account_id',p_actor,'command_id',p_command,'group_id',d->>'group_id','group_incarnation',d->>'group_incarnation',
  'operation',(d->>'operation')::smallint,'codec_version',1,'input_bytes',p_input,'input_digest',sha256(p_input),
  'intake_receipt_id',gen_random_uuid(),'accepted_at',at_time,'accepted_session_id',p_family,
  'account_security_generation',(m#>>'{account,account_security_generation}')::bigint,
  'designation_receipt_id',m#>>'{designation,receipt_id}','designation_revision',(m#>>'{designation,revision}')::bigint,
  'observed_group_revision',(m#>>'{topology,revision}')::bigint,
  'accepted_policy_tag',CASE WHEN m->'policy'='null'::jsonb THEN 0 ELSE 1 END,
  'accepted_policy_revision',m#>>'{policy,revision}','accepted_policy_head_digest',m#>>'{policy,head_digest}',
  'acceptance_xid',pg_current_xact_id()::text,'acceptance_backend_pid',pg_backend_pid(),'audit_id',gen_random_uuid()));
 INSERT INTO public.native_group_process_inputs_v1 SELECT accepted_input.*;
 -- The adapter appends the exact returned audit UUID before any closure read.
 inserted:=true; RETURN NEXT;
END
$body$;

CREATE FUNCTION public.identity_native_group_process_execute_v1(p_actor uuid,p_family uuid,p_command uuid)
RETURNS TABLE(inserted boolean,accepted_input public.native_group_process_inputs_v1,terminal public.native_group_process_results_v1)
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE m jsonb; source jsonb; d jsonb; at_time timestamptz; expiry_value timestamptz;
 f public.native_group_process_effects_v1; policy public.native_group_identity_policy_heads_v1;
 head public.native_group_process_heads_v1; version_row public.native_group_process_versions_v1;
 history public.native_group_process_head_revisions_v1; policy_json jsonb; head_json jsonb; content jsonb;
BEGIN
 -- Namespace discovery is a selector only. Rows are immutable; their Group and
 -- actor binding is reloaded after the deterministic Group-first lock plan.
 SELECT i.* INTO STRICT accepted_input FROM public.native_group_process_inputs_v1 i
  WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
 m:=public.identity_native_group_process_material_v1(p_actor,p_family,accepted_input.group_id,
  accepted_input.group_incarnation,p_command,4::smallint,NULL);
 SELECT i.* INTO STRICT accepted_input FROM public.native_group_process_inputs_v1 i
  WHERE i.actor_account_id=p_actor AND i.command_id=p_command;
 IF accepted_input.acceptance_xid=pg_current_xact_id() THEN RAISE EXCEPTION 'native_group_process.separate_commit_required'; END IF;
 IF m#>'{original,terminal}'<>'null'::jsonb THEN
  -- Terminal replay belongs to the retained read-own scope, never this writer.
  RAISE EXCEPTION 'native_group_process.pending_required';
 END IF;
 source:=m->'source'; d:=public.native_group_process_decode_v1(accepted_input.input_bytes);
 at_time:=clock_timestamp(); policy_json:=m->'policy'; head_json:=m->'head';
 f:=jsonb_populate_record(NULL::public.native_group_process_effects_v1,source-'registered_actions'||jsonb_build_object(
  'actor_account_id',p_actor,'command_id',p_command,'group_id',accepted_input.group_id,'group_incarnation',accepted_input.group_incarnation,
  'operation',accepted_input.operation,'input_digest',accepted_input.input_digest,'intake_receipt_id',accepted_input.intake_receipt_id,
  'effect_id',gen_random_uuid(),'result_receipt_id',gen_random_uuid(),'accepted_at',accepted_input.accepted_at,
  'executed_at',at_time,'execution_session_id',p_family,
  'account_security_generation',(m#>>'{account,account_security_generation}')::bigint,
  'designation_receipt_id',m#>>'{designation,receipt_id}','designation_revision',(m#>>'{designation,revision}')::bigint,
  'observed_group_revision',(m#>>'{topology,revision}')::bigint,
  'policy_before_tag',CASE WHEN policy_json='null'::jsonb THEN 0 ELSE 1 END,
  'policy_before_revision',policy_json->>'revision','policy_before_head_digest',policy_json->>'head_digest',
  'requested_process_id',d->>'process_id','before_process_id',head_json->>'process_id',
  'before_head_revision',head_json->>'head_revision','before_content_version',head_json->>'content_version',
  'before_content_digest',head_json->>'content_digest','before_head_digest',head_json->>'head_digest',
  'before_state',head_json->>'state','before_expires_at',head_json->>'expires_at',
  'effect_xid',pg_current_xact_id()::text,'effect_backend_pid',pg_backend_pid()));
 f.terminal_code:=public.native_group_process_classify_v1(f,d);
 f.policy_after_tag:=f.policy_before_tag; f.policy_after_revision:=f.policy_before_revision; f.policy_after_head_digest:=f.policy_before_head_digest;
 f.after_process_id:=f.before_process_id; f.after_head_revision:=f.before_head_revision; f.after_content_version:=f.before_content_version;
 f.after_content_digest:=f.before_content_digest; f.after_head_digest:=f.before_head_digest; f.after_state:=f.before_state; f.after_expires_at:=f.before_expires_at;
 IF f.terminal_code='ADOPTED' THEN
  policy:=jsonb_populate_record(NULL::public.native_group_identity_policy_heads_v1,source-ARRAY['cedar_sdk_version','cedar_language_version']||jsonb_build_object(
   'group_id',f.group_id,'group_incarnation',f.group_incarnation,'revision',1,'first_actor_account_id',p_actor,
   'first_command_id',p_command,'first_input_digest',f.input_digest,'activation_receipt_id',f.result_receipt_id,'activated_at',at_time));
  policy.head_digest:=sha256(public.native_group_process_policy_head_bytes_v1(policy));
  f.policy_after_tag:=1; f.policy_after_revision:=1; f.policy_after_head_digest:=policy.head_digest;
 END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED') THEN
  content:=d->'content';
  -- Parse integer seconds through PostgreSQL's interval grammar, never a float
  -- to_timestamp. Exact microsecond roundtrip is mandatory before persistence.
  expiry_value:=((timestamp '1970-01-01'+(((d->>'expiry_us')::bigint/1000000)::text||' seconds')::interval) AT TIME ZONE 'UTC');
  IF public.native_group_process_micros_v1(expiry_value) IS DISTINCT FROM (d->>'expiry_us')::bigint THEN
   RAISE EXCEPTION 'native_group_process.expiry_unavailable';
  END IF;
  version_row:=jsonb_populate_record(NULL::public.native_group_process_versions_v1,content||jsonb_build_object(
   'group_id',f.group_id,'group_incarnation',f.group_incarnation,'process_id',f.requested_process_id,
   'version',coalesce(f.before_content_version,0)+1,'schema_id','GROUP_VERIFIER_PROCESS_V1',
   'actor_account_id',p_actor,'designation_receipt_id',f.designation_receipt_id,'designation_revision',f.designation_revision,
   'policy_revision',f.policy_after_revision,'policy_head_digest',f.policy_after_head_digest,
   'adopt_command_id',p_command,'input_digest',f.input_digest,'admitted_at',accepted_input.accepted_at,
   'expires_at',expiry_value,'operator_responsibility',1));
  version_row.content_digest:=sha256(public.native_group_process_version_bytes_v1(version_row));
  f.after_process_id:=f.requested_process_id; f.after_content_version:=version_row.version;
  f.after_content_digest:=version_row.content_digest; f.after_expires_at:=expiry_value; f.after_state:='ACTIVE';
 END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN
  f.after_head_revision:=coalesce(f.before_head_revision,0)+1;
  IF f.terminal_code='SUSPENDED' THEN f.after_state:='SUSPENDED'; END IF;
  history:=jsonb_populate_record(NULL::public.native_group_process_head_revisions_v1,jsonb_build_object(
   'group_id',f.group_id,'group_incarnation',f.group_incarnation,'process_id',f.after_process_id,'head_revision',f.after_head_revision,
   'content_version',f.after_content_version,'content_digest',f.after_content_digest,'state',f.after_state,'expires_at',f.after_expires_at,
   'last_actor_account_id',p_actor,'last_command_id',p_command,'last_input_digest',f.input_digest,
   'result_receipt_id',f.result_receipt_id,'updated_at',at_time,'before_head_digest',f.before_head_digest));
  history.head_digest:=sha256(public.native_group_process_head_bytes_v1(history)); f.after_head_digest:=history.head_digest;
 END IF;
 f.effect_census:=jsonb_build_object('policy_heads',CASE WHEN f.terminal_code='ADOPTED' THEN 1 ELSE 0 END,
  'versions',CASE WHEN f.terminal_code IN ('ADOPTED','REPLACED') THEN 1 ELSE 0 END,
  'heads',CASE WHEN f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN 1 ELSE 0 END,
  'head_revisions',CASE WHEN f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN 1 ELSE 0 END,
  'effects',1,'results',1,'audits',1);
 terminal:=jsonb_populate_record(NULL::public.native_group_process_results_v1,to_jsonb(f)||jsonb_build_object('layout_version',2,'audit_id',gen_random_uuid()));
 terminal.result_bytes:=public.native_group_process_result_bytes_v2(terminal); terminal.result_digest:=sha256(terminal.result_bytes);
 -- Frame precedes provisional participants. All reciprocal FKs are deferred;
 -- no complete material/closure is read until the adapter appends shared audit.
 INSERT INTO public.native_group_process_effects_v1 SELECT f.*;
 INSERT INTO public.native_group_process_results_v1 SELECT terminal.*;
 IF f.terminal_code='ADOPTED' THEN INSERT INTO public.native_group_identity_policy_heads_v1 SELECT policy.*; END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED') THEN INSERT INTO public.native_group_process_versions_v1 SELECT version_row.*; END IF;
 IF f.terminal_code IN ('ADOPTED','REPLACED','SUSPENDED') THEN
  INSERT INTO public.native_group_process_head_revisions_v1 SELECT history.*;
  IF f.terminal_code='ADOPTED' THEN
   INSERT INTO public.native_group_process_heads_v1 SELECT history.*;
  ELSE
   UPDATE public.native_group_process_heads_v1 h SET
    head_revision=history.head_revision,content_version=history.content_version,content_digest=history.content_digest,
    state=history.state,expires_at=history.expires_at,last_actor_account_id=history.last_actor_account_id,
    last_command_id=history.last_command_id,last_input_digest=history.last_input_digest,result_receipt_id=history.result_receipt_id,
    updated_at=history.updated_at,head_digest=history.head_digest,before_head_digest=history.before_head_digest
   WHERE h.group_id=f.group_id AND h.group_incarnation=f.group_incarnation
    AND (h.process_id,h.head_revision,h.content_version,h.content_digest,h.head_digest,h.state,h.expires_at)
     IS NOT DISTINCT FROM (f.before_process_id,f.before_head_revision,f.before_content_version,f.before_content_digest,
      f.before_head_digest,f.before_state,f.before_expires_at);
   IF NOT FOUND THEN RAISE EXCEPTION 'native_group_process.head_conflict'; END IF;
  END IF;
 END IF;
 inserted:=true; RETURN NEXT;
EXCEPTION WHEN no_data_found OR too_many_rows THEN RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;

-- source: ops/native-group-process/audit-v1.sql
-- Uninstalled Group audit source, part of the complete owner successor only.
-- Group has no Company scope. Read visibility is limited to its exact protected
-- frames; it does not expose unrelated NULL-org Account audit events.
GRANT SELECT(id) ON public.audit_events TO console_account_owner;
CREATE POLICY native_group_process_account_audit_read_v1 ON public.audit_events
 FOR SELECT TO console_account_owner USING (
  org_id IS NULL AND branch_id IS NULL AND before_snap IS NULL AND (
   (action='identity.group_process.accept' AND target_type='native_group_process_inputs_v1'
    AND EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1 i WHERE i.audit_id=audit_events.id
     AND i.actor_account_id=audit_events.actor AND i.intake_receipt_id::text=audit_events.target_id
     AND i.accepted_at=audit_events.occurred_at AND audit_events.after_snap=public.native_group_process_accept_snapshot_v1(i)))
   OR (action='identity.group_process.complete' AND target_type='native_group_process_results_v1'
    AND EXISTS(SELECT 1 FROM public.native_group_process_results_v1 r WHERE r.audit_id=audit_events.id
     AND r.actor_account_id=audit_events.actor AND r.result_receipt_id::text=audit_events.target_id
     AND r.executed_at=audit_events.occurred_at AND audit_events.after_snap=public.native_group_process_complete_snapshot_v1(r)))
  )
 );

CREATE FUNCTION public.native_group_process_audit_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
DECLARE i public.native_group_process_inputs_v1; r public.native_group_process_results_v1;
 frame_group uuid; frame_actor uuid; frame_session uuid; frame_generation bigint;
 frame_xid xid8; frame_pid integer; frame_target text; frame_receipt uuid; frame_time timestamptz;
 frame_payload jsonb; account_material jsonb;
BEGIN
 IF TG_OP IN ('UPDATE','DELETE') AND OLD.action IN ('identity.group_process.accept','identity.group_process.complete') THEN
  RAISE EXCEPTION 'native_group_process.audit_immutable';
 END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 IF NEW.action NOT IN ('identity.group_process.accept','identity.group_process.complete') THEN RETURN NEW; END IF;
 IF TG_OP<>'INSERT' THEN RAISE EXCEPTION 'native_group_process.audit_immutable'; END IF;
 -- These selectors confer no authority. The exact same-transaction frame and
 -- fixed Group-first locks are checked before any protected event is admitted.
 IF NEW.action='identity.group_process.accept' THEN
  SELECT x.* INTO STRICT i FROM public.native_group_process_inputs_v1 x WHERE x.audit_id=NEW.id;
  frame_group:=i.group_id; frame_actor:=i.actor_account_id; frame_session:=i.accepted_session_id;
  frame_generation:=i.account_security_generation; frame_xid:=i.acceptance_xid; frame_pid:=i.acceptance_backend_pid;
  frame_target:='native_group_process_inputs_v1'; frame_receipt:=i.intake_receipt_id; frame_time:=i.accepted_at;
  frame_payload:=public.native_group_process_accept_snapshot_v1(i);
 ELSE
  SELECT x.* INTO STRICT r FROM public.native_group_process_results_v1 x WHERE x.audit_id=NEW.id;
  frame_group:=r.group_id; frame_actor:=r.actor_account_id; frame_session:=r.execution_session_id;
  frame_generation:=r.account_security_generation; frame_xid:=r.effect_xid; frame_pid:=r.effect_backend_pid;
  frame_target:='native_group_process_results_v1'; frame_receipt:=r.result_receipt_id; frame_time:=r.executed_at;
  frame_payload:=public.native_group_process_complete_snapshot_v1(r);
 END IF;
 IF frame_xid IS DISTINCT FROM pg_current_xact_id() OR frame_pid IS DISTINCT FROM pg_backend_pid() THEN
  RAISE EXCEPTION 'native_group_process.audit_frame_invalid';
 END IF;
 PERFORM public.native_group_process_group_guard_v1(frame_group,true);
 PERFORM 1 FROM public.group_authority_lock_shared_v1(frame_group);
 account_material:=public.native_group_process_account_material_v1(frame_actor,frame_session);
 IF (account_material->>'account_security_generation')::bigint IS DISTINCT FROM frame_generation
  OR (NEW.actor,NEW.target_type,NEW.target_id,NEW.occurred_at,NEW.after_snap)
   IS DISTINCT FROM (frame_actor,frame_target,frame_receipt::text,frame_time,frame_payload)
  OR NEW.org_id IS NOT NULL OR NEW.branch_id IS NOT NULL OR NEW.before_snap IS NOT NULL
  OR num_nonnulls(NEW.ip,NEW.user_agent,NEW.auth_method,NEW.device,NEW.classification_badges,NEW.anomaly,NEW.reason)<>0
  OR NEW.trace_id IS NULL OR octet_length(NEW.trace_id)<>32 OR NEW.trace_id !~ '^[0-9a-f]{32}$' OR NEW.trace_id=repeat('0',32)
  OR NEW.span_id IS NULL OR octet_length(NEW.span_id)<>16 OR NEW.span_id !~ '^[0-9a-f]{16}$' OR NEW.span_id=repeat('0',16) THEN
  RAISE EXCEPTION 'native_group_process.audit_frame_invalid';
 END IF;
 RETURN NEW;
END
$body$;

CREATE FUNCTION public.native_group_process_audit_truncate_guard_v1() RETURNS trigger
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
AS $body$
BEGIN
 IF EXISTS(SELECT 1 FROM public.native_group_process_inputs_v1)
  OR EXISTS(SELECT 1 FROM public.native_group_process_results_v1) THEN
  RAISE EXCEPTION 'native_group_process.audit_immutable';
 END IF;
 RETURN NULL;
END
$body$;

CREATE TRIGGER native_group_process_audit_guard_v1 BEFORE INSERT OR UPDATE OR DELETE ON public.audit_events
 FOR EACH ROW EXECUTE FUNCTION public.native_group_process_audit_guard_v1();
CREATE TRIGGER native_group_process_audit_truncate_guard_v1 BEFORE TRUNCATE ON public.audit_events
 FOR EACH STATEMENT EXECUTE FUNCTION public.native_group_process_audit_truncate_guard_v1();
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_group_process_audit_guard_v1;
ALTER TABLE public.audit_events ENABLE ALWAYS TRIGGER native_group_process_audit_truncate_guard_v1;
ALTER FUNCTION public.native_group_process_audit_guard_v1() OWNER TO console_account_owner;
ALTER FUNCTION public.native_group_process_audit_truncate_guard_v1() OWNER TO console_account_owner;
REVOKE ALL ON FUNCTION public.native_group_process_audit_guard_v1(),public.native_group_process_audit_truncate_guard_v1()
 FROM PUBLIC,console_app,console_rt,console_auth_rt;

-- source: ops/native-group-process/discovery-v1.sql
-- Additive read ABI. Install only with the complete reviewed owner/ACL/custody
-- successor. A returned selector is never authentication or authorization.
CREATE FUNCTION public.identity_native_group_process_incarnation_selector_v1(
 p_actor uuid,p_family uuid,p_group uuid,p_command uuid) RETURNS uuid
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE incarnation uuid; original jsonb; origin_columns integer;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR current_setting('server_encoding') IS DISTINCT FROM 'UTF8'
  OR num_nonnulls(p_actor,p_family,p_group)<>3
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_family,p_group,p_command) THEN
  RAISE EXCEPTION 'native_group_process.material_unavailable';
 END IF;
 PERFORM public.native_group_process_group_guard_v1(p_group,false);
 IF p_command IS NULL THEN
  -- Native birth retains this same historical guard. Take any live topology
  -- row before Account/family; a later scope rechecks the selected incarnation.
  PERFORM 1 FROM public.group_authority_lock_shared_v1(p_group);
 END IF;
 PERFORM public.native_group_process_account_material_v1(p_actor,p_family);
 IF p_command IS NULL THEN
  SELECT num_nonnulls(g.origin_account_id,g.origin_command_id,g.origin_receipt_id) INTO origin_columns
   FROM public.groups g WHERE g.id=p_group;
  IF NOT FOUND OR origin_columns=0 THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
  END IF;
  IF origin_columns<>3 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  SELECT h.incarnation INTO incarnation FROM public.group_authority_heads h WHERE h.group_id=p_group;
  IF NOT FOUND THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  PERFORM public.native_group_process_current_context_v1(p_actor,p_family,p_group,incarnation,false);
  PERFORM public.native_group_process_assert_current_v1(p_group,incarnation);
 ELSE
  -- Historical receipt selection has no live-head/designation prerequisite.
  -- The immutable input supplies the incarnation; do not reinterpret bytes.
  PERFORM public.native_group_process_command_guard_v1(p_actor,p_command,false);
  SELECT i.group_incarnation INTO incarnation FROM public.native_group_process_inputs_v1 i
   WHERE i.actor_account_id=p_actor AND i.command_id=p_command AND i.group_id=p_group;
  IF NOT FOUND THEN
   RAISE EXCEPTION USING ERRCODE='42501',MESSAGE='native_group_process.not_found';
  END IF;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  original:=public.native_group_process_original_material_v1(p_actor,p_group,incarnation,p_command);
  IF original IS NULL THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 END IF;
 RETURN incarnation;
END
$body$;

CREATE FUNCTION public.identity_native_group_process_navigation_candidates_v1(
 p_actor uuid,p_family uuid,p_original bytea) RETURNS jsonb
LANGUAGE plpgsql VOLATILE SECURITY DEFINER PARALLEL UNSAFE
SET search_path=pg_catalog,pg_temp SET row_security=on
SET TimeZone='UTC' SET bytea_output='hex' SET DateStyle='ISO, YMD' SET IntervalStyle='postgres'
AS $body$
DECLARE original jsonb; snapshot_text text; captured jsonb; snapshot_bytes bytea;
 old_groups uuid[]:=ARRAY[]::uuid[]; current_groups uuid[]; guarded_groups uuid[]; after_groups uuid[];
 selected_group uuid; candidate jsonb; previous_group uuid; group_value jsonb; history_value jsonb;
 candidates jsonb:='[]'::jsonb; groups_value jsonb:='[]'::jsonb; consent_value jsonb:='[]'::jsonb;
 material jsonb; account_value jsonb; designation_value jsonb; item record; head_revision bigint;
 budget integer:=1048576; additional integer; first_group boolean:=true; first_history boolean;
 root public.accounts; security public.account_security; enrollment public.account_security_events;
 family record; terms record; designation public.deployment_operator_head; designation_receipt public.deployment_operator_receipts;
 rows_to_validate jsonb:='[]'::jsonb; checked_row jsonb; expected_keys text[];
 -- Exact schema types/nullability; no field-name inference. Native closure
 -- tightens nullable database columns only where that row purpose requires it.
 row_shapes jsonb:=$shapes${"account_security":{"account_id":["uuid",false],"context_generation":["integer",false],"revision":["integer",false],"security_generation":["integer",false],"security_state":["string",false],"updated_at":["timestamp",false]},"account_security_events":{"account_id":["uuid",false],"actor_account_id":["uuid",false],"evidence_ref":["object",false],"id":["uuid",false],"kind":["string",false],"occurred_at":["timestamp",false],"payload":["object",false],"session_id":["uuid",false]},"accounts":{"created_at":["timestamp",false],"id":["uuid",false]},"company_enrollment_receipts":{"account_id":["uuid",false],"action_refs":["array",false],"administrative_account_id":["uuid",false],"catalog_version":["string",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_at":["timestamp",false],"designation_receipt_id":["uuid",false],"group_id":["uuid",false],"input_digest":["bytea",false],"manifest_digest":["bytea",false],"org_id":["uuid",false],"property_refs":["array",false],"receipt_id":["uuid",false],"root_assignment_id":["uuid",false],"root_revision":["integer",false],"session_id":["uuid",false]},"company_enrollment_requests":{"account_id":["uuid",false],"codec_version":["integer",false],"command_id":["uuid",false],"committed_receipt_id":["uuid",false],"created_at":["timestamp",false],"designation_receipt_id":["uuid",false],"expires_at":["timestamp",false],"input_bytes":["bytea",true],"input_digest":["bytea",false],"state":["string",false],"terminal_at":["timestamp",false]},"consent":{"accepted_at":["timestamp",false],"content_sha256":["bytea",false],"manifest_sha256":["bytea",false],"terms_kind":["string",false]},"deployment_operator_head":{"account_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"receipt_id":["uuid",false],"revision":["integer",false],"singleton":["integer",false],"system_identifier":["string",false]},"deployment_operator_receipts":{"account_id":["uuid",false],"command_id":["uuid",false],"database_name":["string",false],"database_oid":["integer",false],"expected_revision":["integer",false],"expected_security_generation":["integer",true],"kind":["string",false],"reason":["string",true],"receipt_id":["uuid",false],"recorded_at":["timestamp",false],"revision":["integer",false],"system_identifier":["string",false]},"family":{"account_security_generation":["integer",false],"assurance":["string",false],"auth_time":["timestamp",false],"created_at":["timestamp",false],"org_id":["uuid",true],"protocol":["string",false],"revoked_at":["timestamp",true],"user_id":["uuid",false]},"group_authority_heads":{"group_id":["uuid",false],"incarnation":["uuid",false],"revision":["integer",false],"state":["string",false]},"groups":{"created_at":["timestamp",false],"id":["uuid",false],"name":["string",false],"origin_account_id":["uuid",false],"origin_command_id":["uuid",false],"origin_receipt_id":["uuid",false],"slug":["string",false],"status":["string",false],"updated_at":["timestamp",false]},"native_group_identity_policy_heads_v1":{"activated_at":["timestamp",false],"activation_receipt_id":["uuid",false],"codec_contract_digest":["bytea",false],"first_actor_account_id":["uuid",false],"first_command_id":["uuid",false],"first_input_digest":["bytea",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"revision":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"native_group_process_head_revisions_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_heads_v1":{"before_head_digest":["bytea",true],"content_digest":["bytea",false],"content_version":["integer",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"head_digest":["bytea",false],"head_revision":["integer",false],"last_actor_account_id":["uuid",false],"last_command_id":["uuid",false],"last_input_digest":["bytea",false],"process_id":["uuid",false],"result_receipt_id":["uuid",false],"state":["string",false],"updated_at":["timestamp",false]},"native_group_process_versions_v1":{"account_possession_procedure":["string",false],"actor_account_id":["uuid",false],"admitted_at":["timestamp",false],"adopt_command_id":["uuid",false],"content_digest":["bytea",false],"designation_receipt_id":["uuid",false],"designation_revision":["integer",false],"duplicate_contradictory_claim_procedure":["string",false],"escalation_adjudication_procedure":["string",false],"evidence_minimization_retention_description":["string",false],"expires_at":["timestamp",false],"group_id":["uuid",false],"group_incarnation":["uuid",false],"input_digest":["bytea",false],"intended_claimant_matching_procedure":["string",false],"method":["string",false],"operator_responsibility":["integer",false],"physical_human_evidence_procedure":["string",false],"policy_head_digest":["bytea",false],"policy_revision":["integer",false],"process_id":["uuid",false],"qualification_criteria_instruction":["string",false],"recipient_responsibility":["string",false],"schema_id":["string",false],"title":["string",false],"version":["integer",false]},"source":{"cedar_language_version":["string",false],"cedar_sdk_version":["string",false],"codec_contract_digest":["bytea",false],"policy_digest":["bytea",false],"registered_actions":["array",false],"registration_manifest_digest":["bytea",false],"registration_manifest_version":["integer",false],"schema_digest":["bytea",false],"schema_id":["string",false]},"terms_head":{"manifest_sha256":["bytea",false],"release_receipt_id":["uuid",false],"revision":["integer",false]}}$shapes$::jsonb;
 document jsonb; required text; incarnation uuid; row_key text; row_value jsonb;
 entry jsonb; revision_value text; field_shape jsonb; value_text text; parsed_time timestamptz;
 birth_request public.company_enrollment_requests; birth_receipt public.company_enrollment_receipts;
 birth_effect public.company_enrollment_effect_bindings; decoded_birth record;
BEGIN
 IF current_setting('transaction_isolation') IS DISTINCT FROM 'read committed'
  OR current_setting('server_encoding') IS DISTINCT FROM 'UTF8'
  OR num_nonnulls(p_actor,p_family)<>2
  OR '00000000-0000-0000-0000-000000000000'::uuid IN(p_actor,p_family) THEN
  RAISE EXCEPTION 'native_group_process.material_unavailable';
 END IF;
 IF p_original IS NOT NULL THEN
  IF octet_length(p_original) NOT BETWEEN 1 AND 1048576 THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  BEGIN
   snapshot_text:=convert_from(p_original,'UTF8'); original:=snapshot_text::jsonb;
  EXCEPTION WHEN invalid_text_representation OR character_not_in_repertoire THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END;
  -- Equality rejects duplicate members and alternate serialization before any
  -- source selector is used. This is an internal observation, not permission.
  IF convert_to(original::text,'UTF8') IS DISTINCT FROM p_original
   OR jsonb_typeof(original) IS DISTINCT FROM 'object'
   OR original->>'protocol' IS DISTINCT FROM 'GROUP_PROCESS_NAVIGATION_SOURCE_V1'
   OR ARRAY(SELECT key FROM jsonb_object_keys(original) key ORDER BY key COLLATE "C")
      IS DISTINCT FROM ARRAY['account','designation','groups','protocol']
   OR original#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR original#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR jsonb_typeof(original->'groups') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(original->'groups')>256 THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(original->'groups') LOOP
   IF jsonb_typeof(candidate) IS DISTINCT FROM 'object'
    OR jsonb_typeof(candidate->'group_id') IS DISTINCT FROM 'string'
    OR candidate->>'group_id' !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
    OR jsonb_typeof(candidate->'group_incarnation') IS DISTINCT FROM 'string'
    OR candidate->>'group_incarnation' !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   selected_group:=(candidate->>'group_id')::uuid;
   incarnation:=(candidate->>'group_incarnation')::uuid;
   IF '00000000-0000-0000-0000-000000000000'::uuid IN(selected_group,incarnation)
    OR (previous_group IS NOT NULL AND selected_group<=previous_group) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   old_groups:=array_append(old_groups,selected_group); previous_group:=selected_group;
  END LOOP;
  document:=original;
  IF jsonb_typeof(document->'account') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF ARRAY(SELECT key FROM jsonb_object_keys(document->'account') key ORDER BY key COLLATE "C")
   IS DISTINCT FROM ARRAY['actor_account_id','consent','enrollment','family','root','security','session_id','terms_head']
   OR document#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR document#>>'{account,root,id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,security,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,family,user_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,kind}' IS DISTINCT FROM 'ENROLLED'
   OR jsonb_typeof(document#>'{account,consent}') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(document#>'{account,consent}') NOT BETWEEN 1 AND 8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  rows_to_validate:=jsonb_build_array(jsonb_build_array('accounts',document#>'{account,root}'),
   jsonb_build_array('account_security',document#>'{account,security}'),
   jsonb_build_array('family',document#>'{account,family}'),
   jsonb_build_array('account_security_events',document#>'{account,enrollment}'),
   jsonb_build_array('terms_head',document#>'{account,terms_head}'));
  required:=NULL;
  FOR entry IN SELECT value FROM jsonb_array_elements(document#>'{account,consent}') LOOP
   IF jsonb_typeof(entry->'terms_kind') IS DISTINCT FROM 'string'
    OR entry->>'terms_kind' !~ '^[a-z][a-z0-9_.-]{0,63}$'
    OR (required IS NOT NULL AND (entry->>'terms_kind') COLLATE "C"<=required COLLATE "C") THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   required:=entry->>'terms_kind'; rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('consent',entry));
  END LOOP;
  IF document->'designation' IS DISTINCT FROM 'null'::jsonb THEN
   IF jsonb_typeof(document->'designation') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   IF ARRAY(SELECT key FROM jsonb_object_keys(document->'designation') key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','receipt'] THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('deployment_operator_head',document#>'{designation,head}'),
    jsonb_build_array('deployment_operator_receipts',document#>'{designation,receipt}'));
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(document->'groups') LOOP
   IF ARRAY(SELECT key FROM jsonb_object_keys(candidate) key ORDER BY key COLLATE "C")
    IS DISTINCT FROM ARRAY['birth_designation_receipt','birth_receipt','birth_request','group','group_id','group_incarnation','head','history','policy','source','topology','version']
    OR candidate#>>'{group,id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
    OR candidate#>>'{birth_receipt,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR jsonb_typeof(candidate->'history') IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   IF (candidate->'policy'='null'::jsonb) IS DISTINCT FROM (candidate->'head'='null'::jsonb)
    OR (candidate->'head'='null'::jsonb) IS DISTINCT FROM (candidate->'version'='null'::jsonb)
    OR ((candidate->'head'='null'::jsonb) IS DISTINCT FROM (jsonb_array_length(candidate->'history')=0)) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('groups',candidate->'group'),jsonb_build_array('group_authority_heads',candidate->'topology'),
    jsonb_build_array('company_enrollment_requests',candidate->'birth_request'),
    jsonb_build_array('company_enrollment_receipts',candidate->'birth_receipt'),
    jsonb_build_array('deployment_operator_receipts',candidate->'birth_designation_receipt'),
    jsonb_build_array('source',candidate->'source'));
   IF candidate->'policy' IS DISTINCT FROM 'null'::jsonb THEN
    rows_to_validate:=rows_to_validate||jsonb_build_array(
     jsonb_build_array('native_group_identity_policy_heads_v1',candidate->'policy'),
     jsonb_build_array('native_group_process_heads_v1',candidate->'head'),
     jsonb_build_array('native_group_process_versions_v1',candidate->'version'));
   END IF;
   head_revision:=0;
   FOR entry IN SELECT value FROM jsonb_array_elements(candidate->'history') LOOP
    IF jsonb_typeof(entry) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    IF ARRAY(SELECT key FROM jsonb_object_keys(entry) key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','reason']
     OR jsonb_typeof(entry->'reason') NOT IN ('string','null')
     OR entry#>>'{head,group_id}' IS DISTINCT FROM candidate->>'group_id'
     OR entry#>>'{head,group_incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
     OR entry#>>'{head,process_id}' IS DISTINCT FROM candidate#>>'{head,process_id}' THEN
     RAISE EXCEPTION 'native_group_process.material_unavailable';
    END IF;
    revision_value:=entry#>>'{head,head_revision}';
    IF revision_value IS NULL OR revision_value !~ '^[1-9][0-9]{0,18}$' OR revision_value::numeric>9223372036854775807
     OR revision_value::numeric<>head_revision::numeric+1 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    head_revision:=revision_value::bigint;
    rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('native_group_process_head_revisions_v1',entry->'head'));
   END LOOP;
   IF head_revision>0 AND (candidate->'history'->-1->'head') IS DISTINCT FROM candidate->'head' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END LOOP;
  FOR checked_row IN SELECT value FROM jsonb_array_elements(rows_to_validate) LOOP
   IF jsonb_typeof(checked_row->1) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   expected_keys:=ARRAY(SELECT key FROM jsonb_object_keys(row_shapes->(checked_row->>0)) key ORDER BY key COLLATE "C");
   IF cardinality(expected_keys)=0 OR ARRAY(SELECT key FROM jsonb_object_keys(checked_row->1) key ORDER BY key COLLATE "C") IS DISTINCT FROM expected_keys THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   FOR row_key,row_value IN SELECT key,value FROM jsonb_each(checked_row->1) LOOP
    field_shape:=row_shapes->(checked_row->>0)->row_key;
    IF row_value='null'::jsonb THEN
     IF field_shape->1 IS DISTINCT FROM 'true'::jsonb THEN
      RAISE EXCEPTION 'native_group_process.material_unavailable';
     END IF;
     CONTINUE;
    END IF;
    value_text:=row_value#>>'{}';
    CASE field_shape->>0
     WHEN 'uuid' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string'
       OR value_text !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
       OR value_text='00000000-0000-0000-0000-000000000000' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'integer' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'number' OR value_text !~ '^(0|[1-9][0-9]{0,18})$' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
      IF value_text::numeric>9223372036854775807 OR (value_text='0' AND row_key<>'expected_revision') THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'timestamp' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
      parsed_time:=value_text::timestamptz;
      IF NOT isfinite(parsed_time) OR to_jsonb(parsed_time) IS DISTINCT FROM row_value THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'bytea' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' OR left(value_text,2) IS DISTINCT FROM '\x'
       OR substring(value_text FROM 3) !~ '^([0-9a-f]{2})*$'
       OR (right(row_key,6)='digest' AND length(value_text)<>66)
       OR (right(row_key,6)='sha256' AND length(value_text)<>66) THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     ELSE
      IF jsonb_typeof(row_value) IS DISTINCT FROM field_shape->>0 THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
    END CASE;
   END LOOP;
  END LOOP;
 END IF;

 SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO current_groups
  FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
 IF cardinality(current_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 IF p_original IS NOT NULL THEN
  SELECT coalesce(array_agg(id ORDER BY id),ARRAY[]::uuid[]) INTO guarded_groups
   FROM (SELECT DISTINCT unnest(old_groups||current_groups) id) required_guards;
  FOREACH selected_group IN ARRAY guarded_groups LOOP
   PERFORM public.native_group_process_group_guard_v1(selected_group,false);
  END LOOP;
  FOREACH selected_group IN ARRAY guarded_groups LOOP
   PERFORM 1 FROM public.group_authority_lock_shared_v1(selected_group);
  END LOOP;
  SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO after_groups
   FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
  IF cardinality(after_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF NOT after_groups<@guarded_groups THEN
   RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
    'session_id',p_family,'status','CONFLICT','candidates','[]'::jsonb,'snapshot',NULL);
  END IF;
  current_groups:=after_groups;
 END IF;

 -- Enumeration takes no Group locks. Account guards end with this transaction
 -- before per-candidate Landing7 scopes. Final recheck has taken all Group
 -- guards already; neither branch acquires a new Group guard after Account.
 material:=public.native_group_process_account_material_v1(p_actor,p_family);
 SELECT a.* INTO STRICT root FROM public.accounts a WHERE a.id=p_actor;
 SELECT s.* INTO STRICT security FROM public.account_security s WHERE s.account_id=p_actor;
 SELECT e.* INTO STRICT enrollment FROM public.account_security_events e WHERE e.account_id=p_actor AND e.kind='ENROLLED';
 SELECT f.* INTO STRICT family FROM public.auth_account_session_shared_material_v1(p_actor,p_family) f;
 SELECT h.* INTO STRICT terms FROM public.account_terms_registration_head_v1() h;
 FOR item IN SELECT c.* FROM public.account_login_consent_v1(p_actor) c ORDER BY c.terms_kind COLLATE "C" LIMIT 9 LOOP
  IF jsonb_array_length(consent_value)>=8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  additional:=octet_length(convert_to(to_jsonb(item)::text,'UTF8'))+2;
  IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  consent_value:=consent_value||jsonb_build_array(to_jsonb(item)); budget:=budget-additional;
 END LOOP;
 IF jsonb_array_length(consent_value)=0 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 account_value:=jsonb_build_object('actor_account_id',p_actor,'session_id',p_family,
  'root',to_jsonb(root),'security',to_jsonb(security),'family',to_jsonb(family),
  'enrollment',to_jsonb(enrollment),'consent',consent_value,'terms_head',to_jsonb(terms));
 PERFORM public.account_company_setup_eligibility_v1(p_actor);
 SELECT h.* INTO designation FROM public.deployment_operator_head h WHERE h.singleton=1 FOR SHARE OF h;
 IF FOUND THEN
  SELECT r.* INTO STRICT designation_receipt FROM public.deployment_operator_receipts r WHERE r.receipt_id=designation.receipt_id;
  IF (designation.receipt_id,designation.revision,designation.account_id,designation.system_identifier,designation.database_name,designation.database_oid)
   IS DISTINCT FROM (designation_receipt.receipt_id,designation_receipt.revision,designation_receipt.account_id,
    designation_receipt.system_identifier,designation_receipt.database_name,designation_receipt.database_oid)
   OR designation_receipt.expected_revision IS DISTINCT FROM designation_receipt.revision-1
   OR ((designation_receipt.kind='DESIGNATE' AND designation_receipt.revision=1
     AND designation_receipt.expected_security_generation>0 AND designation_receipt.reason IS NULL)
    OR (designation_receipt.kind='REVOKE' AND designation_receipt.revision>1
     AND designation_receipt.expected_security_generation IS NULL AND designation_receipt.reason IS NOT NULL
     AND octet_length(designation_receipt.reason) BETWEEN 1 AND 512
     AND length(btrim(designation_receipt.reason,E' \t\n\r\f\013'))>0)) IS DISTINCT FROM true
   OR designation.system_identifier IS DISTINCT FROM (SELECT system_identifier::text FROM pg_control_system())
   OR designation.database_name IS DISTINCT FROM current_database()::text
   OR designation.database_oid IS DISTINCT FROM (SELECT oid::bigint FROM pg_database WHERE datname=current_database()) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  designation_value:=jsonb_build_object('head',to_jsonb(designation),'receipt',to_jsonb(designation_receipt));
 END IF;
 captured:=jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_SOURCE_V1',
  'account',account_value,'designation',designation_value,'groups',groups_value);
 -- The base already contains consent. Reset to its exact canonical byte size;
 -- subsequent accounting replaces the empty array with bounded complete rows.
 budget:=1048576-octet_length(convert_to(captured::text,'UTF8'));
 IF budget<0 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;

 FOREACH selected_group IN ARRAY current_groups LOOP
  -- One SPI snapshot binds policy/head/content. History is the immutable prefix
  -- ending at that captured head, even when enumeration races a later commit.
  SELECT jsonb_build_object('group_id',g.id,'group_incarnation',t.incarnation,
    'group',to_jsonb(g),'topology',to_jsonb(t),'birth_request',to_jsonb(q),'birth_receipt',to_jsonb(r),
    'birth_designation_receipt',to_jsonb(d),'source',public.native_group_process_source_v1(g.id,t.incarnation),
    'policy',to_jsonb(p),'head',to_jsonb(h),'version',to_jsonb(v),'history','[]'::jsonb)
   INTO STRICT group_value FROM public.groups g
   LEFT JOIN public.group_authority_heads t ON t.group_id=g.id
   LEFT JOIN public.company_enrollment_receipts r ON
    (r.account_id,r.command_id,r.receipt_id,r.group_id)=(g.origin_account_id,g.origin_command_id,g.origin_receipt_id,g.id)
   LEFT JOIN public.company_enrollment_requests q ON
    (q.account_id,q.command_id,q.committed_receipt_id,q.state)=(r.account_id,r.command_id,r.receipt_id,'COMMITTED')
   LEFT JOIN public.deployment_operator_receipts d ON d.receipt_id=r.designation_receipt_id
   LEFT JOIN public.native_group_identity_policy_heads_v1 p ON (p.group_id,p.group_incarnation)=(g.id,t.incarnation)
   LEFT JOIN public.native_group_process_heads_v1 h ON (h.group_id,h.group_incarnation)=(g.id,t.incarnation)
   LEFT JOIN public.native_group_process_versions_v1 v ON
    (v.group_id,v.group_incarnation,v.process_id,v.version)=(h.group_id,h.group_incarnation,h.process_id,h.content_version)
   WHERE g.id=selected_group;
  incarnation:=(group_value->>'group_incarnation')::uuid;
  IF incarnation IS NULL OR incarnation='00000000-0000-0000-0000-000000000000'::uuid THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  FOREACH required IN ARRAY ARRAY['group','topology','birth_request','birth_receipt','birth_designation_receipt','source'] LOOP
   IF jsonb_typeof(group_value->required) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  END LOOP;
  IF group_value#>>'{birth_designation_receipt,kind}' IS DISTINCT FROM 'DESIGNATE'
   OR group_value#>>'{birth_designation_receipt,system_identifier}' IS DISTINCT FROM (SELECT system_identifier::text FROM pg_control_system())
   OR group_value#>>'{birth_designation_receipt,database_name}' IS DISTINCT FROM current_database()::text
   OR group_value#>>'{birth_designation_receipt,database_oid}' IS DISTINCT FROM (SELECT oid::bigint::text FROM pg_database WHERE datname=current_database()) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  SELECT q.* INTO STRICT birth_request FROM public.company_enrollment_requests q WHERE
   (q.account_id,q.command_id)=((group_value#>>'{group,origin_account_id}')::uuid,(group_value#>>'{group,origin_command_id}')::uuid);
  SELECT r.* INTO STRICT birth_receipt FROM public.company_enrollment_receipts r WHERE
   (r.account_id,r.command_id)=(birth_request.account_id,birth_request.command_id);
  SELECT b.* INTO STRICT birth_effect FROM public.company_enrollment_effect_bindings b WHERE
   (b.account_id,b.command_id)=(birth_request.account_id,birth_request.command_id);
  IF to_jsonb(birth_request) IS DISTINCT FROM group_value->'birth_request'
   OR to_jsonb(birth_receipt) IS DISTINCT FROM group_value->'birth_receipt'
   OR birth_request.state IS DISTINCT FROM 'COMMITTED'
   OR (birth_request.codec_version,birth_request.input_digest,birth_request.designation_receipt_id,
       birth_request.committed_receipt_id,birth_request.terminal_at)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.input_digest,birth_receipt.designation_receipt_id,
       birth_receipt.receipt_id,birth_receipt.committed_at)
   OR (birth_effect.codec_version,birth_effect.input_digest,birth_effect.designation_receipt_id,
       birth_effect.org_id,birth_effect.group_id,birth_effect.receipt_id,birth_effect.administrative_account_id,
       birth_effect.catalog_version,birth_effect.manifest_digest,birth_effect.session_id,birth_effect.started_at)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.input_digest,birth_receipt.designation_receipt_id,
       birth_receipt.org_id,birth_receipt.group_id,birth_receipt.receipt_id,birth_receipt.administrative_account_id,
       birth_receipt.catalog_version,birth_receipt.manifest_digest,birth_receipt.session_id,birth_receipt.committed_at)
   OR birth_receipt.codec_version<>1 OR birth_receipt.root_revision<>1
   OR birth_receipt.catalog_version IS DISTINCT FROM 'native-company-identity-2026-09-19.1'
   OR birth_receipt.manifest_digest IS DISTINCT FROM decode('0d3d0c3bc0357c0394b02400295f77231178cd5dc22a668a90880fc92a089935','hex')
   OR birth_receipt.committed_at<birth_request.created_at OR birth_receipt.committed_at>=birth_request.expires_at
   OR public.native_group_process_micros_v1(birth_receipt.committed_at) IS DISTINCT FROM
      public.native_group_process_micros_v1((group_value#>>'{group,created_at}')::timestamptz)
   OR group_value#>>'{group,created_at}' IS DISTINCT FROM group_value#>>'{group,updated_at}'
   OR birth_receipt.account_id::text IS DISTINCT FROM group_value#>>'{birth_designation_receipt,account_id}'
   OR group_value#>>'{birth_designation_receipt,revision}' IS DISTINCT FROM '1'
   OR group_value#>>'{birth_designation_receipt,expected_revision}' IS DISTINCT FROM '0'
   OR group_value#>>'{birth_designation_receipt,expected_security_generation}' IS NULL
   OR (group_value#>>'{birth_designation_receipt,expected_security_generation}')::bigint<1
   OR group_value#>'{birth_designation_receipt,reason}' IS DISTINCT FROM 'null'::jsonb
   OR (SELECT count(*) FROM public.company_enrollment_request_events e
      WHERE (e.account_id,e.command_id)=(birth_request.account_id,birth_request.command_id))<>2
   OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE (e.account_id,e.command_id,e.event_revision)=(birth_request.account_id,birth_request.command_id,1)
      AND e.from_state IS NULL AND e.to_state='PENDING' AND e.reason_code='PREPARED'
      AND e.occurred_at=birth_request.created_at AND e.actor_account_id=birth_request.account_id)
   OR NOT EXISTS(SELECT 1 FROM public.company_enrollment_request_events e
     WHERE (e.account_id,e.command_id,e.event_revision)=(birth_request.account_id,birth_request.command_id,2)
      AND e.from_state='PENDING' AND e.to_state='COMMITTED' AND e.reason_code='COMMITTED'
      AND e.occurred_at=birth_receipt.committed_at AND e.actor_account_id=birth_receipt.account_id
      AND e.session_id=birth_receipt.session_id) THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  -- Retention may remove terminal payload; decode only retained historical bytes.
  IF birth_request.input_bytes IS NOT NULL THEN
   SELECT * INTO STRICT decoded_birth FROM public.company_enrollment_decode_input_v1(birth_request.input_bytes);
   IF (decoded_birth.codec_version,decoded_birth.account_id,decoded_birth.command_id,decoded_birth.input_digest,
       decoded_birth.administrative_account_id,decoded_birth.company_name)
    IS DISTINCT FROM (birth_receipt.codec_version,birth_receipt.account_id,birth_receipt.command_id,birth_receipt.input_digest,
       birth_receipt.administrative_account_id,group_value#>>'{group,name}')
    OR decoded_birth.group_id IS NOT NULL THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END IF;
  additional:=octet_length(convert_to(group_value::text,'UTF8'))+CASE WHEN first_group THEN 0 ELSE 2 END;
  IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  budget:=budget-additional;
  history_value:='[]'::jsonb; first_history:=true;
  head_revision:=(group_value#>>'{head,head_revision}')::bigint;
  FOR item IN SELECT h.*,i.operation,i.input_bytes FROM public.native_group_process_head_revisions_v1 h
   LEFT JOIN public.native_group_process_inputs_v1 i ON (i.actor_account_id,i.command_id)=(h.last_actor_account_id,h.last_command_id)
   WHERE h.group_id=selected_group AND h.group_incarnation=incarnation AND h.head_revision<=head_revision ORDER BY h.head_revision
  LOOP
   IF item.operation IS NULL OR item.input_bytes IS NULL THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   candidate:=jsonb_build_object('head',to_jsonb(item)-ARRAY['operation','input_bytes'],
    'reason',CASE WHEN item.operation=6 THEN public.native_group_process_decode_v1(item.input_bytes)->>'reason' END);
   additional:=octet_length(convert_to(candidate::text,'UTF8'))+CASE WHEN first_history THEN 0 ELSE 2 END;
   IF additional>budget THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   budget:=budget-additional;
   history_value:=history_value||jsonb_build_array(candidate); first_history:=false;
  END LOOP;
  -- Full owner closure work starts only after the complete history fits.
  PERFORM public.native_group_process_assert_current_v1(selected_group,incarnation);
  group_value:=jsonb_set(group_value,'{history}',history_value);
  groups_value:=groups_value||jsonb_build_array(group_value); first_group:=false;
  candidates:=candidates||jsonb_build_array(jsonb_build_object('group_id',selected_group,'group_incarnation',incarnation));
 END LOOP;
 captured:=jsonb_set(captured,'{groups}',groups_value);
 snapshot_bytes:=convert_to(captured::text,'UTF8');
 IF octet_length(snapshot_bytes) NOT BETWEEN 1 AND 1048576 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;

 -- Recheck the actual row shapes too: adding a column cannot silently broaden
 -- this ephemeral internal projection. The same frozen parser governs both.
 document:=captured;
  IF jsonb_typeof(document->'account') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  IF ARRAY(SELECT key FROM jsonb_object_keys(document->'account') key ORDER BY key COLLATE "C")
   IS DISTINCT FROM ARRAY['actor_account_id','consent','enrollment','family','root','security','session_id','terms_head']
   OR document#>>'{account,actor_account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,session_id}' IS DISTINCT FROM p_family::text
   OR document#>>'{account,root,id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,security,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,family,user_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,account_id}' IS DISTINCT FROM p_actor::text
   OR document#>>'{account,enrollment,kind}' IS DISTINCT FROM 'ENROLLED'
   OR jsonb_typeof(document#>'{account,consent}') IS DISTINCT FROM 'array' THEN
   RAISE EXCEPTION 'native_group_process.material_unavailable';
  END IF;
  IF jsonb_array_length(document#>'{account,consent}') NOT BETWEEN 1 AND 8 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
  rows_to_validate:=jsonb_build_array(jsonb_build_array('accounts',document#>'{account,root}'),
   jsonb_build_array('account_security',document#>'{account,security}'),
   jsonb_build_array('family',document#>'{account,family}'),
   jsonb_build_array('account_security_events',document#>'{account,enrollment}'),
   jsonb_build_array('terms_head',document#>'{account,terms_head}'));
  required:=NULL;
  FOR entry IN SELECT value FROM jsonb_array_elements(document#>'{account,consent}') LOOP
   IF jsonb_typeof(entry->'terms_kind') IS DISTINCT FROM 'string'
    OR entry->>'terms_kind' !~ '^[a-z][a-z0-9_.-]{0,63}$'
    OR (required IS NOT NULL AND (entry->>'terms_kind') COLLATE "C"<=required COLLATE "C") THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   required:=entry->>'terms_kind'; rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('consent',entry));
  END LOOP;
  IF document->'designation' IS DISTINCT FROM 'null'::jsonb THEN
   IF jsonb_typeof(document->'designation') IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   IF ARRAY(SELECT key FROM jsonb_object_keys(document->'designation') key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','receipt'] THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('deployment_operator_head',document#>'{designation,head}'),
    jsonb_build_array('deployment_operator_receipts',document#>'{designation,receipt}'));
  END IF;
  FOR candidate IN SELECT value FROM jsonb_array_elements(document->'groups') LOOP
   IF ARRAY(SELECT key FROM jsonb_object_keys(candidate) key ORDER BY key COLLATE "C")
    IS DISTINCT FROM ARRAY['birth_designation_receipt','birth_receipt','birth_request','group','group_id','group_incarnation','head','history','policy','source','topology','version']
    OR candidate#>>'{group,id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR candidate#>>'{topology,incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
    OR candidate#>>'{birth_receipt,group_id}' IS DISTINCT FROM candidate->>'group_id'
    OR jsonb_typeof(candidate->'history') IS DISTINCT FROM 'array' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   IF (candidate->'policy'='null'::jsonb) IS DISTINCT FROM (candidate->'head'='null'::jsonb)
    OR (candidate->'head'='null'::jsonb) IS DISTINCT FROM (candidate->'version'='null'::jsonb)
    OR ((candidate->'head'='null'::jsonb) IS DISTINCT FROM (jsonb_array_length(candidate->'history')=0)) THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   rows_to_validate:=rows_to_validate||jsonb_build_array(
    jsonb_build_array('groups',candidate->'group'),jsonb_build_array('group_authority_heads',candidate->'topology'),
    jsonb_build_array('company_enrollment_requests',candidate->'birth_request'),
    jsonb_build_array('company_enrollment_receipts',candidate->'birth_receipt'),
    jsonb_build_array('deployment_operator_receipts',candidate->'birth_designation_receipt'),
    jsonb_build_array('source',candidate->'source'));
   IF candidate->'policy' IS DISTINCT FROM 'null'::jsonb THEN
    rows_to_validate:=rows_to_validate||jsonb_build_array(
     jsonb_build_array('native_group_identity_policy_heads_v1',candidate->'policy'),
     jsonb_build_array('native_group_process_heads_v1',candidate->'head'),
     jsonb_build_array('native_group_process_versions_v1',candidate->'version'));
   END IF;
   head_revision:=0;
   FOR entry IN SELECT value FROM jsonb_array_elements(candidate->'history') LOOP
    IF jsonb_typeof(entry) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    IF ARRAY(SELECT key FROM jsonb_object_keys(entry) key ORDER BY key COLLATE "C") IS DISTINCT FROM ARRAY['head','reason']
     OR jsonb_typeof(entry->'reason') NOT IN ('string','null')
     OR entry#>>'{head,group_id}' IS DISTINCT FROM candidate->>'group_id'
     OR entry#>>'{head,group_incarnation}' IS DISTINCT FROM candidate->>'group_incarnation'
     OR entry#>>'{head,process_id}' IS DISTINCT FROM candidate#>>'{head,process_id}' THEN
     RAISE EXCEPTION 'native_group_process.material_unavailable';
    END IF;
    revision_value:=entry#>>'{head,head_revision}';
    IF revision_value IS NULL OR revision_value !~ '^[1-9][0-9]{0,18}$' OR revision_value::numeric>9223372036854775807
     OR revision_value::numeric<>head_revision::numeric+1 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
    head_revision:=revision_value::bigint;
    rows_to_validate:=rows_to_validate||jsonb_build_array(jsonb_build_array('native_group_process_head_revisions_v1',entry->'head'));
   END LOOP;
   IF head_revision>0 AND (candidate->'history'->-1->'head') IS DISTINCT FROM candidate->'head' THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
  END LOOP;
  FOR checked_row IN SELECT value FROM jsonb_array_elements(rows_to_validate) LOOP
   IF jsonb_typeof(checked_row->1) IS DISTINCT FROM 'object' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
   expected_keys:=ARRAY(SELECT key FROM jsonb_object_keys(row_shapes->(checked_row->>0)) key ORDER BY key COLLATE "C");
   IF cardinality(expected_keys)=0 OR ARRAY(SELECT key FROM jsonb_object_keys(checked_row->1) key ORDER BY key COLLATE "C") IS DISTINCT FROM expected_keys THEN
    RAISE EXCEPTION 'native_group_process.material_unavailable';
   END IF;
   FOR row_key,row_value IN SELECT key,value FROM jsonb_each(checked_row->1) LOOP
    field_shape:=row_shapes->(checked_row->>0)->row_key;
    IF row_value='null'::jsonb THEN
     IF field_shape->1 IS DISTINCT FROM 'true'::jsonb THEN
      RAISE EXCEPTION 'native_group_process.material_unavailable';
     END IF;
     CONTINUE;
    END IF;
    value_text:=row_value#>>'{}';
    CASE field_shape->>0
     WHEN 'uuid' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string'
       OR value_text !~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
       OR value_text='00000000-0000-0000-0000-000000000000' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'integer' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'number' OR value_text !~ '^(0|[1-9][0-9]{0,18})$' THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
      IF value_text::numeric>9223372036854775807 OR (value_text='0' AND row_key<>'expected_revision') THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'timestamp' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
      parsed_time:=value_text::timestamptz;
      IF NOT isfinite(parsed_time) OR to_jsonb(parsed_time) IS DISTINCT FROM row_value THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     WHEN 'bytea' THEN
      IF jsonb_typeof(row_value) IS DISTINCT FROM 'string' OR left(value_text,2) IS DISTINCT FROM '\x'
       OR substring(value_text FROM 3) !~ '^([0-9a-f]{2})*$'
       OR (right(row_key,6)='digest' AND length(value_text)<>66)
       OR (right(row_key,6)='sha256' AND length(value_text)<>66) THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
     ELSE
      IF jsonb_typeof(row_value) IS DISTINCT FROM field_shape->>0 THEN
       RAISE EXCEPTION 'native_group_process.material_unavailable';
      END IF;
    END CASE;
   END LOOP;
  END LOOP;
 SELECT coalesce(array_agg(g.id ORDER BY g.id),ARRAY[]::uuid[]) INTO after_groups
  FROM (SELECT id FROM public.groups WHERE num_nonnulls(origin_account_id,origin_command_id,origin_receipt_id)>0 ORDER BY id LIMIT 257) g;
 IF cardinality(after_groups)>256 THEN RAISE EXCEPTION 'native_group_process.material_unavailable'; END IF;
 IF p_original IS NOT NULL AND (after_groups IS DISTINCT FROM current_groups OR snapshot_bytes IS DISTINCT FROM p_original) THEN
  RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
   'session_id',p_family,'status','CONFLICT','candidates','[]'::jsonb,'snapshot',NULL);
 END IF;
 RETURN jsonb_build_object('protocol','GROUP_PROCESS_NAVIGATION_V1','actor_account_id',p_actor,
  'session_id',p_family,'status','MATCH','candidates',candidates,'snapshot',snapshot_bytes);
EXCEPTION WHEN no_data_found OR too_many_rows THEN
 RAISE EXCEPTION 'native_group_process.material_unavailable';
END
$body$;

-- source: ops/native-group-process/acl-v1.sql
-- Uninstalled exact Group ABI/ACL roster. No new owner role or membership.
-- Complete custody still verifies effective privileges/defaults and all source.
DO $acl$
DECLARE routine record; grantee_name text; relation text; column_name text;
BEGIN
 FOR routine IN SELECT * FROM (VALUES
  ('public.native_group_process_uuid_v1(bytea,integer)',NULL::text),
  ('public.native_group_process_i64_v1(bytea,integer)',NULL::text),
  ('public.native_group_process_text_v1(text,integer)',NULL::text),
  ('public.native_group_process_micros_v1(timestamptz)',NULL::text),
  ('public.native_group_process_decode_v1(bytea)',NULL::text),
  ('public.native_group_process_policy_ref_v1(smallint,bigint,bytea)',NULL::text),
  ('public.native_group_process_head_ref_v1(uuid,bigint,bigint,bytea,bytea,text,timestamptz)',NULL::text),
  ('public.native_group_process_action_roster_v1()',NULL::text),
  ('public.native_group_process_registration_bytes_v1(uuid,uuid,text,bytea,bytea,bytea,jsonb)',NULL::text),
  ('public.native_group_process_policy_head_bytes_v1(public.native_group_identity_policy_heads_v1)',NULL::text),
  ('public.native_group_process_version_bytes_v1(public.native_group_process_versions_v1)',NULL::text),
  ('public.native_group_process_head_bytes_v1(public.native_group_process_head_revisions_v1)',NULL::text),
  ('public.native_group_process_result_bytes_v2(public.native_group_process_results_v1)',NULL::text),
  ('public.native_group_process_registered_actions_v1()',NULL::text),
  ('public.native_group_process_source_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_group_guard_v1(uuid,boolean)',NULL::text),
  ('public.native_group_process_command_guard_v1(uuid,uuid,boolean)',NULL::text),
  ('public.native_group_process_topology_fence_v1()',NULL::text),
  ('public.native_group_process_account_material_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_original_material_v1(uuid,uuid,uuid,uuid)',NULL::text),
  ('public.identity_native_group_process_material_v1(uuid,uuid,uuid,uuid,uuid,smallint,bytea)','console_rt'),
  ('public.identity_native_group_process_incarnation_selector_v1(uuid,uuid,uuid,uuid)','console_rt'),
  ('public.identity_native_group_process_navigation_candidates_v1(uuid,uuid,bytea)','console_rt'),
  ('public.native_group_process_accept_snapshot_v1(public.native_group_process_inputs_v1)',NULL::text),
  ('public.native_group_process_complete_snapshot_v1(public.native_group_process_results_v1)',NULL::text),
  ('public.native_group_process_assert_input_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_assert_result_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_assert_current_v1(uuid,uuid)',NULL::text),
  ('public.native_group_process_audit_guard_v1()',NULL::text),
  ('public.native_group_process_audit_truncate_guard_v1()',NULL::text),
  ('public.native_group_process_current_context_v1(uuid,uuid,uuid,uuid,boolean)',NULL::text),
  ('public.native_group_process_classify_v1(public.native_group_process_effects_v1,jsonb)',NULL::text),
  ('public.native_group_process_validate_frame_v1(public.native_group_process_effects_v1)',NULL::text),
  ('public.native_group_process_input_guard_v1()',NULL::text),
  ('public.native_group_process_effect_guard_v1()',NULL::text),
  ('public.native_group_process_participant_guard_v1()',NULL::text),
  ('public.native_group_process_deferred_guard_v1()',NULL::text),
  ('public.native_group_process_immutable_statement_v1()',NULL::text),
  ('public.native_group_process_head_statement_v1()',NULL::text),
  ('public.identity_native_group_process_prepare_v1(uuid,uuid,uuid,bytea)','console_rt'),
  ('public.identity_native_group_process_execute_v1(uuid,uuid,uuid)','console_rt')
 ) required(signature,executor_name) LOOP
  EXECUTE format('ALTER FUNCTION %s OWNER TO console_account_owner',routine.signature);
  EXECUTE format('REVOKE ALL ON FUNCTION %s FROM PUBLIC',routine.signature);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_proc p
   CROSS JOIN LATERAL aclexplode(p.proacl) a JOIN pg_roles r ON r.oid=a.grantee
   WHERE p.oid=routine.signature::regprocedure AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL ON FUNCTION %s FROM %I',routine.signature,grantee_name); END LOOP;
  IF routine.executor_name IS NOT NULL THEN
   EXECUTE format('GRANT EXECUTE ON FUNCTION %s TO %I',routine.signature,routine.executor_name);
  END IF;
 END LOOP;
 FOREACH relation IN ARRAY ARRAY['native_group_process_inputs_v1','native_group_process_effects_v1','native_group_process_results_v1',
  'native_group_identity_policy_heads_v1','native_group_process_versions_v1','native_group_process_head_revisions_v1','native_group_process_heads_v1'] LOOP
  EXECUTE format('REVOKE ALL ON TABLE public.%I FROM PUBLIC',relation);
  FOR grantee_name IN SELECT DISTINCT r.rolname FROM pg_class c
   CROSS JOIN LATERAL aclexplode(c.relacl) a JOIN pg_roles r ON r.oid=a.grantee
   WHERE c.oid=format('public.%I',relation)::regclass AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL ON TABLE public.%I FROM %I',relation,grantee_name); END LOOP;
  FOR column_name IN SELECT a.attname FROM pg_attribute a
   WHERE a.attrelid=format('public.%I',relation)::regclass AND a.attnum>0 AND NOT a.attisdropped
  LOOP EXECUTE format('REVOKE ALL (%I) ON TABLE public.%I FROM PUBLIC',column_name,relation); END LOOP;
  FOR column_name,grantee_name IN SELECT DISTINCT a.attname,r.rolname FROM pg_attribute a
   CROSS JOIN LATERAL aclexplode(a.attacl) grant_entry JOIN pg_roles r ON r.oid=grant_entry.grantee
   WHERE a.attrelid=format('public.%I',relation)::regclass AND a.attnum>0 AND NOT a.attisdropped
    AND r.rolname<>'console_account_owner'
  LOOP EXECUTE format('REVOKE ALL (%I) ON TABLE public.%I FROM %I',column_name,relation,grantee_name); END LOOP;
 END LOOP;
END
$acl$;
