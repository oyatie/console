import hashlib,json,os,pathlib,re,subprocess,time
repo=pathlib.Path('/private/tmp/console-production-tdd-preparation-20260913')
sha='ec673d27bb2e29e1d52b6e9372bd7516ac125eae'
out=pathlib.Path('/private/tmp/console-ec673d27-recovery');out.mkdir()
workflow=(repo/'.github/workflows/ci.yml').read_text()
commands=re.findall(r'^\s+run: (CONSOLE_RECOVERY_CUT=.*supervise_recovery\.py.*)$',workflow,re.M)
assert len(commands)==18
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==sha
assert not subprocess.check_output(['git','status','--porcelain'],cwd=repo)
env=os.environ.copy();env.update(CARGO_TARGET_DIR='/private/tmp/console-account30-cargo',RUSTUP_TOOLCHAIN='1.98.1',RUSTC_WRAPPER='',SQLX_OFFLINE='true',DOCKER_CONTEXT='colima-console-custody-20260914',GITHUB_WORKSPACE=str(repo))
results=[]
for i,command in enumerate(commands,1):
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==sha
 assert not subprocess.check_output(['git','status','--porcelain'],cwd=repo)
 name=command.split(' -- --exact')[0].split()[-1];log=out/f'{i:02}.log';started=time.monotonic()
 with log.open('w') as f:proc=subprocess.run(command,cwd=repo,env=env,shell=True,stdout=f,stderr=subprocess.STDOUT)
 text=log.read_text();counts=re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',text)
 valid=proc.returncode==0 and len(counts)==1 and counts[0][:5]==('ok','1','0','0','0')
 row={'index':i,'name':name,'sha':sha,'command':command,'exit':proc.returncode,'seconds':round(time.monotonic()-started,2),'counts':counts,'exact_one_pass':valid,'log_sha256':hashlib.sha256(log.read_bytes()).hexdigest()};results.append(row)
 (out/'results.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(row),flush=True)
 if not valid:raise SystemExit(1)
print('All18 exact supervised cases passed.',flush=True)
