#!/usr/bin/env bash
# Runs one iOS UI shard locally against the toolchain the hosted job pins.
#
# Why this exists: the hosted iOS suite takes 45-70 minutes and is the only
# signal this repository had for these tests, so every hypothesis about a
# failure cost a full cycle. Running a shard locally takes minutes, and it is
# what turned an AX5 failure that three plausible theories had failed to explain
# into a measured root cause — a fixture whose display order depended on how
# long the Xcode build took.
#
# Fidelity is the whole point, so the shard's fixture profile, content size,
# selectors and camera-privacy handling are read from the workflow rather than
# passed in: a local run that quietly disagrees with CI produces confident
# evidence about a configuration CI never executes.
#
# NOT reproduced here: the hosted job's supply-chain hardening — Mach-O
# entitlement auditing, artifact secret scanning, log masking. Those protect the
# runner, not the test outcome. Never cite this script as evidence for them.
#
#   usage: scripts/run-ios-ui-shard.sh <shard-name>
#   e.g.:  scripts/run-ios-ui-shard.sh dynamic-type-ax5
#
# Requires: Xcode with the iOS runtime the workflow pins, Docker, psql, xcodegen.
set -euo pipefail

SHARD="${1:-}"
if [[ -z "$SHARD" ]]; then
  echo "usage: $(basename "$0") <shard-name>" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

: "${CONSOLE_IOS_LOCAL_PGPORT:=55444}"
: "${CONSOLE_IOS_LOCAL_BACKEND_PORT:=8137}"
: "${CONSOLE_IOS_LOCAL_CONTAINER:=console-iosui}"
: "${DEVELOPER_DIR:=/Applications/Xcode.app/Contents/Developer}"
export DEVELOPER_DIR

DB=console_ios_ui
APP_PASSWORD=console-e2e-owner-change-me
WORK="$ROOT/.tmp/ios-ui-local"
AUTH="$WORK/auth"
DERIVED="$WORK/derived"
CONSOLE_APP_BIN="$ROOT/.tmp/cargo-target-e2e/debug/console-app"
PG_IMAGE="postgres:18.4@sha256:4aabea78cf39b90e834caf3af7d602a18565f6fe2508705c8d01aa63245c2e20"

step() { printf '\n=== %s\n' "$1"; }

# Fail loudly rather than run a shard CI does not have.
eval "$(node "$ROOT/scripts/ios-ui-shard-config.mjs" "$SHARD")"
step "shard $SHARD"
printf 'profile=%s content_size=%s watchdog=%ss selectors=%d\n' \
  "$SHARD_FIXTURE_PROFILE" "$SHARD_CONTENT_SIZE" "$SHARD_TIMEOUT_SECONDS" "${#SHARD_SELECTORS[@]}"

install -d -m 700 "$WORK" "$AUTH"

step "backend binary"
SQLX_OFFLINE=true CARGO_TARGET_DIR="$ROOT/.tmp/cargo-target-e2e" \
  cargo build --manifest-path backend/Cargo.toml -p console-app
test -x "$CONSOLE_APP_BIN"

