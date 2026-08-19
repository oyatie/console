#!/usr/bin/env bash
# Run candidate-bound disposable-PostgreSQL integration tests via Cargo.
# Same Docker contract as tools/buck/test_needs_postgres.sh; Cargo driver for rust-cache.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
map_path="${repo_root}/tools/ci/postgres-cargo-map.json"
only_csv=""
workflow_only=0
num_threads=1
shard_id=""
keep_going=1

usage() {
  echo "usage: cargo_needs_postgres.sh [--map PATH] [--workflow-only] [--only name[,name...]] [--shard-id app|platform|ontology|domain-a|domain-b] [--num-threads N] [--keep-going|--fail-fast]" >&2
  exit 2
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --map) map_path="$2"; shift 2 ;;
    --map=*) map_path="${1#*=}"; shift ;;
    --workflow-only) workflow_only=1; shift ;;
    --only) only_csv="$2"; shift 2 ;;
    --only=*) only_csv="${1#*=}"; shift ;;
    --shard-id) shard_id="$2"; shift 2 ;;
    --shard-id=*) shard_id="${1#*=}"; shift ;;
    --num-threads) num_threads="$2"; shift 2 ;;
    --num-threads=*) num_threads="${1#*=}"; shift ;;
    --keep-going) keep_going=1; shift ;;
    --fail-fast) keep_going=0; shift ;;
    -h|--help) usage ;;
    *) usage ;;
  esac
done

case "${shard_id}" in
  ""|app|platform|ontology|domain-a|domain-b) ;;
  domain)
    echo "cargo-postgres: --shard-id domain retired in S2; use domain-a or domain-b" >&2
    exit 2
    ;;
  *)
    echo "cargo-postgres: invalid --shard-id ${shard_id} (want app|platform|ontology|domain-a|domain-b)" >&2
    exit 2
    ;;
esac

[[ -f "${map_path}" ]] || { echo "cargo-postgres: map missing: ${map_path}" >&2; exit 1; }

postgres_image="postgres:18.4@sha256:65f70a152846cf504dff86e807007e9aeac98c3aeb7b62541b2c55ab9d264e56"
container_name="console-cargo-postgres-${USER:-user}-$$"
database="console_cargo_test_$$_contract"
container_env_file=""
test_env_file=""

cleanup() {
  docker rm -f "${container_name}" >/dev/null 2>&1 || true
  [[ -z "${container_env_file}" ]] || rm -f "${container_env_file}"
  [[ -z "${test_env_file}" ]] || rm -f "${test_env_file}"
}
trap cleanup EXIT

secret() { openssl rand -hex 32; }
admin_password="$(secret)"
app_password="$(secret)"
runtime_password="$(secret)"
leave_command_password="$(secret)"
ontology_command_password="$(secret)"
platform_force_command_password="$(secret)"

umask 077
container_env_file="$(mktemp "${TMPDIR:-/tmp}/console-cargo-postgres-container.XXXXXX")"
chmod 600 "${container_env_file}"
{
  printf 'POSTGRES_DB=%s\nPOSTGRES_USER=console_buck_admin\nPOSTGRES_PASSWORD=%s\n' "${database}" "${admin_password}"
  printf 'POSTGRES_HOST=127.0.0.1\nPOSTGRES_PORT=5432\nPOSTGRES_ADMIN_USER=console_buck_admin\nPOSTGRES_ADMIN_PASSWORD=%s\n' "${admin_password}"
  printf 'CONSOLE_APP_POSTGRES_PASSWORD=%s\nCONSOLE_RT_POSTGRES_PASSWORD=%s\n' "${app_password}" "${runtime_password}"
  printf 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=%s\n' \
    "${leave_command_password}" "${ontology_command_password}" "${platform_force_command_password}"
} >"${container_env_file}"

