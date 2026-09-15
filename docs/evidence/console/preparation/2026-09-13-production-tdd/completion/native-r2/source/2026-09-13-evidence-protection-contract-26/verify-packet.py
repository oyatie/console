#!/usr/bin/env python3
"""Static regular-blob custody, schema-site census and accounting; no runtime semantic proof."""
import argparse,csv,hashlib,json,re,subprocess,collections
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument('--commit',required=True);a=ap.parse_args()
root=Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip());p=Path(__file__).resolve().parent;rel=p.relative_to(root).as_posix()
def git(*args):return subprocess.check_output(['git',*args],cwd=root)
def blob(sha,path):
 row=git('ls-tree',sha,'--',path).decode().split();assert row[:2]==['100644','blob'],path
 return row[2],git('cat-file','blob',row[2])
sha=git('rev-parse',a.commit).decode().strip()
names=['entry.txt','schema-reference-sites.json','reference-rules.json','dependency-templates.json','action-inventory.json','budgets.json','contract.txt','lifecycle.csv','proof-cases.csv','choices.txt','readiness.json','source-identities.json','scope.json','review.html','verify-packet.py'];b={}
for n in names:
 _,raw=blob(sha,rel+'/'+n);assert raw==(p/n).read_bytes(),n;b[n]=raw
j=lambda n:json.loads(b[n]);scope=j('scope.json');ready=j('readiness.json');sources=j('source-identities.json')['sources'];sb={}
assert len(sources)==scope['source_blobs']==16
assert len({(s['commit'],s['path']) for s in sources})==len(sources)
for s in sources:
 oid,raw=blob(s['commit'],s['path']);assert oid==s['git_blob'] and hashlib.sha256(raw).hexdigest()==s['sha256'];sb[s['path']]=raw
for x in [scope,ready]:
 assert all(x[k] is False for k in ['whole_design_approved','w04_complete','implementation_authorized'])
 assert x['product_tests_executed']==x['verus_proofs_executed']==0 and x['sql_executed'] is False
assert scope['product_tests_written']==scope['product_tests_discovered']==0
source=json.loads(sb['docs/evidence/console/design/2026-09-12-source-payload-contract-19/source-types.schema.json']);census=j('schema-reference-sites.json');rules=j('reference-rules.json');expected={}
def esc(s):return s.replace('~','~0').replace('/','~1')
def walk(x,parts):
 if isinstance(x,dict):
  for k,v in x.items():
   if k=='$ref':
    assert v.startswith('#/$defs/');expected['/'+('/'.join(esc(y) for y in parts+[k]))]=(parts[1] if len(parts)>1 and parts[0]=='$defs' else 'ROOT',v.removeprefix('#/$defs/'))
   else:walk(v,parts+[k])
 elif isinstance(x,list):
  for i,v in enumerate(x):walk(v,parts+[str(i)])
walk(source,[])
assert len(source['$defs'])==census['definition_count']==scope['schema_definitions']==89
actual={s['pointer']:(s['enclosing_definition'],s['target_definition']) for s in census['sites']}
assert actual==expected and len(actual)==len(census['sites'])==scope['schema_reference_sites']==census['site_count']==345
assert len(rules['atomic_rules'])==scope['atomic_reference_rules']==21
assert len(rules['raw_identity_rules'])==scope['unwrapped_identity_rules']==23
seenraw=set()
for s in census['sites']:
 target=s['target_definition'];rawrule=s['raw_identity_rule']
 assert s['owner_or_handler']
 if rawrule:
  assert target=='UUID' and rawrule in rules['raw_identity_rules'];seenraw.add(rawrule)
  assert rules['raw_identity_rules'][rawrule]['classification']==s['classification']
 elif target in rules['atomic_rules']:assert rules['atomic_rules'][target]['classification']==s['classification']
 elif target in rules['carrier_rules']:assert rules['carrier_rules'][target]==s['classification']
 elif target=='UUID':assert s['classification']=='REFERENCE_COMPONENT'
 else:assert s['classification'] in ['VALUE','DESCEND_TYPED_SCHEMA']
assert seenraw==set(rules['raw_identity_rules'])
templates=j('dependency-templates.json');ids={t['id'] for t in templates['templates']}
assert len(ids)==len(templates['templates'])==scope['dependency_templates']==14
assert set(templates['outputs'])=={'RETAIN_EXACT','RECHECK_CURRENT','QUERY_WITNESS','CONTROL_LINEAGE'}
actions=j('action-inventory.json');assert len(actions['actions'])==scope['action_sites']==14 and len(actions['family_overlays'])==scope['family_overlays']==7
assert len({x['action_site'] for x in actions['actions']})==14
for x in actions['actions']+actions['family_overlays']:assert set(x['templates'])<=ids and len(x['templates'])==len(set(x['templates']))
bud=j('budgets.json');l=bud['limits'];h=bud['inherited19'];w=bud['arithmetic_witnesses']
assert all(isinstance(v,int) and v>0 for v in l.values())
assert w['worst_event_plus_day_bytes']==2000*(h['event_package_bytes']+h['day_package_bytes'])==88080384000
assert l['fresh_external_bytes'] < w['worst_event_plus_day_bytes'] < l['covered_bytes_upper_bound']
assert w['worst_event_rows']==2000*h['event_rows']==8194000 and w['worst_day_rows']==2000*h['day_rows']==62000
assert w['representative_root_units']==2000*3+256+128==6384 < l['distinct_unit_locks'] <= l['total_distinct_locks']
assert w['all_pilot_workloads_proven'] is False and w['worst_case_fresh_read_admitted'] is False
assert (h['direct_trust_refs'],h['distinct_trust_nodes'],h['qualification_edges_depth'])==(32,256,3)
for n,k,c in [('lifecycle.csv','lifecycle_sites',16),('proof-cases.csv','nonexecuted_cases',29)]:
 rows=list(csv.DictReader(b[n].decode().splitlines()));assert len(rows)==scope[k]==c
 if n=='proof-cases.csv':assert len({r['id'] for r in rows})==c and all(r['status']=='PROPOSED_NOT_EXECUTED' for r in rows)
 else:assert len({r['site'] for r in rows})==c
links=re.findall(r'href="([^"#]+)"',b['review.html'].decode());assert all(x in names for x in links)
print(json.dumps({'candidate_sha':sha,'core_files':len(names),'source_blobs':len(sources),'schema_definitions':89,'schema_reference_sites':345,'atomic_rules':21,'raw_identity_rules':23,'dependency_templates':14,'action_sites':14,'family_overlays':7,'lifecycle_sites':16,'nonexecuted_cases':29,'reader_links':len(links),'worst_event_plus_day_bytes':w['worst_event_plus_day_bytes'],'validation_scope':'Static regular-blob/source identity, complete schema $ref census, classification references and bound arithmetic only. No semantic extractor, SQL parse/catalog/runtime, workload feasibility or Verus proof.','product_tests_discovered':0,'product_tests_executed':0,'sql_executed':False,'verus_proofs_executed':0,'implementation_authorized':False},indent=2))
