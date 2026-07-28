# FANOUT PLAN — catalog expansion on the governed object engine

**Status: APPROVED 2026-07-28 — execution authorized.**
Date: 2026-07-28 · Mode: deliberate (`--deliberate`) · Rev 3 (Architect: premise corrected · Critic: ITERATE fixes applied)
Supersedes the fan-out sections of `docs/ideas/governed-object-engine-PLAN.md` §3–§4 and
`docs/program/LANE-PROTOCOL.md` §4

---

## 0. Premise, corrected

**Rev 1 of this plan was built on a false premise and is retracted.** It claimed the built-in catalog
manifest digest was a whole-catalog global lock making type-per-lane sharding impossible
("collision probability 100%, by construction"). That is wrong.

The error: migration `0204_ontology_catalog_additive_upgrade.sql` opens with a header comment stating
the **problem it was written to solve** — *"Adding a 28th built-in object type was therefore
impossible for any live tenant."* Rev 1 quoted that header as current state and never read the 200
lines beneath it that fix it. Reading the comment instead of the code.

**Verified against the migration, not its prose:**

| Fact | Evidence |
|---|---|
| The digest is computed over `p_manifest`, a **function parameter** — not "the catalog" | `0204:87` |
| The allowlist is looked up **by `catalog_version`** | `0204:88-90` |
| `catalog_version` is the allowlist's **PRIMARY KEY** — versions are PK-disjoint | `0165:121-125` |
| Shape validation checks only *is-object / version-matches / `object_types` is-array* — **no completeness or superset check** | `0204:81-85` |
| Install markers are **append-only**, PK `(org_id, catalog_version)`; *"an upgrade appends, it does not rewrite"* | `0204:42-45` |
| Pass 1 leaves keys the tenant already holds **untouched** | `0204:130-181` |
| Pass 2 resolves a new type's links against **published heads from earlier catalog versions**, by `stable_key` **string** | `0204:183-186, 201-211` |
| Arbitrary version/manifest installs are **already proven** in the test suite | `builtin_catalog_additive_upgrade_as_runtime_role.rs:176-211` |

**The lock is not an engine property. It is an authoring convention living in exactly two places:**
`BUILTIN_CATALOG_VERSION` (`seed.rs:68`) and the 27-element `drafts` vec (`seed.rs:1181-1209`).

**Therefore type-per-lane sharding is viable today, with no refactor.** A lane ships its own
`<TYPE>_CATALOG_VERSION`, its own manifest in its own file, and one migration inserting one allowlist
row. Two lanes doing this collide on nothing: different files, PK-disjoint version strings, different
migration numbers, different digests over different manifests. Links reference other types by
`stable_key` **string**, resolved in the database at install time — never a Rust symbol from another
lane, so lanes do not compile against each other.

**Do NOT "fix" this by refactoring the digest chain.** `0165:115-118` states the security property:
*"runtime can only present a manifest whose canonical JSONB digest was pinned by a migration."* One
allowlist row per type is cheap and PK-disjoint; the evidentiary guarantee is not. Rejected.

---

## 1. Principles

1. **Manufacture the immutable target before parallelizing** — and write it against surfaces that
   already exist, so it is never written twice.
2. **Parallelize only what is provably disjoint.** Disjointness is *structural*, demonstrated by a dry
   run — never procedural, never assumed.
3. **Never repair a gate by weakening it, and never move the target.**
4. **Reproduce the original failure, not the artifact you touched.**
5. **Scope is load-bearing.** Org, employee, HR, payroll. That's it.

## 2. Decision drivers

1. **Catalog installs are already additive and version-keyed** (§0) — so the shard boundary can be the
   type itself, and coordination collapses to pre-reservation (§4 P0) plus three CI gates.
2. **The ontology REST surface is already type-agnostic** — `/instances`, `/instances/{id}/history`,
   `/traverse`, `/lifecycle`, `/actions/{key}/preflight|execute` are generic and already specified in
   `openapi.yaml`. An `Instance`+`InstanceRevision` type needs **zero** new routes.
3. **No immutable verification target exists yet.**
4. **Landing is serialized regardless of work parallelism** — squash-only merges orphan the candidate
   SHA and each merge invalidates ~390 authority-train bindings. O(N²).

## 3. Options considered

