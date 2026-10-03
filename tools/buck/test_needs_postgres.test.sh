#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
harness="${repo_root}/tools/buck/test_needs_postgres.sh"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/console-buck-postgres-test.XXXXXX")"
fake_bin="${scratch}/bin"; log="${scratch}/calls.log"; mkdir -p "${fake_bin}"
export TMPDIR="${scratch}"
trap 'rm -rf "${scratch}"' EXIT
cat >"${fake_bin}/docker" <<'DOCKER'
#!/usr/bin/env bash
{ printf 'docker'; printf ' %q' "$@"; printf '\n'; } >>"${HARNESS_LOG}"
sequence_value() {
  local sequence="$1" attempt_file="$2" default="$3"
  if [[ -z "${sequence}" ]]; then printf '%s' "${default}"; return; fi
  local attempt=0 index
  [[ -f "${attempt_file}" ]] && attempt="$(cat "${attempt_file}")"
  attempt=$((attempt + 1)); printf '%s' "${attempt}" >"${attempt_file}"
  local -a values
  IFS=, read -r -a values <<<"${sequence}"
  index=$((attempt - 1)); (( index < ${#values[@]} )) || index=$((${#values[@]} - 1))
  printf '%s' "${values[index]}"
}
case "$1" in
  run)
    run_name=""; cidfile=""
    while (($#)); do
      case "$1" in
        --name) run_name="$2"; shift ;;
        --env-file) printf '%s\n' "$2" >"${HARNESS_LOG}.container-env-file"; shift ;;
        --cidfile) cidfile="$2"; shift ;;
      esac
      shift
    done
    printf '%s\n' "${run_name}" >"${HARNESS_LOG}.run-name"
    if [[ -n "${cidfile}" ]]; then
      python3 - "${cidfile}" <<'PY_CIDFILE' || exit 1
import os, pathlib, stat, sys
p = pathlib.Path(sys.argv[1])
assert not p.exists(), "Docker --cidfile requires a nonexistent file"
assert p.parent.is_dir() and not p.parent.is_symlink()
assert stat.S_IMODE(p.parent.stat().st_mode) == 0o700, "CID parent must be private"
assert p.parent.stat().st_uid == os.getuid()
with p.open("x") as stream: stream.write("")
PY_CIDFILE
      printf '%s\n' "${cidfile}" >"${HARNESS_LOG}.cidfile"
    fi
    if [[ "${FAKE_DOCKER_RUN_STATUS:-0}" != 0 ]]; then
      touch "${HARNESS_LOG}.preexisting-container-present"
      echo 'fake container name already in use' >&2
      exit "${FAKE_DOCKER_RUN_STATUS}"
    fi
    cid=0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
    printf '%s\n' "${cid}" >"${HARNESS_LOG}.created-cid"
    touch "${HARNESS_LOG}.owned-container-present" "${HARNESS_LOG}.owned-volume-present"
    [[ -z "${cidfile}" ]] || printf '%s' "${cid}" >"${cidfile}"
    if [[ "${FAKE_DOCKER_RUN_PAUSE_BEFORE_STDOUT:-0}" == 1 ]]; then
      printf '%s\n' "$$" >"${HARNESS_LOG}.dockerpid"
      touch "${HARNESS_LOG}.run-ready"
      exec /bin/sleep 30
    fi
    printf '%s\n' "${cid}" ;;
  cp)
    if [[ "$3" == *:/topology.env ]]; then
      cut -d= -f1 "$2" | sort >"${HARNESS_LOG}.topology-env-keys"
      printf '%s\n' "$2" >"${HARNESS_LOG}.topology-env-file"
      python3 - "$2" <<'PY_STARTUP_TOPOLOGY' || exit 1
