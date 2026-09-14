"""Read-only retained packet custody verification; not application coverage."""
from pathlib import Path
import gzip,hashlib,json,stat
ROOT=Path(__file__).resolve().parents[1]
def digest(b):return hashlib.sha256(b).hexdigest()
def regular(root,relative):
 p=Path(relative)
 if p.is_absolute() or '..' in p.parts:raise ValueError('unsafe packet member')
 for part in [root/p,*list((root/p).parents)[:len(p.parts)-1]]:
  if part.is_symlink():raise ValueError('symlink member')
 path=root/p
 if not stat.S_ISREG(path.stat().st_mode):raise ValueError('nonregular member')
 return path.read_bytes()
def verify(folder):
 location=json.loads(regular(folder,'relocation-verification.json'))
 raw=regular(folder,'freeze-manifest.json');assert digest(raw)==(location.get('manifest_sha256') or location.get('source_manifest_sha256') or location['original_manifest_sha256'])
 original=json.loads(raw);assert ('files' in original) != ('members' in original)
 roster=original.get('files',original.get('members'));assert isinstance(roster,list) and roster
 members={e['path']:e for e in roster}
 assert len(members)==len(roster)
 mapping=location.get('files',location.get('mapping'));assert isinstance(mapping,list)
 if any(e.get('original',e.get('source_path'))=='freeze-manifest.json' for e in mapping):
  assert 'freeze-manifest.json' not in members
  members['freeze-manifest.json']={'path':'freeze-manifest.json','sha256':digest(raw),'bytes':len(raw)}
 assert len(mapping)==len(members)
 seen=set()
 for entry in mapping:
  source=entry.get('original',entry.get('source_path'));assert source in members and source not in seen;seen.add(source)
  target=entry.get('retained',entry.get('retained_path'));retained=regular(folder,target)
  if 'retained_sha256' in entry:assert digest(retained)==entry['retained_sha256']
  data=gzip.decompress(retained) if target.endswith('.gz') and not source.endswith('.gz') else retained
  expected=entry.get('original_sha256',entry.get('raw_sha256',entry.get('sha256')))
  assert digest(data)==members[source]['sha256']==expected
  assert len(data)==members[source]['bytes']
 return len(members)
def verify_all(root,roster):
 entries=roster['packets'];assert entries,'empty consolidated roster'
 names=[e['packet'] for e in entries];assert len(names)==len(set(names))
 discovered={p.name for p in root.iterdir() if p.is_dir() and (p/'relocation-verification.json').is_file()}
 assert discovered==set(names),'missing or unregistered custody packet'
 total=0
 for entry in entries:
  name=entry['packet'];assert len(Path(name).parts)==1
  assert digest(regular(root, name+'/freeze-manifest.json'))==entry['manifest_sha256']
  total+=verify(root/name)
 return len(entries),total
if __name__=='__main__':
 roster=json.loads(Path(__file__).with_name('required-packets.json').read_text())
 packets,total=verify_all(ROOT,roster)
 print('PASS',packets,'required packets',total,'retained members; runtime tests executed:0')
