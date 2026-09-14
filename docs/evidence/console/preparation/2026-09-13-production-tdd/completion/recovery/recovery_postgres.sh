#!/usr/bin/env bash
# Local synthetic fixture only. Caller supplies a real repository test command.
set -euo pipefail
repo_root="${1:?usage: recovery_postgres.sh REPO -- COMMAND...}"; shift
[[ "${1:-}" == -- ]] || exit 2
shift
[[ $# -gt 0 ]] || exit 2
source "$repo_root/tools/lanes/no-credential-in-argv.sh" "$@"
image='postgres:18.6@sha256:4ef4dbc939d61acea57712655ddb4b4ab27419c913f94cca0cd57cb3ea3c2280'
# Require the image locally: this fixture never pulls or contacts a provider.
docker image inspect "$image" >/dev/null
umask 077
scratch="$(mktemp -d "${TMPDIR:-/tmp}/console-recovery.XXXXXX")"
run="console-recovery-$$-$(openssl rand -hex 4)"
primary="$run-p"; standby="$run-s"; seed="$run-seed"; network="$run-net"
pv="$run-p-data"; sv="$run-s-data"
relay_pid=''
echo "recovery-fixture run=$run evidence=$scratch"
cleanup() {
  code=$?
  trap - EXIT
  [[ -z "$relay_pid" ]] || kill "$relay_pid" 2>/dev/null || true
  docker rm -fv "$seed" "$standby" "$primary" >/dev/null 2>&1 || true
  docker volume rm "$sv" "$pv" >/dev/null 2>&1 || true
  docker network rm "$network" >/dev/null 2>&1 || true
  # Keep safe evidence, remove ephemeral credentials only.
  rm -f "$scratch/container.env" "$scratch/pgpass"
  if docker ps -aq --filter "label=console.recovery.run=$run" | rg -q .; then
    echo 'FAIL: owned fixture container leaked' >&2; code=1
  fi
  if docker volume ls -q --filter "label=console.recovery.run=$run" | rg -q .; then
    echo 'FAIL: owned fixture volume leaked' >&2; code=1
  fi
  if docker network ls -q --filter "label=console.recovery.run=$run" | rg -q .; then
    echo 'FAIL: owned fixture network leaked' >&2; code=1
  fi
  echo "recovery-fixture evidence=$scratch exit=$code"
  exit "$code"
}
trap cleanup EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
pw() { openssl rand -hex 32; }
admin="$(pw)"; app="$(pw)"; rt="$(pw)"; leave="$(pw)"; ont="$(pw)"; force="$(pw)"; repl="$(pw)"
{
  printf 'POSTGRES_DB=console_recovery\nPOSTGRES_USER=console_buck_admin\nPOSTGRES_PASSWORD=%s\n' "$admin"
  printf 'POSTGRES_HOST=127.0.0.1\nPOSTGRES_PORT=5432\nPOSTGRES_ADMIN_USER=console_buck_admin\nPOSTGRES_ADMIN_PASSWORD=%s\n' "$admin"
  printf 'CONSOLE_APP_POSTGRES_PASSWORD=%s\nCONSOLE_RT_POSTGRES_PASSWORD=%s\n' "$app" "$rt"
  printf 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=%s\n' "$leave" "$ont" "$force"
  printf 'CONSOLE_RECOVERY_REPLICATION_PASSWORD=%s\n' "$repl"
} > "$scratch/container.env"
docker network create --driver bridge --opt com.docker.network.bridge.enable_ip_masquerade=false --label "console.recovery.run=$run" "$network" >/dev/null
docker volume create --label "console.recovery.run=$run" "$pv" >/dev/null
docker volume create --label "console.recovery.run=$run" "$sv" >/dev/null
docker run -d --name "$primary" --label "console.recovery.run=$run" --network "$network" \
  --network-alias primary -p 127.0.0.1::5432 --env-file "$scratch/container.env" \
  -v "$pv:/var/lib/postgresql" "$image" \
  -c fsync=on -c full_page_writes=on -c synchronous_commit=remote_apply \
  -c wal_level=replica -c max_wal_senders=4 -c max_replication_slots=2 \
  -c max_slot_wal_keep_size=128MB -c hot_standby=on \
  -c log_statement=none -c log_min_error_statement=panic \
  -c log_parameter_max_length=0 -c log_parameter_max_length_on_error=0 >/dev/null
ready() {
  local c="$1"
  for ((i=0;i<60;i++)); do
    if [[ "$(docker exec "$c" sh -c 'cat /proc/1/comm' 2>/dev/null || true)" == postgres ]] && \
      docker exec "$c" pg_isready -U console_buck_admin -d console_recovery >/dev/null 2>&1; then return; fi
    sleep 1
  done
  echo "FAIL: fixture readiness deadline for $c" >&2; return 1
}
ready "$primary"
docker cp "$repo_root/ops/postgres-reconcile-topology.sh" "$primary:/topology.sh" >/dev/null
docker cp "$scratch/container.env" "$primary:/topology.env" >/dev/null
docker exec "$primary" sh -ceu 'set -a; . /topology.env; exec bash /topology.sh' > "$scratch/topology.log" 2>&1
docker exec -i "$primary" sh -ceu 'export PGPASSWORD="$POSTGRES_PASSWORD"; exec psql -X -v ON_ERROR_STOP=1 -U console_buck_admin -d console_recovery' <<'SQL' > "$scratch/replication-setup.log" 2>&1
\getenv replica_password CONSOLE_RECOVERY_REPLICATION_PASSWORD
CREATE ROLE console_fixture_replica LOGIN REPLICATION PASSWORD :'replica_password';
SELECT pg_create_physical_replication_slot('console_recovery_s1');
SQL
# The generated HBA includes password-protected replication for the private network.
docker exec "$primary" sh -ceu 'printf "host replication console_fixture_replica all scram-sha-256\n" >> "$PGDATA/pg_hba.conf"'
docker exec "$primary" sh -ceu 'export PGPASSWORD="$POSTGRES_PASSWORD"; psql -X -U console_buck_admin -d console_recovery -c "SELECT pg_reload_conf()"' >/dev/null
# Libpq passfile is mounted only into the local seed and standby containers.
printf 'primary:5432:*:console_fixture_replica:%s\n' "$repl" > "$scratch/pgpass"
docker create --name "$seed" --label "console.recovery.run=$run" --network "$network" \
  -v "$sv:/var/lib/postgresql" \
  --entrypoint bash "$image" -ceu '
    mkdir -p /var/lib/postgresql/18/docker
    cp /fixture.pgpass /var/lib/postgresql/replication.pgpass
    chmod 600 /var/lib/postgresql/replication.pgpass
    chown -R postgres:postgres /var/lib/postgresql
    gosu postgres pg_basebackup -D /var/lib/postgresql/18/docker -R -X stream --checkpoint=fast \
      --slot=console_recovery_s1 \
      --dbname="host=primary port=5432 user=console_fixture_replica application_name=console_recovery_s1 passfile=/var/lib/postgresql/replication.pgpass"
  ' > "$scratch/seed-create.log" 2>&1
docker cp "$scratch/pgpass" "$seed:/fixture.pgpass" >/dev/null
docker start --attach "$seed" > "$scratch/basebackup.log" 2>&1
docker run -d --name "$standby" --label "console.recovery.run=$run" --network "$network" \
  -p 127.0.0.1::5432 --env-file "$scratch/container.env" -v "$sv:/var/lib/postgresql" "$image" \
  -c fsync=on -c full_page_writes=on -c synchronous_commit=remote_apply -c hot_standby=on \
  -c wal_level=replica -c max_wal_senders=4 -c max_replication_slots=2 \
  -c log_statement=none -c log_min_error_statement=panic \
  -c log_parameter_max_length=0 -c log_parameter_max_length_on_error=0 >/dev/null
ready "$standby"
for ((i=0;i<60;i++)); do
  seen="$(docker exec "$primary" sh -ceu 'export PGPASSWORD="$POSTGRES_PASSWORD"; psql -XAt -U console_buck_admin -d console_recovery -c "SELECT count(*) FROM pg_stat_replication WHERE application_name='"'"'console_recovery_s1'"'"' AND state='"'"'streaming'"'"'"')"
  [[ "$seen" == 1 ]] && break
  [[ $i -lt 59 ]] || { echo 'FAIL: no exact streaming standby' >&2; exit 1; }
  sleep 1
done
docker exec -i "$primary" sh -ceu 'export PGPASSWORD="$POSTGRES_PASSWORD"; exec psql -X -v ON_ERROR_STOP=1 -U console_buck_admin -d console_recovery' <<'SQL' > "$scratch/durability-config.log" 2>&1
ALTER SYSTEM SET synchronous_standby_names = 'FIRST 1 (console_recovery_s1)';
ALTER SYSTEM SET synchronous_commit = 'remote_apply';
SELECT pg_reload_conf();
SQL
pp="$(docker port "$primary" 5432/tcp)"; pp="${pp##*:}"
sp="$(docker port "$standby" 5432/tcp)"; sp="${sp##*:}"
export DATABASE_URL="postgres://console_buck_admin:$admin@127.0.0.1:$pp/console_recovery?options%5Bconsole.sqlx_test_bootstrap%5D=buck-sqlx-superuser-v1"
export CONSOLE_APALIS_ADMIN_DATABASE_URL="$DATABASE_URL"
export CONSOLE_APALIS_OWNER_DATABASE_URL="postgres://console_app:$app@127.0.0.1:$pp/console_recovery"
export CONSOLE_APALIS_RUNTIME_DATABASE_URL="postgres://console_rt:$rt@127.0.0.1:$pp/console_recovery"
export CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL="postgres://console_leave_cmd:$leave@127.0.0.1:$pp/console_recovery"
export CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL="postgres://console_ontology_cmd:$ont@127.0.0.1:$pp/console_recovery"
export CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL="postgres://console_platform_force_cmd:$force@127.0.0.1:$pp/console_recovery"
export CONSOLE_RECOVERY_STANDBY_PORT="$sp"
export CONSOLE_RECOVERY_PRIMARY="$primary" CONSOLE_RECOVERY_STANDBY="$standby" CONSOLE_RECOVERY_RUN="$run"
export CONSOLE_RECOVERY_CONTROL="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/recovery_control.sh"
export CONSOLE_RECOVERY_EVIDENCE="$scratch" SQLX_OFFLINE=true RUST_TEST_THREADS=1
"$CONSOLE_RECOVERY_CONTROL" assert-topology
# Canonical enforcement has its own disposable migrated DB, preserves SQLx bootstrap.
bash "$repo_root/backend/ci/gates/writer-ownership/canonical-enforce.sh" "$repo_root" "$primary" "canonical_probe_$$" > "$scratch/ownership.log" 2>&1
docker image inspect --format '{{.Id}} {{.Os}} {{.Architecture}}' "$image" > "$scratch/image.txt"
git -C "$repo_root" rev-parse HEAD > "$scratch/source-sha.txt"
if [[ -n "${CONSOLE_RECOVERY_CUT:-}" ]]; then
  case "$CONSOLE_RECOVERY_CUT" in before-send|after-commit-response) ;; *) exit 2 ;; esac
  python3 "$(dirname "$CONSOLE_RECOVERY_CONTROL")/commit_relay.py" \
    --upstream-port "$pp" --cut "$CONSOLE_RECOVERY_CUT" \
    --events "$scratch/relay.jsonl" --port-file "$scratch/relay.port" \
    > "$scratch/relay-process.log" 2>&1 &
  relay_pid=$!
  for ((i=0;i<50;i++)); do
    [[ -s "$scratch/relay.port" ]] && break
    kill -0 "$relay_pid" 2>/dev/null || { echo 'FAIL: relay exited'; exit 1; }
    sleep .1
  done
  [[ -s "$scratch/relay.port" ]] || { echo 'FAIL: relay startup deadline'; exit 1; }
  CONSOLE_RECOVERY_RELAY_PORT="$(< "$scratch/relay.port")"
  export CONSOLE_RECOVERY_RELAY_PORT
fi
cd "$repo_root"
"$@" 2>&1 | tee "$scratch/test.log"
