#!/usr/bin/env bash
# EXPERIMENT X4 -- FEASIBILITY PROBE. NOT A CI GATE, NOT PRODUCTION CODE.
#
# Container/hygiene recipe copied from tools/lanes/pgtest.sh. It diverges in one
# way that matters: it keeps the generated console_rt password so the assertions
# can run as the genuine NOSUPERUSER NOBYPASSRLS runtime role
# (ops/postgres-reconcile-topology.sh:303). A superuser bypasses RLS and would
# produce a meaningless GREEN.
#
# Runs the KNOWN-BAD CONTROLS FIRST and requires them to leak. A probe with no
# demonstrated failure mode is not evidence.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$here/../../../.." && pwd)"

image="postgres:18.4@sha256:65f70a152846cf504dff86e807007e9aeac98c3aeb7b62541b2c55ab9d264e56"
name="x4probe-$$"
db="x4probe_$$"

ORG_A='aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'
ORG_B='bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb'
PARTY1='11111111-1111-4111-8111-111111111111'   # the human at BOTH orgs
PARTY2='22222222-2222-4222-8222-222222222222'   # visible only to org B

envf=""
# Hygiene asserted on THIS run's own container only, per LANE-PROTOCOL.md:152-154.
cleanup() {
  docker rm -fv "$name" >/dev/null 2>&1 || true
  [ -n "$envf" ] && rm -f "$envf" 2>/dev/null
  local leaked
  leaked="$(docker ps -aq --filter "name=^${name}$" 2>/dev/null | wc -l | tr -d ' ')"
  if [ "$leaked" != "0" ]; then echo "LEAK: container ${name} survived cleanup" >&2
  else echo "clean: ${name} removed with its volume"; fi
}
trap cleanup EXIT

pw() { openssl rand -hex 32; }
admin="$(pw)"; app="$(pw)"; rt="$(pw)"; leave="$(pw)"; ont="$(pw)"; force="$(pw)"

umask 077
envf="$(mktemp "${TMPDIR:-/tmp}/x4probe-pg.XXXXXX")"; chmod 600 "$envf"
{
  printf 'POSTGRES_DB=%s\nPOSTGRES_USER=console_buck_admin\nPOSTGRES_PASSWORD=%s\n' "$db" "$admin"
  printf 'POSTGRES_HOST=127.0.0.1\nPOSTGRES_PORT=5432\nPOSTGRES_ADMIN_USER=console_buck_admin\nPOSTGRES_ADMIN_PASSWORD=%s\n' "$admin"
  printf 'CONSOLE_APP_POSTGRES_PASSWORD=%s\nCONSOLE_RT_POSTGRES_PASSWORD=%s\n' "$app" "$rt"
  printf 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=%s\n' "$leave" "$ont" "$force"
} >"$envf"

docker run -d --rm --name "$name" -p 127.0.0.1::5432 --env-file "$envf" "$image" >/dev/null
docker cp "$repo_root/ops/postgres-reconcile-topology.sh" "$name:/topology.sh" >/dev/null
docker cp "$envf" "$name:/topology.env" >/dev/null
for i in $(seq 1 40); do
  if [ "$(docker exec "$name" cat /proc/1/comm 2>/dev/null || true)" = "postgres" ] \
     && docker exec "$name" pg_isready -h 127.0.0.1 -U console_buck_admin -d "$db" >/dev/null 2>&1; then break; fi
  [ "$i" = 40 ] && { echo "postgres never became healthy"; exit 1; }
  sleep 1
done
docker exec "$name" sh -ceu 'set -a; . /topology.env; exec bash /topology.sh' >/dev/null
echo "topology reconciled; console_rt exists as NOSUPERUSER NOBYPASSRLS"

# --- role identity proof: everything below runs as a non-superuser ------------
as_admin() { docker exec -i -e PGPASSWORD="$admin" "$name" \
  psql -X -q -h 127.0.0.1 -U console_buck_admin -d "$db" -v ON_ERROR_STOP=1 "$@"; }
# console_rt is NOINHERIT/NOBYPASSRLS; -Atq gives bare parseable rows.
as_rt() { docker exec -i -e PGPASSWORD="$rt" "$name" \
  psql -X -Atq -F '|' -h 127.0.0.1 -U console_rt -d "$db" -v ON_ERROR_STOP=1; }
# no ON_ERROR_STOP: used where an error IS the expected result.
as_rt_err() { docker exec -i -e PGPASSWORD="$rt" "$name" \
  psql -X -Atq -F '|' -h 127.0.0.1 -U console_rt -d "$db" 2>&1; }

