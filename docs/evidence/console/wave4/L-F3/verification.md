# L-F3 — code-grammar unification · Stage-2 adversarial verification

Lane: `w4-f3-grammar` · worktree `w4-f3-grammar-20260725` · branch
`claude/w4-f3-grammar-20260725` · base `4cabe239`.

Verifier did not author the code under review. Everything below was re-derived
from the tree, not taken from the build stage's report.

---

## 1. Verdict

**The refactor is real and the fix is at the root cause.** Two commits
(`5d1e70ae` test-first, `1f2d83e2` refactor) converge the composer's hand-rolled
bare-code regex and the object-card's prefix→slug map onto
`ontology/codeGrammar` + the runtime object-type registry. Three of the
charter's four call sites are genuinely converged; the fourth (`KIND_META`)
cannot be, for a reason verified below.

Two findings were opened by this pass and **fixed here**; two are recorded
unfixed with their reasons.

## 2. Root cause, re-derived

The charter's claim is that adding an object code costs four frontend edits.
Re-derived at base `4cabe239`:

| Site | Pre-refactor state | Verified |
|---|---|---|
| `composer/grammar.ts` `BARE_CODE_RE` | `/(^\|[\s([{])([A-Z]{1,8}-[0-9]{1,10}(?:-[0-9]{1,6})?)/gu` — never imported `codeGrammar` | yes |
| `objectcard/kinds.ts` `COMPOSER_KIND_TO_SLUG` | `Partial<Record<string, string>>`, five rows, a typo silently unlinks a kind | yes |
| `objectcard/kinds.ts` `SLUG_META` | nine hardcoded label/tone rows | yes |
| `composer/objectKinds.ts` `KIND_META` | eleven hardcoded kinds (prefix/tone/label) | yes |

The regex was the root cause of three distinct defects, all reproduced:

- **uppercase-only prefix** — `Bid` is in `FALLBACK_CODE_PREFIXES` and could
  never match;
- **digit-only body** — `OT-FINANCE`, `PAY-CHO` never matched;
- **truncation** — `WO-2026-Q1-07` matched only `WO-2026`, because the second
  segment was `[0-9]{1,6}`.

The fix replaces the *body* with `objectCodeBodySource()` and keeps the boundary
rule `(^|[\s([{])` local. That split is correct and load-bearing: `codeGrammar`'s
own regexes are `\b`-anchored, and `\b` would match `.WO-1` and `가WO-1`, which
the composer's grammar (shared with `@` and `#`) must not.

## 3. Red-green proof (run by this pass, not quoted)

The equivalence bar and the deltas were re-proven by reverting the three source
files to `4cabe239` with the tests left in place.

```
git checkout 4cabe239 -- composer/grammar.ts composer/objectKinds.ts objectcard/kinds.ts
npx vitest run src/console/composer/codeGrammarConvergence.test.ts
  -> 8 failed | 106 passed (114)
```

