#!/usr/bin/env bash
set -euo pipefail
set -a
. /experiment/secrets.env
set +a
admin() { PGPASSWORD="$POSTGRES_ADMIN_PASSWORD" psql -X -h127.0.0.1 -U "$POSTGRES_ADMIN_USER" -v ON_ERROR_STOP=1 -q "$@"; }
app() { PGPASSWORD="$CONSOLE_APP_POSTGRES_PASSWORD" psql -X -h127.0.0.1 -U console_app -v ON_ERROR_STOP=1 -q "$@"; }
admin -d postgres -c 'CREATE DATABASE db_a'
POSTGRES_DB=db_a bash /experiment/topology.sh
admin -d db_a <<'SQL'
CREATE ROLE console_account_owner NOLOGIN NOSUPERUSER NOBYPASSRLS NOINHERIT NOCREATEDB NOCREATEROLE NOREPLICATION;
CREATE ROLE console_terms_owner NOLOGIN NOSUPERUSER NOBYPASSRLS NOINHERIT NOCREATEDB NOCREATEROLE NOREPLICATION;
SQL
for file in /experiment/migrations/*.sql; do app -d db_a -f "$file" >/dev/null; done
echo 'PASS db_a225legacySQL'
POSTGRES_DB=db_a CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES=1 bash /experiment/topology.sh
admin -d db_a -At -f /experiment/legacy-snapshot.sql > /experiment/legacy-before.json
app -d db_a -1 -f /experiment/staged-schema.sql
if app -d db_a -c 'ALTER TABLE public.accounts OWNER TO console_account_owner' > /experiment/app-owner-refusal.log 2>&1; then
  echo 'FAIL app ownership transfer unexpectedly allowed'; exit 1
fi
grep -q 'must be able to SET ROLE' /experiment/app-owner-refusal.log
echo 'PASS real console_app ownership transfer refused'
if admin -d db_a -1 -f /experiment/transfer-fail.sql > /experiment/transfer-fail.log 2>&1; then
  echo 'FAIL injection unexpectedly succeeded'; exit 1
fi
grep -q 'division by zero' /experiment/transfer-fail.log
admin -d db_a -f /experiment/check-rolled-back.sql
echo 'PASS injected failure rolls back all6ownerships'
admin -d db_a -1 -f /experiment/transfer.sql
admin -d db_a -f /experiment/check-custody.sql
admin -d db_a -At -f /experiment/custody-snapshot.sql > /experiment/custody-before.json
admin -d db_a -1 -f /experiment/transfer.sql
POSTGRES_DB=db_a CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES=1 bash /experiment/topology.sh
admin -d db_a -f /experiment/check-custody.sql
admin -d db_a -At -f /experiment/custody-snapshot.sql > /experiment/custody-after.json
cmp /experiment/custody-before.json /experiment/custody-after.json
admin -d db_a -At -f /experiment/legacy-snapshot.sql > /experiment/legacy-after.json
cmp /experiment/legacy-before.json /experiment/legacy-after.json
echo 'PASS finalizer+base topology replay preserve custody+legacy catalog'
for table in accounts account_security account_security_events account_terms_acceptances account_terms_head account_terms_release_receipts; do
 if PGPASSWORD="$CONSOLE_RT_POSTGRES_PASSWORD" psql -X -h127.0.0.1 -U console_rt -d db_a -v ON_ERROR_STOP=1 -q -c "SELECT * FROM public.$table LIMIT 0" > /experiment/refusal-$table.log 2>&1; then echo 'FAIL runtime read'; exit 1; fi
 grep -q 'permission denied' /experiment/refusal-$table.log
done
echo 'PASS six actual runtime LOGIN privilege refusals'
admin -d postgres -c 'CREATE DATABASE db_b'
POSTGRES_DB=db_b bash /experiment/topology.sh
for file in /experiment/migrations/*.sql; do app -d db_b -f "$file" >/dev/null; done
POSTGRES_DB=db_b CONSOLE_TOPOLOGY_REQUIRE_CANONICAL_TABLES=1 bash /experiment/topology.sh
admin -d db_a -f /experiment/check-custody.sql
echo 'PASS db_b225legacySQL after db_a custodyfinalization; db_a custodyretained'
echo 'EXPERIMENT PASS; SQL files only, no SQLx ledger or application startup claim'
