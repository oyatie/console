-- Additive native Company business policy. Installed only by the atomic custody
-- successor after exact prior-profile verification; never source independently.
CREATE TABLE public.native_company_policy_inputs_v1 (
 actor_account_id uuid NOT NULL REFERENCES public.accounts(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 command_id uuid NOT NULL CHECK(command_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 org_id uuid NOT NULL REFERENCES public.organizations(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
 operation smallint NOT NULL CHECK(operation BETWEEN 1 AND 3),
 codec_version smallint NOT NULL CHECK(codec_version=1),
 input_bytes bytea NOT NULL CHECK(octet_length(input_bytes) IN (123,148,155,180)),
 input_digest bytea NOT NULL CHECK(input_digest=sha256(input_bytes)),
 intake_receipt_id uuid NOT NULL UNIQUE CHECK(intake_receipt_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 accepted_at timestamptz NOT NULL CHECK(isfinite(accepted_at)),
 execution_not_after timestamptz NOT NULL CHECK(execution_not_after=accepted_at+interval '168 hours'),
 accepting_session_id uuid NOT NULL CHECK(accepting_session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 acceptance_xid xid8 NOT NULL,
 acceptance_backend_pid integer NOT NULL CHECK(acceptance_backend_pid>0),
 PRIMARY KEY(actor_account_id,command_id),
 UNIQUE(actor_account_id,command_id,org_id,operation,codec_version,intake_receipt_id,input_digest)
);
CREATE INDEX native_company_policy_inputs_v1_outstanding
 ON public.native_company_policy_inputs_v1(org_id,execution_not_after,actor_account_id,command_id);

CREATE TABLE public.native_company_policy_receipts_v1 (
 actor_account_id uuid NOT NULL,
 command_id uuid NOT NULL,
 org_id uuid NOT NULL,
 operation smallint NOT NULL CHECK(operation BETWEEN 1 AND 3),
 codec_version smallint NOT NULL CHECK(codec_version=1),
 intake_receipt_id uuid NOT NULL,
 input_digest bytea NOT NULL CHECK(octet_length(input_digest)=32),
 receipt_id uuid NOT NULL UNIQUE CHECK(receipt_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 outcome text NOT NULL CHECK(outcome IN ('COMMITTED','REJECTED')),
 result_code text NOT NULL,
 execution_session_id uuid NOT NULL CHECK(execution_session_id<>'00000000-0000-0000-0000-000000000000'::uuid),
 executed_at timestamptz NOT NULL CHECK(isfinite(executed_at)),
 effect_xid xid8 NOT NULL,
 effect_backend_pid integer NOT NULL CHECK(effect_backend_pid>0),
 epoch_before bigint NOT NULL CHECK(epoch_before>=1),
 epoch_after bigint NOT NULL CHECK(epoch_after>=1),
 catalog_version text NOT NULL CHECK(catalog_version='native-payroll-collection-read-v1'),
 manifest_digest bytea NOT NULL CHECK(manifest_digest=decode('07781514029d5f8f7e96221d6504387324c8c0f2513ded2b214d0bd683ce3ddd','hex')),
 predecessor_receipt_id uuid,
 committed_epoch bigint GENERATED ALWAYS AS (CASE WHEN outcome='COMMITTED' THEN epoch_after END) STORED,
 installed_object_type_id uuid,
 recipient_account_id uuid,
 role_id uuid,
 role_revision bigint,
 assignment_id uuid,
 assignment_revision_before bigint,
 assignment_revision_after bigint,
 assignment_state_after text,
 assignment_valid_from timestamptz,
 assignment_valid_until timestamptz,
 PRIMARY KEY(actor_account_id,command_id),
 UNIQUE(org_id,receipt_id),
 UNIQUE(org_id,committed_epoch),
 UNIQUE(org_id,receipt_id,committed_epoch),
 FOREIGN KEY(actor_account_id,command_id,org_id,operation,codec_version,intake_receipt_id,input_digest)
  REFERENCES public.native_company_policy_inputs_v1(actor_account_id,command_id,org_id,operation,codec_version,intake_receipt_id,input_digest)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,predecessor_receipt_id,epoch_before)
  REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id,committed_epoch)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,installed_object_type_id) REFERENCES public.ont_object_types(org_id,id)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(org_id,recipient_account_id) REFERENCES public.company_actors(org_id,account_id)
  ON UPDATE RESTRICT ON DELETE RESTRICT,
 FOREIGN KEY(org_id,role_id,role_revision) REFERENCES public.policy_role_revisions(org_id,role_id,revision)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 FOREIGN KEY(org_id,assignment_id,assignment_revision_after)
  REFERENCES public.policy_assignment_revisions(org_id,assignment_id,revision)
  ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 CHECK((epoch_before=1)=(predecessor_receipt_id IS NULL)),
 CHECK(((outcome='REJECTED' AND epoch_after=epoch_before
   AND result_code IN ('intake_expired','revision_conflict','grant_expiry_invalid','recipient_ineligible')
   AND num_nonnulls(installed_object_type_id,recipient_account_id,role_id,role_revision,assignment_id,
    assignment_revision_before,assignment_revision_after,assignment_state_after,assignment_valid_from,assignment_valid_until)=0)
  OR (outcome='COMMITTED' AND epoch_after::numeric=epoch_before::numeric+1
   AND ((operation=1 AND result_code='installed' AND installed_object_type_id IS NOT NULL
     AND num_nonnulls(recipient_account_id,role_id,role_revision,assignment_id,assignment_revision_before,
      assignment_revision_after,assignment_state_after,assignment_valid_from,assignment_valid_until)=0)
    OR (operation IN (2,3) AND installed_object_type_id IS NULL AND recipient_account_id IS NOT NULL
     AND role_id IS NOT NULL AND role_revision=1 AND assignment_id IS NOT NULL
     AND assignment_revision_after IS NOT NULL AND assignment_revision_after>=1
     AND ((assignment_revision_before IS NULL AND operation=2 AND assignment_revision_after=1)
       OR (assignment_revision_before>=1 AND assignment_revision_after::numeric=assignment_revision_before::numeric+1))
     AND assignment_valid_from IS NOT NULL AND isfinite(assignment_valid_from)
     AND assignment_valid_until IS NOT NULL AND isfinite(assignment_valid_until)
     AND assignment_valid_until>assignment_valid_from
     AND ((operation=2 AND result_code='granted' AND assignment_state_after='ACTIVE')
       OR (operation=3 AND result_code='revoked' AND assignment_state_after='REVOKED')))))) IS TRUE)
);

ALTER TABLE public.policy_roles ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT policy_roles_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.policy_role_revisions ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT policy_role_revisions_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.user_role_assignments ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT user_role_assignments_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.policy_assignment_revisions ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT policy_assignment_revisions_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.native_company_catalog_installs ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT native_company_catalog_installs_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.ont_builtin_catalog_installs ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT ont_builtin_catalog_installs_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.ont_object_types ADD COLUMN policy_receipt_id uuid,
 ADD CONSTRAINT ont_object_types_policy_receipt_fk FOREIGN KEY(org_id,policy_receipt_id)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE public.company_authority_heads ADD COLUMN current_policy_receipt_id uuid,
 ADD CONSTRAINT company_authority_heads_policy_receipt_fk FOREIGN KEY(org_id,current_policy_receipt_id,epoch)
 REFERENCES public.native_company_policy_receipts_v1(org_id,receipt_id,committed_epoch)
 ON UPDATE RESTRICT ON DELETE RESTRICT DEFERRABLE INITIALLY DEFERRED,
 ADD CONSTRAINT company_authority_heads_policy_receipt_shape CHECK((epoch=1)=(current_policy_receipt_id IS NULL));

ALTER TABLE public.native_company_policy_inputs_v1 OWNER TO console_account_owner;
ALTER TABLE public.native_company_policy_receipts_v1 OWNER TO console_account_owner;
REVOKE ALL ON public.native_company_policy_inputs_v1,public.native_company_policy_receipts_v1 FROM PUBLIC;
ALTER TABLE public.native_company_policy_inputs_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_policy_inputs_v1 FORCE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_policy_receipts_v1 ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.native_company_policy_receipts_v1 FORCE ROW LEVEL SECURITY;
CREATE POLICY org_isolation ON public.native_company_policy_inputs_v1
 USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid)
 WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
CREATE POLICY org_isolation ON public.native_company_policy_receipts_v1
 USING(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid)
 WITH CHECK(org_id=NULLIF(current_setting('app.current_org',true),'')::uuid);
