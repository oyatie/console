import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { evaluateRequestBodyContract } from "./check-request-body-contract.mjs";

// Source-analysis fixtures only: these do not execute or replace the Auth owner.
// They use the evaluator's existing public boundary and temp-repository pattern.
const nativeRepoRoot = fileURLToPath(new URL("..", import.meta.url));
const nativeOwnerPath = "backend/crates/platform/auth-rest/src/account_browser.rs";
const nativeRouterPath = "backend/crates/platform/auth-rest/src/lib.rs";
const nativeFixtureRoots = [];
after(() => {
  for (const root of nativeFixtureRoots) rmSync(root, { recursive: true, force: true });
});

function nativeFixture(files) {
  const root = mkdtempSync(join(tmpdir(), "native-request-body-contract-"));
  nativeFixtureRoots.push(root);
  for (const [relative, contents] of Object.entries(files)) {
    const target = join(root, relative);
    mkdirSync(dirname(target), { recursive: true });
    writeFileSync(target, contents);
  }
  return root;
}

// These are exact app-owned top-level fields, not an invented wire grammar for
// nested WebAuthn. The existing checker proves names/requiredness, not crypto.
const nativeOperations = [
  { handler: "registration_start", path: "/api/v2/auth/registration/start", type: "RegistrationStartInput", fields: ["terms_version"] },
  { handler: "registration_finish", path: "/api/v2/auth/registration/finish", type: "RegistrationFinishInput", fields: ["ceremony_id", "credential", "accept_terms_version", "accept_items"] },
  { handler: "login_start", path: "/api/v2/auth/passkey/login/start", type: "EmptyInput", fields: [] },
  { handler: "login_finish", path: "/api/v2/auth/passkey/login/finish", type: "LoginFinishInput", fields: ["ceremony_id", "assertion"] },
  { handler: "logout", path: "/api/v2/auth/logout", type: "EmptyInput", fields: [] },
  { handler: "refresh", path: "/api/v2/auth/token/refresh", type: "EmptyInput", fields: [] },
];

function nativeSpec(operations, mutate = () => {}) {
  const document = { openapi: "3.1.0", info: { title: "Native source fixture", version: "1" }, paths: {}, components: { schemas: {} } };
  for (const operation of operations) {
    document.paths[operation.path] = { post: {
      requestBody: { required: true, content: { "application/json": { schema: { $ref: `#/components/schemas/${operation.type}` } } } },
      responses: { "200": { description: "Source fixture; does not model response semantics" } },
    } };
    document.components.schemas[operation.type] = {
      type: "object", additionalProperties: false,
      required: operation.fields,
      properties: Object.fromEntries(operation.fields.map((field) => [field, {}])),
    };
  }
  mutate(document);
  // JSON is YAML-compatible and avoids a second fixture-only schema generator.
  return JSON.stringify(document);
}

function nativeRouter(operations, moduleDeclaration = "mod account_browser;") {
  return `${moduleDeclaration}\n${operations.map((operation, index) => `pub const NATIVE_PATH_${index}: &str = "${operation.path}";`).join("\n")}
pub fn router() -> Router {
    Router::new()
${operations.map((operation, index) => `        .route(NATIVE_PATH_${index}, post(account_browser::${operation.handler}))`).join("\n")}
}
`;
}

function changeExact(source, old, next) {
  assert.equal(source.split(old).length, 2, `fixture anchor must occur exactly once: ${old}`);
  return source.replace(old, next);
}

function changeNativeHandler(source, handler, change) {
  const marker = `pub(super) async fn ${handler}(`;
  const start = source.indexOf(marker);
  assert.ok(start >= 0, `missing retained native handler ${handler}`);
  const next = source.indexOf("pub(super) async fn ", start + marker.length);
  const end = next < 0 ? source.length : next;
  const before = source.slice(start, end);
  const after = change(before);
  assert.notEqual(after, before, `mutation must change ${handler}`);
  return source.slice(0, start) + after + source.slice(end);
}

