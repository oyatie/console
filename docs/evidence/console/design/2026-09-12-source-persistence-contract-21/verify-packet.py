#!/usr/bin/env python3
"""Static custody and finite result frame checks; no product/SQL/Verus execution."""
import argparse,csv,json,hashlib,subprocess,uuid,re
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);a=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*x):return subprocess.check_output(['git',*x],cwd=root)
sha=git('rev-parse',a.commit).decode().strip()
def blob(c,path):
 m=git('ls-tree',c,'--',path).decode().split();assert m and m[0]=='100644' and m[1]=='blob',path
 return m[2],git('show',c+':'+path)
files=['entry.txt','persistence-contract.txt','information-flow.txt','source-key-matrix.json','result-membership.json','membership-witnesses.json','sink-census.csv','proof-cases.csv','readiness.json','scope.json','source-identities.json','review.html','verify-packet.py']
b={}
for f in files:
 _,v=blob(sha,rel+'/'+f);assert v==(p/f).read_bytes(),f;b[f]=v
j=lambda n:json.loads(b[n]);s=j('scope.json');r=j('readiness.json')
for k in ['implementation_authorized','whole_design_approved','w04_complete']:assert s[k] is False and r[k] is False
assert s['product_tests_written']==s['product_tests_executed']==s['verus_proofs_executed']==0 and s['sql_executed'] is False
sources=j('source-identities.json')['sources'];assert len(set((x['commit'],x['path']) for x in sources))==len(sources)
sourcebytes={}
for x in sources:
 oid,v=blob(x['commit'],x['path']);assert oid==x['git_blob'] and hashlib.sha256(v).hexdigest()==x['sha256'];sourcebytes[x['path']]=v
keys=j('source-key-matrix.json')['rows'];assert len(keys)==s['source_kinds']==7 and [x['code'] for x in keys]==list(range(1,8))
assert len({x['table'] for x in keys})==7
for x in keys:assert x['primary_key']==['org_id',x['root_column'],'revision'] and x['claim_allowed']==(x['code']>=5)
contract=j('result-membership.json');f=contract['frame'];vectors=j('membership-witnesses.json')['vectors'];assert len(vectors)==s['synthetic_frames']==3
width=1+1+16+8+32+1+16+8+32;assert f['member_bytes']==width==115
for v in vectors:
 members=v['members'];assert 1<=len(members)<=f['maximum_members']==1024
 order=lambda m:(m['family_code'],uuid.UUID(m['root_id']).bytes,int(m['revision']),m['effect_kind'])
 assert members==sorted(members,key=order) and len({order(m) for m in members})==len(members)
 data=f['prefix_utf8'].encode()+bytes.fromhex(f['separator_hex'])+uuid.UUID(v['org_id']).bytes+uuid.UUID(v['command_id']).bytes+len(members).to_bytes(4,'big')
 for m in members:
  assert 1<=m['family_code']<=7 and m['effect_kind'] in [1,2] and 0<int(m['revision'])<2**63
  claim=m['effect_kind']==2
  assert all(m[k] is not None for k in ['successor_root_id','successor_revision','successor_custody_digest']) if claim else all(m[k] is None for k in ['successor_root_id','successor_revision','successor_custody_digest'])
  if claim:
   assert m['family_code']>=5 and m['successor_root_id']!=m['root_id'] and 0<int(m['successor_revision'])<2**63
   successor=[x for x in members if x['family_code']==m['family_code'] and x['effect_kind']==1 and x['root_id']==m['successor_root_id'] and x['revision']==m['successor_revision']]
   assert len(successor)==1 and successor[0]['custody_digest']==m['successor_custody_digest']
  piece=bytes([m['family_code'],m['effect_kind']])+uuid.UUID(m['root_id']).bytes+int(m['revision']).to_bytes(8,'big')+bytes.fromhex(m['custody_digest'])+bytes([claim])+(uuid.UUID(m['successor_root_id']).bytes if claim else bytes(16))+int(m['successor_revision'] or '0').to_bytes(8,'big')+(bytes.fromhex(m['successor_custody_digest']) if claim else bytes(32))
  assert len(piece)==width;data+=piece
 assert data.hex()==v['frame_hex'] and len(data)==v['frame_bytes'] and hashlib.sha256(data).hexdigest()==v['sha256']
maximum=len(f['prefix_utf8'].encode())+1+16+16+4+width*1024;assert maximum==f['maximum_framed_bytes']
counts={}
for file,key in [('sink-census.csv','sink_rows'),('proof-cases.csv','nonexecuted_cases')]:
 rows=list(csv.DictReader(b[file].decode().splitlines()));assert len(rows)==s[key] and len({x['id'] for x in rows})==len(rows);counts[key]=len(rows)
 if file=='sink-census.csv':
  for x in rows:assert x['path'] in sourcebytes and 1<=int(x['line'])<=len(sourcebytes[x['path']].splitlines())
 else:assert all(x['status']=='PROPOSED_NOT_EXECUTED' for x in rows)
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in files for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(files),'source_blobs':len(sources),'source_kinds':len(keys),**counts,'synthetic_frames':len(vectors),'member_frame_bytes':width,'max1024_member_frame_bytes':maximum,'reader_links':len(links),'validation_scope':'Static custody/reference/frame arithmetic only; no confidentiality theorem, full schema semantics, SQL or product execution','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'implementation_authorized':False},indent=2))
