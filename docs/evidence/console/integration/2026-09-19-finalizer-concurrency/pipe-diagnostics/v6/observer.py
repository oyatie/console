#!/usr/bin/env python3
"""Temporary observer: exact ordinary nextest argv, cold then warm, same cluster."""
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time
from urllib.parse import parse_qsl, unquote, urlsplit

CARGO = "/Users/jasonlee/.cargo/bin/cargo"
PSQL = "/opt/homebrew/bin/psql"
args = sys.argv[1:]
if args[:2] != ["nextest", "run"]:
    os.execv(CARGO, [CARGO, *args])

os.umask(0o077)
output = Path(tempfile.mkdtemp(prefix="run-", dir=Path(__file__).resolve().parent))
receipt = {"argv_sha256": hashlib.sha256(json.dumps(args).encode()).hexdigest(),
           "expected_tests_per_phase": 15, "phases": [], "accepted": False}
receipt_path = output / "receipt.json"

def save():
    receipt_path.write_text(json.dumps(receipt, indent=2) + "\n")

def interrupted(signum, _frame):
    raise SystemExit(128 + signum)

signal.signal(signal.SIGTERM, interrupted)
signal.signal(signal.SIGINT, interrupted)

def observe():
    if not os.environ.get("DATABASE_URL"):
        raise RuntimeError("DATABASE_URL prerequisite absent")
    query = """SELECT json_build_object(
      'credential_role_present', EXISTS(SELECT 1 FROM pg_catalog.pg_roles
        WHERE rolname='console_credential_owner'),
      'system_identifier', (pg_catalog.pg_control_system()).system_identifier::text,
      'database', current_database(),
      'server_version_num', current_setting('server_version_num'),
      'marked_admin', session_user='console_buck_admin' AND current_user=session_user
        AND current_setting('console.sqlx_test_bootstrap',true)='buck-sqlx-superuser-v1')::text"""
    uri = urlsplit(os.environ["DATABASE_URL"])
    if (uri.scheme not in ("postgres", "postgresql") or uri.fragment
        or parse_qsl(uri.query, keep_blank_values=True, strict_parsing=True)
        != [("options[console.sqlx_test_bootstrap]", "buck-sqlx-superuser-v1")]):
        raise RuntimeError("unexpected disposable SQLx connection options")
    # Translate the exact admitted SQLx URI into private libpq fields. PGDATABASE
    # supplied by the environment is a database name, not an expanded conninfo URI.
    user = unquote(uri.username or "")
    password = unquote(uri.password or "")
    database = unquote(uri.path[1:]) if uri.path.startswith("/") else ""
    if (uri.hostname != "127.0.0.1" or not uri.port or user != "console_buck_admin"
        or not password or not database or "/" in database
        or any("\x00" in value for value in (user, password, database))):
        raise RuntimeError("unexpected disposable SQLx connection identity")
    env = {key: value for key, value in os.environ.items() if not key.startswith("PG")}
    env.update({"PGHOST": uri.hostname, "PGPORT": str(uri.port), "PGUSER": user,
                "PGPASSWORD": password, "PGDATABASE": database,
                "PGOPTIONS": "-c console.sqlx_test_bootstrap=buck-sqlx-superuser-v1",
                "PGCONNECT_TIMEOUT": "5"})
    result = subprocess.run([PSQL, "-X", "-w", "-A", "-t", "-v", "ON_ERROR_STOP=1", "-c", query],
                            env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            text=True, timeout=15)
    if result.returncode:
        # Classify fixed phrases without emitting server text, identifiers or secrets.
        error = result.stderr.lower()
        categories = [("password authentication failed", "authentication"),
                      ("connection refused", "connection-refused"),
                      ("does not exist", "missing-database-or-object"),
                      ("permission denied", "permission"),
                      ("timeout", "timeout")]
        category = next((label for phrase, label in categories if phrase in error), "unclassified")
        raise RuntimeError(f"read-only PostgreSQL observation failed, status {result.returncode}, category {category}")
    observed = json.loads(result.stdout)
    if observed.get("marked_admin") is not True:
        raise RuntimeError("marked disposable administrator prerequisite absent")
    return observed

