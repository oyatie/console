import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { chmodSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, it } from "node:test";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const CRATE = "backend/crates/payroll/ui";
const MANIFEST = `${CRATE}/bundle.lock.json`;
const GATE = "scripts/check-wasm-bundle-drift.mjs";

/** A miniature repo carrying everything the gate reads, as a git repo.
 *
 * It must be a real repository because the gate asks `git ls-files` for the
 * source list rather than walking the directory -- CI only ever has tracked
 * files, and walking recorded whatever happened to be lying around (a Finder
 * visit put `.DS_Store` in the lock, and rebuilding regenerated it).
 */
function fixture() {
  const dir = mkdtempSync(join(tmpdir(), "wasm-drift-"));
  cpSync(join(ROOT, CRATE), join(dir, CRATE), { recursive: true });
  mkdirSync(join(dir, "tools/ui"), { recursive: true });
  cpSync(join(ROOT, "tools/ui/build-payroll-wasm.sh"), join(dir, "tools/ui/build-payroll-wasm.sh"));
  cpSync(join(ROOT, "backend/Cargo.toml"), join(dir, "backend/Cargo.toml"));
  cpSync(join(ROOT, "rust-toolchain.toml"), join(dir, "rust-toolchain.toml"));
  // Every native source/recipe/graph/tool input recorded by the real bundle
  // is copied from this exact candidate, not synthesized or resealed here.
  for (const rel of Object.keys(JSON.parse(readFileSync(join(ROOT, MANIFEST), "utf8")).inputs)) {
    mkdirSync(dirname(join(dir, rel)), { recursive: true });
    cpSync(join(ROOT, rel), join(dir, rel));
  }
  mkdirSync(join(dir, "tools/ui"), { recursive: true });
  cpSync(join(ROOT, "tools/ui/wasm_bundle.py"), join(dir, "tools/ui/wasm_bundle.py"));
  // The repo's own .gitignore, so the fixture models the real tree. Without it
  // `.DS_Store` here is untracked-and-NOT-ignored, and the strict write check
  // fires on the very noise it was written to stay silent about.
  cpSync(join(ROOT, ".gitignore"), join(dir, ".gitignore"));
  mkdirSync(join(dir, "scripts"), { recursive: true });
  cpSync(join(ROOT, GATE), join(dir, GATE));
  const git = (...args) => execFileSync("git", ["-C", dir, ...args], { stdio: "pipe" });
  git("init", "-q");
  git("add", "-A");
  return dir;
}

/** Exit code of the gate against a fixture. 0 = the bundle was accepted. */
function gate(dir) {
  try {
    execFileSync("node", [join(dir, GATE)], { stdio: "pipe" });
    return 0;
  } catch (error) {
    return error.status;
  }
}

/** Exercise the compatibility --write entrypoint with a version-only CLI.
 * Strict source-enumeration failures retain their specific diagnostics. A
 * source-clean call must still refuse: a version string is no evidence that
 * this executable produced the old artifacts from the current source inputs.
 */
function writeManifest(dir) {
  const shim = join(dir, "shim");
  mkdirSync(shim, { recursive: true });
  const bin = join(shim, "wasm-bindgen");
  writeFileSync(bin, "#!/bin/sh\necho 'wasm-bindgen 0.2.123'\n");
  chmodSync(bin, 0o755);
  const env = { ...process.env, PATH: `${shim}:${process.env.PATH ?? ""}` };
  try {
    execFileSync("node", [join(dir, GATE), "--write"], { stdio: "pipe", env });
    return { status: 0, stderr: "" };
  } catch (error) {
    return { status: error.status, stderr: String(error.stderr ?? "") };
  }
}

/** stderr of the gate, so a case can assert it failed for its OWN reason. */
function why(dir) {
  try {
    execFileSync("node", [join(dir, GATE)], { stdio: "pipe" });
    return "";
  } catch (error) {
    return String(error.stderr ?? "");
  }
}

const read = (dir, rel) => readFileSync(join(dir, rel), "utf8");
const write = (dir, rel, text) => writeFileSync(join(dir, rel), text);
const track = (dir) => execFileSync("git", ["-C", dir, "add", "-A"], { stdio: "pipe" });
const manifest = (dir, mutate) => {
  const m = JSON.parse(read(dir, MANIFEST));
  write(dir, MANIFEST, JSON.stringify(mutate(m) ?? m, null, 2));
};

