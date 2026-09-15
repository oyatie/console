from pathlib import Path
import hashlib,json,mmap,re,subprocess
ROOT=Path('/private/tmp/console-production-tdd-preparation-20260913'); OUT=Path('/private/tmp/account14-0a625d32-green-independent-fast'); EVIDENCE=Path('/private/tmp/account-deactivation14-probe-21367db1cb414e57b6dbf995f0349fcd')
HEAD='0a625d323be1184f94af549e20c7f47bfbd63702';SOURCE='backend/crates/identity/adapter-postgres/tests/deactivate_revokes_credentials.rs'
sha=lambda d:hashlib.sha256(d).hexdigest()
def git(*args):return subprocess.check_output(['git','-C',str(ROOT),*args])
contract=Path('/private/tmp/account-deactivation14-probe-v2/contract.json').read_bytes();assert sha(contract)=='28329ea489e991ae32dc8b84ac33e8f0f500ce0d5fb64d2d207fa7d15fa1b478'; contract=json.loads(contract)
probe=Path('/private/tmp/account-deactivation14-probe-v2/probe.py').read_bytes();assert sha(probe)=='b45d1aa74eb2a980e6103682d17b1381a50d72a416d892f397f8c818af172e71'
driver=Path('/private/tmp/account-deactivation6-1cef89ea-v1-fast/driver.sh').read_bytes();assert sha(driver)==contract['driver_sha256']
receipt=json.loads((EVIDENCE/'driver-receipt.json').read_text());assert receipt['driver_sha256']==sha(driver)
for name,digest in receipt['artifacts'].items(): assert sha((EVIDENCE/name).read_bytes())==digest,name
before=(EVIDENCE/'source-before.json').read_bytes();after=(EVIDENCE/'source-after.json').read_bytes();assert before==after
source=json.loads(before);assert source['head']==HEAD and source['status_porcelain']=='';files=source['files'];assert len(files)==1005
# Validate every captured path against regular immutable Git blobs in one batch.
tree={}
for row in git('ls-tree','-r','-z',HEAD).split(b'\0'):
 if row:
  meta,path=row.split(b'\t',1);tree[path.decode()]=meta.decode().split()
for path in files:assert tree[path][0] in ['100644','100755'] and tree[path][1]=='blob'
payload=''.join(f'{HEAD}:{path}\n' for path in files).encode();batch=subprocess.run(['git','-C',str(ROOT),'cat-file','--batch'],input=payload,capture_output=True);assert batch.returncode==0
raw=batch.stdout;pos=0
for path,digest in files.items():
 end=raw.index(b'\n',pos); oid,kind,size=raw[pos:end].decode().split();assert kind=='blob' and oid==tree[path][2];pos=end+1;size=int(size);blob=raw[pos:pos+size];assert sha(blob)==digest,path;pos+=size;assert raw[pos:pos+1]==b'\n';pos+=1
assert pos==len(raw)
assert files[SOURCE]==contract['source_sha256']=='e06b77f8841cf3e7f5bc595756c1b8933c136e9cbfae1a536fac2862375dc471'
log=(EVIDENCE/'probe.log').read_bytes();text=re.sub(r'\x1b\[[0-9;]*m','',log.decode())
rows=re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',text,re.M);assert [n for n,_ in rows]==contract['roster'] and all(s=='ok' for _,s in rows)
assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',text)==[('ok','14','0','0','0','0')]
assert text.count('running 14 tests\n')==1
assert 'Compiling console-platform-test-support v0.1.0 (/private/tmp/console-production-tdd-preparation-20260913/backend/crates/platform/test-support)' in text
assert 'cargo test --locked --manifest-path backend/Cargo.toml -p console-identity-adapter-postgres --test deactivate_revokes_credentials -- --test-threads=1' in text
assert receipt['probe_exit']==0 and (EVIDENCE/'probe-exit.txt').read_text().strip()=='0'
assert receipt['source_pre_post']=='exactly equal' and receipt['prebuild_interception_count']==1
assert (EVIDENCE/'containers-after.txt').read_bytes()==b''
assert (EVIDENCE/'cleanup-final-list-error.txt').read_bytes()==b''
assert receipt['cleanup']=='confirmed owned container absent via successful bounded docker ps -a'
# Read the current artifact without executing it, to qualify repaired path custody.
binary=Path('/private/tmp/console-account30-cargo/debug/deps/deactivate_revokes_credentials-7f1d12c6a5032217')
with binary.open('rb') as f:
 data=mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ)
 binary_record={'path':str(binary),'sha256':hashlib.sha256(data).hexdigest(),'size_bytes':len(data),'contains_intended_test_support_manifest':data.find(b'/private/tmp/console-production-tdd-preparation-20260913/backend/crates/platform/test-support')>=0,'contains_old_test_support_manifest':data.find(b'/private/tmp/console-payroll-retained-capacity-regression-20260915/backend/crates/platform/test-support')>=0};data.close()
assert binary_record['contains_intended_test_support_manifest'] and not binary_record['contains_old_test_support_manifest']
retained={}
for name in ['driver-receipt.json','source-before.json','source-after.json','probe.log','probe-exit.txt','cleanup.txt','containers-after.txt','prebuild-dispatch.txt']:
 b=(EVIDENCE/name).read_bytes();(OUT/name).write_bytes(b);retained[name]=sha(b)
(OUT/'driver.sh').write_bytes(driver);(OUT/'probe.py').write_bytes(probe);(OUT/'contract.json').write_text(json.dumps(contract,indent=2)+'\n')
(out_source:=OUT/'deactivate_revokes_credentials.rs').write_bytes(git('show',f'{HEAD}:{SOURCE}'))
mech={'head':HEAD,'driver_sha256':sha(driver),'probe_sha256':sha(probe),'all_receipt_artifact_hashes_verified':True,'receipt_artifact_count':len(receipt['artifacts']),'source_prepost_byte_equal_clean_head':True,'regular_git_source_blobs_checked':1005,'test_source_matches_exact_approved_v4':True,'raw_test_roster':rows,'counts':{'executed':14,'passed':14,'failed':0,'ignored':0,'filtered':0,'cargo_invocations_in_saved_run':1,'new_cargo_invocations_by_reviewer':0,'new_database_commands_by_reviewer':0,'shared_writes_by_reviewer':0},'binary_readback':binary_record,'cleanup_confirmed':True,'retained_artifacts':retained}
(OUT/'mechanical.json').write_text(json.dumps(mech,indent=2)+'\n')
print(json.dumps({'mechanical_sha256':sha((OUT/'mechanical.json').read_bytes()),'source_blobs':1005,'driver_receipt_sha256':sha((EVIDENCE/'driver-receipt.json').read_bytes()),'probe_log_sha256':sha(log),'binary_readback':binary_record},indent=2))
