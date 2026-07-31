#!/usr/bin/env bash
# EXPERIMENT X4b -- FEASIBILITY PROBE. NOT A CI GATE, NOT PRODUCTION CODE.
#
# Tests review finding B9 (docs/ideas/ecosystem-plan-review.md:271-300): that a
# GROUP-SCOPED grant cannot live where the ecosystem plan puts it (Tier N,
# ont_instances + ont_links), because ont_links FKs BOTH endpoints to
# ont_instances(id, org_id) (0155:76-77) and ont_instances is FORCE-RLS
# org-isolated (0155:93-98).
#
# X4 tested tenant VISIBILITY with a replica schema. X4b tests the GRANT-SCOPE
# half against the REAL SHIPPED TABLES: every migration under
# backend/crates/platform/db/migrations is applied in order, so ont_instances,
# ont_links, groups and group_memberships are the production objects, not
# stand-ins. A replica could be wrong about the very FK under test.
#
# Container/hygiene recipe copied from tools/lanes/pgtest.sh, keeping the
# generated console_rt password so assertions run as the genuine
# NOSUPERUSER NOBYPASSRLS runtime role (ops/postgres-reconcile-topology.sh:303).
# A superuser bypasses RLS and would produce a meaningless GREEN.
#
# Runs the KNOWN-BAD CONTROLS FIRST. A probe with no demonstrated failure mode
# is not evidence.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$here/../../../.." && pwd)"

image="postgres:18.4@sha256:65f70a152846cf504dff86e807007e9aeac98c3aeb7b62541b2c55ab9d264e56"
name="x4bprobe-$$"
bootdb="x4bprobe_$$"       # container bootstrap db; topology.sh reconciles roles here
# 0196_platform_force_command_and_fk_closure.sql:34-42 refuses to apply unless it
# is either console_app applying to a console_app-owned database, or the Buck
# SQLx superuser bootstrap: CURRENT_USER = console_buck_admin, the startup marker
# console.sqlx_test_bootstrap = buck-sqlx-superuser-v1, a database matching
# ^_sqlx_test_[A-Za-z0-9_]{52}$, and datdba = the applier. We take the second
# path, which is the one the shipped test harness itself uses
# (tools/lanes/pgtest.sh builds the identical DATABASE_URL option).
db="_sqlx_test_$(openssl rand -hex 26)"

# Fixed ids, shared with probe.sql.
ORG_A='aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'
ORG_B='bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb'
GROUP_G='99999999-9999-4999-8999-999999999999'
USER_HR_A='11111111-1111-4111-8111-111111111111'   # HR officer, 소속 = org A
LINK_TYPE_SCOPE='d1000000-0000-4000-8000-000000000001'   # grant_scope link type (org A)
CTRL_INSTANCE='e1000000-0000-4000-8000-0000000000ff'     # org A row that must be invisible from B
ORG_A_INSTANCE='e1000000-0000-4000-8000-000000000001'    # `organization` scope instance, org A
GROUP_INSTANCE_A='e1000000-0000-4000-8000-000000000002'  # `group` scope instance minted in A
GROUP_INSTANCE_B='e2000000-0000-4000-8000-000000000002'  # `group` scope instance minted in B

envf=""
# Hygiene asserted on THIS run's own container only, per LANE-PROTOCOL.md.
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
envf="$(mktemp "${TMPDIR:-/tmp}/x4bprobe-pg.XXXXXX")"; chmod 600 "$envf"
{
  printf 'POSTGRES_DB=%s\nPOSTGRES_USER=console_buck_admin\nPOSTGRES_PASSWORD=%s\n' "$bootdb" "$admin"
  printf 'POSTGRES_HOST=127.0.0.1\nPOSTGRES_PORT=5432\nPOSTGRES_ADMIN_USER=console_buck_admin\nPOSTGRES_ADMIN_PASSWORD=%s\n' "$admin"
  printf 'CONSOLE_APP_POSTGRES_PASSWORD=%s\nCONSOLE_RT_POSTGRES_PASSWORD=%s\n' "$app" "$rt"
  printf 'CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=%s\nCONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=%s\n' "$leave" "$ont" "$force"
} >"$envf"

