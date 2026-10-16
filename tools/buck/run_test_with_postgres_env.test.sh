#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
wrapper="${repo_root}/tools/buck/run_test_with_postgres_env.sh"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/console-buck-wrapper-test.XXXXXX")"
trap 'rm -rf "${scratch}"' EXIT
valid="${scratch}/valid.env"
cat >"${valid}" <<'EOF'
DATABASE_URL=postgres://admin:secret@localhost/db
CONSOLE_APALIS_OWNER_DATABASE_URL=postgres://app:secret@localhost/db
CONSOLE_APALIS_RUNTIME_DATABASE_URL=postgres://rt:secret@localhost/db
CONSOLE_APALIS_ADMIN_DATABASE_URL=postgres://admin:secret@localhost/db
EOF
chmod 600 "${valid}"
CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/env | grep -Fqx 'DATABASE_URL=postgres://admin:secret@localhost/db'
malicious="${scratch}/malicious.env"
printf 'DATABASE_URL=$(touch %s)\n' "${scratch}/executed" >"${malicious}"
printf 'CONSOLE_APALIS_OWNER_DATABASE_URL=x\nCONSOLE_APALIS_RUNTIME_DATABASE_URL=x\nCONSOLE_APALIS_ADMIN_DATABASE_URL=x\n' >>"${malicious}"
chmod 600 "${malicious}"
if CONSOLE_BUCK_POSTGRES_ENV_FILE="${malicious}" "${wrapper}" /usr/bin/true >"${scratch}/wrapper.stdout" 2>"${scratch}/wrapper.stderr"; then exit 1; fi
[[ ! -e "${scratch}/executed" ]]
! grep -Fq 'touch ' "${scratch}/wrapper.stdout"
! grep -Fq 'touch ' "${scratch}/wrapper.stderr"
chmod 644 "${valid}"
if CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/true; then exit 1; fi
# GNU stat fallback: force BSD form to fail, then delegate -c to system stat.
mkdir "${scratch}/bin"
cat >"${scratch}/bin/stat" <<'STAT'
#!/usr/bin/env bash
if [[ "$1" == -f ]]; then exit 1; fi
if [[ "$1" == -c && "$2" == %a ]]; then echo 600; exit 0; fi
exit 1
STAT
chmod 755 "${scratch}/bin/stat"
chmod 600 "${valid}"
PATH="${scratch}/bin:${PATH}" CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/true
exact_log="${scratch}/exact.log"
cat >"${scratch}/test-binary" <<'BINARY'
#!/usr/bin/env bash
if [[ "$1" == --list ]]; then
  [[ "$#" == 3 && "$2" == --exact && "$3" == one_exact_test ]] || exit 1
  printf 'one_exact_test: test\n\n1 test, 0 benchmarks\n'
  exit 0
fi
printf '%s\n' "$@" >"${EXACT_LOG}"
BINARY
chmod 755 "${scratch}/test-binary"
CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" CONSOLE_BUCK_RUST_TEST_EXACT=one_exact_test EXACT_LOG="${exact_log}" "${wrapper}" "${scratch}/test-binary"
[[ "$(cat "${exact_log}")" == $'--exact\none_exact_test' ]]
if CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" CONSOLE_BUCK_RUST_TEST_EXACT='bad test' "${wrapper}" /usr/bin/true; then exit 1; fi
# The restricted Account/command transports remain data, with duplicate refusal.
for binding in AUTH:auth_rt LEAVE_COMMAND:leave_cmd ONTOLOGY_COMMAND:ontology_cmd PLATFORM_FORCE_COMMAND:platform_force_cmd; do
  key="CONSOLE_TEST_${binding%%:*}_DATABASE_URL"
  value="postgres://console_${binding#*:}:secret@localhost/db"
  printf '%s=%s\n' "${key}" "${value}" >>"${valid}"
  CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/env | grep -Fqx "${key}=${value}"
  cp "${valid}" "${scratch}/duplicate.env"
  printf '%s=%s\n' "${key}" "${value}" >>"${scratch}/duplicate.env"
  chmod 600 "${scratch}/duplicate.env"
  if CONSOLE_BUCK_POSTGRES_ENV_FILE="${scratch}/duplicate.env" "${wrapper}" /usr/bin/true; then exit 1; fi
done
# All historical eight keys above remain valid without the optional startup key.
CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/true
cat >"${scratch}/startup-child" <<'STARTUP_CHILD'
#!/usr/bin/env bash
touch "${STARTUP_CHILD_MARKER}"
STARTUP_CHILD
chmod 700 "${scratch}/startup-child"
for required_key in DATABASE_URL CONSOLE_APALIS_OWNER_DATABASE_URL CONSOLE_APALIS_RUNTIME_DATABASE_URL CONSOLE_APALIS_ADMIN_DATABASE_URL; do
  grep -v "^${required_key}=" "${valid}" >"${scratch}/startup-incomplete.env"
  chmod 600 "${scratch}/startup-incomplete.env"
  if STARTUP_CHILD_MARKER="${scratch}/startup-child-ran" CONSOLE_BUCK_POSTGRES_ENV_FILE="${scratch}/startup-incomplete.env" "${wrapper}" "${scratch}/startup-child" >"${scratch}/startup.stdout" 2>"${scratch}/startup.stderr"; then exit 1; fi
  grep -Fq 'incomplete environment file' "${scratch}/startup.stderr"
  [[ ! -e "${scratch}/startup-child-ran" ]]