function nativeBodyReport({ operations = nativeOperations, mutateOwner = (source) => source, mutateSpec, mutateRouter = (source) => source, extraFiles = {} } = {}) {
  const owner = readFileSync(join(nativeRepoRoot, nativeOwnerPath), "utf8");
  return evaluateRequestBodyContract({ repoRoot: nativeFixture({
    [nativeRouterPath]: mutateRouter(nativeRouter(operations)),
    [nativeOwnerPath]: mutateOwner(owner),
    "backend/openapi/openapi.yaml": nativeSpec(operations, mutateSpec),
    "scripts/request-body-contract-undecidable.json": JSON.stringify({ version: 1, body: [], enum: [] }),
    ...extraFiles,
  }) });
}

function expectNativeResolved(report, count) {
  assert.equal(report.population, count, JSON.stringify(report));
  assert.equal(report.resolved, count, JSON.stringify(report));
  assert.equal(report.skipped, 0, JSON.stringify(report));
  assert.deepEqual(report.observedRegister, { version: 1, body: [], enum: [] });
  assert.deepEqual(report.registerFindings, []);
}

function expectNativeUnresolved(report, count = 1) {
  assert.equal(report.population, count, JSON.stringify(report));
  assert.equal(report.resolved, 0, JSON.stringify(report));
  assert.equal(report.skipped, count, JSON.stringify(report));
  assert.equal(report.observedRegister.body.length, count);
  assert.deepEqual(report.findings, [], "unproven source must not fabricate a DTO comparison");
  assert.notEqual(report.registerFindings.length, 0, "unresolved source remains visible to the exact register");
}

function qualifiedJsonReport({ moduleDeclaration = "mod actual;", route = "actual::submit", source = "", files = {}, specFields = ["expected"], extraRouter = "" } = {}) {
  const definition = `#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub expected: String,
}
pub async fn submit(Json(input): Json<Input>) -> Response { todo!() }
`;
  return evaluateRequestBodyContract({ repoRoot: nativeFixture({
    "backend/crates/widget/rest/src/lib.rs": `${moduleDeclaration}
pub const WIDGET_PATH: &str = "/widget";
pub fn router() -> Router { Router::new().route(WIDGET_PATH, post(${route})) }
${extraRouter}
`,
    "backend/crates/widget/rest/src/actual.rs": source || definition,
    "backend/openapi/openapi.yaml": nativeSpec([{ path: "/widget", type: "Input", fields: specFields }]),
    "scripts/request-body-contract-undecidable.json": JSON.stringify({ version: 1, body: [], enum: [] }),
    ...files,
  }) });
}

