#!/usr/bin/env python3
"""Static identities/SQL declaration census/framing/arithmetic only; no SQL execution."""
import argparse,csv,json,hashlib,subprocess,uuid,re
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);a=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*x):return subprocess.check_output(['git',*x],cwd=root)
sha=git('rev-parse',a.commit).decode().strip()
def blob(c,path):
 row=git('ls-tree',c,'--',path).decode().split();assert row[:2]==['100644','blob'],path
 return row[2],git('show',c+':'+path)
files=['entry.txt','effect-manifest.sql.txt','transaction-contract.txt','implementation-closure-map.json','proof-cases.csv','scope.json','readiness.json','source-identities.json','review.html','verify-packet.py'];b={}
for n in files:
 _,raw=blob(sha,rel+'/'+n);assert raw==(p/n).read_bytes(),n;b[n]=raw
j=lambda n:json.loads(b[n]);scope=j('scope.json');ready=j('readiness.json')
for k in ['implementation_authorized','whole_design_approved','w04_complete']:assert scope[k] is False and ready[k] is False
assert scope['product_tests_written']==scope['product_tests_executed']==scope['verus_proofs_executed']==0 and scope['sql_executed'] is False
sources=j('source-identities.json')['sources'];sourcebytes={}
assert len({(x['commit'],x['path']) for x in sources})==len(sources)
for x in sources:
 oid,raw=blob(x['commit'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(raw).hexdigest()==x['sha256'];sourcebytes[x['path']]=raw
sql=b['effect-manifest.sql.txt'].decode()
# Declaration/token census does not parse/type-check SQL and is not runtime enforcement proof.
assert len(re.findall(r'^CREATE TABLE ',sql,re.M))==scope['sql_tables_proposed']==1
assert len(re.findall(r'^CREATE FUNCTION ',sql,re.M))==scope['sql_functions_proposed']==4
assert len(re.findall(r'^CREATE (?:CONSTRAINT )?TRIGGER ',sql,re.M))==scope['sql_triggers_proposed']==4
assert sql.count('LANGUAGE plpgsql SECURITY INVOKER')==3 and sql.count('LANGUAGE plpgsql SECURITY DEFINER')==1
assert 'ON DELETE RESTRICT NOT DEFERRABLE' in sql
assert 'AFTER INSERT ON public.ont_action_command_receipts DEFERRABLE INITIALLY DEFERRED' in sql
assert 'NEW.ordinal>=v_count' in sql and 'v_order_ok IS NOT TRUE' in sql and 'LIMIT 1025' in sql
assert "string_agg(member_bytes,''::bytea ORDER BY ordinal)" in sql
assert 'SECURITY DEFINER SET search_path=pg_catalog SET row_security=on' in sql
# Independently rebuild fixed SQL frame shape from pinned21 arithmetic-only witnesses.
vpath='docs/evidence/console/design/2026-09-12-source-persistence-contract-21/membership-witnesses.json'
vectors=json.loads(sourcebytes[vpath])['vectors'];width=2+16+8+32+57;assert width==115
for v in vectors:
 raw=b'console.source21.effects'+bytes([0])+uuid.UUID(v['org_id']).bytes+uuid.UUID(v['command_id']).bytes+len(v['members']).to_bytes(4,'big')
 for m in v['members']:
  piece=bytes([m['family_code'],m['effect_kind']])+uuid.UUID(m['root_id']).bytes+int(m['revision']).to_bytes(8,'big')+bytes.fromhex(m['custody_digest'])
  piece+=bytes([1])+uuid.UUID(m['successor_root_id']).bytes+int(m['successor_revision']).to_bytes(8,'big')+bytes.fromhex(m['successor_custody_digest']) if m['effect_kind']==2 else bytes(57)
  assert len(piece)==width;raw+=piece
 assert raw.hex()==v['frame_hex'] and len(raw)==v['frame_bytes'] and hashlib.sha256(raw).hexdigest()==v['sha256']
# Finite ordinal saturation arithmetic, not a proof of actual database concurrency.
for count in range(1,1025):
 permitted=set(range(count));occupied=set(range(count));assert len(occupied)==count and not permitted-occupied
 assert -1 not in permitted and count not in permitted
rows=list(csv.DictReader(b['proof-cases.csv'].decode().splitlines()));assert len(rows)==scope['nonexecuted_cases']==19 and len({r['id'] for r in rows})==19
assert all(r['status']=='PROPOSED_NOT_EXECUTED' for r in rows)
closure=j('implementation-closure-map.json');assert [x['id'] for x in closure['items']]==['F0'+str(i) for i in range(1,9)]
assert [x['id'] for x in closure['phases_before_implementation']]==[1,2,3,4] and closure['current_phase']==1
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in files for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(files),'source_blobs':len(sources),'sql_tables':1,'sql_functions':4,'sql_triggers':4,'framing_witnesses':len(vectors),'ordinal_cardinalities_checked':1024,'nonexecuted_cases':len(rows),'closure_items':len(closure['items']),'reader_links':len(links),'validation_scope':'Static custody/token census/framing/saturation arithmetic only; not SQL parsing/typecheck/runtime or Verus proof','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'implementation_authorized':False},indent=2))