done
startup_value='postgres://console_auth_startup:startup-secret@localhost/db'
printf 'CONSOLE_STARTUP_AUTH_DATABASE_URL=%s\n' "${startup_value}" >>"${valid}"
CONSOLE_STARTUP_AUTH_DATABASE_URL='postgres://wrong:inherited@wrong/db' CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/env | grep -Fqx "CONSOLE_STARTUP_AUTH_DATABASE_URL=${startup_value}"
for invalid_kind in duplicate empty substitution unknown; do
  cp "${valid}" "${scratch}/startup-invalid.env"
  case "${invalid_kind}" in
    duplicate) printf 'CONSOLE_STARTUP_AUTH_DATABASE_URL=%s\n' "${startup_value}" >>"${scratch}/startup-invalid.env" ;;
    empty) printf 'CONSOLE_STARTUP_AUTH_DATABASE_URL=\n' >>"${scratch}/startup-invalid.env" ;;
    substitution) printf 'CONSOLE_STARTUP_AUTH_DATABASE_URL=$(touch %s)\n' "${scratch}/startup-executed" >>"${scratch}/startup-invalid.env" ;;
    unknown) printf 'CONSOLE_STARTUP_AUTH_DATABASE_URL_EXTRA=%s\n' "${startup_value}" >>"${scratch}/startup-invalid.env" ;;
  esac
  chmod 600 "${scratch}/startup-invalid.env"
  if STARTUP_CHILD_MARKER="${scratch}/startup-child-ran" CONSOLE_BUCK_POSTGRES_ENV_FILE="${scratch}/startup-invalid.env" "${wrapper}" "${scratch}/startup-child" >"${scratch}/startup.stdout" 2>"${scratch}/startup.stderr"; then exit 1; fi
  case "${invalid_kind}" in
    duplicate) grep -Fq 'duplicate environment key' "${scratch}/startup.stderr" ;;
    empty|substitution) grep -Fq 'malformed environment file' "${scratch}/startup.stderr" ;;
    unknown) grep -Fq 'unexpected environment key' "${scratch}/startup.stderr" ;;
  esac
  [[ ! -e "${scratch}/startup-executed" && ! -e "${scratch}/startup-child-ran" ]]
  ! grep -Eq 'startup-secret|touch |postgres://' "${scratch}/startup.stdout" "${scratch}/startup.stderr"
done
# Exact native selection must discover exactly the named test before execution.
# Fake libtest proves wrapper plumbing; it does not certify native test execution.
cat >"${scratch}/discovery-binary" <<'DISCOVERY_BINARY'
#!/usr/bin/env bash
set -euo pipefail
[[ "${DATABASE_URL}" == 'postgres://admin:secret@localhost/db' ]]
[[ "${CONSOLE_APALIS_OWNER_DATABASE_URL}" == 'postgres://app:secret@localhost/db' ]]
[[ "$1" == fixed-arg ]]
shift
if [[ "$1" == --list ]]; then
  [[ "$#" == 3 && "$2" == --exact && "$3" == selected_test ]]
  printf 'listed\n' >>"${DISCOVERY_LOG}"
  case "${DISCOVERY_MODE}" in
    valid|child-failure) printf 'selected_test: test\n\n1 test, 0 benchmarks\n' ;;
    empty) printf '\n0 tests, 0 benchmarks\n' ;;
    ambiguous) printf 'selected_test: test\nselected_test: test\n\n2 tests, 0 benchmarks\n' ;;
    error) printf 'postgres://secret:should-not-leak@localhost/db\n' >&2; exit 23 ;;
    wrong-name) printf 'other_test: test\n\n1 test, 0 benchmarks\n' ;;
    benchmark) printf 'selected_test: benchmark\n\n0 tests, 1 benchmark\n' ;;
    no-summary) printf 'selected_test: test\n' ;;
    extra-output) printf 'selected_test: test\n\n1 test, 0 benchmarks\nunexpected\n' ;;
    *) exit 91 ;;
  esac
  exit 0
fi
[[ "$#" == 2 && "$1" == --exact && "$2" == selected_test ]]
printf 'executed\n' >>"${DISCOVERY_LOG}"
[[ "${DISCOVERY_MODE}" != child-failure ]] || exit 29
DISCOVERY_BINARY
chmod 755 "${scratch}/discovery-binary"
discovery_log="${scratch}/discovery.log"
DISCOVERY_MODE=valid DISCOVERY_LOG="${discovery_log}" CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" CONSOLE_BUCK_RUST_TEST_EXACT=selected_test "${wrapper}" "${scratch}/discovery-binary" fixed-arg
[[ "$(cat "${discovery_log}")" == $'listed\nexecuted' ]]
for mode in empty ambiguous error wrong-name benchmark no-summary extra-output; do
  : >"${discovery_log}"
  if DISCOVERY_MODE="${mode}" DISCOVERY_LOG="${discovery_log}" CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" CONSOLE_BUCK_RUST_TEST_EXACT=selected_test "${wrapper}" "${scratch}/discovery-binary" fixed-arg >"${scratch}/discovery.stdout" 2>"${scratch}/discovery.stderr"; then
    echo "EXACT_DISCOVERY_REQUIRED: ${mode}" >&2
    exit 1
  fi
  [[ "$(cat "${discovery_log}")" == listed ]]
  grep -Fq 'exact Rust test discovery failed' "${scratch}/discovery.stderr"
  ! grep -Eq 'postgres://|should-not-leak' "${scratch}/discovery.stdout" "${scratch}/discovery.stderr"
done
: >"${discovery_log}"
rc=0
DISCOVERY_MODE=child-failure DISCOVERY_LOG="${discovery_log}" CONSOLE_BUCK_POSTGRES_ENV_FILE="${valid}" CONSOLE_BUCK_RUST_TEST_EXACT=selected_test "${wrapper}" "${scratch}/discovery-binary" fixed-arg || rc=$?
[[ "${rc}" == 29 && "$(cat "${discovery_log}")" == $'listed\nexecuted' ]]

echo 'run_test_with_postgres_env: PASS'
