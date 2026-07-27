#!/usr/bin/env bash
# Surgically rewrite the @sha256 image digests in the prod kustomize overlay.
#
#   bump-prod-digests.sh <console-app-digest> <console-web-digest>
#
# Each digest must be a full `sha256:<64-hex>` string (the exact value
# docker/build-push-action emits as steps.build.outputs.digest, i.e. the same
# immutable artifact image-release scanned + cosign-signed). This rewrites ONLY
# the two `digest:` lines under the `console-app` / `console-web` image entries in
#   deploy/apps/console/overlays/prod/kustomization.yaml
# leaving every comment, blank line and the patches block byte-for-byte intact
# (a kustomize/yq rewrite would reflow the whole, heavily-commented file). Pure
# awk + coreutils so it needs neither kustomize nor yq on the runner.
#
# Idempotent: re-running with the same digests is a no-op and exits 0. Exits
# non-zero (touching nothing) if a digest is malformed or an entry is missing.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OVERLAY="${REPO_ROOT}/deploy/apps/console/overlays/prod/kustomization.yaml"

if [[ $# -ne 2 ]]; then
  echo "usage: bump-prod-digests.sh <console-app-sha256-digest> <console-web-sha256-digest>" >&2
  exit 2
fi

APP_DIGEST="$1"
WEB_DIGEST="$2"

# Validate shape up front so a bad value never lands in the manifest.
digest_re='^sha256:[0-9a-f]{64}$'
for d in "${APP_DIGEST}" "${WEB_DIGEST}"; do
  if [[ ! "${d}" =~ ${digest_re} ]]; then
    echo "bump: refusing to write malformed digest '${d}' (want sha256:<64 hex>)" >&2
    exit 1
  fi
done

if [[ ! -f "${OVERLAY}" ]]; then
  echo "bump: overlay not found: ${OVERLAY}" >&2
  exit 1
fi

# Rewrite the `digest:` line that follows each `- name: <image>` entry. Track
# which image block we are inside; rewrite the first digest line seen in it,
# then clear the marker so a malformed extra digest line is left untouched.
tmp="$(mktemp)"
trap 'rm -f "${tmp}"' EXIT
awk -v app="${APP_DIGEST}" -v web="${WEB_DIGEST}" '
  /^[[:space:]]*-[[:space:]]*name:[[:space:]]*console-app[[:space:]]*$/ { cur = "app" }
  /^[[:space:]]*-[[:space:]]*name:[[:space:]]*console-web[[:space:]]*$/ { cur = "web" }
  /^[[:space:]]*digest:[[:space:]]/ && cur == "app" { sub(/digest:[[:space:]].*/, "digest: " app); cur = ""; seen_app = 1 }
  /^[[:space:]]*digest:[[:space:]]/ && cur == "web" { sub(/digest:[[:space:]].*/, "digest: " web); cur = ""; seen_web = 1 }
  { print }
  END {
    if (!seen_app) { print "bump: no digest line found for console-app" > "/dev/stderr"; exit 3 }
    if (!seen_web) { print "bump: no digest line found for console-web" > "/dev/stderr"; exit 3 }
  }
' "${OVERLAY}" > "${tmp}"

mv "${tmp}" "${OVERLAY}"

echo "bump: console-app -> ${APP_DIGEST}" >&2
echo "bump: console-web -> ${WEB_DIGEST}" >&2
echo "bump: wrote ${OVERLAY}" >&2
