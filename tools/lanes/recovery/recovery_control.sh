#!/usr/bin/env bash
# Exact owned-container controls. No arbitrary SQL, endpoint or container argument.
set -euo pipefail
run="${CONSOLE_RECOVERY_RUN:?}"
p="${CONSOLE_RECOVERY_PRIMARY:?}"
s="${CONSOLE_RECOVERY_STANDBY:?}"
[[ "$p" == "$run-p" && "$s" == "$run-s" && "$run" == console-recovery-* ]] || exit 2
for c in "$p" "$s"; do
  [[ "$(docker inspect --format '{{index .Config.Labels "console.recovery.run"}}' "$c")" == "$run" ]] || exit 2
done
psql_in() {
  docker exec -i "$1" sh -ceu 'export PGPASSWORD="$POSTGRES_PASSWORD"; exec psql -XAt -v ON_ERROR_STOP=1 -U console_buck_admin -d console_recovery'
}
case "${1:-}" in
  pause-replay)
    psql_in "$s" <<'SQL'
SELECT pg_wal_replay_pause();
SQL
    for ((i=0;i<50;i++)); do
      state="$(psql_in "$s" <<<'SELECT pg_get_wal_replay_pause_state();')"
      [[ "$state" == paused ]] && exit 0
      sleep .1
    done
    exit 1 ;;
  resume-replay) psql_in "$s" <<<'SELECT pg_wal_replay_resume();' ;;
  assert-topology)
    state="$(psql_in "$p" <<'SQL'
SELECT current_setting('fsync')='on' AND current_setting('full_page_writes')='on'
AND current_setting('synchronous_commit')='remote_apply'
AND current_setting('synchronous_standby_names')='FIRST 1 (console_recovery_s1)'
AND NOT pg_is_in_recovery()
AND (SELECT count(*)=1 FROM pg_stat_replication WHERE application_name='console_recovery_s1' AND state='streaming' AND sync_state='sync');
SQL
    )"
    [[ "$state" == t ]] || { echo 'FAIL: primary durability topology'; exit 1; }
    state="$(psql_in "$s" <<<'SELECT pg_is_in_recovery() AND current_setting('\''fsync'\'')='\''on'\'' AND current_setting('\''full_page_writes'\'')='\''on'\'';')"
    [[ "$state" == t ]] || { echo 'FAIL: standby durability topology'; exit 1; }
    echo 'topology: PG primary plus exact synchronous standby; remote_apply enabled' ;;
  watermarks)
    psql_in "$p" <<'SQL'
SELECT 'primary', pg_current_wal_flush_lsn(), timeline_id FROM pg_control_checkpoint();
SELECT 'standby', application_name, sync_state, flush_lsn, replay_lsn FROM pg_stat_replication;
SQL
    psql_in "$s" <<'SQL'
SELECT 'replay', pg_last_wal_replay_lsn(), timeline_id FROM pg_control_checkpoint();
SQL
    ;;
  *) echo 'usage: recovery_control.sh pause-replay|resume-replay|assert-topology|watermarks' >&2; exit 2 ;;
esac