as_admin -f - <"$here/probe.sql" >/dev/null
echo "probe schema applied"
echo

fails=0
check() { # label expected actual
  if [ "$2" = "$3" ]; then printf 'PASS  %-58s %s\n' "$1" "$3"
  else printf 'FAIL  %-58s expected=%s actual=%s\n' "$1" "$2" "$3"; fails=$((fails+1)); fi
}

echo "=== role identity: the assertions must NOT be running as a superuser ==="
identity="$(printf "SELECT session_user, current_setting('is_superuser'), rolbypassrls FROM pg_roles WHERE rolname = session_user;" | as_rt)"
echo "  $identity"
check "session_user|is_superuser|rolbypassrls" "console_rt|off|f" "$identity"
echo

echo "############################################################"
echo "### KNOWN-BAD CONTROLS -- these MUST be observed RED     ###"
echo "############################################################"
echo
echo "--- CONTROL 1: identical edge table, RLS never enabled ---"
echo "    armed as org A, asking for org B's edge. MUST return B's row."
c1_sql="SET app.current_org = '$ORG_A';
SELECT org_id || ' ' || party_id || ' ' || relationship_kind
FROM x4probe_edge_control_norls WHERE org_id = '$ORG_B' ORDER BY valid_from;"
echo "    \$ psql -U console_rt <<'SQL'"
printf '%s\n' "$c1_sql" | sed 's/^/      /'
c1_out="$(printf '%s\n' "$c1_sql" | as_rt)"
echo "    OUTPUT:"; printf '%s\n' "$c1_out" | sed 's/^/      /'
c1_count="$(printf '%s\n' "$c1_out" | grep -c "$ORG_B" || true)"
check "CONTROL 1 leaks org B's edges to org A (RED expected)" "2" "$c1_count"
echo

echo "--- CONTROL 2: the 0060:99 shape -- definer trusting a parameter ---"
echo "    armed as org A, passing org B's id. MUST leak B's edges."
c2_sql="SET app.current_org = '$ORG_A';
SELECT party_id || ' ' || relationship_kind FROM x4probe_resolve_leaky('$ORG_B');"
echo "    \$ psql -U console_rt <<'SQL'"
printf '%s\n' "$c2_sql" | sed 's/^/      /'
c2_out="$(printf '%s\n' "$c2_sql" | as_rt)"
echo "    OUTPUT:"; printf '%s\n' "$c2_out" | sed 's/^/      /'
c2_count="$(printf '%s\n' "$c2_out" | grep -c . || true)"
check "CONTROL 2 leaks via parameter-trusting definer (RED expected)" "2" "$c2_count"
echo

echo "--- CONTROL 3: correct RLS, but UNIQUE key omits org_id ---"
echo "    armed as org A, insert a row colliding with B's INVISIBLE row."
echo "    MUST raise 23505 -- proving the unique index leaks edge existence"
echo "    from below RLS when org_id does not lead the key."
c3_sql="SET app.current_org = '$ORG_A';
INSERT INTO x4probe_edge_control_uniqleak (org_id, party_id, relationship_kind, valid_from, reason)
VALUES ('$ORG_A', '$PARTY1', 'EMPLOYMENT', '2026-02-01T00:00:00Z', 'probing for B row');"
echo "    \$ psql -U console_rt <<'SQL'"
printf '%s\n' "$c3_sql" | sed 's/^/      /'
c3_out="$(printf '%s\n' "$c3_sql" | as_rt_err)"
echo "    OUTPUT:"; printf '%s\n' "$c3_out" | sed 's/^/      /'
check "CONTROL 3 unique index leaks B's edge existence (RED expected)" "1" \
  "$(printf '%s\n' "$c3_out" | grep -c 'duplicate key value violates unique constraint' || true)"
echo

if [ "$fails" -ne 0 ]; then
  echo "!!! CONTROLS DID NOT LEAK. The harness is not exercising RLS."
  echo "!!! Every result below would be void. Stopping, per the brief."
  exit 1
fi
echo "controls leaked as required -- the harness demonstrably exercises RLS."
echo

