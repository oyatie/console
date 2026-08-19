#!/usr/bin/env python3
"""Offline contract tests for tools/lanes/fanout.py's isolation algebra.

Run:  python3 tools/lanes/fanout.test.py

These are the two rules a lane harness cannot get wrong without lying about
isolation, and both were wrong here:

  1. `_in_slice` decided membership with exact-set equality (`f not in allow`),
     so a lane declaring a DIRECTORY saw every file beneath it as an out-of-slice
     breach. A harness whose isolation report is false in the safe direction
     still trains you to ignore it.
  2. `_build_env` set RUSTC_WRAPPER=sccache without CARGO_INCREMENTAL=0. Per
     scripts/console/lane-env.sh:26-37 that makes the cache invisible rather than
     cold: 0 hits, 0 misses, 0 non-cacheable. The wrapper looked configured and
     bought nothing.

No agent, no worktree, no network -- this proves the LOGIC, not any lane's
judgement.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def _load():
    spec = importlib.util.spec_from_file_location("fanout_under_test", HERE / "fanout.py")
    module = importlib.util.module_from_spec(spec)
    # Registered before exec: @dataclass resolves annotations through
    # sys.modules[cls.__module__] and raises AttributeError without it.
    sys.modules["fanout_under_test"] = module
    spec.loader.exec_module(module)
    return module


FANOUT = _load()
FAILURES: list[str] = []


def check(name: str, ok: bool, detail: str = "") -> None:
    print(f"{'PASS' if ok else 'FAIL'} {name}{'' if ok else f'  -- {detail}'}")
    if not ok:
        FAILURES.append(name)


def main() -> int:
    in_slice = FANOUT._in_slice

    # --- slice membership is a PATH prefix rule, not string or set equality ---
    check(
        "a declared directory covers files beneath it",
        in_slice("backend/crates/payroll/domain/src/lib.rs", ["backend/crates/payroll"]),
        "this is the exact-set bug: a lane owning a directory reported every file in it as a breach",
    )
    check("a declared path matches itself", in_slice("a/b", ["a/b"]))
    check("a declared file matches exactly", in_slice("a/b.rs", ["a/b.rs"]))
    check(
        "a shared string prefix is NOT a shared directory",
        not in_slice("backend/crates/payrollx/src/lib.rs", ["backend/crates/payroll"]),
        "payrollx is not inside payroll",
    )
    check("a path outside every entry is out of slice", not in_slice("scripts/x.mjs", ["backend/crates/payroll"]))
    check(
        "an empty allow list denies everything",
        not in_slice("a/b.rs", []),
        "a guard that examines zero subjects must not pass everything",
    )
    check("a trailing slash is tolerated", in_slice("a/b.rs", ["a/"]))
    check("blank entries are ignored, not treated as a wildcard", not in_slice("a/b.rs", ["", "   "]))

    ok = False
    try:
        in_slice("a/b", ["../etc"])
    except ValueError:
        ok = True
    check(
        "'..' in an allow entry is refused, not normalised",
        ok,
        "collapsing ownership across directories is what a slice must not do",
    )

    # --- the lane build environment must make sccache visible ---
    env = FANOUT._build_env()
    if env.get("RUSTC_WRAPPER") == "sccache":
        check(
            "CARGO_INCREMENTAL is pinned to 0 whenever sccache is the wrapper",
            env.get("CARGO_INCREMENTAL") == "0",
            "incremental compilations are never cacheable; sccache never SEES them "
            "(lane-env.sh:26-37 measured 0 hits / 0 misses / 0 non-cacheable)",
        )
        check("the cache ceiling is set", bool(env.get("SCCACHE_CACHE_SIZE")))
    else:
        print("SKIP sccache env checks -- sccache is not installed on this machine")

    print()
    if FAILURES:
        print(f"{len(FAILURES)} FAILED: {', '.join(FAILURES)}")
        return 1
    print("ALL PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
