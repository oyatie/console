import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  PATH_FLOOR,
  RESPONSE_FLOOR,
  evaluateErrorEnvelope,
  evaluateOpenapiErrorEnvelope,
  evaluateRuntimeErrorEnvelope,
} from "./check-openapi-error-envelope.mjs";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-openapi-error-envelope.mjs", import.meta.url));
const fixtureRoots = [];

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "openapi-error-envelope-"));
  fixtureRoots.push(root);
  for (const [relative, contents] of Object.entries(files)) {
    const absolute = join(root, relative);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, contents);
  }
  return root;
}

const ERROR_BODY = `      content:
        application/json:
          schema:
            $ref: '#/components/schemas/ErrorBody'`;

function spec({ responses, paths }) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
${paths}
components:
  responses:
${responses}
  schemas:
    ErrorBody:
      type: object
      required: [error]
      properties:
        error:
          type: object
`;
}

const COMPLETE_RESPONSES = `    RequestTimeout:
      description: Request exceeded the server timeout budget.
${ERROR_BODY}
    PayloadTooLarge:
      description: Request body exceeded the configured size limit.
${ERROR_BODY}
    TooManyRequests:
      description: Rate limit exceeded.
      headers:
        Retry-After:
          description: Seconds until the client may retry.
          schema: { type: integer, minimum: 1 }
          required: true
${ERROR_BODY}
    Unauthorized:
      description: Missing or invalid bearer token.
${ERROR_BODY}`;

const COMPLETE_PATHS = `  /api/work-orders:
    post:
      responses:
        '201': { description: created }
        '408':
          $ref: '#/components/responses/RequestTimeout'
        '413':
          $ref: '#/components/responses/PayloadTooLarge'
        '429':
          $ref: '#/components/responses/TooManyRequests'`;

const JSON_ERROR_SOURCE = `
fn error_response(status: StatusCode, code: &'static str, message: &str) -> Response {
    (
        status,
        Json(ErrorBody {
            error: ErrorPayload {
                code,
                message: message.to_owned(),
            },
        }),
    )
        .into_response()
}
`;

const PLAINTEXT_ERROR_SOURCE = `
fn error_response(status: StatusCode, message: &str) -> Response {
    (status, message.to_owned()).into_response()
}
`;

const RETRY_AFTER_SOURCE = `
response.headers_mut().insert(header::RETRY_AFTER, HeaderValue::from_static("60"));
`;

describe("openapi HTTP error-envelope gate", () => {
  it("reports TooManyRequests without Retry-After", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": spec({
        responses: `    RequestTimeout:
      description: timeout
${ERROR_BODY}
    PayloadTooLarge:
      description: too large
${ERROR_BODY}
    TooManyRequests:
      description: rate limited
${ERROR_BODY}`,
        paths: COMPLETE_PATHS,
      }),
    });

    const { findings } = evaluateOpenapiErrorEnvelope({ repoRoot: root });

    assert.ok(
      findings.some((finding) => /Retry-After/.test(finding.message)),
      JSON.stringify(findings, null, 2),
    );
  });

  it("reports missing documented 408 and 413", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": spec({
        responses: COMPLETE_RESPONSES,
        paths: `  /api/v1/auth/signup:
    post:
      responses:
        '202': { description: accepted }
        '429':
          $ref: '#/components/responses/TooManyRequests'`,
      }),
    });

    const { findings, documented408, documented413 } = evaluateOpenapiErrorEnvelope({
      repoRoot: root,
    });

    assert.equal(documented408, 0);
    assert.equal(documented413, 0);
    assert.ok(findings.some((finding) => /408/.test(finding.message)));
    assert.ok(findings.some((finding) => /413/.test(finding.message)));
  });

  it("accepts shared RequestTimeout/PayloadTooLarge plus Retry-After on 429", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": spec({
        responses: COMPLETE_RESPONSES,
        paths: COMPLETE_PATHS,
      }),
    });

    const { findings, documented408, documented413, documented429 } =
      evaluateOpenapiErrorEnvelope({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(documented408, 1);
    assert.equal(documented413, 1);
    assert.equal(documented429, 1);
  });

  it("reports plaintext request-context error_response", () => {
    const root = fixture({
      "backend/crates/platform/request-context/src/lib.rs": PLAINTEXT_ERROR_SOURCE,
      "backend/app/src/lib.rs": RETRY_AFTER_SOURCE,
    });

    const { findings } = evaluateRuntimeErrorEnvelope({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].location, /request-context/);
    assert.match(findings[0].message, /JSON ErrorBody/);
  });

  it("reports 429 production without Retry-After", () => {
    const root = fixture({
      "backend/crates/platform/request-context/src/lib.rs": JSON_ERROR_SOURCE,
      "backend/app/src/lib.rs": "fn build_router() {}",
    });

    const { findings } = evaluateRuntimeErrorEnvelope({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /Retry-After/);
  });

  it("accepts JSON ErrorBody plus Retry-After in runtime sources", () => {
    const root = fixture({
      "backend/crates/platform/request-context/src/lib.rs": JSON_ERROR_SOURCE + RETRY_AFTER_SOURCE,
      "backend/app/src/lib.rs": "fn build_router() {}",
    });

    const { findings } = evaluateRuntimeErrorEnvelope({ repoRoot: root });

    assert.deepEqual(findings, []);
  });

  it("exits 1 naming the floor when the document contains almost no paths", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": spec({
        responses: COMPLETE_RESPONSES,
        paths: COMPLETE_PATHS,
      }),
      "backend/crates/platform/request-context/src/lib.rs": JSON_ERROR_SOURCE + RETRY_AFTER_SOURCE,
      "backend/app/src/lib.rs": "fn build_router() {}",
    });

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /below the floor/);
  });

  it("exits 1 loudly when the document is not parseable YAML at all", () => {
    const root = fixture({
      "backend/openapi/openapi.yaml": "openapi: 3.1.0\n  bad-indent: {\n",
      "backend/crates/platform/request-context/src/lib.rs": JSON_ERROR_SOURCE,
      "backend/app/src/lib.rs": RETRY_AFTER_SOURCE,
    });

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /cannot evaluate/);
  });

  // The live document + middleware is the hole this lane closes. This
  // assertion is red on origin/dev (plaintext 401, undocumented 408/413, 429
  // without Retry-After) and green only after those three siblings match
  // ErrorBody.
  it("exits 0 against this repository, above the floors, with no envelope holes", () => {
    const { findings, paths, responses } = evaluateErrorEnvelope({ repoRoot });

    assert.deepEqual(findings, [], JSON.stringify(findings.slice(0, 8), null, 2));
    assert.ok(paths >= PATH_FLOOR, `walker degraded: saw ${paths} paths, floor ${PATH_FLOOR}`);
    assert.ok(
      responses >= RESPONSE_FLOOR,
      `walker degraded: saw ${responses} shared responses, floor ${RESPONSE_FLOOR}`,
    );

    const result = spawnSync(process.execPath, [cli], { encoding: "utf8" });

    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /openapi error-envelope gate passed/);
  });
});