docker run -d --rm --name "$name" -p 127.0.0.1::5432 --env-file "$envf" "$image" >/dev/null
docker cp "$repo_root/ops/postgres-reconcile-topology.sh" "$name:/topology.sh" >/dev/null
docker cp "$envf" "$name:/topology.env" >/dev/null
for i in $(seq 1 40); do
  if [ "$(docker exec "$name" cat /proc/1/comm 2>/dev/null || true)" = "postgres" ] \
     && docker exec "$name" pg_isready -h 127.0.0.1 -U console_buck_admin -d "$bootdb" >/dev/null 2>&1; then break; fi
  [ "$i" = 40 ] && { echo "postgres never became healthy"; exit 1; }
  sleep 1
done
docker exec "$name" sh -ceu 'set -a; . /topology.env; exec bash /topology.sh' >/dev/null
echo "topology reconciled; console_rt exists as NOSUPERUSER NOBYPASSRLS"

# The migration target database, owned by the applier (0196's requirement).
docker exec -i -e PGPASSWORD="$admin" "$name" \
  psql -X -q -h 127.0.0.1 -U console_buck_admin -d "$bootdb" -v ON_ERROR_STOP=1 \
  -c "CREATE DATABASE \"$db\";" >/dev/null

# PGOPTIONS carries 0196's startup marker on every admin connection.
bootopt='-c console.sqlx_test_bootstrap=buck-sqlx-superuser-v1'
as_admin() { docker exec -i -e PGPASSWORD="$admin" -e PGOPTIONS="$bootopt" "$name" \
  psql -X -q -h 127.0.0.1 -U console_buck_admin -d "$db" -v ON_ERROR_STOP=1 "$@"; }
# console_rt is NOINHERIT/NOBYPASSRLS; -Atq gives bare parseable rows.
as_rt() { docker exec -i -e PGPASSWORD="$rt" "$name" \
  psql -X -Atq -F '|' -h 127.0.0.1 -U console_rt -d "$db" -v ON_ERROR_STOP=1; }
# no ON_ERROR_STOP: used where an error IS the expected result.
as_rt_err() { docker exec -i -e PGPASSWORD="$rt" "$name" \
  psql -X -Atq -F '|' -h 127.0.0.1 -U console_rt -d "$db" 2>&1; }

