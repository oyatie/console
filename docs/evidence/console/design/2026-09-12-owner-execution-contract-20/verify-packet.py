#!/usr/bin/env python3
"""Static packet custody/reference/fixture shape checks; no product or SQL execution."""
import argparse,csv,hashlib,json,re,subprocess,datetime
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);args=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*a):return subprocess.check_output(['git',*a],cwd=root)
sha=git('rev-parse',args.commit).decode().strip()
def blob(c,path):
 row=git('ls-tree',c,'--',path).decode().split();assert row[0] in ('100644','100755') and row[1]=='blob',(c,path)
 return row[2],git('show',c+':'+path)
files=['entry.txt','execution-contract.txt','locking-and-constraints.txt','payload-and-adoption.txt','privilege-and-enrollment.txt','owner-interfaces.json','caller-census.csv','constraint-owner-matrix.csv','readiness.json','guard-and-owner-schema.sql.txt','source-identities.json','input-reference.schema.json','boundary-witnesses.json','proof-cases.csv','scope.json','review.html','verification-strategy.txt','verus-sources.json','rust-pin-receipt.json','verify-packet.py']
b={}
for n in files:
 _,raw=blob(sha,rel+'/'+n);assert raw==(p/n).read_bytes(),n;b[n]=raw
j=lambda n:json.loads(b[n]);scope=j('scope.json');ready=j('readiness.json')
for k in ('w04_complete','whole_design_approved','implementation_authorized'):assert scope[k] is False and ready[k] is False
assert scope['product_tests_written']==scope['product_tests_executed']==scope['verus_proofs_executed']==0
assert scope['sql_executed'] is False
sources=j('source-identities.json')['sources'];assert len(sources)==scope['source_blobs'];assert len({(x['commit'],x['path']) for x in sources})==len(sources)
source_bytes={}
for s in sources:
 oid,raw=blob(s['commit'],s['path']);assert oid==s['git_blob'] and hashlib.sha256(raw).hexdigest()==s['sha256'];source_bytes[s['path']]=raw
external_path='docs/evidence/console/design/2026-09-12-source-payload-contract-19/source-types.schema.json'
external=json.loads(source_bytes[external_path]);schema=j('input-reference.schema.json');refs=[];objects=[]
def normalize(x,external_context=False):
 if isinstance(x,dict):
  out={}
  for k,v in x.items():
   if k=='$ref':
    refs.append(v)
    if v.startswith('#/$defs/'):out[k]='#/$defs/'+('source19_' if external_context else '')+v[8:]
    else:
     assert v.startswith('../2026-09-12-source-payload-contract-19/source-types.schema.json#/$defs/')
     out[k]='#/$defs/source19_'+v.split('#/$defs/')[1]
   else:out[k]=normalize(v,external_context)
  if x.get('type')=='object':
   objects.append(x);assert x.get('additionalProperties') is False and set(x['required'])==set(x['properties'])
  return out
 if isinstance(x,list):return [normalize(v,external_context) for v in x]
 return x
local=normalize(schema);ext=normalize(external,True)
defs=dict(local['$defs']);defs.update({'source19_'+k:v for k,v in ext['$defs'].items()})
def children(x):
 if isinstance(x,dict):
  if '$ref' in x:yield x['$ref'][8:]
  for v in x.values():yield from children(v)
 elif isinstance(x,list):
  for v in x:yield from children(v)
def acyclic(n,stack):
 assert n in defs and n not in stack,(n,stack)
 for c in children(defs[n]):acyclic(c,stack+(n,))
for n in defs:acyclic(n,())
canon=lambda x:json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def depth(x):
 if isinstance(x,dict):return 1+max((depth(v) for v in x.values()),default=0)
 if isinstance(x,list):return 1+max((depth(v) for v in x),default=0)
 assert x is None or isinstance(x,(str,bool));return 0
