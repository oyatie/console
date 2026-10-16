# Original full Account Buck run: immutable failure audit

The original run executed 325 Rust test bodies: 286 passed and 39 failed, 0 ignored and 0 filtered, in 11,120.98 seconds. This historical full run remains RED. This audit performed no rerun, test alteration, or repository edit.

## Custody

Buck trace 73046cd6-08ba-46d4-835f-c3f9d8bce692 records git base 01e069883b998d9c430500c85a6d464c263d4ae3 **with local changes**. Do not label it an exact later source candidate. Full dirty source tree and full test binary SHA are not established. buck-custody-extract.json preserves exact launch argv, executor, threads=1, timeout=21600, revision events and test output hash. Its action digest is an action identity, not a binary SHA. The declared wrapper image is pinned PostgreSQL18.4; observed server version, memory allocation and original server logs were not captured. Private credential-file content was not read. Original human log is preserved as original-buck-log.gz; each complete failure block is separately hashed.

## Logged failure groups

| Count | Actual logged cause | Disposition |
|---:|---|---|
| 19 | Required distinct startup LOGIN environment NotPresent | Startup credential transport subsequently repaired, targeted Cargo exact designation proved green; do not relabel all19 passed. |
| 2 | bounded clean retry Elapsed(()) | Exact post-repair rerun required; no cause assumed from deadline alone. |
| 4 | SQL57014 statement timeout during catalog finalizers | Later JIT/coordination repair relevant; these four exact tests are not proved green by available lifecycle evidence. |
| 1 | raw concurrent finalizer SQL55P03 lock timeout on organizations | Same retained concurrent case later passes within28/28 lifecycle; fullsuite remains unqualified. |
| 7 | UnexpectedEof, expected5bytes but read0 | Original server cause unknown. Collect original runtime/server evidence if retained; otherwise rerun with observed server capture. Do not infer OOM. |
| 1 | SSLRequest returned0x00 in SQLx setup | Original server cause unknown; preserve TLS and assertion, capture runtime evidence. |
| 1 | anonymous root Cache-Control missing | Current response defect, plus statically missing discoverable actions; reuse existing public native page in bounded root composition. |
| 3 | service_unavailable versus unavailable | Route-specific imported oracle conflicts with retained JSON contract; reviewed semantic reconciliation required. |
| 1 | actual actor foreign-key catalog mismatch |40missing keys and39legacy references; requires owning transition, not fixture substitution. |

failures.json gives every exact test name, category, panic location, original log line, full block hash, and actionable exact Buck probe command. test-outcomes.json records all325outcomes and was checked to exactly match all39panic blocks. actor-catalog-mismatches.json enumerates all40affected columns without collapsing79diagnostics into79tests.

## Subsequent repair evidence, bounded claims

Startup transport commit f275e8afd061bdc0725c8183fe725d97fb8fec9f has24/24Cargo transport tests, two passing Buck shell suites, and one actual designation Cargo probe1/1 at base6d2f085d. The captured subsequent /private/tmp/console-startup-buck-designation-v2.log failed to build because native_company_setup.rs was not found in the staged Buck source tree; it is not a successful actual Buck execution. A future successful descendant proof may supersede this hold, never rewrite this observation.

Finalizer repairs f7458c73199383b446f225d47801e0634a36cb28 and3432fbd5594c13effdc0497b81f9cf9061c1c371 have6/6targeted probes,28/28retained lifecycle, cold-create/retry1each and5corruption controls. Exact original concurrent finalizer test is green in that lifecycle. Separate16GiB ordinary cohort30/30functional outcomes retains cold output-pipe LEAK operational hold. Neither a separate earlier8GiB cohort OOM nor separate nextest pipe diagnosis establishes the cause of the original Buck/libtest EOFs. No workload or production qualification is claimed.

## Next boundary order

1. Rebind current source and repair actual Buck source prerequisite, then run exact actual designation probe under the repaired transport; preserve secret confidentiality and capture observed database/runtime identity.
2. Rerun the original19startup-prerequisite failures under correct transport, plus the exact2retry and4statement-timeout bodies after finalizer repairs. Do not substitute aggregate test count for names.
3. Rerun sevenEOF and oneSSLsetup failures with server logs, observed version/resources and exit/restart information. EOF alone never establishes OOM, and changing TLS is not diagnosis.
4. Independently admit the current root response RED, the reviewed route-specific error oracle integration, and actor key transition. The last is substantial canonical ownership work requiring populated migration validation.
5. Only after exact focused repairs pass, rerun the original full target on one source candidate with full environment custody. Keep operational cleanup, browser, workload and production qualification separate.

All commands in failures.json are proposals only, not executed evidence. Missing owner probes, missing infrastructure and zero executed tests cannot qualify a lane. Full-plan release acceptance remains open.
