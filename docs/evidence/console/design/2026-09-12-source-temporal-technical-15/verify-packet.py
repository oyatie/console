"""Evidence structure, custody and synthetic encoding checks; no product runtime tests."""
import csv, datetime as dt, hashlib, json, re, subprocess, sys, uuid
from html.parser import HTMLParser
from pathlib import Path
p = Path(__file__).resolve().parent
repo = next(x for x in p.parents if (x / '.git').exists())
rel = p.relative_to(repo)
def read(name):
    def pairs(items):
        result = {}
        for k, v in items:
            assert k not in result, (name, 'duplicate', k)
            result[k] = v
        return result
    return json.loads((p / name).read_text(), object_pairs_hook=pairs)
receipt = read('review-receipt.json') if (p / 'review-receipt.json').exists() else None
sha = sys.argv[1] if len(sys.argv) == 2 else receipt['reviewed_candidate_sha']
core = ['entry.txt','evidence-assessment-contract.txt','historical-attendance-contract.txt','integration-and-readiness.txt','np1-command-vectors.json','proof-cases.csv','review.html','schema-candidate.json','scope.json','source-actions.json','source-guard-contract.txt','source-identities.json','synthetic-tax-source.csv','temporal-witnesses.json','wire-types.json','writer-enrollment.csv']
def git(*args): return subprocess.check_output(['git', *args], cwd=repo)
for name in core:
    assert (p / name).read_bytes() == git('show', f'{sha}:{rel/name}'), name
    assert git('ls-tree', sha, str(rel/name)).startswith(b'100644 blob '), name
sources = read('source-identities.json')['sources']
for s in sources:
    spec = f"{s['commit']}:{s['path']}"
    assert git('ls-tree', s['commit'], s['path']).startswith(b'100644 blob ')
    assert git('rev-parse', spec).decode().strip() == s['git_blob']
    assert hashlib.sha256(git('show', spec)).hexdigest() == s['sha256']
scope, schema, wire = read('scope.json'), read('schema-candidate.json'), read('wire-types.json')
assert len(sources) == scope['source_blobs'] == 27
assert len(schema['tables']) == len({t['name'] for t in schema['tables']}) == scope['proposed_tables'] == 15
assert len(schema['source_families']) == len(set(schema['source_families'])) == scope['source_families'] == 10
for t in schema['tables']:
    assert set(t['primary_key']) <= set(t['columns']) and t['constraints'] and t['access'] and t['lock']
def rows(name):
    with (p / name).open() as f: result = list(csv.DictReader(f))
    assert all(None not in r and all(r.values()) for r in result), name
    return result
cases, writers = rows('proof-cases.csv'), rows('writer-enrollment.csv')
assert len(cases) == len({x['id'] for x in cases}) == scope['nonexecutable_proof_cases'] == 47
assert all(x['status'] == 'NONEXECUTABLE_CANDIDATE' for x in cases)
assert len(writers) == len({x['id'] for x in writers}) == scope['writer_surfaces'] == 14
assert {x['source_path'] for x in writers} <= {s['path'] for s in sources}
actions = read('source-actions.json')['actions']
assert len(actions) == scope['source_actions'] == 12
assert len(wire['definitions']) == scope['source_definitions'] == 22
commands = {c['id']: c for c in wire['commands']}
assert len(commands) == len(wire['commands']) == scope['np1_commands'] == 10
def restricted(v, depth=0):
    assert depth <= 8
    if isinstance(v, dict):
        assert all(isinstance(k, str) and k.isascii() for k in v)
        for x in v.values(): restricted(x, depth+1)
    elif isinstance(v, list):
        for x in v: restricted(x, depth+1)
    else:
        assert v is None or isinstance(v, (str, bool)), 'JSON numbers forbidden'
        if isinstance(v, str):
            assert chr(0) not in v
            v.encode('utf8')
def shape(value, definition):
    if isinstance(definition, dict):
        if 'tagged_union' in definition:
            assert value['kind'] in definition['tagged_union']
            return shape({k:v for k,v in value.items() if k!='kind'}, definition['tagged_union'][value['kind']])
        assert isinstance(value, dict) and set(value) == set(definition), (value, definition)
        for k, d in definition.items(): shape(value[k], d)
        return
    d = definition
    if d in wire['definitions']: return shape(value, wire['definitions'][d])
    m = re.fullmatch(r'(.+)\[(\d+)\.\.(\d+)\]', d)
    if m:
        assert isinstance(value, list) and int(m[2]) <= len(value) <= int(m[3])
        for x in value: shape(x, m[1])
        return
    if '|' in d:
        if d.endswith('|null'):
            if value is None: return
            return shape(value, d[:-5])
        assert value in d.split('|'); return
    if d == 'null': assert value is None; return
    if d == 'true': assert value is True; return
    assert isinstance(value, str), (d, value)
    if d == 'UUID': assert str(uuid.UUID(value)) == value and uuid.UUID(value).int != 0
    elif d == 'Sha256': assert re.fullmatch('[0-9a-f]{64}', value)
    elif d == 'Date': assert dt.date.fromisoformat(value).isoformat() == value
    elif d in ['PositiveRevision','RevisionOrZero','Won','NonnegativeWon','PositiveWon','UnixMicroseconds']:
        assert re.fullmatch(r'0|-?[1-9][0-9]*', value) and -(2**63) <= int(value) < 2**63
        if d in ['PositiveRevision','PositiveWon']: assert int(value) > 0
        if d in ['RevisionOrZero','NonnegativeWon']: assert int(value) >= 0
    elif d.startswith('IntegerString'):
        lo, hi = map(int, d.removeprefix('IntegerString').split('..'))
        assert re.fullmatch(r'0|[1-9][0-9]*', value) and lo <= int(value) <= hi
    elif d == 'SourceFamily': assert value in schema['source_families']
    elif d == 'RegisteredAsciiId': assert re.fullmatch('[A-Za-z][A-Za-z0-9_]{0,127}', value)
    elif d.startswith('Text'):
        if d != 'Text':
            lo, hi = map(int, d.removeprefix('Text').split('..')); assert lo <= len(value) <= hi
    else: raise AssertionError(('Unhandled evidence shape', d))
