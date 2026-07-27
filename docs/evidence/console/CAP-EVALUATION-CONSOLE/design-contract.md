# CAP-EVALUATION-CONSOLE — design contract (buildable spec for backend + frontend lanes)

Provisional migration: `backend/crates/platform/db/migrations/0190_create_evaluation.sql`.
Owning crate: `backend/crates/evaluation/{domain,application,adapter-postgres,rest}`
(new_crate mode, sales-style split). Route base: `/api/v1/evaluation`.
All money-free, org-scoped HR-sensitive data; RLS + deny-by-default features.

## 1. State machines

### 1.1 Cycle (`evaluation_cycles.stage`)
```
DRAFT ──open──▶ OPEN ──start-calibration──▶ CALIBRATION ──finalize──▶ FINALIZED ──archive──▶ ARCHIVED
```
- `open` preflight (fail-closed, §4-29): ≥1 subject; every subject has ≥1 goal
  and an assigned manager. Blockers only.
- `start-calibration` preflight: every subject's MANAGER review SUBMITTED
  (**blocker**); SELF review missing = **advisory** (listed, not blocking —
  self-service linkage is an open substrate item).
- `finalize` preflight: every subject has `calibrated_grade` (**blocker**).
  Finalize issues one RV- code per subject in the same transaction and stamps
  `final_grade = calibrated_grade`, `finalized_at`; one audit event per subject
  plus one for the cycle.
- No hard delete anywhere. `archive` hides from active lists only.

### 1.2 Review (`evaluation_reviews.status`)
```
DRAFT ──submit──▶ SUBMITTED        (terminal; one review per (subject, kind))
```
- Draft upsert (PUT) allowed only while cycle is OPEN; grade optional in draft
  (server-persisted drafts satisfy the module completion contract's
  "drafts survive refresh/retry").
