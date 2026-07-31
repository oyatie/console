# Provenance belongs on the link, not in the prose around it

> `Status: RESEARCH — sourced 2026-07-30. Asserts no Korean legal conclusion and changes no control's HOLD.`
>
> Prior art: `Graphify-Labs/graphify` (Apache-2.0/MIT), a codebase→knowledge-graph tool. Studied for its
> data model only; nothing installed, nothing pointed at this tree. Its popularity metrics are unverified
> here and no claim rests on them.

## The one idea worth taking

Graphify's edges carry provenance as a **field**, not as a discipline:

```
{"source": "id_a", "target": "id_b", "relation": "calls|imports|uses|…",
 "confidence": "EXTRACTED|INFERRED|AMBIGUOUS"}
```

- **EXTRACTED** — explicitly stated in the source (an import statement, a direct call).
- **INFERRED** — a reasonable deduction (call-graph second pass, co-occurrence).
- **AMBIGUOUS** — uncertain; **routed to a report for human review** rather than dropped.

Three values, and the third is the load-bearing one.

## Why this repository specifically needs it

We already have the rule. `console-jurisdiction-register.json` states it as prose:

> `uncertainty_rule`: *"Missing, stale, conflicting, or unqualified authority is HOLD; agents may not invent
> certainty."*

Today produced five corrections and **every one was the same defect**: an inferred fact presented as an
extracted one.

| What was claimed | What it actually was |
|---|---|
| The kernel's 고용보험 citation is stale because 고용보험법 시행령 changed 2026-07-01 | A name match. The rate is in 징수법 시행령 제12조; `lsiSeq=280527` resolves to exactly that decree, 공포 2025-12-23, matching the pinned `efYd`. The citation was fresh. |
| Competence is a third relation (ADR-0034 draft) | Contradicted ADR-0028 Decision 6, which had already decided it as a role condition attribute. Written from the first three Decision items only. |
| The buck2 graph is broken — `prelude/` is absent | `[external_cells] prelude = bundled`. Refuted by X8. |
| Bun grouped errors "never by file, explicitly to prevent task fragmentation" | Errors *were* grouped by file inside a crate; the rationale was write-collision avoidance. |
| Four code comments describing a live problem | All four problems were already fixed; three comments were written by the hand that closed the gap. |

Prose rules did not prevent any of these, and `scripts/check-adrs.mjs` structurally cannot see the second
one — it pairs `amends`/`amended_by` and `supersedes`/`superseded_by` only, so a `related`-only record can
contradict its own target silently.
The rule needs somewhere to live that a writer cannot skip.

## What our model has, and the exact hole

Verified by read, not inferred:

- `pub enum LinkCardinality` in `backend/crates/ontology/domain/src/lib.rs` carries `OneOne | OneMany |
  ManyMany`, plus an authored reverse title. **No provenance field.**
- `BackingKind` in the same file separates the two backings — *"One user-authored object instance"* versus
  *"User-authored type with an owned effective-dated instance store"* — but at the **object type** level,
  not per link.
- Property **derivation** already exists and is done correctly. In
  `backend/crates/ontology/adapter-postgres/tests/property_derivation_as_runtime_role.rs`: *"the derived
  value is inside the FIXITY hash: `verify_chain` recomputes"*, and *"a caller-sent value is OVERWRITTEN —
  `params_schema` lists the derived value derived rather than defaulted"*. So the precedent for "derived,
  yet committed and tamper-evident" is already in our code.

The hole is that **provenance exists for property values and not for links** — and the two open
organisation decisions both turn on precisely that distinction:

- **Competence** is *authored* (ADR-0028 Decision 6: a condition attribute on a custom role).
- **Group designation** should be *derived* from control edges rather than stored — and it is currently
  stored **twice**. `0060_create_groups_and_membership.sql` both adds
  `ADD COLUMN group_id UUID NULL REFERENCES groups(id) ON DELETE RESTRICT` to `organizations`, and creates
  `group_memberships` with `PRIMARY KEY (group_id, org_id)` *and* `UNIQUE (org_id)`. Two single-valued
  stores of one fact, free to disagree, with no rule naming which is authoritative.

Those are two different provenance classes on the same graph, and nothing in the link model can express
the difference. That is how a stored value and a derived one end up disagreeing with no mechanism to say
which is authoritative.

## What NOT to borrow

Graphify is weaker than us in the two places that matter most here, so this is a one-idea import:

- **Conflict resolution: absent.** Its docs define no rule for edges that disagree, and multiple edges
  between the same nodes are permitted when relations differ. That is the same hole as the ADR
  contradiction above — evidence the gap is common, not a design to copy. We need our own rule.
- **Incremental update: unspecified.** `watch.py` writes a flag file; rebuild-versus-diff is undocumented.
  Our effective-dated fixity-chained revisions where **state is a fold** are strictly stronger, and the
  same fold is what makes authority replayable. Do not trade that away.

## The proposal, narrowly

Add a provenance enum to the **link instance**, mirroring the property-derivation precedent so derived
links are inside the fixity hash exactly as derived values already are:

- `Authored` — a human asserted this edge through an audited action.
- `Derived` — computed from other edges by a named, deterministic rule; the rule id is stored with it.
- `Unresolved` — the sources conflict or are insufficient. **Readable, and reported.** Not silently
  dropped and not upgraded to `Authored` by a writer in a hurry.

`Unresolved` is `HOLD` expressed as a data value rather than a policy sentence, which is the whole point:
it gives a writer somewhere to put "I do not know" that is cheaper than inventing certainty. Every error
in the table above was a case where the only two available options were *claim it* or *say nothing*.

Deliberately out of scope here: no schema is written, no ADR is amended, and no migration is proposed.
This is prior-art analysis feeding the authored-versus-derived decisions already open in ADR-0028 and the
group-designation question — it does not pre-empt them.

## Open question it does not answer

If a `Derived` link's rule changes, is the old link invalidated, recomputed, or retained with its old rule
id? The register's `change_rule` invalidates dependent evidence on any change, which argues for
invalidation — but authority must stay **replayable**, which argues for retention with the rule id that
produced it. Both cannot be true of the same row, and this is the same tension as the erasure-versus-PITR
conflict named in [`korean-legal-sources.md`](korean-legal-sources.md): a record you can reconstruct is a
record you have not invalidated.