### Option A — monolithic manifest + serial Catalog Owner *(Rev 1; rejected)*
- **Con:** premise factually wrong (§0). Also blocks every lane's *integration* tests on the owner's
  batch, forces edits to three shared files, and puts a human in a mechanical loop.

### Option B — per-lane catalog versions *(recommended)*
Lane owns `adapter-postgres/src/catalog/<type>.rs` containing its own `<TYPE>_CATALOG_VERSION` and
`<type>_manifest()`, plus one migration inserting one allowlist row at its computed digest.
- **Pro:** uses `0204` as designed; **zero shared-file edits by any lane** once §4 P0 pre-reserves the
  slots; lanes install and test **immediately** in their own worktree; enforcement is three CI gates,
  one of which already exists in draft form; deletes the Catalog Owner role and pre-mortem S2 entirely.
- **Con 1:** N catalog versions must install in a deterministic order at bootstrap. *Mitigation:* the
  pre-reserved ordered array (§4 P0). `0204:201-210` raises `link_target_not_found` when a declared
  link's target is not yet a published head — but **only for declared links**: the guard is
  `NULLIF(btrim(...), '')`, so a lane that *omits* a link declaration gets silence, not an error.
  Database enforcement is therefore partial, and CI gate 2's topological-sort check is what closes it.
- **Con 2 (priced honestly):** catalog versions are immutable once installed, so **each lane gets
  essentially one shot at its type's schema** — see S4.

### Option C — refactor to per-type manifests + incremental digest *(rejected)*
Would be principled if the lock were real. Option B achieves the same with no engine change, and
touching the digest chain risks the guarantee `0165:115-118` exists to provide.

## 4. Phases — fan-out is phase 3

### Phase 0 · Pre-reservation *(one commit, before anything else)*

Converts the two per-lane shared-file appends from **procedural** to **structural**, which is what
Principle 2 demands and what makes §10.3 satisfiable at all.

In a single commit, land:
- all five `mod <type>;` lines in `adapter-postgres/src/catalog/mod.rs`, referencing files that do
  not exist yet — **each lane then only creates its own file and edits nothing shared**;
