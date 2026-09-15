#!/usr/bin/env python3
"""Static custody/DDL census/reference topology only; no SQL or product execution."""
import argparse,csv,hashlib,json,re,subprocess
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);a=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*a):return subprocess.check_output(['git',*a],cwd=root)
sha=git('rev-parse',a.commit).decode().strip()
def blob(c,path):
 row=git('ls-tree',c,'--',path).decode().split();assert row[:2]==['100644','blob'],path
 return row[2],git('cat-file','blob',row[2])
names=['entry.txt','binding-adoption.sql.txt','contract.txt','choices.txt','proof-cases.csv','lifecycle.csv','source-identities.json','scope.json','readiness.json','review.html','verify-packet.py'];b={}
for n in names:
 _,raw=blob(sha,rel+'/'+n);assert raw==(p/n).read_bytes(),n;b[n]=raw
j=lambda n:json.loads(b[n]);scope=j('scope.json');ready=j('readiness.json')
for k in ['whole_design_approved','implementation_authorized','w04_complete']:assert scope[k] is False and ready[k] is False
assert scope['product_tests_written']==scope['product_tests_discovered']==scope['product_tests_executed']==scope['verus_proofs_executed']==0 and scope['sql_executed'] is False
sources=j('source-identities.json')['sources'];assert len(sources)==scope['source_blobs']==15
assert len({(x['commit'],x['path']) for x in sources})==len(sources);sourcebytes={}
for x in sources:
 oid,raw=blob(x['commit'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(raw).hexdigest()==x['sha256'];sourcebytes[x['path']]=raw
sql=b['binding-adoption.sql.txt'].decode();counts={'new_tables':len(re.findall(r'^CREATE TABLE ',sql,re.M)),'functions':len(re.findall(r'^CREATE FUNCTION ',sql,re.M)),'triggers':len(re.findall(r'^CREATE (?:CONSTRAINT )?TRIGGER ',sql,re.M)),'foreign_keys':sql.count('FOREIGN KEY(')}
assert counts==scope['counts']=={'new_tables':1,'functions':3,'triggers':3,'foreign_keys':9}
assert 'REFERENCES public.ont_source_submissions' not in sql
assert 'FROM public.ont_source_submissions s WHERE s.org_id=NEW.org_id AND s.unit_id=NEW.unit_id' in sql
assert 'WHEN(OLD.snapshot_digest IS NULL AND NEW.snapshot_digest IS NOT NULL)' in sql
assert "IF NOT FOUND OR v_protocol IS DISTINCT FROM 'SOURCE_EFFECTS21'" in sql
assert scope['nonexecuted_cases']==21
assert 'LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog SET row_security=on' in sql
assert 'adoption25_effect_adoption FOREIGN KEY(org_id,command_id,ordinal,submitted_intent_slot,submission_unit_id,submission_snapshot_digest)' in sql
assert 'adoption25_adoption_effect FOREIGN KEY(org_id,source_command_id,source_effect_ordinal,snapshot_unit_id,source_snapshot_digest)' in sql
assert 'adoption25_payload_binding FOREIGN KEY(org_id,unit_id,attempt_id,command_id,actor_account_id,command_fingerprint,source_input_digest)' in sql
# Named declaration census, not SQL parsing/typechecking or semantic enforcement proof.
identifiers=re.findall(r'(?:ADD CONSTRAINT|CREATE CONSTRAINT TRIGGER|CREATE TRIGGER) (\w+)',sql);assert all(len(x)<=63 for x in identifiers)
# Pin24 capacity arithmetic and distinguish logical tuple completeness from real DB guards.
vpath='docs/evidence/console/design/2026-09-13-submission-custody-contract-24/framing-vectors.json';v=json.loads(sourcebytes[vpath]);overhead=len(b'console.source24.submission')+1+15
assert overhead==43 and overhead+16384+65536+65536==v['capacity_arithmetic']['maximum_frame_bytes']==147499
assert overhead+3==46
assert [m for m in range(16) if all(bool(m&(1<<i)) for i in range(4)) or not any(m&(1<<i) for i in range(4))]==[0,15]
for n,count in [('proof-cases.csv',21),('lifecycle.csv',7)]:
 rows=list(csv.DictReader(b[n].decode().splitlines()));assert len(rows)==count
 if n=='proof-cases.csv':assert len({r['id'] for r in rows})==count and all(r['status']=='PROPOSED_NOT_EXECUTED' for r in rows)
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in names for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(names),'source_blobs':len(sources),'counts':counts,'frame_overhead_bytes':overhead,'maximum_frame_bytes':147499,'tuple_presence_masks':16,'nonexecuted_cases':21,'lifecycle_rows':7,'reader_links':len(links),'validation_scope':'Static regular-blob/reference/SQL token census and tuple/frame arithmetic only; no SQL parse/typecheck/catalog/runtime, semantic lifecycle proof or Verus.','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'implementation_authorized':False},indent=2))
