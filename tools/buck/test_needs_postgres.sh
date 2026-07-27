#!/usr/bin/env bash
# Run Buck2 tests that need PostgreSQL against a disposable local container.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
postgres_image="postgres:18.4@sha256:65f70a152846cf504dff86e807007e9aeac98c3aeb7b62541b2c55ab9d264e56"
container_name="mnt-buck-postgres-${USER:-user}-$$"
buck_bin="${MNT_BUCK_NEEDS_POSTGRES_TEST_BUCK:-${repo_root}/tools/buck2}"
isolation_name="${MNT_BUCK_NEEDS_POSTGRES_ISOLATION_DIR:-}"
if [[ -n "${isolation_name}" && ! "${isolation_name}" =~ ^[[:alnum:]_.-]+$ ]]; then
  echo "buck-postgres: isolation name must contain only letters, digits, dot, underscore, or dash" >&2
  exit 1
fi
database="mnt_buck_test_$$_contract"
container_env_file=""
test_env_file=""
active_buck_pid=""
exact_test="${MNT_BUCK_NEEDS_POSTGRES_TEST_EXACT:-}"
if [[ -n "${exact_test}" && ! "${exact_test}" =~ ^[[:alnum:]_:]+$ ]]; then
  echo "buck-postgres: exact Rust test name contains unsupported characters" >&2
  exit 1
fi
for arg in "$@"; do
  case "${arg}" in
    //backend/*|root//backend/*)
      echo "buck-postgres: raw backend test targets bypass the credential loader; use a //tools/buck PostgreSQL wrapper" >&2
      exit 2
      ;;
  esac
done

cleanup() {
  docker rm -f "${container_name}" >/dev/null 2>&1 || true
  [[ -z "${container_env_file}" ]] || rm -f "${container_env_file}"
  [[ -z "${test_env_file}" ]] || rm -f "${test_env_file}"
}

on_signal() {
  local status="$1"
  trap - HUP INT TERM
  if [[ -n "${active_buck_pid}" ]]; then
    kill -TERM "${active_buck_pid}" >/dev/null 2>&1 || true
    wait "${active_buck_pid}" >/dev/null 2>&1 || true
    active_buck_pid=""
  fi
  exit "${status}"
}
trap cleanup EXIT
trap 'on_signal 129' HUP
trap 'on_signal 130' INT
trap 'on_signal 143' TERM

