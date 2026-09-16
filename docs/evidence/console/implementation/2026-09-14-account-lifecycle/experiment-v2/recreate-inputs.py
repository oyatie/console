"""Recreate source-bound v2 inputs from archived source files plus exact Git blobs."""
from pathlib import Path
import hashlib,json,shutil,subprocess,sys
archive,repo,destination=map(Path,sys.argv[1:])
assert not destination.exists(), 'destination must be fresh'
manifest=json.loads((archive/'sources.json').read_text())
destination.mkdir();(destination/'migrations').mkdir()
for name,digest in manifest['source_files'].items():
 data=(archive/name).read_bytes();assert hashlib.sha256(data).hexdigest()==digest
 (destination/name).write_bytes(data)
for name,digest in manifest['migration_files'].items():
 data=subprocess.check_output(['git','show',manifest['base_sha']+':backend/crates/platform/db/migrations/'+name],cwd=repo)
 assert hashlib.sha256(data).hexdigest()==digest
 (destination/'migrations'/name).write_bytes(data)
shutil.copyfile(archive/'sources.json',destination/'sources.json')
print(destination)
