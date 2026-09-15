#!/usr/bin/env bash
# One-shot, separately authenticated operator transport. No app credentials.
set +x
set -euo pipefail
umask 077

# Opt-in only: the existing account-only command remains unchanged.
observer_profile=0
case "$#:${1:-}" in
  0:) ;;
  1:--with-durability-observer) observer_profile=1 ;;
  *) printf '%s\n' account_custody.invalid_profile >&2; exit 1 ;;
esac

fail() { printf '%s\n' "$1" >&2; exit 1; }
for name in POSTGRES_HOST POSTGRES_PORT POSTGRES_DB POSTGRES_ADMIN_USER \
  POSTGRES_ADMIN_PASSWORD_FILE PGSSLROOTCERT ACCOUNT_CUSTODY_EXPECTED_OPERATOR \
  ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER ACCOUNT_CUSTODY_EXPECTED_DATABASE \
  ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID ACCOUNT_CUSTODY_EXPECTED_TLS_HOST; do
  [[ -n "${!name:-}" ]] || fail account_custody.descriptor_invalid
done
for name in POSTGRES_DB POSTGRES_ADMIN_USER ACCOUNT_CUSTODY_EXPECTED_OPERATOR \
  ACCOUNT_CUSTODY_EXPECTED_DATABASE; do
  [[ "${!name}" =~ ^[a-zA-Z_][a-zA-Z0-9_]{0,62}$ ]] || fail account_custody.descriptor_invalid
done
[[ "${POSTGRES_HOST}" =~ ^[a-zA-Z0-9.:-]+$ ]] || fail account_custody.descriptor_invalid
[[ "${POSTGRES_HOST}" == "${ACCOUNT_CUSTODY_EXPECTED_TLS_HOST}" ]] || fail account_custody.target_mismatch
[[ "${POSTGRES_PORT}" =~ ^[1-9][0-9]{0,4}$ ]] && (( POSTGRES_PORT <= 65535 )) || fail account_custody.descriptor_invalid
[[ "${ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER}" =~ ^[1-9][0-9]{0,19}$ ]] || fail account_custody.descriptor_invalid
[[ "${ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID}" =~ ^[1-9][0-9]{0,9}$ ]] || fail account_custody.descriptor_invalid
[[ -f "${POSTGRES_ADMIN_PASSWORD_FILE}" && -r "${POSTGRES_ADMIN_PASSWORD_FILE}" ]] || fail account_custody.descriptor_invalid
[[ -f "${PGSSLROOTCERT}" && -r "${PGSSLROOTCERT}" ]] || fail account_custody.descriptor_invalid

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
psql_binary="$(command -v psql)" || fail account_custody.psql_unavailable
private_dir="$(mktemp -d /tmp/console-account-custody.XXXXXXXX)"
trap 'rm -rf -- "${private_dir}"' EXIT
trap 'exit 1' HUP INT TERM

# libpq never receives a password in argv or the child environment. The source
# may be a protected projected Secret; the actual passfile is private and owned
# by this process, independent of the source mount's symlink implementation.
password="$(cat -- "${POSTGRES_ADMIN_PASSWORD_FILE}")"
[[ -n "${password}" && "${password}" != *$'\n'* && "${password}" != *$'\r'* ]] || fail account_custody.descriptor_invalid
password="${password//\\/\\\\}"
password="${password//:/\\:}"
pass_host="${POSTGRES_HOST//:/\\:}"
printf '%s:%s:%s:%s:%s\n' "${pass_host}" "${POSTGRES_PORT}" "${POSTGRES_DB}" \
  "${POSTGRES_ADMIN_USER}" "${password}" >"${private_dir}/pgpass"
unset password pass_host

cat >"${private_dir}/preflight.sql" <<'SQL'
SELECT session_user=current_user AND session_user=:'expected_operator'
  AND (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname=session_user)
  AND session_user NOT IN ('console_app','console_rt','console_auth_rt',
    'console_leave_cmd','console_leave_definer','console_ontology_cmd',
    'console_ontology_writer','console_platform_force_cmd',
    'console_account_owner','console_terms_owner') AS operator_ok \gset
\if :operator_ok
\else
  DO $$ BEGIN RAISE EXCEPTION 'account_custody.operator_identity_mismatch'; END $$;