secret() { openssl rand -hex 32; }
admin_password="$(secret)"
app_password="$(secret)"
runtime_password="$(secret)"
leave_command_password="$(secret)"
ontology_command_password="$(secret)"
platform_force_command_password="$(secret)"
passwords=("${admin_password}" "${app_password}" "${runtime_password}" "${leave_command_password}" "${ontology_command_password}" "${platform_force_command_password}")
for ((i = 0; i < ${#passwords[@]}; i++)); do
  for ((j = i + 1; j < ${#passwords[@]}; j++)); do
    if [[ "${passwords[i]}" == "${passwords[j]}" ]]; then
      echo "buck-postgres: generated passwords must be pairwise distinct" >&2
      exit 1
    fi
  done
done

umask 077
container_env_file="$(mktemp "${TMPDIR:-/tmp}/mnt-buck-postgres-container.XXXXXX")"
chmod 600 "${container_env_file}"
{
  printf 'POSTGRES_DB=%s\nPOSTGRES_USER=mnt_buck_admin\nPOSTGRES_PASSWORD=%s\n' "${database}" "${admin_password}"
  printf 'POSTGRES_HOST=127.0.0.1\nPOSTGRES_PORT=5432\nPOSTGRES_ADMIN_USER=mnt_buck_admin\nPOSTGRES_ADMIN_PASSWORD=%s\n' "${admin_password}"
  printf 'MNT_APP_POSTGRES_PASSWORD=%s\nMNT_RT_POSTGRES_PASSWORD=%s\n' "${app_password}" "${runtime_password}"
  printf 'MNT_LEAVE_COMMAND_POSTGRES_PASSWORD=%s\nMNT_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=%s\nMNT_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=%s\n' "${leave_command_password}" "${ontology_command_password}" "${platform_force_command_password}"
} >"${container_env_file}"

docker run -d --rm --name "${container_name}" -p 127.0.0.1::5432 \
  --env-file "${container_env_file}" "${postgres_image}" >/dev/null
docker cp "${repo_root}/ops/postgres-reconcile-topology.sh" "${container_name}:/topology.sh"
docker cp "${container_env_file}" "${container_name}:/topology.env"

for attempt in {1..30}; do
  if ! pid1_comm="$(docker exec "${container_name}" cat /proc/1/comm 2>/dev/null)"; then
    if [[ "${attempt}" == 30 ]]; then
      echo "buck-postgres: could not inspect disposable PostgreSQL PID 1 after 30 attempts" >&2
      exit 1
    fi
    sleep 1
    continue
  fi
  if [[ "${pid1_comm}" == "postgres" ]] && docker exec "${container_name}" pg_isready -h 127.0.0.1 -U mnt_buck_admin -d "${database}" >/dev/null 2>&1; then break; fi
  if [[ "${attempt}" == 30 ]]; then echo "buck-postgres: disposable PostgreSQL did not become healthy" >&2; exit 1; fi
  sleep 1
done
# The protected file path, not any value, is in Docker's host argv.
docker exec "${container_name}" sh -ceu 'set -a; . /topology.env; exec bash /topology.sh'

port_mapping="$(docker port "${container_name}" 5432/tcp)"
port="${port_mapping##*:}"
if [[ ! "${port}" =~ ^[0-9]+$ ]]; then echo "buck-postgres: could not resolve disposable PostgreSQL loopback port" >&2; exit 1; fi
database_url="postgres://mnt_buck_admin:${admin_password}@127.0.0.1:${port}/${database}?options%5Bmnt.sqlx_test_bootstrap%5D=buck-sqlx-superuser-v1"
apalis_owner_database_url="postgres://mnt_app:${app_password}@127.0.0.1:${port}/${database}"
apalis_runtime_database_url="postgres://mnt_rt:${runtime_password}@127.0.0.1:${port}/${database}"
test_env_file="$(mktemp "${TMPDIR:-/tmp}/mnt-buck-postgres-env.XXXXXX")"
chmod 600 "${test_env_file}"
{
  printf 'DATABASE_URL=%s\n' "${database_url}"
  printf 'MNT_APALIS_OWNER_DATABASE_URL=%s\n' "${apalis_owner_database_url}"
  printf 'MNT_APALIS_RUNTIME_DATABASE_URL=%s\n' "${apalis_runtime_database_url}"
  printf 'MNT_APALIS_ADMIN_DATABASE_URL=%s\n' "${database_url}"
} >"${test_env_file}"

# Only a mode-0600 path crosses Buck's test-executor argv. The wrapper parses
# this fixed data file inside the test process without evaluating its contents.
test_executor_args=(--env "MNT_BUCK_POSTGRES_ENV_FILE=${test_env_file}" --env RUST_TEST_THREADS=1)
if [[ -n "${exact_test}" ]]; then
  test_executor_args+=(--env "MNT_BUCK_RUST_TEST_EXACT=${exact_test}")
fi
# Reuse the caller's/default Buck daemon so PostgreSQL integration tests share
# the same-worktree analysis and compile cache. Callers that need an isolated
# daemon can still opt in explicitly without forcing every run cold.
if [[ -n "${isolation_name}" ]]; then
  BUCK_ISOLATION_DIR="${isolation_name}" "${buck_bin}" test --local-only "$@" \
    -- "${test_executor_args[@]}" &
else
  "${buck_bin}" test --local-only "$@" -- "${test_executor_args[@]}" &
fi
active_buck_pid="$!"
if wait "${active_buck_pid}"; then
  buck_status=0
else
  buck_status="$?"
fi
active_buck_pid=""
exit "${buck_status}"
