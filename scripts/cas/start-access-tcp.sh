#!/usr/bin/env bash
# Start Cloudflare Access TCP forwarders for shared CAS (GHA / remote path).
# Containers are left running by default so later steps can use them.
# Tear down with: scripts/cas/start-access-tcp.sh --cleanup
#
# Env (neutral preferred; see load-cas-env.sh for OYA_* debt fallbacks):
#   CF_ACCESS_WRITE_CLIENT_ID / CF_ACCESS_WRITE_CLIENT_SECRET
#   CF_ACCESS_READ_CLIENT_ID / CF_ACCESS_READ_CLIENT_SECRET
#   CAS_WRITE_HOST / CAS_READ_HOST (default cw/cr.oyatie.dev)
#   CAS_LOCAL_WRITE_PORT / CAS_LOCAL_READ_PORT (default 55051 / 55052)
#
# Flags:
#   --role writer|reader|both   (default both)
#   --cleanup                   remove console-cas-access-* containers and exit
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
# shellcheck source=scripts/cas/load-cas-env.sh
source "$ROOT/scripts/cas/load-cas-env.sh"

ROLE="both"
CLEANUP=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --role) ROLE="$2"; shift 2 ;;
    --cleanup) CLEANUP=1; shift ;;
    -h|--help)
      sed -n '2,18p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

if [[ "$CLEANUP" -eq 1 ]]; then
  docker rm -f console-cas-access-cw console-cas-access-cr >/dev/null 2>&1 || true
  echo "cleaned Access TCP forwarders"
  exit 0
fi

need() { command -v "$1" >/dev/null || { echo "missing: $1" >&2; exit 4; }; }
need docker
need curl
need python3

resolve_a() {
  curl -fsS -H 'accept: application/dns-json' "https://1.1.1.1/dns-query?name=$1&type=A" \
    | python3 -c 'import sys,json; a=json.load(sys.stdin).get("Answer") or []; print(next(x["data"] for x in a if x.get("type")==1))'
}

start_forwarder() {
  local name="$1" host="$2" ip="$3" port="$4" id="$5" secret="$6"
  docker rm -f "$name" >/dev/null 2>&1 || true
  docker run -d --name "$name" \
    --add-host "${host}:${ip}" \
    -p "127.0.0.1:${port}:${port}" \
    -e "TUNNEL_SERVICE_TOKEN_ID=${id}" \
    -e "TUNNEL_SERVICE_TOKEN_SECRET=${secret}" \
    "$CAS_CLOUDFLARED_IMAGE" \
    access tcp --hostname "$host" --url "0.0.0.0:${port}" >/dev/null
  echo "started Access TCP forwarder name=${name} host=${host} port=${port}"
}

echo "=== console CAS Access TCP forwarders ==="
echo "write_host=${CAS_WRITE_HOST} read_host=${CAS_READ_HOST} role=${ROLE}"

STARTED=0
if [[ "$ROLE" == "writer" || "$ROLE" == "both" ]]; then
  [[ -n "${CF_ACCESS_WRITE_CLIENT_ID:-}" && -n "${CF_ACCESS_WRITE_CLIENT_SECRET:-}" ]] \
    || { echo "missing CF_ACCESS_WRITE_CLIENT_{ID,SECRET}" >&2; exit 1; }
  WRITE_IP="$(resolve_a "$CAS_WRITE_HOST")"
  start_forwarder console-cas-access-cw "$CAS_WRITE_HOST" "$WRITE_IP" \
    "$CAS_LOCAL_WRITE_PORT" "$CF_ACCESS_WRITE_CLIENT_ID" "$CF_ACCESS_WRITE_CLIENT_SECRET"
  STARTED=1
fi
if [[ "$ROLE" == "reader" || "$ROLE" == "both" ]]; then
  [[ -n "${CF_ACCESS_READ_CLIENT_ID:-}" && -n "${CF_ACCESS_READ_CLIENT_SECRET:-}" ]] \
    || { echo "missing CF_ACCESS_READ_CLIENT_{ID,SECRET}" >&2; exit 1; }
  READ_IP="$(resolve_a "$CAS_READ_HOST")"
  start_forwarder console-cas-access-cr "$CAS_READ_HOST" "$READ_IP" \
    "$CAS_LOCAL_READ_PORT" "$CF_ACCESS_READ_CLIENT_ID" "$CF_ACCESS_READ_CLIENT_SECRET"
  STARTED=1
fi
[[ "$STARTED" -eq 1 ]] || { echo "role must be writer|reader|both" >&2; exit 2; }

sleep 2
echo "Access TCP forwarders ready"
