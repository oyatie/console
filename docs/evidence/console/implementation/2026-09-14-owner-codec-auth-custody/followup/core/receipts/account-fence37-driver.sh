#!/usr/bin/env bash
# Evidence-only harness adapter. No product source is modified.
set -euo pipefail
[[ $# == 3 ]] || { echo 'usage: driver ROOT EXPECTED_SHA NEW_EVIDENCE_DIRECTORY' >&2; exit 2; }
export FENCE_ROOT="$(cd "$1" && pwd)"
expected_sha="$2"
evidence_dir="$3"
[[ "${evidence_dir}" == /private/tmp/* ]] || { echo 'evidence directory must be under /private/tmp' >&2; exit 2; }
mkdir "${evidence_dir}"
chmod 700 "${evidence_dir}"
export FENCE_CONTAINER_RECORD="${evidence_dir}/container-name.txt"
export FENCE_BUILD_RECORD="${evidence_dir}/prebuild-dispatch.txt"
export FENCE_IMAGE='postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'
export FENCE_LABEL="account-fence37-$(openssl rand -hex 16)"
map_path="${evidence_dir}/postgres-map.json"

snapshot() {
  python3 - "${FENCE_ROOT}" "$1" <<'PY'
import hashlib,json,pathlib,subprocess,sys
root=pathlib.Path(sys.argv[1])
files=['backend/Cargo.toml','backend/Cargo.lock','backend/rust-toolchain.toml',
 'backend/app/Cargo.toml','backend/app/tests/auth_rest.rs',
 'backend/app/tests/auth_rest/account_fence_projection.rs','backend/app/src/lib.rs',
 'backend/app/tests/auth_rest/fixtures/account-custody-dormant-v1-7af6dfd4.sql',
 'backend/app/src/account_custody_state.sql','ops/postgres-reconcile-topology.sh',
 'ops/generate-account-custody.py','ops/postgres-finalize-account-custody.sql',
 'ops/postgres-finalize-account-custody.sh','ops/account-custody-migrations.sha384',
 'tools/ci/cargo_needs_postgres.sh','tools/ci/cargo-test-runner.sh',
 'tools/ci/postgres-cargo-map.json','tools/ci/postgres-partition.mjs']
def git(*args):return subprocess.check_output(['git','-C',str(root),*args],text=True).strip()
data={'head':git('rev-parse','HEAD'),'status_porcelain':git('status','--porcelain'),
 'files':{name:hashlib.sha256((root/name).read_bytes()).hexdigest() for name in files}}
pathlib.Path(sys.argv[2]).write_text(json.dumps(data,sort_keys=True,indent=2)+'\n')
PY
}
snapshot "${evidence_dir}/source-before.json"
python3 - "${evidence_dir}/source-before.json" "${expected_sha}" <<'PY'
import json,sys
s=json.load(open(sys.argv[1]))
assert s['head']==sys.argv[2], 'candidate HEAD differs'
assert not s['status_porcelain'], 'candidate tree is not clean'
PY
python3 - "${FENCE_ROOT}" "${map_path}" <<'PY'
import json,pathlib,sys
root=pathlib.Path(sys.argv[1]); data=json.loads((root/'tools/ci/postgres-cargo-map.json').read_text())
entries=[e for e in data['entries'] if e.get('name')=='app-auth-rest-pg']
assert len(entries)==1
e=entries[0]
assert e['package']=='console-app' and e['test']=='auth_rest'
expected=['cargo','test','--locked','--manifest-path','backend/Cargo.toml','-p','console-app','--test','auth_rest','--','--test-threads=1']
assert e['cargo_argv']==expected, 'unreviewed underlying test command'
e['cargo_argv'].insert(e['cargo_argv'].index('--'),'account_fence_projection_')
# Preserve the whole map: partition validation precedes --only selection.
pathlib.Path(sys.argv[2]).write_text(json.dumps(data,indent=2)+'\n')
PY

cargo() {
  # Only narrow the existing seven-argument package prebuild. A different
  # command is passed untouched; the actual Python subprocess test invocation
  # in cargo-test-runner.sh does not resolve this Bash function at all.
  if [[ $# == 7 && "$1" == test && "$2" == --locked && "$3" == --manifest-path && "$4" == "${FENCE_ROOT}/backend/Cargo.toml" && "$5" == --no-run && "$6" == -p && "$7" == console-app ]]; then
    printf '%s\n' 'matched exact seven-argument prebuild; appended --test auth_rest' >> "${FENCE_BUILD_RECORD}"
    command cargo "$@" --test auth_rest
  else
    command cargo "$@"
  fi
}
docker() {
  local name='' previous='' argument='' matched_image=0
  if [[ "${1:-}" == run ]]; then
    for argument in "$@"; do
      [[ "${previous}" != --name ]] || name="${argument}"
      [[ "${argument}" != "${FENCE_IMAGE}" ]] || matched_image=1
      previous="${argument}"
    done
  fi
  if [[ "${1:-}" == run && "${name}" == console-cargo-postgres-* && "${matched_image}" == 1 ]]; then
    [[ ! -e "${FENCE_CONTAINER_RECORD}" ]] || { echo 'unexpected second scoped container' >&2; return 2; }
    printf '%s\n' "${name}" > "${FENCE_CONTAINER_RECORD}"
    command docker run --tmpfs /var/lib/postgresql:rw,size=4g \
      --label "console.evidence=${FENCE_LABEL}" "${@:2}"
  else
    command docker "$@"
  fi
}
export -f cargo docker
export RUSTUP_TOOLCHAIN=1.98.1 RUSTC_WRAPPER= CARGO_INCREMENTAL=1
export CARGO_TARGET_DIR=/private/tmp/console-account30-cargo SQLX_OFFLINE=true
cd "${FENCE_ROOT}"
set +e
bash tools/ci/cargo_needs_postgres.sh --map "${map_path}" --only app-auth-rest-pg --num-threads 1 > "${evidence_dir}/probe.log" 2>&1
probe_status=$?
set -e
printf '%s\n' "${probe_status}" > "${evidence_dir}/probe-exit.txt"

# Successful Docker listing plus no exact name is required. A daemon failure
# is not accepted as absence. If harness cleanup failed, remove only this run's
# exact name after independently matching our explicit ownership label.
docker_bounded() {
  python3 - "$@" <<'PYBOUND'
import subprocess,sys
try:
    result=subprocess.run(['docker',*sys.argv[1:]],capture_output=True,timeout=30)
    sys.stdout.buffer.write(result.stdout);sys.stderr.buffer.write(result.stderr)
    raise SystemExit(result.returncode)
except subprocess.TimeoutExpired as error:
    if error.stdout:sys.stdout.buffer.write(error.stdout)
    if error.stderr:sys.stderr.buffer.write(error.stderr)
    print('bounded Docker command timed out',file=sys.stderr)
    raise SystemExit(124)
PYBOUND
}
cleanup_status='no container launch recorded'
cleanup_failed=0
if [[ -f "${FENCE_CONTAINER_RECORD}" ]]; then
  name="$(cat "${FENCE_CONTAINER_RECORD}")"
  if docker_bounded ps -a --format '{{.Names}}' > "${evidence_dir}/containers-before-cleanup.txt" 2> "${evidence_dir}/cleanup-list-error.txt"; then
    if rg -Fxq "${name}" "${evidence_dir}/containers-before-cleanup.txt"; then
      if label="$(docker_bounded inspect --format '{{index .Config.Labels "console.evidence"}}' "${name}" 2> "${evidence_dir}/cleanup-inspect-error.txt")"; then
        printf '%s\n' "${label}" > "${evidence_dir}/cleanup-label.txt"
        if [[ "${label}" == "${FENCE_LABEL}" ]]; then
          # Removal failure/timeout never skips independent absence readback.
          set +e
          docker_bounded rm -f "${name}" > "${evidence_dir}/fallback-cleanup.txt" 2> "${evidence_dir}/fallback-cleanup-error.txt"
          removal_status=$?
          set -e
          printf '%s\n' "${removal_status}" > "${evidence_dir}/fallback-cleanup-exit.txt"
        else
          cleanup_failed=1
        fi
      else
        cleanup_failed=1
      fi
    fi
  else
    cleanup_failed=1
  fi
  if docker_bounded ps -a --format '{{.Names}}' > "${evidence_dir}/containers-after.txt" 2> "${evidence_dir}/cleanup-final-list-error.txt"; then
    if rg -Fxq "${name}" "${evidence_dir}/containers-after.txt"; then
      cleanup_failed=1
    fi
  else
    cleanup_failed=1
  fi
  if [[ "${cleanup_failed}" == 0 ]]; then
    cleanup_status='confirmed owned container absent via successful bounded docker ps -a'
  else
    cleanup_status='owned container cleanup/identity/absence verification failed; see exact command artifacts'
  fi
fi
printf '%s\n' "${cleanup_status}" > "${evidence_dir}/cleanup.txt"
printf '%s\n' "${FENCE_LABEL}" > "${evidence_dir}/ownership-label.txt"
snapshot "${evidence_dir}/source-after.json"
cmp "${evidence_dir}/source-before.json" "${evidence_dir}/source-after.json"
python3 - "${evidence_dir}" "$0" <<'PY'
import hashlib,json,pathlib,sys
d=pathlib.Path(sys.argv[1]); driver=pathlib.Path(sys.argv[2])
paths=[p for p in d.iterdir() if p.is_file()]
data={'driver_sha256':hashlib.sha256(driver.read_bytes()).hexdigest(),
 'artifacts':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
 'prebuild_interception_count':len((d/'prebuild-dispatch.txt').read_text().splitlines()) if (d/'prebuild-dispatch.txt').exists() else 0,
 'storage':'owned labeled PostgreSQL container only; 4GiB tmpfs; no durability proof',
 'source_pre_post':'exactly equal','probe_exit':int((d/'probe-exit.txt').read_text()),
 'cleanup':(d/'cleanup.txt').read_text().strip()}
(d/'driver-receipt.json').write_text(json.dumps(data,indent=2)+'\n')
PY
[[ "${cleanup_failed}" == 0 ]] || exit 2
exit "${probe_status}"
