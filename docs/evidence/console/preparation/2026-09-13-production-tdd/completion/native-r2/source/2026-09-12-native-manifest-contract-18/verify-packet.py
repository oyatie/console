#!/usr/bin/env python3
"""Static evidence/identity/canonical-vector checks; no product or SQL tests."""
import argparse,csv,hashlib,json,re,subprocess
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);a=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*args):return subprocess.check_output(['git',*args],cwd=root)
sha=git('rev-parse',a.commit).decode().strip()
def blob(commit,path):
 row=git('ls-tree',commit,'--',path).decode().split();assert row[:2]==['100644','blob'],(commit,path)
 return row[2],git('show',commit+':'+path)
files=['entry.txt','native-contract.txt','native-types.schema.json','source-slot-cardinality.json','physical-contract.sql.txt','owner-and-migration.txt','scalar-and-encoding.txt','normalization-vectors.json','proof-cases.csv','readiness.json','scope.json','source-identities.json','review.html','verify-packet.py']
b={}
for name in files:
 _,v=blob(sha,rel+'/'+name);assert v==(p/name).read_bytes(),name;b[name]=v
j=lambda name:json.loads(b[name])
s=j('scope.json');rd=j('readiness.json')
for k in ('w04_complete','whole_design_approved','implementation_authorized'):assert s[k] is False and rd[k] is False
assert s['product_tests_executed']==s['product_tests_written']==0
src=j('source-identities.json')['sources'];assert len(src)==s['source_blobs']==40
assert len({(x['commit'],x['path']) for x in src})==40
for x in src:
 oid,v=blob(x['commit'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(v).hexdigest()==x['sha256']
schema=j('native-types.schema.json');defs=schema['$defs'];assert len(defs)==s['schema_definitions']==43
refs=[];objects=[]
def visit(x):
 if isinstance(x,dict):
  if '$ref' in x:refs.append(x['$ref'])
  if x.get('type')=='object':objects.append(x);assert x.get('additionalProperties') is False;assert set(x['properties'])==set(x['required'])
  for v in x.values():visit(v)
 elif isinstance(x,list):
  for v in x:visit(v)
visit(schema)
assert all(x.startswith('#/$defs/') and x.split('/')[-1] in defs for x in refs)
# Detect recursive/unbounded reference definitions, not just spelling closure.
def children(x):
 if isinstance(x,dict):
  if '$ref' in x:yield x['$ref'].split('/')[-1]
  for v in x.values():yield from children(v)
 elif isinstance(x,list):
  for v in x:yield from children(v)
def acyclic(n,stack):
 assert n not in stack,(n,stack)
 for x in children(defs[n]):acyclic(x,stack+(n,))
for n in defs:acyclic(n,())
v=j('normalization-vectors.json');assert len(v['vectors'])+3==18 and len(v['invalid_raw_fragment_oracles'])==10
canon=lambda x:json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
vm={x['id']:x for x in v['vectors']}
for x in vm.values():
 raw=canon(x['value']);assert raw.hex()==x['canonical_utf8_hex'];assert len(raw)<=65536
 assert hashlib.sha256(x['tag'].encode()+bytes([0])+raw).hexdigest()==x['sha256']
for field,tag,members,key in [('manifest_frame','input-manifest','units','custody_vector'),('resolved_input_frame','input-manifest','units','custody_vector'),('batch_frame','calculation-batch','outcomes','outcome_vector')]:
 f=v[field];raw=('console.payroll.native18.'+tag).encode()+bytes([0])+bytes.fromhex(vm[f['header_vector']]['sha256'])+len(f[members]).to_bytes(4,'big')
 assert f[members]==sorted(f[members],key=lambda x:x['employee_id'])
 for x in f[members]:raw+=bytes.fromhex(x['employee_id'].replace('-',''))+bytes.fromhex(vm[x[key]]['sha256'])
 assert raw.hex()==f['raw_hex'] and hashlib.sha256(raw).hexdigest()==f['sha256']
assert vm['V16']['value']['input_unit_digest']==vm['V13']['sha256'] and vm['V16']['value']['input_unit_digest']!=vm['V12']['sha256']
assert vm['V17']['value']['input_manifest_digest']==v['resolved_input_frame']['sha256']
assert vm['V06']['value']['semantic_digest']==vm['V07']['value']['semantic_digest']==vm['V04']['sha256']
assert vm['V06']['sha256']!=vm['V07']['sha256'] and vm['V08']['sha256']!=vm['V09']['sha256']
card=j('source-slot-cardinality.json');assert len(card['kinds'])==6 and len(card['rules'])==12
sql=b['physical-contract.sql.txt'].decode();tables=re.findall(r'CREATE TABLE public\.([a-z_]+) \(',sql)
assert len(tables)==len(set(tables))==7 and 'payroll_calculation_units' in tables
assert 'calculation_revision integer NOT NULL' in sql and 'NULLIF(reference_revision,0)' in sql
assert 'line_id,input_unit_digest) REFERENCES public.payroll_input_units(org_id,run_id,input_revision,employee_id,line_id,custody_digest)' in sql
assert 'DEFERRABLE INITIALLY DEFERRED' in sql and 'native18_result_outcome_fk' in sql
proof=list(csv.DictReader(b['proof-cases.csv'].decode().splitlines()));assert len(proof)==30 and len({x['id'] for x in proof})==30
assert all(x['status']=='PROPOSED_NOT_EXECUTED' for x in proof)
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert len(links)==12 and all(x in files for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(files),'source_blobs':len(src),'closed_shape_definitions':len(defs),'object_shapes':len(objects),'reference_links_resolved':len(refs),'native_table_declarations':len(tables),'canonical_reference_vectors':18,'invalid_fragment_oracles_not_executed':10,'nonexecutable_proof_cases':30,'reader_links':len(links),'w04_complete':False,'product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False},indent=2))