describe("native typed raw-body contract source binding", () => {
  for (const operation of nativeOperations) {
    it(`resolves actual ${operation.handler} as ${operation.type}`, () => {
      const report = nativeBodyReport({ operations: [operation] });
      expectNativeResolved(report, 1);
      assert.deepEqual(report.findings, []);
    });
  }

  it("compares all six retained bodies without adding undecidable entries", () => {
    const report = nativeBodyReport();
    expectNativeResolved(report, 6);
    assert.deepEqual(report.findings, []);
  });

  it("detects a wrong property on all three nonempty inputs", () => {
    const operations = nativeOperations.filter((operation) => operation.fields.length > 0);
    const report = nativeBodyReport({ operations, mutateSpec(document) {
      for (const operation of operations) document.components.schemas[operation.type].properties.invented = {};
    } });
    expectNativeResolved(report, 3);
    assert.deepEqual(report.findings, operations.map((operation) => ({
      operation: `POST ${operation.path}`,
      message: `spec property "invented" is not a field of ${operation.type} (deny_unknown_fields => 422)`,
    })).sort((left, right) => left.operation.localeCompare(right.operation)));
  });

  it("detects omitted required native wrapper fields", () => {
    const operation = nativeOperations[0];
    const report = nativeBodyReport({ operations: [operation], mutateSpec(document) {
      document.components.schemas.RegistrationStartInput.required = [];
    } });
    expectNativeResolved(report, 1);
    assert.deepEqual(report.findings, [{ operation: `POST ${operation.path}`, message: "RegistrationStartInput.terms_version is required by the handler but not in spec required[]" }]);
  });

  it("does not mistake an empty typed body for an absent body", () => {
    const operations = nativeOperations.filter((operation) => operation.type === "EmptyInput");
    const report = nativeBodyReport({ operations, mutateSpec(document) {
      document.components.schemas.EmptyInput.properties.token = {};
    } });
    expectNativeResolved(report, 3);
    assert.equal(report.findings.length, 3);
    assert.ok(report.findings.every((finding) => finding.message === 'spec property "token" is not a field of EmptyInput (deny_unknown_fields => 422)'));
  });

  it("records a recognized typed body omitted from OpenAPI instead of hiding it", () => {
    const operation = nativeOperations[2];
    const report = nativeBodyReport({ operations: [operation], mutateSpec(document) {
      delete document.paths[operation.path].post.requestBody;
    } });
    expectNativeUnresolved(report);
    assert.equal(report.observedRegister.body[0].reason, "no_openapi_request_body");
    assert.equal(report.observedRegister.body[0].body_type, "EmptyInput");
  });

  it("resolves a qualified direct Json handler in its actual module", () => {
    const report = qualifiedJsonReport({ extraRouter: `#[serde(deny_unknown_fields)]
struct Input { decoy: String }
` });
    expectNativeResolved(report, 1);
    assert.deepEqual(report.findings, []);
  });

  it("finds direct Json drift through the qualified handler instead of a caller decoy", () => {
    const report = qualifiedJsonReport({ specFields: ["decoy"], extraRouter: `#[serde(deny_unknown_fields)]
struct Input { decoy: String }
` });
    expectNativeResolved(report, 1);
    assert.ok(report.findings.some((finding) => finding.message === 'spec property "decoy" is not a field of Input (deny_unknown_fields => 422)'));
  });

  it("follows the declared path-remapped handler module", () => {
    const report = qualifiedJsonReport({ moduleDeclaration: '#[path = "actual.rs"]\nmod browser;', route: "browser::submit" });
    expectNativeResolved(report, 1);
    assert.deepEqual(report.findings, []);
  });

  it("keeps a correctly resolved permissive Json DTO undecidable with truthful type metadata", () => {
    const report = qualifiedJsonReport({ source: `#[derive(Deserialize)]
pub struct Input {
    pub expected: String,
}
pub async fn submit(Json(input): Json<Input>) -> Response { todo!() }
` });
    expectNativeUnresolved(report);
    assert.equal(report.observedRegister.body[0].body_type, "Input");
    assert.equal(report.observedRegister.body[0].reason, "rust_struct_not_strict");
  });

  it("does not fall back from an unknown handler module to a matching bare function", () => {
    expectNativeUnresolved(qualifiedJsonReport({ route: "missing::submit" }));
  });

  it("keeps a cfg-conditional handler module unresolved", () => {
    expectNativeUnresolved(qualifiedJsonReport({ moduleDeclaration: "#[cfg(any())]\nmod actual;" }));
  });

  it("ignores same-path route decoys inside comments, strings and macro tokens", () => {
    const report = qualifiedJsonReport({ extraRouter: `
// Router::new().route(WIDGET_PATH, post(decoy));
const ROUTE_TEXT: &str = r#"Router::new().route(WIDGET_PATH, post(decoy))"#;
macro_rules! unused_routes { () => { Router::new().route(WIDGET_PATH, post(decoy)) }; }
pub async fn decoy(Json(input): Json<Decoy>) -> Response { todo!() }
#[serde(deny_unknown_fields)]
struct Decoy { wrong: String }
` });
    expectNativeResolved(report, 1);
    assert.deepEqual(report.findings, []);
  });

  const start = nativeOperations[0];
  const read = "let input: RegistrationStartInput = read_json(body, START_BODY_LIMIT).await?;";
  const replaceStartRead = (replacement) => (source) => changeNativeHandler(source, "registration_start", (handler) => changeExact(handler, read, replacement));
  it("ignores shadow-shaped literal/comment text beside a real typed read", () => {
    const report = nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`
    // fn read_json; let body = Body::empty(); type RegistrationStartInput = Other;
    let note = r#"use other::read_json; mod serde_json {}"#;
    ${read}`) });
    expectNativeResolved(report, 1);
    assert.deepEqual(report.findings, []);
  });
  for (const [name, replacement] of [
    ["comment", `// ${read}\n    return Err(BrowserError::InvalidRequest);`],
    ["raw string", `let decoy = r#"${read}"#;\n    return Err(BrowserError::InvalidRequest);`],
    ["nested function", `async fn decoy(body: Body) -> Result<(), BrowserError> { ${read} Ok(()) }\n    return Err(BrowserError::InvalidRequest);`],
    ["closure", `let decoy = async move { ${read} };\n    return Err(BrowserError::InvalidRequest);`],
    ["conditional branch", `if false { ${read} }\n    return Err(BrowserError::InvalidRequest);`],
    ["unexpanded macro", `macro_rules! decoy { () => { ${read} }; }\n    return Err(BrowserError::InvalidRequest);`],
  ]) {
    it(`does not infer network input from a typed read in a ${name}`, () => {
      expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(replacement) }));
    });
  }

  it("rejects a same-name helper that no longer parses JSON", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner(source) {
      return changeExact(source, "serde_json::from_slice(&bytes).map_err(|_| BrowserError::InvalidRequest)", "panic!(\"fixture: same name no longer establishes JSON parsing\")");
    } }));
  });

  it("rejects a helper parsing unrelated bytes despite retaining the parser name", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner(source) {
      return changeExact(source, "serde_json::from_slice(&bytes).map_err(|_| BrowserError::InvalidRequest)", 'serde_json::from_slice(b"{}").map_err(|_| BrowserError::InvalidRequest)');
    } }));
  });

  it("rejects a helper whose original parse exists only in a nested decoy", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner(source) {
      const original = "serde_json::from_slice(&bytes).map_err(|_| BrowserError::InvalidRequest)";
      return changeExact(source, original, `fn decoy<T: de::DeserializeOwned>(bytes: &[u8]) -> Result<T, BrowserError> {
        ${original}
    }
    panic!("outer helper no longer parses")`);
    } }));
  });

  it("does not trust a nested helper shadowing the known parser", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`
    async fn read_json<T>(body: Body, limit: usize) -> Result<T, BrowserError> { panic!("shadow") }
    ${read}`) }));
  });

  it("refuses a function-scope helper import even when declared after the read", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`${read}\n    use replacement::read_json;`) }));
  });

  it("refuses a function-scope parser function even when declared after the read", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`${read}\n    async fn read_json<T>(body: Body, limit: usize) -> Result<T, BrowserError> { panic!("hoisted shadow") }`) }));
  });

  it("refuses a function-scope type alias shadowing the bound DTO", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`${read}
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ShadowInput { #[serde(rename = "other")] terms_version: String }
    type RegistrationStartInput = ShadowInput;`) }));
  });

  it("refuses a parameter-local parser closure shadow", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`let read_json = |body, limit| async { panic!("shadow") };\n    ${read}`) }));
  });

  it("does not bind a typed read of an unrelated body expression", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(read.replace("read_json(body,", "read_json(Body::empty(),")) }));
  });

  it("does not bind a network body parameter shadowed before parsing", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`let body = Body::empty();\n    ${read}`) }));
  });

  it("refuses competing typed reads instead of choosing first or last", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`${read}\n    let competing: EmptyInput = read_json(Body::empty(), START_BODY_LIMIT).await?;`) }));
  });

  it("does not accept an unrecognized body limit expression", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(read.replace("START_BODY_LIMIT", "usize::MAX")) }));
  });

  it("does not trust a function-local constant shadowing the admitted body limit", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: replaceStartRead(`${read}\n    const START_BODY_LIMIT: usize = usize::MAX;`) }));
  });

  it("refuses an imported alias that replaces serde_json under the known helper", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: (source) => changeExact(source, "use axum::Json;", "use replacement_json as serde_json;\nuse axum::Json;") }));
  });

  it("refuses replacing the bounded body reader import", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner(source) {
      return changeExact(source, "use axum::body::{Body, to_bytes};", "use axum::body::Body;\nuse replacement::to_bytes;");
    } }));
  });

  it("refuses a module shadowing serde_json under the known helper", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [start], mutateOwner: (source) => `${source}\nmod serde_json { pub fn from_slice<T>(bytes: &[u8]) -> Result<T, ()> { panic!("shadow") } }\n` }));
  });

  it("refuses a foreign parser sharing read_json's name", () => {
    const source = `#[serde(deny_unknown_fields)]
pub struct Input { expected: String }
async fn read_json<T>(body: Body, limit: usize) -> Result<T, Error> { panic!("unproven helper") }
pub async fn submit(body: Body) -> Response {
    let input: Input = read_json(body, 4096).await?;
    todo!()
}
`;
    expectNativeUnresolved(qualifiedJsonReport({ source }));
  });
});


