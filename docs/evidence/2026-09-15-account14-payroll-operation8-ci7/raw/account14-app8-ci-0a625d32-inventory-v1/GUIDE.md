# Account14 / App8 admitted CI candidate

Private author `/root/inventory_repair`; root remains sole integration, CI,
generated-file, Cargo and database writer. Base:
`0a625d323be1184f94af549e20c7f47bfbd63702`. Seven exact preimages are pinned in
`preimages.json`; the final App8 source is
`0f02b3aec6f15449015b22a54b70453b66f2a5d17fb7670f3a241cce53b2a7cb`.
Independent reviewer: `/root/fast_gate_review`.

Root admission is `/private/tmp/account14-app8-ci-0a625d32-admission/lane.json`
and its `result.json`. The existing probe was84 executed,82PASS/2FAIL with the
live App8 source outside the old three-case selection. This author did not repeat
that root RED or modify its evidence. Private same-command verification below is
GREEN. This package is pending independent review before root import.

The patch adds five separately supervised App commands after the original13,
preserving their exact argv, order, fault modes, setup-success conditions and
failure propagation. Both App rosters name the same eight source cases. The
shared source/command oracle still rejects extra, missing, ignored, disabled or
misrouted cases. Existing per-case mutation loops naturally add30 Node controls.
The explicit supervised union count grows13→18; domain-b run steps17→22;
all run steps151→156; the three-class bypass matrix453→468. No old control is
removed or weakened.

Account14 stays in its existing complete unfiltered target. The static attribute
baseline pins Account6→14, App192→197 and ontology/rest19→21. Its named dark set,
dark_baseline1, deferred sets and feature variants remain exact. No Buck, map,
shard placement, job metadata, aggregate, setup digest, lockfile or product change.
The entire parsed workflow returns to its exact base after removing only the
five new steps; source checks also prove exact old checker/control preservation.

Validation uses a private archive of the exact base, with only these seven files
replaced and node_modules linked read-only for installed dependencies. A private
Git index contains the original9867 paths and differs from base only at these
seven paths; its tree is `f25e2dd2341f8427a093c97afd1f8beab481f7aa`.
The validation snapshot is support material, not an import target. Import only
the patch or seven regular files under `post/` after approval.

Executed checks, with logs retained:

- `node --test scripts/lib/recovery-test-invocations.test.mjs`:114/114PASS,
  0ignored/skipped; same admitted executable/arguments with private output log.
- `PYTHONDONTWRITEBYTECODE=1 python3 tools/buck/test_preparation_wiring.py -v`:
  15/15PASS.
- `npm run check:executed-tests`:15/15 Node regressionsPASS, then resolverPASS;
  raw output387defined,387reachable,3193static attributes,dark1 unchanged.
- `node scripts/check-ci-preflight.mjs`:PASS with private tracked index.
- `node --test --test-name-pattern="accepts the workflow's cheap preflight and
  protected expensive jobs|rejects every run-step condition, soft-failure, and
  retained-text early-exit bypass" scripts/check-ci-preflight.test.mjs`:
  selected2/executed2/PASS2, including all468 bypass mutations. This is not a
  claim that the whole preflight test suite ran here.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools/lanes/recovery
  -p 'test_*.py' -v`:21/21PASS with approved access to inspect its owned test
  subprocess via ps. Docker is stubbed by these unit tests; real Docker/DB0.
- `node verify-source.mjs`:10 source preservation checksPASS.
-10 mechanical checksPASS, including exact seven-path indexed diff and four
  private git apply/check/inverse commands. Applied postimages and restored
  originals match exactly. No shared writes, Cargo calls or Rust test executions.

Earlier environment attempts remain in the package: the archive initially had
no Git index, so the direct preflight gate failed domain-unit source resolution
and the focused suite reported1PASS/1FAIL. Building the required private index
resolved both without changing candidate source. The first recovery-unit run was
20PASS/1ERROR because the sandbox denied ps for the owned-process death check;
the unchanged21-case rerun with approved process access passed. These are retained
as environment evidence and are not counted as product failures or admission RED.

Mechanical guide for root: verify the immutable preimages, obtain Fast's review,
apply `candidate.patch` serially, rerun the admitted probe and required affected
gates against the actual integration head. Preserve genuine discovered/executed
counts and any unexpected first failure. The root process owns remaining full
preflight, generated/reachability and final CI closure.

Pre-mortem: a stale roster rejects all supervised reachability, grouping commands
hides later cases, or baseline-only growth masks unwired selectors. Blast radius:
seven CI/accounting files. Detection: exact rosters, old-command inverse proof,
same probe and mutation tests. Rollback: reject this private patch before import;
after import, root inverses only the reviewed CI change while retaining source
and failed evidence. Stop on source drift, unreviewed selector, weakened checks,
unexpected failure or scope expansion. Lenses: Essentialism, Systems Thinking,
Red Team, Operability / Day-2, Blast radius, Zero trust.

HOLD: independent Fast approval, root exact-head import/verification/full required
gates, complete product runtime closure and production authority. No additional
runtime, deployment, compliance or release conclusion is claimed.
