from pathlib import Path
import subprocess,tempfile,shutil,os,signal
root=Path.cwd()
keys=['DATABASE_URL','CONSOLE_APALIS_OWNER_DATABASE_URL','CONSOLE_APALIS_RUNTIME_DATABASE_URL','CONSOLE_APALIS_ADMIN_DATABASE_URL']
for key in keys:
 with tempfile.TemporaryDirectory(prefix='console-buck-content-oracle-') as d:
  dest=Path(d)/'tools/buck';dest.mkdir(parents=True)
  for name in ['test_needs_postgres.sh','test_needs_postgres.test.sh']:shutil.copy2(root/'tools/buck'/name,dest/name)
  f=dest/'test_needs_postgres.sh';s=f.read_text();needle="'"+key+'=%s';assert s.count(needle)==1
  f.write_text(s.replace(needle,"'BROKEN_"+key+'=%s'))
  proc=subprocess.Popen(['bash',str(dest/'test_needs_postgres.test.sh')],stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,start_new_session=True)
  try:out,_=proc.communicate(timeout=30)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.communicate();raise RuntimeError('inconclusive timeout')
  expected='credential fixture: missing required key '+key
  print(key,'exit',proc.returncode,'expected_guard',expected in out,flush=True)
  assert proc.returncode==66 and expected in out,out
print('PASS: four exact-key content guards reject their independently malformed harness output')
