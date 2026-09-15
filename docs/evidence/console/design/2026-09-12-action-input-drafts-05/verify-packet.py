"""Evidence consistency only; no product behavior is exercised."""
import csv
import hashlib
import json
import re
import subprocess
from html.parser import HTMLParser
from pathlib import Path

p = Path(__file__).resolve().parent
repo = next(parent for parent in p.parents if (parent / '.git').exists())
relative = p.relative_to(repo)
scope = json.loads((p / 'scope.json').read_text())
receipt = json.loads((p / 'review-receipt.json').read_text())
sha = receipt['reviewed_candidate_sha']
core = ['entry.txt', 'draft-contract.txt', 'entity-revision-contract.txt',
        'entity-versioning-direction.txt', 'acceptance-plan.csv',
        'screen-states.csv', 'source-identities.json', 'scope.json']
for name in core:
    blob = subprocess.check_output(['git', 'show', f'{sha}:{relative / name}'], cwd=repo)
    assert blob == (p / name).read_bytes(), name
sources = json.loads((p / 'source-identities.json').read_text())
for item in sources['sources']:
    spec = f"{sources['source_base_sha']}:{item['path']}"
    blob = subprocess.check_output(['git', 'show', spec], cwd=repo)
    oid = subprocess.check_output(['git', 'rev-parse', spec], cwd=repo).decode().strip()
    assert oid == item['git_blob'], item['path']
    assert hashlib.sha256(blob).hexdigest() == item['sha256'], item['path']
with (p / 'acceptance-plan.csv').open() as f:
    scenarios = list(csv.DictReader(f))
with (p / 'screen-states.csv').open() as f:
    screens = list(csv.DictReader(f))
assert len(scenarios) == scope['scenarios'] == 50
assert len(screens) == scope['screen_states'] == 18
assert [r['id'] for r in scenarios] == [f'F{i:02}' for i in range(1, 51)]
contract = (p / 'draft-contract.txt').read_text() + (p / 'entity-revision-contract.txt').read_text()
declared = set(re.findall(r'^FD1\.\d+\b', contract, re.M))
assert len(declared) == 17
for row in scenarios:
    assert None not in row and all(row.values()), row
    assert set(row['contract_refs'].split(',')) <= declared, row['id']
for row in screens:
    assert None not in row and all(row.values()), row

class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        for key, value in attrs:
            if key == 'href':
                self.links.append(value)

links = Links()
links.feed((p / 'review.html').read_text())
for href in links.links:
    assert (p / href.split('#')[0]).is_file(), href
json_files = list(p.glob('*.json'))
for file in json_files:
    json.loads(file.read_text())
assert not receipt['whole_design_consensus'] and not receipt['implementation_authorized']
assert receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p / 'record.json').exists():
    for item in json.loads((p / 'record.json').read_text())['files']:
        blob = (p / item['path']).read_bytes()
        assert len(blob) == item['bytes']
        assert hashlib.sha256(blob).hexdigest() == item['sha256']
print(json.dumps({'result': 'PASS', 'reviewed_sha': sha, 'core_files_verified': len(core),
                  'source_blobs_verified': len(sources['sources']),
                  'scenarios_discovered': len(scenarios), 'product_scenarios_executed': 0,
                  'screen_states': len(screens), 'contract_sections': len(declared),
                  'reader_links_verified': len(links.links), 'json_files_parsed': len(json_files),
                  'record_manifest_checked': (p / 'record.json').exists()}))
