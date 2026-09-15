# Ordinary payroll staging Operation control — private test candidate V1

Owner/author: `/root/fast_gate_review` (Fast). Root is sole integration writer and sole Cargo, PostgreSQL, Docker, CI, generated-file, and authority operator. Requested independent source reviewer: `/root/capture_review`; review verdict is pending in a separate receipt. This is additive acceptance authoring, not an implementer lane. Expected baseline is positive; green control does not admit an implementation lane.

Exact base: `c13016282540c49546af1c0c75db45faef7fc466`. Observed shared head during mechanical verification: `59c38d607745e00c8e8e89d9d6e15ed163688aa3`. Immutable target: `backend/app/src/durability_composition_tests.rs`. Target preimage SHA256: `28100825e0cdbf3519fe70de8e1fb63ba10c5207007c67c66882ac6abd90a8a7`. Candidate SHA256: `0f02b3aec6f15449015b22a54b70453b66f2a5d17fb7670f3a241cce53b2a7cb`. Patch SHA256: `0e62463ef150e376bd0216e21128e3f3f3643ea8b299bc68a485e6e30180a12b`. The retained 11 source files are Git regular blobs from the exact base; root's typed implementation has advanced some other files. Shared target remains the exact base preimage. All private packet files are bound by `manifest.json`; revise through a fresh version once frozen.

## Acceptance and scope

One new SQLx case, `required_workflow_provenance_refusal_is_operation_not_unknown`, uses real App Worker construction, the actual Business pool, and the existing `seed_completion` engine helper. Before drainer startup, the actual PayRun owner's public staging port durably creates the same event/run/period/job/source draft with intentionally conflicting connector provenance. No fake stage implementation, synthetic stage result, observer result, or direct draft write supplies the refusal.

The actual spawned App drainer must positively log the ordinary staging-failure prefix plus the exact owner provenance-conflict error and event/source identity. The test rejects every captured `outcome=unknown` event correlated by event ID, run ID, or source. The message prefix recognizes existing c130 and typed implementation log suffixes; this is telemetry recognition in the test, not production classification from error text. The original `saw_unknown` predicate is unchanged.

The test waits for the real drainer stop log, which follows completion of its current pass, before closing App pools. It then requires exact equality of the helper's owner and standby snapshots: selected event, organization workflow runs, selected run nodes, organization payroll drafts and command receipts, and that event's workflow drain audits. It does not claim a whole-database snapshot. No ACK, attempt-count change, draft mutation, or successful drain audit is permitted in the observed refusal pass.

The original seven SQLx case bodies and all original helpers are byte-preserved, except the capture predicate additionally retains ordinary-stage errors and stop logs. The packet changes one test source file only. App8 inventory, CI wiring, static test cardinality, primary5 runtime, typed interface implementation and contracts are root-owned separate work.

## Mechanical import and runtime guide

1. Verify `manifest.json` hashes and its exact base/target preimage. Re-read the independent source review. Reject source or test semantic drift; create a new reviewed packet if needed.
2. Root serializes import. Confirm target SHA256 still equals the preimage above; run `git apply --check` on the exact patch, apply it, and verify the resulting target SHA256. This private packet never writes the shared target.
3. Root runs the exact new case in a fresh supervised process (global subscriber is exclusive). Do not run both log-capture cases in one process. Expected discovered/executed count: one/one, zero ignored, on a supported recovery environment. Capture the actual counts and first-failure evidence; a source roster count is not execution.

```sh
CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "/private/tmp/console-production-tdd-preparation-20260913" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery durability_composition_tests::required_workflow_provenance_refusal_is_operation_not_unknown -- --exact --test-threads=1 --nocapture
```

4. Preserve primary5 and existing App7; any subsequent roster or CI expansion remains serialized and independently reviewed. No test removal, skip, quarantine, or weakened assertion is authorized.

`mechanical-verification.json` records one rustfmt parser/check and four isolated git apply/check/inverse operations, candidate/inverse byte equality, original7 preservation, 8 source cases, and 0 runtime cases. There is no Cargo typecheck or database evidence from Fast. Root must bind runtime evidence to its final exact candidate SHA and classify any failure before claiming success.

## Pre-mortem and controls

- False refusal witness from unrelated traffic: positive assertion binds exact event/source and exact conflict error; negative UNKNOWN assertion binds any event/run/source match. Fresh-process subscriber avoids cross-case capture.
- Fixture accidentally targets a different natural key: construct the owner's request from real event/run, PostgreSQL-cast and zipped period, and actual job; only connector differs. Confirm the persisted real draft source and unchanged selected history before starting the drainer.
- Cleanup masks incorrect late ACK: await real drainer stop after ordinary refusal before pool close, then compare both owner and standby snapshots.
- Compile/environment failure mistaken for semantics: no runtime result is claimed here. Root records exact discovery/execution and the first actual failure; a missing ordinary witness alone needs log/source classification.

Blast radius: one additive test and its existing telemetry collector, private artifacts only. Detection: exact positive ordinary refusal, negative UNKNOWN correlation, stopped-pass witness and equality snapshots. Rollback: root may reverse only this exact additive patch after verifying candidate bytes, preserving other work and all historical evidence; no destructive shared Git operation. Stop conditions: target preimage mismatch, incompatible typed interface, omitted original case/assertion, inability to supervise a fresh exact test, failed independent review, or absent real ordinary-refusal witness. Remaining HOLDs: compilation/runtime acceptance and App8 CI/cardinality pending root evidence; approval/admission UNKNOWN, 408/lost-response contract and all unrelated account/native-enrollment work remain separate. No production, payment, legal/compliance, or global capacity claim is authorized.
