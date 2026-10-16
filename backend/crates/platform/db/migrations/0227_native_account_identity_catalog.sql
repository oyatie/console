-- Native identity catalogs staged without runtime writers. Account foreign
-- keys and final ownership are attached by the atomic privileged finalizer;
-- the migration login cannot alter an already-finalized Account root.
CREATE TABLE public.company_actors (
    org_id uuid NOT NULL REFERENCES public.organizations(id) ON UPDATE RESTRICT ON DELETE RESTRICT,
    account_id uuid NOT NULL,
    admission_receipt_id uuid NOT NULL,
    entitlement_ref jsonb NOT NULL,
    created_at timestamptz NOT NULL,
    PRIMARY KEY(org_id,account_id),
    CHECK (org_id <> '00000000-0000-0000-0000-000000000000'::uuid
       AND account_id <> '00000000-0000-0000-0000-000000000000'::uuid
       AND admission_receipt_id <> '00000000-0000-0000-0000-000000000000'::uuid),
    -- Closed ControlEvidence shape. The later admission owner additionally
    -- proves exact Account/Company receipt authority; this lane grants no writer.
    CHECK (pg_column_size(entitlement_ref) <= 16384 AND COALESCE(((jsonb_typeof(entitlement_ref)='object'
 AND entitlement_ref ?& ARRAY['kind','evidence']
 AND entitlement_ref - ARRAY['kind','evidence']='{}'::jsonb
 AND ((entitlement_ref->'kind')='"COMPANY"'::jsonb)
 AND ((jsonb_typeof((entitlement_ref->'evidence'))='object'
 AND (entitlement_ref->'evidence') ?& ARRAY['kind','revision']
 AND (entitlement_ref->'evidence') - ARRAY['kind','revision']='{}'::jsonb
 AND (((entitlement_ref->'evidence')->'kind')='"GOVERNED"'::jsonb)
 AND (jsonb_typeof(((entitlement_ref->'evidence')->'revision'))='object'
 AND ((entitlement_ref->'evidence')->'revision') ?& ARRAY['org_id','object_type_id','instance_id','revision_id','version','row_hash']
 AND ((entitlement_ref->'evidence')->'revision') - ARRAY['org_id','object_type_id','instance_id','revision_id','version','row_hash']='{}'::jsonb
 AND (NOT ((((entitlement_ref->'evidence')->'revision')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((entitlement_ref->'evidence')->'revision')->'org_id'))='string'
 AND ((((entitlement_ref->'evidence')->'revision')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((entitlement_ref->'evidence')->'revision')->'object_type_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((entitlement_ref->'evidence')->'revision')->'object_type_id'))='string'
 AND ((((entitlement_ref->'evidence')->'revision')->'object_type_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((entitlement_ref->'evidence')->'revision')->'instance_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((entitlement_ref->'evidence')->'revision')->'instance_id'))='string'
 AND ((((entitlement_ref->'evidence')->'revision')->'instance_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((entitlement_ref->'evidence')->'revision')->'revision_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((entitlement_ref->'evidence')->'revision')->'revision_id'))='string'
 AND ((((entitlement_ref->'evidence')->'revision')->'revision_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof((((entitlement_ref->'evidence')->'revision')->'version'))='string'
 AND ((((entitlement_ref->'evidence')->'revision')->'version')#>>'{}') ~ '^[1-9][0-9]{0,18}$'
 AND CASE WHEN ((((entitlement_ref->'evidence')->'revision')->'version')#>>'{}') ~ '^[1-9][0-9]{0,18}$' THEN ((((entitlement_ref->'evidence')->'revision')->'version')#>>'{}')::numeric <= 9223372036854775807 ELSE false END)
 AND (jsonb_typeof((((entitlement_ref->'evidence')->'revision')->'row_hash'))='string'
 AND ((((entitlement_ref->'evidence')->'revision')->'row_hash')#>>'{}') ~ '^[0-9a-f]{64}$')))
 OR (jsonb_typeof((entitlement_ref->'evidence'))='object'
 AND (entitlement_ref->'evidence') ?& ARRAY['kind','evidence']
 AND (entitlement_ref->'evidence') - ARRAY['kind','evidence']='{}'::jsonb
 AND (((entitlement_ref->'evidence')->'kind')='"RETAINED"'::jsonb)
 AND (jsonb_typeof(((entitlement_ref->'evidence')->'evidence'))='object'
 AND ((entitlement_ref->'evidence')->'evidence') ?& ARRAY['unit','evidence_digest','encoded_size']
 AND ((entitlement_ref->'evidence')->'evidence') - ARRAY['unit','evidence_digest','encoded_size']='{}'::jsonb
 AND (jsonb_typeof((((entitlement_ref->'evidence')->'evidence')->'unit'))='object'
 AND (((entitlement_ref->'evidence')->'evidence')->'unit') ?& ARRAY['org_id','unit_id','identity_digest']
 AND (((entitlement_ref->'evidence')->'evidence')->'unit') - ARRAY['org_id','unit_id','identity_digest']='{}'::jsonb
 AND (NOT (((((entitlement_ref->'evidence')->'evidence')->'unit')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((entitlement_ref->'evidence')->'evidence')->'unit')->'org_id'))='string'
 AND (((((entitlement_ref->'evidence')->'evidence')->'unit')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((((entitlement_ref->'evidence')->'evidence')->'unit')->'unit_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((entitlement_ref->'evidence')->'evidence')->'unit')->'unit_id'))='string'
 AND (((((entitlement_ref->'evidence')->'evidence')->'unit')->'unit_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof(((((entitlement_ref->'evidence')->'evidence')->'unit')->'identity_digest'))='string'
 AND (((((entitlement_ref->'evidence')->'evidence')->'unit')->'identity_digest')#>>'{}') ~ '^[0-9a-f]{64}$'))
 AND (jsonb_typeof((((entitlement_ref->'evidence')->'evidence')->'evidence_digest'))='string'
 AND ((((entitlement_ref->'evidence')->'evidence')->'evidence_digest')#>>'{}') ~ '^[0-9a-f]{64}$')
 AND (CASE WHEN jsonb_typeof((((entitlement_ref->'evidence')->'evidence')->'encoded_size'))='number' THEN ((((entitlement_ref->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric=trunc(((((entitlement_ref->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric) AND ((((entitlement_ref->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric>=0 AND ((((entitlement_ref->'evidence')->'evidence')->'encoded_size')#>>'{}')::numeric<=9223372036854775807 ELSE false END)))
 OR (jsonb_typeof((entitlement_ref->'evidence'))='object'
 AND (entitlement_ref->'evidence') ?& ARRAY['kind','receipt']
 AND (entitlement_ref->'evidence') - ARRAY['kind','receipt']='{}'::jsonb
 AND (((entitlement_ref->'evidence')->'kind')='"OWNER_RECEIPT"'::jsonb)
 AND ((jsonb_typeof(((entitlement_ref->'evidence')->'receipt'))='object'
 AND ((entitlement_ref->'evidence')->'receipt') ?& ARRAY['kind','command','receipt_id']
 AND ((entitlement_ref->'evidence')->'receipt') - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND ((((entitlement_ref->'evidence')->'receipt')->'kind')='"COMPANY"'::jsonb)
 AND (jsonb_typeof((((entitlement_ref->'evidence')->'receipt')->'command'))='object'
 AND (((entitlement_ref->'evidence')->'receipt')->'command') ?& ARRAY['org_id','command_id']
 AND (((entitlement_ref->'evidence')->'receipt')->'command') - ARRAY['org_id','command_id']='{}'::jsonb
 AND (NOT (((((entitlement_ref->'evidence')->'receipt')->'command')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((entitlement_ref->'evidence')->'receipt')->'command')->'org_id'))='string'
 AND (((((entitlement_ref->'evidence')->'receipt')->'command')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((((entitlement_ref->'evidence')->'receipt')->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((entitlement_ref->'evidence')->'receipt')->'command')->'command_id'))='string'
 AND (((((entitlement_ref->'evidence')->'receipt')->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT ((((entitlement_ref->'evidence')->'receipt')->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((entitlement_ref->'evidence')->'receipt')->'receipt_id'))='string'
 AND ((((entitlement_ref->'evidence')->'receipt')->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(((entitlement_ref->'evidence')->'receipt'))='object'
 AND ((entitlement_ref->'evidence')->'receipt') ?& ARRAY['kind','command','receipt_id']
 AND ((entitlement_ref->'evidence')->'receipt') - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND ((((entitlement_ref->'evidence')->'receipt')->'kind')='"GROUP"'::jsonb)
 AND (jsonb_typeof((((entitlement_ref->'evidence')->'receipt')->'command'))='object'
 AND (((entitlement_ref->'evidence')->'receipt')->'command') ?& ARRAY['group_id','command_id']
 AND (((entitlement_ref->'evidence')->'receipt')->'command') - ARRAY['group_id','command_id']='{}'::jsonb
 AND (NOT (((((entitlement_ref->'evidence')->'receipt')->'command')->'group_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((entitlement_ref->'evidence')->'receipt')->'command')->'group_id'))='string'
 AND (((((entitlement_ref->'evidence')->'receipt')->'command')->'group_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((((entitlement_ref->'evidence')->'receipt')->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((((entitlement_ref->'evidence')->'receipt')->'command')->'command_id'))='string'
 AND (((((entitlement_ref->'evidence')->'receipt')->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT ((((entitlement_ref->'evidence')->'receipt')->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((entitlement_ref->'evidence')->'receipt')->'receipt_id'))='string'
 AND ((((entitlement_ref->'evidence')->'receipt')->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))))))
 OR (jsonb_typeof(entitlement_ref)='object'
 AND entitlement_ref ?& ARRAY['kind','command','receipt_id']
 AND entitlement_ref - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND ((entitlement_ref->'kind')='"GROUP_RECEIPT"'::jsonb)
 AND (jsonb_typeof((entitlement_ref->'command'))='object'
 AND (entitlement_ref->'command') ?& ARRAY['group_id','command_id']
 AND (entitlement_ref->'command') - ARRAY['group_id','command_id']='{}'::jsonb
 AND (NOT (((entitlement_ref->'command')->'group_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((entitlement_ref->'command')->'group_id'))='string'
 AND (((entitlement_ref->'command')->'group_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((entitlement_ref->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((entitlement_ref->'command')->'command_id'))='string'
 AND (((entitlement_ref->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT ((entitlement_ref->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((entitlement_ref->'receipt_id'))='string'
 AND ((entitlement_ref->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(entitlement_ref)='object'
 AND entitlement_ref ?& ARRAY['kind','account_id','event_id']
 AND entitlement_ref - ARRAY['kind','account_id','event_id']='{}'::jsonb
 AND ((entitlement_ref->'kind')='"ACCOUNT_SECURITY_EVENT"'::jsonb)
 AND (NOT ((entitlement_ref->'account_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((entitlement_ref->'account_id'))='string'
 AND ((entitlement_ref->'account_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((entitlement_ref->'event_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((entitlement_ref->'event_id'))='string'
 AND ((entitlement_ref->'event_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(entitlement_ref)='object'
 AND entitlement_ref ?& ARRAY['kind','manifest']
 AND entitlement_ref - ARRAY['kind','manifest']='{}'::jsonb
 AND ((entitlement_ref->'kind')='"INDEPENDENT_CUSTODY"'::jsonb)
 AND (jsonb_typeof((entitlement_ref->'manifest'))='object'
 AND (entitlement_ref->'manifest') ?& ARRAY['holder_object_id','immutable_version','content_sha256','encoded_bytes','encryption_key_binding_id']
 AND (entitlement_ref->'manifest') - ARRAY['holder_object_id','immutable_version','content_sha256','encoded_bytes','encryption_key_binding_id']='{}'::jsonb
 AND (NOT (((entitlement_ref->'manifest')->'holder_object_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((entitlement_ref->'manifest')->'holder_object_id'))='string'
 AND (((entitlement_ref->'manifest')->'holder_object_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof(((entitlement_ref->'manifest')->'immutable_version'))='string'
 AND char_length((((entitlement_ref->'manifest')->'immutable_version')#>>'{}'))>=1
 AND char_length((((entitlement_ref->'manifest')->'immutable_version')#>>'{}'))<=1024)
 AND (jsonb_typeof(((entitlement_ref->'manifest')->'content_sha256'))='string'
 AND (((entitlement_ref->'manifest')->'content_sha256')#>>'{}') ~ '^[0-9a-f]{64}$')
 AND (CASE WHEN jsonb_typeof(((entitlement_ref->'manifest')->'encoded_bytes'))='number' THEN (((entitlement_ref->'manifest')->'encoded_bytes')#>>'{}')::numeric=trunc((((entitlement_ref->'manifest')->'encoded_bytes')#>>'{}')::numeric) AND (((entitlement_ref->'manifest')->'encoded_bytes')#>>'{}')::numeric>=1 AND (((entitlement_ref->'manifest')->'encoded_bytes')#>>'{}')::numeric<=4194304 ELSE false END)
 AND (NOT (((entitlement_ref->'manifest')->'encryption_key_binding_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((entitlement_ref->'manifest')->'encryption_key_binding_id'))='string'
 AND (((entitlement_ref->'manifest')->'encryption_key_binding_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')))),false))
);
CREATE INDEX company_actors_account_idx ON public.company_actors(account_id);
ALTER TABLE public.company_actors ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.company_actors FORCE ROW LEVEL SECURITY;
CREATE POLICY company_actors_company_scope ON public.company_actors
    USING (org_id = NULLIF(current_setting('app.current_org',true),'')::uuid)
    WITH CHECK (org_id = NULLIF(current_setting('app.current_org',true),'')::uuid);

CREATE TABLE public.account_context_candidates (
    account_id uuid NOT NULL CHECK (account_id <> '00000000-0000-0000-0000-000000000000'::uuid),
    source_key jsonb NOT NULL,
    context_kind text NOT NULL CHECK (context_kind IN ('COMPANY','GROUP')),
    context_id uuid NOT NULL CHECK (context_id <> '00000000-0000-0000-0000-000000000000'::uuid),
    source_revision bigint NOT NULL CHECK (source_revision >= 1),
    incarnation uuid CHECK (incarnation <> '00000000-0000-0000-0000-000000000000'::uuid),
    state text NOT NULL CHECK (state IN ('CURRENT','HISTORICAL')),
    PRIMARY KEY(account_id,source_key,context_kind,context_id),
    CHECK (pg_column_size(source_key) <= 2048 AND COALESCE(((jsonb_typeof(source_key)='object'
 AND source_key ?& ARRAY['kind','group_id','membership_id','incarnation']
 AND source_key - ARRAY['kind','group_id','membership_id','incarnation']='{}'::jsonb
 AND ((source_key->'kind')='"GROUP_MEMBERSHIP"'::jsonb)
 AND (NOT ((source_key->'group_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((source_key->'group_id'))='string'
 AND ((source_key->'group_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((source_key->'membership_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((source_key->'membership_id'))='string'
 AND ((source_key->'membership_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((source_key->'incarnation')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((source_key->'incarnation'))='string'
 AND ((source_key->'incarnation')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof(source_key)='object'
 AND source_key ?& ARRAY['kind','org_id','assignment_id','revision']
 AND source_key - ARRAY['kind','org_id','assignment_id','revision']='{}'::jsonb
 AND ((source_key->'kind')='"COMPANY_POLICY"'::jsonb)
 AND (NOT ((source_key->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((source_key->'org_id'))='string'
 AND ((source_key->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((source_key->'assignment_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((source_key->'assignment_id'))='string'
 AND ((source_key->'assignment_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof((source_key->'revision'))='string'
 AND ((source_key->'revision')#>>'{}') ~ '^[1-9][0-9]{0,18}$'
 AND CASE WHEN ((source_key->'revision')#>>'{}') ~ '^[1-9][0-9]{0,18}$' THEN ((source_key->'revision')#>>'{}')::numeric <= 9223372036854775807 ELSE false END))
 OR (jsonb_typeof(source_key)='object'
 AND source_key ?& ARRAY['kind','subject','receipt']
 AND source_key - ARRAY['kind','subject','receipt']='{}'::jsonb
 AND ((source_key->'kind')='"HISTORICAL_ENTITLEMENT"'::jsonb)
 AND (jsonb_typeof((source_key->'subject'))='object'
 AND (source_key->'subject') ?& ARRAY['org_id','person_id','employment_id','binding']
 AND (source_key->'subject') - ARRAY['org_id','person_id','employment_id','binding']='{}'::jsonb
 AND (NOT (((source_key->'subject')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((source_key->'subject')->'org_id'))='string'
 AND (((source_key->'subject')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((source_key->'subject')->'person_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((source_key->'subject')->'person_id'))='string'
 AND (((source_key->'subject')->'person_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT (((source_key->'subject')->'employment_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((source_key->'subject')->'employment_id'))='string'
 AND (((source_key->'subject')->'employment_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof(((source_key->'subject')->'binding'))='object'
 AND ((source_key->'subject')->'binding') ?& ARRAY['org_id','employee_id','employment_id','binding_id','binding_revision']
 AND ((source_key->'subject')->'binding') - ARRAY['org_id','employee_id','employment_id','binding_id','binding_revision']='{}'::jsonb
 AND (NOT ((((source_key->'subject')->'binding')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'subject')->'binding')->'org_id'))='string'
 AND ((((source_key->'subject')->'binding')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((source_key->'subject')->'binding')->'employee_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'subject')->'binding')->'employee_id'))='string'
 AND ((((source_key->'subject')->'binding')->'employee_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((source_key->'subject')->'binding')->'employment_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'subject')->'binding')->'employment_id'))='string'
 AND ((((source_key->'subject')->'binding')->'employment_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((source_key->'subject')->'binding')->'binding_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'subject')->'binding')->'binding_id'))='string'
 AND ((((source_key->'subject')->'binding')->'binding_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (jsonb_typeof((((source_key->'subject')->'binding')->'binding_revision'))='string'
 AND ((((source_key->'subject')->'binding')->'binding_revision')#>>'{}') ~ '^[1-9][0-9]{0,18}$'
 AND CASE WHEN ((((source_key->'subject')->'binding')->'binding_revision')#>>'{}') ~ '^[1-9][0-9]{0,18}$' THEN ((((source_key->'subject')->'binding')->'binding_revision')#>>'{}')::numeric <= 9223372036854775807 ELSE false END)))
 AND ((jsonb_typeof((source_key->'receipt'))='object'
 AND (source_key->'receipt') ?& ARRAY['kind','command','receipt_id']
 AND (source_key->'receipt') - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND (((source_key->'receipt')->'kind')='"COMPANY"'::jsonb)
 AND (jsonb_typeof(((source_key->'receipt')->'command'))='object'
 AND ((source_key->'receipt')->'command') ?& ARRAY['org_id','command_id']
 AND ((source_key->'receipt')->'command') - ARRAY['org_id','command_id']='{}'::jsonb
 AND (NOT ((((source_key->'receipt')->'command')->'org_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'receipt')->'command')->'org_id'))='string'
 AND ((((source_key->'receipt')->'command')->'org_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((source_key->'receipt')->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'receipt')->'command')->'command_id'))='string'
 AND ((((source_key->'receipt')->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT (((source_key->'receipt')->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((source_key->'receipt')->'receipt_id'))='string'
 AND (((source_key->'receipt')->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 OR (jsonb_typeof((source_key->'receipt'))='object'
 AND (source_key->'receipt') ?& ARRAY['kind','command','receipt_id']
 AND (source_key->'receipt') - ARRAY['kind','command','receipt_id']='{}'::jsonb
 AND (((source_key->'receipt')->'kind')='"GROUP"'::jsonb)
 AND (jsonb_typeof(((source_key->'receipt')->'command'))='object'
 AND ((source_key->'receipt')->'command') ?& ARRAY['group_id','command_id']
 AND ((source_key->'receipt')->'command') - ARRAY['group_id','command_id']='{}'::jsonb
 AND (NOT ((((source_key->'receipt')->'command')->'group_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'receipt')->'command')->'group_id'))='string'
 AND ((((source_key->'receipt')->'command')->'group_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$')
 AND (NOT ((((source_key->'receipt')->'command')->'command_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof((((source_key->'receipt')->'command')->'command_id'))='string'
 AND ((((source_key->'receipt')->'command')->'command_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))
 AND (NOT (((source_key->'receipt')->'receipt_id')='"00000000-0000-0000-0000-000000000000"'::jsonb)
 AND jsonb_typeof(((source_key->'receipt')->'receipt_id'))='string'
 AND (((source_key->'receipt')->'receipt_id')#>>'{}') ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'))))),false))
);

-- The two catalogs stay inaccessible between numbered DDL and finalization.
-- Remove inherited defaults and owner self-grants exactly as migration0226.
DO $custody$
DECLARE relation_name text; grantee_name text;
BEGIN
    FOREACH relation_name IN ARRAY ARRAY['company_actors','account_context_candidates'] LOOP
        EXECUTE format('REVOKE ALL ON TABLE public.%I FROM PUBLIC',relation_name);
        FOR grantee_name IN
            SELECT DISTINCT role.rolname FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace ns ON ns.oid=relation.relnamespace
            CROSS JOIN LATERAL pg_catalog.aclexplode(relation.relacl) acl
            JOIN pg_catalog.pg_roles role ON role.oid=acl.grantee
            WHERE ns.nspname='public' AND relation.relname=relation_name
        LOOP
            EXECUTE format('REVOKE ALL ON TABLE public.%I FROM %I',relation_name,grantee_name);
        END LOOP;
    END LOOP;
END
$custody$;
COMMENT ON COLUMN public.company_actors.org_id IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.company_actors.account_id IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.company_actors.admission_receipt_id IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.company_actors.entitlement_ref IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.company_actors.created_at IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.account_id IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.source_key IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.context_kind IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.context_id IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.source_revision IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.incarnation IS 'pd:personal — Account-linked attribution or context data';
COMMENT ON COLUMN public.account_context_candidates.state IS 'pd:personal — Account-linked attribution or context data';
