from pathlib import Path
import subprocess,tempfile,shutil,os,signal,sys
root=Path.cwd()
def run_case(bad):
 with tempfile.TemporaryDirectory(prefix="console-buck-credential-oracle-") as d:
  work=Path(d);dest=work/'tools/buck';dest.mkdir(parents=True)
  for name in ['test_needs_postgres.sh','test_needs_postgres.test.sh']:
   shutil.copy2(root/'tools/buck'/name,dest/name)
  if bad:
   f=dest/'test_needs_postgres.sh';s=f.read_text();needle='chmod 600 "${test_env_file}"';assert s.count(needle)==1
   f.write_text(s.replace(needle,'chmod 644 "${test_env_file}"'))
  proc=subprocess.Popen(['bash',str(dest/'test_needs_postgres.test.sh')],cwd=work,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True,start_new_session=True)
  try: output,_=proc.communicate(timeout=120)
  except subprocess.TimeoutExpired:
   os.killpg(proc.pid,signal.SIGKILL);output,_=proc.communicate();raise RuntimeError('fixture timeout; not a valid oracle result')
  return proc.returncode,output
ok,good=run_case(False);print('CONTROL_EXIT',ok,flush=True)
assert ok==0 and 'test_needs_postgres: PASS' in good,good
bad,out=run_case(True);print('MODE644_MUTANT_EXIT',bad,flush=True);print(out,flush=True)
assert bad!=0,'checker accepted mode644 credential file; existing permission assertion did not enforce'
assert 'credential fixture: expected mode0600 file' in out,'mutant failed outside declared mode guard'
print('PASS: original harness accepted; actual mode644 mutant rejected at credential-file guard')
