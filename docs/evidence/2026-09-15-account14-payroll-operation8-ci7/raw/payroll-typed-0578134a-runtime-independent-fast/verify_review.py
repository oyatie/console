from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path('/private/tmp/console-production-tdd-preparation-20260913'); OUT=Path('/private/tmp/payroll-typed-0578134a-runtime-independent-fast')
RUNTIME=Path('/private/tmp/payroll-59c38d60-runtime-v2'); VERIFY=Path('/private/tmp/payroll-59c38d60-verification'); ADMISSION=Path('/private/tmp/payroll-typed-c1301628-admission')
TYPED='59c38d607745e00c8e8e89d9d6e15ed163688aa3'; FIX='0578134acc1099c7d96738300c310bbea6449e25'; OBSERVED='0a625d323be1184f94af549e20c7f47bfbd63702'
sha=lambda b:hashlib.sha256(b).hexdigest()
def git(*args):return subprocess.check_output(['git','-C',str(ROOT),*args])
def keep(label,data):
 p=OUT/label; p.parent.mkdir(parents=True,exist_ok=True); p.write_bytes(data); return {'path':label,'sha256':sha(data),'size_bytes':len(data)}
paths=['backend/crates/ontology/rest/openapi/paths/api__v1__ontology__actions__action_key__execute.post.yaml','backend/openapi/openapi.yaml']
assert git('rev-parse',FIX+'^').decode().strip()==TYPED
assert git('diff','--name-only',TYPED,FIX).decode().splitlines()==paths
bindings=[]
for path in paths:
 b=git('show',f'{FIX}:{path}'); bindings.append(keep('sources/'+path,b)); assert b==git('show',f'{OBSERVED}:{path}')
 text=b.decode()
 for phrase in ['Canonical projected owners commit their effect','For canonical projected actions, after service recovery retry','Replaying a confirmed canonical receipt also','other projected handlers own their audit behavior.','A request timeout (408), disconnect, or lost response','do not prove rollback, and receipt absence alone does not prove no effect.']:
  assert text.count(phrase)==1,(path,phrase)
 assert 'Action executed; the mutation and its audit row committed atomically.' not in text
patch=git('diff','--binary','--full-index',TYPED,FIX); keep('correction.diff',patch)
check_cmd=['git','-C',str(ROOT),'diff','--check',TYPED,FIX]; check=subprocess.run(check_cmd,capture_output=True,text=True); assert check.returncode==0
case_names={1:'required_api_completion_unknown_reconciles_same_command_after_replay',2:'required_api_receipt_absent_unknown_renews_approval_with_same_command',4:'required_workflow_typed_unknown_preserves_pending_and_failed_events'}
records=json.loads((RUNTIME/'probe-results.json').read_text()); assert len(records)==3
keep('runtime/probe-results.json',(RUNTIME/'probe-results.json').read_bytes())
base_records={r['case']:r for r in json.loads(Path('/private/tmp/payroll-typed-c1301628-red/results.json').read_text())}
summary_re=re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;')
classified=[]
for r in records:
 case=int(r['lane'].rsplit('-',1)[1]); name=case_names[case]; b=(RUNTIME/f'{case}.log').read_bytes(); s=b.decode()
 assert r['sha']==TYPED and r['exit']==0 and sha(b)==r['log_sha256']
 assert b==(ADMISSION/f'{case}.log').read_bytes()
 assert r['command'].split(' > ',1)[0]==base_records[case]['command']
 assert s.count('running 1 test\n')==1
 assert f'test durability_composition_tests::{name} ...' in s
 counts=summary_re.findall(s); assert counts==[('1','0','0','0','29')]
 assert 'panicked at' not in s and 'test result: FAILED' not in s
 assert re.search(r'recovery-fixture evidence=.+ exit=0\n?$',s)
 evidence=re.search(r'recovery-fixture run=([^ ]+) evidence=(.+)',s); assert evidence
 keep(f'runtime/{case}.log',b)
 classified.append({'case':case,'test_name':name,'sha':TYPED,'command':r['command'],'command_matches_initial_base_probe_without_redirection':True,'status':'PASS','counts':{'executed':1,'passed':1,'failed':0,'ignored':0,'measured':0,'filtered_out':29},'log_sha256':sha(b),'supervisor_run':evidence.group(1),'supervisor_evidence_path':evidence.group(2),'supervisor_exit':0})