# Limited static fixture shape checker, not a full JSON Schema implementation.
def shape(value,s):
 allowed={'$ref','oneOf','anyOf','type','properties','required','additionalProperties','items','minItems','maxItems','minLength','maxLength','pattern','const','enum','not'}
 assert set(s)<=allowed,('unsupported fixture schema vocabulary',set(s)-allowed)
 if '$ref' in s:return shape(value,defs[s['$ref'][8:]])
 for union in ('oneOf','anyOf'):
  if union in s:
   matches=0
   for branch in s[union]:
    try:shape(value,branch);matches+=1
    except AssertionError:pass
   assert matches==1 if union=='oneOf' else matches>=1
 if 'const' in s:assert type(value)==type(s['const']) and value==s['const']
 if 'enum' in s:assert value in s['enum']
 if 'not' in s:
  try:shape(value,s['not'])
  except AssertionError:pass
  else:raise AssertionError('forbidden fixture shape')
 typ=s.get('type')
 if typ=='object':
  assert isinstance(value,dict) and set(value)==set(s['required'])
  for k,x in value.items():shape(x,s['properties'][k])
 elif typ=='array':
  assert isinstance(value,list) and s.get('minItems',0)<=len(value)<=s.get('maxItems',256)
  for x in value:shape(x,s['items'])
 elif typ=='string':
  assert isinstance(value,str) and s.get('minLength',0)<=len(value)<=s.get('maxLength',100000)
  if 'pattern' in s:assert re.search(s['pattern'],value)
 elif typ=='null':assert value is None
 elif typ=='boolean':assert isinstance(value,bool)
 else:assert typ is None

w=j('boundary-witnesses.json')['witnesses'];assert len(w)==3
for x in w:
 raw=canon(x['value']);assert raw.hex()==x['canonical_utf8_hex'] and len(raw)==x['bytes']<=65536
 assert hashlib.sha256(raw).hexdigest()==x['sha256'] and depth(x['value'])<=8
shape(w[0]['value'],defs['CalendarDayPatch20']);shape(w[1]['value'],defs['SourcePayloadRef20'])
assert w[2]['value']['body']==w[0]['value'] # budget-only skeleton; not full command conformance
counts={}
for name,key in [('caller-census.csv','caller_rows'),('constraint-owner-matrix.csv','constraint_rows'),('proof-cases.csv','nonexecutable_proof_cases')]:
 rows=list(csv.DictReader(b[name].decode().splitlines()));assert len(rows)==scope[key] and len({x['id'] for x in rows})==len(rows);counts[key]=len(rows)
 if name=='proof-cases.csv':assert all(x['status']=='PROPOSED_NOT_EXECUTED' for x in rows)
ops=j('owner-interfaces.json')['operations'];assert len(ops)==scope['operations'] and len({o['action'] for o in ops})==len(ops)
vs=j('verus-sources.json');assert re.fullmatch('[0-9a-f]{40}',vs['commit'])
assert len({x['url'] for x in vs['sources']})==len(vs['sources'])
for x in vs['sources']:
 assert vs['commit'] in x['url'] and re.fullmatch('[0-9a-f]{64}',x['sha256']) and x['bytes']>0
# Remote entries are retrieval identities; offline verifier does not claim independent download validation.
rp=j('rust-pin-receipt.json');assert rp['candidate_sha']=='2fbe2684155808a88fbc83636d5d27900e09c931'
for x in rp['files']:
 oid,raw=blob(rp['candidate_sha'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(raw).hexdigest()==x['sha256']
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in files for x in links)
# 366 consecutive dates plus one preceding/following day need at most14 calendar months.
# Exhaust all starts in a Gregorian400-year cycle; serialization arithmetic only.
start=datetime.date(2000,1,1);end=datetime.date(2400,1,1);maximum=0
while start<end:
 first=start-datetime.timedelta(days=1);last=start+datetime.timedelta(days=366)
 months=(last.year-first.year)*12+last.month-first.month+1;maximum=max(maximum,months)
 start+=datetime.timedelta(days=1)
assert maximum==14
print(json.dumps({'candidate_sha':sha,'core_files':len(files),'source_blobs':len(sources),'local_schema_definitions':len(local['$defs']),'operations':len(ops),**counts,'synthetic_witnesses':len(w),'maximum_witness_depth':max(depth(x['value']) for x in w),'calendar_period_bound':maximum,'calendar_start_dates_checked':146097,'reader_links':len(links),'validation_scope':'Static identities/refs/limited fixture shapes/budgets/calendar arithmetic only; no SQL/Cedar/runtime execution or Verus proof','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'separate_rust_lane':rp['candidate_sha'],'implementation_authorized':False},indent=2))
