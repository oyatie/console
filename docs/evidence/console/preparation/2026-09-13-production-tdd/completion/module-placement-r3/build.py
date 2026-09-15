from pathlib import Path
import json,re,hashlib,tomllib,argparse
parser=argparse.ArgumentParser();parser.add_argument('--root',required=True);ROOT=Path(parser.parse_args().root).resolve()
C=ROOT/'docs/evidence/console/preparation/2026-09-13-production-tdd/completion'
OUT=Path(__file__).parent
m=json.loads((OUT/'selection-base.json').read_text())
packets={e['source_packet']:C/e['source_packet'] for e in m['entries']}
packets.update({'security-s5':C/'security-s5','browser-r4':C/'browser-r4','native-oracle-s8':C/'native-oracle-s8','migration-remaining-r4':C/'migration-remaining-r4','native-lock-s4':C/'native-assertions-s4'})
assert packets['browser-r4'].exists(),'installed browser-r4 evidence missing'
for e in m['entries']:
 if e['source_packet']=='security-s4':e['source_packet']='security-s5'
 if (packets['native-oracle-s8']/e['source']).exists():e['source_packet']='native-oracle-s8'
 if e['source']=='native-lock-custody-tests.rs':e['source_packet']='native-lock-s4'
 if e['source'] in ['browser-auth-producer.rs','payroll-source-facts-producer.rs','two-subject-producer.rs']:e['source_packet']='browser-r4'
def add(packet,source,mount,roots,parent=None):
 e=dict(source_packet=packet,source=source,target='backend/app/tests/support/native/'+source,mount=mount,integration_roots=roots)
 if parent:e['parent_module']=parent
 m['entries'].append(e)
for name in ['browser-case-producer.rs','browser-runtime-helpers.rs']:add('browser-r4',name,'shared-module',['native_acceptance'])
add('browser-r4','browser-acceptance-producer.rs','test-module',['native_acceptance'])
for name in ['migration-batch-process.rs','bounded-process-output.rs']:add('migration-remaining-r4',name,'shared-module',['account_migration'])
add('migration-remaining-r4','migration-remaining-tests.rs','nested-test-module',['account_migration'],'migration_transition_acceptance')
for name,alias in [('legacy-transport-producer.rs','legacy_transport'),('migrated-primary-producer.rs','migrated_primary')]:
 add('migration-remaining-r4',name,'nested-shared-module',['account_migration'],'migration_transition_acceptance::migration_remaining_tests');m['entries'][-1]['module_alias']=alias
# Top-level crate use-tree branches, plus directly qualified crate references.
def crate_refs(s):
 refs=set(re.findall(r'\bcrate::([A-Za-z_]\w*)',s))
 for match in re.finditer(r'\bcrate::\{',s):
  depth=0;piece=''
  for ch in s[match.end():]:
   if ch=='}' and depth==0:
    if piece.strip():refs.add(re.match(r'\s*(\w+)',piece).group(1))
    break
   if ch==',' and depth==0:
    if piece.strip():refs.add(re.match(r'\s*(\w+)',piece).group(1))
    piece='';continue
   if ch in '{(<[':depth+=1
   if ch in '})>]':depth-=1
   piece+=ch
 return sorted(refs)