import sys, pathlib, stat, hashlib, os
p = pathlib.Path(sys.argv[1])
assert stat.S_IMODE(p.stat().st_mode) == 0o600
values = dict(line.split("=", 1) for line in p.read_text().splitlines())
keys = ["POSTGRES_ADMIN_PASSWORD", "CONSOLE_APP_POSTGRES_PASSWORD", "CONSOLE_RT_POSTGRES_PASSWORD", "CONSOLE_AUTH_POSTGRES_PASSWORD", "CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD", "CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD", "CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD", "CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD"]
assert all(values[k] for k in keys)
assert len({values[k] for k in keys}) == len(keys)
pathlib.Path(os.environ["HARNESS_LOG"] + ".startup-digest").write_text(hashlib.sha256(values["CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD"].encode()).hexdigest())
PY_STARTUP_TOPOLOGY
    fi
    exit 0 ;;
  exec)
    if [[ " $* " == *" /proc/1/comm "* ]]; then
      pid1_status="$(sequence_value "${FAKE_DOCKER_PID1_STATUS_SEQUENCE:-}" "${HARNESS_LOG}.pid1-status-attempt" "${FAKE_DOCKER_PID1_STATUS:-0}")"
      if [[ "${pid1_status}" != 0 ]]; then
        printf '%s\n' "${FAKE_DOCKER_PID1_ERROR_OUTPUT:-fake PID1 probe failed}" >&2
        exit "${pid1_status}"
      fi
      sequence_value "${FAKE_DOCKER_PID1_COMM_SEQUENCE:-}" "${HARNESS_LOG}.pid1-attempt" "${FAKE_DOCKER_PID1_COMM:-postgres}"
      printf '\n'
      exit 0
    fi
    if [[ " $* " == *" pg_isready "* ]]; then
      readiness_status="$(sequence_value "${FAKE_DOCKER_READINESS_STATUS_SEQUENCE:-}" "${HARNESS_LOG}.readiness-attempt" "${FAKE_DOCKER_READINESS_STATUS:-0}")"
      exit "${readiness_status}"
    fi
    if [[ " $* " == *" /topology.sh "* ]]; then
      printf 'topology\n' >>"${HARNESS_LOG}"
      exit "${FAKE_DOCKER_EXEC_STATUS:-0}"
    fi
    exit 0 ;;
  image) exit "${FAKE_DOCKER_IMAGE_INSPECT_STATUS:-1}" ;;
  pull) exit "$(sequence_value "${FAKE_DOCKER_PULL_STATUS_SEQUENCE:-}" "${HARNESS_LOG}.pull-attempt" "${FAKE_DOCKER_PULL_STATUS:-0}")" ;;
  port) echo 127.0.0.1:49123 ;;
  rm)
    if [[ "${FAKE_DOCKER_RM_STATUS:-0}" != 0 ]]; then
      echo 'fake-rm-private postgres://secret-password@127.0.0.1/private' >&2
      exit "${FAKE_DOCKER_RM_STATUS}"
    fi
    python3 - "${HARNESS_LOG}" "$@" <<'PY_REMOVE'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); args = sys.argv[3:]
options = [arg for arg in args if arg.startswith("-")]
targets = [arg for arg in args if not arg.startswith("-")]
volume = any(arg == "--volumes" or (arg.startswith("-") and not arg.startswith("--") and "v" in arg) for arg in options)
def marker(suffix): return pathlib.Path(str(p) + suffix)
name = marker(".run-name").read_text().strip() if marker(".run-name").exists() else ""
cid = marker(".created-cid").read_text().strip() if marker(".created-cid").exists() else ""
if name and name in targets:
    marker(".preexisting-container-present").unlink(missing_ok=True)
if any(target and target in (name, cid) for target in targets):
    marker(".owned-container-present").unlink(missing_ok=True)
    if volume: marker(".owned-volume-present").unlink(missing_ok=True)
PY_REMOVE
    ;;
  *) exit 1 ;;
esac
DOCKER
cat >"${fake_bin}/openssl" <<'OPENSSL'
#!/usr/bin/env bash
count_file="${HARNESS_LOG}.secrets"; count=0; [[ -f "${count_file}" ]] && count="$(cat "${count_file}")"; count=$((count+1)); printf '%s' "${count}" >"${count_file}"; printf 'secret-%s\n' "${count}"
OPENSSL
cat >"${scratch}/buck" <<'BUCK'
#!/usr/bin/env bash
{ printf 'buck'; printf ' %q' "$@"; printf '\n'; } >>"${HARNESS_LOG}"
printf 'buck-isolation %s\n' "${BUCK_ISOLATION_DIR-<unset>}" >>"${HARNESS_LOG}"
# Optional test-only argv recorder; observes the real harness's Buck handoff.
if [[ -n "${BROWSER_ARGV_CAPTURE_DIR:-}" ]]; then
  python3 - "${BROWSER_ARGV_CAPTURE_DIR}" "$@" <<'PY_BROWSER_ARGV' || exit 1
import json, pathlib, sys
root = pathlib.Path(sys.argv[1])
args = sys.argv[2:]
(root / (args[0] + ".json")).write_text(json.dumps(args))
if args[0] == "test":
    values = [value.split("=", 1)[1] for value in args if value.startswith("CONSOLE_BUCK_POSTGRES_ENV_FILE=")]
    assert len(values) == 1
    keys = {line.split("=", 1)[0] for line in pathlib.Path(values[0]).read_text().splitlines()}
    assert keys == {"DATABASE_URL", "CONSOLE_APALIS_OWNER_DATABASE_URL", "CONSOLE_APALIS_RUNTIME_DATABASE_URL", "CONSOLE_APALIS_ADMIN_DATABASE_URL", "CONSOLE_TEST_AUTH_DATABASE_URL", "CONSOLE_STARTUP_AUTH_DATABASE_URL", "CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL", "CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL", "CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL"}, "browser config polluted credential file"
