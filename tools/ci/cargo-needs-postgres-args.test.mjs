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
import os, sys, pathlib, stat, hashlib
args = sys.argv[1:]
root = pathlib.Path(os.environ["TRANSPORT_TEST_ROOT"])
with (root / "argv.log").open("a") as f: f.write(repr(args) + "\\n")
if args[0] == "cp" and args[2].endswith(":/topology.env"):
    p = pathlib.Path(args[1])
    assert stat.S_IMODE(p.stat().st_mode) == 0o600
    env = dict(line.split("=", 1) for line in p.read_text().splitlines())
    assert env["CONSOLE_AUTH_POSTGRES_PASSWORD"]
    keys = ["CONSOLE_APP_POSTGRES_PASSWORD", "CONSOLE_AUTH_POSTGRES_PASSWORD", "CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD", "CONSOLE_RT_POSTGRES_PASSWORD", "CONSOLE_LEAVE_COMMAND_POSTGRES_PASSWORD", "CONSOLE_ONTOLOGY_COMMAND_POSTGRES_PASSWORD", "CONSOLE_PLATFORM_FORCE_COMMAND_POSTGRES_PASSWORD", "POSTGRES_ADMIN_PASSWORD"]
    assert len({env[k] for k in keys}) == len(keys)
    (root / "topology-path").write_text(str(p))
    (root / "startup-digest").write_text(hashlib.sha256(env["CONSOLE_STARTUP_AUTH_POSTGRES_PASSWORD"].encode()).hexdigest())
elif args[0] == "port": print("127.0.0.1:49123")
elif args[0] == "exec" and args[-1] == "/proc/1/comm": print("postgres")
`, true);
    write(join(bin, "cargo"), `#!/usr/bin/env python3
import os, sys, pathlib, urllib.parse, stat, hashlib
root = pathlib.Path(os.environ["TRANSPORT_TEST_ROOT"])
with (root / "argv.log").open("a") as f: f.write(repr(sys.argv[1:]) + "\\n")
roles = {"DATABASE_URL": "console_buck_admin", "CONSOLE_APALIS_OWNER_DATABASE_URL": "console_app", "CONSOLE_APALIS_RUNTIME_DATABASE_URL": "console_rt", "CONSOLE_TEST_AUTH_DATABASE_URL": "console_auth_rt", "CONSOLE_STARTUP_AUTH_DATABASE_URL": "console_auth_startup", "CONSOLE_TEST_LEAVE_COMMAND_DATABASE_URL": "console_leave_cmd", "CONSOLE_TEST_ONTOLOGY_COMMAND_DATABASE_URL": "console_ontology_cmd", "CONSOLE_TEST_PLATFORM_FORCE_COMMAND_DATABASE_URL": "console_platform_force_cmd"}
urls = [urllib.parse.urlparse(os.environ[k]) for k in roles]
assert all(url.username == role for url, role in zip(urls, roles.values()))
assert len({url.password for url in urls}) == len(urls)
startup = urllib.parse.urlparse(os.environ["CONSOLE_STARTUP_AUTH_DATABASE_URL"])
assert startup.password and not startup.query and not startup.fragment
assert hashlib.sha256(startup.password.encode()).hexdigest() == (root / "startup-digest").read_text()
assert all(url.password not in (root / "argv.log").read_text() for url in urls)
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
        env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, TMPDIR: scratch, TRANSPORT_TEST_ROOT: scratch, CONSOLE_STARTUP_AUTH_DATABASE_URL: "postgres://wrong:inherited@wrong:1/wrong" },
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

// Preliminary review draft; append to cargo-needs-postgres-args.test.mjs.
const isolationIds = [
  "console-app::auth_rest",
  "console-leave-adapter-postgres::leave_migration_expand_contract",
  "console-ontology-adapter-postgres::key_revision_migration_upgrade",
  "console-platform-db::attendance_console_migration_contract",
  "console-platform-db::group_of_one_expand_contract",
  "console-platform-jobs::apalis_adapter",
  "console-platform-jobs::apalis_schema_contract",
];
const isolationId = (row) => {
  const args = row.cargo_argv;
  const at = args.indexOf("--test");
  return at >= 0 ? `${row.package}::${args[at + 1]}` : row.package;
};

