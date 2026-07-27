import assert from "node:assert/strict";
import test from "node:test";

import type { operations } from "../src/schema.js";

type AcceptResponses = operations["acceptComplianceEvidenceBinding"]["responses"];
type Forbidden = AcceptResponses[403]["content"]["application/json"];
type Conflict = AcceptResponses[409]["content"]["application/json"];
type Unavailable = AcceptResponses[503]["content"]["application/json"];

const forbidden = { error: { code: "forbidden", message: "compliance management is required" } } satisfies Forbidden;
const conflict = { error: { code: "conflict", message: "only a proposed evidence binding may be accepted" } } satisfies Conflict;
const unavailable = { error: { code: "service_unavailable", message: "JWT verification is not configured" } } satisfies Unavailable;

test("generated evidence acceptance operation preserves typed truth responses", () => {
  assert.equal(forbidden.error.code, "forbidden");
  assert.equal(conflict.error.code, "conflict");
  assert.equal(unavailable.error.code, "service_unavailable");
});
