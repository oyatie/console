from pathlib import Path
import os,secrets,subprocess,json,time,hashlib
root=Path(__file__).resolve().parent
image='postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'
container='console-account-lifecycle-experiment-'+secrets.token_hex(6)
env=dict(os.environ,DOCKER_CONTEXT='colima-console-design30')
values={'POSTGRES_HOST':'127.0.0.1','POSTGRES_PORT':'5432','POSTGRES_DB':'postgres','POSTGRES_ADMIN_USER':'experiment_admin','POSTGRES_USER':'experiment_admin'}
for k in ['POSTGRES_ADMIN_PASSWORD','CONSOLE_APP_POSTGRES_PASSWORD','CONSOLE_RT_POSTGRES_PASSWORD','CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD','CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD','CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD']: values[k]=secrets.token_hex(32)
values['POSTGRES_PASSWORD']=values['POSTGRES_ADMIN_PASSWORD']
secret=root/'secrets.env';fd=os.open(secret,os.O_WRONLY|os.O_CREAT|os.O_TRUNC,0o600)
with os.fdopen(fd,'w') as f:f.write(''.join(k+'='+v+'\n' for k,v in values.items()))
commands=[]
def run(args,log):
 with (root/log).open('wb') as f:r=subprocess.run(args,env=env,stdout=f,stderr=subprocess.STDOUT)
 commands.append({'argv':args,'exit_code':r.returncode,'log':log});return r.returncode
try:
 assert run(['docker','image','inspect',image],'image.log')==0
 assert run(['docker','run','-d','--rm','--name',container,'--env-file',str(secret),image],'container.log')==0
 for i in range(30):
  r=subprocess.run(['docker','exec',container,'pg_isready','-U','experiment_admin','-d','postgres'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
  if r.returncode==0:break
  time.sleep(1)
 else:raise RuntimeError('postgres not ready')
 assert run(['docker','cp',str(root),container+':/experiment'],'copy.log')==0
 code=run(['docker','exec',container,'bash','/experiment/in-container.sh'],'experiment.log')
 for name in ['app-owner-refusal.log','transfer-fail.log','legacy-before.json','legacy-after.json','custody-before.json','custody-after.json']:
  run(['docker','cp',container+':/experiment/'+name,str(root/name)],'copy-'+name+'.log')
 status='PASS' if code==0 else 'FAIL'
finally:
 cleanup=run(['docker','rm','-f',container],'cleanup.log')
 secret.unlink(missing_ok=True)
 record={'classification':'SQL_FILES_DISPOSABLE_EXPERIMENT_NOT_SQLX_OR_APP_STARTUP_PROOF','status':locals().get('status','FAIL'),'image':image,'owned_container':container,'cleanup_exit':cleanup,'secret_file_removed':not secret.exists(),'commands':commands,'sources':json.loads((root/'sources.json').read_text()),'artifacts':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in root.iterdir() if p.is_file() and p.name not in ['result.json']}}
 (root/'result.json').write_text(json.dumps(record,indent=2)+'\n')
 print(record['status'],container,'cleanup',cleanup)