PY_BROWSER_ARGV
fi
env_file=""; for arg in "$@"; do case "${arg}" in CONSOLE_BUCK_POSTGRES_ENV_FILE=*) env_file="${arg#*=}";; esac; done
[[ -f "${env_file}" && "$(stat -f '%Lp' "${env_file}")" == 600 ]]
grep -Fq 'DATABASE_URL=postgres://console_buck_admin:' "${env_file}"
grep -Fq 'CONSOLE_APALIS_OWNER_DATABASE_URL=postgres://console_app:' "${env_file}"
grep -Fq 'CONSOLE_APALIS_RUNTIME_DATABASE_URL=postgres://console_rt:' "${env_file}"
grep -Fq 'CONSOLE_APALIS_ADMIN_DATABASE_URL=postgres://console_buck_admin:' "${env_file}"
grep -Fq 'CONSOLE_TEST_AUTH_DATABASE_URL=postgres://console_auth_rt:' "${env_file}"
grep -Fq 'CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL=postgres://console_leave_cmd:' "${env_file}"
grep -Fq 'CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL=postgres://console_ontology_cmd:' "${env_file}"
grep -Fq 'CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL=postgres://console_platform_force_cmd:' "${env_file}"

# The build phase has no executor file; only inspect the actual test handoff.
if [[ "$1" == test ]]; then
  [[ -f "${env_file}" ]] || exit 1
  python3 - "${env_file}" <<'PY_STARTUP_TEST' || exit 1
import sys, pathlib, stat, hashlib, os, urllib.parse
p = pathlib.Path(sys.argv[1])
assert stat.S_IMODE(p.stat().st_mode) == 0o600
values = dict(line.split("=", 1) for line in p.read_text().splitlines())
roles = {"DATABASE_URL": "console_buck_admin", "CONSOLE_APALIS_OWNER_DATABASE_URL": "console_app", "CONSOLE_APALIS_RUNTIME_DATABASE_URL": "console_rt", "CONSOLE_TEST_AUTH_DATABASE_URL": "console_auth_rt", "CONSOLE_STARTUP_AUTH_DATABASE_URL": "console_auth_startup", "CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL": "console_leave_cmd", "CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL": "console_ontology_cmd", "CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL": "console_platform_force_cmd"}
urls = [urllib.parse.urlparse(values[k]) for k in roles]
assert all(u.username == role and u.password for u, role in zip(urls, roles.values()))
assert len({u.password for u in urls}) == len(urls)
assert len({(u.hostname, u.port, u.path) for u in urls}) == 1
startup = urllib.parse.urlparse(values["CONSOLE_STARTUP_AUTH_DATABASE_URL"])
assert startup.hostname == "127.0.0.1" and startup.port == 49123
assert not startup.query and not startup.fragment
assert hashlib.sha256(startup.password.encode()).hexdigest() == pathlib.Path(os.environ["HARNESS_LOG"] + ".startup-digest").read_text()
assert all(u.password not in pathlib.Path(os.environ["HARNESS_LOG"]).read_text() for u in urls)
PY_STARTUP_TEST
fi
printf '%s\n' "${env_file}" >>"${HARNESS_LOG}.envfiles"
if [[ "${FAKE_BUCK_SLEEP:-0}" == 1 ]]; then printf "%s\n" "$$" >"${HARNESS_LOG}.childpid"; exec sleep 30; fi
if [[ "$1" == test ]]; then exit "${FAKE_BUCK_TEST_STATUS:-${FAKE_BUCK_STATUS:-0}}"; fi
exit "${FAKE_BUCK_STATUS:-0}"
BUCK
cat >"${fake_bin}/sleep" <<'SLEEP'
#!/usr/bin/env bash
if [[ "${FAKE_SLEEP_INSTANT:-0}" == 1 ]]; then exit 0; fi
exec /bin/sleep "$@"
SLEEP
chmod +x "${fake_bin}/docker" "${fake_bin}/openssl" "${fake_bin}/sleep" "${scratch}/buck"
# Direct negative controls exercise the exact fake Docker's refusal boundary.
python3 - "${scratch}" "${fake_bin}/docker" <<'PY_CID_CONTROLS'
import json, os, pathlib, subprocess, sys
scratch, docker = map(pathlib.Path, sys.argv[1:])
for case, mode, existing in [("parent-0755", 0o755, False), ("existing-cidfile", 0o700, True)]:
    directory = scratch / case; directory.mkdir(); directory.chmod(mode)
    cidfile = directory / "container.cid"; sentinel = directory / "sentinel"
    sentinel.write_text("unrelated sentinel must remain\n")
    if existing: cidfile.write_text("preexisting CID sentinel\n")
    log = scratch / (case + ".log")
    result = subprocess.run([str(docker), "run", "--name", case, "--cidfile", str(cidfile)],
                            env=dict(os.environ, HARNESS_LOG=str(log)), capture_output=True,
                            text=True, timeout=30)
    raw = {"case": case, "argv": result.args, "exit": result.returncode,
           "stdout": result.stdout, "stderr": result.stderr,
           "sentinel_unchanged": sentinel.read_text() == "unrelated sentinel must remain\n",
           "cidfile_preserved": cidfile.read_text() == "preexisting CID sentinel\n" if existing else not cidfile.exists(),
           "owned_markers_absent": all(not pathlib.Path(str(log) + suffix).exists()
               for suffix in (".created-cid", ".owned-container-present", ".owned-volume-present"))}
    print(json.dumps(raw), flush=True)
    assert result.returncode != 0 and not result.stdout, "fake Docker validator failure was masked"
    assert raw["sentinel_unchanged"] and raw["cidfile_preserved"] and raw["owned_markers_absent"]
