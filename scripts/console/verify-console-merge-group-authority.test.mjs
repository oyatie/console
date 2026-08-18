#!/usr/bin/env node
import test from "node:test";
import assert from "node:assert/strict";
import {
  evaluateCheckRuns,
  pullRequestNumberFromHeadRef,
} from "./verify-console-merge-group-authority.mjs";

const SHA = "a".repeat(40);

test("recognises a single-entry merge-group ref", () => {
  assert.equal(pullRequestNumberFromHeadRef(`refs/heads/gh-readonly-queue/main/pr-793-${SHA}`), 793);
  assert.equal(pullRequestNumberFromHeadRef(`gh-readonly-queue/main/pr-1-${SHA}`), 1);
});

test("refuses any ref it cannot resolve to exactly one pull request", () => {
  // The batched case matters most: raising max_entries_to_build without
  // teaching this script to enumerate the group must fail closed, not pass.
  for (const ref of [
    "",
    null,
    "refs/heads/main",
    "refs/heads/gh-readonly-queue/main/pr-793",              // no sha
    `refs/heads/gh-readonly-queue/main/pr-abc-${SHA}`,        // non-numeric
    `refs/heads/gh-readonly-queue/main/pr-0-${SHA}`,          // not a real PR number
    "refs/heads/gh-readonly-queue/main/batch-793-794",        // hypothetical batch shape
  ]) {
    assert.equal(pullRequestNumberFromHeadRef(ref), null, `must refuse ${JSON.stringify(ref)}`);
  }
});

test("accepts only a check that concluded success", () => {
  const name = "authenticate-console-authority";
  assert.equal(evaluateCheckRuns([{ name, conclusion: "success" }], name).ok, true);

  for (const conclusion of ["failure", "cancelled", "timed_out", "neutral", "skipped", null]) {
    const verdict = evaluateCheckRuns([{ name, conclusion }], name);
    assert.equal(verdict.ok, false, `must refuse conclusion ${conclusion}`);
    assert.match(verdict.reason, /did not conclude success/);
  }
});

test("a missing or unreadable check is refused, not treated as absent-therefore-fine", () => {
  const name = "authenticate-console-authority";
  assert.equal(evaluateCheckRuns([], name).ok, false);
  assert.match(evaluateCheckRuns([], name).reason, /no authenticate-console-authority run/);
  assert.equal(evaluateCheckRuns([{ name: "Required / CI", conclusion: "success" }], name).ok, false);
  assert.equal(evaluateCheckRuns(null, name).ok, false);
  assert.equal(evaluateCheckRuns(undefined, name).ok, false);
});

test("a re-run that produced one success and one failure is refused", () => {
  // Taking the newest run would let a green re-run paper over a red one; the
  // queue's guarantee is that the PR passed, not that it passed once.
  const name = "authenticate-console-authority";
  const verdict = evaluateCheckRuns(
    [{ name, conclusion: "success" }, { name, conclusion: "failure" }],
    name,
  );
  assert.equal(verdict.ok, false);
});
