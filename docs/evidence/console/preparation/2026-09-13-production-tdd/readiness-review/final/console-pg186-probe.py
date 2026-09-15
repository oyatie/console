from pathlib import Path
import re,subprocess,sys
expected="postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280"
paths=[("tools/lanes/pgtest.sh",r'^image="([^"\n]+)"'),("tools/ci/cargo_needs_postgres.sh",r'^postgres_image="([^"\n]+)"'),("tools/buck/test_needs_postgres.sh",r'^postgres_image="([^"\n]+)"'),("backend/ci/gates/writer-ownership/tests/census_executes_against_postgres.rs",r'const IMAGE: &str =\s*"([^"\n]+)";')]
images=set();failed=False
for name,pattern in paths:
 matches=re.findall(pattern,Path(name).read_text(),re.M)
 if len(matches)!=1:raise RuntimeError(f"ambiguous or absent active pin: {name}")
 images.add(matches[0]);ok=matches[0]==expected;failed|=not ok
 print(f"PIN {'PASS' if ok else 'FAIL'} {name}: {matches[0]}",flush=True)
for image in sorted(images):
 result=subprocess.run(['docker','run','--rm','--network','none',image,'postgres','--version'],capture_output=True,text=True,timeout=60)
 if result.returncode:raise RuntimeError(result.stderr)
 print('ACTUAL_BINARY '+result.stdout.strip(),flush=True)
 ok=bool(re.fullmatch(r'postgres \(PostgreSQL\) 18\.6(?: \([^\n]*\))?\n?',result.stdout));failed|=not ok
print('Classification: fixed-version dependency prerequisite; not a CVE reproducer or application authorization test.',flush=True)
sys.exit(1 if failed else 0)