assert [c['case'] for c in classified]==[1,2,4]
# Source identity allows a completed PASS to prove the whole reviewed case ran,
# rather than merely the assertion that failed on baseline.
app=git('show',f'{TYPED}:backend/app/src/durability_composition_tests.rs')
assert sha(app)=='28100825e0cdbf3519fe70de8e1fb63ba10c5207007c67c66882ac6abd90a8a7'
assert app==Path('/private/tmp/payroll-typed-unknown-e27623ab-tests-v4-capture/post/backend/app/src/durability_composition_tests.rs').read_bytes()
keep('sources/backend/app/src/durability_composition_tests.rs',app)
# Environment denial is independent of product semantics and ran no test.
denied_records=json.loads((VERIFY/'probe-results.json').read_text()); assert len(denied_records)==1
r=denied_records[0]; b=(VERIFY/'1.log').read_bytes(); assert r['exit']==1 and sha(b)==r['log_sha256']
assert 'permission denied while trying to connect to the docker API' in b.decode() and 'running 1 test' not in b.decode()
keep('denied/1.log',b);keep('denied/probe-results.json',(VERIFY/'probe-results.json').read_bytes())
unit=[]
commands={
'ontology-rest.log':'CARGO_TARGET_DIR=/private/tmp/console-account30-cargo RUSTUP_TOOLCHAIN=1.98.1 RUSTC_WRAPPER= SQLX_OFFLINE=true cargo test --locked --manifest-path backend/Cargo.toml -p console-ontology-rest --lib projected_dispatch_derivation -- --test-threads=1 --nocapture > /private/tmp/payroll-59c38d60-verification/ontology-rest.log 2>&1',
'workflow-domain.log':'CARGO_TARGET_DIR=/private/tmp/console-account30-cargo RUSTUP_TOOLCHAIN=1.98.1 RUSTC_WRAPPER= SQLX_OFFLINE=true cargo test --locked --manifest-path backend/Cargo.toml -p console-workflow-domain --lib -- --test-threads=1 > /private/tmp/payroll-59c38d60-verification/workflow-domain.log 2>&1'
}
for name,expected,filtered in [('ontology-rest.log',11,10),('workflow-domain.log',5,0)]:
 b=(VERIFY/name).read_bytes(); s=b.decode(); counts=summary_re.findall(s); assert counts==[(str(expected),'0','0','0',str(filtered))]
 names=re.findall(r'^test (\S+) \.\.\. ok$',s,re.M);assert len(names)==expected and len(set(names))==expected
 assert s.count(f'running {expected} tests\n')==1
 keep('units/'+name,b)
 unit.append({'log':name,'sha':TYPED,'command':commands[name],'command_metadata_source':'Root message; raw log independently confirms package, executed roster and totals but does not embed invocation or Git head.','root_reported_tool_exit':0,'status':'PASS','test_names':names,'counts':{'executed':expected,'passed':expected,'failed':0,'ignored':0,'measured':0,'filtered_out':filtered},'log_sha256':sha(b)})
keep('openapi-correction.log',(VERIFY/'openapi-correction.log').read_bytes())
# Ensure the independently retained historical admission RED logs were not
# replaced by the later success logs; classify prior assertion boundaries.
red_preservation=[]
for case in [1,2,4]:
 b=(VERIFY/'admission-red-preserved'/f'{case}.log').read_bytes();s=b.decode();assert 'test result: FAILED. 0 passed; 1 failed; 0 ignored;' in s
 assert (('app/src/durability_composition_tests.rs:1251:5' in s) if case in [1,2] else ('app/src/durability_composition_tests.rs:1639:10' in s))
 red_preservation.append({'case':case,'path':str(VERIFY/'admission-red-preserved'/f'{case}.log'),'sha256':sha(b),'classification':'Historical semantic RED retained independently of later success log.'})
