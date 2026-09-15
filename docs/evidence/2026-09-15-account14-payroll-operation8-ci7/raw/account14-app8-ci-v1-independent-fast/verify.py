from pathlib import Path
import hashlib,json,re,subprocess,tempfile
ROOT=Path('/private/tmp/console-production-tdd-preparation-20260913');P=Path('/private/tmp/account14-app8-ci-0a625d32-inventory-v1');OUT=Path('/private/tmp/account14-app8-ci-v1-independent-fast');BASE='0a625d323be1184f94af549e20c7f47bfbd63702'
sha=lambda b:hashlib.sha256(b).hexdigest()
def git(*args):return subprocess.check_output(['git','-C',str(ROOT),*args])
m=json.loads((P/'manifest.json').read_text());assert sha((P/'manifest.json').read_bytes())=='10f02dd6af65419f8cc6a13498c3a692a64173f12ed956a96af94145ed6040f1'
for path,digest in m['files'].items():assert sha((P/path).read_bytes())==digest,path
assert sha(Path(m['map_binding']['path']).read_bytes())==m['map_binding']['sha256']
for path,digest in m['admission_bindings'].items():assert sha(Path(path).read_bytes())==digest,path
paths=list(m['postimages']);assert len(paths)==7
for path in paths:
 b=(P/'base'/path).read_bytes();post=(P/'post'/path).read_bytes()
 assert git('show',f'{BASE}:{path}')==b and (ROOT/path).read_bytes()==b
 assert sha(post)==m['postimages'][path] and (P/'validation'/path).read_bytes()==post
app=git('show',f'{BASE}:backend/app/src/durability_composition_tests.rs');assert sha(app)==m['app8_source_sha256']=='0f02b3aec6f15449015b22a54b70453b66f2a5d17fb7670f3a241cce53b2a7cb'
names=['durability_composition_tests::'+x for x in re.findall(r'#\[sqlx::test\(migrations = false\)\]\nasync fn (\w+)\(',app.decode())];assert len(names)==8
new=names[3:];assert len(new)==5
read=lambda side,path:(P/side/path).read_text()
# Independently undo every authorized textual change and require exact base.
wf='.github/workflows/ci.yml'; before=read('base',wf);after=read('post',wf)
cond="${{ !cancelled() && needs.preflight.outputs.run_live_postgres == 'true' && steps.recovery-checkout.outcome == 'success' && steps.recovery-toolchain.outcome == 'success' && steps.recovery-image.outcome == 'success' }}"
commands=[]
for name in new:
 cmd='CONSOLE_RECOVERY_CUT= python3 tools/lanes/recovery/supervise_recovery.py "$GITHUB_WORKSPACE" -- cargo test --locked --manifest-path backend/Cargo.toml -p console-app --lib --features test-recovery '+name+' -- --exact --test-threads=1 --nocapture'
 block='      - name: App recovery '+name.split('::')[1]+'\n        if: '+cond+'\n        run: '+cmd+'\n\n'
 assert after.count(block)==1;after=after.replace(block,'');commands.append(cmd)
assert after==before
for path in ['scripts/lib/recovery-test-invocations.mjs','tools/buck/test_preparation_wiring.py']:
 before=read('base',path);after=read('post',path)
 for name in new:
  line=next(line for line in after.splitlines(True) if line.strip()=='"'+name+'",');assert after.count(line)==1;after=after.replace(line,'')
 assert after==before,path
path='scripts/lib/recovery-test-invocations.test.mjs'
assert read('post',path).replace('the isolated installer and eight app cases','the isolated installer and three app cases').replace('assert.equal(result.invocations.length, 18);','assert.equal(result.invocations.length, 13);')==read('base',path)
path='scripts/check-ci-preflight.mjs'; before=read('base',path);after=read('post',path);pattern=r'const recoveryCommands = (\[[\s\S]*?\]);';old_block=re.search(pattern,before);new_block=re.search(pattern,after);oldcmds=json.loads(old_block[1]);newcmds=json.loads(new_block[1]);assert len(oldcmds)==13 and newcmds==oldcmds+commands
after=after.replace(new_block[0],old_block[0])
for i,name in enumerate(new,13):
 line=f'    proofRun("App recovery {name.split("::")[1]}", recoveryCommands[{i}], {{ if: recoveryRunCondition }}),\n';assert after.count(line)==1;after=after.replace(line,'')
