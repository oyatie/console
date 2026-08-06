#!/usr/bin/env node
/**
 * ops.tip-serial-contention — report open PRs that share tip/baseline writers.
 * Usage: node tools/ci/assess-tip-contention.mjs
 * Requires: gh CLI authenticated.
 */
import { spawnSync } from "node:child_process";

const TIP_PATHS = [
  "docs/documentation-manifest.seed.json",
  "docs/documentation-index.json",
  "docs/program/executed-tests-baseline.json",
];

function ghJson(args) {
  const r = spawnSync("gh", args, { encoding: "utf8" });
  if (r.status !== 0) {
    console.error(r.stderr || r.stdout);
    process.exit(r.status ?? 1);
  }
  return JSON.parse(r.stdout);
}

const prs = ghJson([
  "pr", "list", "--state", "open", "--limit", "30",
  "--json", "number,title,headRefName,mergeStateStatus,url",
]);

const tipWriters = [];
for (const pr of prs) {
  const r = spawnSync("gh", ["pr", "diff", String(pr.number), "--name-only"], { encoding: "utf8" });
  if (r.status !== 0) continue;
  const files = (r.stdout || "").split("\n").filter(Boolean);
  const hits = files.filter(
    (f) =>
      TIP_PATHS.includes(f) ||
      f.startsWith("docs/program/ledger/") ||
      f.startsWith(".grok/"),
  );
  if (hits.length) {
    tipWriters.push({
      number: pr.number,
      title: pr.title,
      ms: pr.mergeStateStatus,
      hits,
      url: pr.url,
    });
  }
}

console.log(JSON.stringify({ tip_writers: tipWriters.length, prs: tipWriters }, null, 2));
if (tipWriters.length >= 2) {
  console.error(
    `ops.tip-serial-contention: ${tipWriters.length} open tip-writing PRs — merge queue serial; do not open another tip PR`,
  );
  process.exit(2);
}
console.error("tip contention OK (fewer than 2 tip writers open)");
