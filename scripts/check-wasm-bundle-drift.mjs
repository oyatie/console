#!/usr/bin/env node
// The CLI and native Buck consumer share the same fail-closed verifier.
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const result = spawnSync("python3", [resolve(root, "tools/ui/wasm_bundle.py"), "check", ...process.argv.slice(2)], { stdio: "inherit" });
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
