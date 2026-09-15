//! R2 approval-ready signature contract. Declarations below describe proposed interfaces;
//! this is not a compiled implementation, constructor, fake owner or test bypass.
//! Existing exact crates are listed in owner-entry-map.json.
//
// platform-auth/account_session:
// async fn verify_account_session(pool:&PgPool, verifier:&JwtVerifier,
//    raw_account_access_token:&str) -> Result<VerifiedAccountSession,AuthError>;
// VerifiedAccountSession private fields: actual Account/sub, refresh-family/sid,
// token jti/iat/nbf/exp/security_generation/auth_time/typed assurance. No serde/ctor.
//
// platform-request-context/account:
// async fn resolve_company_context(pool:&PgPool, verifier:&JwtVerifier, token:&str,
//    selection:&UntrustedCompanySelection, serving:&ServingAdmissionBinding)
//    -> Result<AuthenticatedCompanyContext,ContextError>;
// async fn resolve_group_context(pool:&PgPool, verifier:&JwtVerifier, token:&str,
//    selection:&UntrustedGroupSelection, serving:&ServingAdmissionBinding)
//    -> Result<AuthenticatedGroupContext,ContextError>;
// Actual private context includes verified session plus exact selected context generation;
// no public constructor/Deserialize/From<Principal>/test-only issuance.
// Distinct actual service resolver issues AuthenticatedWorkerContext for fixed job namespace.
//
// ontology/application/action30 (pure, no SQLx):
// enum OwnerExecution<R> { Completed(R), Observation(ResultObservation) }
// struct UnadmittedControl<I> { command_id:Uuid, input:I } // plain intent, not authority
// struct UnadmittedWorkerControl<I> { command_id:Uuid, input:I } // worker resolver still mandatory
// fn compile_registered_input_patches<I:SealedRegisteredInput>(schema:&InputSchemaSnapshot,
//    input:&I) -> Result<Vec<Patch>,InputError>; // stable field IDs, every exact typed field
//
// payroll/adapter-postgres/action30:
// async fn payroll_prepare_inputs(pool:&PgPool, auth:&AuthenticatedCompanyContext,
//    attempt:&AttemptRef) -> Result<OwnerExecution<RunResult>,PayrollActionError>;
// async fn read_run_control(pool:&PgPool, auth:&AuthenticatedCompanyContext, run:Uuid)
//    -> Result<RunExpected,PayrollActionError>; // ALL six fields, no defaults
// async fn read_preparation_control(pool:&PgPool, auth:&AuthenticatedCompanyContext, run:Uuid)
//    -> Result<PreparationControl,PayrollActionError>; // RunExpected + membership revision
// async fn advance_native_preparation_once(pool:&PgPool, worker:&AuthenticatedWorkerContext,
//    subject:&ReconciliationSubject) -> Result<(),PayrollActionError>; // real claim/current guards
// async fn observe_native_command(pool:&PgPool, auth:&AuthenticatedCompanyContext,
//    subject:&ReconciliationSubject) -> Result<OwnerExecution<RunResult>,PayrollActionError>;
//
// Every owner adapter-local private function:
// async fn resolve_current_ctx_in_conn(conn:&mut PgConnection,
//    auth:&AuthenticatedCompanyContext, stored:&StoredCommandLocator)->Result<OwnerCommandCtx,E>;
// async fn PRIVATE_FINALIZER(conn:&mut PgConnection,ctx:&OwnerCommandCtx,
//    input:&ExactOwnerInput,protected:&PreparedProtectionPlan)->Result<ExactOwnerResult,E>;
// Revalidate exact live auth/session/context/serving, policy/field/source/task/gate/hold guards
// under approved locks immediately before effect; private context itself is not permission.
//
// platform/group/action30:
// struct GroupRequestIdentity { command:GroupCommandRef, action:GroupActionRef,
//    original_account_id:Uuid }
// struct GroupRegisteredActionRequest<I> { identity:GroupRequestIdentity, input:I }
// async fn group_operation_create(pool:&PgPool,auth:&AuthenticatedGroupContext,
//    request:GroupRegisteredActionRequest<GroupCreate>)->Result<OwnerExecution<GroupResult>,E>;
// No Company command envelope, no sentinel Company, no app.current_org for Group control.

// Supporting proposed fixed owner reads used by the sequential source:
// ontology/adapter-postgres/action30:
// async fn read_registered_draft_binding<I:SealedRegisteredInput>(pool,auth,selection)
//   -> Result<RegisteredDraftBinding,OwnerActionError>;
// RegisteredDraftBinding fields: action:ActionRef,target:Target,schema:InputSchemaRef,
// schema_snapshot:InputSchemaSnapshot,custody:DraftCustody,intent_slot:String,
// gate_request_intent:Option<GovernedRevision>. Current owner resolves these from
// exact registered I and untrusted target selection, not caller-provided authority.
// async fn read_draft_continuation(pool,auth,draft_id)->Result<DraftAck,OwnerActionError>;
// Pure UntrustedActionTargetSelection constructors existing_run/existing_review_basis/
// existing_governance_request carry only untrusted object selection UUIDs.
// Pure ResultObservation::subject()->Option<&ReconciliationSubject> and
// unknown_after_interrupted_wait(subject)->ResultObservation implement the existing
// closed UNKNOWN shape and fixed repair contract. They never manufacture success.
// payroll/adapter-postgres/action30:
// struct PreparationControl { expected:RunExpected, expected_membership_revision:Positive,
//   scope_basis_ref:GovernedRevision,profile_ref:GovernedRevision }
// struct AttendanceCloseControl { expected:RunExpected,close_basis:AttendanceCloseBasisRef }
// async fn read_attendance_close_control(pool,auth,run)->Result<AttendanceCloseControl,E>;
// async fn read_complete_review_bases(pool,auth,cycle:&ReviewCycleRef)->Result<Vec<ReviewBasis>,E>;
// The last read proves exact current cycle/membership completeness under existing
// full-closure/page/chunk/request budgets; overbudget/incomplete is a typed refusal.
// async fn read_review_completion(pool,auth,cycle:&ReviewCycleRef)->Result<CompletionRef,E>;
// CompletionRef is the exact approved action-types.schema.json result ref;
// no construction from count equality, local decisions, or predicted future effect.
// RunExpected returns every required nullable field explicitly. Actual owner CAS
// tests must independently mutate each of all six expectations after real sealing.
// The source intentionally leaves owner-error From conversions and actual DTO/
// module implementations as prerequisites; no compiled Rust test claim is made.
