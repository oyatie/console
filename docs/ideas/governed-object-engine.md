> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Governed Object Engine

> Status: **idea one-pager, pending approval.** Output of `/deep-interview` (round 1) + `/idea-refine`, 2026-07-28.
>
> **Premise corrected after this was written.** Direct inspection on 2026-07-28 showed the two hardest
> components already exist and are tested: the ontology engine (15,372 LOC, `backend/crates/ontology/`)
> and Cedar partial-eval → SQL residual (7,246 LOC, `backend/crates/platform/authz/`), plus the audit
> chain (2,325 LOC) and Cedar pinned `4.11.2`. "Reimagine" therefore targets the **substrate and the
> composition, not the engine** — see `docs/ideas/governed-object-engine-PLAN.md` §0 and
> `docs/PIVOT-2026-07-28.md` §7. Read the assumption checklist below with that in mind: assumption 2
> (Cedar residual viability) is largely answered by existing code rather than open.

## Problem Statement

**How might we** build the smallest governed object engine on which any company's organization,
people, and pay can be expressed as typed objects — where every read is policy-filtered and every
write is a policy-checked action — such that AWS Cedar and Palantir Foundry are the fair comparison?

## Recommended Direction

**An event-log-first, domain-neutral object engine with a narrow default catalog.**

The substrate is an append-only, effective-dated, fixity-stamped event log; current state is *derived*
by folding immutable records, never mutated in place. This is not a stylistic choice — the benchmark
brief's finding is that Foundry, Cedar, Workday, Temporal and SAP-GL all reduce to this shape. Building
it first means as-of reconstruction, lineage, and tamper-evident audit are *inherent properties* rather
than features bolted on later, which is exactly where the previous system accumulated its debt.

On that substrate: a type registry (typed props, link types with cardinality, actions, derived
analytics), a generic instance store, and Cedar as the single authorization spine — with partial
evaluation lowering to a SQL `WHERE` residual so deny-by-omission list filtering is real rather than
client-side pretence.

The engine is **generic by construction**; Korean statutory rules (4대보험, 연차 촉진, 주52h, 최저임금)
become the *first ruleset built on it* and serve as the proof of expressiveness. If the engine can
express Korean payroll without special-casing, it can express any company's.

**Reuse posture: the existing backend is a quarry, not a foundation.** Harvest the patterns that were
expensive to get right — RLS arming as a non-superuser runtime role, the L20 audit canonicalizer,
the statutory rule encodings, migration discipline. Owe nothing to the existing crate shape, naming,
or table layout. `mnt` / `maintenance` / `mnt_rt` are deprecated; greenfield means we never perform the
dangerous live-role rename — we start with new names.

## Key Assumptions to Validate

- [ ] **A folded event log can serve policy-filtered list queries at company scale.** Test: 10k
      employees, 3 years of events; list endpoint with Cedar residual applied; measure p95. This is
      the single riskiest assumption in the design.
- [ ] **Cedar partial evaluation → SQL residual is viable for our predicates.** Test: express the 12
      seed policies (p1–p12) from the prior system; confirm each lowers to SQL or fails *closed* with
      a named untranslatable term. `is_authorized_partial` is experimental — prove it or fall back to
      lowering our own condition grammar.
- [ ] **Korean payroll is expressible in the generic type/action model without escape hatches.** Test:
      one full pay cycle — attendance → 연장근로 approval → pay run → payslip — with zero bespoke tables.
- [ ] **Effective-dating + as-of reconstruction is usable, not just correct.** Test: reconstruct the
      org chart as of an arbitrary past date and diff against the event log.

## MVP Scope

**One vertical slice: hire a person into a position.**

That single flow exercises every load-bearing part at once — type registry, instance store, event log,
effective dating, action dispatch, Cedar authorize + residual read filtering, and audit. If it works,
the rest of org/employee/HR/payroll is more of the same shape.

**In:** event-log substrate · type registry (props, link types, actions) · instance store · Cedar
authorize + partial-eval residual · one action (`hire`) routed through the guardrail preflight ·
as-of query · audit event per transition · REST surface for the above.

**Out of MVP (but in the overall scope):** payroll calculation, HR processes beyond hire, analytics /
derived properties, no-code authoring surfaces, ingest pipeline, evidence/WORM.

## Not Doing (and Why)

- **No frontend.** Deleted deliberately. The 23KB shell returns *last*, as the acceptance test, once
  the engine stands. Building UI now would let the engine be shaped by screen convenience.
- **No migration from the existing system.** Reuse is by reading the old code, not by importing it.
  A migration path would drag the FSM-era model into the new one — the exact thing we're escaping.
- **No AI/LLM judgment anywhere.** Carried forward from §4-28/§4-38: automation is deterministic
  (same input = same output, rule named in the audit) or it is manual. Non-negotiable, and it is also
  the differentiator against Foundry's AIP.
- **No domain beyond org / employee / HR / payroll.** "These are the building blocks of a company,
  that's it." ERP, field ops, comms, compliance modules are explicitly out.
- **No multi-language clients yet.** The Kotlin/Swift/TS generators were a 3-gate tax on every change.
  Reintroduce only when a real consumer exists.

## Open Questions

- **Naming.** `mnt-*` is dead and the repo is now `console`. New crate prefix and DB role name need
  deciding before the first migration — this is cheap now and expensive later.
- **Repo boundary.** New crate tree beside the existing `backend/`, or a new top-level tree with the
  old one deleted once harvested? Affects whether CI can stay green during the build.
- **Generic engine vs. narrow catalog tension.** "Generic for any company" and "only these four
  domains" are compatible only if the *engine* is general and the *catalog* is small. Worth stating
  explicitly as an invariant so the engine doesn't grow domain assumptions.
- **What replaces the deleted CI gates.** Partly resolved since this was written: `package.json` no
  longer declares the removed workspaces or the frontend scripts, and `ci.yml` no longer references
  deleted trees. Still stale as of 2026-07-28: `image-release.yml` (path filters `clients/**`,
  `web/**`, and an `mnt-web` digest step). There is no `release.yml` — release automation is
  `release-please.yml`. Open part of the question stands: which gates replace the three deleted
  OpenAPI client-drift gates and `check-i18n`.
