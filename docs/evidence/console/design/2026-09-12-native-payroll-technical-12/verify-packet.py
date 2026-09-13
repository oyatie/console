"""Evidence custody, references and independent integer arithmetic; no product tests."""
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
core = ['entry.txt', 'native-contract.txt', 'execution-contract.txt', 'migration-and-proof.txt',
        'numeric-oracles.json', 'wire-candidate.json', 'owner-map.csv', 'remaining-work.csv',
        'source-identities.json', 'scope.json', 'review.html']
for name in core:
    expected = subprocess.check_output(['git', 'show', f'{sha}:{rel / name}'], cwd=repo)
    assert (p / name).read_bytes() == expected, name
sources = json.loads((p / 'source-identities.json').read_text())['sources']
for s in sources:
    spec = f"{s['commit']}:{s['path']}"
    b = subprocess.check_output(['git', 'show', spec], cwd=repo)
    oid = subprocess.check_output(['git', 'rev-parse', spec], cwd=repo).decode().strip()
    assert s['git_blob'] == oid and s['sha256'] == hashlib.sha256(b).hexdigest(), spec
with (p / 'owner-map.csv').open() as f:
    owners = list(csv.DictReader(f))
assert len(owners) == 14
known_paths = {s['path'] for s in sources}
for o in owners:
    path = ('backend/crates/' + o['path_from_backend_crates']).replace('backend/crates/../app/', 'backend/app/')
    assert path in known_paths, path
with (p / 'remaining-work.csv').open() as f:
    remaining = list(csv.DictReader(f))
assert len(remaining) == 8 and all(x['status'] in {'OPEN', 'NOT_STARTED', 'SEPARATE_OPEN'} for x in remaining)
scope = json.loads((p / 'scope.json').read_text())
wire = json.loads((p / 'wire-candidate.json').read_text())
assert len(wire['commands']) == scope['logical_mutations'] == 10
assert len({x['id'] for x in wire['commands']}) == 10
assert len({(x['method'], x['route']) for x in wire['commands']}) == 10
for c in wire['commands']:
    assert all(c.values()) and c['method'] == 'POST' and c['route'].startswith('/api/v1/payroll/')
n = json.loads((p / 'numeric-oracles.json').read_text())
assert len(n['positive_cases']) == scope['numeric_positive_oracles'] == 2
assert len(n['refusal_cases']) == scope['numeric_refusal_oracles'] == 5
trunc10 = lambda value: value // 10 * 10
round10 = lambda value: (value + 5) // 10 * 10
for c in n['positive_cases']:
    g = c['gross_won']
    pen = trunc10((c['pension_basis_won'] // 1000 * 1000) * 475 // 10000)
    ht = max(20160, min(9183480, trunc10(g * 719 // 10000)))
    ct = trunc10(ht * 9448 // 71900)
    assert trunc10(ht // 2) == round10(ht // 2) == c['health_employee_won']
    assert trunc10(ct // 2) == round10(ct // 2) == c['care_employee_won']
    ei = g * 9 // 1000
    insurance = pen + ht // 2 + ct // 2 + ei
    total = insurance + c['income_tax_won'] + c['local_tax_won']
    assert (pen, ht, ct, ei, insurance, total, g - total) == tuple(c[k] for k in (
        'pension_won', 'health_total_won', 'care_total_won', 'employment_won',
        'insurance_won', 'total_deductions_won', 'net_won'))
    assert c['net_won'] >= 0 and c['payable'] is False
# Two concrete refusal arithmetic witnesses; product guard execution remains pending.
care = trunc10(trunc10(3200000 * 719 // 10000) * 9448 // 71900)
assert care == 30230 and trunc10(care // 2) == 15110 and round10(care // 2) == 15120
assert 3000000 - (291520 + 3000000 + 300000) < 0
assert 3000000 < 400 * 10320
class Links(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []
    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            self.links.append(dict(attrs)['href'])
reader = Links()
reader.feed((p / 'review.html').read_text())
for link in reader.links:
    assert (p / link).is_file(), link
for f in p.glob('*.json'):
    json.loads(f.read_text())
assert receipt['whole_design_consensus'] is False and receipt['implementation_authorized'] is False
assert receipt['w04_complete'] is False and receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p / 'record.json').exists():
    record = json.loads((p / 'record.json').read_text())
    assert {f.name for f in p.iterdir() if f.is_file()} == {'record.json'} | {f['path'] for f in record['files']}
    for item in record['files']:
        b = (p / item['path']).read_bytes()
        assert len(b) == item['bytes'] and hashlib.sha256(b).hexdigest() == item['sha256']
print(json.dumps(dict(result='PASS', reviewed_sha=sha, core_files_verified=len(core), source_blobs_verified=len(sources),
    owner_mappings=len(owners), proposed_mutations=len(wire['commands']), arithmetic_positive_cases_verified=2,
    refusal_arithmetic_witnesses_verified=3, refusal_scenarios_discovered=5, reader_links_verified=len(reader.links),
    product_tests_discovered=0, product_tests_executed=0, record_manifest_checked=(p / 'record.json').exists())))
