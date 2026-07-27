#!/usr/bin/env bash
# Recreate the dedicated e2e database and apply the sqlx schema migrations.
#
# Drops + recreates `console_e2e` (a clean slate every run so cold-start seeding is
# deterministic), then runs the app's `migrate` role which applies the embedded
# migrations and exits. A distinct local cluster administrator reconciles the
# roles; migration and seed SQL use migration-only console_app with BYPASSRLS.
set -euo pipefail

PG_SUPERUSER="${E2E_PG_SUPERUSER:-${USER}}"
PG_SUPERUSER_PASSWORD="${E2E_PG_SUPERUSER_PASSWORD:-}"
PG_HOST="${E2E_PG_HOST:-localhost}"
PG_PORT="${E2E_PG_PORT:-5432}"
DB_NAME="${E2E_DB_NAME:-console_e2e}"
ADMIN_URL="postgres://${PG_SUPERUSER}@${PG_HOST}:${PG_PORT}/postgres"
export PGPASSWORD="${PG_SUPERUSER_PASSWORD}"
CONSOLE_APP_POSTGRES_PASSWORD="${E2E_CONSOLE_APP_POSTGRES_PASSWORD:-console-e2e-owner-change-me}"
CONSOLE_RT_POSTGRES_PASSWORD="${E2E_CONSOLE_RT_POSTGRES_PASSWORD:-console-e2e-runtime-change-me}"
CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD="${E2E_CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD:-console-e2e-leave-command-change-me}"
CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD="${E2E_CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD:-console-e2e-ontology-command-change-me}"
CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD="${E2E_CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD:-console-e2e-platform-force-command-change-me}"
DATABASE_URL="postgres://console_app:${CONSOLE_APP_POSTGRES_PASSWORD}@${PG_HOST}:${PG_PORT}/${DB_NAME}"
CONSOLE_APP_PSQL_ARGS=(
  --host "${PG_HOST}"
  --port "${PG_PORT}"
  --username console_app
  --dbname "${DB_NAME}"
  --set ON_ERROR_STOP=1
)

owner_psql() {
  PGPASSWORD="${CONSOLE_APP_POSTGRES_PASSWORD}" psql "${CONSOLE_APP_PSQL_ARGS[@]}" "$@"
}

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BACKEND_DIR="${REPO_ROOT}/backend"
MIGRATIONS_DIR="${BACKEND_DIR}/crates/platform/db/migrations"
CONSOLE_APP_BIN="${CONSOLE_APP_BIN:-}"

run_source_migrations() {
  # Build/run from source: force sqlx offline so the apalis-postgres dep (and
  # our own queries) compile against the committed `.sqlx` cache, not the empty
  # console_e2e DB (which lacks `apalis.jobs` until migrations run).
  ( cd "${BACKEND_DIR}" && \
    CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}" \
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/.tmp/cargo-target-e2e}" \
    SQLX_OFFLINE=true CONSOLE_APP_ROLE=migrate DATABASE_URL="${DATABASE_URL}" cargo run -q -p console-app )
}

migration_file_count() {
  find "${MIGRATIONS_DIR}" -type f -name '*.sql' | wc -l | tr -d ' '
}

applied_migration_count() {
  owner_psql -Atqc "SELECT count(*) FROM _sqlx_migrations"
}

create_clean_db() {
  psql "${ADMIN_URL}" -v ON_ERROR_STOP=1 -q -c "DROP DATABASE IF EXISTS ${DB_NAME} WITH (FORCE);"
  psql "${ADMIN_URL}" -v ON_ERROR_STOP=1 -q -c "CREATE DATABASE ${DB_NAME};"
}

echo "db: dropping + recreating ${DB_NAME}" >&2
create_clean_db

echo "db: reconciling seven-role topology from cluster administrator" >&2
POSTGRES_HOST="${PG_HOST}" \
POSTGRES_PORT="${PG_PORT}" \
POSTGRES_DB="${DB_NAME}" \
POSTGRES_ADMIN_USER="${PG_SUPERUSER}" \
POSTGRES_ADMIN_PASSWORD="${PG_SUPERUSER_PASSWORD}" \
CONSOLE_APP_POSTGRES_PASSWORD="${CONSOLE_APP_POSTGRES_PASSWORD}" \
CONSOLE_RT_POSTGRES_PASSWORD="${CONSOLE_RT_POSTGRES_PASSWORD}" \
CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD="${CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD}" \
CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD="${CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD}" \
CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD="${CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD}" \
  "${REPO_ROOT}/ops/postgres-reconcile-topology.sh"

echo "db: applying migrations directly as console_app (CONSOLE_APP_ROLE=migrate)" >&2
if [[ -n "${CONSOLE_APP_BIN}" && -x "${CONSOLE_APP_BIN}" ]]; then
  newer_migration="$(find "${MIGRATIONS_DIR}" -type f -name '*.sql' -newer "${CONSOLE_APP_BIN}" -print -quit)"
  if [[ -n "${newer_migration}" ]]; then
    echo "db: ${CONSOLE_APP_BIN} is older than ${newer_migration#"${REPO_ROOT}/"}; using source migrations" >&2
    run_source_migrations
  else
    CONSOLE_APP_ROLE=migrate DATABASE_URL="${DATABASE_URL}" "${CONSOLE_APP_BIN}"

    expected_migrations="$(migration_file_count)"
    applied_migrations="$(applied_migration_count)"
    if [[ "${applied_migrations}" != "${expected_migrations}" ]]; then
      echo "db: ${CONSOLE_APP_BIN} applied ${applied_migrations}/${expected_migrations} checked-out migrations; recreating DB with source migrations" >&2
      create_clean_db
      run_source_migrations
    fi
  fi
else
  run_source_migrations
fi

echo "db: seeding tenant fixtures" >&2
owner_psql -q -f "$(dirname "${BASH_SOURCE[0]}")/seed.sql"

echo "db: seeding MECHANIC story fixtures" >&2
owner_psql -q -f "$(dirname "${BASH_SOURCE[0]}")/seed-mech.sql"

echo "db: seeding ADMIN/SUPER_ADMIN story fixtures" >&2
owner_psql -q -f "$(dirname "${BASH_SOURCE[0]}")/seed-admin.sql"

echo "db: seeding EXECUTIVE story fixtures" >&2
owner_psql -q -f "$(dirname "${BASH_SOURCE[0]}")/seed-exec.sql"

echo "db: seeding RECEPTIONIST story fixtures" >&2
owner_psql -q -f "$(dirname "${BASH_SOURCE[0]}")/seed-recp.sql"

echo "db: ready (user=console_app host=${PG_HOST} port=${PG_PORT} db=${DB_NAME}; password redacted)" >&2
