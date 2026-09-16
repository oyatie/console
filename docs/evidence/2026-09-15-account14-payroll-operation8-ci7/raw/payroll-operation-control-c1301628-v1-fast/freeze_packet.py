from pathlib import Path
import datetime, difflib, hashlib, json, re, subprocess, tempfile
ROOT=Path('/private/tmp/console-production-tdd-preparation-20260913')
OUT=Path('/private/tmp/payroll-operation-control-c1301628-v1-fast')
BASE='c13016282540c49546af1c0c75db45faef7fc466'
TARGET='backend/app/src/durability_composition_tests.rs'
TEST='required_workflow_provenance_refusal_is_operation_not_unknown'
sha=lambda b:hashlib.sha256(b).hexdigest()
def git(*args): return subprocess.check_output(['git', '-C', str(ROOT), *args])
def dump(name,obj): (OUT/name).write_text(json.dumps(obj,indent=2)+'\n')
base=git('show',f'{BASE}:{TARGET}')
assert sha(base)=='28100825e0cdbf3519fe70de8e1fb63ba10c5207007c67c66882ac6abd90a8a7'
assert (OUT/'base.rs').read_bytes()==base
assert (ROOT/TARGET).read_bytes()==base, 'Target preimage drift: stop import'
candidate=(OUT/'durability_composition_tests.rs').read_bytes()
old='        if fields.0.contains_key("outcome") {\n'
new='''        if fields.0.contains_key("outcome")
            || fields.0.get("message").is_some_and(|message| {
                message.starts_with("payroll draft staging failed;")
                    || message == "workflow payroll outbox drainer stopping"
            })
        {
'''
assert base.decode().count(old)==1
expanded=base.decode().replace(old,new)
assert candidate.decode().startswith(expanded+'\nimpl UnknownOutcomes {\n')
added=candidate.decode()[len(expanded):]
assert not any(x in added for x in ['impl PayrollDraftStaging','control("pause-replay")','control("clear-sync-policy")','UPDATE workflow_outbox_events','INSERT INTO payroll_draft_runs'])
roster=lambda s:re.findall(r'#\[sqlx::test\(migrations = false\)\]\nasync fn (\w+)\(',s)
base_roster=roster(base.decode()); candidate_roster=roster(candidate.decode())
assert len(base_roster)==7 and candidate_roster==base_roster+[TEST]
patch=''.join(difflib.unified_diff(base.decode().splitlines(True),candidate.decode().splitlines(True),fromfile='a/'+TARGET,tofile='b/'+TARGET))
assert (OUT/'candidate.patch').read_text()==patch
paths=[TARGET,'backend/app/src/workflow_drain.rs','backend/app/src/lib.rs','backend/crates/payroll/adapter-postgres/src/pay_run.rs','backend/crates/payroll/adapter-postgres/src/lib.rs','backend/crates/workflow/adapter-postgres/src/lib.rs','backend/crates/workflow/domain/src/lib.rs','backend/crates/kernel/core/src/error.rs','backend/crates/platform/test-support/src/lib.rs','backend/crates/platform/db/src/durability.rs','tools/lanes/recovery/supervise_recovery.py']
head=git('rev-parse','HEAD').decode().strip()
bindings=[]
for path in paths:
    tree=git('ls-tree',BASE,'--',path).decode().strip()
    mode,kind,blob=tree.split('\t')[0].split()
    assert mode in ('100644','100755') and kind=='blob'
    data=git('show',f'{BASE}:{path}')
    local=OUT/'sources'/BASE/path
    local.parent.mkdir(parents=True,exist_ok=True)
    local.write_bytes(data)
    current=(ROOT/path).read_bytes()
    bindings.append({'path':path,'mode':mode,'git_blob':blob,'sha256':sha(data),'retained_path':str(local.relative_to(OUT)),'observed_working_sha256':sha(current),'observed_working_matches_base':current==data})
