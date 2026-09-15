import json, pathlib, subprocess, time, uuid
name='console-fk-owner-selfgrant-'+uuid.uuid4().hex
label='console.fk-owner-probe='+name
image='postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'
record={'container':name,'image':image,'network':'none','storage':'256MiB tmpfs; no durability claim','cases':[]}
def docker(*args, timeout=30):
 return subprocess.run(['docker',*args],capture_output=True,text=True,timeout=timeout)
def sql(query):
 p=docker('exec','-i',name,'psql','-X','-U','postgres','-d','postgres','-v','ON_ERROR_STOP=1','-Atc',query)
 return {'exit':p.returncode,'stdout':p.stdout.strip(),'stderr':p.stderr.strip()}
started=False
try:
 p=docker('run','-d','--name',name,'--label',label,'--network','none','--tmpfs','/var/lib/postgresql:rw,size=256m','-e','POSTGRES_HOST_AUTH_METHOD=trust',image)
 assert p.returncode==0,p.stderr
 started=True
 for _ in range(100):
  p=docker('exec',name,'pg_isready','-U','postgres',timeout=5)
  if p.returncode==0:break
  time.sleep(.2)
 else:raise AssertionError('owned PostgreSQL not ready')
 setup=sql("CREATE ROLE fixture_owner NOLOGIN NOINHERIT; CREATE TABLE public.accounts(id uuid PRIMARY KEY, note text); CREATE TABLE public.account_security(account_id uuid PRIMARY KEY REFERENCES public.accounts(id)); ALTER TABLE public.accounts OWNER TO fixture_owner; ALTER TABLE public.account_security OWNER TO fixture_owner; REVOKE ALL ON public.accounts,public.account_security FROM fixture_owner; GRANT SELECT ON public.accounts,public.account_security TO fixture_owner; INSERT INTO public.accounts VALUES('10000000-0000-0000-0000-000000000001','retained');")
 assert setup['exit']==0,setup
 insert="INSERT INTO public.account_security VALUES('10000000-0000-0000-0000-000000000001');"
 for title,grant in [('owner SELECT only',None),('owner REFERENCES too','GRANT REFERENCES ON public.accounts TO fixture_owner;'),('owner UPDATE(id) instead of REFERENCES','REVOKE REFERENCES ON public.accounts FROM fixture_owner; SET ROLE fixture_owner; GRANT UPDATE(id) ON public.accounts TO fixture_owner; RESET ROLE;')]:
  if grant:
   p=sql(grant);assert p['exit']==0,p
  record['cases'].append({'case':title,**sql(insert)})
 record['acl']=sql("SELECT c.relname,c.relacl,a.attname,a.attacl FROM pg_class c JOIN pg_attribute a ON a.attrelid=c.oid WHERE c.oid='public.accounts'::regclass AND a.attnum>0 ORDER BY a.attnum;")
 assert [c['exit'] for c in record['cases']]==[1,1,0],record
 record['result']='PASS: FK owner requires row-lock privilege; exact UPDATE(id) suffices'
finally:
 if started:
  p=docker('inspect','--format','{{index .Config.Labels "console.fk-owner-probe"}}',name)
  assert p.returncode==0 and p.stdout.strip()==name,'owned label mismatch; refusing removal'
  p=docker('rm','-f',name);assert p.returncode==0,p.stderr
  p=docker('ps','-a','--format','{{.Names}}');assert p.returncode==0 and name not in p.stdout.splitlines()
  record['cleanup']='exact label checked; owned container removed; successful absence readback'
 pathlib.Path('/private/tmp/fk-owner-selfgrant-probe-result.json').write_text(json.dumps(record,indent=2)+'\n')
 print(json.dumps(record,indent=2))
