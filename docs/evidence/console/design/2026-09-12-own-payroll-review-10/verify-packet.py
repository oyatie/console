"""Evidence identity/CSV/link checks only; no product scenario is executed."""
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
receipt = json.loads((p / 'review-receipt.json').read_text())
scope = json.loads((p / 'scope.json').read_text())
sha = receipt['reviewed_candidate_sha']
core = ['entry.txt', 'own-result-contract.txt', 'handover-triggers.txt', 'acceptance-plan.csv',
        'screen-states.csv', 'source-identities.json', 'scope.json', 'review.html']
for name in core:
    blob = subprocess.check_output(['git', 'show', f'{sha}:{relative / name}'], cwd=repo)
    assert blob == (p / name).read_bytes(), name
sources = json.loads((p / 'source-identities.json').read_text())['sources']
for item in sources:
    spec = f"{item['commit']}:{item['path']}"
    blob = subprocess.check_output(['git', 'show', spec], cwd=repo)
    oid = subprocess.check_output(['git', 'rev-parse', spec], cwd=repo).decode().strip()
    assert oid == item['git_blob']
    assert hashlib.sha256(blob).hexdigest() == item['sha256']
with (p / 'acceptance-plan.csv').open() as f:
    cases = list(csv.DictReader(f))
with (p / 'screen-states.csv').open() as f:
    screens = list(csv.DictReader(f))
assert len(cases) == scope['scenarios']
assert len(screens) == scope['screen_states']
assert [r['id'] for r in cases] == [f'P{i:02}' for i in range(1, len(cases) + 1)]
assert [r['id'] for r in screens] == [f'V{i:02}' for i in range(1, len(screens) + 1)]
sections = set(re.findall(r'^(?:PR1|WA2)\.\d+\b', (p / 'own-result-contract.txt').read_text() + '\n' + (p / 'handover-triggers.txt').read_text(), re.M))
assert len(sections) == 14
for row in cases:
    assert None not in row and all(row.values())
    assert set(row['contract_refs'].split(',')) <= sections
for row in screens:
    assert None not in row and all(row.values())

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
for file in p.glob('*.json'):
    json.loads(file.read_text())
assert receipt['whole_design_consensus'] is False
assert receipt['implementation_authorized'] is False
assert receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p / 'record.json').exists():
    for item in json.loads((p / 'record.json').read_text())['files']:
        blob = (p / item['path']).read_bytes()
        assert len(blob) == item['bytes']
        assert hashlib.sha256(blob).hexdigest() == item['sha256']
print(json.dumps({'result': 'PASS', 'reviewed_sha': sha, 'core_files_verified': len(core),
                  'source_blobs_verified': len(sources), 'scenarios_discovered': len(cases),
                  'product_scenarios_executed': 0, 'screen_states': len(screens),
                  'contract_sections': len(sections), 'reader_links_verified': len(links.links),
                  'record_manifest_checked': (p / 'record.json').exists()}))