- the ordered bootstrap install array with all five version slots (§3's con, made real);
- each lane's reserved migration-number block.

Same trick already used for migration numbers. Without it two lanes append to one file tail and
conflict in the same hunk — git does not know the lines are independent.

**Note:** the ordered multi-version bootstrap does not exist yet. `seed.rs:1259-1273` hardcodes a
single version and manifest, and its only callers (`:1097, :1118, :1321`) pass key subsets of it.
Building it is a **named phase-2 deliverable**, not an assumption — §10.7 depends on it.

### Phase 1 · `company-conformance` — the immutable target *(serial, blocking)*

**Driver decision.** `PLAN` §0.5 (use-case layer) and §3 P2 (public REST) contradict. Resolution:
**scenario logic written once, driven through two adapters that both exist today.**

REST is not a smoke afterthought. **RLS arming, Cedar enforcement and org-scoping live at the adapter
boundary**, so a use-case-only suite passes against a route that forgot to arm `app.current_org` — a
class this repo has already been burned by, where a superuser test masked a totally broken read path.
Acceptance criterion 6 is only meaningfully proven at a real door.

**Correction from Architect review:** Rev 1 made a *per-type use-case* driver primary. Those
signatures do not exist until phase 2, so the suite would have been written twice — moving the target,
violating Principle 3. Both drivers therefore bind to surfaces that exist **now**:

```
company-conformance/
  scenarios.rs         ← logic + assertions, written ONCE, immutable
  drivers/rest.rs      ← the generic ontology REST surface (exists today)
  drivers/store.rs     ← PgOntologyStore action dispatch (exists today)
  fixtures/<type>.rs   ← per-type scenario DATA — a lane may add its own
```

Scenario: found a company → create org units → define positions → hire people → transfer one → run a
pay cycle → reconstruct the org as-of a past date → prove a non-privileged principal cannot see rows
outside policy.

**Ownership — the Rev 1 contradiction, resolved.** Rev 1 said both "a lane may not edit it" and "each
lane owns their conformance scenario slice", and its own S1 tripwire would have fired on every lane
PR. Now: **drivers and assertions are immutable and owned outside the lanes; scenario *data* is
per-type fixture files a lane may add.** Adding a fixture is not editing the target.

**It must fail meaningfully before work starts** — demonstrated red. A suite that passes against an
empty implementation is the vacuous-gate class in a new costume. Against today's tree the REST driver
returns a real unknown-type error for `org_unit`, which is meaningful red and needs no rewrite later.

### Phase 2 · OrgUnit end to end *(serial, 1 implementer + 2 adversarial reviewers)*

The `.zig`-reference analogue. Built by hand, exceptionally well; every later type transliterates from
it and reviewers check fidelity **against it**. Exercises registry + instance store + event log +
effective dating + action dispatch + Cedar authorize/residual + audit + as-of in one slice.

**Exit condition:** its slice of phase 1 is green through **both** drivers.

### Phase 3 · Fan-out *(authorized only after phase 2 exits and §10.3 passes)*

| Lane | Owns |
|---|---|
| Lane 1 | Position — `catalog/position.rs`, its allowlist migration, Cedar policy, tests, fixture |
| Lane 2 | Person — same shape |
| Lane 3 | Employment — same shape (links Person × Position **by stable_key string**) |
| Lane 4 | PayRun — same shape |
| Lane 5 | float — fixer / driver work |

**No Catalog Owner role.** Deleted with its premise. Replaced by two CI gates (§5).

**Install order** (database-enforced, not agent-enforced): OrgUnit → Position → Person → Employment →
PayRun. A wrong order raises `link_target_not_found` at install.

## 5. Reservation scheme (concrete)

| Shared resource | Discipline | Verified |
|---|---|---|
| `seed.rs` `BUILTIN_CATALOG_VERSION` + 27-draft vec | **frozen.** Lanes do not touch it; new types are new files | `seed.rs:68, 1181-1209` |
| `adapter-postgres/src/catalog/mod.rs` | **PRE-RESERVED**, not appended. All five `mod` lines land in one commit before fan-out; a lane then creates only its own new file and edits nothing shared. *(Rev 2 called this "textually append-only" — that was a claim about agent behaviour, not a merge property. Two lanes appending to one file tail conflict in the same hunk; git does not know the lines are independent.)* | same trick as migration numbers |
| ordered bootstrap install array | **PRE-RESERVED** in the same commit — see §4 · P0 | `seed.rs:1259-1273` |
| catalog allowlist migration | one row per type, keyed by the lane's own `catalog_version` | PK-disjoint, `0165:122` |
| migrations sequence | pre-reserved block per lane; take the number immediately before push | 204 files, highest `0204` |
| **`backend/openapi/openapi.yaml`** | **NOT TOUCHED by catalog lanes** — the ontology REST surface is generic. Any lane proposing a bespoke route must escalate | `openapi.yaml:11981-13010`; `ontology/rest/src/lib.rs:194-205, 226-243` |
| **`backend/app/tests/openapi_drift.rs`** | 37 per-crate `include_str!` entries — **unreserved in Rev 1.** No catalog lane should need it; if one does, that is the escalation signal | `:44-265` |
| **`key_write_cas_as_runtime_role.rs:959`** | `assert_eq!(object_types.len(), 27)` — **unreserved in Rev 1.** Stays 27 under Option B because lanes do not extend the built-in vec | `:959` |
| `docs/specs/cedar-pbac-coexistence-map.json` | **NOT touched by catalog lanes — Rev 2 invented this bottleneck.** The map is keyed by **domain**, not object type (`identity.policy`, `workflow.guards`); no entry is keyed by an ontology type. Per-type policy is authored **at runtime from the registry** — `ontology/rest/src/lib.rs:426-437` reads the type's own `properties` into `DeclaredAttr` and splices them into the Cedar schema. A lane adds policy without touching this file. If one needs to, that is the escalation signal | `map.rs:42`; `rest/src/lib.rs:426-437, 1654` |
| `backend/Cargo.toml` `members` | 38 globs already cover new crates under existing groups → **no edit needed**. Never create a crate dir without a valid `Cargo.toml` in the same change | `Cargo.toml:8-49` |
| `third-party/rust/BUCK` | reindeer-generated; serialize dependency additions, one owner | — |
| per-crate `BUCK` files | **generated** by `gen_first_party.py`; never hand-edit — the drift gate rejects it | 169 under `backend/` |
| `company-conformance` drivers + assertions | outside the lanes; escalation required. Fixtures are lane-addable | — |

### The three CI gates that replace the Catalog Owner

1. **Every `*_CATALOG_VERSION` has an allowlist row at its computed digest.** A generalizable form
   already exists at `builtin_catalog_additive_upgrade_as_runtime_role.rs:292-312` — it computes the
   digest and prints the exact `decode('<hex>','hex')` the migration needs.
2. **No two lanes claim the same `catalog_version` or `stable_key`**, and the install order is a
   **topological sort** of declared `to_stable_key` links.
3. **No manifest's `stable_key` set intersects an already-allowlisted version's** — the edit-drop
   guard (S4). Without this, a correction ships green and does nothing.

**Cost, corrected:** Rev 2 said "~30 lines". The real pattern is a gate **crate** —
`backend/ci/gates/rls-arming/` is `Cargo.toml` + `BUCK` + `src/lib.rs` (340) + `src/main.rs` (40)
≈ 380 lines plus a CI job. Estimate was ~10× low. Bounded, though: `backend/Cargo.toml:48` already
globs `ci/gates/*` (no member edit) and BUCK files are generated.

**Owner: phase 2.** These are the sole coordination mechanism replacing a human owner and sit on the
critical path — they cannot be unowned.

## 6. Structural guards against our own measured failures

| Failure (measured this session) | Structural prevention |
|---|---|
| **Shared-checkout contamination** — 4 repair lanes edited one working tree; every lane reported foreign "scope violations"; one file mutated between two of its own reads | Lanes run in **real git worktrees** (`~/Developer/console-lanes/lane-{1..5}`). Disjoint file lists were proven insufficient. |
| **Briefing error constrained the design** — an exclusive-file list omitted the file the correct fix needed; the implementer knowingly shipped the second-best design and said so | Lanes own a **coherent vertical slice**, not a file list, plus an explicit **escalation path**: *"if the correct fix lies outside your slice, stop and report — do not implement the second-best fix."* |
| **Verified the artifact, not the failure** — a repair fixed one file and reported green; the job ran three commands and a second file carried the same defect | Every lane must **reproduce the original failing symptom end to end**, red→green. Testing the file you touched is not evidence. |
| **Probes that can only say GREEN** — two probes were broken and would have reported false success | **Every probe proven RED on a known-bad input** before its GREEN is trusted. |
| **Agent mortality** — 2/7 then 1/6 agents died on session limits; once the only verifier died, a lane shipped unverified | **Never schedule the only verifier last.** Verification runs per-slice as it completes. Lanes **write findings to disk as they go** — three agents went idle without ever returning a report. |
| **Gate-fixing anti-pattern** — 3/6 CRITICALs self-inflicted by making gates pass | Conformance drivers/assertions owned outside lanes. Reviewers explicitly check for **repair-by-weakening**. |
| **Reading the comment instead of the code** — Rev 1 of this very plan | Every load-bearing claim must cite `file:line` of **executable code**, never a header comment. Architect review verifies claims by execution. |
| **Anti-parallel landing** — squash-only merges orphan the candidate SHA; each merge invalidates ~390 bindings | **Parallelise the work, serialise the landing.** Lanes collect onto one integration branch → a single C with one T → one trip through the train per batch. |

## 6.5 Pipeline audit — pattern vs anti-pattern

Bun's rule is *"edit the process, not the outputs."* Applied to our own process, using only what this
session measured. Goal: productive **and** excellent — friction that buys neither is waste.

### PATTERN — keep, and spend more here

| Practice | Measured return |
|---|---|
| **1 implementer + 2 adversarial diff-only reviewers** | The highest-ROI mechanism by a wide margin. Caught 38 fabricated doc claims, 21 live-infra misclassifications, this plan's **false premise**, and a repair that fixed 1 of 3 commands while reporting green. Every one was reported clean by its producer. |
| **Verify by execution; cite `file:line` of code, never a comment** | Rev 1's premise died to a header comment. Two of my own probes were broken (`.length` on an object; `root//pkg:name` vs `//pkg:name`) and could only have returned GREEN. |
| **Probe must be proven RED on a known-bad input** | Directly follows from the above. A probe with no demonstrated failure mode is not evidence. |
| **Reproduce the original failure, not the artifact touched** | The authority-gate repair passed its own file's tests and left the gate red. |
| **Real worktrees per lane** | Shared checkout produced cross-lane contamination in 4/4 lanes; one saw a file mutate between its own two reads. |
| **Consensus planning (Planner→Architect→Critic)** | Found premise-level defects, not nits. Justified for a plan that gates months of work — **not** for routine changes; that would be ceremony. |

### ANTI-PATTERN — change the pipeline

| Practice | Why it is friction without return |
|---|---|
| **The ~390-binding authority-train rebind** | The registers bind C's exact SHA — and the repo is **squash-only**, so that commit is destroyed by the very merge it authorizes. We bind 390 references to a SHA that does not survive. Cost is real and recurring: 3 train rebuilds today, O(N²) across lanes, and the ledger blames it for **four consecutive releases losing verified work**. The provenance goal is legitimate; binding to a doomed SHA is not the way to reach it. **The post-merge binding already exists and already works** — `bind-merged-console-authority-squash` fires on `closed`, binds the **surviving squash SHA**, and emits a receipt; verified green on #506, #507 and #508 today. So the pre-merge 390-reference rebind is paid to bind a commit that is destroyed, after which a job binds the real one anyway. **Recommend: keep the post-merge squash binding as the provenance anchor; reduce the pre-merge rebind to one per batch (§6), not one per lane.** |
| **Restating derived facts in prose across multiple documents** | Migration count, `include_str!` line numbers, worktree count and crate count were **simultaneously wrong in three planning docs** — 11 corrections in §11. Docs drift faster than anyone re-reads them. **Recommend: generate these into the docs from the tree, or reference one source; never restate.** |
| **oh-my-codex conductor provenance guard** | Fails closed on its *own* corrupted state and seals every codex tool call — *"the guard rejects all tools before execution"* — surviving `DISABLE_OMC=1`. Zero quality gained, an entire delegation path lost. **Recommend: `hooks = false` or repair the pointer** (owner's call; see `delegation-economics.md` §3.5). |
| **A gate that can never pass** | `authenticate-console-authority` was red on 4+ PRs, merged past twice by this program. It trained everyone to ignore CI, and was hiding a real RCE the whole time. Mirror image of a false-green. **Rule: a gate that cannot pass is a defect with the same severity as one that always passes.** |

### The distinction that decides it

Ceremony that **catches defects** is engineering excellence — adversarial review costs real time and
repeatedly earned it this session. Ceremony that **preserves an invariant the next step destroys** is
waste. The authority train is currently the second kind, and it is the single largest friction source
measured.

## 7. Bun mechanisms preserved

- **Errors grouped by crate, not file** — `cargo check -p <crate>`; one crate active per lane.
- **1 implementer + 2 adversarial reviewers + 1 fixer.** Reviewers get the **diff only**, are told to
  assume it is wrong, never see the implementer's reasoning. *The one mechanism that demonstrably
  worked here* — it caught 38 fabrications, 21 live-infra misclassifications, this session's
  incomplete authority-gate repair, and Rev 1's false premise.
- **`git stash` / `git reset` banned.** Commit or abandon. Atomic per-file commits.
- **"Edit the process, not the outputs."** Bad output → fix the prompt and rerun, never hand-patch.
- **Reject solutions needing paragraph-long justification** — a workaround that must be explained is a
  defect.

## 7.5 Delegation — codex / agy

Full analysis: `docs/ideas/delegation-economics.md`. Summary of what is **measured**, not assumed:

- `codex` is authenticated by **ChatGPT subscription**, so it draws from a **different budget pool**
  than this session. That is the real argument, not price: this session lost 2 of 7 then 1 of 6 agents
  to session limits, and one casualty was the *only* adversarial verifier, so a lane shipped
  unverified.
- **Cross-family adversarial review is the highest-value use.** Reviewer #1 Claude, reviewer #2
  `codex exec --model gpt-5.6-sol`. A second Claude reviewing Claude shares blind spots; this session
  an independent reviewer killed a premise I had verified twice.
- **`gpt-5.6-sol` completes in 61 steps / 60k output tokens** vs `claude-opus-5`'s 99 / 118k at a
  statistically indistinguishable pass rate (73±3 vs 74±4). **In a collision-sensitive fan-out, step
  count is a safety metric** — every step is another chance to touch a file outside the lane.
- **Do not reach for `claude-sonnet-5` as the "cheap mechanical" option** — it is the worst
  cost-per-success on the board (268 steps, $48.89) — nor `gemini-3.1-pro` (12% pass), which is also
  `agy`'s **silent fallback when `--model` is passed in the space-separated form**.
- **Measured: delegation overhead is large and roughly fixed** — >5 min for a lookup answerable in
  ~2 s in-session. Delegate whole lane slices, never facts. This killed a routing recommendation of
  mine within the same session it was written.

**Nothing above is trusted until calibrated.** `delegation-economics.md` §6 runs all candidate models
against the *already-solved* OrgUnit slice and routes by measured **files-touched-outside-lane**,
because the leaderboard does not measure collision — the property this fan-out actually depends on.

## 8. Pre-mortem (deliberate mode)

**S1 — The conformance target is edited to fit the implementation.** A lane hits friction and "fixes"
a driver or assertion; the immutable target silently becomes mutable and everything after is
unfalsifiable. *Mitigation:* drivers + assertions outside lanes; fixtures are the only lane-writable
part; CI diffs drivers/assertions independently. *Early warning:* any PR touching
`company-conformance/{scenarios,drivers}` from a lane branch.

**S2 — ~~Catalog digest thrash~~ — DELETED.** Its premise was false (§0). Recorded as deleted rather
than silently dropped, because the reasoning error is the reusable lesson.

**S2′ (replacement) — Install-order deadlock at bootstrap.** Lanes ship types whose links form a
cycle, or the ordered install array drifts from the declared links, and a fresh tenant bootstrap fails
with `link_target_not_found` while every lane's own worktree passes. *Mitigation:* CI gate 2 also
asserts the install order is a topological sort of declared `to_stable_key` links; a fresh-tenant
bootstrap runs in CI. *Early warning:* any lane declaring a link to a type later in the order.

**S4 — A lane's type ships wrong, and the correction is silently dropped.** In a five-lane fan-out a
type landing with a wrong property is a high-probability event. The natural fix — a new catalog
version carrying the correction — **installs green, returns success, and changes nothing.** The
repo's own test says so: *"an edit-only version must not revise a retained key — the edit is dropped,
silently"* (`builtin_catalog_additive_upgrade_as_runtime_role.rs:728`), and *"a caller has no signal
that its edit was dropped"* (`:667-668`). This is the one failure mode the engine actively hides, and
Rev 2 built its whole coordination model on additive installs without naming it.
*Mitigation:* CI gate 3 rejects any manifest whose `stable_key` set intersects an already-allowlisted
version's, so the dead correction never merges. **The real correction path is a new `stable_key` plus
deprecation of the old type — never an edit.** A type is only freely correctable *before* its version
is installed anywhere. *Early warning:* a lane proposing a second version of its own type.
**Consequence to price honestly: under Option B each lane gets essentially one shot at its type's
schema.**

**S3 — Scope reopens.** The engine makes an ERP module easy; "building blocks of a company" grows.
This system has been reframed three times; the failure mode is not wrong architecture, it is scope
outrunning delivery. *Mitigation:* the four-domain boundary is an explicit gate; the conformance suite
does not grow to accommodate additions. *Early warning:* a proposed type not traceable to a
conformance scenario step — `CATALOG.md` already requires each type to name the step it satisfies.

## 9. Test plan (expanded — deliberate mode)

- **Unit:** per-crate Rust tests; domain logic pure and DB-free where possible. Note an engine type is
  *data*, so its unit surface is thin — the load-bearing tier is integration.
- **Integration:** `#[sqlx::test]` against **disposable** Postgres (always `--rm`; 707 orphaned volumes
  once filled the VM), asserted **as the non-superuser runtime role** (superuser BYPASSRLS masks
  broken RLS), `--test-threads=1` for the known `XX000` flake. Every ontology integration test installs
  the catalog first (`config_object_types_as_runtime_role.rs:96` and siblings) — under Option B a lane
  installs **its own** version and is never blocked.
- **Conformance (immutable):** phase 1 scenarios through **both** drivers. The definition of done.
- **Authorization:** every Cedar policy either lowers to a SQL residual or fails **closed** with a
  named untranslatable term. Negative test: a non-privileged principal cannot read out-of-policy rows,
  asserted at the **REST** door where the arming actually happens.
- **Temporal:** as-of reconstruction diffed against the event log; effective-dated future changes must
  not leak into present reads.
- **Contract:** `openapi_drift` — **genuinely wired** at `ci.yml:597`
  (`//backend/app:console-app-itest-openapi_drift`).
- **Determinism:** same input → same output for every automated decision, rule string captured in the
  audit event. No AI/LLM judgment anywhere.
- **Observability:** every action emits an audit event on the fixity chain; conformance asserts the
  chain **verifies** after the full scenario, not merely that rows exist.

## 10. Acceptance criteria

1. `company-conformance` exists, drivers+assertions owned outside the lanes, and fails **red for the
   right reason** — which is now *defined*, not left to judgement. In one run: a **positive control**
   (`customer`, an existing built-in key) returns 200 **and** `org_unit` returns the specific
   unknown-type error code. A bare non-2xx is insufficient — a 404 is indistinguishable from a typo,
   an unseeded tenant, or a broken harness. Each phase publishes its **expected-red set**, so "red for
   the right reason *today*" is answerable rather than asserted.
   *Driver note:* `/object-types/{key}` is stable-key addressed (`rest/src/lib.rs:195`), but
   `/instances` takes a type **UUID** (`list_instances`, `:416`) — the driver must resolve key→UUID.
2. OrgUnit passes **its slice** through both drivers, reviewed by two adversarial diff-only reviewers.
   **"Slice" is defined by enumerating the assertion ids** in phase 1's ledger — otherwise the exit
   condition is self-certified and the implementer and reviewers can disagree about whether phase 2
   has ended.
3. **Fan-out dry run:** two lanes land a type slice with **zero overlapping file edits**, demonstrated
   on a real branch. *Satisfiable only because P0 pre-reserves the `mod` lines and install-order slots
   (§4, §5). Rev 2 mandated a shared-file append per lane, which made this criterion unpassable — and
   an unpassable gate gets weakened to unblock fan-out, the exact anti-pattern §6 names.*
4. Position, Person, Employment, PayRun exist as engine types per `CATALOG.md` — no bespoke tables,
   **no bespoke REST routes**.
5. As-of reconstruction of the org chart at an arbitrary past date matches the event log.
6. **6a (gate for fan-out, achievable today):** a principal without `RoleManage` is denied at the REST
   door, and a read with `app.current_org` unarmed vs armed is asserted through the driver — the RLS
   class this repo was actually burned by.
   **6b (NOT a fan-out gate — has an unscheduled dependency):** residual row-filtering, i.e. a
   non-privileged principal cannot read out-of-policy *rows*. Today every ontology endpoint gates on
   binary org-wide `RoleManage` and `list_instances` applies **no residual predicate**; the code says
   so itself — *"dark/unwired surface … L-WIRE assigns per-endpoint ontology features when it merges
   this router live"* (`rest/src/lib.rs:1646-1648`). Rev 2's criterion 6 would have passed vacuously
   or silently absorbed L-WIRE. **L-WIRE must be charted and scheduled before 6b is claimable.**
7. A tenant holding an earlier catalog version receives the new types **additively**, and a fresh
   tenant bootstraps through the full ordered install.
8. **A dropped edit is detected, not silent** (S4) — CI gate 3 rejects a manifest whose `stable_key`
   set intersects an already-allowlisted version's.

## 11. Corrections owed to existing documents

| Document | Claim | Correction |
|---|---|---|
| **this plan, Rev 1** | catalog digest is a whole-catalog global lock | **false** — digest is per-manifest, allowlist PK is `catalog_version` (§0) |
| `PLAN` §8 Follow-ups | buck2 "is dropped" | **RETAINED** — §0.5 corrected this; §8 still carries the original error |
| `PLAN` §2, §3 P4 | migrations highest `0168` | 204 files, highest **`0204`** |
| `PLAN` §2 | `include_str!` at `app/src/lib.rs:187`, `app/tests/openapi_drift.rs:6` | `backend/app/src/lib.rs:214`, `backend/app/tests/openapi_drift.rs:8` |
| `PLAN` §0.5 | 653 worktrees | consolidated to **6** |
| `PLAN` §0.5 | 128 crates / 31 groups | **160 workspace members** (150 under `crates/` in 38 groups + `app` + 9 `ci/gates/*`) |
| `PLAN` §5 S2 | substrate rewrite risk with a mitigation citing dropped P3 | marked MOOT at §0 but still listed — remove |
| `PLAN` §0.5 / §3 P2 | conformance boundary contradiction | resolved: two drivers, both against surfaces that exist today (§4) |
| `LANE-PROTOCOL` §4 | "`main` is not protected … the one true blocker" | **resolved 2026-07-28** — 12 required contexts, `strict`, `enforce_admins` |
| `LANE-PROTOCOL` §4 | "`openapi_drift.rs` is never invoked by CI" | **stale** — wired at `ci.yml:597` |
| `LANE-PROTOCOL` §4, §9 | migrations `0168`; 168 BUCK files; `app/src/lib.rs:187` | `0204`; **169**; `backend/app/src/lib.rs:214` |
| `CATALOG.md` §P3 cost model | engine type = "0 crates, 0 BUCK, 0 reindeer" | **correct as written** — Rev 1 wrongly proposed amending it. Add only: *one migration row for the allowlist digest, PK-disjoint per type* |

## 11.5 Still missing — owed before fan-out

Raised by Critic review of Rev 2; each is a gap, not a disagreement.

- **Batch rollback is undescribed.** §6 lands lanes as one integration branch → a single C+T. Reverting
  a merged squashed C+T with ~390 rebindings has no procedure, and `LANE-PROTOCOL` §4 attributes four
  consecutive hand-rebuilt releases to exactly this.
- **Lane progress observability has no schema.** §6 requires lanes to write findings to disk as they
  go — correct, it addresses the measured 2/7 and 1/6 agent mortality — but with no status schema and
  no aggregate view, *"lane 3 is stuck"* and *"lane 3 died"* stay indistinguishable. That is the exact
  condition that produced three agents going idle without reporting.
- **No ratchet on the conformance suite.** The scenario includes "run a pay cycle", which cannot go
  green until Lane 4 lands. The suite is therefore red for the program's entire duration; without a
  per-phase expected-red ledger (§10.1) nobody can tell a correct red from a regression.
- **`gen_first_party.py` divergence is inherited, unmentioned.** `LANE-PROTOCOL` §4 records that it
  discovers members by walking the filesystem and never reads `Cargo.toml`'s `members`, so the Buck2
  and cargo graphs can diverge indefinitely with no gate comparing them.
- **`git stash`/`reset` ban vs worktree hygiene** (§7 vs `LANE-PROTOCOL` §3 "a lane is returned
  clean") — the interaction is unstated.
- **Is `L-WIRE` chartered?** `rest/src/lib.rs:1647` names it as owner of per-endpoint ontology
  features, and criterion 6b depends on it. No charter was found. This decides whether 6b is a
  scheduling fix or a new dependency.

## 12. ADR

- **Decision:** Shard catalog expansion **by type**, each lane owning its own `catalog_version`,
  manifest file, allowlist migration, Cedar policy, tests and conformance fixture. Coordination is two
  CI gates, not a human owner. Gate fan-out behind an immutable dual-driver conformance suite bound to
  surfaces that already exist, and a hand-built OrgUnit reference.
- **Drivers:** catalog installs are already additive and version-keyed, so type-per-lane is structurally
  disjoint; the ontology REST surface is already generic, so types need no new routes; no immutable
  target exists yet; landing is serialized by the authority train.
- **Alternatives:** monolithic manifest + serial Catalog Owner (Rev 1 — rejected, false premise);
  refactor the digest chain (unnecessary, and risks the `0165:115-118` guarantee).
- **Consequences:** N catalog versions install in a database-enforced order; the conformance suite is a
  hard dependency on all parallel work; the Cedar coexistence map becomes a serialized shared file; the
  scope boundary must be actively defended.
- **Follow-ups:** amend the stale claims in §11 before any lane reads these documents; build the two CI
  gates before fan-out; run the §10.3 two-lane dry run; decide whether the residual
  candidate-execution in the `pull_request_target` authority job is removed.
