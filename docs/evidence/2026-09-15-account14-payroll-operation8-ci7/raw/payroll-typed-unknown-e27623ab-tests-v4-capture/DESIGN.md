# Exact next typed completion boundary candidate

Private test author: /root/capture_review. Root is sole integration, product,
Cargo, PostgreSQL, Docker, OpenAPI, generated, CI and authority writer.
Base e27623abacb115b5026e8cae68a5a7079b6f901a; both primary test preimages are
also byte-identical at928e1e4815f5e6336f85667bc27529b5c4fe8b0c.
No implementation admission or runtime success is claimed by this artifact.

## Concrete smallest contract

Existing HTTP ErrorBody only: HTTP503, code `completion_unknown`, message:

> Completion could not be confirmed. Retry after service recovery with the original command_id and unchanged business input. A consumed approval may need renewal for the same action and target.

No endpoint, optional command response field, global ErrorKind, migration or
new dependency. Add this borrowing method to CanonicalPortError:

```rust
fn is_completion_unknown(&self) -> bool { false }
```

Only PayRunError overrides it with `matches!(self, Self::DurabilityUnknown(_))`.
The generic canonical dispatcher checks that method before consuming the error
into KernelError. Classified errors become unit `ActionError::CompletionUnknown`;
RestError maps that variant to the exact existing envelope above. Retain private
diagnostics and validated command/target in structured internal logging if needed.
Unclassified errors retain their existing mapping; false does not prove rollback.
No text prefixes/downcasts. Keep ordinary validation409/422, operation500,
spawn-blocking join failure and post-owner audit failure separate.

Consumer-owned workflow-domain adds only:

```rust
#[derive(Debug)]
pub enum PayrollStageError { Operation(KernelError), CompletionUnknown }
pub type PayrollStageFuture<'a> =
    Pin<Box<dyn Future<Output = Result<bool, PayrollStageError>> + Send + 'a>>;
```

Use std Display/Error, preserve Operation's underlying Display, and narrow
From<KernelError>. Change only PayrollDraftStaging::stage to this future alias;
retain its Send/lifetime/object safety. Preserve the other eight WorkflowRuntimePort
PortFuture methods. Actual PgPayRunPort maps CompletionError directly. The real
drainer distinguishes CompletionUnknown and emits `outcome="unknown"` with
`event_id`, `run_id`, `source_label`, preserving existing PENDING or FAILED and
all attempt/delivery fields. Existing aggregate Result<u64,KernelError> remains.

Native caller deadline/retained pool capacity is unchanged: UNKNOWN does not
imply native COMMIT cancellation or rollback. Same-channel native completion
must drain before retained connection capacity can be released. No reinstated
native timer or local durability fallback.

## Approval, audit and request boundaries

Preserve canonical command_id, actor, target and business input. Receipt replay
uses existing digest/authority checks and repairs audit idempotently. If approval
was consumed but no receipt exists, retry with the same consumed reference can
fail403; request and approve a new reference for the same action/target through
the existing governance flow, then change only four_eyes_request_ref. Preserve
both append-only approval histories. A confirmed owner can still be followed by
an ordinary500 audit failure; this also reconciles by same-command replay.

The outer HTTP TimeoutLayer may return408 or a disconnect may lose the response
while spawn_blocking continues. These are not typed503 responses and do not prove
rollback. This candidate does not add a408 test or timeout policy change. Root must
correct the existing projected200 OpenAPI prose claiming mutation+audit atomicity
when editing the public contract; actual canonical owner+receipt and audit are
separate transactions. Generated/contract/CI changes remain root-owned.

## Executable primary acceptance, existing interfaces only

`primary.patch` appends four exact SQLx app cases and one narrow dispatcher
negative control. Original App3 and all previous helper bytes are preserved.
No primary test references the proposed new trait method/error types.

1. required_api_completion_unknown_reconciles_same_command_after_replay:
   actual native payroll COMMIT SyncRep waiter, bounded503 before resume,
   no snapshot-visible receipt/effect/audit, original waiter still active;
   resume, exact committed receipt/effect, same-command replay repairs one audit,
   stable repeated result/rows and standby equality.
