"""Check evidence identities and links; exercises no product behavior."""
import csv
import hashlib
import json
import subprocess
from html.parser import HTMLParser
from pathlib import Path

p = Path(__file__).resolve().parent
repo = next(parent for parent in p.parents if (parent / '.git').exists())
relative = p.relative_to(repo)
receipt = json.loads((p / 'review-receipt.json').read_text())
sha = receipt['reviewed_candidate_sha']
core = ['entry.txt', 'readiness.txt', 'assignment-next.txt', 'work-packages.csv',
        'source-identities.json', 'scope.json', 'review.html']
for name in core:
    blob = subprocess.check_output(['git', 'show', f'{sha}:{relative / name}'], cwd=repo)
    assert blob == (p / name).read_bytes(), name
sources = json.loads((p / 'source-identities.json').read_text())
for item in sources['sources']:
    spec = f"{item['commit']}:{item['path']}"
    blob = subprocess.check_output(['git', 'show', spec], cwd=repo)
    oid = subprocess.check_output(['git', 'rev-parse', spec], cwd=repo).decode().strip()
    assert oid == item['git_blob']
    assert hashlib.sha256(blob).hexdigest() == item['sha256']
local_root = Path('/Users/jasonlee/Developer/console')
for item in sources['local_read_only_evidence']:
    blob = (local_root / item['path']).read_bytes()
    assert len(blob) == item['bytes']
    assert hashlib.sha256(blob).hexdigest() == item['sha256']
with (p / 'work-packages.csv').open() as f:
    rows = list(csv.DictReader(f))
assert len(rows) == 16
assert len({r['id'] for r in rows}) == 16
assert [r['id'] for r in rows if r['readiness_level'] == 'FIRST_IMPLEMENTATION'] == [f'W{i:02}' for i in range(1, 7)]
for row in rows:
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
assert receipt['b04_closed'] is False
assert receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p / 'record.json').exists():
    for item in json.loads((p / 'record.json').read_text())['files']:
        blob = (p / item['path']).read_bytes()
        assert len(blob) == item['bytes']
        assert hashlib.sha256(blob).hexdigest() == item['sha256']
print(json.dumps({'result': 'PASS', 'reviewed_sha': sha, 'core_files_verified': len(core),
                  'git_source_blobs_verified': len(sources['sources']),
                  'read_only_local_source_hashes_verified': len(sources['local_read_only_evidence']),
                  'work_packages_discovered': len(rows), 'product_tests_executed': 0,
                  'reader_links_verified': len(links.links),
                  'record_manifest_checked': (p / 'record.json').exists()}))
