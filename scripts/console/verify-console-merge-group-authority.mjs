#!/usr/bin/env node
/**
 * Merge-queue half of `authenticate-console-authority`.
 *
 * The PR-time job authenticates a pull request's exact coordinates from
 * protected code. A merge group is not a pull request -- it is the queue's
 * candidate merge commit -- so there are no PR coordinates to authenticate and
 * re-running that check against it would assert nothing.
 *
 * What the queue actually depends on is narrower and checkable: every pull
 * request in the group must ALREADY have passed authentication, because the
 * check is required and a PR cannot be queued otherwise. This verifies exactly
 * that, and fails closed if it cannot.
 *
 * Security posture matches the PR-time job: protected code only, read-only
 * metadata, no PR bytes executed, no write or OIDC scope.
 *
 * BATCH SIZE 1 IS ASSUMED, and asserted rather than hoped for. GitHub names a
 * merge-group branch `gh-readonly-queue/<base>/pr-<number>-<sha>`, which
 * identifies one pull request. With batching enabled the group can contain
 * several PRs whose numbers are not all recoverable from the ref, so this
 * refuses to guess: raise `max_entries_to_build` only together with a change
 * here that enumerates the group.
 */
import { spawnSync } from "node:child_process";

const fail = (message) => {
  console.error(`console merge-group authority: ${message}`);
  process.exitCode = 1;
  return false;
};

/** @returns {number|null} the single PR number a merge-group ref identifies. */
export function pullRequestNumberFromHeadRef(headRef) {
  if (typeof headRef !== "string" || !headRef) return null;
  // refs/heads/gh-readonly-queue/main/pr-123-<40-hex>
  const match = /(?:^|\/)gh-readonly-queue\/[^/]+\/pr-(\d+)-[0-9a-f]{40}$/.exec(headRef.trim());
  if (!match) return null;
  const number = Number(match[1]);
  return Number.isSafeInteger(number) && number > 0 ? number : null;
}

/** @returns {{ok: boolean, reason: string}} */
export function evaluateCheckRuns(runs, checkName) {
  if (!Array.isArray(runs)) return { ok: false, reason: "unreadable check-run list" };
  const named = runs.filter((run) => run && run.name === checkName);
  if (named.length === 0) return { ok: false, reason: `no ${checkName} run on the queued head` };
  const conclusions = new Set(named.map((run) => run.conclusion));
  if (conclusions.size !== 1 || !conclusions.has("success")) {
    return {
      ok: false,
      reason: `${checkName} did not conclude success on the queued head (${[...conclusions].join(", ")})`,
    };
  }
  return { ok: true, reason: "ok" };
}

function parseArguments(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i];
    const value = argv[i + 1];
    if (!key?.startsWith("--") || value === undefined) return null;
    out[key.slice(2)] = value;
  }
  return out;
}

const isMain = process.argv[1]?.endsWith("verify-console-merge-group-authority.mjs");
if (isMain) {
  const args = parseArguments(process.argv.slice(2));
  const headRef = args?.["head-ref"];
  const repository = args?.repository;
  if (!args || !headRef || !repository) {
    fail("usage: --head-ref <ref> --repository <owner/repo>");
  } else {
    const number = pullRequestNumberFromHeadRef(headRef);
    if (number === null) {
      // Includes the batched case: a ref this does not recognise is refused
      // rather than waved through.
      fail(`cannot identify a single queued pull request from head ref ${JSON.stringify(headRef)}`);
    } else {
      const pr = spawnSync("gh", [
        "api", `repos/${repository}/pulls/${number}`, "--jq", ".head.sha",
      ], { encoding: "utf8" });
      const headSha = pr.status === 0 ? pr.stdout.trim() : "";
      if (!/^[0-9a-f]{40}$/.test(headSha)) {
        fail(`cannot read the head SHA of queued pull request #${number}`);
      } else {
        const checks = spawnSync("gh", [
          "api", `repos/${repository}/commits/${headSha}/check-runs`,
          "--jq", "[.check_runs[] | {name, conclusion}]",
        ], { encoding: "utf8" });
        let runs = null;
        try {
          runs = checks.status === 0 ? JSON.parse(checks.stdout) : null;
        } catch {
          runs = null;
        }
        const verdict = evaluateCheckRuns(runs, "authenticate-console-authority");
        if (!verdict.ok) {
          fail(`pull request #${number} at ${headSha}: ${verdict.reason}`);
        } else {
          console.log(
            `console merge-group authority OK (pull request #${number} at ${headSha} `
            + "passed authenticate-console-authority before entering the queue)",
          );
        }
      }
    }
  }
}
