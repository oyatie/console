#!/usr/bin/env python3
"""Evidence consistency only. Does not run product, SQL, authorization or load tests."""
import argparse, csv, hashlib, json, re, subprocess
from pathlib import Path
ap=argparse.ArgumentParser(); ap.add_argument('--commit', required=True); args=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip())
p=Path(__file__).resolve().parent; rel=p.relative_to(root).as_posix()
def git(*args): return subprocess.check_output(['git',*args],cwd=root)
def blob(commit,path):
    row=git('ls-tree',commit,'--',path).decode().split()
    assert row[:2]==['100644','blob'], (commit,path,row)
    return row[2],git('show',commit+':'+path)
def read(name):
    _,b=blob(args.commit,rel+'/'+name)
    assert (p/name).read_bytes()==b, 'working copy differs: '+name
    return b
core=['entry.txt','integration-contract.txt','roles-and-migration.txt','retention-and-recovery.txt','workload-and-proof.txt','readiness.txt','storage-catalog.json','constraints.csv','retention-records.json','closure-register.json','workloads.json','proof-cases.csv','source-identities.json','scope.json','review.html','verify-packet.py']
contents={name:read(name) for name in core}
def j(name):return json.loads(contents[name])
scope=j('scope.json'); assert all(scope[k] is False for k in ['w04_complete','whole_design_approved','implementation_authorized'])
assert scope['product_tests_executed']==scope['product_tests_written']==0
src=j('source-identities.json')['sources']; seen=set()
for x in src:
    key=(x['commit'],x['path']); assert key not in seen; seen.add(key)
    oid,b=blob(*key); assert oid==x['git_blob'] and hashlib.sha256(b).hexdigest()==x['sha256']
assert len(src)==scope['source_blobs']
cat=j('storage-catalog.json')['records']; assert len(cat)==len({x['name'] for x in cat})==71
for x in cat:
    assert (x['source_commit'],x['source']) in seen
    assert set(x['proposed_key'])<=set(x['column_names'])
    if x['cell']=='COMPANY': assert 'org_id' in x['proposed_key']
assert not {'payroll_runs','employee_contract_wage_heads'}&{x['name'] for x in cat}
cs=list(csv.DictReader(contents['constraints.csv'].decode().splitlines())); assert len(cs)==26 and len({x['id'] for x in cs})==26
cl=j('closure-register.json')['items']; assert [x['id'] for x in cl]==['F%02d'%i for i in range(1,9)]
assert all(x['status']=='OPEN' and x['required_artifacts'] and x['closure_rule'] for x in cl)
r=j('retention-records.json'); assert len(r['common_records'])==9 and len(r['admission_records'])==2
assert len({x['name'] for x in cat+r['common_records']+r['admission_records']})==82
w=j('workloads.json'); assert w['status']=='UNMEASURED_TEST_ACCEPTANCE_CANDIDATE'
assert sum(w['fixtures']['balanced'])==sum(w['fixtures']['skew'])==2000
assert w['fixtures']['monthly_subject_units']==2000*12
assert w['fixtures']['duty_events']==2000*22*2*12
e=w['editing']; rev=e['editors']*e['typing_duty_fraction']/e['save_interval_seconds']*e['hours_per_day']*3600*e['days']
assert rev==e['revisions']==633600
assert e['raw_payload_bytes']==rev*e['payload_bytes_average']; assert e['boundary_raw_payload_bytes']==rev*e['payload_bytes_boundary']
b=w['pool_budget']; assert sum(b['per_serving_replica'].values())==20
assert 20*b['max_replicas_including_surge']+sum(b[k] for k in ['additional_workers','migrator','monitoring','operators','reserve'])==b['max_connections']==60
assert w['existing_migration_0167_settings']['transaction_timeout_ms']==45000
assert b'transaction_timeout45s' in contents['workload-and-proof.txt']
proof=list(csv.DictReader(contents['proof-cases.csv'].decode().splitlines())); assert len(proof)==scope['nonexecutable_proof_cases']==35
assert len({x['id'] for x in proof})==35 and all(x['status']=='PROPOSED_NOT_EXECUTED' for x in proof)
assert all(x['contract'] in {a['id'] for a in cs+cl} for x in proof)
links=re.findall(r'href="([^"#]+)"',contents['review.html'].decode()); assert len(links)==14
assert all(x in core for x in links)
print(json.dumps({'candidate_sha':args.commit,'evidence_core_files':len(core),'source_blobs':len(src),'conceptual_records':len(cat),'shared_custody_records':9,'admission_records':2,'constraints':26,'open_design_items':8,'nonexecutable_proof_cases':len(proof),'reader_links':len(links),'workload_arithmetic':'pass','product_tests_discovered':0,'product_tests_executed':0,'w04_complete':False},indent=2))
