import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const gate = "scripts/check-workflow-runtime-m2-cedar-guards.mjs";
const audit = "backend/crates/platform/db/src/audit_tx.rs";
const sources = [gate, audit, "package.json", ".github/workflows/ci.yml",
  "backend/crates/platform/authz/src/cedar_pbac.rs",
  "backend/crates/platform/authz/tests/cedar_pbac_legacy_only_observe_and_record.rs"];

test("audited transaction gate follows the owning helper and rejects broken effects", async (t) => {
  const fixture = mkdtempSync(join(tmpdir(), "console-audit-gate-"));
  try {
    for (const path of sources) {
      const destination = join(fixture, path);
      mkdirSync(dirname(destination), { recursive: true });
      writeFileSync(destination, readFileSync(join(root, path)));
    }
    const original = readFileSync(join(fixture, audit), "utf8");
    const run = () => spawnSync(process.execPath, [join(fixture, gate)], { encoding: "utf8" });
    await t.test("current entry and shared helper pass", () => {
      const result = run();
      assert.equal(result.status, 0, result.stderr);
    });
    // Mutate only the shared helper: an intact sibling function must not mask it.
    const start = original.indexOf("async fn with_audits_in_tx<");
    assert.ok(start > 0);
    for (const [name, before, after, diagnostic] of [
      ["tenant binding", "set_current_org(&mut tx, org).await.map_err(E::from)?;", "", "must arm app.current_org"],
      ["audit insertion", "insert_audit_event_tx(&mut tx, event)", "unrelated_insert(&mut tx, event)", "must insert audit events and commit"],
      ["commit", "tx.commit()", "tx.rollback()", "must insert audit events and commit"],
      ["rollback", "tx.rollback()", "drop(tx)", "must roll back"],
    ]) {
      await t.test(`rejects missing ${name}`, () => {
        const helper = original.slice(start);
        assert.ok(helper.includes(before));
        writeFileSync(join(fixture, audit), original.slice(0, start) + helper.replace(before, after));
        const result = run();
        assert.equal(result.status, 1, result.stdout);
        assert.ok(result.stderr.includes(diagnostic), result.stderr);
      });
    }
    await t.test("rejects disconnected entry even with an intact helper", () => {
      const call = "with_audits_in_tx(tx, org, f).await";
      assert.ok(original.includes(call));
      writeFileSync(join(fixture, audit), original.replace(call, `/* ${call} */ unimplemented!()`));
      const result = run();
      assert.equal(result.status, 1, result.stdout);
      assert.match(result.stderr, /must delegate its opened transaction/);
    });
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});
