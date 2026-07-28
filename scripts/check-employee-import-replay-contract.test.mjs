import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

function read(relativePath) {
  return readFileSync(new URL(`../${relativePath}`, import.meta.url), "utf8");
}

function between(text, start, end) {
  const startIndex = text.indexOf(start);
  const endIndex = text.indexOf(end, startIndex + start.length);
  assert.notEqual(startIndex, -1, `missing start marker: ${start}`);
  assert.notEqual(endIndex, -1, `missing end marker: ${end}`);
  return text.slice(startIndex, endIndex);
}

test("employee import replay accounting is required in the OpenAPI wire contract", () => {
  const openapi = read("backend/openapi/openapi.yaml");
  for (const schema of [
    between(openapi, "    EmployeeImportCompanySummary:\n", "    EmployeeImportColumn:\n"),
    between(openapi, "    EmployeeImportReport:\n", "    HrOrgChartEmployee:\n"),
  ]) {
    assert.match(schema, /required:\n(?:      - [^\n]+\n)*      - skipped\n/);
    assert.match(schema, /        skipped:\n          type: integer\n          minimum: 0\n/);
  }
});

test("hosted CI runs the employee import replay contract", () => {
  const workflow = read(".github/workflows/ci.yml");
  assert.match(workflow, /^\s*run:\s+npm run test:employee-import-contract\s*$/m);
});
