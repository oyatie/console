#!/usr/bin/env bash
# LC02: real TLS PostgreSQL, production migrator and production wrapper.
set -euo pipefail
exec python3 "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/postgres-finalize-account-custody.integration.test.py" "$@"