dump('source-bindings.json',{'base_sha':BASE,'observed_shared_head':head,'target_preimage_matches':True,'rule':'All 11 authority/source bindings are immutable c130 Git blobs. Root implementation drift outside the unchanged target is expected, not a source-bind failure. Observed working hashes are a point-in-time readback only.','sources':bindings})
commands=[]
def run(argv,cwd):
    p=subprocess.run(argv,cwd=cwd,text=True,capture_output=True)
    commands.append({'argv':argv,'cwd':str(cwd),'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr})
    assert p.returncode==0,p.stderr
run(['rustfmt','--check','--edition','2024',str(OUT/'durability_composition_tests.rs')],OUT)
with tempfile.TemporaryDirectory(prefix='payroll-operation-private-mechanical-',dir='/private/tmp') as t:
    work=Path(t)
    target=work/TARGET; target.parent.mkdir(parents=True); target.write_bytes(base)
    p=str(OUT/'candidate.patch')
    run(['git','apply','--check',p],work)
    run(['git','apply',p],work)
    assert target.read_bytes()==candidate
    run(['git','apply','--reverse','--check',p],work)
    run(['git','apply','--reverse',p],work)
    assert target.read_bytes()==base
assert (ROOT/TARGET).read_bytes()==base
verification={'base_sha':BASE,'observed_shared_head':head,'verified_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'commands':commands,'counts':{'mechanical_commands_executed':len(commands),'rustfmt_checks_executed':1,'private_patch_checks_or_applies_executed':4,'source_cases_before':7,'source_cases_after':8,'runtime_cases_discovered':0,'runtime_cases_executed':0,'cargo_commands_executed':0,'database_commands_executed':0,'shared_repository_writes':0},'preservation':{'original_seven_test_bodies_byte_identical':True,'all_original_file_bytes_preserved_except_additive_logger_capture':True,'original_saw_unknown_predicate_byte_identical':True,'applied_candidate_byte_identical':True,'inverse_restores_base_byte_identical':True,'shared_target_still_base':True},'limitations':['Rustfmt parses syntax; no Rust typecheck or runtime result is claimed.','Eight is a source roster count, not Cargo-discovered or executed count.','Positive/negative log assertions and owner/standby preservation await root-supervised execution.']}
dump('mechanical-verification.json',verification)
command='CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "/private/tmp/console-production-tdd-preparation-20260913" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery durability_composition_tests::'+TEST+' -- --exact --test-threads=1 --nocapture'
guide=f'''# Ordinary payroll staging Operation control — private test candidate V1

Owner/author: `/root/fast_gate_review` (Fast). Root is sole integration writer and sole Cargo, PostgreSQL, Docker, CI, generated-file, and authority operator. Requested independent source reviewer: `/root/capture_review`; review verdict is pending in a separate receipt. This is additive acceptance authoring, not an implementer lane. Expected baseline is positive; green control does not admit an implementation lane.

Exact base: `{BASE}`. Observed shared head during mechanical verification: `{head}`. Immutable target: `{TARGET}`. Target preimage SHA256: `{sha(base)}`. Candidate SHA256: `{sha(candidate)}`. Patch SHA256: `{sha(patch.encode())}`. The retained 11 source files are Git regular blobs from the exact base; root's typed implementation has advanced some other files. Shared target remains the exact base preimage. All private packet files are bound by `manifest.json`; revise through a fresh version once frozen.

## Acceptance and scope

One new SQLx case, `{TEST}`, uses real App Worker construction, the actual Business pool, and the existing `seed_completion` engine helper. Before drainer startup, the actual PayRun owner's public staging port durably creates the same event/run/period/job/source draft with intentionally conflicting connector provenance. No fake stage implementation, synthetic stage result, observer result, or direct draft write supplies the refusal.

The actual spawned App drainer must positively log the ordinary staging-failure prefix plus the exact owner provenance-conflict error and event/source identity. The test rejects every captured `outcome=unknown` event correlated by event ID, run ID, or source. The message prefix recognizes existing c130 and typed implementation log suffixes; this is telemetry recognition in the test, not production classification from error text. The original `saw_unknown` predicate is unchanged.

The test waits for the real drainer stop log, which follows completion of its current pass, before closing App pools. It then requires exact equality of the helper's owner and standby snapshots: selected event, organization workflow runs, selected run nodes, organization payroll drafts and command receipts, and that event's workflow drain audits. It does not claim a whole-database snapshot. No ACK, attempt-count change, draft mutation, or successful drain audit is permitted in the observed refusal pass.

The original seven SQLx case bodies and all original helpers are byte-preserved, except the capture predicate additionally retains ordinary-stage errors and stop logs. The packet changes one test source file only. App8 inventory, CI wiring, static test cardinality, primary5 runtime, typed interface implementation and contracts are root-owned separate work.

## Mechanical import and runtime guide

1. Verify `manifest.json` hashes and its exact base/target preimage. Re-read the independent source review. Reject source or test semantic drift; create a new reviewed packet if needed.
2. Root serializes import. Confirm target SHA256 still equals the preimage above; run `git apply --check` on the exact patch, apply it, and verify the resulting target SHA256. This private packet never writes the shared target.
3. Root runs the exact new case in a fresh supervised process (global subscriber is exclusive). Do not run both log-capture cases in one process. Expected discovered/executed count: one/one, zero ignored, on a supported recovery environment. Capture the actual counts and first-failure evidence; a source roster count is not execution.

```sh
{command}
```

4. Preserve primary5 and existing App7; any subsequent roster or CI expansion remains serialized and independently reviewed. No test removal, skip, quarantine, or weakened assertion is authorized.

`mechanical-verification.json` records one rustfmt parser/check and four isolated git apply/check/inverse operations, candidate/inverse byte equality, original7 preservation, 8 source cases, and 0 runtime cases. There is no Cargo typecheck or database evidence from Fast. Root must bind runtime evidence to its final exact candidate SHA and classify any failure before claiming success.

## Pre-mortem and controls

- False refusal witness from unrelated traffic: positive assertion binds exact event/source and exact conflict error; negative UNKNOWN assertion binds any event/run/source match. Fresh-process subscriber avoids cross-case capture.
- Fixture accidentally targets a different natural key: construct the owner's request from real event/run, PostgreSQL-cast and zipped period, and actual job; only connector differs. Confirm the persisted real draft source and unchanged selected history before starting the drainer.
- Cleanup masks incorrect late ACK: await real drainer stop after ordinary refusal before pool close, then compare both owner and standby snapshots.
- Compile/environment failure mistaken for semantics: no runtime result is claimed here. Root records exact discovery/execution and the first actual failure; a missing ordinary witness alone needs log/source classification.

Blast radius: one additive test and its existing telemetry collector, private artifacts only. Detection: exact positive ordinary refusal, negative UNKNOWN correlation, stopped-pass witness and equality snapshots. Rollback: root may reverse only this exact additive patch after verifying candidate bytes, preserving other work and all historical evidence; no destructive shared Git operation. Stop conditions: target preimage mismatch, incompatible typed interface, omitted original case/assertion, inability to supervise a fresh exact test, failed independent review, or absent real ordinary-refusal witness. Remaining HOLDs: compilation/runtime acceptance and App8 CI/cardinality pending root evidence; approval/admission UNKNOWN, 408/lost-response contract and all unrelated account/native-enrollment work remain separate. No production, payment, legal/compliance, or global capacity claim is authorized.
'''
(OUT/'GUIDE.md').write_text(guide)
files=[]
for f in sorted(OUT.rglob('*')):
    if f.is_file() and f.name!='manifest.json':
        data=f.read_bytes(); files.append({'path':str(f.relative_to(OUT)),'size_bytes':len(data),'sha256':sha(data)})
manifest={'version':1,'status':'frozen_for_independent_source_review','owner':'/root/fast_gate_review','base_sha':BASE,'observed_shared_head':head,'target':TARGET,'preimage_sha256':sha(base),'candidate_sha256':sha(candidate),'patch_sha256':sha(patch.encode()),'test_name':TEST,'expected_baseline':'PASS; ordinary control is not a RED implementation lane','source_case_counts':{'before':7,'after':8},'runtime_cases_executed':0,'files':files}
dump('manifest.json',manifest)
print(json.dumps({'manifest_sha256':sha((OUT/'manifest.json').read_bytes()),'candidate_sha256':sha(candidate),'patch_sha256':sha(patch.encode()),'files':len(files),'source_bindings':len(bindings),'observed_head':head,'source_drift_paths':[s['path'] for s in bindings if not s['observed_working_matches_base']],'mechanical_commands':len(commands)},indent=2))
