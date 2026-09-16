"""Evidence custody/structure/encoding only; no Rust/SQL/runtime or product tests."""
import csv, hashlib, json, subprocess, sys
from pathlib import Path
from html.parser import HTMLParser
p=Path(__file__).resolve().parent
repo=next(x for x in p.parents if (x/'.git').exists())
rel=p.relative_to(repo)
def read(n):
    def pairs(xs):
        d={}
        for k,v in xs:
            assert k not in d,(n,k)
            d[k]=v
        return d
    return json.loads((p/n).read_text(),object_pairs_hook=pairs)
receipt=read('review-receipt.json') if (p/'review-receipt.json').exists() else None
sha=sys.argv[1] if len(sys.argv)==2 else receipt['reviewed_candidate_sha']
def git(*args):return subprocess.check_output(['git',*args],cwd=repo)
core=['action-inputs.json','common-action-principle.txt','entry.txt','group-and-navigation.txt','group-command-witnesses.json','participating-ports.json','proof-cases.csv','review-and-publication.txt','review.html','schema-candidate.json','scope.json','source-identities.json','work-and-capacity.txt']
for n in core:
    assert git('ls-tree',sha,str(rel/n)).startswith(b'100644 blob '),n
    assert git('show',f'{sha}:{rel/n}')==(p/n).read_bytes(),n
sources=read('source-identities.json')['sources']
for s in sources:
    spec=f"{s['commit']}:{s['path']}"
    assert git('ls-tree',s['commit'],s['path']).startswith(b'100644 blob ')
    assert git('rev-parse',spec).decode().strip()==s['git_blob']
    assert hashlib.sha256(git('show',spec)).hexdigest()==s['sha256']
scope,schema,actions,ports=map(read,['scope.json','schema-candidate.json','action-inputs.json','participating-ports.json'])
assert len(sources)==len({(s['commit'],s['path']) for s in sources})==scope['source_blobs']==37
assert len(schema['tables'])==len({t['name'] for t in schema['tables']})==scope['proposed_tables']==30
assert len(schema['extensions'])==scope['existing_extensions']==5
for t in schema['tables']:
    assert t['cell'] in ['COMPANY','GROUP','ACCOUNT']
    assert set(t['primary_key'])<=set(t['columns'])
    assert t['owner'] and t['lock'] and t['constraints'] and t['access']
    assert (t['cell']!='COMPANY' or 'org_id' in t['primary_key'])
assert len(actions['actions'])==len({a['action'] for a in actions['actions']})==scope['action_bindings']==16
assert len(ports['ports'])==len({a['method'] for a in ports['ports']})==scope['participating_ports']==16
for a in actions['actions']:assert a['target'] and a['input'] and a['outcome']
for a in ports['ports']:assert a['owner'] and a['input'] and a['result']
with (p/'proof-cases.csv').open() as f:cases=list(csv.DictReader(f))
assert len(cases)==len({c['id'] for c in cases})==scope['nonexecutable_proof_cases']==70
assert all(None not in c and all(c.values()) and c['status']=='NONEXECUTABLE_CANDIDATE' for c in cases)
def restricted(v,depth=0):
    assert depth<=8
    if isinstance(v,dict):
        assert all(isinstance(k,str) and k.isascii() for k in v)
        for x in v.values():restricted(x,depth+1)
    elif isinstance(v,list):
        for x in v:restricted(x,depth+1)
    else:
        assert v is None or isinstance(v,(str,bool))
        if isinstance(v,str):assert chr(0) not in v;v.encode('utf8')
data=read('group-command-witnesses.json')
assert len(data['vectors'])==scope['group_encoding_witnesses']==2
for v in data['vectors']:
    e=v['envelope'];restricted(e)
    assert set(e)==set(data['fields']) and e['protocol']=='GROUP_CONTROL_V1'
    assert 'org_id' not in e and e['group_id'] and e['actor_account_id']
    raw=json.dumps(e,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
    assert len(raw)<=65536 and raw.decode()==v['canonical_utf8']
    pre=b'console.group-command.ccf1\0'+raw
    assert pre.hex()==v['preimage_hex'] and hashlib.sha256(pre).hexdigest()==v['sha256']
assert len({v['sha256'] for v in data['vectors']})==2
class Links(HTMLParser):
    def __init__(self):super().__init__();self.links=[]
    def handle_starttag(self,tag,attrs):
        if tag=='a':self.links.append(dict(attrs)['href'])
l=Links();l.feed((p/'review.html').read_text())
for href in l.links:assert (p/href).is_file(),href
for f in p.glob('*.json'):read(f.name)
assert not scope['w04_complete'] and not scope['whole_design_approved'] and not scope['implementation_authorized']
if receipt:
    assert sha==receipt['reviewed_candidate_sha']
    assert not receipt['whole_design_consensus'] and not receipt['implementation_authorized'] and not receipt['w04_complete']
    assert receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p/'record.json').exists():
    record=read('record.json')
    assert {x.name for x in p.iterdir()}=={'record.json'}|{x['path'] for x in record['files']}
    for f in record['files']:
        b=(p/f['path']).read_bytes();assert len(b)==f['bytes'] and hashlib.sha256(b).hexdigest()==f['sha256']
    assert hashlib.sha256((p/record['predecessor_record']).read_bytes()).hexdigest()==record['predecessor_record_sha256']
print(json.dumps(dict(result='PASS',reviewed_sha=sha,core_files_verified=len(core),source_blobs_verified=len(sources),proposed_tables=len(schema['tables']),existing_extensions=len(schema['extensions']),participating_ports=len(ports['ports']),action_bindings=len(actions['actions']),nonexecutable_cases_discovered=len(cases),group_encoding_witnesses_verified=len(data['vectors']),reader_links_verified=len(l.links),product_tests_discovered=0,product_tests_executed=0,record_manifest_checked=(p/'record.json').exists())))