2. required_api_receipt_absent_unknown_renews_approval_with_same_command:
   real governance consumption then real observer EXECUTE revocation causes
   owner admission UNKNOWN;503, absent receipt/effect/audit, spent approval403;
   existing governance renewal changes only its reference, exactly one
   receipt/effect/audit and two immutable approval histories, standby equality.
3. required_api_confirmed_owner_audit_failure_replays_and_repairs:
   real audit INSERT revocation after startup, effective privilege check,
   confirmed owner then ordinary500/internal; restore exact ACL, same-command
   replay repairs one audit without changing receipt/effect, standby equality.
4. required_workflow_typed_unknown_preserves_pending_and_failed_events:
   actual engine emits each event; separate PENDING and FAILED input-history
   cases use real AppState + spawn; observe native explicit confirmation and
   bounded typed outcome log; complete event remains unchanged during UNKNOWN;
   restart spawn after recovery, stable draft/identity/history, exactly one ACK
   attempt increment and audit, standby equality.
5. diagnostic_unknown_prefix_does_not_classify_an_operation_error:
   ordinary CanonicalPortError with UNKNOWN-looking Display passes through the
   actual dispatcher and RestError as500/internal. Negative prefix classifier
   control; it should already pass baseline and is not its own RED lane.

Expected baseline product RED: cases1/2 require503 where current dispatcher
returns500; case4 requires structured unknown classification absent today.
Case3 is a surrounding behavior control expected green. Compilation/fixture
failures are not genuine product RED; root must classify them separately.
Each SQLx test needs its own fresh owned recovery supervisor process. Total
app source roster becomes7. Run existing3 as preservation controls.

## Explicit future-interface compatibility only

`compatibility.patch` MUST NOT be used as executable baseline admission. Apply
only with separately reviewed typed interface implementation. It adds a typed
positive fake-port display-independence control, amends exactly two recovery
KernelError field assertions to PayrollStageError::CompletionUnknown, and changes
LockAfterGate's import/signature to PayrollStageFuture. Preserve every original
capacity/native/identity/row/timing/PoolClosed assertion. These two semantic test
amendments need explicit independent review; frozen prior review cannot substitute.

Inventory's exhaustive e276 census is attached unchanged:9 canonical error impls,
2 staging impls,14 direct stage calls,31 bound sources. Default false keeps the
other8 existing canonical impls source-compatible. Stage impacts are actual owner,
LockAfterGate and two recovery field accesses; other callers infer result types.
Revalidate source bindings if integration changes any affected source.

## Mechanical guide and admission

`run-primary.sh` contains exact root-only commands for the four app cases and
negative control. It has not been executed. Root must append the four supervisor
commands to CI and extend the closed app7 source roster/reachability controls,
without touching the13 already passing supervisor commands. Preserve genuine
compiler discovery and execution counts; source lexical counts are not runtime.

Pre-mortem: wrong phase fault, invisible native waiter, required wire field
missing, diagnostic-only classifier or FAILED overwritten to PENDING.
Blast radius: private test-only postimages and separate future compatibility.
Detection: exact preimages, append-only proof, parser/mechanical patch checks,
actual owner/router/worker witnesses, independent review and root runtime.
Rollback: reject private artifacts; shared integration never mutated here.
Stop: source drift, fake outcome, weak/deleted existing assertions, unbounded
extra work, non-executable probe, unapproved public taxonomy/endpoint/schema.
Reviewer: requested /root/inventory_repair for source seams and
/root/fast_gate_review for immutable independent review (not yet approved).
HOLD: independent acceptance approval, root executable RED/admission, exact-head
compile/discovery/runtime/CI, public contract correction and production authority.
Lenses: Cartesian doubt, Essentialism, Chesterton's Fence, Red Team, Operability,
Blast-radius, Zero-trust. No legal, payment or production-exposure conclusion.

Independent review closure limit: the primary worker case pins UNKNOWN classification only. An ordinary staging Operation/non-unknown-log negative control remains a follow-up closure HOLD; existing operation behavior regression suites still apply.