The 8 reds are exactly the intended deltas (3 "fixed", 1 "narrowed", 2 "one
edit", plus the 2 added by this pass). **Zero equivalence rows failed** — the
106 assertions covering all 21 fallback prefixes × parse shape and the frozen
`kindFromCode`/`linkTargetFromCode` truth table are green against both the old
and the new code, which is what makes this a refactor rather than a rewrite.

Isolation run — `objectcard/kinds.ts` reverted alone, the other two at HEAD:

```
  × linkifies, resolves a card slug and labels it once primed
    AssertionError: expected undefined to deeply equal { kind: 'deal', id: 'DL-1042' }
  -> 1 failed | 111 passed
```

So the registry fallback in `linkTargetFromCode` is independently load-bearing,
not carried by the grammar change. It is on a live path:
`useObjectCard.ts:189` `addRelation`.

**Type-binding claim re-verified, not accepted.** Injecting `codePrefix: "ZZ-"`
into `KIND_META` and running `npx tsc -b --force`:

```
src/console/composer/objectKinds.ts(70,15): error TS2322: Type '"ZZ-"' is not
assignable to type '"AP-" | "WO-" | "AT-" | ... | undefined'.
```

Reverted after the check.

## 4. Findings opened by this pass

### F-1 · `slugLabel`'s registry fallback was proven only by a fabricated fixture — FIXED

`slugLabel` now falls back to `registeredObjectType(slug)?.description`, and the
lane's test proves it with `description: "영업 기회"`. **No row the backend
actually seeds looks like that.** Every `object_types.description` in the repo is
English prose:

| Migration | Example |
|---|---|
| `0102_create_object_types_and_links.sql:29-45` | `org_unit` → `'Organizational unit (region/branch/worksite)'` |
| `0115_seed_identity_object_kinds.sql:14-16` | `account` → `'Login account (user credential subject)'` |
| `0131_create_series.sql:17` | `series` → `'A user-authored series grouping recurring instances'` |
| `0188_create_attendance_console.sql:7` | `attendance_exception` → `'Employee attendance exception'` |

`GET /api/v1/object-types` (`backend/app/src/objects.rs:1560-1599`) serves that
column verbatim. So against real data the fallback injects English prose — in
some rows a full parenthetical caption — into a Korean UI label slot, which is a
DESIGN §4-12 breach the `check-ui-strings` gate structurally cannot catch
(it scans source literals, not runtime values).

The behaviour is *consistent with an existing repo practice*
(`modules/typeRegistry.ts:352-354` already uses `description` as a dynamic type's
name, with a comment calling it "the backend's human label"), so it was not
reverted unilaterally — that pattern is outside this lane's roots and reverting
here would fork it.

**Fixed by making the contract explicit and executable** rather than by changing
behaviour:

- `slugLabel`'s doc comment now carries a TRUTH BOUND naming the four migrations
  and stating that a Korean label from this path is an obligation on the lane
  that registers the type — **L-X7 owes it for `deal`**.
- A new test pins the real shape (`series` → the English prose verbatim)
  alongside the Korean-fixture test, so the wiring proof and the contract proof
  can no longer be mistaken for each other.

### F-2 · the named chip gap was prose, now executable — FIXED

The build stage correctly reported that the zero-edit claim covers parse, card
slug and card label but **not** the composer chip: `kindFromCode` returns the
closed `ObjectKind` union, so a runtime-registered prefix has no tone and no
Korean label and `TokenText.tsx:73-84` renders it as inert raw text. That was
recorded only in prose, where it can rot.

Pinned as an assertion in the "one edit" describe: after priming `DL-`,
`spans()` yields a `codeLink`, `linkTargetFromCode` yields
`{kind:"deal", id:"DL-1042"}`, and `kindFromCode("DL-1042")` is **`undefined`**.
L-X10 now discovers the boundary in CI rather than in review.

## 5. Findings recorded, not fixed

### F-3 · `slugLabel` / `slugTone` / `SLUG_META` are dead code (pre-existing, D-5)

Zero production consumers, at base **and** at HEAD:

```
git grep -n "slugLabel\|slugTone\|SLUG_META" 4cabe239 -- web/src
  -> only objectcard/kinds.ts itself
```

They are not in `objectcard/index.ts`, and `ObjectCard.tsx` renders relation
chips from `relation.linkType`, never from `slugLabel`. `ko.console.objectCard.kinds`
(9 keys, `ko.ts:1205-1215`) exists solely to feed `slugLabel`, so the whole
subtree is unreachable. **D-5 says delete.**

Not deleted here: the natural wire site is `ObjectCard.tsx` (L-F2's root, this
lane's `must_not_touch`), `ko.ts` is a shared collision root, and L-X10's
`must_not_touch` includes `console/objectcard/**` — so neither dependent lane
could re-add it. Deleting a helper two in-flight lanes may intend to wire is a
cross-lane break, not a cleanup. **Route to L-F2 or L-X10: wire it or delete it.**

Consequence for the lane's claim: the "new object code costs zero frontend
edits" story is fully true for **parse** (`grammar.ts`) and **card slug**
(`linkTargetFromCode`, live via `useObjectCard`), and true-but-unreachable for
**card label** (`slugLabel`).

### F-4 · `react-router-dom` — real, but pre-existing on the SPINE, not wave-4

The build stage flagged this as something that "will otherwise read as a wave-4
regression at integration". Verified: it is already on the spine.

```
git grep -c react-router-dom 4cabe239 -- web/src              -> 12 files
(spine pr488-design-mirror-sync @ 0fb0d1e3) same 12 files
web/package.json:31  "react-router": "^8.3.0"
node_modules/react-router -> 8.3.0 installed (hoisted at repo root)
node_modules/react-router-dom -> absent, and absent from package-lock.json
```

react-router 7 folded `react-router-dom` into `react-router`, so v8 ships no
`-dom` package. Six console screens import a module that is neither declared nor
resolvable. This single cause produces all 7 `tsc` errors and all 10 failing test
files on this branch. **Not introduced by wave 4 and not this lane's to fix** —
but it is a live red on the spine, so the integrator should not attribute it to a
wave-4 lane.

## 6. Accepted widening, re-checked

An alphanumeric code body means text like `IN-HOUSE`, `AT-HOME`, `C-suite` or
`R-value` after a *registered* prefix now parses as a `codeLink` where it did not
before. Re-checked and accepted:

- it is **already** the grammar in `window/objDrag.ts:56,67`,
  `messenger/messengerModel.ts:42` and the approval composer, all of which build
  from the same `codeGrammar` source, so this **removes** a divergence — the same
  text already rendered differently in the messenger than in the composer;
- it renders inert: `TokenText.tsx:81-84` returns `span.raw` for any span whose
  kind is unknown **or** whose `resolveObject` misses, so an unresolvable
  `C-suite` is plain text, not a dead link;
- the only side effect is `TokenComposer.tsx:167-185`, where `hasBareCode`
  triggers one permission-scoped object fetch. A wasted fetch, not a leak — and
  the narrowing half (`COVID-19` no longer parses) removes more spurious fetches
  than the widening adds.

The `COVID-19` behaviour change is confirmed **not user-visible**: pre-refactor
it parsed and `kindFromCode` returned `undefined`, so `TokenText` already
rendered it as raw text. `TokenText.test.tsx`'s inert-render case passes
identically before and after.

## 7. Ownership

Files touched across both lane commits and this verification pass:

```
web/src/console/composer/grammar.ts
web/src/console/composer/objectKinds.ts
web/src/console/composer/grammar.test.ts
web/src/console/composer/codeGrammarConvergence.test.ts
web/src/console/objectcard/kinds.ts
docs/evidence/console/wave4/L-F3/verification.md
```

All inside the declared roots. **No shared collision root touched** — no
`ko.ts`, no `nav.ts`, no `screens/registry.ts`, no `openapi.yaml`, no migration,
no `backend/app/**`. **No manifest is owed by this lane**: the zero-edit path
deliberately requires no `ko.ts` key, which is the point of the refactor.
`codeGrammar.ts` is listed as an L-F3 root by the charter but was correctly left
untouched — the hook it needed already existed.

## 8. Gate set, re-run after the fixes

From `web/`:

| Gate | Result |
|---|---|
| `npx vitest run src/console/composer/codeGrammarConvergence.test.ts` | **114/114** |
| `npx vitest run` over composer, objectcard, ontology, module, messenger, window, components/console, lib | **592/592**, 49 files |
| `npx vitest run` (full web) | 2807/2808 · 10 files failed — all F-4, byte-identical with the lane stashed |
| `npx tsc -b --force` | 7 errors, **all** F-4, **zero** in owned files (`grep -cE "composer\|objectcard\|ontology"` → 0) |
| `npx eslint src/console/composer src/console/objectcard src/console/ontology --max-warnings 0` | clean |
| `node scripts/check-ui-strings.mjs` | clean |
| `node scripts/check-console-purity.mjs` | clean, 570 files |
| stub scan (`TODO\|FIXME\|XXX\|HACK\|test.skip\|.only\|placeholder\|stub`) over all 5 touched source files | clean |

Frontend bar: no `className` was introduced (these are three pure-logic modules
and their tests — the purity gate passes over all 570 console files); no token
colours changed; no inline Hangul in non-test source; no captions added; no
a11y-bearing markup touched.