echo "############################################################"
echo "### THE THREE ASSERTIONS (Variant A: no definer at all)  ###"
echo "############################################################"
echo
echo "--- 1. armed as org A: party resolves, only A's edge visible ---"
a1_sql="SET app.current_org = '$ORG_A';
SELECT p.id || ' ' || p.party_kind || ' ' || v.org_id || ' ' || v.relationship_kind
FROM x4probe_party p JOIN x4probe_party_org_visibility v ON v.party_id = p.id
ORDER BY v.valid_from;"
printf '%s\n' "$a1_sql" | sed 's/^/    /'
a1_out="$(printf '%s\n' "$a1_sql" | as_rt)"
echo "  OUTPUT:"; printf '%s\n' "$a1_out" | sed 's/^/    /'
check "A resolves the party" "1" "$(printf '%s\n' "$a1_out" | grep -c "$PARTY1" || true)"
check "A sees exactly one edge" "1" "$(printf '%s\n' "$a1_out" | grep -c . || true)"
check "A sees no edge of org B" "0" "$(printf '%s\n' "$a1_out" | grep -c "$ORG_B" || true)"
echo

echo "--- 2. armed as org B: the SAME party resolves, only B's edge visible ---"
a2_sql="SET app.current_org = '$ORG_B';
SELECT p.id || ' ' || p.party_kind || ' ' || v.org_id || ' ' || v.relationship_kind
FROM x4probe_party p JOIN x4probe_party_org_visibility v ON v.party_id = p.id
WHERE p.id = '$PARTY1' ORDER BY v.valid_from;"
printf '%s\n' "$a2_sql" | sed 's/^/    /'
a2_out="$(printf '%s\n' "$a2_sql" | as_rt)"
echo "  OUTPUT:"; printf '%s\n' "$a2_out" | sed 's/^/    /'
check "B resolves the same party id" "1" "$(printf '%s\n' "$a2_out" | grep -c "$PARTY1" || true)"
check "B sees exactly one edge for that party" "1" "$(printf '%s\n' "$a2_out" | grep -c . || true)"
check "B sees no edge of org A" "0" "$(printf '%s\n' "$a2_out" | grep -c "$ORG_A" || true)"
echo

echo "--- 3. CONFIDENTIALITY: armed as A, no query may reveal B's edge exists ---"
echo "    DN-0003:84-86 requires denied data omitted \"including counts and"
echo "    relationship existence\", so a leaking COUNT is a failure."
echo
run3() { # label expected sql
  local out
  out="$(printf "SET app.current_org = '$ORG_A';\n%s\n" "$3" | as_rt)"
  printf '  q: %s\n' "$3"
  printf '     -> %s\n' "${out:-<empty>}"
  check "$1" "$2" "$out"
}
run3 "3a unqualified COUNT of edges"                "1" "SELECT count(*) FROM x4probe_party_org_visibility;"
run3 "3b COUNT of edges for the shared party"       "1" "SELECT count(*) FROM x4probe_party_org_visibility WHERE party_id = '$PARTY1';"
run3 "3c COUNT filtered to org B explicitly"        "0" "SELECT count(*) FROM x4probe_party_org_visibility WHERE org_id = '$ORG_B';"
run3 "3d COUNT DISTINCT org_id (cardinality probe)" "1" "SELECT count(DISTINCT org_id) FROM x4probe_party_org_visibility;"
run3 "3e EXISTS probe for B's edge"                 "f" "SELECT EXISTS (SELECT 1 FROM x4probe_party_org_visibility WHERE org_id = '$ORG_B');"
run3 "3f correlated EXISTS via the party row"       "0" "SELECT count(*) FROM x4probe_party p WHERE EXISTS (SELECT 1 FROM x4probe_party_org_visibility v WHERE v.party_id = p.id AND v.org_id = '$ORG_B');"
run3 "3g max(valid_from) -- aggregate side channel" "2026-01-01 00:00:00+00" "SELECT max(valid_from)::text FROM x4probe_party_org_visibility;"
run3 "3h 'does anyone ELSE employ my employee?'"    "0" "SELECT count(*) FROM x4probe_party_org_visibility WHERE party_id = '$PARTY1' AND org_id <> '$ORG_A';"
echo "     (0 = A learns nothing. This is the confidentiality question stated"
echo "      directly: company A must not learn its employee also works at B.)"
echo

echo "--- 3h-truth: prove the hidden row REALLY EXISTS while A cannot see it ---"
echo "    Without this, 3b/3h could pass simply because there is no second edge."
truth="$(as_admin -Atq -c "SELECT count(*) FROM x4probe_party_org_visibility WHERE party_id = '$PARTY1';")"
seen="$(printf "SET app.current_org = '$ORG_A';\nSELECT count(*) FROM x4probe_party_org_visibility WHERE party_id = '$PARTY1';" | as_rt)"
printf '  ground truth (superuser, RLS bypassed): %s edges for the shared party\n' "$truth"
printf '  visible to console_rt armed as org A  : %s\n' "$seen"
check "3h-truth two edges exist in the table" "2" "$truth"
check "3h-truth org A can see only one of them" "1" "$seen"
echo

