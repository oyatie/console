#!/usr/bin/env bash
# Parse a Buck2 build log for warm-cache proof metrics. Never prints secrets.
#
# Usage:
#   scripts/cas/assert-warm-metrics.sh --mode upload --log path
#   scripts/cas/assert-warm-metrics.sh --mode hit --log path
#
# upload: requires Network Up > 0B OR cache upload evidence
# hit:    requires Cache hits > 0% OR cached > 0 OR remote cache hit lines
set -euo pipefail

MODE=""
LOG=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode) MODE="$2"; shift 2 ;;
    --log) LOG="$2"; shift 2 ;;
    -h|--help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

[[ -n "$MODE" && -n "$LOG" ]] || { echo "--mode and --log required" >&2; exit 2; }
[[ -f "$LOG" ]] || { echo "missing log: $LOG" >&2; exit 1; }

python3 - "$MODE" "$LOG" <<'PY'
import re, sys
mode, path = sys.argv[1], sys.argv[2]
text = open(path, errors="replace").read()

def parse_bytes(s: str) -> int:
    s = s.strip().upper().replace(" ", "")
    m = re.fullmatch(r"([0-9]*\.?[0-9]+)([KMGTP]?I?B)", s)
    if not m:
        return -1
    n = float(m.group(1))
    unit = m.group(2)
    mul = {
        "B": 1,
        "KB": 1000, "MB": 1000**2, "GB": 1000**3,
        "KIB": 1024, "MIB": 1024**2, "GIB": 1024**3,
        "TIB": 1024**4, "PIB": 1024**5,
    }.get(unit, 1)
    return int(n * mul)

cache_hits = None
m = re.search(r"Cache hits:\s*([0-9]+(?:\.[0-9]+)?)%", text)
if m:
    cache_hits = float(m.group(1))

cached = remote = local = None
m = re.search(
    r"Commands:\s*([0-9]+)\s*\(cached:\s*([0-9]+),\s*remote:\s*([0-9]+),\s*local:\s*([0-9]+)\)",
    text,
)
if m:
    cached, remote, local = int(m.group(2)), int(m.group(3)), int(m.group(4))

up_bytes = None
m = re.search(r"Network:\s*Up:\s*([0-9]*\.?[0-9]+\s*[KMGTP]?i?B)", text, re.I)
if m:
    up_bytes = parse_bytes(m.group(1))

upload_events = len(re.findall(r"cache upload|Uploaded|ActionCache.*Update", text, re.I))
remote_hit_lines = len(re.findall(r"remote cache hit|Cache hit|RE.*cache", text, re.I))

print(f"metrics cache_hits_pct={cache_hits} cached={cached} remote={remote} local={local} up_bytes={up_bytes} upload_events={upload_events} remote_hit_lines={remote_hit_lines}")

ok = False
if mode == "upload":
    ok = (up_bytes is not None and up_bytes > 0) or upload_events > 0
    if not ok:
        print("ASSERT_FAIL upload: need Network Up > 0B or upload events", file=sys.stderr)
        sys.exit(1)
elif mode == "hit":
    ok = (
        (cache_hits is not None and cache_hits > 0)
        or (cached is not None and cached > 0)
        or remote_hit_lines > 0
    )
    if not ok:
        print("ASSERT_FAIL hit: need Cache hits > 0% or cached > 0 or remote hit lines", file=sys.stderr)
        sys.exit(1)
else:
    print(f"unknown mode: {mode}", file=sys.stderr)
    sys.exit(2)

print(f"ASSERT_OK mode={mode}")
PY