step "postgres on 127.0.0.1:$CONSOLE_IOS_LOCAL_PGPORT"
if ! docker exec "$CONSOLE_IOS_LOCAL_CONTAINER" pg_isready -U mntadmin -d postgres >/dev/null 2>&1; then
  docker rm -f "$CONSOLE_IOS_LOCAL_CONTAINER" >/dev/null 2>&1 || true
  # --rm so an interrupted run cannot leave an orphaned volume behind.
  docker run -d --rm --name "$CONSOLE_IOS_LOCAL_CONTAINER" \
    -p "127.0.0.1:$CONSOLE_IOS_LOCAL_PGPORT:5432" \
    -e POSTGRES_DB=postgres -e POSTGRES_USER=mntadmin -e POSTGRES_PASSWORD=iosui-admin \
    "$PG_IMAGE" >/dev/null
  for _ in $(seq 1 60); do
    docker exec "$CONSOLE_IOS_LOCAL_CONTAINER" pg_isready -U mntadmin -d postgres >/dev/null 2>&1 && break
    sleep 1
  done
  docker cp ops/postgres-reconcile-topology.sh "$CONSOLE_IOS_LOCAL_CONTAINER:/reconcile.sh" >/dev/null
  docker exec \
    -e POSTGRES_HOST=127.0.0.1 -e POSTGRES_DB=postgres \
    -e POSTGRES_ADMIN_USER=mntadmin -e POSTGRES_ADMIN_PASSWORD=iosui-admin \
    -e CONSOLE_APP_POSTGRES_PASSWORD="$APP_PASSWORD" \
    -e CONSOLE_RT_POSTGRES_PASSWORD=console-e2e-runtime-change-me \
    -e CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD=console-e2e-leave-command-change-me \
    -e CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD=console-e2e-ontology-command-change-me \
    -e CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD=console-e2e-platform-force-command-change-me \
    "$CONSOLE_IOS_LOCAL_CONTAINER" bash /reconcile.sh >/dev/null
fi

pg_env=(
  E2E_PG_SUPERUSER=mntadmin E2E_PG_SUPERUSER_PASSWORD=iosui-admin
  E2E_PG_HOST=127.0.0.1 E2E_PG_PORT="$CONSOLE_IOS_LOCAL_PGPORT" E2E_DB_NAME="$DB"
)

step "database"
env "${pg_env[@]}" CONSOLE_APP_BIN="$CONSOLE_APP_BIN" bash e2e/harness/db.sh >"$WORK/db.log" 2>&1
tail -1 "$WORK/db.log"

step "backend on 127.0.0.1:$CONSOLE_IOS_LOCAL_BACKEND_PORT"
if [[ -s "$AUTH/backend.pid" ]] && kill -0 "$(cat "$AUTH/backend.pid")" 2>/dev/null; then
  kill -TERM "$(cat "$AUTH/backend.pid")" 2>/dev/null || true
  sleep 2
fi
# boot-backend.sh refuses to kill a listener it does not own, which is right and
# is also the least obvious way this script fails: a backend left by an earlier
# run under a different auth directory holds the port, and the harness error
# reads like a bug in the harness. Say what to do instead.
if ! python3 -c "
import socket,sys
s=socket.socket()
sys.exit(0 if s.connect_ex(('127.0.0.1', $CONSOLE_IOS_LOCAL_BACKEND_PORT)) != 0 else 1)
"; then
  cat >&2 <<MSG
port $CONSOLE_IOS_LOCAL_BACKEND_PORT is already serving. Something else owns it —
most likely a backend from an earlier local run under a different auth
directory. Either stop that process, or run this shard on another port:

  CONSOLE_IOS_LOCAL_BACKEND_PORT=8138 scripts/run-ios-ui-shard.sh $SHARD
MSG
  exit 1
fi
env "${pg_env[@]}" CONSOLE_APP_BIN="$CONSOLE_APP_BIN" CONSOLE_IOS_COLDSTART_OTP="$(openssl rand -hex 32)" \
  node scripts/boot-ios-ui-backend.mjs "$ROOT" "$AUTH" "$CONSOLE_IOS_LOCAL_BACKEND_PORT" \
  >"$WORK/boot.log" 2>&1
tail -1 "$WORK/boot.log"

