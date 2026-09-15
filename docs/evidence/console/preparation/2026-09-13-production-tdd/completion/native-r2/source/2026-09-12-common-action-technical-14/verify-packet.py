"""Evidence byte/structure/encoding witnesses only. No product tests or SQL proof."""
import csv, hashlib, json, subprocess
from pathlib import Path
from html.parser import HTMLParser
p=Path(__file__).resolve().parent
repo=next(x for x in p.parents if (x/'.git').exists())
rel=p.relative_to(repo)
r=json.loads((p/'review-receipt.json').read_text())
sha=r['reviewed_candidate_sha']
core=['boundary-ports.json','draft-attempt-contract.txt','encoding-contract.txt','entry.txt','fingerprint-vectors.json','migration-and-proof.txt','proof-cases.csv','registration-contract.txt','registration.csv','review.html','schema-candidate.json','scope.json','source-identities.json']
for name in core:
    actual=(p/name).read_bytes()
    expected=subprocess.check_output(['git','show',f'{sha}:{rel/name}'],cwd=repo)
    assert actual==expected,name
sources=json.loads((p/'source-identities.json').read_text())['sources']
for s in sources:
    spec=f"{s['commit']}:{s['path']}"
    b=subprocess.check_output(['git','show',spec],cwd=repo)
    oid=subprocess.check_output(['git','rev-parse',spec],cwd=repo).decode().strip()
    assert hashlib.sha256(b).hexdigest()==s['sha256'] and oid==s['git_blob'],spec
scope=json.loads((p/'scope.json').read_text())
schema=json.loads((p/'schema-candidate.json').read_text())
assert len(schema['tables'])==len({t['name'] for t in schema['tables']})==scope['proposed_tables']==4
for t in schema['tables']:
    assert set(t['primary_key'])<=set(t['columns']) and t['constraints'] and t['lock'] and t['access']
ports=json.loads((p/'boundary-ports.json').read_text())
assert len(ports['participants'])==len({v['method'] for v in ports['participants']})==scope['participant_contracts']==12
assert len(ports['operations'])==len({v['key'] for v in ports['operations']})==scope['operation_contracts']==12
with (p/'registration.csv').open() as f: mappings=list(csv.DictReader(f))
assert len(mappings)==len({x['np1_command'] for x in mappings})==scope['np1_registration_mappings']==10
assert all('.' not in x['local_key'] and '.' in x['owner_action'] for x in mappings)
with (p/'proof-cases.csv').open() as f: cases=list(csv.DictReader(f))
assert len(cases)==len({x['id'] for x in cases})==scope['nonexecutable_proof_cases']==34
assert all(None not in x and all(x.values()) for x in cases+mappings)
def restricted(v):
    if isinstance(v,dict):
        assert all(isinstance(k,str) and k.isascii() for k in v)
        for x in v.values(): restricted(x)
    elif isinstance(v,list):
        for x in v: restricted(x)
    else:
        assert v is None or isinstance(v,(str,bool)), 'JSON numeric values not admitted'
        if isinstance(v,str): assert chr(0) not in v, 'Decoded NUL not admitted'
def encode(v):
    restricted(v)
    return json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
data=json.loads((p/'fingerprint-vectors.json').read_text()); vectors=data['vectors']
assert len(vectors)==scope['encoding_witnesses']==10
hashes={}
for v in vectors:
    raw=encode(v['value'])
    prefix=b'console.command.ccf1\x00' if v['kind']=='COMMAND' else b'console.draft-revision.ccf1\x00'+bytes.fromhex(v['previous_hash'])
    assert raw.decode()==v['canonical_utf8']
    assert (prefix+raw).hex()==v['preimage_hex']
    assert hashlib.sha256(prefix+raw).hexdigest()==v['sha256']
    hashes[v['id']]=v['sha256']
for a,b in data['equal_hash_pairs']: assert hashes[a]==hashes[b]
for b in data['different_from_F01']: assert hashes[b]!=hashes['F01_WAGE_ENVELOPE']
v1,v2=vectors[-2:]
assert v2['previous_hash']==v1['sha256']
assert int(v2['value']['valid_from_us'])==max(int(v2['value']['recorded_at_us']),int(v1['value']['valid_from_us'])+1)
class Links(HTMLParser):
    def __init__(self): super().__init__(); self.links=[]
    def handle_starttag(self,tag,attrs):
        if tag=='a': self.links.append(dict(attrs)['href'])
l=Links();l.feed((p/'review.html').read_text())
for href in l.links: assert (p/href).is_file(),href
for f in p.glob('*.json'): json.loads(f.read_text())
assert len(sources)==scope['source_blobs']==24
assert r['implementation_authorized'] is False and r['whole_design_consensus'] is False and r['w04_complete'] is False
assert r['design_sha_approval'] is None and r['test_sha_approval'] is None
if (p/'record.json').exists():
    record=json.loads((p/'record.json').read_text())
    assert {x.name for x in p.iterdir()}=={'record.json'}|{x['path'] for x in record['files']}
    for item in record['files']:
        b=(p/item['path']).read_bytes()
        assert len(b)==item['bytes'] and hashlib.sha256(b).hexdigest()==item['sha256']
print(json.dumps(dict(result='PASS',reviewed_sha=sha,core_files_verified=len(core),source_blobs_verified=len(sources),proposed_tables=len(schema['tables']),registration_mappings=len(mappings),participant_contracts=len(ports['participants']),operation_contracts=len(ports['operations']),nonexecutable_cases_discovered=len(cases),encoding_witnesses_verified=len(vectors),decoder_refusal_candidates_discovered=len(data['later_decoder_refusals']),reader_links_verified=len(l.links),product_tests_discovered=0,product_tests_executed=0,record_manifest_checked=(p/'record.json').exists())))