describe("native parser shadowing review regressions", () => {
  it("does not resolve a qualified router namespace through a shadowing local import", () => {
    const input = (field) => `#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub ${field}: String,
}
pub async fn submit(Json(input): Json<Input>) -> Response { todo!() }
`;
    const root = nativeFixture({
      "backend/crates/widget/rest/src/lib.rs": `mod actual;
mod alternate;
pub const WIDGET_PATH: &str = "/widget";
pub fn router() -> Router {
    use crate::alternate as actual;
    Router::new().route(WIDGET_PATH, post(actual::submit))
}
`,
      "backend/crates/widget/rest/src/actual.rs": input("expected"),
      "backend/crates/widget/rest/src/alternate.rs": input("actual_wire"),
      "backend/openapi/openapi.yaml": nativeSpec([{ path: "/widget", type: "Input", fields: ["expected"] }]),
      "scripts/request-body-contract-undecidable.json": JSON.stringify({ version: 1, body: [], enum: [] }),
    });
    expectNativeUnresolved(evaluateRequestBodyContract({ repoRoot: root }));
  });

  it("does not trust an extern-crate alias replacing the reviewed serde_json namespace", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [nativeOperations[0]], mutateOwner(source) {
      return changeExact(source, "use axum::Json;", "extern crate replacement as serde_json;\nuse axum::Json;");
    } }));
  });

  it("does not trust a disabled limit declaration beside a different live binding", () => {
    expectNativeUnresolved(nativeBodyReport({ operations: [nativeOperations[0]], mutateOwner(source) {
      const original = "pub(super) const START_BODY_LIMIT: usize = 4 * 1024;";
      return changeExact(source, original, `#[cfg(any())]\n${original}\npub(super) const START_BODY_LIMIT: usize = usize::MAX;`);
    } }));
  });
});