step "xcode project"
cd "$ROOT/ios"
CI_PLIST="$WORK/Info.ci.plist"
SPEC="$ROOT/ios/project.ci.yml"
cp Sources/ConsoleApp/Info.plist "$CI_PLIST"
/usr/libexec/PlistBuddy -c 'Add :NSAppTransportSecurity dict' "$CI_PLIST" >/dev/null
/usr/libexec/PlistBuddy -c 'Add :NSAppTransportSecurity:NSAllowsLocalNetworking bool true' "$CI_PLIST" >/dev/null
node "$ROOT/scripts/ios-ui-local-project-spec.mjs" "$CI_PLIST" "$SPEC"
xcodegen generate --spec "$SPEC" >/dev/null
xcodebuild -resolvePackageDependencies -onlyUsePackageVersionsFromResolvedFile \
  -project Console.xcodeproj -scheme ConsoleApp >"$WORK/resolve.log" 2>&1

step "simulator"
SIM_RUNTIME="$(grep -oE 'com\.apple\.CoreSimulator\.SimRuntime\.iOS-[0-9-]+' "$ROOT/.github/workflows/ios-ui-tests.yml" | head -1)"
SIM_TYPE="$(grep -oE 'com\.apple\.CoreSimulator\.SimDeviceType\.[A-Za-z0-9-]+' "$ROOT/.github/workflows/ios-ui-tests.yml" | head -1)"
UUID_FILE="$WORK/sim.uuid"
if [[ -s "$UUID_FILE" ]] && xcrun simctl list devices -j \
  | python3 -c 'import json,sys;t=sys.argv[1];d=json.load(sys.stdin)["devices"];sys.exit(0 if any(x.get("udid")==t for g in d.values() for x in g) else 1)' "$(cat "$UUID_FILE")"; then
  UUID="$(cat "$UUID_FILE")"
else
  UUID="$(xcrun simctl create "console-local-$SHARD" "$SIM_TYPE" "$SIM_RUNTIME")"
  printf '%s' "$UUID" > "$UUID_FILE"
fi
# The hosted job gets a new device per run; a reused one carries app state that
# changes which rows a lazy List realizes for accessibility queries.
xcrun simctl shutdown "$UUID" 2>/dev/null || true
xcrun simctl erase "$UUID"
xcrun simctl boot "$UUID" 2>/dev/null || true
xcrun simctl bootstatus "$UUID" -b >/dev/null
xcrun simctl ui "$UUID" content_size "$SHARD_CONTENT_SIZE" >/dev/null
if [[ "$SHARD_RESET_CAMERA_PRIVACY" == 1 ]]; then
  xcrun simctl privacy "$UUID" reset camera
fi
echo "$UUID content_size=$(xcrun simctl ui "$UUID" content_size)"

step "build-for-testing"
xcodebuild build-for-testing -onlyUsePackageVersionsFromResolvedFile \
  -project Console.xcodeproj -scheme ConsoleApp \
  -destination "platform=iOS Simulator,id=$UUID" -derivedDataPath "$DERIVED" >"$WORK/build.log" 2>&1

step "session"
OTP="$(openssl rand -hex 32)"
HASH="$(printf %s "$OTP" | shasum -a 256 | awk '{print $1}')"
PGPASSWORD="$APP_PASSWORD" psql -h 127.0.0.1 -p "$CONSOLE_IOS_LOCAL_PGPORT" -U console_app -d "$DB" \
  -q -v ON_ERROR_STOP=1 -v "otp_hash=$HASH" -v "fixture_profile=$SHARD_FIXTURE_PROFILE" \
  -f "$ROOT/e2e/harness/seed-mobile-ci.sql" >/dev/null
export CONSOLE_UITEST_BASE_URL="http://127.0.0.1:$CONSOLE_IOS_LOCAL_BACKEND_PORT"
python3 -c 'import json,sys;print(json.dumps({"otp":sys.argv[1]}))' "$OTP" > "$AUTH/otp.json"
curl --fail --silent --show-error -X POST "$CONSOLE_UITEST_BASE_URL/api/v1/auth/otp/redeem" \
  -H 'content-type: application/json' -H 'x-maintenance-client: mobile' \
  --data-binary @- < "$AUTH/otp.json" > "$AUTH/tokens.json"
