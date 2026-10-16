#!/bin/sh
set -eu
if [ "$1" = test ]; then
  exec /private/tmp/console-company-admission-repair-20260921/tools/buck2 "$@" --env RUST_BACKTRACE=1
fi
exec /private/tmp/console-company-admission-repair-20260921/tools/buck2 "$@"