entries=m['entries'];by_alias={};assets={};unresolved=[]
for e in entries:
 e['module_alias']=e.get('module_alias',e['source'].replace('.append','').removesuffix('.rs').replace('-','_'))
 p=packets[e['source_packet']]/e['source'];e['source_repository_path']=str(p.relative_to(ROOT));s=p.read_text();e['source_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
 e['crate_refs']=crate_refs(s);e['super_refs']=re.findall(r'\bsuper::[^;]+;',s);e['include_assets']=[]
 e['source_test_names']=re.findall(r'#\[(?:test|(?:sqlx|tokio)::test)(?:\([^]]*\))?\]\s*(?:async\s+)?fn\s+(\w+)',s)
 if e['source']=='browser-acceptance-producer.rs':e['source_test_names']+=re.findall(r'browser_case!\((\w+),',s)
 if 'nested' not in e['mount'] and e['mount']!='owner-local-test-module':by_alias[e['module_alias']]=e
 for rel in sorted(set(re.findall(r'include(?:_str|_bytes)?!\s*\(\s*"([^"]+)"',s))):
  src=p.parent/rel
  if not src.exists():
   candidates=[x/rel for x in packets.values() if (x/rel).exists()]
   assert candidates,(e['source'],rel);assert len({hashlib.sha256(x.read_bytes()).hexdigest() for x in candidates})==1,(e['source'],rel,'ambiguous');src=candidates[0]
  target=str(Path(e['target']).parent/rel);pin=hashlib.sha256(src.read_bytes()).hexdigest()
  assert target not in assets or assets[target]['source_sha256']==pin
  assets[target]={'source_repository_path':str(src.relative_to(ROOT)),'source_sha256':pin,'target':target,'consumers':assets.get(target,{}).get('consumers',[])+[e['module_alias']]};e['include_assets'].append(target)
# Macro exported by negative_assertions is a crate-root name, not separate module.
macro_alias={'assert_registered_refusal':'negative_assertions'}
roots=['native_acceptance','auth_rest','account_migration']
for root in roots:
 required={e['module_alias'] for e in entries if root in e['integration_roots'] and e['mount'] in ['test-module','nested-test-module']}
 changed=True
 while changed:
  changed=False
  for e in entries:
   if e['module_alias'] not in required:continue
   for ref in e['crate_refs']:
    ref=macro_alias.get(ref,ref)
    if ref not in by_alias:unresolved.append({'root':root,'source':e['source'],'reference':ref});continue
    if ref not in required:required.add(ref);changed=True
 for e in entries:
  if e['mount']=='shared-module':
   if root in e['integration_roots']:e['integration_roots'].remove(root)
   if e['module_alias'] in required:e['integration_roots'].append(root)
# Exact source nesting, not duplicate include in every root.
for e in entries:
 if e['mount']=='owner-local-test-module':
  if e['source'] in ['rust-codec-assertions.rs','native-stream-budget-tests.rs']:
   e['package']='console-payroll-adapter-postgres';e['parent_owner']='backend/crates/payroll/adapter-postgres/src/lib.rs';e['declaration']='# [cfg(test)] mod '+Path(e['target']).stem+';';e['nesting_note']='Sibling of native28: source use super::native28 resolves at crate root.'
  elif e['source']=='authority-vector-acceptance.append.rs':
   e['package']='console-platform-request-context';e['parent_owner']=e['target'];e['declaration']='append source directly INSIDE account owner; source already declares cfg(test) child module'
  else:
   e['package']='console-app';e['parent_owner']=e['target'];e['declaration']='append source directly inside existing console_telemetry module; source declares cfg(test) child'
  e['commands']=['cargo test --manifest-path backend/Cargo.toml -p '+e['package']+' --lib '+n+' -- --exact --list' for n in []]
  e['commands']=[{'discover':'cargo test --manifest-path backend/Cargo.toml -p '+e['package']+' --lib '+n+' -- --list','execute':'cargo test --manifest-path backend/Cargo.toml -p '+e['package']+' --lib '+n+' -- --nocapture','required_leaf':n} for n in e['source_test_names']]
 else:
  e['package']='console-app'
  e['commands']=[{'discover':f'cargo test --manifest-path backend/Cargo.toml -p console-app --test {r} {e["module_alias"]} -- --list','execute':f'cargo test --manifest-path backend/Cargo.toml -p console-app --test {r} {e["module_alias"]} -- --test-threads=1 --nocapture'} for r in e['integration_roots']] if 'test-module' in e['mount'] else []
  if 'nested' not in e['mount']:e['declaration']=f'#[path = "support/native/{e["source"]}"] mod {e["module_alias"]};'
  else:e['declaration']='Use existing #[path] child declarations' if e['mount']=='nested-shared-module' else '#[path = "migration-remaining-tests.rs"] mod migration_remaining_tests; inside migration_transition_acceptance'
# Known root-private imports require these exact parents and must not be reused elsewhere.
private_roots={'context-owner-fixture.rs':'auth_rest','terms-publication-acceptance.rs':'auth_rest','publication-boundary-acceptance.rs':'auth_rest','publication-completion-tests.rs':'auth_rest','migration-transition-acceptance.rs':'account_migration','migration-rollback-tests.rs':'account_migration'}
for e in entries:
 if e['source'] in private_roots:
  assert e['integration_roots']==[private_roots[e['source']]],e
  e['root_private_contract']=private_roots[e['source']]+' existing private helper scope; mount directly as child of integration root'
runtime_assets=[]
for name,kind in [('history_oracle.py','runtime-subprocess'),('test_history_oracle.py','independent-checker-tests')]:
 source=C/'draft-r3'/name
 runtime_assets.append(dict(source_repository_path=str(source.relative_to(ROOT)),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),target='backend/app/tests/support/draft/'+name,classification=kind,consumers=['draft_owner_acceptance']))
for source in sorted((C/'browser-r4/playwright').glob('*.cjs')):
 runtime_assets.append(dict(source_repository_path=str(source.relative_to(ROOT)),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),target='backend/app/tests/browser/'+source.name,classification='playwright-runtime-source',consumers=['browser_acceptance_producer']))
m['runtime_assets']=runtime_assets
m['runtime_commands']=[dict(command='python3 -B -m unittest -v test_history_oracle',cwd='backend/app/tests/support/draft'),dict(command='node /actual/pinned/playwright/cli.js test --config backend/app/tests/browser/playwright.config.cjs --list',note='15 application leaves plus10 oracle leaves; actual fixture owner supplies case-specific env at execution')]
m['status']='MECHANICAL_PLACEMENT_CANDIDATE_NOT_ADMISSION';m['assets']=list(assets.values());m['unresolved_crate_references']=unresolved
m['pending_successors']=[]
m['authority_hold']='current_controls shared reason/control compiler is a pending consolidated design amendment, not already approved authority'
m['rules']=[x.replace('shared-module contains no test bodies; emit #[path] declarations at each named integration root','shared-module has no workflow tests; bounded_process_output helper tests are the explicit exception mounted once under account_migration') for x in m['rules']]
for e in entries:
 e['contains_inline_tests']=bool(e['source_test_names'])
 if e['source']=='bounded-process-output.rs':e['classification_note']='Helper machinery tests intentionally discovered once under account_migration, not domain acceptance cases.'
(OUT/'module-placement.json').write_text(json.dumps(m,indent=2)+'\n')
for root in roots:
 declarations=[e['declaration'] for e in entries if root in e['integration_roots'] and e['mount'] in ['shared-module','test-module']]
 (OUT/(root+'-declarations.rs')).write_text('// Proposed integration-root declarations; do not replace existing test root.\n'+'\n'.join(declarations)+'\n')
print('entries',len(entries),'assets',len(assets),'unresolved crate refs',unresolved)
