#!/usr/bin/env python3
"""Durable lane-fanout harness: parallel agents, isolation measured, not assumed.

Encodes what was MEASURED on 2026-07-28 rather than what was assumed:

  * 4 concurrent codex lanes in separate git worktrees completed in 37s with
    ZERO files touched outside their slice and 4/4 outputs correct.
  * codex is unusable without a precise flag contract (see CODEX_BASE_FLAGS);
    three separate layers each silently sealed it, turning a 10s task into a
    17-minute hang that looked like model latency.
  * Cross-family adversarial review is the highest-return mechanism available:
    it caught 38 fabricated doc claims, 21 live-infra misclassifications, a
    false plan premise verified twice by its author, and a repair that fixed
    1 of 3 commands while reporting green.
  * Verification probes broke THREE times while the code under test was fine.
    A probe that has never been shown to fail is not evidence.

Why Python and not Rust: this is development tooling, not a product gate. The
nine `backend/ci/gates/*` crates are Rust because they gate the shipped system;
a Rust harness here would cost a Cargo member plus a generated BUCK file to buy
audit guarantees an orchestrator does not need. Python is already this repo's
second tooling language (`check-production-promotion-authority.py`,
`gen_first_party.py`).

Usage:
    python3 tools/lanes/fanout.py run     --spec lanes.json
    python3 tools/lanes/fanout.py challenge --question "..." [--file path]
    python3 tools/lanes/fanout.py verify  --probe cmd --bad-input cmd
"""

from __future__ import annotations

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass, field, asdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
LANES_ROOT = Path.home() / "Developer" / "console-lanes"

# --- The codex contract -----------------------------------------------------
# Every flag here was earned by a failed run. Do not trim without re-measuring.
#   features.hooks=false      -> bypasses the oh-my-codex layer for THIS call
#                               (its PreToolUse guard denied every tool call;
#                               its Stop hook sent the agent into session-pointer
#                               recovery after it had already answered)
#   developer_instructions='' -> strips the OMX orchestration prompt, which
#                               otherwise mandates `omx ralplan preflight` and
#                               stops before doing the actual task
#   stdin=DEVNULL             -> without it codex blocks on
#                               "Reading additional input from stdin..."
#   --json                    -> structured events; the answer arrives as an
#                               agent_message item, no prose scraping
#   capture to file, not pipe -> pipes are the non-TTY shape that hid output
CODEX_BASE_FLAGS = [
    "--skip-git-repo-check",
    "--json",
    "-c", "features.hooks=false",
    "-c", 'developer_instructions=""',
]


@dataclass
class LaneResult:
    lane: str
    ok: bool
    seconds: float
    changed_files: list[str] = field(default_factory=list)
    out_of_slice: list[str] = field(default_factory=list)
    answer: str = ""
    error: str = ""


def _build_env() -> dict:
    """Environment for a lane: shared compilation cache, private target dir.

    Lane worktrees each get their own `backend/target`, so they never contend on
    the cargo build lock — but they also share nothing, and every lane recompiles
    the entire dependency graph cold. Measured: `Cache hits: 0%` across 4,084
    commands, and sccache reported 0 requests EVER, because `lane-env.sh` was
    opt-in and nothing sourced it.

    sccache caches at the rustc-invocation level in a user-global directory, so
    lanes — and other repos on this machine — reuse each other's artifacts while
    keeping separate target dirs. Set here rather than in `.cargo/config.toml`
    deliberately: that file applies in CI too, and CI runners have no sccache, so
    every Rust job would fail.
    """
    env = dict(os.environ)
    if shutil.which("sccache"):
        env.setdefault("RUSTC_WRAPPER", "sccache")
        env.setdefault("SCCACHE_CACHE_SIZE", "50G")
    return env


def _codex(prompt: str, cwd: Path, sandbox: str, model: str, log: Path) -> tuple[bool, str, str]:
    """One codex call under the proven contract. Returns (ok, answer, error)."""
    cmd = ["codex", "exec", "--model", model, "--sandbox", sandbox,
           *CODEX_BASE_FLAGS, "-C", str(cwd), prompt]
    with log.open("wb") as fh:
        proc = subprocess.run(cmd, stdin=subprocess.DEVNULL, stdout=fh,
                              stderr=subprocess.PIPE, env=_build_env())
    answer, err = "", proc.stderr.decode("utf-8", "replace")[-2000:]
    for line in log.read_text("utf-8", "replace").splitlines():
        if not line.startswith("{"):
            continue
        try:
            evt = json.loads(line)
        except json.JSONDecodeError:
            continue
        item = evt.get("item", {})
        if evt.get("type") == "item.completed" and item.get("type") == "agent_message":
            answer = item.get("text", "")
    return proc.returncode == 0, answer, err


def _git(cwd: Path, *args: str) -> str:
    return subprocess.run(["git", "-C", str(cwd), *args],
                          capture_output=True, text=True).stdout.strip()


def _changed(worktree: Path) -> list[str]:
    out = _git(worktree, "status", "--porcelain")
    return [ln[3:].strip() for ln in out.splitlines() if ln.strip()]


