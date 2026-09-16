"""Evidence integrity and structural references only; no product or SQL tests."""
import csv
import hashlib
import json
import subprocess
from html.parser import HTMLParser
from pathlib import Path
p = Path(__file__).resolve().parent
repo = next(x for x in p.parents if (x / '.git').exists())
rel = p.relative_to(repo)
receipt = json.loads((p / 'review-receipt.json').read_text())
sha = receipt['reviewed_candidate_sha']
core = ['entry.txt','account-contract.txt','policy-contract.txt','migration-and-proof.txt',
        'schema-candidate.json','boundary-api.json','writer-enrollment.csv','proof-cases.csv',
        'scope.json','source-identities.json','review.html']
for name in core:
    expected = subprocess.check_output(['git','show',f'{sha}:{rel / name}'],cwd=repo)
    assert expected == (p / name).read_bytes(), name
sources = json.loads((p / 'source-identities.json').read_text())['sources']
for s in sources:
    spec = f"{s['commit']}:{s['path']}"
    b = subprocess.check_output(['git','show',spec],cwd=repo)
    oid = subprocess.check_output(['git','rev-parse',spec],cwd=repo).decode().strip()
    assert s['git_blob'] == oid and s['sha256'] == hashlib.sha256(b).hexdigest(),spec
scope = json.loads((p / 'scope.json').read_text())
schema = json.loads((p / 'schema-candidate.json').read_text())
tables = schema['tables']
assert len(tables) == len({x['name'] for x in tables}) == scope['proposed_tables'] == 18
for t in tables:
    assert t['status'] == 'PROPOSED_NEW_TABLE'
    assert set(t['primary_key']) <= set(t['columns']) and t['constraints'] and t['access'] and t['lock']
api = json.loads((p / 'boundary-api.json').read_text())
assert len(api['routes']) == scope['api_boundary_routes'] == 11
assert len({(x['method'],x['route']) for x in api['routes']}) == 11
assert scope['deferred_api_boundaries'] == len(api['deferred_boundaries']) == 1
assert all(r['route'] != '/api/v2/accounts/me/contexts' for r in api['routes'])
assert api['deferred_boundaries'][0]['status'] == 'UNSPECIFIED_NOT_ADMITTED_BLOCKING_W04b_04'
for r in api['routes']:
    assert all(r.values()) and r['route'].startswith('/api/')
with (p / 'writer-enrollment.csv').open() as f:
    writers = list(csv.DictReader(f))
assert len(writers) == scope['source_writer_surfaces'] == 22
assert len({w['id'] for w in writers}) == 22
known_paths = {s['path'] for s in sources}
for w in writers:
    assert w['source_path'] in known_paths and None not in w and all(w.values())
with (p / 'proof-cases.csv').open() as f:
    cases = list(csv.DictReader(f))
assert len(cases) == scope['nonexecutable_proof_cases'] == 29
assert len({c['id'] for c in cases}) == 29
assert all(None not in c and all(c.values()) for c in cases)
class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
    def handle_starttag(self,tag,attrs):
        if tag == 'a':
            self.links.append(dict(attrs)['href'])
l = Links()
l.feed((p / 'review.html').read_text())
for href in l.links:
    assert (p / href).is_file(),href
for f in p.glob('*.json'):
    json.loads(f.read_text())
assert receipt['implementation_authorized'] is False and receipt['whole_design_consensus'] is False
assert receipt['w04_complete'] is False and receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p / 'record.json').exists():
    record = json.loads((p / 'record.json').read_text())
    assert {f.name for f in p.iterdir() if f.is_file()} == {'record.json'} | {x['path'] for x in record['files']}
    for item in record['files']:
        b = (p / item['path']).read_bytes()
        assert len(b) == item['bytes'] and hashlib.sha256(b).hexdigest() == item['sha256']
print(json.dumps(dict(result='PASS',reviewed_sha=sha,core_files_verified=len(core),source_blobs_verified=len(sources),
    proposed_tables=len(tables),api_boundaries=len(api['routes']),writer_source_surfaces=len(writers),
    nonexecutable_cases_discovered=len(cases),reader_links_verified=len(l.links),
    product_tests_executed=0,record_manifest_checked=(p / 'record.json').exists())))
