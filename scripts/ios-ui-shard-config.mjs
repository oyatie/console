#!/usr/bin/env node
// Resolves an iOS UI shard's configuration out of .github/workflows/ios-ui-tests.yml.
//
// The local runner reads its fixture profile, content size and selectors from
// here rather than taking them as arguments, because a local run that silently
// disagrees with the hosted job is worse than no local run at all: it produces
// confident evidence about a configuration CI never executes. This program lost
// most of a day to exactly that — a shard that reproduced neither the hosted red
// nor the hosted green because the two runs differed in ways nobody had listed.
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const WORKFLOW = ".github/workflows/ios-ui-tests.yml";

/** The `case "$shard_name" in … esac` body that assigns per-shard settings. */
export function shardCaseBody(workflow) {
  const start = workflow.indexOf('case "$shard_name" in');
  if (start === -1) throw new Error("shard case block not found");
  const end = workflow.indexOf("esac", start);
  if (end === -1) throw new Error("shard case block is unterminated");
  return workflow.slice(start, end);
}

/** Defaults assigned before the case block; every shard inherits them. */
export function shardDefaults(workflow) {
  const before = workflow.slice(0, workflow.indexOf('case "$shard_name" in'));
  const last = (pattern) => {
    const matches = [...before.matchAll(pattern)];
    return matches.length ? matches[matches.length - 1][1] : undefined;
  };
  const profile = last(/SHARD_FIXTURE_PROFILE=([A-Za-z0-9-]+)/g);
  const contentSize = last(/SHARD_CONTENT_SIZE=([A-Za-z0-9-]+)/g);
  if (!profile || !contentSize) throw new Error("shard defaults not found");
  return { fixtureProfile: profile, contentSize };
}

export function shardConfig(name, workflow) {
  if (!/^[a-z0-9-]+$/.test(name)) throw new Error(`invalid shard name: ${name}`);
  const body = shardCaseBody(workflow);
  const arm = new RegExp(`\\n\\s*${name}\\)\\n([\\s\\S]*?)\\n\\s*;;`);
  const found = arm.exec(body);
  if (!found) throw new Error(`shard ${name} is not declared in ${WORKFLOW}`);
  const block = found[1];

  const scalar = (key) => {
    const hit = new RegExp(`${key}=([A-Za-z0-9-]+)`).exec(block);
    return hit ? hit[1] : undefined;
  };
  const selectorsBlock = /SHARD_SELECTORS=\(([\s\S]*?)\)/.exec(block);
  if (!selectorsBlock) throw new Error(`shard ${name} declares no selectors`);
  const selectors = selectorsBlock[1]
    .split(/\s+/)
    .map((entry) => entry.trim())
    .filter(Boolean);
  if (selectors.length === 0) throw new Error(`shard ${name} declares no selectors`);

  const timeout = scalar("SHARD_TIMEOUT_SECONDS");
  if (!timeout) throw new Error(`shard ${name} declares no timeout`);

  const defaults = shardDefaults(workflow);
  return {
    name,
    fixtureProfile: scalar("SHARD_FIXTURE_PROFILE") ?? defaults.fixtureProfile,
    contentSize: scalar("SHARD_CONTENT_SIZE") ?? defaults.contentSize,
    timeoutSeconds: Number(timeout),
    selectors,
    // Mirrors the hosted per-shard setup: only this shard meets an unanswered
    // system prompt, and only because the job resets the grant first.
    resetsCameraPrivacy: /privacy "\$UUID" reset camera/.test(workflow) && name === "camera-capture",
  };
}

export function main(argv, { root = REPO_ROOT, log = console.log } = {}) {
  const [name] = argv;
  if (!name) throw new Error("usage: ios-ui-shard-config.mjs <shard-name>");
  const config = shardConfig(name, readFileSync(resolve(root, WORKFLOW), "utf8"));
  // Shell-consumable: the runner evals this.
  log(`SHARD_FIXTURE_PROFILE=${config.fixtureProfile}`);
  log(`SHARD_CONTENT_SIZE=${config.contentSize}`);
  log(`SHARD_TIMEOUT_SECONDS=${config.timeoutSeconds}`);
  log(`SHARD_RESET_CAMERA_PRIVACY=${config.resetsCameraPrivacy ? 1 : 0}`);
  log(`SHARD_SELECTORS=(${config.selectors.join(" ")})`);
  return config;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`ios-ui-shard-config: ${error.message}`);
    process.exit(1);
  }
}