# --- the REAL shipped schema, not a replica -----------------------------------
# Migrations are plain SQL applied in filename order (the same order sqlx uses).
# The ontology tables under test (ont_instances/ont_links, 0155) and the group
# tables (0060) are therefore the production definitions.
echo "applying shipped migrations from backend/crates/platform/db/migrations ..."
docker exec "$name" mkdir -p /migrations >/dev/null
docker cp "$repo_root/backend/crates/platform/db/migrations/." "$name:/migrations/" >/dev/null
docker exec -i -e PGPASSWORD="$admin" -e PGOPTIONS="$bootopt" "$name" bash -c '
  set -euo pipefail
  n=0
  for f in $(ls /migrations/*.sql | sort); do
    if ! psql -X -q -h 127.0.0.1 -U console_buck_admin -d "'"$db"'" -v ON_ERROR_STOP=1 \
         -f "$f" >/tmp/mig.log 2>&1; then
      echo "  MIGRATION FAILED: $f"; grep -v NOTICE /tmp/mig.log | head -20; exit 1
    fi
    n=$((n+1))
  done
  echo "  applied $n migrations"'
mig_count="$(as_admin -Atq -c "SELECT count(*) FROM pg_class WHERE relname IN ('ont_instances','ont_links','groups','group_memberships','object_links');")"
echo "  substrate tables present (ont_instances, ont_links, groups, group_memberships, object_links): $mig_count/5"
[ "$mig_count" = "5" ] || { echo "FATAL: the real substrate did not materialise; results would be void"; exit 1; }
echo

as_admin -f - <"$here/probe.sql" >/dev/null
echo "probe seed applied (2 orgs in 1 group; grant object types; grant instances)"
echo

fails=0
check() { # label expected actual
  if [ "$2" = "$3" ]; then printf 'PASS  %-58s %s\n' "$1" "$3"
  else printf 'FAIL  %-58s expected=%s actual=%s\n' "$1" "$2" "$3"; fails=$((fails+1)); fi
}
show() { printf '  $ psql -U console_rt <<SQL\n'; printf '%s\n' "$1" | sed 's/^/      /'; }
out()  { printf '  OUTPUT:\n'; printf '%s\n' "${1:-<empty>}" | sed 's/^/      /'; }

echo "=== role identity: the assertions must NOT be running as a superuser ==="
identity="$(printf "SELECT session_user, current_setting('is_superuser'), rolbypassrls FROM pg_roles WHERE rolname = session_user;" | as_rt)"
echo "  $identity"
check "session_user|is_superuser|rolbypassrls" "console_rt|off|f" "$identity"
echo

echo "=== B9's citations, verified against the RUNNING schema (not the .sql file) ==="
fk_def="$(as_admin -Atq -c "
SELECT string_agg(pg_get_constraintdef(oid), ' ; ' ORDER BY conname)
FROM pg_constraint
WHERE conrelid = 'ont_links'::regclass AND contype = 'f'
  AND confrelid = 'ont_instances'::regclass;")"
printf '  ont_links FKs to ont_instances:\n    %s\n' "$fk_def"
check "B9(a) BOTH ont_links endpoints FK to ont_instances(id, org_id)" "2" \
  "$(printf '%s' "$fk_def" | grep -o 'REFERENCES ont_instances(id, org_id)' | grep -c . || true)"
rls_state="$(as_admin -Atq -c "SELECT relrowsecurity::text || '|' || relforcerowsecurity::text FROM pg_class WHERE relname = 'ont_instances';")"
printf '  ont_instances (relrowsecurity|relforcerowsecurity): %s\n' "$rls_state"
check "ont_instances has RLS ENABLEd and FORCEd" "true|true" "$rls_state"
grp_tier="$(as_admin -Atq -c "SELECT count(*) FROM information_schema.columns WHERE table_name = 'groups' AND column_name = 'org_id';")"
printf '  groups.org_id columns: %s  (0 = groups is not tenant-scoped, i.e. a different tier)\n' "$grp_tier"
check "B9(a) groups is NOT an org-scoped table" "0" "$grp_tier"
grp_in_ont="$(as_admin -Atq -c "SELECT count(*) FROM ont_instances i JOIN groups g ON g.id = i.id;")"
printf '  ont_instances rows whose id is a groups id: %s\n' "$grp_in_ont"
check "B9(a) no groups row has an ont_instances row" "0" "$grp_in_ont"
echo

echo "############################################################"
echo "### KNOWN-BAD CONTROLS -- run FIRST, MUST be observed RED ###"
echo "############################################################"
echo
echo "--- CONTROL 1 (the brief's control): org isolation IS live on the real"
echo "    ont_instances. Row minted in org A; armed as org B; MUST be invisible."
c1_sql="SET app.current_org = '$ORG_B';
SELECT id::text || ' ' || title FROM ont_instances WHERE id = '$CTRL_INSTANCE';"
show "$c1_sql"
c1_out="$(printf '%s\n' "$c1_sql" | as_rt)"
out "$c1_out"
c1_rows="$(printf '%s' "$c1_out" | grep -c . || true)"
check "CONTROL 1 org B cannot see org A's ont_instances row" "0" "$c1_rows"

echo
echo "    ...and the same row IS there. Ground truth, RLS bypassed (superuser):"
c1_truth="$(as_admin -Atq -c "SELECT org_id::text FROM ont_instances WHERE id = '$CTRL_INSTANCE';")"
printf '      %s\n' "$c1_truth"
check "CONTROL 1 the hidden row really exists, owned by org A" "$ORG_A" "$c1_truth"

echo
echo "--- CONTROL 2: proof CONTROL 1's emptiness is caused by RLS and not by a"
echo "    broken query. Byte-identical query shape against a copy of the same"
echo "    rows with RLS never enabled. Armed as B, it MUST LEAK org A's row."
c2_sql="SET app.current_org = '$ORG_B';
SELECT id::text || ' ' || title FROM x4b_instances_control_norls WHERE id = '$CTRL_INSTANCE';"
show "$c2_sql"
c2_out="$(printf '%s\n' "$c2_sql" | as_rt)"
out "$c2_out"
check "CONTROL 2 leaks org A's row to org B (RED expected)" "1" \
  "$(printf '%s' "$c2_out" | grep -c "$CTRL_INSTANCE" || true)"

if [ "$fails" -ne 0 ]; then
  echo
  echo "!!! CONTROLS FAILED. Either RLS is not being exercised (CONTROL 1 saw the"
  echo "!!! row) or the query cannot return rows at all (CONTROL 2 saw nothing)."
  echo "!!! Every result below would be void. Stopping, per the brief."
  exit 1
fi
echo
echo "controls behaved as required: the query CAN return the row (CONTROL 2),"
echo "and RLS is what stops it (CONTROL 1). The harness exercises org isolation."
echo

echo "############################################################"
echo "### CASE 1 (BASELINE): company-scoped grant, org A -> org A"
echo "############################################################"
echo "  The easy half, exactly as X4 tested it. A grant instance in org A whose"
echo "  scope is org A's own 'organization' instance, read from org A. The"
echo "  authority input a 결재 raise needs: (subject, capability, scope)."
b1_sql="SET app.current_org = '$ORG_A';
SELECT g.title,
       r.attributes->>'capability' AS capability,
       r.attributes->>'subject_party_id' AS subject,
       s.title AS scope_title
FROM ont_instances g
  JOIN ont_instance_revisions r ON r.instance_id = g.id AND r.valid_to IS NULL
  JOIN ont_links l ON l.from_instance_id = g.id AND l.valid_to IS NULL
  JOIN ont_instances s ON s.id = l.to_instance_id
WHERE g.title = 'grant: company-scoped, org A';"
show "$b1_sql"
b1_out="$(printf '%s\n' "$b1_sql" | as_rt)"
out "$b1_out"
check "CASE 1 the company-scoped grant resolves with its scope" "1" \
  "$(printf '%s' "$b1_out" | grep -c 'purchase.approve' || true)"
echo

echo "############################################################"
echo "### CASE 2 (THE FALSIFYING CASE): group-scoped grant, read from a sibling"
echo "############################################################"
echo "  The owner's requirement 3: an HR officer whose 소속 is subsidiary A holds"
echo "  authority over the whole group. The grant is minted in org A (the only"
echo "  org the Tier N substrate permits -- ont_instances.org_id is NOT NULL,"
echo "  0155:18) and scoped to group G, of which A and B are both members."
echo
echo "  2a. does group G really contain BOTH orgs? (ground truth, owner-only table)"
mem="$(as_admin -Atq -c "SELECT count(*) FROM group_memberships WHERE group_id = '$GROUP_G';")"
printf '      group_memberships rows for G: %s\n' "$mem"
check "CASE 2a both orgs are members of group G" "2" "$mem"
echo
echo "  2b. read the group-scoped grant from org A (where it was minted)"
b2a_sql="SET app.current_org = '$ORG_A';
SELECT g.title, r.attributes->>'capability', r.attributes->>'scope_level', r.attributes->>'scope_node_id'
FROM ont_instances g
  JOIN ont_instance_revisions r ON r.instance_id = g.id AND r.valid_to IS NULL
WHERE g.title = 'grant: group-scoped over G';"
show "$b2a_sql"
b2a_out="$(printf '%s\n' "$b2a_sql" | as_rt)"
out "$b2a_out"
check "CASE 2b org A (the minting org) can read it" "1" \
  "$(printf '%s' "$b2a_out" | grep -c 'group-scoped over G' || true)"
echo
echo "  2c. THE TEST. Arm app.current_org = org B -- a sibling in the SAME group,"
echo "      raising a 결재 whose competent unit is at group scope. Ask for the"
echo "      eligible-approver authority input. B9 says this returns nothing."
b2b_sql="SET app.current_org = '$ORG_B';
SELECT g.title, r.attributes->>'capability', r.attributes->>'scope_node_id'
FROM ont_instances g
  JOIN ont_instance_revisions r ON r.instance_id = g.id AND r.valid_to IS NULL
WHERE r.attributes->>'scope_level' = 'group'
  AND r.attributes->>'scope_node_id' = '$GROUP_G';"
show "$b2b_sql"
b2b_out="$(printf '%s\n' "$b2b_sql" | as_rt)"
out "$b2b_out"
b2b_rows="$(printf '%s' "$b2b_out" | grep -c . || true)"
check "CASE 2c org B gets ZERO rows for the group-scoped grant" "0" "$b2b_rows"
echo
echo "  2d. is it merely that no grant matched? Ground truth, RLS bypassed:"
b2t="$(as_admin -Atq -c "SELECT count(*) FROM ont_instance_revisions WHERE attributes->>'scope_level' = 'group' AND attributes->>'scope_node_id' = '$GROUP_G';")"
printf '      group-scoped grant revisions in the table: %s\n' "$b2t"
check "CASE 2d the grant IS in the table, B just cannot reach it" "1" "$b2t"
echo
echo "  2e. can org B reach it by naming org A explicitly? (the obvious workaround)"
b2e_sql="SET app.current_org = '$ORG_B';
SELECT count(*) FROM ont_instances WHERE org_id = '$ORG_A';"
show "$b2e_sql"
b2e_out="$(printf '%s\n' "$b2e_sql" | as_rt)"
out "$b2e_out"
check "CASE 2e naming org A explicitly still returns 0" "0" "$b2e_out"
echo
echo "  2f. can org B arm app.current_org to the GROUP id instead?"
echo "      (org-hierarchy.md:181 states invariant C1: app.current_org is"
echo "       ALWAYS a real Org id, NEVER a Group id. Measure what happens.)"
b2f_sql="SET app.current_org = '$GROUP_G';
SELECT count(*) FROM ont_instances;"
show "$b2f_sql"
b2f_out="$(printf '%s\n' "$b2f_sql" | as_rt_err)"
out "$b2f_out"
check "CASE 2f arming the group id yields no instances either" "1" \
  "$(printf '%s' "$b2f_out" | grep -c '^0$' || true)"
echo

echo "############################################################"
echo "### CASE 3 (THE EDGE CASE): can an ont_link point at a group at all?"
echo "############################################################"
echo "  §4.3:666 specifies grant_scope 'grant -> org_unit | organization | group'"
echo "  Stored as: ont_link. Three ways to try it; all as console_rt, armed as A."
echo
echo "  3a. edge whose to_instance_id is the REAL groups.id (Tier G row)"
c3a_sql="SET app.current_org = '$ORG_A';
INSERT INTO ont_links (org_id, link_type_id, from_instance_id, to_instance_id, valid_from)
VALUES ('$ORG_A', '$LINK_TYPE_SCOPE', '$ORG_A_INSTANCE', '$GROUP_G', now());"
show "$c3a_sql"
c3a_out="$(printf '%s\n' "$c3a_sql" | as_rt_err)"
out "$c3a_out"
check "CASE 3a FK rejects an edge to a groups row" "1" \
  "$(printf '%s' "$c3a_out" | grep -c 'violates foreign key constraint' || true)"
echo
echo "  3b. edge to a 'group' ont_instance minted in ORG B (the sibling that"
echo "      actually needs to see it). Composite FK requires the same org_id."
c3b_sql="SET app.current_org = '$ORG_A';
INSERT INTO ont_links (org_id, link_type_id, from_instance_id, to_instance_id, valid_from)
VALUES ('$ORG_A', '$LINK_TYPE_SCOPE', '$ORG_A_INSTANCE', '$GROUP_INSTANCE_B', now());"
show "$c3b_sql"
c3b_out="$(printf '%s\n' "$c3b_sql" | as_rt_err)"
out "$c3b_out"
check "CASE 3b FK rejects a cross-org edge (A -> B's group instance)" "1" \
  "$(printf '%s' "$c3b_out" | grep -c 'violates foreign key constraint\|new row violates row-level security policy' || true)"
echo
echo "  3c. edge to a 'group' ont_instance minted in ORG A -- the only shape the"
echo "      FK allows. Expressible? Yes. Useful to org B? Measured next."
c3c_sql="SET app.current_org = '$ORG_A';
INSERT INTO ont_links (org_id, link_type_id, from_instance_id, to_instance_id, valid_from)
VALUES ('$ORG_A', '$LINK_TYPE_SCOPE', '$ORG_A_INSTANCE', '$GROUP_INSTANCE_A', now())
RETURNING 'edge-accepted';"
show "$c3c_sql"
c3c_out="$(printf '%s\n' "$c3c_sql" | as_rt_err)"
out "$c3c_out"
check "CASE 3c same-org group-instance edge IS accepted" "1" \
  "$(printf '%s' "$c3c_out" | grep -c 'edge-accepted' || true)"
echo
echo "  3d. ...and org B reads that accepted edge:"
c3d_sql="SET app.current_org = '$ORG_B';
SELECT count(*) FROM ont_links WHERE from_instance_id = '$ORG_A_INSTANCE';"
show "$c3d_sql"
c3d_out="$(printf '%s\n' "$c3d_sql" | as_rt)"
out "$c3d_out"
check "CASE 3d org B sees 0 of org A's scope edges" "0" "$c3d_out"
echo

echo "############################################################"
echo "### WHAT WOULD CARRY IT: the two candidate substrates, measured"
echo "############################################################"
echo "  S1. object_links (0102:53-69) -- opaque TEXT ids, no FK to either"
echo "      endpoint, so a group id is STORABLE. But it still has"
echo "      org_id NOT NULL + FORCE RLS (0102:76-79). Insert as A, read as B."
echo
echo "  S1a. FIRST, a correction to the review's own description of this table."
echo "       0102:61-62 comments link_type 'Free-form-but-validated so new link"
echo "       types need no migration'. That is STALE: 0130:24-31 created a"
echo "       link_types table with link_type as its PRIMARY KEY, 0130:75 added"
echo "       object_links_link_type_fkey, and 0132:8 validated it. Measure it:"
s1a_sql="SET app.current_org = '$ORG_A';
INSERT INTO object_links (org_id, src_kind, src_id, dst_kind, dst_id, link_type)
VALUES ('$ORG_A', 'person', 'x4b-grant-1', 'person', '$GROUP_G', 'grant_scope')
RETURNING 'accepted';"
show "$s1a_sql"
s1a_out="$(printf '%s\n' "$s1a_sql" | as_rt_err)"
out "$s1a_out"
check "S1a a NEW link_type ('grant_scope') is REJECTED -- migration needed" "1" \
  "$(printf '%s' "$s1a_out" | grep -c 'violates foreign key constraint "object_links_link_type_fkey"' || true)"
vocab="$(as_rt <<<"SELECT count(*) FROM link_types;")"
printf '  seeded link_types vocabulary size (0130:37-49): %s\n' "$vocab"
echo "  and console_rt's privileges on the vocabulary table:"
vocab_w="$(printf "INSERT INTO link_types (link_type, description) VALUES ('grant_scope','x4b');" | as_rt_err)"
out "$vocab_w"
check "S1a console_rt cannot extend the vocabulary at runtime" "1" \
  "$(printf '%s' "$vocab_w" | grep -c 'permission denied for table link_types' || true)"
echo
echo "  S1b. now with a SEEDED link_type, so only the org floor is under test."
s1_sql="SET app.current_org = '$ORG_A';
INSERT INTO object_links (org_id, src_kind, src_id, dst_kind, dst_id, link_type)
VALUES ('$ORG_A', 'person', 'x4b-grant-1', 'person', '$GROUP_G', 'authorized_by')
RETURNING 'object_links-insert-accepted';"
show "$s1_sql"
s1_out="$(printf '%s\n' "$s1_sql" | as_rt_err)"
out "$s1_out"
check "S1b object_links accepts a group id as an opaque dst_id" "1" \
  "$(printf '%s' "$s1_out" | grep -c 'object_links-insert-accepted' || true)"
s1b_out="$(printf "SET app.current_org = '$ORG_B';\nSELECT count(*) FROM object_links WHERE dst_id = '$GROUP_G';" | as_rt)"
printf '  read back armed as org B -> %s\n' "$s1b_out"
check "S1b but org B still cannot read it (same org floor)" "0" "$s1b_out"
echo
echo "  S2. Tier O + definer -- the SHIPPED answer for cross-org authority."
echo "      group_role_grants (0060:40-49) + group_role_grants_for_user"
echo "      (0060:99-126). No org_id, no RLS, no console_rt grant."
s2a_out="$(printf "SET app.current_org = '$ORG_B';\nSELECT count(*) FROM group_role_grants;" | as_rt_err)"
printf '  q: SELECT count(*) FROM group_role_grants;  (armed as B)\n     -> %s\n' "$(printf '%s' "$s2a_out" | tr '\n' ' ')"
check "S2 console_rt has NO raw read on group_role_grants" "1" \
  "$(printf '%s' "$s2a_out" | grep -c 'permission denied for table group_role_grants' || true)"
s2b_out="$(printf "SET app.current_org = '$ORG_B';\nSELECT group_id::text || ' ' || group_role FROM group_role_grants_for_user('$USER_HR_A');" | as_rt)"
printf '  q: SELECT ... FROM group_role_grants_for_user(<org A user>);  (armed as B)\n     -> %s\n' "${s2b_out:-<empty>}"
check "S2 the definer DOES return org A's holder to org B" "1" \
  "$(printf '%s' "$s2b_out" | grep -c "$GROUP_G" || true)"
echo "     ^ this is the mechanism that works -- and it works by being Tier O"
echo "       behind a definer, NOT by being a Tier N ont_instance."
echo
echo "  S2-cost. The definer's org predicate, measured. group_role_grants_for_user"
echo "           filters on p_user only (0060:113-114); it never reads"
echo "           app.current_org. Same class as X4's CONTROL 2."
s2c_gucs="$(as_admin -Atq -c "SELECT coalesce(string_agg(DISTINCT g[1], ','), '<none>') FROM pg_proc, LATERAL regexp_matches(prosrc, 'app\.[a-z_]+', 'g') AS g WHERE proname = 'group_role_grants_for_user';")"
printf '  GUC names referenced by group_role_grants_for_user: %s\n' "$s2c_gucs"
check "S2-cost the shipped definer has NO org predicate" "<none>" "$s2c_gucs"
echo "     Not a defect by itself -- cross-group authority is deliberately not"
echo "     org-scoped -- but it means the caller, not the database, is the org"
echo "     floor for every group-scoped authority read. That is the cost."
echo

echo "############################################################"
echo "### GUC INVENTORY: did anything here need a new tenancy dimension?"
echo "############################################################"
guc_names="$(as_admin -Atq -c "
SELECT coalesce(string_agg(DISTINCT g[1], ','), '<none>')
FROM pg_policies,
     LATERAL regexp_matches(coalesce(qual,'') || ' ' || coalesce(with_check,''), 'app\.[a-z_]+', 'g') AS g
WHERE tablename IN ('ont_instances','ont_instance_revisions','ont_links','object_links');")"
printf '  GUCs in the policies of every table this experiment read: %s\n' "$guc_names"
check "only app.current_org -- no app.current_group exists" "app.current_org" "$guc_names"
echo

echo "############################################################"
if [ "$fails" -eq 0 ]; then echo "### ALL ASSERTIONS PASSED (controls behaved, cases measured)"
else echo "### $fails ASSERTION(S) FAILED"; fi
echo "############################################################"
exit "$fails"
