"""Evidence identities, data references and reader parity; no product test execution."""
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
scope = json.loads((p / 'scope.json').read_text())
sha = receipt['reviewed_candidate_sha']
core = ['entry.txt', 'screen-contract.txt', 'screen-map.json', 'command-map.json',
        'state-matrix.csv', 'input-overlays.json', 'journeys.json',
        'source-identities.json', 'scope.json', 'review.html']
for name in core:
    blob = subprocess.check_output(['git', 'show', f'{sha}:{relative / name}'], cwd=repo)
    assert blob == (p / name).read_bytes(), name
sources = json.loads((p / 'source-identities.json').read_text())['sources']
for item in sources:
    spec = f"{item['commit']}:{item['path']}"
    blob = subprocess.check_output(['git', 'show', spec], cwd=repo)
    oid = subprocess.check_output(['git', 'rev-parse', spec], cwd=repo).decode().strip()
    assert oid == item['git_blob'] and hashlib.sha256(blob).hexdigest() == item['sha256']
screens = json.loads((p / 'screen-map.json').read_text())
commands = json.loads((p / 'command-map.json').read_text())
overlays = json.loads((p / 'input-overlays.json').read_text())
journeys = json.loads((p / 'journeys.json').read_text())
with (p / 'state-matrix.csv').open() as f:
    matrix = list(csv.DictReader(f))
ids = {x['id'] for x in screens}
cmd_ids = {x['id'] for x in commands}
assert len(ids) == len(screens) == scope['main_screens'] + scope['auxiliary_surfaces'] == 16
assert len(cmd_ids) == len(commands) == scope['logical_commands'] == 37
assert len(matrix) == scope['state_rows'] == 160
states = {'LOADING', 'EMPTY', 'DENIED', 'READY', 'INCOMPLETE', 'PARTIAL', 'STALE', 'FAILED', 'UNKNOWN', 'SUCCESS'}
assert {(x['screen'], x['state']) for x in matrix} == {(s, t) for s in ids for t in states}
for row in matrix:
    assert None not in row and all(row.values())
    assert set(row['allowed_actions'].split(';')) <= cmd_ids | {'NONE'}
    assert row['applicability'] in {'APPLIES', 'NOT_APPLICABLE'}
for s in screens:
    assert set(s['ready_actions']) <= cmd_ids
for c in commands:
    assert c['when_offered'] and c['receipt_or_guard']
assert len(overlays) == scope['input_overlays'] == 7
for o in overlays:
    assert set(o['allowed_actions']) <= cmd_ids | {'NONE'}
assert len(journeys) == scope['nonexecutable_journeys'] == 13
for j in journeys:
    for step in j['steps']:
        assert step['surface'] in ids and step['command'] in cmd_ids and step['expected']

class Reader(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
        self.in_data = False
        self.data = ''

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == 'a':
            self.links.append(a['href'])
        if tag == 'script' and a.get('id') == 'contract-data':
            self.in_data = True

    def handle_endtag(self, tag):
        if tag == 'script':
            self.in_data = False

    def handle_data(self, data):
        if self.in_data:
            self.data += data

reader = Reader()
reader.feed((p / 'review.html').read_text())
for href in reader.links:
    assert (p / href.split('#')[0]).is_file(), href
assert json.loads(reader.data) == dict(screens=screens, commands=commands, matrix=matrix, journeys=journeys)
assert receipt['whole_design_consensus'] is False and receipt['implementation_authorized'] is False
assert receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
for f in p.glob('*.json'):
    json.loads(f.read_text())
if (p / 'record.json').exists():
    for item in json.loads((p / 'record.json').read_text())['files']:
        b = (p / item['path']).read_bytes()
        assert len(b) == item['bytes'] and hashlib.sha256(b).hexdigest() == item['sha256']
print(json.dumps({'result': 'PASS', 'reviewed_sha': sha, 'core_files_verified': len(core),
                  'source_blobs_verified': len(sources), 'surfaces': len(screens),
                  'logical_commands': len(commands), 'matrix_rows': len(matrix),
                  'input_overlays': len(overlays), 'journeys_discovered': len(journeys),
                  'journey_steps': sum(len(j['steps']) for j in journeys),
                  'product_tests_executed': 0, 'reader_data_parity': True,
                  'reader_links_verified': len(reader.links),
                  'record_manifest_checked': (p / 'record.json').exists()}))
