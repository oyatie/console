from pathlib import Path
import hashlib,json,subprocess
ROOT=Path('/private/tmp/console-production-tdd-preparation-20260913')
OUT=Path('/private/tmp/payroll-typed-59c38d60-independent-fast')
PACK=Path('/private/tmp/payroll-typed-unknown-e27623ab-tests-v4-capture')
BASE='c13016282540c49546af1c0c75db45faef7fc466'; HEAD='59c38d607745e00c8e8e89d9d6e15ed163688aa3'
sha=lambda d:hashlib.sha256(d).hexdigest()
def git(*args):return subprocess.check_output(['git','-C',str(ROOT),*args])
assert sha((PACK/'DESIGN.md').read_bytes())=='1ca5c188e600c036ba6309e29498cbc9aa58db5ae6956bcc80c7b72575ca2cbd'
assert sha((PACK/'compatibility.patch').read_bytes())=='5f824ceec87f4d1bad7ec264041c3fd502a37f09a2abb153c7c60a6c9d54f72a'
assert sha((PACK/'manifest.json').read_bytes())=='e14dfe66a20403ef23bba7136403000b950edfbac4f90e1b31f9cdec5ef670b0'
paths=git('diff','--name-only',BASE,HEAD).decode().splitlines(); assert len(paths)==10
patch=git('diff','--binary','--full-index',BASE,HEAD);(OUT/'candidate.diff').write_bytes(patch)
source_records=[]
for path in paths+['backend/app/src/lib.rs','backend/app/src/workflow_drain.rs','backend/app/src/durability_composition_tests.rs','backend/crates/platform/db/src/durability.rs']:
    data=git('show',f'{HEAD}:{path}'); f=OUT/'sources'/path; f.parent.mkdir(parents=True,exist_ok=True); f.write_bytes(data)
    mode,kind,blob=git('ls-tree',HEAD,'--',path).decode().split('\t')[0].split(); assert kind=='blob' and mode in ['100644','100755']
    source_records.append({'path':path,'candidate_git_blob':blob,'candidate_sha256':sha(data),'base_sha256':sha(git('show',f'{BASE}:{path}')),'changed':path in paths})
compat_paths=['backend/crates/ontology/rest/src/projected_dispatch_derivation.rs','backend/crates/payroll/adapter-postgres/tests/recovery.rs','backend/crates/workflow/adapter-postgres/tests/payroll_drain_period_lock.rs']
commands=[]
for path in compat_paths:
    data=(PACK/'compatibility-post'/path).read_bytes()
    argv=['rustfmt','--edition','2024','--emit','stdout']
    p=subprocess.run(argv,input=data,capture_output=True)
    commands.append({'argv':argv,'stdin_source':str(PACK/'compatibility-post'/path),'stdin_sha256':sha(data),'exit_code':p.returncode,'stderr':p.stderr.decode(),'stdout_sha256':sha(p.stdout)})
    assert p.returncode==0 and p.stdout==git('show',f'{HEAD}:{path}'),path
p=subprocess.run(['git','-C',str(ROOT),'diff','--check',BASE,HEAD],capture_output=True,text=True)
commands.append({'argv':['git','-C',str(ROOT),'diff','--check',BASE,HEAD],'exit_code':p.returncode,'stdout':p.stdout,'stderr':p.stderr}); assert p.returncode==0
for path in ['backend/app/src/durability_composition_tests.rs','backend/app/src/workflow_drain.rs','backend/crates/platform/db/src/durability.rs']:
    assert git('show',f'{BASE}:{path}')==git('show',f'{HEAD}:{path}')
result={'base_sha':BASE,'candidate_sha':HEAD,'candidate_tree':git('rev-parse',f'{HEAD}^{{tree}}').decode().strip(),'candidate_diff_sha256':sha(patch),'changed_paths':paths,'sources':source_records,'compatibility_postimages_match_after_rustfmt':compat_paths,'preserved_app7_and_durability_primitive':True,'commands':commands,'counts':{'changed_paths':10,'bound_source_files':14,'mechanical_commands_executed':4,'compatibility_postimages_compared':3,'runtime_tests_discovered':0,'runtime_tests_executed':0,'cargo_invocations':0,'database_commands':0,'shared_repository_writes':0}}
(OUT/'mechanical.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'candidate_tree':result['candidate_tree'],'candidate_diff_sha256':sha(patch),'mechanical_sha256':sha((OUT/'mechanical.json').read_bytes()),'counts':result['counts']},indent=2))
