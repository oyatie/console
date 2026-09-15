#!/usr/bin/env python3
"""Static evidence identities, closed reference graph and synthetic encoding arithmetic only."""
import argparse,csv,hashlib,json,re,subprocess,copy,datetime
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);args=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*a):return subprocess.check_output(['git',*a],cwd=root)
sha=git('rev-parse',args.commit).decode().strip()
def blob(c,path):
 row=git('ls-tree',c,'--',path).decode().split();assert row[:2]==['100644','blob'],(c,path)
 return row[2],git('show',c+':'+path)
files=['entry.txt','source-types.schema.json','source-contract.txt','encoding-and-packages.txt','state-contract.json','dependency-contract.csv','repair-and-readiness.txt','normalization-vectors.json','proof-cases.csv','source-identities.json','scope.json','readiness.json','review.html','verify-packet.py']
b={}
for n in files:
 _,raw=blob(sha,rel+'/'+n);assert raw==(p/n).read_bytes(),n;b[n]=raw
j=lambda n:json.loads(b[n]);scope=j('scope.json');ready=j('readiness.json')
for k in ('w04_complete','whole_design_approved','implementation_authorized'):assert scope[k] is False and ready[k] is False
assert scope['product_tests_written']==scope['product_tests_executed']==0
sources=j('source-identities.json')['sources'];assert len(sources)==scope['source_blobs'];assert len({(x['commit'],x['path']) for x in sources})==len(sources)
for s in sources:
 oid,raw=blob(s['commit'],s['path']);assert oid==s['git_blob'] and hashlib.sha256(raw).hexdigest()==s['sha256']
schema=j('source-types.schema.json');defs=schema['$defs'];assert len(defs)==scope['schema_definitions'];refs=[];objects=[]
def visit(x):
 if isinstance(x,dict):
  if '$ref' in x:refs.append(x['$ref'])
  if x.get('type')=='object':objects.append(x);assert x.get('additionalProperties') is False;assert set(x['required'])==set(x['properties'])
  for v in x.values():visit(v)
 elif isinstance(x,list):
  for v in x:visit(v)
visit(schema);assert all(r.startswith('#/$defs/') and r[8:] in defs for r in refs)
def children(x):
 if isinstance(x,dict):
  if '$ref' in x:yield x['$ref'][8:]
  for v in x.values():yield from children(v)
 elif isinstance(x,list):
  for v in x:yield from children(v)
def acyclic(n,stack):
 assert n not in stack,(n,stack)
 for c in children(defs[n]):acyclic(c,stack+(n,))
for n in defs:acyclic(n,())
seen=set()
def reachable(n):
 if n in seen:return
 seen.add(n)
 for c in children(defs[n]):reachable(c)
for r in schema['oneOf']:reachable(r['$ref'][8:])
assert seen==set(defs)
canon=lambda x:json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
hashbytes=lambda b:hashlib.sha256(b).hexdigest()
def depth(x):
 if isinstance(x,dict):return 1+max((depth(v) for v in x.values()),default=0)
 if isinstance(x,list):return 1+max((depth(v) for v in x),default=0)
 assert x is None or isinstance(x,(str,bool));return 0
# Deliberately limited static fixture shape checker, not a product JSON Schema validator.
# Handles exactly the schema vocabulary below; unknown keywords fail instead of being ignored.
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
v=j('normalization-vectors.json');vm={x['id']:x for x in v['vectors']}
for x in vm.values():
 name,_,member=x['shape'].partition('@');shape(x['value'][member] if member else x['value'],defs[name])
 raw=canon(x['value']);assert raw.hex()==x['canonical_utf8_hex'];assert len(raw)<=65536 and depth(x['value'])<=8
 assert hashbytes(x['tag'].encode()+bytes([0])+raw)==x['sha256']
for fkey,tag,idkey in [('event_frame','event-set','event'),('close_frame','close-members','employee')]:
 f=v[fkey];hv=vm[f['header_vector']];members=[vm[k] for k in f['member_vectors']]
 ids=[x['value']['ref']['attendance_record_id'] if idkey=='event' else x['value']['employee_id'] for x in members]
 assert len(set(ids))==len(ids)
 if idkey=='event':assert members==sorted(members,key=lambda x:(int(x['value']['occurred_at_us']),x['value']['ref']['attendance_record_id']))
 else:assert ids==sorted(ids)
 raw=('console.source19.'+tag).encode()+bytes([0])+bytes.fromhex(hv['sha256'])+len(ids).to_bytes(4,'big')
 for ident,x in zip(ids,members):raw+=bytes.fromhex(ident.replace('-',''))+bytes.fromhex(x['sha256'])
 assert raw.hex()==f['raw_hex'] and hashbytes(raw)==f['sha256']
 file=b''.join(canon(x['value'])+bytes([10]) for x in [hv,*members]);assert file.hex()==f['file_utf8_hex'] and hashbytes(file)==f['file_sha256'];assert f['file_sha256']!=f['sha256']
 assert all(len(canon(x['value']))<=8192 for x in [hv,*members])
 countkey='event_count' if idkey=='event' else 'member_count';assert int(hv['value'][countkey])==len(ids)
