#!/usr/bin/env python3
"""Private source/patch checks only. Never invokes Cargo or a database."""
from pathlib import Path
import ast
import difflib
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile

PACKAGE = Path(__file__).resolve().parent
REFERENCE = Path('/private/tmp/console-production-tdd-preparation-20260913')
BASE = PACKAGE / 'base'
POST = PACKAGE / 'post'
PREIMAGES = json.loads((PACKAGE / 'preimages.json').read_text())
sha = lambda data: hashlib.sha256(data).hexdigest()
checks = []


def check(name, condition):
    assert condition, name
    checks.append(name)


def command(argv, **kwargs):
    result = subprocess.run(argv, capture_output=True, **kwargs)
    check(' '.join(argv), result.returncode == 0)
    return result.stdout


check('exact four base preimages', all(sha((BASE / p).read_bytes()) == h for p, h in PREIMAGES.items()))
check('four integration preimages unchanged', all(sha((REFERENCE / p).read_bytes()) == h for p, h in PREIMAGES.items()))
check('exact four postimage paths', sorted(str(p.relative_to(POST)) for p in POST.rglob('*') if p.is_file()) == sorted(PREIMAGES))
test_path = REFERENCE / 'backend/crates/identity/adapter-postgres/tests/deactivate_revokes_credentials.rs'
check('reviewed14 source unchanged', sha(test_path.read_bytes()) == 'e06b77f8841cf3e7f5bc595756c1b8933c136e9cbfae1a536fac2862375dc471')
rust = POST / 'backend/crates/identity/adapter-postgres/src/lib.rs'
formatted = command(['rustfmt', '--edition', '2024'], input=rust.read_bytes())
check('Rust stdin format exact', formatted == rust.read_bytes())


def constants(path):
    return {n.targets[0].id: ast.literal_eval(n.value)
            for n in ast.parse(path.read_text()).body
            if isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name)
            and isinstance(n.value, ast.Constant)}


old = constants(BASE / 'ops/generate-account-custody.py')
new = constants(POST / 'ops/generate-account-custody.py')
retained = ['FENCE_BODY', 'GUARD_BODY', 'CURRENT_BODY', 'ROOT_GUARD_BODY',
            'ROOT_BRIDGE_BODY', 'USER_ID_GUARD_BODY', 'INSTALL', 'ROOT_INSTALL']
check('six old bodies and two old install blocks exact', all(old[name] == new[name] for name in retained))
source = POST / 'ops/generate-account-custody.py'
namespace = {'__file__': str(source), '__name__': 'private_check'}
exec(compile(source.read_text(), str(source), 'exec'), namespace)


class Inputs:
    """Read immutable migration references without copying or writing them."""
    def __truediv__(self, path):
        reference = path in ['backend/crates/platform/db/migrations', 'ops/account-custody-migrations.sha384']
        return (REFERENCE if reference else POST) / path


namespace['ROOT'] = Inputs()
generated = namespace['generated_files']()
check('private finalizer generation exact', generated['ops/postgres-finalize-account-custody.sql'].encode() == (POST / 'ops/postgres-finalize-account-custody.sql').read_bytes())
check('migration ledger unchanged', generated['ops/account-custody-migrations.sha384'].encode() == (REFERENCE / 'ops/account-custody-migrations.sha384').read_bytes())
sys.argv = ['ops/generate-account-custody.py', '--check']
namespace['main']()
check('existing generator --check via private read references', True)
patch = ''.join(''.join(difflib.unified_diff((BASE / path).read_text().splitlines(True),
    (POST / path).read_text().splitlines(True), fromfile='a/' + path, tofile='b/' + path))
    for path in PREIMAGES)
check('candidate patch exact', patch.encode() == (PACKAGE / 'candidate.patch').read_bytes())
temporary = Path(tempfile.mkdtemp(prefix='account-deactivation14-private-patch-check-', dir='/private/tmp'))
shutil.copytree(BASE, temporary / 'tree')
for flags in [['--check'], [], ['--reverse', '--check'], ['--reverse']]:
    command(['git', 'apply', *flags, str(PACKAGE / 'candidate.patch')], cwd=temporary / 'tree')
    if not flags:
        check('applied postimages exact', all((temporary / 'tree' / p).read_bytes() == (POST / p).read_bytes() for p in PREIMAGES))
check('inverse restores exact preimages', all((temporary / 'tree' / p).read_bytes() == (BASE / p).read_bytes() for p in PREIMAGES))
check('no patch whitespace errors', all(not line[1:].rstrip('\n').endswith((' ', '\t')) for line in patch.splitlines(True) if line.startswith('+') and not line.startswith('+++')))
print(json.dumps({'status': 'PASS', 'checks': checks, 'check_count': len(checks),
    'rustfmt_invocations': 1, 'private_git_apply_invocations': 4,
    'compiler_discovered': 0, 'rust_executed': 0, 'cargo_invocations': 0,
    'database_commands': 0, 'integration_writes': 0,
    'patch_check_directory': str(temporary)}, indent=2))
