#!/usr/bin/env python3
"""Bound one local recovery harness process group; stdlib, no global cleanup.

Fixed limits: 600 seconds wall clock; 10 seconds for owned-group cleanup.
Timeout/interrupt is never product RED or success. The shell EXIT trap owns
Docker cleanup; if cleanup cannot finish, preserve its exact resource labels
and report timeout rather than concealing a possible owned-resource leak.
"""
import json
import os
import pathlib
import signal
import subprocess
import sys
import time

WALL_SECONDS = 600
GRACE_SECONDS = 10


def supervise(argv, wall_seconds=WALL_SECONDS, grace_seconds=GRACE_SECONDS):
    interrupted = []

    def on_signal(signum, _frame):
        interrupted.append(signum)

    previous = {s: signal.signal(s, on_signal) for s in (signal.SIGTERM, signal.SIGINT)}
    child = None
    try:
        child = subprocess.Popen(argv, start_new_session=True)
        deadline = time.monotonic() + wall_seconds
        while child.poll() is None and not interrupted and time.monotonic() < deadline:
            try:
                return child.wait(timeout=min(0.2, max(0.001, deadline - time.monotonic())))
            except subprocess.TimeoutExpired:
                pass
        if child.poll() is not None:
            return child.returncode
        cause = "interrupt" if interrupted else "wall_clock_timeout"
        print(json.dumps({"recovery_supervisor": cause, "classification": "FIXTURE_INCOMPLETE"}), flush=True)
        try:
            os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        # Do not return as soon as the leader exits: descendants may survive it.
        grace_deadline = time.monotonic() + grace_seconds
        while time.monotonic() < grace_deadline:
            child.poll()
            try:
                os.killpg(child.pid, 0)
            except ProcessLookupError:
                break
            time.sleep(max(0, min(0.1, grace_deadline - time.monotonic())))
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait(timeout=2)
        return 128 + interrupted[0] if interrupted else 124
    finally:
        for signum, handler in previous.items():
            signal.signal(signum, handler)


def main():
    if len(sys.argv) < 4 or sys.argv[2] != "--":
        raise SystemExit("usage: supervise_recovery.py REPO -- COMMAND...")
    harness = pathlib.Path(__file__).resolve().with_name("recovery_postgres.sh")
    repo = pathlib.Path(sys.argv[1]).resolve(strict=True)
    raise SystemExit(supervise(["bash", str(harness), str(repo), "--", *sys.argv[3:]]))


if __name__ == "__main__":
    main()