describe("native ancestor namespace and macro review regressions", () => {
  for (const namespace of ["serde_json", "axum"]) {
    it(`does not trust a root extern-crate alias replacing ${namespace}`, () => {
      expectNativeUnresolved(nativeBodyReport({ operations: [nativeOperations[0]], mutateRouter(source) {
        return `extern crate replacement as ${namespace};\n${source}`;
      } }));
    });
  }

  it("does not trust an ancestor macro that replaces the native handler body limit", () => {
    const read = "let input: RegistrationStartInput = read_json(body, START_BODY_LIMIT).await?;";
    expectNativeUnresolved(nativeBodyReport({ operations: [nativeOperations[0]],
      mutateRouter: (source) => `macro_rules! shadow { () => { const START_BODY_LIMIT: usize = usize::MAX; } }\n${source}`,
      mutateOwner: (source) => changeNativeHandler(source, "registration_start", (handler) => changeExact(handler, read, `shadow!(); ${read}`)),
    }));
  });

  it("does not resolve a qualified router namespace through a macro-generated alias", () => {
    const input = (field) => `#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub ${field}: String,
}
pub async fn submit(Json(input): Json<Input>) -> Response { todo!() }
`;
    const root = nativeFixture({
      "backend/crates/widget/rest/src/lib.rs": `macro_rules! shadow { () => { use crate::alternate as actual; } }
mod actual;
mod alternate;
pub const WIDGET_PATH: &str = "/widget";
pub fn router() -> Router {
    shadow!();
    Router::new().route(WIDGET_PATH, post(actual::submit))
}
`,
      "backend/crates/widget/rest/src/actual.rs": input("expected"),
      "backend/crates/widget/rest/src/alternate.rs": input("actual_wire"),
      "backend/openapi/openapi.yaml": nativeSpec([{ path: "/widget", type: "Input", fields: ["expected"] }]),
      "scripts/request-body-contract-undecidable.json": JSON.stringify({ version: 1, body: [], enum: [] }),
    });
    expectNativeUnresolved(evaluateRequestBodyContract({ repoRoot: root }));
  });

  it("does not trust an ancestor macro that replaces the native parser with an unbounded reader", () => {
    const read = "let input: RegistrationStartInput = read_json(body, START_BODY_LIMIT).await?;";
    expectNativeUnresolved(nativeBodyReport({ operations: [nativeOperations[0]],
      mutateRouter: (source) => `macro_rules! shadow { () => {
        async fn read_json(body: Body, _limit: usize) -> Result<RegistrationStartInput, BrowserError> {
          let bytes = to_bytes(body, usize::MAX).await.map_err(|_| BrowserError::RequestTooLarge)?;
          serde_json::from_slice(&bytes).map_err(|_| BrowserError::InvalidRequest)
        }
      } }\n${source}`,
      mutateOwner: (source) => changeNativeHandler(source, "registration_start", (handler) => changeExact(handler, read, `shadow!(); ${read}`)),
    }));
  });
});


