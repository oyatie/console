#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
wrapper="${repo_root}/tools/buck/run_test_with_postgres_env.sh"
scratch="$(mktemp -d "${TMPDIR:-/tmp}/mnt-buck-wrapper-test.XXXXXX")"
trap 'rm -rf "${scratch}"' EXIT
valid="${scratch}/valid.env"
cat >"${valid}" <<'EOF'
DATABASE_URL=postgres://admin:secret@localhost/db
MNT_APALIS_OWNER_DATABASE_URL=postgres://app:secret@localhost/db
MNT_APALIS_RUNTIME_DATABASE_URL=postgres://rt:secret@localhost/db
MNT_APALIS_ADMIN_DATABASE_URL=postgres://admin:secret@localhost/db
EOF
chmod 600 "${valid}"
MNT_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/env | grep -Fqx 'DATABASE_URL=postgres://admin:secret@localhost/db'
malicious="${scratch}/malicious.env"
printf 'DATABASE_URL=$(touch %s)\n' "${scratch}/executed" >"${malicious}"
printf 'MNT_APALIS_OWNER_DATABASE_URL=x\nMNT_APALIS_RUNTIME_DATABASE_URL=x\nMNT_APALIS_ADMIN_DATABASE_URL=x\n' >>"${malicious}"
chmod 600 "${malicious}"
if MNT_BUCK_POSTGRES_ENV_FILE="${malicious}" "${wrapper}" /usr/bin/true >"${scratch}/wrapper.stdout" 2>"${scratch}/wrapper.stderr"; then exit 1; fi
[[ ! -e "${scratch}/executed" ]]
! grep -Fq 'touch ' "${scratch}/wrapper.stdout"
! grep -Fq 'touch ' "${scratch}/wrapper.stderr"
chmod 644 "${valid}"
if MNT_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/true; then exit 1; fi
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
PATH="${scratch}/bin:${PATH}" MNT_BUCK_POSTGRES_ENV_FILE="${valid}" "${wrapper}" /usr/bin/true
exact_log="${scratch}/exact.log"
cat >"${scratch}/test-binary" <<'BINARY'
#!/usr/bin/env bash
printf '%s\n' "$@" >"${EXACT_LOG}"
BINARY
chmod 755 "${scratch}/test-binary"
MNT_BUCK_POSTGRES_ENV_FILE="${valid}" MNT_BUCK_RUST_TEST_EXACT=one_exact_test EXACT_LOG="${exact_log}" "${wrapper}" "${scratch}/test-binary"
[[ "$(cat "${exact_log}")" == $'--exact\none_exact_test' ]]
if MNT_BUCK_POSTGRES_ENV_FILE="${valid}" MNT_BUCK_RUST_TEST_EXACT='bad test' "${wrapper}" /usr/bin/true; then exit 1; fi
echo 'run_test_with_postgres_env: PASS'
