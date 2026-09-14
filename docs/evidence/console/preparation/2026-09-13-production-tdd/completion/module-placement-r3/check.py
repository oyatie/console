from pathlib import Path
import json,subprocess,tempfile,shutil,copy,argparse
P=Path(__file__).parent
ROOT=None
def regenerate():
 with tempfile.TemporaryDirectory(prefix='console-mount-census-') as tmp:
  d=Path(tmp)
  for name in ('build.py','selection-base.json'):shutil.copyfile(P/name,d/name)
  subprocess.run(['python3',str(d/'build.py'),'--root',str(ROOT)],check=True,capture_output=True,text=True)
  return json.loads((d/'module-placement.json').read_text()),{f.name:f.read_bytes() for f in d.glob('*-declarations.rs')}
def verify(candidate,declarations=None):
 expected,generated=regenerate()
 # Reviewed selection-base + deterministic overlay rules are the roster. The
 # generator rereads all actual selected Rust bytes, refs, includes and hashes.
 assert candidate==expected,'mapping differs from regenerated exact selected source/placement roster'
 actual=declarations if declarations is not None else {name:(P/name).read_bytes() for name in generated}
 assert actual==generated,'declaration bytes differ from exact mount projection'
 assert not candidate['unresolved_crate_references']
 return True
if __name__=='__main__':
 parser=argparse.ArgumentParser();parser.add_argument('--root',required=True);ROOT=Path(parser.parse_args().root).resolve()
 m=json.loads((P/'module-placement.json').read_text());assert verify(m)
 mutations={
 'empty_roster':lambda b:b.update(entries=[]),
 'omit_leaf':lambda b:b['entries'].pop(next(i for i,e in enumerate(b['entries']) if e['source']=='browser-acceptance-producer.rs')),
 'erase_refs':lambda b:[e.update(crate_refs=[]) for e in b['entries']],
 'wrong_parent':lambda b:next(e for e in b['entries'] if e['source']=='migration-remaining-tests.rs').update(parent_module='native_fixture'),
 'erase_declarations':lambda b:[e.update(declaration='') for e in b['entries']],
 'omit_asset':lambda b:b['assets'].pop(),
 'omit_runtime_asset':lambda b:b['runtime_assets'].pop(),
 'wrong_source_hash':lambda b:b['entries'][0].update(source_sha256='0'*64),
 'duplicate_module':lambda b:b['entries'].append(copy.deepcopy(b['entries'][0])),
 }
 for label,mutate in mutations.items():
  bad=copy.deepcopy(m);mutate(bad)
  try:verify(bad)
  except AssertionError:pass
  else:raise AssertionError('hostile mapping accepted: '+label)
 _,decl=regenerate();decl[next(iter(decl))]=b'// omitted declarations'
 try:verify(m,decl)
 except AssertionError:pass
 else:raise AssertionError('hostile declaration file accepted')
 print('1 exact regeneration control +10 hostile controls PASS; source roster/mount mechanics only, not Rust compiler proof')
