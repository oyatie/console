import json, os, pathlib, subprocess, uuid
repo=pathlib.Path('/private/tmp/console-production-tdd-preparation-20260913')
out=pathlib.Path('/private/tmp')/('recovery-test-local-cfg-v2-run-'+uuid.uuid4().hex);out.mkdir()
env=os.environ.copy();env.update(CARGO_TARGET_DIR='/private/tmp/console-account30-cargo',RUSTUP_TOOLCHAIN='1.98.1',RUSTC_WRAPPER='',SQLX_OFFLINE='true')
rows=[]
commands=[(name,['cargo','run','--locked','-p','console-gate-'+name],repo/'backend') for name in ('audit-coverage','rls-arming')]
commands.append(('recovery-roster',['node','--test','scripts/lib/recovery-test-invocations.test.mjs'],repo))
for name,command,cwd in commands:
 with (out/(name+'.log')).open('w') as f:r=subprocess.run(command,cwd=cwd,env=env,stdout=f,stderr=subprocess.STDOUT)
 rows.append({'name':name,'command':command,'cwd':str(cwd),'exit':r.returncode})
(out/'result.json').write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'results':rows},indent=2)+'\n')
print(out)
raise SystemExit(0 if all(r['exit']==0 for r in rows) else 1)