\endif
SELECT current_database()=:'expected_database'
  AND (SELECT oid::text FROM pg_catalog.pg_database WHERE datname=current_database())=:'expected_database_oid'
  AND (SELECT system_identifier::text FROM pg_catalog.pg_control_system())=:'expected_system_identifier'
  AS target_ok \gset
\if :target_ok
\else
  DO $$ BEGIN RAISE EXCEPTION 'account_custody.target_mismatch'; END $$;
\endif
SELECT EXISTS (SELECT 1 FROM pg_catalog.pg_class c
  JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
  WHERE n.nspname='public' AND c.relname='_sqlx_migrations' AND c.relkind='r'
    AND NOT c.relispartition AND pg_catalog.pg_get_userbyid(c.relowner)='console_app') AS ledger_exists \gset
\if :ledger_exists
\else
  DO $$ BEGIN RAISE EXCEPTION 'account_custody.migration_ledger_missing'; END $$;
\endif
WITH expected(version,checksum) AS (VALUES
SQL
previous=0
separator=''
while IFS=$'\t' read -r version digest extra; do
  [[ "${version}" =~ ^[1-9][0-9]{0,17}$ && "${digest}" =~ ^[0-9a-f]{96}$ && -z "${extra}" ]] || fail account_custody.migration_checksum_mismatch
  (( version > previous )) || fail account_custody.migration_checksum_mismatch
  printf '%s(%s,decode(\047%s\047,\047hex\047))\n' "${separator}" "${version}" "${digest}" >>"${private_dir}/preflight.sql"
  separator=','
  previous="${version}"
done <"${script_dir}/account-custody-migrations.sha384"
(( previous > 0 )) || fail account_custody.migration_checksum_mismatch
cat >>"${private_dir}/preflight.sql" <<'SQL'
)
SELECT NOT EXISTS (
  SELECT 1 FROM expected e FULL JOIN public._sqlx_migrations m USING(version)
  WHERE e.version IS NULL OR m.version IS NULL OR m.success IS DISTINCT FROM true
    OR m.checksum IS DISTINCT FROM e.checksum
) AND (SELECT count(*) FROM expected)=(SELECT count(*) FROM public._sqlx_migrations) AS ledger_ok \gset
\if :ledger_ok
\else
  DO $$ BEGIN RAISE EXCEPTION 'account_custody.migration_checksum_mismatch'; END $$;
\endif
SQL

installer_files=(--file "${script_dir}/postgres-finalize-account-custody.sql")
if [[ "$observer_profile" == 1 ]]; then
  installer_files+=(--file "${script_dir}/postgres-install-durability-observer.sql")
fi

# Clear all ambient libpq settings, service files, passwords and client keys.
# These fixed options take effect before the DO begins; its own SET cannot
# enforce a timeout on the already-running outer statement.
env -i LC_ALL=C \
  PGPASSFILE="${private_dir}/pgpass" PGSSLMODE=verify-full PGGSSENCMODE=disable PGSSLROOTCERT="${PGSSLROOTCERT}" \
  PGSSLCERT="${private_dir}/no-client-cert" PGSSLKEY="${private_dir}/no-client-key" \
  PGSERVICEFILE="${private_dir}/no-service" PGCONNECT_TIMEOUT=10 \
  PGOPTIONS='-c search_path=pg_catalog,pg_temp -c statement_timeout=60000 -c lock_timeout=5000' \
  "${psql_binary}" -X -w --quiet --set ON_ERROR_STOP=1 --single-transaction \
  --host "${POSTGRES_HOST}" --port "${POSTGRES_PORT}" \
  --username "${POSTGRES_ADMIN_USER}" --dbname "${POSTGRES_DB}" \
  --set "expected_operator=${ACCOUNT_CUSTODY_EXPECTED_OPERATOR}" \
  --set "expected_database=${ACCOUNT_CUSTODY_EXPECTED_DATABASE}" \
  --set "expected_database_oid=${ACCOUNT_CUSTODY_EXPECTED_DATABASE_OID}" \
  --set "expected_system_identifier=${ACCOUNT_CUSTODY_EXPECTED_SYSTEM_IDENTIFIER}" \
  --file "${private_dir}/preflight.sql" "${installer_files[@]}"
printf '%s\n' account_custody.finalized