vectors = read('np1-command-vectors.json')
assert len(vectors['vectors']) == scope['encoding_witnesses'] == 10
assert {v['id'] for v in vectors['vectors']} == set(commands)
for v in vectors['vectors']:
    e = v['normalized_envelope']; restricted(e)
    raw = json.dumps(e, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    assert len(raw) <= 65536 and raw.decode() == v['canonical_utf8']
    preimage = b'console.command.ccf1\0' + raw
    assert preimage.hex() == v['preimage_hex'] and hashlib.sha256(preimage).hexdigest() == v['sha256']
    c = commands[v['id']]
    assert e['owner_action'] == c['action'] and e['codec_version'] == 'source15-v1'
    shape(e['body'], c['body']); shape(e['expected'], c['expected'])
    if v['id'] in ['CORRECT_WAGE','REVOKE_SOURCE','REOPEN_INPUTS']:
        assert isinstance(e['reason'], str) and e['reason'].strip() and len(e['reason']) <= 2000
artifact = vectors['source_artifact']
assert artifact['classification'] == 'NONPRODUCTION_FIXTURE'
assert hashlib.sha256((p / artifact['path']).read_bytes()).hexdigest() == artifact['sha256']
assert len(rows(artifact['path'])) == 2
temporal = read('temporal-witnesses.json'); w = temporal['earning_window']; ts = temporal['witnesses']
assert len(ts) == scope['temporal_witnesses'] == 6
start = dt.datetime.fromisoformat(w['start_local']); end = dt.datetime.fromisoformat(w['end_exclusive_local'])
assert start == dt.datetime.fromisoformat(w['start_utc']) and end == dt.datetime.fromisoformat(w['end_exclusive_utc'])
assert int(start.timestamp())*1000000 == int(w['start_us']) and int(end.timestamp())*1000000 == int(w['end_us'])
assert ts[0]['employment_start_us'] == w['start_us'] and ts[0]['employment_end_us'] == w['end_us']
in_at, out_at = map(dt.datetime.fromisoformat, [ts[1]['clock_in'], ts[1]['clock_out']])
assert int((out_at-in_at).total_seconds()) == int(ts[1]['expected_full_seconds'])
assert int((min(out_at,end)-max(in_at,start)).total_seconds()) == int(ts[1]['expected_earning_seconds'])
assert dt.date.fromisoformat(ts[2]['new_base_effective']) < dt.date.fromisoformat(ts[2]['selected_span_end_exclusive'])
assert dt.date.fromisoformat(ts[2]['pay_date'])+dt.timedelta(days=1) == dt.date.fromisoformat(ts[2]['selected_span_end_exclusive'])
assert start < dt.datetime.fromisoformat(ts[3]['split_at']) < end and ts[3]['employment_count'] == '1' and ts[3]['segment_count'] == '2'
assert int(ts[4]['new_family_generation']) > int(ts[4]['old_family_generation']) and ts[4]['same_exact_unit_basis'] is True
assert int(ts[5]['required_duty_count']) > 0 and ts[5]['clock_events'] == '0' and ts[5]['attest'] is True
class Links(HTMLParser):
    def __init__(self): super().__init__(); self.links = []
    def handle_starttag(self, tag, attrs):
        if tag == 'a': self.links.append(dict(attrs)['href'])
links = Links(); links.feed((p / 'review.html').read_text())
for href in links.links: assert (p / href).is_file(), href
for f in p.glob('*.json'): read(f.name)
assert not scope['implementation_authorized'] and not scope['whole_design_approved'] and not scope['w04_complete']
if receipt:
    assert receipt['reviewed_candidate_sha'] == sha
    assert not receipt['implementation_authorized'] and not receipt['whole_design_consensus'] and not receipt['w04_complete']
    assert receipt['design_sha_approval'] is None and receipt['test_sha_approval'] is None
if (p / 'record.json').exists():
    record = read('record.json')
    assert {x.name for x in p.iterdir()} == {'record.json'} | {x['path'] for x in record['files']}
    for item in record['files']:
        b = (p / item['path']).read_bytes()
        assert len(b) == item['bytes'] and hashlib.sha256(b).hexdigest() == item['sha256']
    assert hashlib.sha256((p / record['predecessor_record']).read_bytes()).hexdigest() == record['predecessor_record_sha256']
print(json.dumps(dict(result='PASS', reviewed_sha=sha, core_files_verified=len(core), source_blobs_verified=len(sources), proposed_tables=15, source_definitions=22, source_actions=12, writer_surfaces=14, nonexecutable_cases_discovered=47, encoding_witnesses_verified=10, temporal_witness_consistency_checked=6, synthetic_csv_rows=2, reader_links_verified=len(links.links), product_tests_discovered=0, product_tests_executed=0, record_manifest_checked=(p/'record.json').exists())))
