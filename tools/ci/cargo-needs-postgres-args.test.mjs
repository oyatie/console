#!/usr/bin/env node
/**
 * Argument-contract tests for tools/ci/cargo_needs_postgres.sh.
 *
 * Failure paths and mocked credential transport are exercised. A harness that silently
 * accepts an unknown runner would run the cargo path while the workflow
 * believed it had asked for nextest, and the two select targets differently.
 */
import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { dirname, resolve, join } from "node:path";
import { fileURLToPath } from "node:url";

import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, readdirSync, unlinkSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";

const harness = resolve(dirname(fileURLToPath(import.meta.url)), "cargo_needs_postgres.sh");

const run = (...args) => spawnSync(harness, args, { encoding: "utf8" });

test("cargo_needs_postgres argument contract", async (t) => {
  await t.test("rejects an unknown runner before touching Docker", () => {
    const r = run("--runner", "bogus", "--shard-id", "domain-b");
    assert.equal(r.status, 2);
    assert.match(r.stderr, /invalid --runner bogus \(want cargo\|nextest\)/);
  });

  await t.test("rejects an empty runner rather than defaulting silently", () => {
    const r = run("--runner", "", "--shard-id", "domain-b");
    assert.equal(r.status, 2, "an empty runner must not fall through to the cargo path");
  });

  await t.test("accepts both supported runners at the validation stage", () => {
    // A valid runner must get PAST runner validation. It then fails on Docker
    // or the map, never with the runner message.
    for (const runner of ["cargo", "nextest"]) {
      const r = run("--runner", runner, "--shard-id", "nope");
      assert.equal(r.status, 2);
      assert.match(r.stderr, /invalid --shard-id/, `${runner} must pass runner validation`);
      assert.doesNotMatch(r.stderr, /invalid --runner/);
    }
  });

  await t.test("supports both --runner X and --runner=X spellings", () => {
    const spaced = run("--runner", "bogus", "--shard-id", "domain-b");
    const equals = run("--runner=bogus", "--shard-id", "domain-b");
    assert.equal(spaced.status, 2);
    assert.equal(equals.status, 2);
    assert.match(equals.stderr, /invalid --runner bogus/);
  });

  await t.test("still rejects the retired domain shard id", () => {
    const r = run("--shard-id", "domain");
    assert.equal(r.status, 2);
    assert.match(r.stderr, /retired in S2/);
  });

  await t.test("usage documents the runner flag", () => {
    const r = run("--help");
    assert.equal(r.status, 2);
    assert.match(r.stderr, /--runner cargo\|nextest/);
  });
});

test("Cargo PostgreSQL harness transports distinct restricted credentials privately", () => {
  // Exercise the real harness and runner; fake only database/build boundaries.
  // This checks transport, not topology correctness or database authorization.
  const scratch = mkdtempSync(join(tmpdir(), "console-cargo-transport-"));
  const write = (path, text, executable = false) => {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, text, { mode: executable ? 0o700 : 0o600 });
  };
  try {
    const ci = join(scratch, "tools/ci");
    const source = dirname(harness);
    for (const name of ["cargo_needs_postgres.sh", "cargo-test-runner.sh", "postgres-partition.mjs", "nextest-filterset.mjs"]) {
      write(join(ci, name), readFileSync(join(source, name), "utf8"), true);
    }
    write(join(scratch, "backend/ci/gates/writer-ownership/canonical-enforce.sh"), "#!/bin/sh\nexit 0\n", true);
    write(join(scratch, "ops/postgres-reconcile-topology.sh"), "# transport fixture\n");
    write(join(ci, "postgres-cargo-map.json"), readFileSync(join(source, "postgres-cargo-map.json"), "utf8"));
    const bin = join(scratch, "bin");
    write(join(bin, "docker"), `#!/usr/bin/env python3
import os, sys, pathlib, stat
args = sys.argv[1:]
root = pathlib.Path(os.environ["TRANSPORT_TEST_ROOT"])
with (root / "argv.log").open("a") as f: f.write(repr(args) + "\\n")
if args[0] == "cp" and args[2].endswith(":/topology.env"):
    p = pathlib.Path(args[1])
    assert stat.S_IMODE(p.stat().st_mode) == 0o600
    env = dict(line.split("=", 1) for line in p.read_text().splitlines())
    assert env["CONSOLE_AUTH_POSTGRES_PASSWORD"]
    keys = ["CONSOLE_APP_POSTGRES_PASSWORD", "CONSOLE_AUTH_POSTGRES_PASSWORD", "CONSOLE_RT_POSTGRES_PASSWORD", "CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD", "CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD", "CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD", "POSTGRES_ADMIN_PASSWORD"]
    assert len({env[k] for k in keys}) == len(keys)
    (root / "topology-path").write_text(str(p))
elif args[0] == "port": print("127.0.0.1:49123")
elif args[0] == "exec" and args[-1] == "/proc/1/comm": print("postgres")
`, true);
    write(join(bin, "cargo"), `#!/usr/bin/env python3
import os, sys, pathlib, urllib.parse, stat
root = pathlib.Path(os.environ["TRANSPORT_TEST_ROOT"])
with (root / "argv.log").open("a") as f: f.write(repr(sys.argv[1:]) + "\\n")
roles = {"DATABASE_URL": "console_buck_admin", "CONSOLE_APALIS_OWNER_DATABASE_URL": "console_app", "CONSOLE_APALIS_RUNTIME_DATABASE_URL": "console_rt", "CONSOLE_TEST_AUTH_DATABASE_URL": "console_auth_rt", "CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL": "console_leave_cmd", "CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL": "console_ontology_cmd", "CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL": "console_platform_force_cmd"}
urls = [urllib.parse.urlparse(os.environ[k]) for k in roles]
assert all(url.username == role for url, role in zip(urls, roles.values()))
assert len({url.password for url in urls}) == len(urls)
assert len({(url.hostname, url.port, url.path) for url in urls}) == 1
assert urls[0].hostname == "127.0.0.1" and urls[0].port == 49123
assert os.environ["CONSOLE_APALIS_ADMIN_DATABASE_URL"] == os.environ["DATABASE_URL"]
files = list(root.glob("console-cargo-postgres-env.*"))
assert len(files) == 1 and stat.S_IMODE(files[0].stat().st_mode) == 0o600
with (root / "cargo-phases").open("a") as f: f.write(("build" if "--no-run" in sys.argv else "test") + "\\n")
`, true);
    for (const runner of ["cargo", "nextest"]) {
      const r = spawnSync(join(ci, "cargo_needs_postgres.sh"), ["--workflow-only", "--only", "app-auth-rest-pg", "--runner", runner], {
        encoding: "utf8",
        env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, TMPDIR: scratch, TRANSPORT_TEST_ROOT: scratch },
      });
      assert.equal(r.status, 0, `${runner}: ${r.stdout}\n${r.stderr}`);
      assert.equal(readFileSync(join(scratch, "cargo-phases"), "utf8"), "build\ntest\n");
      const calls = readFileSync(join(scratch, "argv.log"), "utf8");
      assert.doesNotMatch(calls + r.stdout + r.stderr, /postgres:\/\/|POSTGRES_PASSWORD=/);
      assert.ok(calls.includes("'rm', '-f'"));
      assert.ok(!existsSync(readFileSync(join(scratch, "topology-path"), "utf8")));
      assert.equal(readdirSync(scratch).filter(name => /^console-cargo-postgres-(env|container)\./.test(name)).length, 0);
      unlinkSync(join(scratch, "cargo-phases"));
    }
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
});
