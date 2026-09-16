#!/usr/bin/env python3
"""Static packet custody/constraint census and finite reference reasoning; no SQL execution."""
import argparse,csv,hashlib,json,re,subprocess
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);args=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*a):return subprocess.check_output(['git',*a],cwd=root)
sha=git('rev-parse',args.commit).decode().strip()
def blob(c,path):
 row=git('ls-tree',c,'--',path).decode().split();assert row[:2]==['100644','blob'],path
 return row[2],git('cat-file','blob',row[2])
names=['entry.txt','source-reference.sql.txt','reference-map.json','source-identities.json','contract.txt','options.txt','input-storage-flow.csv','proof-cases.csv','scope.json','readiness.json','review.html','verify-packet.py'];b={}
for name in names:
 _,raw=blob(sha,rel+'/'+name);assert raw==(p/name).read_bytes(),name;b[name]=raw
j=lambda name:json.loads(b[name]);scope=j('scope.json');ready=j('readiness.json')
for name in ['whole_design_approved','implementation_authorized','w04_complete']:assert scope[name] is False and ready[name] is False
assert scope['product_tests_discovered']==scope['product_tests_executed']==scope['product_tests_written']==scope['verus_proofs_executed']==0 and scope['sql_executed'] is False
sources=j('source-identities.json')['sources'];assert len({(x['commit'],x['path']) for x in sources})==len(sources)
for x in sources:
 oid,raw=blob(x['commit'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(raw).hexdigest()==x['sha256']
sql=b['source-reference.sql.txt'].decode();counts={'new_tables':len(re.findall(r'^CREATE TABLE ',sql,re.M)),'foreign_keys':sql.count(' FOREIGN KEY('),'unique_targets':sql.count(' UNIQUE('),'generated_columns':sql.count('GENERATED ALWAYS AS')};assert counts==scope['structural_counts']=={'new_tables':0,'foreign_keys':35,'unique_targets':21,'generated_columns':44}
constraints=re.findall(r'ADD CONSTRAINT (\w+)',sql);assert len(constraints)==len(set(constraints)) and all(len(x)<=63 for x in constraints)
assert sql.count('MATCH SIMPLE ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED')==35
families=j('reference-map.json')['families'];assert [r['family_code'] for r in families]==list(range(1,8))
for r in families:
 k=r['family_code'];t=r['native_table'];rc=r['root_column']
 assert f'revision_root_{k} uuid GENERATED ALWAYS AS (CASE WHEN family_code={k} AND effect_kind=1 THEN root_id END) STORED' in sql
 assert f'REFERENCES public.{t}(org_id,{rc},revision,command_id,custody_digest)' in sql
 assert f'sr23_s{k}_effect FOREIGN KEY(org_id,effect_family,{rc},revision,effect_kind,command_id,custody_digest)' in sql
 if k>=5:
  assert f'sr23_e{k}_revoked FOREIGN KEY(org_id,claim_root_{k},revision,claim_revoke_kind,custody_digest)' in sql
  assert f'sr23_e{k}_verified FOREIGN KEY(org_id,claim_successor_root_{k},successor_revision,claim_verify_kind,command_id,successor_custody_digest)' in sql
  assert f'sr23_e{k}_head FOREIGN KEY(org_id,claim_root_{k},revision,command_id)' in sql
# Finite selected branch reasoning, not evaluation of generated PostgreSQL expressions.
branches=0
for family in range(1,8):
 for kind in (1,2):
  if kind==2 and family<5:continue
  active=[k for k in range(1,8) if k==family and kind==1] if kind==1 else [k for k in range(5,8) if k==family]
  assert active==[family];branches+=1
assert branches==10
# Independently stated five paths; exact prior-kind and admission identity are SQL obligations above.
paths=[['PROPOSE@1'],['PROPOSE@1','VERIFY@2'],['PROPOSE@1','REVOKE@2'],['PROPOSE@1','VERIFY@2','REVOKE@3'],['PROPOSE@1','VERIFY@2','SUPERSEDE@3']]
assert j('reference-map.json')['source19_event_paths']==paths
assert [v for v in paths if v[-1]=='REVOKE@3']==[['PROPOSE@1','VERIFY@2','REVOKE@3']]
for name,count in [('input-storage-flow.csv',11),('proof-cases.csv',20)]:
 rows=list(csv.DictReader(b[name].decode().splitlines()));assert len(rows)==count and len({r['id'] for r in rows})==count
 if name=='proof-cases.csv':assert all(r['status']=='PROPOSED_NOT_EXECUTED' for r in rows)
assert scope['screen_cases']==11 and scope['nonexecuted_cases']==20
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in names for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(names),'source_blobs':len(sources),'structural_counts':counts,'selected_reference_branches':branches,'admission_paths':len(paths),'screen_cases':11,'nonexecuted_cases':20,'reader_links':len(links),'validation_scope':'Static regular-blob/source custody; SQL token/reference census; finite conceptual branches only. No SQL parse/typecheck/catalog/runtime, performance or Verus proof.','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'implementation_authorized':False},indent=2))