report={
'reviewer':'/root/fast_gate_review','verdict':'APPROVE_BOUNDED_F1_CORRECTION_AND_CLASSIFY_SAVED_TYPED_RUNTIME_GREEN',
'implementation_sha':TYPED,'implementation_tree':git('rev-parse',TYPED+'^{tree}').decode().strip(),'correction_sha':FIX,'correction_tree':git('rev-parse',FIX+'^{tree}').decode().strip(),'correction_diff_sha256':sha(patch),'observed_later_head':OBSERVED,
'previous_review_sha256':'947c7a9b6df8e1ef68928dd92d0abce3a4468d8142ceba930066047788bc52d8',
'finding_resolution':{'id':'F1','status':'CLOSED_AT_0578134a','conclusion':'Both source and generated OpenAPI now scope immutable receipt, split canonical audit, same-command reconciliation and audit repair to canonical projected owners. The 200 description explicitly leaves other projected handlers their own audit behavior. Generic 408/disconnect/lost-response uncertainty and no-rollback-inference caution remain. These corrected blobs remain unchanged at observed0a625d32.','source_bindings':bindings},
'runtime_cases':classified,
'acceptance_reached':[
{'case':1,'assessment':'Completed test passes positive native payroll SyncRep witness, bounded fixed503 envelope, no snapshot-visible effect/receipt/audit while native completion remains waiting, resume and backend disappearance, immutable one receipt/draft and idempotent same-command audit repair, repeated replay and standby equality.'},
{'case':2,'assessment':'Completed test passes actual observer EXECUTE removal/effective denial/restoration, fixed503 after governance consumption with absent effect/receipt/audit, spent-reference403, fresh approval changing only four_eyes_request_ref, exactly one owner effect/receipt/audit, retained original decision/consumption plus second history, repeated replay and standby equality.'},
{'case':4,'assessment':'Completed test reaches both PENDING and FAILED iterations, actual native staging confirmation, local draft with no ACK/receipt/drain audit, typed unknown telemetry, exact event preservation during unknown, and recovery using the same event/draft with one ACK attempt increment and one audit plus standby equality. Two internal status histories remain one executed SQLx case.'}
],
'unit_suites':unit,
'failed_environment_attempt':{'path':str(VERIFY/'1.log'),'log_sha256':denied_records[0]['log_sha256'],'sha':TYPED,'command':denied_records[0]['command'],'exit':1,'classification':'ENVIRONMENT_DENIED_BEFORE_TEST_EXECUTION','executed':0,'reason':'Permission denied connecting to the existing Docker socket. Preserved separately; not product RED and not a skipped test.'},
'historical_admission_red_preservation':red_preservation,
'counts':{'saved_successful_invocations_reviewed':5,'saved_runtime_cases_executed':19,'saved_runtime_cases_passed':19,'saved_runtime_cases_failed':0,'saved_runtime_cases_ignored':0,'saved_environment_denied_invocations':1,'saved_environment_denied_cases_executed':0,'new_cargo_invocations_by_reviewer':0,'new_database_commands_by_reviewer':0,'shared_repository_writes_by_reviewer':0},
'reviewer_mechanical_commands':[{'argv':check_cmd,'exit_code':check.returncode,'stdout':check.stdout,'stderr':check.stderr}],
'limits':['Runtime evidence is bound to59c38d60; docs correction is bound to0578134a. Later Account5b85ab7b and Operation0a625d32 are not final-head runtime proof from these saved results.','Ordinary Operation control is independently source-approved and now integrated, but its runtime result is outside this receipt.','Existing App3, confirmed-owner/audit500 control, recovery9, period-lock suite and final CI enrollment still require their scoped final evidence; none is inferred from these19 tests.','No newly executed408/disconnect case, global capacity, native enrollment, production, legal/compliance or payment-execution claim.'],
'pending_findings_in_bounded_correction':[],
'lenses':['Cartesian doubt','Essentialism / YAGNI','Red Team','Operability / Day-2','Blast-radius / cell-based','Zero-trust / defense-in-depth']
}
(OUT/'review.json').write_text(json.dumps(report,indent=2)+'\n')
files={str(f.relative_to(OUT)):sha(f.read_bytes()) for f in sorted(OUT.rglob('*')) if f.is_file() and f.name!='manifest.json'}
(OUT/'manifest.json').write_text(json.dumps({'implementation_sha':TYPED,'correction_sha':FIX,'files':files},indent=2)+'\n')
print(json.dumps({'review_sha256':sha((OUT/'review.json').read_bytes()),'manifest_sha256':sha((OUT/'manifest.json').read_bytes()),'correction_tree':report['correction_tree'],'correction_diff_sha256':sha(patch),'counts':report['counts']},indent=2))