echo "--- 3i UNIQUE-constraint side channel ---"
echo "    The SAME insert that CONTROL 3 just leaked 23505 on, now against the"
echo "    real key from §4.1:527 -- UNIQUE (org_id, party_id, relationship_kind,"
echo "    valid_from). org_id leads, so it must be accepted, revealing nothing."
u_sql="SET app.current_org = '$ORG_A';
BEGIN;
INSERT INTO x4probe_party_org_visibility (org_id, party_id, relationship_kind, valid_from, reason)
VALUES ('$ORG_A', '$PARTY1', 'EMPLOYMENT', '2026-02-01T00:00:00Z', 'collides with B row but for org A');
SELECT 'insert-accepted-no-collision';
ROLLBACK;"
printf '%s\n' "$u_sql" | sed 's/^/    /'
u_out="$(printf '%s\n' "$u_sql" | as_rt_err)"
echo "  OUTPUT:"; printf '%s\n' "$u_out" | sed 's/^/    /'
check "3i no 23505 leak of B's edge existence" "1" "$(printf '%s\n' "$u_out" | grep -c 'insert-accepted-no-collision' || true)"
echo

echo "--- 3j WITH CHECK blocks forging an edge for org B ---"
w_sql="SET app.current_org = '$ORG_A';
INSERT INTO x4probe_party_org_visibility (org_id, party_id, relationship_kind, reason)
VALUES ('$ORG_B', '$PARTY1', 'EMPLOYMENT', 'forged by org A');"
printf '%s\n' "$w_sql" | sed 's/^/    /'
w_out="$(printf '%s\n' "$w_sql" | as_rt_err)"
echo "  OUTPUT:"; printf '%s\n' "$w_out" | sed 's/^/    /'
check "3j row-security violation, not a data-bearing error" "1" \
  "$(printf '%s\n' "$w_out" | grep -c 'new row violates row-level security policy' || true)"
echo

echo "--- 3k UPDATE/DELETE cannot reach B's edge ---"
ud_out="$(printf "SET app.current_org = '$ORG_A';
WITH u AS (UPDATE x4probe_party_org_visibility SET reason = 'tampered' WHERE org_id = '$ORG_B' RETURNING 1)
SELECT count(*) FROM u;" | as_rt)"
printf '     UPDATE ... WHERE org_id = B -> rows affected: %s\n' "$ud_out"
check "3k UPDATE touches zero of B's rows" "0" "$ud_out"
echo

echo "=== THE HONEST PART: what Variant A DOES expose ==="
echo "  party has a console_rt grant and no RLS, so probe what A learns from it."
pa_out="$(printf "SET app.current_org = '$ORG_A';\nSELECT count(*) FROM x4probe_party;" | as_rt)"
printf '  q: SELECT count(*) FROM x4probe_party;  -> %s\n' "$pa_out"
echo "  Two parties exist platform-wide; org A holds an edge to only ONE."
if [ "$pa_out" = "2" ]; then
  echo "  OBSERVED: A can count parties it holds no edge to (platform cardinality)."
  echo "  This is NOT the confidential fact of §4.2:627 (which parties A holds"
  echo "  edges to) -- but it is a real cross-tenant aggregate, reported as a"
  echo "  finding, not hidden. It is exactly what §4.1:506 'definer-mediated'"
  echo "  and §4.2:630-631 'no console_rt grant' close. See Variant B."
fi
pb_out="$(printf "SET app.current_org = '$ORG_A';\nSELECT count(*) FROM x4probe_party WHERE id = '$PARTY2';" | as_rt)"
printf '  q: SELECT count(*) FROM x4probe_party WHERE id = PARTY2 -> %s\n' "$pb_out"
echo "  Note: reveals the party EXISTS, still not that org B holds an edge to it."
echo

