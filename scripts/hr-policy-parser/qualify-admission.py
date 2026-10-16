"""Qualify the exact reviewed HR invocation; never launches fanout or an implementer."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
FIXED_COMMIT = '32e7e921180b6d571e2b60c0017528a4c243c461'
FIXED_SHA = 'ef8534f14c7c6c0f691b3883b21a4e95e566a069f2241d9f1e2de64f502ad4b4'
PROBE = 'scripts/run-native-hr-policy-parser-contract.sh'
LANE = 'native-hr-policy-parser-v4'

def regular(file):
    p = Path(file)
    if p.is_symlink() or not p.is_file():
        raise ValueError('not a regular file')
    return p.read_bytes()

def pin(file, expected):
    if not isinstance(expected, str) or len(expected) != 64 or hashlib.sha256(regular(file)).hexdigest() != expected:
        raise ValueError('pin mismatch')

def qualify_spec(spec):
    if not isinstance(spec, list) or len(spec) != 1 or not isinstance(spec[0], dict):
        raise ValueError('HR spec must contain exactly one lane')
    item = spec[0]
    if item.get('lane') != LANE or item.get('probe') != PROBE or type(item.get('red_exit')) is not int or item['red_exit'] != 101:
        raise ValueError('HR requires the frozen lane/probe and explicit builtin integer101')
    return item

def qualify_invocation(binding, argv, cwd):
    qualify_spec(binding['spec_value'])
    expected = [binding['python']['path'], binding['fixed_tool']['path'], 'admit', '--spec', binding['spec']['path']]
    if argv != expected or str(cwd) != binding['cwd'] or binding['cwd'] != str(ROOT):
        raise ValueError('HR invocation/cwd mismatch; default, --probe and borrowed run are forbidden')
    if binding['fixed_tool']['commit'] != FIXED_COMMIT or binding['fixed_tool']['sha256'] != FIXED_SHA:
        raise ValueError('wrong shared admission owner')
    return expected

def qualify(binding_file, binding_sha, argv, cwd):
    if not Path(binding_file).is_absolute():
        raise ValueError('binding must be an exact absolute path')
    pin(binding_file, binding_sha)
    b = json.loads(regular(binding_file))
    if b.get('schema') != 'console.hr-policy-parser.operational-binding.v1' or b.get('status') != 'REVIEWED_EXACT_TEST_CANDIDATE':
        raise ValueError('binding is not an independently reviewed exact candidate')
    qualify_invocation(b, argv, cwd)
    if Path(sys.executable).resolve() != Path(b['python']['path']):
        raise ValueError('qualifier Python is not pinned')
    for name in ['python', 'node', 'git', 'dotslash', 'buck_binary']:
        tool = b[name]
        p = Path(tool['path'])
        if not p.is_absolute() or p.resolve() != p or not os.access(p, os.X_OK):
            raise ValueError('tool path is not canonical executable')
        pin(p, tool['sha256'])
    for field in ['fixed_tool', 'spec']:
        if not Path(b[field]['path']).is_absolute():
            raise ValueError('fixed tool/spec path must be absolute')
    pin(b['fixed_tool']['path'], FIXED_SHA)
    pin(b['spec']['path'], b['spec']['sha256'])
    spec = json.loads(regular(b['spec']['path']))
    qualify_spec(spec)
    if spec != b['spec_value']:
        raise ValueError('spec content differs from reviewed binding')
    required = {PROBE, 'scripts/check-native-hr-policy-parser-contract.mjs', 'scripts/check-native-hr-policy-parser-contract.test.mjs', 'scripts/hr-policy-parser/binding.mjs', 'scripts/hr-policy-parser/native-run.mjs', 'scripts/hr-policy-parser/observations.mjs', 'scripts/hr-policy-parser/qualify-admission.py', 'backend/crates/identity/application/tests/native_hr_policy_parser_contract.rs', 'backend/crates/identity/application/tests/hr_policy_fixtures/literals.rs', 'backend/crates/identity/application/tests/hr_policy_fixtures/assert_fields.rs', 'tools/buck2', 'tools/buck/gen_first_party.py', 'backend/crates/identity/application/BUCK'}
    if not isinstance(b.get('sources'), list) or not required <= {source['path'] for source in b['sources']}:
        raise ValueError('missing required HR source/integration pins')
    for source in b['sources']:
        p = Path(source['path'])
        if p.is_absolute() or '..' in p.parts:
            raise ValueError('unsafe source')
        pin(ROOT / p, source['sha256'])
    if not os.access(ROOT / PROBE, os.X_OK):
        raise ValueError('launcher is not executable')
    git = b['git']['path']
    for args, expected in [(['rev-parse', 'HEAD'], b['candidate_head']), (['rev-parse', 'HEAD^{tree}'], b['candidate_tree']), (['status', '--porcelain=v1', '--untracked-files=all'], '')]:
        result = subprocess.run([git, '-C', str(ROOT), *args], capture_output=True, text=True, timeout=10, check=True)
        if result.stdout.strip() != expected:
            raise ValueError('candidate is not the exact clean reviewed tree')
    fixed_root = Path(b['fixed_tool']['path']).resolve().parents[2]
    blob = subprocess.run([git, '-C', str(fixed_root), 'show', FIXED_COMMIT + ':tools/lanes/fanout.py'], capture_output=True, timeout=10, check=True).stdout
    if hashlib.sha256(blob).hexdigest() != FIXED_SHA:
        raise ValueError('fixed tool commit/source authentication failed')
    if os.environ.get('CONSOLE_HR_POLICY_NODE') != b['node']['path'] or os.environ.get('CONSOLE_HR_POLICY_BINDING') != str(Path(binding_file)) or os.environ.get('CONSOLE_HR_POLICY_BINDING_SHA256') != binding_sha:
        raise ValueError('HR launcher environment differs from exact reviewed binding')
    return {'qualified': True, 'argv': argv, 'cwd': str(ROOT), 'spec': spec, 'implementation_started': False}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binding', required=True)
    parser.add_argument('--binding-sha256', required=True)
    parser.add_argument('invocation', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    invocation = args.invocation[1:] if args.invocation[:1] == ['--'] else args.invocation
    try:
        print(json.dumps(qualify(args.binding, args.binding_sha256, invocation, Path.cwd())))
    except (ValueError, KeyError, OSError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        print('HR admission qualification refused: ' + str(error), file=sys.stderr)
        sys.exit(126)
