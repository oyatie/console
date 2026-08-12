#!/usr/bin/env bash
# Thin adapter: normalize CAS env for console scripts.
#
# Preferred neutral names (CAS_*, CF_ACCESS_*). Existing OYA_CAS_* secret names are
# reorg debt — accepted as fallbacks only so the canary can run before secret rename.
# Do not proliferate OYA_/oya_ in new workflow IDs, job names, or receipt keys.
#
# Usage: source scripts/cas/load-cas-env.sh
set -euo pipefail

_cas_fallback() {
  local preferred="$1" legacy="$2"
  if [[ -z "${!preferred:-}" && -n "${!legacy:-}" ]]; then
    export "$preferred"="${!legacy}"
  fi
}

# Hosts / instance (GHA secrets today: OYA_CAS_* — debt)
_cas_fallback CAS_WRITE_HOST OYA_CAS_WRITE_HOST
_cas_fallback CAS_READ_HOST OYA_CAS_READ_HOST
_cas_fallback CAS_INSTANCE OYA_CAS_INSTANCE

# TLS PEM material (GHA secrets today: OYA_CAS_TLS_* — debt)
_cas_fallback CAS_TLS_CA OYA_CAS_TLS_CA
_cas_fallback CAS_TLS_CLIENT_CERT_WRITER OYA_CAS_TLS_CLIENT_CERT_WRITER
_cas_fallback CAS_TLS_CLIENT_KEY_WRITER OYA_CAS_TLS_CLIENT_KEY_WRITER
_cas_fallback CAS_TLS_CLIENT_CERT_READER OYA_CAS_TLS_CLIENT_CERT_READER
_cas_fallback CAS_TLS_CLIENT_KEY_READER OYA_CAS_TLS_CLIENT_KEY_READER

# Access service tokens — CF_ACCESS_* is already neutral Cloudflare naming
: "${CAS_WRITE_HOST:=cw.oyatie.dev}"
: "${CAS_READ_HOST:=cr.oyatie.dev}"
: "${CAS_INSTANCE:=main}"
: "${CAS_LOCAL_WRITE_PORT:=55051}"
: "${CAS_LOCAL_READ_PORT:=55052}"
: "${CAS_CLOUDFLARED_IMAGE:=cloudflare/cloudflared:2026.7.3}"