describe("the committed hydration bundle cannot drift from its source", () => {
  // Each case names the check it is meant to exercise, and asserts on the
  // MESSAGE as well as the exit code. An earlier version asserted only
  // `exit != 0`, and two cases passed for reasons unrelated to their names:
  // the wasm-bindgen case was caught by the Cargo.toml input hash, and the
  // missing-manifest case was satisfied by an uncaught TypeError.
  const DRIFTS = {
    "the crate source is edited without rebundling": {
      drift: (d) => { write(d, `${CRATE}/src/lib.rs`, `${read(d, `${CRATE}/src/lib.rs`)}\n// edit\n`); track(d); },
      because: /src\/lib\.rs has changed/,
    },
    "a source file is added that the bundle was not built from": {
      drift: (d) => { write(d, `${CRATE}/src/new_island.rs`, "fn island() {}\n"); track(d); },
      because: /new_island\.rs is a source file the committed bundle was not built from/,
    },
    "a source file is deleted": {
      // The backstop for the skip-guard in the hash comparison: that loop only
      // compares files present in BOTH, so a deletion is invisible to it.
      drift: (d) => { rmSync(join(d, `${CRATE}/src/island_script.js`)); track(d); },
      because: /island_script\.js was built into the committed bundle but no longer exists/,
    },
    "the build recipe changes": {
      // The script is what turns sources into bytes -- `--release`, the feature
      // set, `--target web`. Its own header says "Do not commit debug wasm".
      drift: (d) => { write(d, "tools/ui/build-payroll-wasm.sh", `${read(d, "tools/ui/build-payroll-wasm.sh")}\n# tweak\n`); track(d); },
      because: /build-payroll-wasm\.sh has changed/,
    },
    "an artifact is edited by hand": {
      drift: (d) => { write(d, `${CRATE}/pkg/console_payroll_ui.js`, `${read(d, `${CRATE}/pkg/console_payroll_ui.js`)}\n// tweak\n`); track(d); },
      because: /console_payroll_ui\.js does not match the hash recorded/,
    },
    "the toolchain pin moves": {
      drift: (d) => { write(d, "rust-toolchain.toml", read(d, "rust-toolchain.toml").replace(/channel = "[^"]+"/, 'channel = "1.97.1"')); track(d); },
      because: /built with channel .* but rust-toolchain\.toml says/,
    },
    "the glue was emitted by a different wasm-bindgen CLI": {
      // Only the recorded CLI moves -- no input file is touched -- so this can
      // only pass if the CLI comparison itself fires.
      drift: (d) => manifest(d, (m) => { m.wasm_bindgen_cli = "0.2.99"; }),
      because: /emitted by wasm-bindgen CLI 0\.2\.99/,
    },
    "the manifest is missing": {
      drift: (d) => rmSync(join(d, MANIFEST)),
      because: /is missing or unparseable/,
    },
    "the manifest is truncated to a falsy scalar": {
      // `0`, `false` and `""` are valid JSON. A truthiness guard skipped every
      // check and printed success over a stale bundle.
      drift: (d) => write(d, MANIFEST, "0"),
      because: /is not a JSON object \(parsed as number\)/,
    },
    "the manifest is an array": {
      drift: (d) => write(d, MANIFEST, "[]"),
      because: /is not a JSON object \(parsed as an array\)/,
    },
    "the manifest records no wasm-bindgen version at all": {
      drift: (d) => manifest(d, (m) => { delete m.wasm_bindgen_cli; delete m.wasm_bindgen; }),
      because: /records no wasm-bindgen version/,
    },
    "only the recorded CLI version is dropped": {
      // Distinct from the case above, and the reason the `?? wasm_bindgen`
      // fallback had to go: with `wasm_bindgen` left correct, that fallback
      // degraded to the comparison which cannot fire -- Cargo.toml is already a
      // hashed input -- so deleting one field switched off the only
      // non-redundant check and the gate passed.
      drift: (d) => manifest(d, (m) => { delete m.wasm_bindgen_cli; }),
      because: /records no wasm-bindgen version/,
    },
  };

  for (const [name, { drift, because }] of Object.entries(DRIFTS)) {
    it(`rejects when ${name}`, () => {
      const dir = fixture();
      try {
        assert.equal(gate(dir), 0, "fixture should start clean");
        drift(dir);
        assert.notEqual(gate(dir), 0, `${name} passed the gate`);
        assert.match(why(dir), because, `${name} failed, but not for its own reason`);
      } finally {
        rmSync(dir, { recursive: true, force: true });
      }
    });
  }

  it("ignores an untracked file that git would not ship", () => {
    // The .DS_Store loop: a Finder visit must not become a build input, or CI
    // fails telling the developer to rebuild and rebuilding re-poisons the lock.
    const dir = fixture();
    try {
      write(dir, `${CRATE}/src/.DS_Store`, "\0\0junk");
      assert.equal(gate(dir), 0, "an untracked stray file must not be treated as a source input");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses to WRITE a manifest whose sources git cannot enumerate", () => {
    // The write-side twin of the fail-open above. With src/ untracked, an
    // unfloored `--write` records the recipe files and no sources at all, and
    // the check side then passes over every later edit to them, forever -- a
    // manifest born blind stays blind. The check side fails closed on a
    // shrinking set; only a floor covers the moment of writing.
    const dir = fixture();
    try {
      execFileSync("git", ["-C", dir, "rm", "-r", "--cached", "-q", `${CRATE}/src`], { stdio: "pipe" });
      const { status, stderr } = writeManifest(dir);
      assert.notEqual(status, 0, "--write accepted a source listing with no sources in it");
      assert.match(stderr, /was not among them/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses to WRITE while an untracked source sits under src/", () => {
    // The anchor alone only asks "is lib.rs tracked?", so it passes while a
    // compiled file is invisible. cargo builds what is on disk; the manifest
    // records what git lists; --write must refuse when those disagree.
    const dir = fixture();
    try {
      write(dir, `${CRATE}/src/table.rs`, "pub fn t() {}\n");
      const { status, stderr } = writeManifest(dir);
      assert.notEqual(status, 0, "--write recorded a manifest while a source was untracked");
      assert.match(stderr, /untracked file\(s\)/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses to WRITE while a gitignored source sits under src/", () => {
    // The nastier twin: invisible to `ls-files` on BOTH sides, so it would
    // never surface later either. An edit to it could never fail this gate.
    const dir = fixture();
    try {
      write(dir, `${CRATE}/src/generated.rs`, "pub fn g() {}\n");
      write(dir, ".gitignore", `${read(dir, ".gitignore")}\ngenerated.rs\n`);
      const { status, stderr } = writeManifest(dir);
      assert.notEqual(status, 0, "--write recorded a manifest while a compiled source was gitignored");
      assert.match(stderr, /gitignored source file\(s\)/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("refuses standalone resealing even with ignored NON-source noise", () => {
    // Preserve the ignored-noise positive control while retiring the old
    // standalone resealing behavior. A fake version CLI must not authorize a
    // new provenance record for artifacts it did not build.
    const dir = fixture();
    try {
      write(dir, `${CRATE}/src/.DS_Store`, "\0\0junk");
      const before = read(dir, MANIFEST);
      const { status, stderr } = writeManifest(dir);
      assert.notEqual(status, 0, "--write must not reseal old outputs without a native producer result");
      assert.match(stderr, /requires an action-bound native producer bundle/);
      assert.equal(read(dir, MANIFEST), before, "refused write must preserve the original manifest");
      assert.equal(gate(dir), 0, "ignored noise must still leave a valid existing bundle accepted");
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("cannot reseal old output after a source change", () => {
    const dir = fixture();
    try {
      const before = read(dir, MANIFEST);
      write(dir, `${CRATE}/src/lib.rs`, `${read(dir, `${CRATE}/src/lib.rs`)}\n// unbuilt edit\n`);
      track(dir);
      const { status, stderr } = writeManifest(dir);
      assert.notEqual(status, 0, "old output was resealed against changed sources");
      assert.match(stderr, /requires an action-bound native producer bundle/);
      assert.equal(read(dir, MANIFEST), before);
      assert.match(why(dir), /src\/lib\.rs has changed/);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("accepts the committed bundle as it stands", () => {
    // The control. Without it every case above is satisfied by a gate that
    // rejects everything. It uses the real crate, so if this fails in CI the
    // bundle genuinely needs rebuilding -- not a fixture problem.
    const dir = fixture();
    try {
      assert.equal(
        gate(dir),
        0,
        "the committed bundle does not match its source. This is not a test-fixture bug: "
          + "run `bash tools/ui/build-payroll-wasm.sh` and commit the result.",
      );
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