echo "############################################################"
echo "### VARIANT B: the plan as literally written (definer)   ###"
echo "############################################################"
echo "  §4.1:506 'platform, definer-mediated' + §4.2:630-631 'no console_rt"
echo "  grant, never directly readable'. Revoke the grant and retest."
as_admin -c "REVOKE SELECT ON x4probe_party FROM console_rt;" >/dev/null
echo
direct_out="$(printf "SET app.current_org = '$ORG_A';\nSELECT count(*) FROM x4probe_party;" | as_rt_err)"
echo "  q: SELECT count(*) FROM x4probe_party;  (grant revoked)"
printf '     -> %s\n' "$(printf '%s\n' "$direct_out" | tr '\n' ' ')"
check "B1 party is not directly readable once the grant is gone" "1" \
  "$(printf '%s\n' "$direct_out" | grep -c 'permission denied for table x4probe_party' || true)"

join_out="$(printf "SET app.current_org = '$ORG_A';
SELECT count(*) FROM x4probe_party p JOIN x4probe_party_org_visibility v ON v.party_id = p.id;" | as_rt_err)"
echo "  q: the plain edge JOIN, with no grant on party"
printf '     -> %s\n' "$(printf '%s\n' "$join_out" | tr '\n' ' ')"
check "B2 the no-definer join STOPS WORKING under §4.1:506" "1" \
  "$(printf '%s\n' "$join_out" | grep -c 'permission denied for table x4probe_party' || true)"

def_out="$(printf "SET app.current_org = '$ORG_A';
SELECT party_id || ' ' || relationship_kind FROM x4probe_resolve_correct() ORDER BY 1;" | as_rt)"
echo "  q: SELECT ... FROM x4probe_resolve_correct();   (armed as A)"
printf '     -> %s\n' "${def_out:-<empty>}"
check "B3 correct definer returns exactly A's one edge" "1" "$(printf '%s\n' "$def_out" | grep -c . || true)"
check "B3 correct definer reveals no B edge" "0" "$(printf '%s\n' "$def_out" | grep -c "$PARTY2" || true)"

defb_out="$(printf "SET app.current_org = '$ORG_B';
SELECT party_id || ' ' || relationship_kind FROM x4probe_resolve_correct() ORDER BY 1;" | as_rt)"
echo "  q: same definer, armed as B"
printf '     -> %s\n' "$(printf '%s\n' "$defb_out" | tr '\n' ' ')"
check "B4 definer is org-sensitive (B sees its two edges)" "2" "$(printf '%s\n' "$defb_out" | grep -c . || true)"
echo

echo "=== GUC INVENTORY: did anything need a second tenancy dimension? ==="
# NOT measured via pg_settings: in PG 18 a placeholder GUC set with
# `SET app.current_org` is readable by current_setting() but does NOT appear in
# pg_settings (verified: current_setting returns the value, pg_settings returns
# 0 rows). Measuring session state was the wrong instrument. The schema itself
# is the right one -- read the GUC names out of the stored policy expressions.
guc_names="$(as_admin -Atq -c "
SELECT DISTINCT g[1]
FROM pg_policies,
     LATERAL regexp_matches(coalesce(qual,'') || ' ' || coalesce(with_check,''), 'app\.[a-z_]+', 'g') AS g
WHERE tablename LIKE 'x4probe%' ORDER BY 1;" | paste -sd, -)"
printf '  GUC names referenced by every policy this probe created: %s\n' "${guc_names:-<none>}"
check "zero new GUCs -- only app.current_org appears" "app.current_org" "$guc_names"

# And the definer resolvers: prove they read app.current_org, not a second GUC.
def_gucs="$(as_admin -Atq -c "
SELECT DISTINCT g[1]
FROM pg_proc, LATERAL regexp_matches(prosrc, 'app\.[a-z_]+', 'g') AS g
WHERE proname = 'x4probe_resolve_correct' ORDER BY 1;" | paste -sd, -)"
printf '  GUC names referenced by the Variant B resolver: %s\n' "${def_gucs:-<none>}"
check "resolver needs no second GUC either" "app.current_org" "$def_gucs"
echo

echo "=== policies created by this probe (proof no existing policy was touched) ==="
as_admin -Atq -c "SELECT tablename || ' / ' || policyname FROM pg_policies WHERE tablename LIKE 'x4probe%' ORDER BY 1;" | sed 's/^/  /'
echo "  existing org_isolation policies altered by this probe: 0 (nothing outside x4probe_*)"
echo

echo "############################################################"
if [ "$fails" -eq 0 ]; then echo "### ALL ASSERTIONS PASSED (controls RED, tests GREEN)    ###"
else echo "### $fails ASSERTION(S) FAILED                                ###"; fi
echo "############################################################"
exit "$fails"
