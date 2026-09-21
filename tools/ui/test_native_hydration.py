#!/usr/bin/env python3
"""Private review candidate: actual producer/SSR boundary, not yet executed.

Exit 0: requested oracle passed. Exit 1: semantic assertion failed.
Exit 2: fixture/tool/positive-control prerequisite failed; never lane RED.
The supplied source revision is archived into a new disposable fixture.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile

CRATE = Path('backend/crates/payroll/ui')
PAIR = (CRATE / 'pkg/console_payroll_ui.js', CRATE / 'pkg/console_payroll_ui_bg.wasm')
SSR = ('//backend/crates/payroll/ui:console-payroll-ui',
       '//backend/crates/payroll/ui:console-payroll-ui-unit')

class Prerequisite(Exception):
    pass

class SemanticFailure(Exception):
    pass

def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def record(p, value):
    p.write_text(json.dumps(value, indent=2) + '\n')

def command(argv, root, evidence, name, env=None):
    proc = subprocess.run(argv, cwd=root, env=env, text=True, capture_output=True)
    (evidence / (name + '.stdout')).write_text(proc.stdout)
    (evidence / (name + '.stderr')).write_text(proc.stderr)
    record(evidence / (name + '.command.json'), {'argv': argv, 'cwd': str(root), 'returncode': proc.returncode})
    return proc

def fixture(args):
    source = Path(args.source).resolve()
    rev = subprocess.check_output(['git', '-C', str(source), 'rev-parse', args.revision + '^{commit}'], text=True).strip()
    if rev != args.revision:
        raise Prerequisite('revision must be the full exact immutable commit SHA')
    run = Path(args.run_dir).resolve()
    run.mkdir(parents=False, exist_ok=False)
    root = run / 'root'; root.mkdir()
    evidence = run / 'evidence'; evidence.mkdir()
    archive = run / 'source.tar'
    with archive.open('wb') as out:
        subprocess.run(['git', '-C', str(source), 'archive', rev], stdout=out, check=True)
    with tarfile.open(archive) as tar:
        tar.extractall(root, filter='data')
    subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
    subprocess.run(['git', '-C', str(root), 'add', '--all'], check=True)
    record(evidence / 'fixture.json', {'source': str(source), 'revision': rev,
           'archive_sha256': digest(archive), 'kind': 'exact Git archive; index only; no synthesized commit'})
    for tool in ('node', 'bash', 'git', 'dotslash'):
        if shutil.which(tool) is None:
            raise Prerequisite('missing tool: ' + tool)
    return root, evidence

def events(root):
    return set((root / 'buck-out/v2/log').glob('*_events.pb.zst'))

def inspect_native_actions(root, evidence, logs):
    """Require actual new compiler and bindgen actions; a no-op cannot pass.

    This intentionally requires a cold real-action qualification run. Cache-only
    or an unrecognised action shape is a prerequisite gap, not semantic RED.
    """
    rows = []
    for index, log in enumerate(sorted(logs)):
        proc = command(['tools/buck2', 'log', 'what-ran', '--format', 'json', str(log)], root, evidence, f'actions-{index}')
        if proc.returncode:
            raise Prerequisite('could not inspect actual Buck event log')
        for line in proc.stdout.splitlines():
            if line.startswith('{'):
                rows.append(json.loads(line))
    compiled = []
    for row in rows:
        identity = row.get('identity', '')
        if 'backend/crates/payroll/ui:' not in identity or '(rustc ' not in identity:
            continue
        argv = row.get('reproducer', {}).get('details', {}).get('command', [])
        for arg in argv:
            if not arg.startswith('@'):
                continue
            response = root / arg[1:]
            if not response.is_file():
                raise Prerequisite('actual compiler response file no longer available')
            lines = response.read_text().splitlines()
            if '--crate-type=cdylib' not in lines or '--target=wasm32-unknown-unknown' not in lines:
                continue
            required = {'--crate-name=console_payroll_ui', '--cfg=feature="hydrate"', '--cfg=feature="islands"'}
            if not required.issubset(lines) or '--cfg=feature="ssr"' in lines:
                raise SemanticFailure('native product compiler features violate hydrate/islands contract')
            if not any(x.startswith('-Copt-level=') and x != '-Copt-level=0' for x in lines):
                raise SemanticFailure('product WASM was not optimized')
            outputs = [x.split('=', 2)[2] for x in lines if x.startswith('--emit=link=')]
            if len(outputs) != 1 or not (root / outputs[0]).is_file():
                raise Prerequisite('actual cdylib link output unavailable or unrecognised')
            wasm = root / outputs[0]
            if wasm.read_bytes()[:8] != b'\0asm\x01\0\0\0':
                raise SemanticFailure('actual native compiler output is not a WASM module')
            linker = [x[len('-Clinker='):] for x in lines if x.startswith('-Clinker=')]
            if len(linker) != 1:
                raise Prerequisite('actual linker identity unavailable')
            link_path = root / linker[0]
            # Buck may wrap the declared linker. Preserve it for independent
            # validation rather than inventing proof from a rustc target flag.
            if not link_path.is_file():
                raise Prerequisite('actual linker executable/wrapper unavailable')
            compiled.append({'action': row, 'response': str(response),
                             'response_sha256': digest(response), 'output': outputs[0],
                             'output_sha256': digest(wasm), 'linker': str(link_path),
                             'linker_sha256': digest(link_path)})
    record(evidence / 'native-compiler-actions.json', compiled)
    if not compiled:
        raise Prerequisite('native WASM cdylib compiler action unavailable; cache-only or unrecognised action evidence cannot establish GREEN or semantic RED')
    # The bindgen action must declare both the actual binary and just-linked
    # module as command inputs. This contract allows a narrow Python wrapper.
    bindgen = []
    for row in rows:
        argv = row.get('reproducer', {}).get('details', {}).get('command', [])
        executables = [x for x in argv if Path(x).name == 'wasm-bindgen' and (root / x).is_file()]
        linked = [c for c in compiled if c['output'] in argv]
        if executables and linked:
            if '--out-dir' not in argv or argv.index('--out-dir') + 1 == len(argv):
                raise Prerequisite('bindgen action must expose its actual --out-dir for output binding')
            output_dir = root / argv[argv.index('--out-dir') + 1]
            output_pair = [output_dir / p.name for p in PAIR]
            if not all(p.is_file() for p in output_pair):
                raise SemanticFailure('actual bindgen action did not emit the required pair')
            version = command([str(root / executables[0]), '--version'], root, evidence, 'bindgen-version')
            if version.returncode or version.stdout.strip() != 'wasm-bindgen 0.2.123':
                raise SemanticFailure('actual bindgen executable is not pinned 0.2.123')
            bindgen.append({'action': row, 'executable': executables[0],
                            'executable_sha256': digest(root / executables[0]),
                            'linked_input_sha256': linked[0]['output_sha256'],
                            'outputs': {p.name: digest(p) for p in output_pair}})
    record(evidence / 'native-bindgen-actions.json', bindgen)
    if not bindgen:
        raise Prerequisite('bindgen action consuming the just-linked native module unavailable; missing action evidence is not semantic RED')
    published = {p.name: digest(root / p) for p in PAIR}
    if not any(action['outputs'] == published for action in bindgen):
        raise SemanticFailure('published pair is not the actual bindgen action output pair')
    # Additional action-bound provenance, pin and macro/linker inspections are
    # mandatory qualification cases, documented separately in this packet.

def producer(root, evidence):
    sentinel_dir = evidence / 'sentinel'; sentinel_dir.mkdir()
    cargo_log = evidence / 'cargo-requests.jsonl'
    cargo = sentinel_dir / 'cargo'
    cargo.write_text('#!' + sys.executable + '\n' + '''import json,os,sys
from pathlib import Path
args=sys.argv[1:]
with Path(os.environ['CONSOLE_ORACLE_CARGO_LOG']).open('a') as out:
    out.write(json.dumps(args)+'\\n')
# Producer must have a generated locked graph. No product compilation or
# on-demand metadata generation is needed by this artifact-producing wrapper.
print('CONSOLE_ORACLE_REFUSED_CARGO: '+repr(args),file=sys.stderr)
sys.exit(97)
''')
    cargo.chmod(0o755)
    env = os.environ.copy()
    env['PATH'] = str(sentinel_dir) + os.pathsep + env['PATH']
    env['CONSOLE_ORACLE_CARGO_LOG'] = str(cargo_log)
    before = events(root)
    proc = command(['bash', 'tools/ui/build-payroll-wasm.sh'], root, evidence, 'producer', env)
    calls = [json.loads(x) for x in cargo_log.read_text().splitlines()] if cargo_log.exists() else []
    product = [x for x in calls if any(a in ('build', 'check', 'test', 'run', 'rustc') for a in x)]
    if product:
        record(evidence / 'semantic-red.json', {'reason': 'actual existing producer requested prohibited Cargo product compilation', 'requests': product})
        raise SemanticFailure('real producer requested Cargo product compilation')
    if proc.returncode:
        raise Prerequisite('producer failed without witnessed prohibited product compilation; inspect command stderr')
    new_logs = events(root) - before
    if not new_logs:
        raise Prerequisite('producer returned success without a new actual Buck event log; missing evidence alone does not distinguish cache reuse from a no-op')
    inspect_native_actions(root, evidence, new_logs)
    for item in PAIR:
        if not (root / item).is_file():
            raise SemanticFailure('producer did not publish required pair: ' + str(item))
    if (root / PAIR[1]).read_bytes()[:8] != b'\0asm\x01\0\0\0':
        raise SemanticFailure('published output is not WASM')
    check = command(['node', 'scripts/check-wasm-bundle-drift.mjs'], root, evidence, 'producer-integrity')
    if check.returncode:
        raise SemanticFailure('producer reported success but its published bundle fails integrity')
    record(evidence / 'published-output-hashes.json', {str(p): digest(root / p) for p in (*PAIR, CRATE / 'bundle.lock.json')})

def consumer(root, evidence):
    # Actual existing library and test resource consumers, no new missing label.
    baseline_check = command(['node', 'scripts/check-wasm-bundle-drift.mjs'], root, evidence, 'baseline-integrity')
    for index, label in enumerate(SSR):
        baseline = command(['tools/buck2', 'build', label], root, evidence, f'consumer-positive-{index}')
        if baseline.returncode:
            raise Prerequisite('existing native SSR/test positive build failed; no consumer RED')
    target = root / PAIR[0]
    original = target.read_bytes()
    target.write_bytes(original + b'\n// native-consumer-oracle deliberate unmatched output\n')
    record(evidence / 'corruption.json', {'path': str(PAIR[0]), 'before_sha256': hashlib.sha256(original).hexdigest(), 'after_sha256': digest(target)})
    corrupted_results = []
    try:
        # Separate commands ensure one validated consumer cannot hide another
        # consumer that still embeds unchecked source-tree resources.
        for index, label in enumerate(SSR):
            corrupted_results.append(command(['tools/buck2', 'build', label], root, evidence, f'consumer-corrupted-{index}'))
    finally:
        target.write_bytes(original)
    accepted = [label for label, result in zip(SSR, corrupted_results) if result.returncode == 0]
    if accepted:
        record(evidence / 'semantic-red.json', {'reason': 'actual consumers accepted corrupt committed JS without validation', 'targets': accepted})
        raise SemanticFailure('actual SSR/test build accepted an unmatched committed output: ' + ', '.join(accepted))
    if baseline_check.returncode:
        raise Prerequisite('baseline bundle integrity is stale; a later failure cannot establish corruption-specific GREEN')
    for corrupted in corrupted_results:
        # Validator-specific output and corrupted filename distinguish this
        # failure from a missing compiler/target or unrelated build regression.
        failure = corrupted.stdout + corrupted.stderr
        if 'Committed hydration bundle is stale' not in failure or PAIR[0].name not in failure:
            raise Prerequisite('consumer failed without the expected bundle-validation diagnostic')
    for index, label in enumerate(SSR):
        repaired = command(['tools/buck2', 'build', label], root, evidence, f'consumer-repaired-{index}')
        if repaired.returncode:
            raise Prerequisite('restored positive control failed')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('boundary', choices=('producer', 'consumer'))
    parser.add_argument('--source', required=True)
    parser.add_argument('--revision', required=True)
    parser.add_argument('--run-dir', required=True, help='new private directory; fixture/evidence are retained')
    args = parser.parse_args()
    try:
        root, evidence = fixture(args)
        (producer if args.boundary == 'producer' else consumer)(root, evidence)
    except SemanticFailure as error:
        print('SEMANTIC_ASSERTION_FAILED: ' + str(error), file=sys.stderr)
        return 1
    except (Prerequisite, OSError, subprocess.SubprocessError, ValueError) as error:
        print('PREREQUISITE_FAILED: ' + str(error), file=sys.stderr)
        return 2
    print('BOUNDARY_ORACLE_PASS: ' + args.boundary + '; broader qualification cases remain separate')
    return 0

if __name__ == '__main__':
    sys.exit(main())