for (const runner of ["cargo", "nextest"]) {
  for (const mode of ["success", "keep-going", "fail-fast", "build-failure", "preflight-failure", "one-alias-group", "terminal-failure", "shard"]) {
    test(`${runner} isolates canonical clusters and preserves selection (${mode})`, () => {
      const scratch = mkdtempSync(join(tmpdir(), "console-isolation-transport-"));
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
        write(join(scratch, "backend/ci/gates/writer-ownership/canonical-enforce.sh"), `#!/usr/bin/env bash
python3 - <<'PY'
import os, pathlib, json, sys
root=pathlib.Path(os.environ["ISOLATION_TEST_ROOT"])
clusters=[json.loads(line) for line in (root/"events.jsonl").read_text().splitlines() if json.loads(line)["kind"]=="cluster"]
if os.environ["ISOLATION_FAILURE_PHASE"]=="preflight" and len(clusters)>1 and not (root/"failure-injected").exists():
    (root/"failure-injected").write_text("preflight")
    with (root/"events.jsonl").open("a") as f: f.write(json.dumps({"kind":"injected","phase":"preflight","cluster":clusters[-1]["cluster"]})+"\\n")
    sys.exit(23)
PY
`, true);
        write(join(scratch, "ops/postgres-reconcile-topology.sh"), "# transport fixture\n");
        write(join(scratch, ".config/nextest.toml"), readFileSync(join(source, "../../.config/nextest.toml"), "utf8"));
        const map = JSON.parse(readFileSync(join(source, "postgres-cargo-map.json"), "utf8"));
        const hazardous = map.entries.filter(row => isolationIds.includes(isolationId(row)));
        assert.equal(hazardous.length, 11, "all aliases in the reviewed map remain covered");
        assert.deepEqual([...new Set(hazardous.map(isolationId))].sort(), [...isolationIds].sort());
        const ordinary = map.entries.filter(row => ["app-audit-api-pg", "app-account-migration-pg"].includes(row.name));
        assert.equal(ordinary.length, 2);
        // Unique custom-map names prove recursive children preserve --map.
        for (const row of [...hazardous, ...ordinary]) {
          row.name += "-custom-selection";
          row.in_workflow_postgres_job = true;
        }
        const customMap = join(scratch, "selected-map.json");
        write(customMap, JSON.stringify(map));
        write(join(ci, "postgres-cargo-map.json"), readFileSync(join(source, "postgres-cargo-map.json"), "utf8"));
        const terminal = ["one-alias-group", "terminal-failure"].includes(mode);
        const selected = terminal
          ? hazardous.filter(row => isolationId(row) === "console-platform-jobs::apalis_adapter")
          : mode === "shard" ? [...hazardous, ...ordinary].filter(row => row.package === "console-app")
          : [...hazardous, ...ordinary];
        let shard = "";
        if (mode === "shard") {
          for (const candidate of ["app", "platform", "ontology", "domain-a", "domain-b"]) {
            const partition = spawnSync(process.execPath, [join(ci, "postgres-partition.mjs"), `--emit-shard=${candidate}`, customMap], { encoding: "utf8" });
            assert.equal(partition.status, 0, partition.stderr);
            if (partition.stdout.trim().split("\n").filter(Boolean).map(JSON.parse).some(row => row.name === selected[0].name)) shard = candidate;
          }
          assert.ok(shard, "actual partition assigns selected package to one shard");
        }
        const expectedGroups = terminal ? 1 : mode === "shard" ? 2 : 8;
        const expectedIds = [...new Set(selected.map(isolationId))].sort();
        const bin = join(scratch, "bin");
        write(join(bin, "docker"), `#!/usr/bin/env python3
import os, sys, pathlib, json, hashlib, stat
root=pathlib.Path(os.environ["ISOLATION_TEST_ROOT"]); args=sys.argv[1:]
def emit(v):
    with (root/"events.jsonl").open("a") as f: f.write(json.dumps(v)+"\\n")
if args[0]=="run":
    name=args[args.index("--name")+1]; p=pathlib.Path(args[args.index("--env-file")+1])
    assert stat.S_IMODE(p.stat().st_mode)==0o600
    env=dict(line.split("=",1) for line in p.read_text().splitlines())
    # Persist only hashes of random fixture credentials, never their values.
    digest=hashlib.sha256(env["CONSOLE_AUTH_POSTGRES_PASSWORD"].encode()).hexdigest()
    emit({"kind":"cluster","cluster":env["POSTGRES_DB"],"name":name,"credential_digest":digest})
elif args[0]=="port": print("127.0.0.1:49123")
elif args[0]=="exec" and args[-1]=="/proc/1/comm": print("postgres")
elif args[0]=="rm": emit({"kind":"removed","name":args[-1]})
`, true);
        write(join(bin, "cargo"), `#!/usr/bin/env python3
import os, sys, pathlib, urllib.parse, json, re
root=pathlib.Path(os.environ["ISOLATION_TEST_ROOT"]); args=sys.argv[1:]
url=urllib.parse.urlparse(os.environ["DATABASE_URL"])
build="--no-run" in args
if "nextest" in args:
    ids=re.findall(r"binary_id\\(=([A-Za-z0-9_.:-]+)\\)", args[args.index("-E")+1])
else:
    pkg=args[args.index("-p")+1]
    ids=[pkg+"::"+args[args.index("--test")+1]] if "--test" in args else [pkg]
with (root/"events.jsonl").open("a") as f:
    f.write(json.dumps({"kind":"build" if build else "test","cluster":url.path.lstrip("/"),"ids":ids,"args":args,"threads":os.environ.get("RUST_TEST_THREADS"),"shard":os.environ.get("CARGO_POSTGRES_SHARD_ID","")})+"\\n")
phase="build" if build else "test"
if os.environ["ISOLATION_FAILURE_PHASE"]==phase and not (root/"failure-injected").exists():
    (root/"failure-injected").write_text(phase)
    with (root/"events.jsonl").open("a") as f: f.write(json.dumps({"kind":"injected","phase":phase,"cluster":url.path.lstrip("/")})+"\\n")
    sys.exit(19)
`, true);
        const failing = ["keep-going", "fail-fast", "build-failure", "preflight-failure", "terminal-failure"].includes(mode);
        const failurePhase = mode === "build-failure" ? "build" : mode === "preflight-failure" ? "preflight" : failing ? "test" : "none";
        const result = spawnSync(join(ci, "cargo_needs_postgres.sh"), [
          "--map", customMap, "--workflow-only", "--only", selected.map(row => row.name).join(","),
          "--runner", runner, "--num-threads", "7", ...(shard ? ["--shard-id", shard] : []), ...(mode === "fail-fast" ? ["--fail-fast"] : []),
        ], {
          encoding: "utf8", timeout: 60000,
          env: { ...process.env, PATH: `${bin}:${process.env.PATH}`, TMPDIR: scratch,
            ISOLATION_TEST_ROOT: scratch, ISOLATION_FAILURE_PHASE: failurePhase },
        });
        assert.equal(result.error, undefined, "recursion/execution must terminate");
        if (failing) assert.ok(Number.isInteger(result.status) && result.status !== 0, `${result.stdout}\n${result.stderr}`);
        else assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
        const events = readFileSync(join(scratch, "events.jsonl"), "utf8").trim().split("\n").map(JSON.parse);
        const clusters = events.filter(e => e.kind === "cluster");
        const runs = events.filter(e => e.kind === "test");
        const injected = events.filter(e => e.kind === "injected");
        assert.equal(injected.length, failing ? 1 : 0, "requested child failure must actually occur");
        if (failing) assert.equal(injected[0].phase, failurePhase);
        if (shard) {
          const announcements = [...result.stdout.matchAll(/cargo test invocations \(shard=([a-z-]+)/g)];
          assert.ok(announcements.length >= expectedGroups, "both terminal groups report the selected shard");
          assert.ok(announcements.every(match => match[1] === shard));
        }
        assert.ok(runs.length > 0, "actual harness must reach the runner");
        assert.equal(new Set(clusters.map(e => e.name)).size, clusters.length);
        assert.equal(new Set(clusters.map(e => e.credential_digest)).size, clusters.length);
        assert.deepEqual(new Set(events.filter(e => e.kind === "removed").map(e => e.name)), new Set(clusters.map(e => e.name)));
        assert.equal(readdirSync(scratch).filter(name => /^console-cargo-/.test(name)).length, 0);
        assert.doesNotMatch(result.stdout + result.stderr, /postgres:\/\/|POSTGRES_PASSWORD=/);
        const groups = new Map();
        for (const run of runs) {
          assert.ok(clusters.some(c => c.cluster === run.cluster));
          if (!groups.has(run.cluster)) groups.set(run.cluster, new Set());
          for (const id of run.ids) groups.get(run.cluster).add(id);
          const hazardousIds = run.ids.filter(id => isolationIds.includes(id));
          if (hazardousIds.length) {
            assert.equal(new Set(hazardousIds).size, 1);
            assert.equal(run.ids.length, 1, "no consumer shares hazardous process cluster");
            assert.equal(run.threads, "1");
            if (runner === "cargo") assert.ok(run.args.includes("--test-threads=1"));
            else {
              assert.equal(run.args[run.args.indexOf("--test-threads") + 1], "1");
              assert.ok(run.args.includes("--config-file") && run.args.includes("ci"));
            }
          } else {
            assert.equal(run.threads, "7", "ordinary cohort keeps requested concurrency");
          }
          if (runner === "nextest" && mode !== "fail-fast") assert.ok(run.args.includes("--no-fail-fast"));
        }
        for (const ids of groups.values()) {
          if ([...ids].some(id => isolationIds.includes(id))) assert.equal(ids.size, 1);
        }
        if (mode === "fail-fast") {
          assert.equal(runs.length, 1, "failure stops additional row/group execution only when requested");
        } else if (["build-failure", "preflight-failure"].includes(mode)) {
          assert.equal(groups.size, expectedGroups - 1, "only failing child can omit execution; every later group runs");
          assert.ok(!groups.has(injected[0].cluster));
          const reached = new Set(runs.flatMap(e => e.ids));
          assert.ok([...reached].every(id => expectedIds.includes(id)));
          const missing = expectedIds.filter(id => !reached.has(id));
          assert.ok((missing.length === 1 && isolationIds.includes(missing[0]))
            || (missing.length === 2 && missing.every(id => ordinary.map(isolationId).includes(id))),
            "exactly one canonical hazardous group or the ordinary cohort failed pre-execution");
          assert.equal(clusters.length, expectedGroups + 1, "all groups receive a fresh child even after failure");
        } else {
          assert.deepEqual([...new Set(runs.flatMap(e => e.ids))].sort(), expectedIds);
          assert.equal(groups.size, expectedGroups);
          if (runner === "cargo") {
            assert.equal(runs.length, selected.length, "Cargo preserves every selected alias invocation");
            for (const row of selected) assert.ok(result.stdout.includes(`=== ${row.name} ===`));
          } else {
            assert.equal(runs.length, expectedGroups);
            const counts = [...result.stdout.matchAll(/running (\d+) targets via cargo-nextest/g)].map(m => Number(m[1]));
            assert.equal(counts.reduce((a, b) => a + b, 0), selected.length,
              "terminal group reports preserve every alias row before canonical filter deduplication");
          }
        }
        if (terminal) assert.equal(clusters.length, 1, "terminal aliases cannot recurse");
      } finally {
        rmSync(scratch, { recursive: true, force: true });
      }
    });
  }
}