if ! docker image inspect "${postgres_image}" >/dev/null 2>&1; then
  pull_attempts=0
  pull_backoff="3 9 27"
  for backoff in ${pull_backoff}; do
    if docker pull --quiet "${postgres_image}" >/dev/null 2>&1; then
      break
    fi
    pull_attempts=$((pull_attempts + 1))
    if [[ "${pull_attempts}" -ge 3 ]]; then
      docker pull "${postgres_image}" >&2 || true
      echo "cargo-postgres: could not pull pinned PostgreSQL image" >&2
      exit 1
    fi
    sleep "${backoff}"
  done
fi

docker run -d --rm --name "${container_name}" -p 127.0.0.1::5432 \
  --env-file "${container_env_file}" "${postgres_image}" \
  -c fsync=off -c synchronous_commit=off -c full_page_writes=off >/dev/null
docker cp "${repo_root}/ops/postgres-reconcile-topology.sh" "${container_name}:/topology.sh"
docker cp "${container_env_file}" "${container_name}:/topology.env"

for attempt in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30; do
  if ! pid1_comm="$(docker exec "${container_name}" cat /proc/1/comm 2>/dev/null)"; then
    if [[ "${attempt}" == 30 ]]; then echo "cargo-postgres: cannot inspect PID 1" >&2; exit 1; fi
    sleep 1; continue
  fi
  if [[ "${pid1_comm}" == "postgres" ]] && docker exec "${container_name}" pg_isready -h 127.0.0.1 -U console_buck_admin -d "${database}" >/dev/null 2>&1; then break; fi
  if [[ "${attempt}" == 30 ]]; then echo "cargo-postgres: not healthy" >&2; exit 1; fi
  sleep 1
done
docker exec "${container_name}" sh -ceu 'set -a; . /topology.env; exec bash /topology.sh'

# The reconcile above runs BEFORE any migration, so its canonical
# writer-ownership census sees zero canonical tables and enforces nothing. Run
# it again on a disposable probe database that has the real schema, with the
# zero-table guard armed. Nothing here touches "${database}", so the cargo tests
# below still see exactly the database they saw before.
bash "${repo_root}/backend/ci/gates/writer-ownership/canonical-enforce.sh" \
  "${repo_root}" "${container_name}" "canonical_probe_$$"

port_mapping="$(docker port "${container_name}" 5432/tcp)"
port="${port_mapping##*:}"
case "${port}" in
  ''|*[!0-9]*) echo "cargo-postgres: bad port" >&2; exit 1 ;;
esac
database_url="postgres://console_buck_admin:${admin_password}@127.0.0.1:${port}/${database}?options%5Bconsole.sqlx_test_bootstrap%5D=buck-sqlx-superuser-v1"
apalis_owner_database_url="postgres://console_app:${app_password}@127.0.0.1:${port}/${database}"
apalis_runtime_database_url="postgres://console_rt:${runtime_password}@127.0.0.1:${port}/${database}"
test_env_file="$(mktemp "${TMPDIR:-/tmp}/console-cargo-postgres-env.XXXXXX")"
chmod 600 "${test_env_file}"
{
  printf 'DATABASE_URL=%s\n' "${database_url}"
  printf 'CONSOLE_APALIS_OWNER_DATABASE_URL=%s\n' "${apalis_owner_database_url}"
  printf 'CONSOLE_APALIS_RUNTIME_DATABASE_URL=%s\n' "${apalis_runtime_database_url}"
  printf 'CONSOLE_APALIS_ADMIN_DATABASE_URL=%s\n' "${database_url}"
} >"${test_env_file}"

while IFS= read -r line || [[ -n "${line}" ]]; do
  case "${line}" in
    *=*) key="${line%%=*}"; value="${line#*=}"; export "${key}=${value}" ;;
  esac
done <"${test_env_file}"

export SQLX_OFFLINE=true
export RUST_TEST_THREADS="${num_threads}"
export CARGO_TERM_COLOR=always

tmp_list="$(mktemp "${TMPDIR:-/tmp}/console-cargo-list.XXXXXX")"
python3 - "${map_path}" "${workflow_only}" "${only_csv}" "${shard_id}" >"${tmp_list}" <<'PY'
import json, sys