- `submit` requires `grade`; kind MANAGER additionally requires ≥1 evidence
  link (mirrors the scorecard's auto-attached context, made honest).
- SUBMITTED reviews are immutable (no update/delete grants).

### 1.3 Subject (derived; no stored stage)
Derived per subject for UI chips: `ENROLLED → IN_REVIEW → REVIEWED →
CALIBRATED → FINALIZED` from review rows + calibration/final columns.

## 2. Authorization (platform/authz `Feature` additions)

Matrix column order `[MEMBER, MECHANIC, RECEPTIONIST, ADMIN, EXECUTIVE, SUPER_ADMIN]`.

| Feature | wire name | matrix | gates |
|---|---|---|---|
| `EvaluationRead` | `evaluation_read` | `[D,D,D,A,A,A]` | cycle list/detail, subject detail, preflight, person ledger (mirrors `EmployeeDirectoryRead` tier — evaluation is HR-sensitive) |
| `EvaluationManage` | `evaluation_manage` | `[D,D,D,A,D,A]` | cycle create/open/start-calibration/finalize/archive, subject add, goal replace, calibrate (mirrors `EmployeeDirectoryManage` tier) |
| `EvaluationSubmit` | `evaluation_submit` | `[D,D,D,A,A,A]` | my-tasks, review draft/submit; per-subject check in code: caller is the subject's `manager_user_id` OR holds `EvaluationManage` |

Code-enforced beyond features (guardrails §3.10):
- **SoD/four-eyes at calibration**: `calibrated_by` MUST differ from the
  MANAGER review's `evaluator_user_id` → 409 `calibration_requires_four_eyes`.
- **Calibration reason**: required when `final grade ≠ MANAGER review grade`.
- **Deny-by-omission**: subject/cycle reads for callers without `EvaluationRead`
  and without assignment → 404 (no existence leak); RLS confines to org.
- **Audited read**: `GET /employees/{employeeId}/reviews` writes an audit event
  (`evaluation_history_viewed`, target_type `employee`) in the same request.

Frontend capability projection (from these features):
`{ canRead, canManage, canSubmit, canCalibrate: canManage }`.

## 3. REST API surface (all JSON; errors `{ "error": { "message": string } }`)

Path consts exported as `EVALUATION_ROUTE_PATHS` for the openapi drift test.

| # | Method + path | Feature | Req → Res | Notes |
|---|---|---|---|---|
| 1 | `POST /api/v1/evaluation/cycles` | Manage | `CreateEvaluationCycleRequest` → `EvaluationCycleDetail` (201) | creates DRAFT |
| 2 | `GET /api/v1/evaluation/cycles?stage&limit&offset` | Read | → `EvaluationCyclePage` | stage filter optional; default excludes ARCHIVED; limit ≤ 100 |
| 3 | `GET /api/v1/evaluation/cycles/{cycleId}` | Read | → `EvaluationCycleDetail` | includes progress aggregates + subjects |
| 4 | `GET /api/v1/evaluation/cycles/{cycleId}/preflight` | Read | → `EvaluationPreflightReport` | blockers/advisories for the next transition (§4-29 checklist UI) |
| 5 | `POST /api/v1/evaluation/cycles/{cycleId}/open` | Manage | → `EvaluationCycleDetail` | 409 + report when blocked |
| 6 | `POST /api/v1/evaluation/cycles/{cycleId}/start-calibration` | Manage | → `EvaluationCycleDetail` | 409 + report when blocked |
| 7 | `POST /api/v1/evaluation/cycles/{cycleId}/finalize` | Manage | → `EvaluationCycleDetail` | issues RV- codes; per-subject audit |
| 8 | `POST /api/v1/evaluation/cycles/{cycleId}/archive` | Manage | → `EvaluationCycleDetail` | FINALIZED→ARCHIVED only |
| 9 | `POST /api/v1/evaluation/subjects` | Manage | `AddEvaluationSubjectRequest` → `EvaluationSubjectDetail` (201) | cycle must be DRAFT or OPEN; 409 duplicate employee |
| 10 | `GET /api/v1/evaluation/subjects/{subjectId}` | Read OR assigned manager w/ Submit | → `EvaluationSubjectDetail` | goals + reviews + evidence + calibration |
| 11 | `PUT /api/v1/evaluation/subjects/{subjectId}/goals` | Manage OR assigned manager w/ Submit | `ReplaceEvaluationGoalsRequest` → `EvaluationSubjectDetail` | replace-set; cycle DRAFT/OPEN only |
| 12 | `PUT /api/v1/evaluation/subjects/{subjectId}/reviews/{kind}` | Submit (per-subject check) | `UpsertEvaluationReviewRequest` → `EvaluationReview` | kind ∈ `self\|manager`; draft upsert; cycle OPEN |
| 13 | `POST /api/v1/evaluation/subjects/{subjectId}/reviews/{kind}/submit` | Submit (per-subject check) | → `EvaluationReview` | grade required; MANAGER needs ≥1 evidence link; 409 already submitted |
| 14 | `POST /api/v1/evaluation/subjects/{subjectId}/calibrate` | Manage | `CalibrateEvaluationRequest` → `EvaluationSubjectDetail` | cycle CALIBRATION; four-eyes; reason on change |
| 15 | `GET /api/v1/evaluation/my-tasks` | Submit | → `EvaluationTaskPage` | OPEN cycles × caller-assigned subjects × missing/draft reviews; powers "내 평가 할 일" |
| 16 | `GET /api/v1/evaluation/employees/{employeeId}/reviews` | Read | → `EvaluationLedgerPage` | FINALIZED entries only; **audited read**; powers person-card 평가 이력 + 인사 원장 |

Status codes: 401 unauthenticated; 403 feature-denied; 404 not-visible
(cross-org via RLS, or per-subject deny-by-omission); 409 FSM/preflight/SoD
violations; 422 field bounds (validated in handler before DB CHECK).

## 4. DTOs (openapi component names — integrator copies into openapi.yaml, tag `evaluation`)

```yaml
EvaluationGrade:            enum [S, A, B, C, D]
EvaluationCycleKind:        enum [REGULAR, PROBATION]
EvaluationCycleStage:       enum [DRAFT, OPEN, CALIBRATION, FINALIZED, ARCHIVED]
EvaluationReviewKind:       enum [SELF, MANAGER]
EvaluationReviewStatus:     enum [DRAFT, SUBMITTED]
EvaluationMetricKind:       enum [KPI, ATTENDANCE, TASK, CUSTOM]
EvaluationEvidenceKind:     enum [ATTENDANCE, WORK_ORDER, APPROVAL, KPI, OTHER]
EvaluationSubjectState:     enum [ENROLLED, IN_REVIEW, REVIEWED, CALIBRATED, FINALIZED]

CreateEvaluationCycleRequest: { name (1..120), kind, period_label (1..60), due_date (date) }
EvaluationCycleSummary:  { id, name, kind, period_label, due_date, stage,
                           subjects_total, manager_submitted, self_submitted,
                           calibrated, finalized, created_at }
EvaluationCycleDetail:   EvaluationCycleSummary + {
                           opened_at?, calibration_started_at?, finalized_at?, archived_at?,
                           created_by,
                           progress_by_unit: [ { org_unit, total, manager_submitted } ],
                           subjects: [EvaluationSubjectSummary] }
EvaluationCyclePage:     { items: [EvaluationCycleSummary], total }

AddEvaluationSubjectRequest: { cycle_id, employee_id, manager_user_id }
EvaluationSubjectSummary: { id, cycle_id, employee_id, employee_name,
                            org_unit?, manager_user_id, state: EvaluationSubjectState,
                            final_grade?, rv_code? }
EvaluationSubjectDetail:  EvaluationSubjectSummary + {
                            goals: [EvaluationGoal],
                            reviews: [EvaluationReview],
                            calibrated_grade?, calibration_reason?, calibrated_by?, calibrated_at?,
                            finalized_at? }

EvaluationGoal:          { id, title (1..200), metric_kind, target_label (1..200),
                           weight_pct (0..100), sort_order }
ReplaceEvaluationGoalsRequest: { goals: [ { title, metric_kind, target_label, weight_pct } ] }
                           # ≤ 20 goals; weights need not sum to 100 (advisory in UI)

UpsertEvaluationReviewRequest: { grade?, note? (≤2000),
                                 evidence_links: [ { object_kind, object_ref (1..120),
                                                    label (1..200) } ] }  # ≤ 10 links
EvaluationReview:        { id, subject_id, kind, status, evaluator_user_id,
                           grade?, note?, evidence_links: [EvaluationEvidenceLink],
                           submitted_at?, updated_at }
EvaluationEvidenceLink:  { id, object_kind, object_ref, label, sort_order }

CalibrateEvaluationRequest: { final_grade, reason? }   # reason required on change
EvaluationTaskPage:      { items: [ { subject_id, cycle_id, cycle_name, due_date,
                                      employee_id, employee_name, kind,
                                      review_status? } ] }
EvaluationLedgerPage:    { items: [ { rv_code, cycle_id, cycle_name, period_label,
                                      final_grade, finalized_at, subject_id } ] }
EvaluationPreflightReport: { next_transition: enum [open, start_calibration, finalize, archive] | null,
                             blockers: [ { code, message, subject_id? } ],
                             advisories: [ { code, message, subject_id? } ] }
```

Traversable links (module completion contract, ≥2 each way):
- Upstream from a subject/review: employee (people module `/api/v1/employees/{id}`),
  evidence objects (AT-/WO-/AP-/KPI refs → their modules).
- Downstream: person ledger entry (16) → person card; audit events
  (target_type/target_id) → audit module; cycle → subjects → reviews.

## 5. Audit actions (`with_audit`, target_type / action)

| Action | target_type | target_id | snapshot |
|---|---|---|---|
| `evaluation_cycle_created` | `evaluation_cycle` | cycle id | after |
| `evaluation_cycle_opened` / `evaluation_calibration_started` / `evaluation_cycle_finalized` / `evaluation_cycle_archived` | `evaluation_cycle` | cycle id | before/after stage |
| `evaluation_subject_added` | `evaluation_subject` | subject id | after |
| `evaluation_goals_replaced` | `evaluation_subject` | subject id | before/after goals |
| `evaluation_review_saved` | `evaluation_review` | review id | after (draft) |
| `evaluation_review_submitted` | `evaluation_review` | review id | after (grade + evidence refs) |
| `evaluation_subject_calibrated` | `evaluation_subject` | subject id | before/after grade + reason |
| `evaluation_subject_finalized` | `evaluation_subject` | subject id | after (rv_code, final_grade) — one per subject inside finalize |
| `evaluation_history_viewed` | `employee` | employee id | none (read audit) |

All org-scoped (`org_id` set → RLS-armed transaction), actor = caller.

## 6. DDL — provisional `0190_create_evaluation.sql`

```sql
-- Performance review cycles (CAP-EVALUATION-CONSOLE). Design authority:
-- docs/design/oyatie-console (screen "review", OT-24, HANDOFF §15/§16).
-- RLS/grant conventions copied from 0172.

CREATE TABLE evaluation_cycles (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id        UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    name          TEXT NOT NULL CHECK (btrim(name) <> '' AND char_length(name) <= 120),
    kind          TEXT NOT NULL CHECK (kind IN ('REGULAR', 'PROBATION')),
    period_label  TEXT NOT NULL CHECK (btrim(period_label) <> '' AND char_length(period_label) <= 60),
    due_date      DATE NOT NULL,
    stage         TEXT NOT NULL DEFAULT 'DRAFT'
                  CHECK (stage IN ('DRAFT', 'OPEN', 'CALIBRATION', 'FINALIZED', 'ARCHIVED')),
    created_by    UUID NOT NULL,
    opened_at     TIMESTAMPTZ,
    calibration_started_at TIMESTAMPTZ,
    finalized_at  TIMESTAMPTZ,
    archived_at   TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (id, org_id),
    FOREIGN KEY (created_by, org_id) REFERENCES users(id, org_id) ON DELETE RESTRICT
);
CREATE INDEX evaluation_cycles_org_stage_idx ON evaluation_cycles (org_id, stage, due_date);

CREATE TABLE evaluation_subjects (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id             UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    cycle_id           UUID NOT NULL,
    employee_id        UUID NOT NULL,
    manager_user_id    UUID NOT NULL,
    calibrated_grade   TEXT CHECK (calibrated_grade IN ('S', 'A', 'B', 'C', 'D')),
    calibration_reason TEXT CHECK (calibration_reason IS NULL
                                   OR (btrim(calibration_reason) <> '' AND char_length(calibration_reason) <= 500)),
    calibrated_by      UUID,
    calibrated_at      TIMESTAMPTZ,
    final_grade        TEXT CHECK (final_grade IN ('S', 'A', 'B', 'C', 'D')),
    rv_code            TEXT CHECK (rv_code ~ '^RV-[0-9]{4,}$'),
    finalized_at       TIMESTAMPTZ,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (id, org_id),
    UNIQUE (cycle_id, employee_id),
    UNIQUE (org_id, rv_code),
    FOREIGN KEY (cycle_id, org_id)        REFERENCES evaluation_cycles(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (employee_id, org_id)     REFERENCES employees(id, org_id)         ON DELETE RESTRICT,
    FOREIGN KEY (manager_user_id, org_id) REFERENCES users(id, org_id)             ON DELETE RESTRICT,
    FOREIGN KEY (calibrated_by, org_id)   REFERENCES users(id, org_id)             ON DELETE RESTRICT,
    CHECK ((calibrated_grade IS NULL) = (calibrated_by IS NULL)),
    CHECK ((calibrated_grade IS NULL) = (calibrated_at IS NULL)),
    CHECK ((final_grade IS NULL) = (rv_code IS NULL)),
    CHECK ((final_grade IS NULL) = (finalized_at IS NULL))
);
CREATE INDEX evaluation_subjects_org_cycle_idx    ON evaluation_subjects (org_id, cycle_id);
CREATE INDEX evaluation_subjects_org_employee_idx ON evaluation_subjects (org_id, employee_id)
    WHERE finalized_at IS NOT NULL;
CREATE INDEX evaluation_subjects_org_manager_idx  ON evaluation_subjects (org_id, manager_user_id);

CREATE TABLE evaluation_goals (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id       UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    subject_id   UUID NOT NULL,
    title        TEXT NOT NULL CHECK (btrim(title) <> '' AND char_length(title) <= 200),
    metric_kind  TEXT NOT NULL CHECK (metric_kind IN ('KPI', 'ATTENDANCE', 'TASK', 'CUSTOM')),
    target_label TEXT NOT NULL CHECK (btrim(target_label) <> '' AND char_length(target_label) <= 200),
    weight_pct   SMALLINT NOT NULL CHECK (weight_pct BETWEEN 0 AND 100),
    sort_order   INTEGER NOT NULL CHECK (sort_order >= 0),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (subject_id, sort_order),
    FOREIGN KEY (subject_id, org_id) REFERENCES evaluation_subjects(id, org_id) ON DELETE RESTRICT
);

CREATE TABLE evaluation_reviews (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id            UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    subject_id        UUID NOT NULL,
    kind              TEXT NOT NULL CHECK (kind IN ('SELF', 'MANAGER')),
    status            TEXT NOT NULL DEFAULT 'DRAFT' CHECK (status IN ('DRAFT', 'SUBMITTED')),
    evaluator_user_id UUID NOT NULL,
    grade             TEXT CHECK (grade IN ('S', 'A', 'B', 'C', 'D')),
    note              TEXT CHECK (note IS NULL OR char_length(note) <= 2000),
    submitted_at      TIMESTAMPTZ,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (id, org_id),
    UNIQUE (subject_id, kind),
    FOREIGN KEY (subject_id, org_id)        REFERENCES evaluation_subjects(id, org_id) ON DELETE RESTRICT,
    FOREIGN KEY (evaluator_user_id, org_id) REFERENCES users(id, org_id)               ON DELETE RESTRICT,
    CHECK (status = 'DRAFT' OR (grade IS NOT NULL AND submitted_at IS NOT NULL))
);

CREATE TABLE evaluation_evidence_links (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id      UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    review_id   UUID NOT NULL,
    object_kind TEXT NOT NULL CHECK (object_kind IN ('ATTENDANCE', 'WORK_ORDER', 'APPROVAL', 'KPI', 'OTHER')),
    object_ref  TEXT NOT NULL CHECK (btrim(object_ref) <> '' AND char_length(object_ref) <= 120),
    label       TEXT NOT NULL CHECK (btrim(label) <> '' AND char_length(label) <= 200),
    sort_order  INTEGER NOT NULL DEFAULT 0 CHECK (sort_order >= 0),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (review_id, sort_order),
    FOREIGN KEY (review_id, org_id) REFERENCES evaluation_reviews(id, org_id) ON DELETE RESTRICT
);

-- RV- code issuance: per-org monotone counter, locked FOR UPDATE at finalize.
CREATE TABLE evaluation_code_counters (
    org_id     UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE RESTRICT,
    next_value INTEGER NOT NULL DEFAULT 2500 CHECK (next_value > 0)
);

DO $$
DECLARE t TEXT;
BEGIN
    FOREACH t IN ARRAY ARRAY['evaluation_cycles', 'evaluation_subjects', 'evaluation_goals',
                             'evaluation_reviews', 'evaluation_evidence_links', 'evaluation_code_counters']
    LOOP
        EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', t);
        EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', t);
        EXECUTE format(
            'CREATE POLICY org_isolation ON %I
               USING (org_id = NULLIF(current_setting(''app.current_org'', true), '''')::uuid)
               WITH CHECK (org_id = NULLIF(current_setting(''app.current_org'', true), '''')::uuid)', t);
    END LOOP;
END $$;

-- No DELETE on lifecycle objects (archive-not-delete); goals/evidence are
-- replace-set editable while their parent is still draft-stage.
GRANT SELECT, INSERT, UPDATE         ON evaluation_cycles         TO mnt_rt;
GRANT SELECT, INSERT, UPDATE         ON evaluation_subjects       TO mnt_rt;
GRANT SELECT, INSERT, UPDATE, DELETE ON evaluation_goals          TO mnt_rt;
GRANT SELECT, INSERT, UPDATE         ON evaluation_reviews        TO mnt_rt;
GRANT SELECT, INSERT, UPDATE, DELETE ON evaluation_evidence_links TO mnt_rt;
GRANT SELECT, INSERT, UPDATE         ON evaluation_code_counters  TO mnt_rt;
```

Note: `evaluation_code_counters` deliberately lacks an org RLS bypass — the
finalize transaction runs under `app.current_org`, so the counter row is only
visible/lockable inside the owning tenant. `mnt_rt` RLS tests must cover: (a)
cross-org invisibility of cycles/subjects/ledger, (b) unarmed-GUC fail-closed
reads, (c) the counter upsert under RLS.

## 7. Build-lane obligations (stage 2/3 checklists)

Backend lane (dark landing — no `build_router`, no openapi touch):
1. Migration 0190 exactly as §6 (renumber only if the integrator reassigns slots).
2. Crates `mnt-evaluation-{domain,application,adapter-postgres,rest}` mirroring
   sales; rest exports `EVALUATION_ROUTE_PATHS` + `pub fn router(EvaluationRestState) -> Router`.
3. `platform/authz`: three Feature variants (§2) with doc comments, `as_str`,
   parse, matrix rows + matrix tests updated.
4. `backend/app/tests/evaluation_cycle_api.rs`: mounts the evaluation router
   directly (scratch DB via `MNT_POSTGRES_DB`), walks STORY-EVALUATION-001
   end-to-end (create→subjects→goals→open→self+manager reviews with evidence→
   start-calibration→calibrate (four-eyes assert)→finalize→ledger read + audit
   rows assert), plus deny cases (403 feature, 404 omission, 409 FSM/SoD).
5. `cargo fmt` + `clippy -D warnings` + package-scoped tests as mnt_rt.

Frontend lane (owns `web/src/console/evaluation/**` only):
1. Production-exemplar file set; screen key `evaluation`; strings in
   `web/src/i18n/evaluation.ts` (nav label already in ko.ts).
2. Layers: cycle list + progress stat bar (chips, no KPI cards) / cycle detail
   with subjects + preflight checklist / subject detail with goals, scorecard
   (grade segment S–D, evidence links as drillable code chips, submit
   fail-closed), calibration action / person history via endpoint 16.
3. Capabilities from §2; `canRead=false` renders denied without fetching;
   draft reviews persist server-side (endpoint 12) so refresh/Back retains them.
4. Until clients are regenerated, type DTOs locally in the module mirroring §4
   names, swap to `components["schemas"]` at integration (note in code, not a
   stub — calls target the real paths).

Integrator (via `integration-manifest.json` beside this file): registry/nav
mount, openapi paths+schemas+tag, client regeneration, `build_router` mount.