rm -f "$AUTH/otp.json"
CONSOLE_UITEST_ACCESS_TOKEN="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["access_token"])' "$AUTH/tokens.json")"
CONSOLE_UITEST_REFRESH_TOKEN="$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["refresh_token"])' "$AUTH/tokens.json")"
export CONSOLE_UITEST_ACCESS_TOKEN CONSOLE_UITEST_REFRESH_TOKEN
export CONSOLE_UITEST_WORK_ORDER_ID_DETAIL=00000000-0000-0000-0000-000000f00004
export CONSOLE_UITEST_WORK_ORDER_ID_START=00000000-0000-0000-0000-000000f00003
export CONSOLE_UITEST_WORK_ORDER_ID_REPORT=00000000-0000-0000-0000-000000f00004
export CONSOLE_UITEST_WORK_ORDER_ID_REPORT_SUCCESS=00000000-0000-0000-0000-000000f00005
export CONSOLE_UITEST_WORK_ORDER_ID_ADMIN_APPROVE=00000000-0000-0000-0000-000000f00007
export CONSOLE_UITEST_WORK_ORDER_ID_ADMIN_REJECT=00000000-0000-0000-0000-000000f00008
export CONSOLE_UITEST_WORK_ORDER_ID_CAMERA=00000000-0000-0000-0000-000000f00004
export CONSOLE_UITEST_MESSENGER_THREAD_ID=00000000-0000-0000-0000-000000c10001
export CONSOLE_UITEST_MESSENGER_INITIAL_MESSAGE_ID=00000000-0000-0000-0000-000000c20001

step "patch xctestrun"
XCTESTRUN="$(find "$DERIVED/Build/Products" -name '*.xctestrun' -print -quit)"
test -n "$XCTESTRUN"
( cd "$ROOT" && python3 scripts/patch-ios-xctestrun.py "$XCTESTRUN" \
  --target ConsoleUITests \
  --ui-target-app-path '__TESTROOT__/Debug-iphonesimulator/ConsoleApp.app' \
  --env CONSOLE_UITEST_BASE_URL --env CONSOLE_UITEST_ACCESS_TOKEN --env CONSOLE_UITEST_REFRESH_TOKEN \
  --env CONSOLE_UITEST_WORK_ORDER_ID_DETAIL --env CONSOLE_UITEST_WORK_ORDER_ID_START \
  --env CONSOLE_UITEST_WORK_ORDER_ID_REPORT --env CONSOLE_UITEST_WORK_ORDER_ID_REPORT_SUCCESS \
  --env CONSOLE_UITEST_WORK_ORDER_ID_ADMIN_APPROVE --env CONSOLE_UITEST_WORK_ORDER_ID_ADMIN_REJECT \
  --env CONSOLE_UITEST_WORK_ORDER_ID_CAMERA --env CONSOLE_UITEST_MESSENGER_THREAD_ID \
  --env CONSOLE_UITEST_MESSENGER_INITIAL_MESSAGE_ID )

step "test"
# xcodebuild exits 64 WITHOUT running when this path already exists, which reads
# exactly like a test failure in a log tail.
rm -rf "$WORK/$SHARD.xcresult"
only_testing=()
for selector in "${SHARD_SELECTORS[@]}"; do only_testing+=("-only-testing:$selector"); done
set +e
xcodebuild test-without-building -parallel-testing-enabled NO \
  "${only_testing[@]}" -xctestrun "$XCTESTRUN" \
  -destination "platform=iOS Simulator,id=$UUID" \
  -resultBundlePath "$WORK/$SHARD.xcresult" >"$WORK/$SHARD.log" 2>&1
STATUS=$?
set -e
grep -aE "Test Case .*\]' (passed|failed)|\.swift:[0-9]+: error:" "$WORK/$SHARD.log" | tail -20
echo "xcodebuild exit=$STATUS  log=$WORK/$SHARD.log"
exit "$STATUS"