def execute(phase):
    log = output / f"{phase['name']}.log"
    phase["log"] = str(log)
    phase["started_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    started = time.monotonic()
    process = None
    watcher = None
    try:
        with log.open("xb") as stream:
            process = subprocess.Popen([CARGO, *args], stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            from process_watch import ProcessWatch
            watcher = ProcessWatch(output / (phase["name"] + "-processes.jsonl"), process.pid)
            for line in process.stdout:
                with (output / (phase["name"] + "-line-times.jsonl")).open("a") as timings:
                    timings.write(json.dumps({"monotonic": time.monotonic(), "line": line.decode("utf-8", "replace")}) + "\n")
                stream.write(line)
                stream.flush()
                sys.stdout.buffer.write(line)
                sys.stdout.buffer.flush()
            phase["returncode"] = process.wait()
    finally:
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        if process is not None:
            phase["returncode"] = process.returncode
        if watcher is not None:
            watcher.close()
        phase["elapsed_seconds"] = round(time.monotonic() - started, 3)
        save()
    plain = re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", log.read_text(errors="replace"))
    summaries = [line for line in plain.splitlines() if re.search(r"\bSummary\s+\[", line)]
    phase["summary"] = summaries[-1] if len(summaries) == 1 else None
    summary = phase["summary"] or ""
    counts = re.search(r"\b(\d+) tests run:\s+(\d+) passed\b", summary)
    skipped = re.search(r"\b(\d+) skipped\b", summary)
    phase["tests_run"] = int(counts[1]) if counts else None
    phase["passed"] = int(counts[2]) if counts else None
    phase["skipped"] = int(skipped[1]) if skipped else None
    leaky = re.search(r"\((\d+) leaky\)", summary)
    phase["leaky"] = int(leaky[1]) if leaky else 0
    phase["leak_annotations"] = len(re.findall(r"^\s*LEAK\s+\[", plain, re.MULTILINE))
    phase["accepted"] = (phase["leaky"] == 0 and phase["leak_annotations"] == 0 and phase["returncode"] == 0 and phase["tests_run"] == 15
                         and phase["passed"] == 15 and phase["skipped"] == 0)
    save()

from diagnostics import Diagnostics
diagnostics = None

print(f"COHORT_OBSERVER_RECEIPT={receipt_path}", flush=True)
save()
try:
    diagnostics = Diagnostics(output)
    original = None
    for name, expected_presence in [("cold", False), ("warm", True)]:
        phase = {"name": name, "accepted": False, "returncode": None}
        receipt["phases"].append(phase)
        observed = observe()
        phase["before"] = observed
        identity = (observed["system_identifier"], observed["database"], observed["server_version_num"])
        if original is None:
            original = identity
        if identity != original or observed["credential_role_present"] is not expected_presence:
            raise RuntimeError(f"{name} same-cluster/role-presence prerequisite failed; phase not run")
        print(f"COHORT_PHASE_BEGIN={name};credential_role_present={expected_presence}", flush=True)
        save()
        execute(phase)
        diagnostics.capture("after-" + name)
        print(f"COHORT_PHASE_END={name};status={phase['returncode']};tests={phase['tests_run']};passed={phase['passed']};skipped={phase['skipped']};seconds={phase['elapsed_seconds']}", flush=True)
    receipt["accepted"] = all(phase["accepted"] for phase in receipt["phases"])
except Exception as error:
    # Exception types are enough for diagnostics; never print connection arguments.
    receipt["observer_error"] = str(error) if isinstance(error, RuntimeError) else type(error).__name__
    print(f"COHORT_OBSERVER_FAILED={receipt['observer_error']}", file=sys.stderr)
finally:
    if diagnostics is not None:
        diagnostics.close()
    save()
sys.exit(0 if receipt["accepted"] else 1)
