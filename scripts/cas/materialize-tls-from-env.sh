#!/usr/bin/env bash
# Materialize client mTLS files from env (GHA secrets or local export).
# Never prints secret values. Writes mode-0600 PEMs under --out-dir.
#
# Env (after scripts/cas/load-cas-env.sh):
#   CAS_TLS_CA
#   CAS_TLS_CLIENT_CERT_{WRITER|READER}
#   CAS_TLS_CLIENT_KEY_{WRITER|READER}
#
# Legacy OYA_CAS_TLS_* accepted via load-cas-env.sh (reorg debt).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=scripts/cas/load-cas-env.sh
source "$ROOT/scripts/cas/load-cas-env.sh"

ROLE="writer"
OUT_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --role) ROLE="$2"; shift 2 ;;
    --out-dir) OUT_DIR="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,16p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

[[ -n "$OUT_DIR" ]] || { echo "--out-dir required" >&2; exit 2; }
case "$ROLE" in
  writer|reader) ;;
  *) echo "role must be writer|reader" >&2; exit 2 ;;
esac

need_var() {
  local n="$1"
  [[ -n "${!n:-}" ]] || { echo "missing env: $n (or legacy OYA_* equivalent)" >&2; exit 1; }
}

need_var CAS_TLS_CA
if [[ "$ROLE" == "writer" ]]; then
  need_var CAS_TLS_CLIENT_CERT_WRITER
  need_var CAS_TLS_CLIENT_KEY_WRITER
else
  need_var CAS_TLS_CLIENT_CERT_READER
  need_var CAS_TLS_CLIENT_KEY_READER
fi

umask 077
mkdir -p "$OUT_DIR"
printf '%s\n' "$CAS_TLS_CA" >"$OUT_DIR/ca.crt"
if [[ "$ROLE" == "writer" ]]; then
  printf '%s\n' "$CAS_TLS_CLIENT_CERT_WRITER" >"$OUT_DIR/client-writer.crt"
  printf '%s\n' "$CAS_TLS_CLIENT_KEY_WRITER" >"$OUT_DIR/client-writer.key"
  chmod 600 "$OUT_DIR/ca.crt" "$OUT_DIR/client-writer.crt" "$OUT_DIR/client-writer.key"
else
  printf '%s\n' "$CAS_TLS_CLIENT_CERT_READER" >"$OUT_DIR/client-reader.crt"
  printf '%s\n' "$CAS_TLS_CLIENT_KEY_READER" >"$OUT_DIR/client-reader.key"
  chmod 600 "$OUT_DIR/ca.crt" "$OUT_DIR/client-reader.crt" "$OUT_DIR/client-reader.key"
fi
echo "materialized TLS role=${ROLE} out_dir=${OUT_DIR} (modes 600; values not printed)"