def run_lanes(spec_path: Path, model: str, sandbox: str) -> int:
    """Fan out one task per lane; measure isolation instead of trusting it.

    Spec: [{"lane": "1", "prompt": "...", "allow": ["path/it/may/touch"]}, ...]
    """
    spec = json.loads(spec_path.read_text())
    logs = REPO / ".lane-logs"
    logs.mkdir(exist_ok=True)

    def one(item: dict) -> LaneResult:
        lane = str(item["lane"])
        wt = LANES_ROOT / f"lane-{lane}"
        if not wt.is_dir():
            return LaneResult(lane, False, 0.0, error=f"missing worktree {wt}")
        # A lane must start clean or its isolation measurement is meaningless.
        if _changed(wt):
            return LaneResult(lane, False, 0.0, error="worktree dirty before start")
        start = time.time()
        ok, answer, err = _codex(item["prompt"], wt, sandbox, model,
                                 logs / f"lane-{lane}.jsonl")
        elapsed = time.time() - start
        changed = _changed(wt)
        allow = set(item.get("allow", []))
        out_of_slice = [f for f in changed if f not in allow]
        return LaneResult(lane, ok and not out_of_slice, round(elapsed, 1),
                          changed, out_of_slice, answer[:4000], err if not ok else "")

    with ThreadPoolExecutor(max_workers=len(spec)) as pool:
        results = list(pool.map(one, spec))

    report = REPO / ".lane-logs" / "fanout-report.json"
    report.write_text(json.dumps([asdict(r) for r in results], indent=2))

    breaches = [r for r in results if r.out_of_slice]
    print(f"\n{'lane':<6}{'ok':<5}{'secs':>7}  changed / OUT-OF-SLICE")
    print("-" * 62)
    for r in sorted(results, key=lambda x: x.lane):
        mark = "yes" if r.ok else "NO"
        print(f"{r.lane:<6}{mark:<5}{r.seconds:>7}  {len(r.changed_files)} / "
              f"{len(r.out_of_slice)}{'  <-- BREACH' if r.out_of_slice else ''}")
        if r.error:
            print(f"       error: {r.error[:120]}")
    print(f"\nreport: {report}")
    if breaches:
        print(f"ISOLATION BREACH in {len(breaches)} lane(s) — fan-out is NOT safe.")
        for r in breaches:
            print(f"  lane-{r.lane}: {r.out_of_slice}")
        return 1
    print("Isolation held: every lane touched only its declared slice.")
    return 0


def challenge(question: str, context_file: str | None, model: str) -> int:
    """Cross-family adversarial challenge.

    The point is NOT redundancy — it is that blind spots correlate WITHIN a
    model family. Run this against your own conclusion before acting on it.
    Claude's side is run by the caller (Agent tool); this drives the codex side
    so both perspectives exist independently.
    """
    ctx = ""
    if context_file:
        p = Path(context_file)
        ctx = f"\n\nDocument under review ({p}):\n{p.read_text()[:12000]}"
    prompt = (
        "You are an ADVERSARIAL reviewer. Assume the following is WRONG and "
        "find the strongest concrete reasons why. Cite file:line. If after "
        "looking hard it is genuinely sound, say so plainly rather than "
        "manufacturing objections.\n\n"
        f"CLAIM UNDER ATTACK:\n{question}{ctx}"
    )
    log = REPO / ".lane-logs" / "challenge-codex.jsonl"
    log.parent.mkdir(exist_ok=True)
    ok, answer, err = _codex(prompt, REPO, "read-only", model, log)
    print(answer or f"(no answer)\n{err}")
    return 0 if ok else 1


def verify_probe(probe: str, bad_input: str) -> int:
    """Enforce: a probe must go RED on a known-bad input before its GREEN counts.

    Three probes broke this session while the code under test was correct —
    `.length` on an object, `root//pkg:name` vs `//pkg:name`, and zsh's
    1-indexed arrays. Each would have produced a confident wrong conclusion.
    """
    bad = subprocess.run(bad_input, shell=True, capture_output=True)
    if bad.returncode == 0:
        print("PROBE INVALID: it PASSED on a known-bad input, so its pass "
              "means nothing.\n"
              f"  bad-input command: {bad_input}")
        return 1
    print(f"probe proved RED on known-bad input (exit {bad.returncode}) — trustworthy")
    real = subprocess.run(probe, shell=True)
    print(f"probe on real input: exit {real.returncode}")
    return real.returncode


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    r = sub.add_parser("run", help="fan out lanes and measure isolation")
    r.add_argument("--spec", required=True, type=Path)
    r.add_argument("--model", default="gpt-5.6-sol")
    r.add_argument("--sandbox", default="workspace-write",
                   choices=["read-only", "workspace-write", "danger-full-access"])

    c = sub.add_parser("challenge", help="cross-family adversarial challenge")
    c.add_argument("--question", required=True)
    c.add_argument("--file", default=None)
    c.add_argument("--model", default="gpt-5.6-sol")

    v = sub.add_parser("verify", help="prove a probe goes RED before trusting GREEN")
    v.add_argument("--probe", required=True)
    v.add_argument("--bad-input", required=True)

    a = ap.parse_args()
    if a.cmd == "run":
        return run_lanes(a.spec, a.model, a.sandbox)
    if a.cmd == "challenge":
        return challenge(a.question, a.file, a.model)
    return verify_probe(a.probe, a.bad_input)


if __name__ == "__main__":
    sys.exit(main())