def package_family(package_name: str) -> str:
    p = package_name or ""
    if p == "console-app":
        return "app"
    if "ontology" in p:
        return "ontology"
    if p.startswith("console-platform") or p == "console-platform-db":
        return "platform"
    return "domain"

def domain_subshard_by_package(entries):
    """Greedy balance by workflow entry count (must match tools/ci/postgres-shard.mjs)."""
    counts = {}
    for e in entries:
        if not e.get("in_workflow_postgres_job"):
            continue
        p = e.get("package") or ""
        if package_family(p) != "domain":
            continue
        counts[p] = counts.get(p, 0) + 1
    ordered = sorted(counts.items(), key=lambda kv: (-kv[1], kv[0]))
    out = {}
    count_a = 0
    count_b = 0
    for pkg, n in ordered:
        if count_a <= count_b:
            out[pkg] = "domain-a"
            count_a += n
        else:
            out[pkg] = "domain-b"
            count_b += n
    return out

def shard_id_for_package(package_name: str, domain_map: dict) -> str:
    family = package_family(package_name)
    if family != "domain":
        return family
    return domain_map.get(package_name, "domain-a")

path, workflow_only, only_csv, shard_id = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
doc = json.load(open(path))
entries = doc["entries"]
domain_map = domain_subshard_by_package(entries)
only_set = set(only_csv.split(",")) if only_csv.strip() else None
for e in entries:
    if workflow_only == "1" and not e.get("in_workflow_postgres_job"):
        continue
    if only_set is not None and e["name"] not in only_set:
        continue
    if shard_id and shard_id_for_package(e.get("package") or "", domain_map) != shard_id:
        continue
    print(json.dumps({"name": e["name"], "package": e["package"], "argv": e["cargo_argv"]}))
PY

if [[ ! -s "${tmp_list}" ]]; then
  echo "cargo-postgres: no map entries selected" >&2
  exit 1
fi

count="$(wc -l <"${tmp_list}" | tr -d ' ')"
if [[ -n "${shard_id}" ]]; then
  echo "cargo-postgres: running ${count} cargo test invocations (shard=${shard_id} threads=${num_threads})"
else
  echo "cargo-postgres: running ${count} cargo test invocations (threads=${num_threads})"
fi

# Unique packages for --no-run build
tmp_pkgs="$(mktemp "${TMPDIR:-/tmp}/console-cargo-pkgs.XXXXXX")"
python3 - "${tmp_list}" >"${tmp_pkgs}" <<'PY'
import json, sys
pkgs=set()
for line in open(sys.argv[1]):
    pkgs.add(json.loads(line)["package"])
for p in sorted(pkgs):
    print(p)
PY

build_args=(cargo test --locked --manifest-path "${repo_root}/backend/Cargo.toml" --no-run)
while IFS= read -r p; do
  [[ -n "${p}" ]] && build_args+=(-p "${p}")
done <"${tmp_pkgs}"
echo "cargo-postgres: building packages..."
( cd "${repo_root}" && "${build_args[@]}" )

# Fail-slow sweep: run every selected binary, collect per-binary pass/fail, print
# a summary table, and exit non-zero if any failed. The keep-going loop lives in
# cargo-test-runner.sh so it is unit-testable without Docker (fake map + stubbed
# cargo). Default is --keep-going; --fail-fast opts back out for local use.
export CARGO_REPO_ROOT="${repo_root}"
# Attributes each `cargo-postgres-timing:` line to its shard, so durations
# harvested from five separate job logs can be re-packed as one population.
export CARGO_POSTGRES_SHARD_ID="${shard_id}"
export RUST_TEST_THREADS="${num_threads}"
if [[ "${keep_going}" == 1 ]]; then
  "${repo_root}/tools/ci/cargo-test-runner.sh" --keep-going <"${tmp_list}"
else
  "${repo_root}/tools/ci/cargo-test-runner.sh" --fail-fast <"${tmp_list}"
fi
status=$?
rm -f "${tmp_list}" "${tmp_pkgs}"
exit "${status}"