assert after==before
path='scripts/check-ci-preflight.test.mjs';before=read('base',path);after=read('post',path)
for newv,oldv in [('observer and eight app cases.','observer and three app cases.'),('"postgres-reachability-domain-b": 22,','"postgres-reachability-domain-b": 17,'),('assert.equal(runStepCount, 156,','assert.equal(runStepCount, 151,'),('Five additional app composition proofs extend the matrix: 156*3 = 468.','Three app composition proofs extend the matrix: 151*3 = 453.'),('assert.equal(mutationCount, 468,','assert.equal(mutationCount, 453,')]:assert after.count(newv)==1;after=after.replace(newv,oldv)
assert after==before
path='docs/program/executed-tests-baseline.json';before=json.loads(read('base',path));after=json.loads(read('post',path))
for path,old,newv in [('backend/app/src/lib.rs',192,197),('backend/crates/identity/adapter-postgres/tests/deactivate_revokes_credentials.rs',6,14),('backend/crates/ontology/rest/src/lib.rs',19,21)]:assert before['test_attribute_baseline'][path]==old and after['test_attribute_baseline'][path]==newv;after['test_attribute_baseline'][path]=old
after['why_app_recovery_variant_is_pinned']=after['why_app_recovery_variant_is_pinned'].replace('eight additional live two-node durability cases','three additional live two-node durability cases');assert after==before
# Existing private tracked index: no candidate source drift, same exact paths,
# and no extra altered tracked file beyond the admitted seven postimages.
val=P/'validation'; idx=subprocess.check_output(['git','-C',str(val),'ls-files','-s','-z']);base_entries=git('ls-tree','-r','-z',BASE)
def parse_entries(rows,index=False):
 out={}
 for r in rows.split(b'\0'):
  if not r:continue
  meta,path=r.split(b'\t',1);parts=meta.decode().split();out[path.decode()]=(parts[0],parts[1] if index else parts[2])
 return out
base_map=parse_entries(base_entries);index_map=parse_entries(idx,True);assert len(index_map)==9867 and set(index_map)==set(base_map)
changed=sorted(path for path in index_map if index_map[path]!=base_map[path]);assert changed==sorted(paths)
for path in paths:
 post=(P/'post'/path).read_bytes();oid=hashlib.sha1(b'blob '+str(len(post)).encode()+b'\0'+post).hexdigest();assert index_map[path]==('100644',oid)
assert subprocess.check_output(['git','-C',str(val),'diff','--name-only'])==b''
commands_run=[]
def run(argv,cwd,log=None):
 r=subprocess.run(argv,cwd=cwd,capture_output=True,text=True);record={'argv':argv,'cwd':str(cwd),'exit_code':r.returncode}
 if log:(OUT/log).write_text(r.stdout+r.stderr);record['log']=log;record['log_sha256']=sha((OUT/log).read_bytes())
 else:record.update(stdout=r.stdout,stderr=r.stderr)
 commands_run.append(record);assert r.returncode==0,r.stderr;return r
with tempfile.TemporaryDirectory(prefix='ci7-fast-apply-',dir='/private/tmp') as t:
 work=Path(t)
 for path in paths:
  target=work/path;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes((P/'base'/path).read_bytes())
 patch=str(P/'candidate.patch')
 run(['git','apply','--check',patch],work);run(['git','apply',patch],work)
 for path in paths:assert (work/path).read_bytes()==(P/'post'/path).read_bytes()
 run(['git','apply','--reverse','--check',patch],work);run(['git','apply','--reverse',patch],work)
 for path in paths:assert (work/path).read_bytes()==(P/'base'/path).read_bytes()
result=run(['node','--test','scripts/lib/recovery-test-invocations.test.mjs'],val,'independent-probe.log')
for field,value in [('tests',114),('pass',114),('fail',0),('cancelled',0),('skipped',0)]:assert re.search(r'\b'+field+r' '+str(value)+r'\b',result.stdout)
assert subprocess.check_output(['git','-C',str(val),'diff','--name-only'])==b''
# Verify reported raw validation and retained environment log bindings.
vr=json.loads((P/'validation-results.json').read_text())
for r in vr['results']+vr['retained_environment_attempts']:assert sha((P/r['log']).read_bytes())==r['log_sha256']
for path,digest in m['files'].items():assert sha((P/path).read_bytes())==digest
for path in paths:assert (ROOT/path).read_bytes()==(P/'base'/path).read_bytes()
mechanical={'candidate_manifest_sha256':sha((P/'manifest.json').read_bytes()),'candidate_patch_sha256':sha((P/'candidate.patch').read_bytes()),'base_sha':BASE,'packet_files_verified':len(m['files']),'admission_bindings_verified':3,'map_verified':True,'preimage_postimage_paths':paths,'source_names':names,'original13_workflow_commands_conditions_order_preserved':True,'all_original_checker_and_test_bytes_preserved_except_named_growth':True,'baseline_changes_only_three_counts_and_note':True,'private_index_paths':9867,'private_index_changed_paths':changed,'private_working_matches_index':True,'mechanical_commands':commands_run,'new_node_tests_executed':114,'new_node_tests_passed':114,'new_node_tests_skipped':0,'new_cargo_commands':0,'new_database_commands':0,'new_docker_commands':0,'shared_writes':0,'frozen_packet_unchanged':True}
(OUT/'mechanical.json').write_text(json.dumps(mechanical,indent=2)+'\n')
print(json.dumps({'mechanical_sha256':sha((OUT/'mechanical.json').read_bytes()),'packet_files':len(m['files']),'commands_executed':len(commands_run),'node_passed':114,'private_index_paths':9867},indent=2))
