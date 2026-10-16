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