print("cidfile-controls: discovered=2 executed=2 failures=0", flush=True)
PY_CID_CONTROLS
raw_target_log="${scratch}/raw-target.log"
if PATH="${fake_bin}:${PATH}" HARNESS_LOG="${raw_target_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" "${harness}" //backend/app:console-app-itest-org_change_api; then exit 1; fi
! grep -q '^docker' "${raw_target_log}" 2>/dev/null
! grep -q '^buck' "${raw_target_log}" 2>/dev/null
entrypoint_ready_log="${scratch}/entrypoint-ready.log"
if PATH="${fake_bin}:${PATH}" HARNESS_LOG="${entrypoint_ready_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_PID1_COMM_SEQUENCE=docker-entrypoint.sh FAKE_SLEEP_INSTANT=1 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres; then exit 1; fi
[[ "$(grep -c 'cat /proc/1/comm' "${entrypoint_ready_log}")" == 30 ]]
[[ "$(grep -c 'pg_isready ' "${entrypoint_ready_log}")" == 0 ]]
if grep -Fxq 'topology' "${entrypoint_ready_log}"; then exit 1; fi
if grep -q '^buck' "${entrypoint_ready_log}" 2>/dev/null; then exit 1; fi
pid1_probe_failure_log="${scratch}/pid1-probe-failure.log"
set +e
pid1_probe_failure_output="$(PATH="${fake_bin}:${PATH}" HARNESS_LOG="${pid1_probe_failure_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_PID1_STATUS_SEQUENCE=1 FAKE_DOCKER_PID1_ERROR_OUTPUT='secret-probe postgres://console_buck_admin:secret-password@127.0.0.1:5432/console_buck_test' FAKE_SLEEP_INSTANT=1 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres 2>&1)"
pid1_probe_failure_status=$?
set -e
[[ "${pid1_probe_failure_status}" != 0 ]]
grep -Fq 'could not inspect disposable PostgreSQL PID 1 after 30 attempts' <<<"${pid1_probe_failure_output}"
if grep -Fq 'secret-probe' <<<"${pid1_probe_failure_output}"; then exit 1; fi
if grep -Fq 'postgres://' <<<"${pid1_probe_failure_output}"; then exit 1; fi
[[ "$(grep -c 'cat /proc/1/comm' "${pid1_probe_failure_log}")" == 30 ]]
[[ "$(grep -c 'pg_isready ' "${pid1_probe_failure_log}")" == 0 ]]
if grep -Fxq 'topology' "${pid1_probe_failure_log}"; then exit 1; fi
if grep -q '^buck' "${pid1_probe_failure_log}" 2>/dev/null; then exit 1; fi
tcp_not_ready_log="${scratch}/tcp-not-ready.log"
if PATH="${fake_bin}:${PATH}" HARNESS_LOG="${tcp_not_ready_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_PID1_COMM_SEQUENCE=postgres FAKE_DOCKER_READINESS_STATUS_SEQUENCE=1 FAKE_SLEEP_INSTANT=1 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres; then exit 1; fi
[[ "$(grep -c 'cat /proc/1/comm' "${tcp_not_ready_log}")" == 30 ]]
[[ "$(grep -c 'pg_isready -h 127.0.0.1 -U console_buck_admin -d ' "${tcp_not_ready_log}")" == 30 ]]
if grep -Fxq 'topology' "${tcp_not_ready_log}"; then exit 1; fi
if grep -q '^buck' "${tcp_not_ready_log}" 2>/dev/null; then exit 1; fi
recovery_log="${scratch}/recovery.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${recovery_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_PID1_COMM_SEQUENCE=docker-entrypoint.sh,postgres,postgres FAKE_DOCKER_READINESS_STATUS_SEQUENCE=1,0 FAKE_SLEEP_INSTANT=1 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
[[ "$(grep -c 'cat /proc/1/comm' "${recovery_log}")" == 3 ]]
[[ "$(grep -c 'pg_isready -h 127.0.0.1 -U console_buck_admin -d ' "${recovery_log}")" == 2 ]]
[[ "$(grep -c '^topology$' "${recovery_log}")" == 1 ]]
[[ "$(grep -c '^buck test ' "${recovery_log}")" == 1 ]]
topology_line="$(grep -n '^topology$' "${recovery_log}" | cut -d: -f1)"
readiness_line="$(grep -n 'pg_isready -h 127.0.0.1 -U console_buck_admin -d ' "${recovery_log}" | tail -1 | cut -d: -f1)"
buck_line="$(grep -n '^buck test ' "${recovery_log}" | cut -d: -f1)"
[[ "${topology_line}" -gt "${readiness_line}" ]]
[[ "${buck_line}" -gt "${topology_line}" ]]
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
calls="$(cat "${log}")"; buck_calls="$(grep '^buck' "${log}")"
grep -Fxq 'buck-isolation <unset>' "${log}"
grep -Fq -- '--env-file ' <<<"${calls}"
grep -Fq -- ':/topology.env' <<<"${calls}"
grep -Fq -- 'sh -ceu set\ -a\;\ .\ /topology.env\;\ exec\ bash\ /topology.sh' <<<"${calls}"
grep -Fq -- 'CONSOLE_BUCK_POSTGRES_ENV_FILE=' <<<"${buck_calls}"
! grep -Fq -- 'secret-' <<<"${calls}"
! grep -Fq -- 'postgres://' <<<"${calls}"
expected_topology_env_keys='CONSOLE_APP_POSTGRES_PASSWORD
CONSOLE_AUTH_POSTGRES_PASSWORD
CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD
CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD
CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD
CONSOLE_RT_POSTGRES_PASSWORD
CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD
POSTGRES_ADMIN_PASSWORD
POSTGRES_ADMIN_USER
POSTGRES_DB
POSTGRES_HOST
POSTGRES_PASSWORD
POSTGRES_PORT
POSTGRES_USER'
[[ "$(cat "${log}.topology-env-keys")" == "${expected_topology_env_keys}" ]]
! grep -Fq 'CONSOLE_PLATFORM_FORCE_COMMAND_PASSWORD' "${log}.topology-env-keys"
while IFS= read -r envfile; do [[ ! -e "${envfile}" ]]; done <"${log}.envfiles"
exact_log="${scratch}/exact.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${exact_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT=one_exact_test "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
grep -Fq 'CONSOLE_BUCK_RUST_TEST_EXACT=one_exact_test' "${exact_log}"
! grep -Fq -- '--timeout' "${exact_log}"
account_log="${scratch}/account-full.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${account_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" "${harness}" //tools/buck:app-auth-rest-pg
grep '^buck test ' "${account_log}" | grep -Fq -- '--timeout 21600'
! grep '^buck build ' "${account_log}" | grep -Fq -- '--timeout'
! grep -Fq 'CONSOLE_BUCK_RUST_TEST_EXACT=' "${account_log}"
account_exact_log="${scratch}/account-exact.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${account_exact_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT=one_exact_test "${harness}" //tools/buck:app-auth-rest-pg
! grep -Fq -- '--timeout' "${account_exact_log}"
mixed_log="${scratch}/account-mixed.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${mixed_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" "${harness}" //tools/buck:app-auth-rest-pg //tools/buck:pr473-ontology-key-revision-postgres
! grep -Fq -- '--timeout' "${mixed_log}"
! grep -Fq -- 'secret-' "${exact_log}"
isolation_log="${scratch}/isolation.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${isolation_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" CONSOLE_BUCK_NEEDS_POSTGRES_ISOLATION_DIR=postgres-proof "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
grep -Fxq 'buck-isolation postgres-proof' "${isolation_log}"
inherited_isolation_log="${scratch}/inherited-isolation.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${inherited_isolation_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" BUCK_ISOLATION_DIR=caller-proof "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
grep -Fxq 'buck-isolation caller-proof' "${inherited_isolation_log}"
invalid_isolation_log="${scratch}/invalid-isolation.log"
if PATH="${fake_bin}:${PATH}" HARNESS_LOG="${invalid_isolation_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" CONSOLE_BUCK_NEEDS_POSTGRES_ISOLATION_DIR='bad isolation' "${harness}" //tools/buck:pr473-ontology-key-revision-postgres; then exit 1; fi
! grep -q '^buck' "${invalid_isolation_log}" 2>/dev/null
if PATH="${fake_bin}:${PATH}" HARNESS_LOG="${exact_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT='bad test' "${harness}" //tools/buck:pr473-ontology-key-revision-postgres; then exit 1; fi
setup_failure_log="${scratch}/setup-failure.log"
setup_failure_status=0
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${setup_failure_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_EXEC_STATUS=23 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres || setup_failure_status="$?"
[[ "${setup_failure_status}" != 0 ]]
grep -Fq 'docker rm -f' "${setup_failure_log}"
! grep -q '^buck' "${setup_failure_log}"
! grep -Fq -- 'secret-' "${setup_failure_log}"
[[ ! -e "$(cat "${setup_failure_log}.topology-env-file")" ]]
buck_failure_log="${scratch}/buck-failure.log"; buck_failure_status=0
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${buck_failure_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_BUCK_STATUS=17 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres || buck_failure_status="$?"
[[ "${buck_failure_status}" != 0 ]]
while IFS= read -r envfile; do [[ ! -e "${envfile}" ]]; done <"${buck_failure_log}.envfiles"
signal_log="${scratch}/signal.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${signal_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_BUCK_SLEEP=1 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres &
harness_pid=$!
for _ in {1..50}; do [[ -s "${signal_log}.childpid" && -s "${signal_log}.envfiles" ]] && break; sleep 0.1; done
[[ -s "${signal_log}.childpid" && -s "${signal_log}.envfiles" ]]
child_pid="$(cat "${signal_log}.childpid")"
kill -TERM "${harness_pid}"
set +e
wait "${harness_pid}"
signal_status=$?
set -e
[[ "${signal_status}" == 143 ]]
! kill -0 "${child_pid}" 2>/dev/null
while IFS= read -r envfile; do [[ ! -e "${envfile}" ]]; done <"${signal_log}.envfiles"
! grep -Fq -- 'secret-' "${signal_log}"
! grep -Fq -- 'postgres://' "${signal_log}"
# The pinned image is pulled explicitly with bounded retry, because `docker run`'s
# implicit pull turned a registry timeout into a bare `exit 125` with no retry —
# observed reddening CI on 2026-07-31 with "context deadline exceeded".

# A transient registry recovers: two failures then success, and the run proceeds.
pull_recovery_log="${scratch}/pull-recovery.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${pull_recovery_log}" \
  CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" \
  FAKE_DOCKER_PULL_STATUS_SEQUENCE=1,1,0 FAKE_SLEEP_INSTANT=1 \
  "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
[[ "$(grep -c '^docker pull' "${pull_recovery_log}")" == 3 ]]
[[ "$(grep -c '^docker run' "${pull_recovery_log}")" == 1 ]]

# A registry that stays down FAILS the run and never starts a container. This is the
# inversion of the old behaviour, where `docker run` pulled implicitly and died 125.
pull_exhausted_log="${scratch}/pull-exhausted.log"
set +e
pull_exhausted_output="$(PATH="${fake_bin}:${PATH}" HARNESS_LOG="${pull_exhausted_log}" \
  CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" \
  FAKE_DOCKER_PULL_STATUS=1 FAKE_SLEEP_INSTANT=1 \
  "${harness}" //tools/buck:pr473-ontology-key-revision-postgres 2>&1)"
pull_exhausted_status=$?
set -e
[[ "${pull_exhausted_status}" != 0 ]]
grep -Fq 'could not pull the pinned PostgreSQL image after 4 attempts' <<<"${pull_exhausted_output}"
# The load-bearing assertion: no container is ever started.
[[ "$(grep -c '^docker run' "${pull_exhausted_log}")" == 0 ]]
# Retry must not leak a generated password into logs or output.
! grep -Fq -- 'secret-' "${pull_exhausted_log}"
! grep -Fq -- 'postgres://' <<<"${pull_exhausted_output}"

# An image already present locally is NOT re-pulled.
pull_cached_log="${scratch}/pull-cached.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${pull_cached_log}" \
  CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" \
  FAKE_DOCKER_IMAGE_INSPECT_STATUS=0 FAKE_SLEEP_INSTANT=1 \
  "${harness}" //tools/buck:pr473-ontology-key-revision-postgres
[[ "$(grep -c '^docker pull' "${pull_cached_log}")" == 0 ]]
[[ "$(grep -c '^docker run' "${pull_cached_log}")" == 1 ]]

# Browser variants explicitly transport only reviewed public driver configuration.
# This mocks Docker/Buck dispatch only; native Rust missing-driver failures remain
# a separately required actual-binary negative control, never inferred here.
python3 - "${scratch}" "${fake_bin}" "${harness}" <<'PY_BROWSER_ENV'
import json, os, pathlib, subprocess, sys
scratch, fake_bin, harness = map(pathlib.Path, sys.argv[1:])
prefixes = ["CONSOLE_BROWSER_JOURNEY", "CONSOLE_COMPANY_BROWSER", "CONSOLE_COMPANY_PREVIEW_BROWSER", "CONSOLE_HYDRATION_BROWSER"]
keys = [prefix + "_" + suffix for prefix in prefixes for suffix in ("DRIVER", "SHA256", "OUTPUT")]
base = dict(os.environ)
for key in keys:
    base.pop(key, None)
marker = scratch / "browser-env-must-not-execute"
public_values = {}
for prefix in prefixes:
    public_values[prefix + "_DRIVER"] = str(scratch / (prefix + " driver $(touch " + str(marker) + ");literal.cjs"))
    public_values[prefix + "_SHA256"] = "a" * 64
    public_values[prefix + "_OUTPUT"] = str(scratch / (prefix + " output with spaces"))
for case, target, supplied in [
    ("all-auth", "app-auth-rest-browser-pg", public_values),
    ("all-hydration", "app-health-readiness-browser-pg", public_values),
    ("all-root-auth", "root//tools/buck:app-auth-rest-browser-pg", public_values),
    ("missing", "app-auth-rest-browser-pg", {}),
    ("empty", "app-auth-rest-browser-pg", {key: "" for key in keys}),
    ("ordinary", "app-auth-rest-pg", public_values),
]:
    capture = scratch / ("browser-" + case)
    capture.mkdir()
    env = dict(base, PATH=str(fake_bin) + os.pathsep + base["PATH"],
               HARNESS_LOG=str(capture / "calls.log"),
               CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK=str(scratch / "buck"),
               CONSOLE_BUCK_NEEDS_POSTGRES_TEST_EXACT="actual_browser_leaf",
               BROWSER_ARGV_CAPTURE_DIR=str(capture),
               CONSOLE_HYDRATION_BROWSER_ARBITRARY="must-not-be-forwarded", **supplied)
    label = target if target.startswith("root//") else "//tools/buck:" + target
    result = subprocess.run([str(harness), label], env=env,
                            text=True, capture_output=True, timeout=30)
    assert result.returncode == 0, "browser harness fixture failed: " + case
    build = json.loads((capture / "build.json").read_text())
    tested = json.loads((capture / "test.json").read_text())
    assert build[0] == "build" and tested[0] == "test"
    assert not any(arg.startswith(tuple(prefixes)) for arg in build), "runtime config entered build argv"
    boundary = tested.index("--")
    forwarded = {}
    for index in range(boundary + 1, len(tested)):
        if tested[index] == "--env":
            key, value = tested[index + 1].split("=", 1)
            assert key not in forwarded, "duplicate runtime variable"
            forwarded[key] = value
    expected = public_values if case.startswith("all-") else {}
    actual = {key: value for key, value in forwarded.items() if key in keys}
    assert actual == expected, "NATIVE_BROWSER_ENV_REQUIRED: " + case
    assert "CONSOLE_HYDRATION_BROWSER_ARBITRARY" not in forwarded
    assert set(forwarded) == set(expected) | {"CONSOLE_BUCK_POSTGRES_ENV_FILE", "RUST_TEST_THREADS", "CONSOLE_BUCK_RUST_TEST_EXACT"}
    assert forwarded["RUST_TEST_THREADS"] == "1"
    assert forwarded["CONSOLE_BUCK_RUST_TEST_EXACT"] == "actual_browser_leaf"
    assert not any("postgres://" in arg or "secret-" in arg for arg in build + tested)
    assert not marker.exists(), "environment value was evaluated as shell code"
    assert not pathlib.Path(forwarded["CONSOLE_BUCK_POSTGRES_ENV_FILE"]).exists(), "credential cleanup omitted"
PY_BROWSER_ENV

# Preserve every preceding contract; add ownership assertions at the end so the
# baseline reports all eight new histories instead of stopping at the first RED.
lifecycle_failures=0; lifecycle_executed=0
lifecycle_case() {
  local name="$1"; shift
  lifecycle_executed=$((lifecycle_executed + 1))
  if "$@"; then echo "owned-lifecycle ${name}: PASS"; else
    echo "owned-lifecycle ${name}: FAIL" >&2
    lifecycle_failures=$((lifecycle_failures + 1))
  fi
}
assert_owned_cleanup() {
  python3 - "$1" "$2" "$3" <<'PY_OWNED'
import pathlib, shlex, sys
p = pathlib.Path(sys.argv[1]); actual, expected = map(int, sys.argv[2:])
assert actual == expected, "primary status changed"
rows = [shlex.split(line) for line in p.read_text().splitlines() if line.startswith("docker rm ")]
assert len(rows) == 1, "exactly one owned removal required"
args = rows[0][2:]; flags = [a for a in args if a.startswith("-")]
targets = [a for a in args if not a.startswith("-")]
cid = pathlib.Path(str(p) + ".created-cid").read_text().strip()
assert targets == [cid], "OWNED_RETURNED_CID_ONLY_REQUIRED"
assert any(a == "--force" or (a.startswith("-") and not a.startswith("--") and "f" in a) for a in flags)
assert any(a == "--volumes" or (a.startswith("-") and not a.startswith("--") and "v" in a) for a in flags), "OWNED_ANONYMOUS_VOLUME_CLEANUP_REQUIRED"
assert not pathlib.Path(str(p) + ".owned-container-present").exists()
assert not pathlib.Path(str(p) + ".owned-volume-present").exists()
assert not list(p.parent.glob("console-buck-postgres-container.*")), "container credentials survived"
assert not list(p.parent.glob("console-buck-postgres-env.*")), "test credentials survived"
record = pathlib.Path(str(p) + ".cidfile")
assert record.exists(), "private native CID path must be recorded"
cidfile = pathlib.Path(record.read_text().strip())
assert not cidfile.exists() and not cidfile.is_symlink(), "CID file survived"
assert not cidfile.parent.exists(), "private CID directory survived"
PY_OWNED
}
lifecycle_case success assert_owned_cleanup "${log}" 0 0
lifecycle_case setup-failure assert_owned_cleanup "${setup_failure_log}" "${setup_failure_status}" 23
lifecycle_case buck-failure assert_owned_cleanup "${buck_failure_log}" "${buck_failure_status}" 17
lifecycle_case term assert_owned_cleanup "${signal_log}" "${signal_status}" 143

creation_failure_log="${scratch}/creation-failure.log"; creation_failure_status=0
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${creation_failure_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_RUN_STATUS=125 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres || creation_failure_status="$?"
lifecycle_case failed-creation-preserves-preexisting python3 - "${creation_failure_log}" "${creation_failure_status}" <<'PY_CREATION'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
assert int(sys.argv[2]) == 125
assert pathlib.Path(str(p) + ".preexisting-container-present").exists(), "preexisting container deleted"
assert not any(line.startswith("docker rm ") for line in p.read_text().splitlines()), "failed creation attempted removal"
assert not any(line.startswith("buck ") for line in p.read_text().splitlines())
assert not list(p.parent.glob("console-buck-postgres-container.*"))
assert not list(p.parent.glob("console-buck-postgres-env.*"))
record = pathlib.Path(str(p) + ".cidfile")
assert record.exists(), "private native CID path must be recorded"
cidfile = pathlib.Path(record.read_text().strip())
assert not cidfile.exists() and not cidfile.is_symlink(), "CID file survived"
assert not cidfile.parent.exists(), "private CID directory survived"
PY_CREATION

for primary_status in 0 17; do
  removal_failure_log="${scratch}/removal-failure-${primary_status}.log"; removal_failure_status=0
  PATH="${fake_bin}:${PATH}" HARNESS_LOG="${removal_failure_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_RM_STATUS=42 FAKE_BUCK_TEST_STATUS="${primary_status}" "${harness}" //tools/buck:pr473-ontology-key-revision-postgres >"${removal_failure_log}.output" 2>&1 || removal_failure_status="$?"
  lifecycle_case "removal-failure-primary-${primary_status}" python3 - "${removal_failure_log}" "${removal_failure_status}" "${primary_status}" <<'PY_RM_FAILURE'
import pathlib, sys
p = pathlib.Path(sys.argv[1]); actual, primary = map(int, sys.argv[2:])
assert actual == primary if primary else actual != 0, "cleanup failure claimed success or masked primary failure"
output = pathlib.Path(str(p) + ".output").read_text()
assert "buck-postgres: owned disposable PostgreSQL cleanup failed" in output, "cleanup failure needs a fixed diagnostic"
assert "fake-rm-private" not in output and "postgres://" not in output and "secret-password" not in output
assert pathlib.Path(str(p) + ".owned-container-present").exists()
assert pathlib.Path(str(p) + ".owned-volume-present").exists()
assert not list(p.parent.glob("console-buck-postgres-container.*")), "container credentials survived failed removal"
assert not list(p.parent.glob("console-buck-postgres-env.*")), "test credentials survived failed removal"
record = pathlib.Path(str(p) + ".cidfile")
assert record.exists(), "private native CID path must be recorded"
cidfile = pathlib.Path(record.read_text().strip())
assert not cidfile.exists() and not cidfile.is_symlink(), "CID file survived"
assert not cidfile.parent.exists(), "private CID directory survived"
PY_RM_FAILURE
done

creation_signal_log="${scratch}/creation-signal.log"
PATH="${fake_bin}:${PATH}" HARNESS_LOG="${creation_signal_log}" CONSOLE_BUCK_NEEDS_POSTGRES_TEST_BUCK="${scratch}/buck" FAKE_DOCKER_RUN_PAUSE_BEFORE_STDOUT=1 "${harness}" //tools/buck:pr473-ontology-key-revision-postgres &
creation_harness_pid=$!
for _ in {1..50}; do [[ -s "${creation_signal_log}.dockerpid" && -e "${creation_signal_log}.run-ready" ]] && break; sleep 0.1; done
[[ -s "${creation_signal_log}.dockerpid" && -e "${creation_signal_log}.run-ready" ]]
creation_docker_pid="$(cat "${creation_signal_log}.dockerpid")"
# The fixture models a process-group TERM after Docker created/wrote the native
# CID but before stdout or command-substitution assignment can return it.
kill -TERM "${creation_harness_pid}" "${creation_docker_pid}"
creation_signal_status=0
wait "${creation_harness_pid}" || creation_signal_status="$?"
! kill -0 "${creation_docker_pid}" 2>/dev/null
lifecycle_case term-during-creation assert_owned_cleanup "${creation_signal_log}" "${creation_signal_status}" 143
printf 'owned-lifecycle: discovered=8 executed=%s failures=%s\n' "${lifecycle_executed}" "${lifecycle_failures}"
[[ "${lifecycle_executed}" == 8 && "${lifecycle_failures}" == 0 ]]

echo 'test_needs_postgres: PASS'
