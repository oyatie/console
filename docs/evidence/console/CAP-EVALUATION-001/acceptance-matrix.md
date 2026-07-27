# Evaluation acceptance matrix

| Story invariant | Domain/application proof | Follow-on integration proof |
| --- | --- | --- |
| Valid lifecycle only | `EvaluationCycle` transition guards | tenant-scoped REST 409 mapping |
| Exact 100% goal weights and freeze at open | domain unit test | persisted goal replacement rejected after opening |
| Expected-version OCC | review/subject/cycle guards; explicit response `version_scope`; calibration is subject-aggregate OCC | concurrent database writers yield one winner |
| Submitted review immutability | private fields plus actor-bound edit/submit commands | REST retry and persisted re-read |
| Role-aware/redacted visibility | `SubjectVisibility` and `can_read_self_content` | RLS-authenticated self/manager/calibrator/unrelated personas |
| Four-eyes calibration | domain unit test | authorized calibration endpoint + audit event |
| Exact idempotency replay/conflict | `decide_idempotency` unit test | unique tenant/action/key receipt transaction |
| All-or-nothing finalization/RV codes | one transaction + one commit port; failure regressions | transaction rollback and monotonic allocator test |