for f in v['amendment_frames']:
 ids=f['amendment_ids'];assert ids==sorted(set(ids));raw=b'console.source19.close-amendments'+bytes([0])+bytes.fromhex(f['org_id'].replace('-',''))+bytes.fromhex(f['close_id'].replace('-',''))+len(ids).to_bytes(4,'big')+b''.join(bytes.fromhex(i.replace('-','')) for i in ids)
 assert raw.hex()==f['raw_hex'] and hashbytes(raw)==f['sha256']
assert len({f['sha256'] for f in v['amendment_frames']})==3
f=v['coverage_day_frame'];header=vm[f['header_vector']];days=[vm[x] for x in f['member_vectors']];dates=[x['value']['date'] for x in days];assert dates==sorted(set(dates))
raw=b'console.source19.coverage-days'+bytes([0])+bytes.fromhex(header['sha256'])+len(days).to_bytes(4,'big')
for x in days:raw+=x['value']['date'].encode()+bytes.fromhex(x['sha256'])
assert raw.hex()==f['raw_hex'] and hashbytes(raw)==f['sha256'];file=b''.join(canon(x['value'])+bytes([10]) for x in [header,*days]);assert file.hex()==f['file_utf8_hex'] and hashbytes(file)==f['file_sha256']
assert int(header['value']['day_count'])==len(days);assert header['value']['event_set_digest']==v['event_frame']['sha256']
coverage=vm['V13']['value']['payload'];assert coverage['coverage_days_ref']['day_set_digest']==f['sha256'] and coverage['outcome']=='BLOCKED'
day=vm['V11']['value'];assert sum(int(x['expected_work_minutes']) for x in day['plan_contributions'])==int(day['expected_work_minutes'])==840
assert [x['plan_digest'] for x in day['plan_contributions']]==[vm['V09']['sha256'],vm['V10']['sha256']]
# Capacity witness checks serialization bounds only; no synthetic row becomes a qualified source.
cap=v['capacity_witness'];sample=[]
for shift in range(cap['days']):
 x=copy.deepcopy(vm[cap['template']]['value']);delta=datetime.timedelta(days=shift)
 x['date']=(datetime.date.fromisoformat(x['date'])+delta).isoformat()
 for c in x['plan_contributions']:c['anchor_date']=(datetime.date.fromisoformat(c['anchor_date'])+delta).isoformat()
 for i in x['intervals']:
  for key in ('from_us','to_us'):i[key]=str(int(i[key])+shift*86400000000)
  if i['plan_anchor_date'] is not None:i['plan_anchor_date']=(datetime.date.fromisoformat(i['plan_anchor_date'])+delta).isoformat()
 shape(x,defs['CoverageDay']);assert depth(x)<=8 and len(canon(x))<=cap['maximum_day_bytes'];sample.append(x)
capacity_header=copy.deepcopy(header['value']);capacity_header['day_count']=str(len(sample));capacity_bytes=len(canon(capacity_header))+1+sum(len(canon(x))+1 for x in sample)
assert cap['minimum_stream_bytes_exclusive']<capacity_bytes<=cap['maximum_stream_bytes']

cal=vm['V01']['value']['payload'];d=cal['weekly'][0]['duties'][0];work=int(d['end_minute'])-int(d['start_minute'])-sum(int(x['end_minute'])-int(x['start_minute']) for x in d['breaks'])
assert work==480==v['arithmetic']['normal_work_minutes'];assert vm['V01']['sha256']!=vm['V02']['sha256']
o=cal['overrides'][0];assert int(o['duties'][0]['end_minute'])-int(o['duties'][0]['start_minute'])==240==v['arithmetic']['override_work_minutes']
a=int(vm['V05']['value']['occurred_at_us']);z=int(vm['V06']['value']['occurred_at_us']);assert (z-a)//60000000==480==v['arithmetic']['cross_month_pair_minutes']
# September starts 24h after the package query window begins; clipping uses exact microseconds.
start=int(vm['V03']['value']['query_from_us'])+86400000000;assert (z-max(a,start))//60000000==360==v['arithmetic']['september_clipped_minutes']
proof=list(csv.DictReader(b['proof-cases.csv'].decode().splitlines()));assert len(proof)==scope['nonexecutable_proof_cases']==40 and len({x['id'] for x in proof})==40;assert all(x['status']=='PROPOSED_NOT_EXECUTED' for x in proof)
st=j('state-contract.json');assert {x['kind'] for x in st['events']}=={'PROPOSE','VERIFY','REVOKE','SUPERSEDE'}
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in files for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(files),'source_blobs':len(sources),'schema_definitions':len(defs),'closed_objects':len(objects),'resolved_schema_refs':len(refs),'root_shapes':len(schema['oneOf']),'canonical_witnesses':len(vm),'framed_witnesses':6,'thirty_day_serialization_bytes':capacity_bytes,'max_witness_container_depth':max(depth(x['value']) for x in vm.values()),'nonexecutable_proof_cases':len(proof),'reader_links':len(links),'validation_scope':'Static packet/reference/limited fixture shape/framing/arithmetic only; no full JSON Schema validator or runtime semantics','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'implementation_authorized':False},indent=2))
