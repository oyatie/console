#!/usr/bin/env python3
"""Static exact-blob and framing checks only; not product, SQL, or full JSON Schema execution."""
import argparse,csv,json,hashlib,re,subprocess
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);a=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*a):return subprocess.check_output(['git',*a],cwd=root)
sha=git('rev-parse',a.commit).decode().strip()
def blob(c,path):
 row=git('ls-tree',c,'--',path).decode().split();assert row[:2]==['100644','blob'],path
 return row[2],git('cat-file','blob',row[2])
names=['entry.txt','submission-types.schema.json','storage.sql.txt','contract.txt','framing-vectors.json','source-identities.json','proof-cases.csv','readiness.json','scope.json','review.html','verify-packet.py'];b={}
for n in names:
 _,raw=blob(sha,rel+'/'+n);assert raw==(p/n).read_bytes(),n;b[n]=raw
j=lambda n:json.loads(b[n]);scope=j('scope.json');ready=j('readiness.json')
for k in ['whole_design_approved','implementation_authorized','w04_complete']:assert scope[k] is False and ready[k] is False
assert scope['product_tests_written']==scope['product_tests_discovered']==scope['product_tests_executed']==scope['verus_proofs_executed']==0 and scope['sql_executed'] is False
sources=j('source-identities.json')['sources'];assert len(sources)==scope['source_blobs']==16
assert len({(x['commit'],x['path']) for x in sources})==len(sources)
for x in sources:
 oid,raw=blob(x['commit'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(raw).hexdigest()==x['sha256']
canon=lambda x:json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
h=lambda x:hashlib.sha256(x).hexdigest();tag=lambda t,b:t.encode()+bytes([0])+b
spec=j('submission-types.schema.json');defs=spec['$defs']
for d in defs.values():assert d['additionalProperties'] is False and set(d['properties'])==set(d['required'])
# Structural ref resolution only; no claim of full JSON Schema validation.
def refs(x):
 if isinstance(x,dict):
  if '$ref' in x:yield x['$ref']
  for v in x.values():yield from refs(v)
 elif isinstance(x,list):
  for v in x:yield from refs(v)
for r in refs(spec):
 file,frag=r.split('#',1);target=spec if not file else json.loads((p/file).read_text())
 for seg in frag.strip('/').split('/'):target=target[seg]
vec=j('framing-vectors.json');vs=vec['vectors'];assert len(vs)==3
for v in vs:
 for key,definition in [('preparation','Preparation24'),('metadata','SubmissionMetadata24')]:assert set(v[key])==set(defs[definition]['required'])
 assert set(v['command']['body'])==set(defs['SourcePayloadRef24']['required'])
 ib=canon(v['input']);cb=canon(v['command']);mb=canon(v['metadata']);prep=v['preparation'];ref=v['command']['body'];si=prep['source']
 assert chr(10) in v['command']['reason'] and chr(92)+'n' not in v['command']['reason']
 assert h(tag('console.source24.input',ib))==si['source_input_digest']
 assert h(tag('console.source24.preparation',canon(prep)))==v['preparation_digest']==ref['preparation_digest']
 assert ref['source']==si and ref['preparation_unit_id']==prep['unit_id']
 assert prep['target_binding_digest']==h(tag('console.source24.target',canon(v['target']))) and v['target']==v['command']['target']
 assert prep['custody_binding_digest']==h(tag('console.source24.custody',canon(v['custody'])))
 assert h(tag('console.command.ccf1',cb))==v['command_fingerprint']==v['metadata']['command_fingerprint']
 assert v['metadata']['preparation']==prep and v['metadata']['command_id']==v['command']['command_id']
 assert v['metadata']['attempt_id']==v['command']['attempt']['attempt_id']
 comps=[mb,cb,ib];assert [len(x) for x in comps]==v['component_bytes'] and [h(x) for x in comps]==v['component_raw_sha256']
 assert all(1<=len(x)<=cap for x,cap in zip(comps,[16384,65536,65536]))
 frame=tag('console.source24.submission',b'')+b''.join(bytes([i])+len(x).to_bytes(4,'big')+x for i,x in enumerate(comps,1))
 assert frame.hex()==v['frame_hex'] and len(frame)==v['frame_bytes'] and h(frame)==v['snapshot_digest']
assert vs[0]['preparation_digest']==vs[1]['preparation_digest'] and vs[0]['command_fingerprint']!=vs[1]['command_fingerprint']
assert vs[0]['preparation_digest']!=vs[2]['preparation_digest'] and len({v['snapshot_digest'] for v in vs})==3
cap=vec['capacity_arithmetic'];maximum=len(tag('console.source24.submission',b''))+3*5+16384+65536+65536
assert maximum==cap['maximum_frame_bytes']==scope['maximum_frame_bytes']==147499
sql=b['storage.sql.txt'].decode();assert len(re.findall(r'^CREATE TABLE ',sql,re.M))==scope['new_tables_proposed']==1
assert 'FOREIGN KEY(org_id,actor_account_id) REFERENCES public.company_actors(org_id,account_id)' in sql
assert 'REFERENCES public.ont_draft_submission_attempts(org_id,attempt_id,domain_command_id,original_account_id,command_fingerprint)' in sql
assert sql.count('FOREIGN KEY(')==4 and sql.count('pg_catalog.sha256(')==3
rows=list(csv.DictReader(b['proof-cases.csv'].decode().splitlines()));assert len(rows)==scope['nonexecuted_cases']==18 and len({r['id'] for r in rows})==18
assert all(r['status']=='PROPOSED_NOT_EXECUTED' for r in rows)
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in names for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(names),'source_blobs':len(sources),'synthetic_frames':3,'schema_reference_resolution':'PASS; not full schema validation','maximum_frame_bytes':maximum,'new_tables_proposed':1,'foreign_keys':4,'nonexecuted_cases':18,'reader_links':len(links),'validation_scope':'Static custody/ref/key/framing arithmetic only. No SQL parse/typecheck/runtime, full JSON Schema validator, product tests or Verus proof.','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'implementation_authorized':False},indent=2))
