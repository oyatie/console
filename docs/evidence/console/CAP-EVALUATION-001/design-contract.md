# CAP-EVALUATION-001 — governed performance review foundation

**Status:** implemented domain/application contract; REST, tenant storage, OpenAPI faces, and route composition are deliberately separate follow-on lanes.

## Scope and authority

`STORY-EVALUATION-001` is a governed review cycle: a manager opens a cycle with weighted goals; an employee and their assigned manager submit evidence-backed assessments; an independent calibrator assigns a grade; a finalizer writes an immutable RV ledger outcome. The source of truth for tenant, actor, and role is the authenticated server context — no command accepts an organization identifier.

## State and mutation contract

```text
DRAFT --open--> OPEN --start-calibration--> CALIBRATION --finalize--> FINALIZED --archive--> ARCHIVED
```

There are no reverse transitions. Every cycle mutation has an expected version. Calibration uses the selected **subject aggregate** version; review edit/submit uses the selected **review** version; cycle transitions use the cycle version. Every command response names `version_scope` (`CYCLE`, `SUBJECT`, or `REVIEW`) and returns the corresponding freshly persisted version; retry replays that exact stored response. The following timestamps are set once at their transition: `opened_at`, `calibration_started_at`, `finalized_at`, `archived_at`; finalization additionally records `finalized_by`.

* Opening requires one or more subjects, every subject has one or more goals, and each subject's goal weights total **exactly 100**. Opening freezes subject identity, manager, home branch, org unit, position, team, rubric, and goals.
* Calibration starts only when every subject has both SELF and MANAGER reviews in `SUBMITTED` state.
* Calibration requires a grade and non-empty rationale. Its actor must differ from both the evaluated subject and their manager.
* Finalization validates every subject before reserving any RV code. A repository adapter locks the cycle, subjects, reviews, RV counter, audit rows, and receipt in one transaction. Therefore a validation/retry failure cannot partially issue RV codes.
* A final subject has an immutable grade, RV code matching `RV-[0-9]{4,}`, and finalization timestamp.

## Reviews, evidence, and rubric

A review moves only `DRAFT -> SUBMITTED`; an expected-version mismatch is a conflict and submitted content is immutable. Domain fields are private: adapters must use the actor-bound edit/submit application commands, which require `evaluation_submit` and the authenticated actor to equal the selected review evaluator. Evidence links use only a server-resolved, tenant-visible governed-object id of kind `ATTENDANCE`, `WORK_ORDER`, `APPROVAL`, or `KPI`; there is intentionally no arbitrary `OTHER`/free-text pseudo-link.

Rubric codes are fixed `S, A, B, C, D`. A tenant rubric supplies a label, description, one or more behavioral anchors, and canonical order for every code; it cannot add or remove a code.

## Visibility and authorization projection

The REST/query adapter must derive relationship from the RLS-scoped subject projection:

| Actor | Permitted projection |
| --- | --- |
| unrelated participant | no subject, aggregate, review, or evidence data |
| subject | own subject and own review |
| assigned manager | assigned subject; self-review content remains redacted until the manager review is submitted |
| calibrator | calibration-stage subjects only |

`evaluation_manage`, `evaluation_submit`, and `evaluation_calibrate` are authorization decisions in the outer adapter. The application guard independently requires the matching server-derived role flag before mutation.

## Future REST contract (not implemented by this slice)

The adapter will expose authenticated, tenant-scoped routes:

* `POST /api/v1/evaluation/cycles/{cycleId}/open`
* `POST /api/v1/evaluation/cycles/{cycleId}/start-calibration`
* `POST /api/v1/evaluation/subjects/{subjectId}/calibrate`
* `POST /api/v1/evaluation/cycles/{cycleId}/finalize`

Every mutating request supplies `expected_version` and an idempotency key. The server calculates a canonical request fingerprint. A tenant/action/key with the same fingerprint replays the stored exact response; the same key with a changed fingerprint returns conflict. The receipt contains tenant, action, key, fingerprint, exact response, and timestamp. The application invokes only `EvaluationRepository::transaction` and a single `EvaluationUnitOfWork::commit`: the adapter must place cycle/subject writes, RV reservation, audit write, and receipt write in that one physical transaction or roll all of them back.

## Acceptance stories and required proof

1. A manager cannot open a cycle until each subject's weighted goals equal 100; opening then freezes identities/goals.
2. A subject and manager can edit only their own draft; stale versions and post-submit edits conflict.
3. A manager cannot read self-review content until their own review is submitted; unrelated users receive no aggregate.
4. Calibration rejects missing reviews and either self/manager as calibrator; accepted calibration requires grade/rationale.
5. Finalization either commits every final grade/RV/audit intent and receipt or commits none.
6. A retry with the exact tenant/action/key/fingerprint returns the prior response; a changed payload conflicts.
7. A stale subject calibration version conflicts; failure to reserve RV, persist audit, or persist receipt leaves cycle, subjects, RV counter, audit, and receipt unchanged.

This foundation has unit coverage for FSM/weights/freeze/OCC/review immutability/visibility/four-eyes/idempotency. Future adapter integration tests must exercise RLS cross-tenant concealment, governed-object resolution, transactional RV allocation, and audit persistence against Postgres.