it("does not resolve a qualified module through a router type parameter", () => {
  expectNativeUnresolved(qualifiedJsonReport({ files: {
    "backend/crates/widget/rest/src/lib.rs": `mod actual;
pub const WIDGET_PATH: &str = "/widget";
pub fn router<actual: Handler>() -> Router {
    Router::new().route(WIDGET_PATH, post(actual::submit))
}
`,
  } }));
});


it("does not resolve the native DTO through a handler type parameter", () => {
  expectNativeUnresolved(nativeBodyReport({ operations: [nativeOperations[2]],
    mutateOwner: (source) => changeExact(source,
      "pub(super) async fn login_start(",
      "pub(super) async fn login_start<EmptyInput: de::DeserializeOwned>(")
      + `\n#[derive(Deserialize)]\n#[serde(deny_unknown_fields)]\npub(super) struct AlternateInput {\n    pub actual_wire: String,\n}\n`,
    mutateRouter: (source) => changeExact(source,
      "post(account_browser::login_start)",
      "post(account_browser::login_start::<account_browser::AlternateInput>)"),
  }));
});


describe("native literal route ownership", () => {
  it("resolves literal paths with qualified native handlers", () => {
    const report = nativeBodyReport({ mutateRouter: (source) => {
      for (const [index, operation] of nativeOperations.entries()) {
        source = source.replace(`.route(NATIVE_PATH_${index},`, `.route("${operation.path}",`);
      }
      return source;
    } });
    expectNativeResolved(report, nativeOperations.length);
  });

  for (const [name, wrap] of [
    ["comment", (route) => `/* ${route} */`],
    ["macro", (route) => `stringify! { ${route} }`],
  ]) {
    it(`does not bind a literal route inside a ${name}`, () => {
      const operation = nativeOperations[0];
      const report = nativeBodyReport({ operations: [operation], mutateRouter: (source) =>
        source.replace(`.route(NATIVE_PATH_0, post(account_browser::${operation.handler}))`,
          wrap(`.route("${operation.path}", post(account_browser::${operation.handler}))`)),
      });
      assert.equal(report.resolved, 0, JSON.stringify(report));
      assert.equal(report.skipped, 1, JSON.stringify(report));
    });
  }
});


it("does not prove a literal route from a commented native handler", () => {
  const operation = nativeOperations[0];
  const report = nativeBodyReport({ operations: [operation], mutateRouter: (source) =>
    source.replace(`.route(NATIVE_PATH_0, post(account_browser::${operation.handler}))`,
      `.route("${operation.path}", /* post(account_browser::${operation.handler})) */ post(other::submit))`),
  });
  expectNativeUnresolved(report);
});
