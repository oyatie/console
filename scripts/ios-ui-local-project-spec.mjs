#!/usr/bin/env node
// Writes a local XcodeGen spec that points MaintenanceFieldApp at a CI-shaped
// Info.plist, mirroring what the hosted job does inline.
//
// The production Info.plist must stay free of App Transport Security exceptions
// — a gate enforces that — so local-network access for the simulator comes from
// a copy, never from editing the shipped file.
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const NEEDLE = "INFOPLIST_FILE: Sources/MaintenanceFieldApp/Info.plist";

export function specWithPlist(source, plistPath) {
  const occurrences = source.split(NEEDLE).length - 1;
  if (occurrences !== 1) {
    throw new Error(`expected exactly one MaintenanceFieldApp INFOPLIST_FILE setting, found ${occurrences}`);
  }
  return source.replace(NEEDLE, `INFOPLIST_FILE: ${plistPath}`);
}

export function main(argv, { root } = {}) {
  const [plistPath, outPath] = argv;
  if (!plistPath || !outPath) {
    throw new Error("usage: ios-ui-local-project-spec.mjs <ci-plist> <out-spec>");
  }
  const base = root ?? resolve(dirname(fileURLToPath(import.meta.url)), "..", "ios");
  const source = readFileSync(resolve(base, "project.yml"), "utf8");
  writeFileSync(outPath, specWithPlist(source, plistPath));
  return outPath;
}

if (process.argv[1] && resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url))) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(`ios-ui-local-project-spec: ${error.message}`);
    process.exit(1);
  }
}
