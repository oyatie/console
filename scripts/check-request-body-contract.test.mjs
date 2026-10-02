import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  evaluateRequestBodyContract,
  jsonRequestSchema,
  LITERAL_PATH_ANCHORS,
  renameField,
  renameVariant,
} from "./check-request-body-contract.mjs";
import {
  hasOwnKey,
  own,
  PROTOTYPE_CHAIN_KEYS,
} from "./own-property.mjs";
import * as yaml from "js-yaml";

const repoRoot = fileURLToPath(new URL("..", import.meta.url));
const cli = fileURLToPath(new URL("./check-request-body-contract.mjs", import.meta.url));
const fixtureRoots = [];

const handoverAnchor = "POST /api/v1/equipment-3r/rental-cases/{case_id}/handover";
const consumptionsAnchor = "POST /api/v1/inventory/items/{item_id}/consumptions";
const receiptsAnchor = "POST /api/v1/inventory/items/{item_id}/receipts";

after(() => {
  for (const root of fixtureRoots) rmSync(root, { recursive: true, force: true });
});

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "request-body-contract-"));
  fixtureRoots.push(root);
  for (const [relative, contents] of Object.entries(files)) {
    const absolute = join(root, relative);
    mkdirSync(dirname(absolute), { recursive: true });
    writeFileSync(absolute, contents);
  }
  return root;
}

function copyRustSources(source, destination) {
  for (const entry of readdirSync(source, { withFileTypes: true })
    .sort((left, right) => left.name.localeCompare(right.name))) {
    if ([".git", "node_modules", "target"].includes(entry.name)) continue;
    const sourcePath = join(source, entry.name);
    const destinationPath = join(destination, entry.name);
    if (entry.isDirectory()) copyRustSources(sourcePath, destinationPath);
    else if (entry.name.endsWith(".rs")) {
      mkdirSync(dirname(destinationPath), { recursive: true });
      writeFileSync(destinationPath, readFileSync(sourcePath));
    }
  }
}

function liveSourceFixture() {
  const root = mkdtempSync(join(tmpdir(), "request-body-contract-live-"));
  fixtureRoots.push(root);
  copyRustSources(join(repoRoot, "backend"), join(root, "backend"));
  for (const path of [
    "backend/openapi/openapi.yaml",
    "scripts/request-body-contract-undecidable.json",
  ]) {
    const destination = join(root, path);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, readFileSync(join(repoRoot, path)));
  }
  return root;
}

function liveCycleKindBoundaryFixture(discriminant) {
  const root = liveSourceFixture();
  const rustPath = join(root, "backend/crates/evaluation/domain/src/lib.rs");
  const rustSource = readFileSync(rustPath, "utf8");
  const discriminatedRust = rustSource.replace(
    "    Regular,\n    Probation,",
    `    Regular = ${discriminant} as isize,\n    Probation,`,
  );
  assert.notEqual(discriminatedRust, rustSource, "CycleKind fixture no longer matches the reviewed source");
  writeFileSync(rustPath, discriminatedRust);

  const openapiPath = join(root, "backend/openapi/openapi.yaml");
  const openapi = readFileSync(openapiPath, "utf8");
  const narrowedOpenapi = openapi.replace(
    "    EvaluationCycleKind:\n      type: string\n      enum: [REGULAR, PROBATION]",
    "    EvaluationCycleKind:\n      type: string\n      enum: [REGULAR]",
  );
  assert.notEqual(narrowedOpenapi, openapi, "EvaluationCycleKind fixture no longer matches OpenAPI");
  writeFileSync(openapiPath, narrowedOpenapi);
  return root;
}

function liveCycleKindScopeDecoyFixture(decoy) {
  const root = liveSourceFixture();
  const rustPath = join(root, "backend/crates/evaluation/rest/src/lib.rs");
  const rustSource = readFileSync(rustPath, "utf8");
  writeFileSync(rustPath, `${rustSource}\n${decoy}\n`);

  const openapiPath = join(root, "backend/openapi/openapi.yaml");
  const openapi = readFileSync(openapiPath, "utf8");
  const narrowedOpenapi = openapi.replace(
    "    EvaluationCycleKind:\n      type: string\n      enum: [REGULAR, PROBATION]",
    "    EvaluationCycleKind:\n      type: string\n      enum: [REGULAR]",
  );
  assert.notEqual(narrowedOpenapi, openapi, "EvaluationCycleKind fixture no longer matches OpenAPI");
  writeFileSync(openapiPath, narrowedOpenapi);
  return root;
}

function assertLiveCycleKindProbationFinding(root) {
  const report = evaluateRequestBodyContract({ repoRoot: root });

  assert.deepEqual(
    {
      population: report.population,
      resolved: report.resolved,
      skipped: report.skipped,
      enumCandidates: report.enumCandidates,
      enumResolved: report.enumResolved,
      enumSkipped: report.enumSkipped,
    },
    {
      population: 299,
      resolved: 124,
      skipped: 175,
      enumCandidates: 50,
      enumResolved: 20,
      enumSkipped: 30,
    },
  );
  assert.deepEqual(report.registerFindings, []);

  const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

  assert.equal(result.status, 1, `${result.stdout}${result.stderr}\n${JSON.stringify(report, null, 2)}`);
  assert.match(result.stderr, /Rust-only enum variant "PROBATION" for kind/);
  assert.deepEqual(report.findings, [{
    operation: "POST /api/v1/evaluation/cycles",
    message: 'Rust-only enum variant "PROBATION" for kind',
  }]);
}

// Both wrapped-const and multi-method route forms, because both appear in the real surface and
// both have already broken a resolver silently.
function widgetLiteralCrate({ derive, fields }) {
  return `use axum::{routing::post, Json, Router};

pub fn router(state: WidgetState) -> Router {
    Router::new()
        .route("/api/v1/widgets/{widget_id}/consumptions", post(consume_widget))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
${derive}
struct ConsumeWidgetBody {
${fields}
}

async fn consume_widget(
    State(state): State<WidgetState>,
    Path(widget_id): Path<Uuid>,
    Json(body): Json<ConsumeWidgetBody>,
) -> Result<Json<WidgetView>, RestError> {
    todo!()
}
`;
}

function widgetCrate({ derive, fields }) {
  return `use axum::{routing::get, routing::post, Json, Router};

pub const WIDGET_CONSUME_PATH: &str =
    "/api/v1/widgets/{widget_id}/consumptions";

pub fn router(state: WidgetState) -> Router {
    Router::new()
        .route(
            WIDGET_CONSUME_PATH,
            get(list_widget_consumptions).post(consume_widget),
        )
        .with_state(state)
}

#[derive(Debug, Deserialize)]
${derive}
struct ConsumeWidgetBody {
${fields}
}

async fn list_widget_consumptions(
    State(state): State<WidgetState>,
) -> Result<Json<Vec<WidgetView>>, RestError> {
    todo!()
}

async fn consume_widget(
    State(state): State<WidgetState>,
    Path(widget_id): Path<Uuid>,
    Json(body): Json<ConsumeWidgetBody>,
) -> Result<Json<WidgetView>, RestError> {
    todo!()
}
`;
}

function widgetSpec({ required, properties, extraSchemas = "" }) {
  return `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
  /api/v1/widgets/{widget_id}/consumptions:
    post:
      operationId: consumeWidget
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ConsumeWidgetRequest'
      responses:
        '200':
          description: ok
components:
  schemas:
    ConsumeWidgetRequest:
      type: object
      additionalProperties: false
      required: [${required.join(", ")}]
      properties: { ${properties} }
${extraSchemas}
`;
}

function widgetFixture({ derive, fields, required, properties, extraSchemas = "" }) {
  return fixture({
    "backend/crates/widget/rest/src/lib.rs": widgetCrate({ derive, fields }),
    "backend/openapi/openapi.yaml": widgetSpec({ required, properties, extraSchemas }),
  });
}

function enumFixture({
  fieldType = "WidgetMode",
  enumSource = `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
    FastMode,
    SafeMode,
}
`,
  property = "{ type: string, enum: [fast_mode, safe_mode] }",
  extraSchemas = "",
} = {}) {
  return fixture({
    "backend/crates/widget/rest/src/lib.rs": `${widgetCrate({
      derive: camelDeny,
      fields: `    quantity_consumed_milli: i64,\n    mode: ${fieldType},`,
    })}\n${enumSource}`,
    "backend/openapi/openapi.yaml": widgetSpec({
      required: ["quantityConsumedMilli", "mode"],
      properties: `quantityConsumedMilli: { type: integer }, mode: ${property}`,
      extraSchemas,
    }),
  });
}

function writeWidgetModeModule(root, path, variants = "FastMode, SafeMode") {
  const absolute = join(root, `backend/crates/widget/rest/src/${path}`);
  mkdirSync(dirname(absolute), { recursive: true });
  writeFileSync(absolute, `#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { ${variants} }
`);
}

function writeObservedRegister(root, report = evaluateRequestBodyContract({ repoRoot: root })) {
  const registerPath = join(root, "scripts/request-body-contract-undecidable.json");
  mkdirSync(dirname(registerPath), { recursive: true });
  writeFileSync(registerPath, `${JSON.stringify(report.observedRegister, null, 2)}\n`);
  return registerPath;
}

const camelDeny = '#[serde(rename_all = "camelCase", deny_unknown_fields)]';
const widgetOperation = "POST /api/v1/widgets/{widget_id}/consumptions";

describe("request body contract gate", () => {
  it("reports a spec property the struct's rename_all makes unreachable", () => {
    // rename_all = "camelCase" + deny_unknown_fields means a snake_case body field is a
    // guaranteed 422, not a maybe.
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,\n    memo: Option<String>,",
      required: ["quantity_consumed_milli"],
      properties: "quantity_consumed_milli: { type: integer }, memo: { type: string }",
    });

    const { findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.equal(findings[0].operation, widgetOperation);
    assert.match(findings[0].message, /quantity_consumed_milli/);
    assert.match(findings[0].message, /ConsumeWidgetBody/);
  });

  it("reports a required spec property that is not a field of the bound struct at all", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,",
      required: ["quantityConsumedMilli", "ghostField"],
      properties: "quantityConsumedMilli: { type: integer }, ghostField: { type: string }",
    });

    const { findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /ghostField/);
  });

  it("reports a handler-required field the spec does not list as required", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,\n    idempotency_key: String,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }, idempotencyKey: { type: string }",
    });

    const { findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /idempotency_key/);
    assert.match(findings[0].message, /required/);
  });

  // The same defect as the test above, differing only in that nothing renames the field. It was
  // NOT reported: the required loop skipped any field whose rust name appeared in the spec's
  // properties, on the premise that the property loop had already reported it — true only when the
  // wire name differs. For a struct with no rename_all, and for every single-word field under any
  // rule, the two names are equal and the property loop reported nothing, so this whole direction
  // was off. Omitting `idempotency_key` is a serde "missing field" failure, not a default.
  it("reports a handler-required field the spec makes optional when nothing renames it", () => {
    const root = widgetFixture({
      derive: "#[serde(deny_unknown_fields)]",
      fields: "    quantity_consumed_milli: i64,\n    idempotency_key: String,",
      required: ["quantity_consumed_milli"],
      properties: "quantity_consumed_milli: { type: integer }, idempotency_key: { type: string }",
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(resolved, 1);
    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /idempotency_key is required by the handler/);
  });

  // The double-report the suppression exists to prevent must still be suppressed: with
  // rename_all = camelCase the property loop already names `idempotency_key` as unreachable, and
  // the required loop must not say the same thing twice.
  it("does not report the renamed-field mismatch twice, once from each side", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,\n    idempotency_key: String,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }, idempotency_key: { type: string }",
    });

    const { findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /spec property "idempotency_key" is not a field/);
  });

  it("treats an Option field as optional rather than a missing required entry", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,\n    memo: Option<String>,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }, memo: { type: string, nullable: true }",
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 1);
  });

  // Added during implementation, not during the RED phase: the first working resolver reported
  // `spec property "ref" is not a field of AmendCloseBody` against
  // backend/crates/attendance/rest/src/lib.rs:1089, which declares `r#ref`. serde publishes a raw
  // identifier under its bare name, so the field was real and the finding was false. Dropping a
  // field is the same degradation the anchors guard, caught here in its loud direction.
  it("reads a raw-identifier field under the name serde publishes it as", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,\n    #[serde(default)]\n    r#ref: Option<String>,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }, ref: { type: string, nullable: true }",
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 1);
  });

  it("skips a struct without deny_unknown_fields instead of counting it as compared", () => {
    // Without deny_unknown_fields an undocumented field is a maybe, not a 422. Undecidable
    // operations must land in `skipped`; folding them into `resolved` would inflate the floor
    // and let the resolver degrade unnoticed.
    const root = widgetFixture({
      derive: '#[serde(rename_all = "camelCase")]',
      fields: "    quantity_consumed_milli: i64,",
      required: ["quantity_consumed_milli"],
      properties: "quantity_consumed_milli: { type: integer }",
    });

    const { resolved, skipped, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 0);
    assert.equal(skipped, 1);
  });

  it("flags an anchor operation that stops resolving instead of silently comparing less", () => {
    // The failure mode this assertion exists for: five separate resolver bugs each exited 0 with
    // findings: 0. A count floor cannot catch a single-operation drop, so named anchors must.
    const inventoryPath = "backend/crates/inventory/rest/src/lib.rs";
    const inventory = readFileSync(join(repoRoot, inventoryPath), "utf8");
    const singleMethod = inventory.replace(
      `        .route(
            INVENTORY_ITEM_CONSUMPTIONS_PATH_TEMPLATE,
            get(list_consumptions).post(consume_item),
        )`,
      "        .route(INVENTORY_ITEM_CONSUMPTIONS_PATH_TEMPLATE, get(list_consumptions))",
    );
    assert.notEqual(
      singleMethod,
      inventory,
      `fixture anchor no longer matches ${inventoryPath}; update the multi-method route replacement`,
    );

    const root = fixture({
      "backend/openapi/openapi.yaml": readFileSync(join(repoRoot, "backend/openapi/openapi.yaml"), "utf8"),
      "backend/crates/equipment/rest/src/lib.rs": readFileSync(
        join(repoRoot, "backend/crates/equipment/rest/src/lib.rs"),
        "utf8",
      ),
      [inventoryPath]: singleMethod,
    });

    const { unresolvedAnchors } = evaluateRequestBodyContract({ repoRoot: root });

    assert.ok(
      unresolvedAnchors.includes(consumptionsAnchor),
      `expected ${consumptionsAnchor} to be reported unresolved, got ${JSON.stringify(unresolvedAnchors)}`,
    );
    assert.ok(
      !unresolvedAnchors.includes(receiptsAnchor),
      `single-line routes must still resolve, got ${JSON.stringify(unresolvedAnchors)}`,
    );
    assert.ok(
      !unresolvedAnchors.includes(handoverAnchor),
      `wrapped-const routes must still resolve, got ${JSON.stringify(unresolvedAnchors)}`,
    );
  });

  it("resolves every anchor and stays above the floor against this repository", () => {
    const { resolved, unresolvedAnchors } = evaluateRequestBodyContract({ repoRoot });

    assert.deepEqual(unresolvedAnchors, []);
    assert.ok(resolved >= 106, `resolver degraded: expected at least 106 resolved operations, got ${resolved}`);
  });

  it("resolves a string-literal .route() JSON body, not only a path const", () => {
    const root = fixture({
      "backend/crates/widget/rest/src/lib.rs": widgetLiteralCrate({
        derive: camelDeny,
        fields: "    quantity_consumed_milli: i64,",
      }),
      "backend/openapi/openapi.yaml": widgetSpec({
        required: ["quantityConsumedMilli"],
        properties: "quantityConsumedMilli: { type: integer }",
      }),
    });

    const { resolved, skipped, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 1, "string-literal .route() must bind the same deny_unknown_fields body as a path const");
    assert.equal(skipped, 0);
  });

  it("resolves the named string-literal .route() bodies against this repository", () => {
    const report = evaluateRequestBodyContract({ repoRoot });

    assert.equal(LITERAL_PATH_ANCHORS.length, 4);
    assert.deepEqual(
      report.unresolvedLiteralAnchors,
      [],
      `string-literal .route() JSON bodies still undecidable: ${JSON.stringify(report.unresolvedLiteralAnchors)}`,
    );
  });

  it("exits 1 naming the floor when the resolver finds nothing to compare", () => {
    // Zero resolved operations is the shape every one of the resolver bugs took: exit 0,
    // findings 0, nothing compared.
    const root = fixture({
      "backend/openapi/openapi.yaml": widgetSpec({
        required: ["quantityConsumedMilli"],
        properties: "quantityConsumedMilli: { type: integer }",
      }),
    });

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(`${result.stdout}${result.stderr}`, /resolved 0 /);
  });

  // The floor's contribution to the EXIT CODE had never been observed. The test above exits 1
  // through `unresolvedAnchors` — its fixture carries no rust at all, so all three anchors are
  // unresolved — and deleting `|| belowFloor` from the exit condition leaves that test green.
  // Found by mutation during the simplification pass, which is late: the floor was the one branch
  // of this gate whose red nobody had seen, and an unproven branch is what this gate exists to
  // find in other people's code.
  //
  // A resolver that keeps every named anchor alive while comparing a third of the surface is the
  // precise degradation the floor exists for, so it is isolated here: all three anchors resolve,
  // findings are zero, and the floor is the only thing left that can fail the run.
  it("exits 1 on the floor alone, with every anchor resolved and no findings", () => {
    const root = fixture(Object.fromEntries([
      "backend/openapi/openapi.yaml",
      "backend/crates/equipment/rest/src/lib.rs",
      "backend/crates/inventory/rest/src/lib.rs",
    ].map((file) => [file, readFileSync(join(repoRoot, file), "utf8")])));

    const { findings, unresolvedAnchors } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, [], "the isolating fixture must produce no findings of its own");
    assert.deepEqual(unresolvedAnchors, [], "every anchor must resolve, or this is the anchor test again");

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    // If these two crates ever cover the floor on their own the status assertion fails loudly
    // rather than passing for the wrong reason.
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /below the floor of \d+/);
  });

  // The exit-0 branch had never executed. While the spec still published snake_case this gate
  // could not pass, and an unpassable gate is the meta-finding's sharper case: it occupies its
  // slot and reads as coverage. The floor of 106 resolved operations means only the real
  // repository can reach this branch — no fixture is large enough — so the assertion lives here.
  it("exits 0 stating what it compared, against this repository", () => {
    const result = spawnSync(process.execPath, [cli, repoRoot], { encoding: "utf8" });

    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
    assert.match(result.stdout, /request body contract gate passed \(resolved \d+, skipped \d+\)/);
  });

  it("lets an explicit #[serde(rename)] override rename_all", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: '    #[serde(rename = "qty")]\n    quantity_consumed_milli: i64,',
      required: ["qty"],
      properties: "qty: { type: integer }",
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 1);
  });

  it("treats #[serde(default)] on a non-Option field as optional", () => {
    // Two things are load-bearing in this fixture. The Option case above short-circuits before
    // `hasDefault` is ever read, so the field must not be an Option; and the name must be
    // multi-word, or the rust-name escape hatch in the required loop suppresses the finding for
    // an unrelated reason and the assertion proves nothing.
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,\n    #[serde(default)]\n    memo_text: String,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }, memoText: { type: string }",
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 1);
  });

  // 32 of the 284 json request bodies in backend/openapi/openapi.yaml are inline rather than
  // $ref — PATCH /api/work-orders/{workOrderId}/priority and POST /api/v1/hr/exit-cases among
  // them — and the inline branch of jsonRequestSchema had never been exercised. A regression
  // there is silent: the operation lands in `skipped`, and skipped operations are not compared.
  it("resolves and compares an inline requestBody schema, not only a $ref", () => {
    const root = fixture({
      "backend/crates/widget/rest/src/lib.rs": widgetCrate({
        derive: camelDeny,
        fields: "    quantity_consumed_milli: i64,",
      }),
      "backend/openapi/openapi.yaml": `openapi: 3.1.0
info:
  title: Fixture
  version: 0.0.1
paths:
  /api/v1/widgets/{widget_id}/consumptions:
    post:
      operationId: consumeWidget
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              additionalProperties: false
              required: [quantity_consumed_milli]
              properties: { quantity_consumed_milli: { type: integer } }
      responses:
        '200':
          description: ok
`,
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(resolved, 1, "an inline schema must resolve, not land in skipped");
    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /quantity_consumed_milli/);
  });

  // The bare-name fallback exists because a handler may bind a struct declared elsewhere, but it
  // returns the first struct of that name found anywhere in backend/**. `AssignBody` exists in two
  // crates. If the fallback ever outranks the handler's own file the gate compares a real request
  // body against an unrelated struct, and the answer it prints is arbitrary in both directions.
  it("binds the struct in the handler's own file, not a same-named one in another crate", () => {
    const root = fixture({
      "backend/crates/aaa-other/rest/src/lib.rs": `${camelDeny}
struct ConsumeWidgetBody {
    totally_different_field: i64,
}
`,
      "backend/crates/widget/rest/src/lib.rs": widgetCrate({
        derive: camelDeny,
        fields: "    quantity_consumed_milli: i64,",
      }),
      "backend/openapi/openapi.yaml": widgetSpec({
        required: ["quantityConsumedMilli"],
        properties: "quantityConsumedMilli: { type: integer }",
      }),
    });

    const { resolved, findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(findings, []);
    assert.equal(resolved, 1);
  });

  // The test above proves the SAFE adjacent case: when the handler's own file declares the struct,
  // the keyed lookup wins. The hazard its own comment names — the fallback firing on an AMBIGUOUS
  // name — was never exercised, and the fallback was wrong there. Measured on this repository:
  // `AssignBody` (attendance vs facilities) and `ListQuery` (attendance vs orgchange) each have two
  // definitions with DIVERGENT `rename_all`, so the first-match pick is decided by directory
  // traversal order. Below, the decoy sorts first, carries no `rename_all`, and therefore publishes
  // the snake_case name the spec publishes; the struct the handler really binds is camelCase, so
  // the spec is a guaranteed 422. Before the uniqueness guard this returned `resolved: 1,
  // findings: []` — the gate counted the operation as compared, fed the count toward the floor and
  // the anchors, and reported green on the exact defect it exists to catch.
  it("refuses to guess between same-named structs instead of comparing an arbitrary one", () => {
    const decoy = `#[serde(deny_unknown_fields)]
struct ConsumeWidgetBody {
    quantity_consumed_milli: i64,
}
`;
    const bound = `${camelDeny}
struct ConsumeWidgetBody {
    quantity_consumed_milli: i64,
}
`;
    // The handler's file declares neither struct, so only the bare name can resolve it.
    const handlerFile = `${widgetCrate({ derive: camelDeny, fields: "    quantity_consumed_milli: i64," })
      .replace(`#[derive(Debug, Deserialize)]\n${camelDeny}\nstruct ConsumeWidgetBody {\n    quantity_consumed_milli: i64,\n}\n`, "")}
mod body;
`;
    assert.ok(!handlerFile.includes("struct ConsumeWidgetBody"), "the handler's file must not declare the struct");

    const files = {
      "backend/crates/aaa-other/rest/src/lib.rs": decoy,
      "backend/crates/widget/rest/src/body.rs": bound,
      "backend/crates/widget/rest/src/lib.rs": handlerFile,
      "backend/openapi/openapi.yaml": widgetSpec({
        required: ["quantity_consumed_milli"],
        properties: "quantity_consumed_milli: { type: integer }",
      }),
    };

    const { resolved, skipped, findings } = evaluateRequestBodyContract({ repoRoot: fixture(files) });

    assert.deepEqual(findings, [], "an undecidable binding must not be reported as a mismatch either");
    assert.equal(resolved, 0, "an ambiguous bare name must not count as an operation this gate compared");
    assert.equal(skipped, 1, "it belongs in skipped, where the floor and the anchors can see it");

    // And the guard must not cost the unambiguous case: delete the decoy and the same fixture
    // resolves, and goes red on the mismatch the false green was hiding.
    delete files["backend/crates/aaa-other/rest/src/lib.rs"];
    const unique = evaluateRequestBodyContract({ repoRoot: fixture(files) });

    assert.equal(unique.resolved, 1, "a unique bare name must still resolve through the fallback");
    // One finding, not two: the renamed-field double-report suppression covers the required[]
    // side, exactly as the "does not report the renamed-field mismatch twice" test asserts.
    assert.deepEqual(unique.findings.map((finding) => finding.message), [
      'spec property "quantity_consumed_milli" is not a field of ConsumeWidgetBody (deny_unknown_fields => 422)',
    ]);
  });

  it("fails closed when a request struct exists only inside an unexpanded macro", () => {
    const authored = `#[derive(Debug, Deserialize)]
${camelDeny}
struct ConsumeWidgetBody {
    quantity_consumed_milli: i64,
}
`;
    const source = widgetCrate({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,",
    });
    assert.ok(source.includes(authored), "macro fixture no longer matches the request struct");
    const macroDefined = source.replace(authored, `macro_rules! define_consume_widget_body {
    () => {
        #[derive(Debug, Deserialize)]
        ${camelDeny}
        struct ConsumeWidgetBody {
            quantity_consumed_milli: i64,
        }
    };
}
define_consume_widget_body!();
`);
    const root = fixture({
      "backend/crates/widget/rest/src/lib.rs": macroDefined,
      "backend/openapi/openapi.yaml": widgetSpec({
        required: ["quantityConsumedMilli"],
        properties: "quantityConsumedMilli: { type: integer }",
      }),
    });

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.resolved, 0);
    assert.equal(report.skipped, 1);
    assert.equal(report.observedRegister.body[0].reason, "rust_struct_not_strict");
  });
});

// Shared regression fixture pattern for the prototype-chain false-resolve class (console-i91 /
// ann-critic on check-openapi-refs). Empty prototype-ful maps answer true for these keys under
// `obj[key] !== undefined` / `obj?.[key]`; own()/hasOwnKey must not.
describe("own-property helper rejects Object.prototype keys", () => {
  it("own() and hasOwnKey() return nothing for every hostile key on an empty map", () => {
    const empty = {};
    for (const key of PROTOTYPE_CHAIN_KEYS) {
      assert.equal(own(empty, key), undefined, `own empty[${key}]`);
      assert.equal(hasOwnKey(empty, key), false, `hasOwnKey empty[${key}]`);
      // Differential: the defect class is still live on plain index / !== undefined.
      assert.notEqual(empty[key], undefined, `control: plain index still sees ${key}`);
    }
  });

  it("own() still returns authored own properties", () => {
    const map = { Todo: { type: "object" }, constructor: { type: "string" } };
    assert.deepEqual(own(map, "Todo"), { type: "object" });
    assert.deepEqual(own(map, "constructor"), { type: "string" });
    assert.equal(own(map, "toString"), undefined);
  });
});

describe("request-body contract rejects prototype-chain schema resolution", () => {
  // Same class as openapi-refs hasTarget: js-yaml component maps inherit Object.prototype, so
  // `components.schemas?.[name]` answered Function for constructor/toString/__proto__.
  it("jsonRequestSchema resolves nothing through the JS prototype chain", () => {
    for (const key of ["constructor", "toString", "__proto__"]) {
      const document = yaml.load(`openapi: 3.1.0
info: { title: Fixture, version: 0.0.1 }
paths:
  /api/v1/widgets/{widget_id}/consumptions:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/${key}'
components:
  schemas:
    ConsumeWidgetRequest:
      type: object
      properties: { quantityConsumedMilli: { type: integer } }
`);
      assert.equal(
        jsonRequestSchema(document, "/api/v1/widgets/{widget_id}/consumptions", "post"),
        null,
        `dangling $ref to ${key} must not resolve via the prototype chain`,
      );
    }
  });

  // Proven fail-open before the fix: a rust field named `constructor` with a rename made
  // `schema.properties[field.name] !== undefined` true via Object.prototype, which suppressed
  // the handler-required finding (0 findings). Own-property lookup reports it.
  it("does not treat Object.prototype names as present in schema.properties", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields:
        '    #[serde(rename = "quantityConsumedMilli")]\n    constructor: i64,',
      required: [],
      properties: "quantityConsumedMilli: { type: integer }",
    });

    const { findings } = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(findings.length, 1, JSON.stringify(findings, null, 2));
    assert.match(findings[0].message, /constructor is required by the handler/);
  });
});

// The wire name is the whole comparison: get it wrong and the gate reads a real mismatch as a
// match. The expectations below are not derived — they are the fixture table from serde's own
// `rename_fields` test in
// ~/.cargo/registry/src/index.crates.io-*/serde_derive_internals-0.29.1/src/case.rs, whose
// `apply_to_field` is the function actually deciding what the server accepts.
describe("serde rename_all reproduction", () => {
  const cases = [
    // field           UPPERCASE      PascalCase    camelCase     kebab-case      SCREAMING-KEBAB-CASE
    ["outcome", "OUTCOME", "Outcome", "outcome", "outcome", "OUTCOME"],
    ["very_tasty", "VERY_TASTY", "VeryTasty", "veryTasty", "very-tasty", "VERY-TASTY"],
    ["a", "A", "A", "a", "a", "A"],
    ["z42", "Z42", "Z42", "z42", "z42", "Z42"],
  ];

  for (const [field, upper, pascal, camel, kebab, screamingKebab] of cases) {
    it(`renames ${field} the way serde does`, () => {
      // `None | LowerCase | SnakeCase => field.to_owned()` — identity, underscores intact. An
      // earlier `words.join("").toLowerCase()` produced `verytasty`, which would have read a
      // spec publishing `very_tasty` as a mismatch, and a spec publishing `verytasty` — a
      // guaranteed 422 under deny_unknown_fields — as correct.
      assert.equal(renameField(field, null), field);
      assert.equal(renameField(field, "snake_case"), field);
      assert.equal(renameField(field, "lowercase"), field);
      // `UpperCase => field.to_ascii_uppercase()` keeps the underscores; it is not a separate
      // rule from SCREAMING_SNAKE_CASE, which is the same expression in serde.
      assert.equal(renameField(field, "UPPERCASE"), upper);
      assert.equal(renameField(field, "SCREAMING_SNAKE_CASE"), upper);
      assert.equal(renameField(field, "PascalCase"), pascal);
      assert.equal(renameField(field, "camelCase"), camel);
      assert.equal(renameField(field, "kebab-case"), kebab);
      assert.equal(renameField(field, "SCREAMING-KEBAB-CASE"), screamingKebab);
    });
  }
});

describe("serde enum-variant reproduction", () => {
  const cases = [
    ["HttpRequest", null, "HttpRequest"],
    ["HttpRequest", "PascalCase", "HttpRequest"],
    ["HttpRequest", "camelCase", "httpRequest"],
    ["HttpRequest", "snake_case", "http_request"],
    ["HttpRequest", "SCREAMING_SNAKE_CASE", "HTTP_REQUEST"],
    ["HttpRequest", "kebab-case", "http-request"],
    ["HttpRequest", "SCREAMING-KEBAB-CASE", "HTTP-REQUEST"],
    ["HttpRequest", "lowercase", "httprequest"],
    ["HttpRequest", "UPPERCASE", "HTTPREQUEST"],
  ];

  it("uses serde's variant rules rather than its different field rules", () => {
    for (const [variant, style, expected] of cases) {
      assert.equal(renameVariant(variant, style), expected, `${variant} under ${style}`);
    }
    assert.equal(renameVariant("HTTPServer", "snake_case"), "h_t_t_p_server");
  });
});

describe("request body enum-variant contract", () => {
  it("compares a qualified Option enum through a nullable oneOf $ref", () => {
    const root = enumFixture({
      fieldType: "Option<crate::WidgetMode>",
      property: `{ oneOf: [
          { $ref: '#/components/schemas/WidgetMode' },
          { type: 'null' }
        ] }`,
      extraSchemas: `    WidgetMode:
      type: string
      enum: [fast_mode, safe_mode]`,
    });

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumCandidates, 1);
    assert.equal(report.enumResolved, 1);
    assert.equal(report.enumSkipped, 0);
    assert.deepEqual(report.findings, []);
  });

  it("compares a qualified Option enum through a JSON Schema type null union", () => {
    const root = enumFixture({
      fieldType: "Option<crate::WidgetMode>",
      property: `{ type: [string, 'null'], enum: [fast_mode, safe_mode, null] }`,
    });

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumCandidates, 1);
    assert.equal(report.enumResolved, 1);
    assert.equal(report.enumSkipped, 0);
    assert.deepEqual(report.findings, []);
  });

  it("reports a variant the spec advertises but serde rejects", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({ property: "{ type: string, enum: [fast_mode, safe_mode, turbo_mode] }" }),
    });

    assert.equal(report.enumResolved, 1);
    assert.equal(report.findings.length, 1, JSON.stringify(report.findings, null, 2));
    assert.match(report.findings[0].message, /spec-only enum variant.*turbo_mode/);
  });

  it("reports a variant serde accepts but the spec omits", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({ property: "{ type: string, enum: [fast_mode] }" }),
    });

    assert.equal(report.enumResolved, 1);
    assert.equal(report.findings.length, 1, JSON.stringify(report.findings, null, 2));
    assert.match(report.findings[0].message, /Rust-only enum variant.*safe_mode/);
  });

  it("does not let a top-level line comment hide the following Rust enum variant", () => {
    const root = liveSourceFixture();
    const rustPath = join(root, "backend/crates/evaluation/domain/src/lib.rs");
    const rustSource = readFileSync(rustPath, "utf8");
    const commentedRust = rustSource.replace(
      "    Regular,\n    Probation,",
      "    Regular,\n    // A top-level comment must not consume the next variant.\n    Probation,",
    );
    assert.notEqual(commentedRust, rustSource, "CycleKind fixture no longer matches the reviewed source");
    writeFileSync(rustPath, commentedRust);

    const openapiPath = join(root, "backend/openapi/openapi.yaml");
    const openapi = readFileSync(openapiPath, "utf8");
    const narrowedOpenapi = openapi.replace(
      "    EvaluationCycleKind:\n      type: string\n      enum: [REGULAR, PROBATION]",
      "    EvaluationCycleKind:\n      type: string\n      enum: [REGULAR]",
    );
    assert.notEqual(narrowedOpenapi, openapi, "EvaluationCycleKind fixture no longer matches OpenAPI");
    writeFileSync(openapiPath, narrowedOpenapi);

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.deepEqual(
      {
        population: report.population,
        resolved: report.resolved,
        skipped: report.skipped,
        enumCandidates: report.enumCandidates,
        enumResolved: report.enumResolved,
        enumSkipped: report.enumSkipped,
      },
      {
        population: 299,
        resolved: 124,
        skipped: 175,
        enumCandidates: 50,
        enumResolved: 20,
        enumSkipped: 30,
      },
    );
    assert.deepEqual(report.registerFindings, []);
    assert.deepEqual(report.findings, [{
      operation: "POST /api/v1/evaluation/cycles",
      message: 'Rust-only enum variant "PROBATION" for kind',
    }]);

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /Rust-only enum variant "PROBATION" for kind/);
  });

  it("does not let a char literal close an outer Rust enum body", () => {
    assertLiveCycleKindProbationFinding(liveCycleKindBoundaryFixture("'}'"));
  });

  it("does not let a byte-char literal close an outer Rust enum body", () => {
    assertLiveCycleKindProbationFinding(liveCycleKindBoundaryFixture("b'}'"));
  });

  it("discovers no enum from an unexpanded macro_rules token tree at module scope", () => {
    assertLiveCycleKindProbationFinding(liveCycleKindScopeDecoyFixture(`macro_rules! cycle_kind_decoy {
    () => {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        enum CycleKind { Regular }
    };
}`));
  });

  it("discovers no cfg-disabled enum nested in a function body", () => {
    assertLiveCycleKindProbationFinding(liveCycleKindScopeDecoyFixture(`fn cycle_kind_decoy() {
    #[cfg(any())]
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum CycleKind { Regular }
}`));
  });

  it("discovers no cfg-disabled enum nested in a const initializer block", () => {
    assertLiveCycleKindProbationFinding(liveCycleKindScopeDecoyFixture(`const CYCLE_KIND_DECOY: () = {
    #[cfg(any())]
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    enum CycleKind { Regular }
};`));
  });

  it("discovers no cfg-disabled enum nested in an impl method body", () => {
    assertLiveCycleKindProbationFinding(liveCycleKindScopeDecoyFixture(`struct CycleKindDecoy;
impl CycleKindDecoy {
    fn decoy() {
        #[cfg(any())]
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        enum CycleKind { Regular }
    }
}`));
  });

  it("discovers no enum from async, closure, trait, or static initializer bodies", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode { FastMode, SafeMode }

fn nested_decoys() {
    let _future = async {
        #[cfg(any())]
        enum WidgetMode { FastMode }
    };
    let _closure = || {
        #[cfg(any())]
        enum WidgetMode { FastMode }
    };
}

trait DecoyTrait {
    fn defaulted() {
        #[cfg(any())]
        enum WidgetMode { FastMode }
    }
}

static DECOY: () = {
    #[cfg(any())]
    enum WidgetMode { FastMode }
};
`,
        property: "{ type: string, enum: [fast_mode] }",
      }),
    });

    assert.equal(report.enumResolved, 1);
    assert.deepEqual(report.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);

  });

  it("resolves an enum through its explicit inline-module path", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        fieldType: "modes::WidgetMode",
        enumSource: `mod modes {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}

mod decoy {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, OtherMode }
}
`,
      }),
    });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, []);
  });

  it("resolves a grouped import from an inline module without guessing another module", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `mod modes {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}

mod decoy {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, OtherMode }
}

use decoy::WidgetMode as DecoyMode;
use modes::{WidgetMode};
`,
      }),
    });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, []);
  });

  it("resolves a qualified enum declared in a file-backed module", () => {
    const root = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: "mod modes;\n",
    });
    writeFileSync(join(root, "backend/crates/widget/rest/src/modes.rs"), `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode, SafeMode }
`);

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, []);
  });

  it("follows a file-backed module's path attribute instead of its physical-name decoy", () => {
    const root = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: `#[path = "actual.rs"]
mod modes;
`,
      property: "{ type: string, enum: [fast_mode] }",
    });
    writeFileSync(join(root, "backend/crates/widget/rest/src/actual.rs"), `#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode, SafeMode }
`);
    writeFileSync(join(root, "backend/crates/widget/rest/src/modes.rs"), `#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode }
`);

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);

    const innerAttributeRoot = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: `#[path = "actual.rs"]\nmod modes;\n`,
      property: "{ type: string, enum: [fast_mode] }",
    });
    const lib = join(innerAttributeRoot, "backend/crates/widget/rest/src/lib.rs");
    writeFileSync(lib, `#![allow(dead_code)]\n${readFileSync(lib, "utf8")}`);
    writeWidgetModeModule(innerAttributeRoot, "actual.rs");
    writeWidgetModeModule(innerAttributeRoot, "modes.rs", "FastMode");
    const innerAttributeReport = evaluateRequestBodyContract({ repoRoot: innerAttributeRoot });
    assert.equal(innerAttributeReport.enumResolved, 1, JSON.stringify(innerAttributeReport, null, 2));
    assert.deepEqual(innerAttributeReport.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);

    const orphanRoot = enumFixture({ fieldType: "modes::WidgetMode", enumSource: "" });
    writeWidgetModeModule(orphanRoot, "modes.rs");
    const orphanReport = evaluateRequestBodyContract({ repoRoot: orphanRoot });
    assert.equal(orphanReport.enumResolved, 0, JSON.stringify(orphanReport, null, 2));
    assert.equal(orphanReport.enumSkipped, 1);
    assert.equal(orphanReport.observedRegister.enum[0].reason, "rust_enum_unresolved");
  });

  it("uses Rust's relative path rules in file modules and inline modules", () => {
    const cases = [
      {
        name: "file-module",
        enumSource: "mod outer;\n",
        declarations: [["outer.rs", `#[path = "actual.rs"]\npub mod modes;\n`]],
        actual: "actual.rs",
        decoy: "outer/modes.rs",
      },
      {
        name: "path-remapped-file-with-ordinary-child",
        enumSource: `#[path = "actual.rs"]\nmod outer;\n`,
        declarations: [["actual.rs", "pub mod modes;\n"]],
        actual: "modes.rs",
        decoy: "outer/modes.rs",
      },
      {
        name: "inline-module-inside-path-remapped-file",
        fieldType: "outer::inline::modes::WidgetMode",
        enumSource: `#[path = "actual.rs"]\nmod outer;\n`,
        declarations: [["actual.rs", `pub mod inline {
    #[path = "mode.rs"]
    pub mod modes;
}
`]],
        actual: "inline/mode.rs",
        decoy: "actual/inline/mode.rs",
      },
      {
        name: "inline-module",
        enumSource: `mod outer {
    #[path = "actual.rs"]
    pub mod modes;
}
`,
        declarations: [],
        actual: "outer/actual.rs",
        decoy: "outer/modes.rs",
      },
      {
        name: "path-remapped-inline-module",
        enumSource: `#[path = "thread_files"]
mod outer {
    #[path = "actual.rs"]
    pub mod modes;
}
`,
        declarations: [],
        actual: "thread_files/actual.rs",
        decoy: "outer/modes.rs",
      },
    ];

    for (const sample of cases) {
      const root = enumFixture({
        fieldType: sample.fieldType ?? "outer::modes::WidgetMode",
        enumSource: sample.enumSource,
        property: "{ type: string, enum: [fast_mode] }",
      });
      for (const [path, source] of sample.declarations) {
        const absolute = join(root, `backend/crates/widget/rest/src/${path}`);
        mkdirSync(dirname(absolute), { recursive: true });
        writeFileSync(absolute, source);
      }
      writeWidgetModeModule(root, sample.actual);
      writeWidgetModeModule(root, sample.decoy, "FastMode");

      const report = evaluateRequestBodyContract({ repoRoot: root });

      assert.equal(report.enumResolved, 1, `${sample.name}: ${JSON.stringify(report, null, 2)}`);
      assert.deepEqual(report.findings, [{
        operation: widgetOperation,
        message: 'Rust-only enum variant "safe_mode" for mode',
      }], sample.name);
    }
  });

  it("ignores path-shaped comments, strings, and unrelated attributes", () => {
    const root = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: `// #[path = "modes.rs"] mod modes;
const PATH_DECOY: &str = "#[path = \\"modes.rs\\"] mod modes;";
#[doc = "#[path = \\"modes.rs\\"]"]
#[path = "actual.rs"]
mod modes;
`,
      property: "{ type: string, enum: [fast_mode] }",
    });
    writeWidgetModeModule(root, "actual.rs");
    writeWidgetModeModule(root, "modes.rs", "FastMode");

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);
  });

  it("propagates cfg uncertainty through a path-remapped file module", () => {
    const root = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: `#[cfg(any())]
#[path = "actual.rs"]
mod modes;
`,
    });
    writeWidgetModeModule(root, "actual.rs");

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 0, JSON.stringify(report, null, 2));
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous");
  });

  it("propagates inner cfg_attr uncertainty from a path target with valid rustc configurations", () => {
    const root = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: `#[path = "actual.rs"]
mod modes;

#[cfg(feature = "hostile")]
macro_rules! hostile_modes {
    () => {
        mod modes {
            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "snake_case")]
            pub enum WidgetMode { FastMode, OtherMode }
        }
    };
}
#[cfg(feature = "hostile")]
hostile_modes!();
`,
      property: "{ type: string, enum: [fast_mode, safe_mode] }",
    });
    writeFileSync(join(root, "backend/crates/widget/rest/src/actual.rs"), `#![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode, SafeMode }
`);

    const controlRoot = join(root, "rustc-inner-attribute-control");
    mkdirSync(controlRoot, { recursive: true });
    writeFileSync(join(controlRoot, "actual.rs"), `#![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]
pub enum WidgetMode { FastMode, SafeMode }
`);
    const controlSource = join(controlRoot, "main.rs");
    writeFileSync(controlSource, `#[path = "actual.rs"]
mod modes;

#[cfg(feature = "hostile")]
macro_rules! hostile_modes {
    () => {
        mod modes {
            pub enum WidgetMode { FastMode, OtherMode }
        }
    };
}
#[cfg(feature = "hostile")]
hostile_modes!();

fn main() {
    let _ = modes::WidgetMode::FastMode;
    #[cfg(not(feature = "hostile"))]
    let _ = modes::WidgetMode::SafeMode;
    #[cfg(feature = "hostile")]
    let _ = modes::WidgetMode::OtherMode;
}
`);
    const compile = (name, cfg = []) => spawnSync("rustc", [
      "--edition=2021",
      "--crate-name",
      `request_body_inner_${name}`,
      ...cfg,
      controlSource,
      "-o",
      join(controlRoot, name),
    ], { encoding: "utf8" });
    const defaultBuild = compile("default");
    assert.equal(defaultBuild.status, 0, `${defaultBuild.stdout}${defaultBuild.stderr}`);
    const hostileBuild = compile("hostile", ["--cfg", 'feature="hostile"']);
    assert.equal(hostileBuild.status, 0, `${hostileBuild.stdout}${hostileBuild.stderr}`);

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 0, JSON.stringify(report, null, 2));
    assert.equal(report.enumSkipped, 1);
    assert.deepEqual(report.findings, []);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous");
  });

  it("propagates inner cfg_attr after Rust file shebang and BOM preludes", () => {
    const preludes = [
      {
        name: "shebang",
        source: "#!/usr/bin/env rustx\n// projected line whitespace\n/* projected block whitespace */\n",
      },
      {
        name: "bom",
        source: "\uFEFF// projected line whitespace\n/* projected block whitespace */\n",
      },
      {
        name: "bom_shebang",
        source: "\uFEFF#!/usr/bin/env rustx\n// projected line whitespace\n/* projected block whitespace */\n",
      },
      {
        name: "shebang_commented_attribute",
        source: "#!/usr/bin/env rustx\n",
        attribute: '#/* projected attribute whitespace */![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]',
      },
    ];

    for (const prelude of preludes) {
      const innerAttribute = prelude.attribute
        ?? '#![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]';
      const root = enumFixture({
        fieldType: "modes::WidgetMode",
        enumSource: `#[path = "actual.rs"]
mod modes;

#[cfg(feature = "hostile")]
macro_rules! hostile_modes {
    () => {
        mod modes {
            #[derive(Debug, Deserialize)]
            #[serde(rename_all = "snake_case")]
            pub enum WidgetMode { FastMode, OtherMode }
        }
    };
}
#[cfg(feature = "hostile")]
hostile_modes!();
`,
        property: "{ type: string, enum: [fast_mode, safe_mode] }",
      });
      writeFileSync(
        join(root, "backend/crates/widget/rest/src/actual.rs"),
        `${prelude.source}${innerAttribute}
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode, SafeMode }
`,
      );

      const controlRoot = join(root, "rustc-file-prelude-control");
      mkdirSync(controlRoot, { recursive: true });
      writeFileSync(
        join(controlRoot, "actual.rs"),
        `${prelude.source}${innerAttribute}
pub enum WidgetMode { FastMode, SafeMode }
`,
      );
      const controlSource = join(controlRoot, "main.rs");
      writeFileSync(controlSource, `#[path = "actual.rs"]
mod modes;

#[cfg(feature = "hostile")]
macro_rules! hostile_modes {
    () => {
        mod modes {
            pub enum WidgetMode { FastMode, OtherMode }
        }
    };
}
#[cfg(feature = "hostile")]
hostile_modes!();

fn main() {
    let _ = modes::WidgetMode::FastMode;
    #[cfg(not(feature = "hostile"))]
    let _ = modes::WidgetMode::SafeMode;
    #[cfg(feature = "hostile")]
    let _ = modes::WidgetMode::OtherMode;
}
`);
      const compile = (configuration, cfg = []) => spawnSync("rustc", [
        "--edition=2021",
        "--crate-name",
        `request_body_prelude_${prelude.name}_${configuration}`,
        ...cfg,
        controlSource,
        "-o",
        join(controlRoot, configuration),
      ], { encoding: "utf8" });
      const defaultBuild = compile("default");
      assert.equal(
        defaultBuild.status,
        0,
        `${prelude.name} default: ${defaultBuild.stdout}${defaultBuild.stderr}`,
      );
      const hostileBuild = compile("hostile", ["--cfg", 'feature="hostile"']);
      assert.equal(
        hostileBuild.status,
        0,
        `${prelude.name} hostile: ${hostileBuild.stdout}${hostileBuild.stderr}`,
      );

      const report = evaluateRequestBodyContract({ repoRoot: root });

      assert.equal(report.enumResolved, 0, `${prelude.name}: ${JSON.stringify(report, null, 2)}`);
      assert.equal(report.enumSkipped, 1, prelude.name);
      assert.deepEqual(report.findings, [], prelude.name);
      assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous", prelude.name);
    }
  });

  it("fails closed on misplaced, duplicate, or unsupported Rust file preludes", () => {
    const conditionalEnum = `#![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode, SafeMode }
`;
    const fileCases = [
      ` #!/usr/bin/env rustx\n${conditionalEnum}`,
      `// shebang is no longer at the start of the file\n#!/usr/bin/env rustx\n${conditionalEnum}`,
      `\uFEFF\uFEFF${conditionalEnum}`,
      ` \uFEFF${conditionalEnum}`,
      `#![allow(dead_code)]\n#!/usr/bin/env rustx\n${conditionalEnum}`,
      `const BEFORE_SHEBANG: () = ();\n#!/usr/bin/env rustx\n${conditionalEnum}`,
    ];
    for (const source of fileCases) {
      const root = enumFixture({
        fieldType: "modes::WidgetMode",
        enumSource: `#[path = "actual.rs"]\nmod modes;\n`,
      });
      writeFileSync(join(root, "backend/crates/widget/rest/src/actual.rs"), source);
      assert.throws(
        () => evaluateRequestBodyContract({ repoRoot: root }),
        /Rust shebang|UTF-8 BOM/,
        source,
      );
    }

    const inlineRoot = enumFixture({
      fieldType: "modes::WidgetMode",
      enumSource: `mod modes {
    #!/usr/bin/env rustx
    ${conditionalEnum}
}
`,
    });
    assert.throws(
      () => evaluateRequestBodyContract({ repoRoot: inlineRoot }),
      /Rust shebang/,
    );
  });

  it("propagates direct, conditional, duplicate, and conflicting inner cfg attributes", () => {
    const innerAttributes = [
      "#![cfg(any())]",
      '#![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]',
      `#![cfg(any())]
#![cfg_attr(feature = "hostile", cfg(not(feature = "hostile")))]`,
    ];
    for (const innerAttribute of innerAttributes) {
      const report = evaluateRequestBodyContract({
        repoRoot: enumFixture({
          fieldType: "modes::WidgetMode",
          enumSource: `mod modes {
    ${innerAttribute}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}
`,
        }),
      });

      assert.equal(report.enumResolved, 0, innerAttribute);
      assert.equal(report.enumSkipped, 1, innerAttribute);
      assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous", innerAttribute);
    }
  });

  it("keeps lint-only inner cfg_attr modules resolvable", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        fieldType: "modes::WidgetMode",
        enumSource: `mod modes {
    #![cfg_attr(test, allow(dead_code))]
    const INNER_ATTRIBUTE_DECOY: &str = "#![cfg(any())]";
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}
`,
        property: "{ type: string, enum: [fast_mode] }",
      }),
    });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);
  });

  it("fails closed on malformed or unsupported inner module attributes", () => {
    const innerAttributes = [
      "#![request_body_transform]",
      '#![path = "actual.rs"]',
      "#![cfg()]",
      '#![cfg_attr(feature = "hostile")]',
      '#![cfg_attr(feature = "hostile", path = "actual.rs")]',
      '#![cfg_attr(feature = "hostile", request_body_transform)]',
    ];
    for (const innerAttribute of innerAttributes) {
      const root = enumFixture({
        fieldType: "modes::WidgetMode",
        enumSource: `mod modes {
    ${innerAttribute}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}
`,
      });
      assert.throws(
        () => evaluateRequestBodyContract({ repoRoot: root }),
        /inner (?:cfg|cfg_attr|module) attribute|unsupported inner module attribute/,
        innerAttribute,
      );
    }
  });

  it("fails closed on unknown, duplicate, conflicting, conditional, or malformed path mappings", () => {
    const cases = [
      `#[request_body_transform]
#[path = "actual.rs"]
mod modes;
`,
      `#[path = "actual.rs"]
#[path = "other.rs"]
mod modes;
`,
      `#[path = "actual.rs"]
mod modes;
#[path = "other.rs"]
mod modes;
`,
      `#[cfg_attr(any(), path = "actual.rs")]
mod modes;
`,
      `#[path = concat!("actual", ".rs")]
mod modes;
`,
    ];

    for (const enumSource of cases) {
      const root = enumFixture({ fieldType: "modes::WidgetMode", enumSource });
      writeWidgetModeModule(root, "actual.rs");
      writeWidgetModeModule(root, "other.rs");
      assert.throws(
        () => evaluateRequestBodyContract({ repoRoot: root }),
        /path|module (?:attribute|declarations)/,
        enumSource,
      );
    }
  });

  it("leaves missing, non-Rust, and ambiguous ordinary module targets unresolved", () => {
    const cases = [
      {
        enumSource: `#[path = "missing.rs"]\nmod modes;\n`,
        prepare() {},
      },
      {
        enumSource: `#[path = "actual.txt"]\nmod modes;\n`,
        prepare(root) {
          writeFileSync(join(root, "backend/crates/widget/rest/src/actual.txt"), "pub enum WidgetMode { FastMode }");
        },
      },
      {
        enumSource: "mod modes;\n",
        prepare(root) {
          writeWidgetModeModule(root, "modes.rs");
          writeWidgetModeModule(root, "modes/mod.rs");
        },
      },
    ];

    for (const sample of cases) {
      const root = enumFixture({ fieldType: "modes::WidgetMode", enumSource: sample.enumSource });
      sample.prepare(root);
      const report = evaluateRequestBodyContract({ repoRoot: root });
      assert.equal(report.enumResolved, 0, JSON.stringify(report, null, 2));
      assert.equal(report.enumSkipped, 1);
      assert.equal(report.observedRegister.enum[0].reason, "rust_enum_unresolved");
    }
  });

  it("resolves a qualified enum through an imported module alias", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        fieldType: "selected_modes::WidgetMode",
        enumSource: `mod modes {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}
use modes as selected_modes;
`,
      }),
    });

    assert.equal(report.enumResolved, 1, JSON.stringify(report, null, 2));
    assert.deepEqual(report.findings, []);
  });

  it("does not resolve an unknown qualified path by bare-name fallback", () => {
    const root = enumFixture({
      fieldType: "unknown_crate::WidgetMode",
      enumSource: "",
    });
    const external = join(root, "backend/crates/other/domain/src/lib.rs");
    mkdirSync(dirname(external), { recursive: true });
    writeFileSync(external, `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetMode { FastMode, SafeMode }
`);

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_unresolved");
  });

  it("fails closed on a cfg-conditional module-scope enum", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `#[cfg(any())]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode { FastMode, SafeMode }
`,
      }),
    });

    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous");
  });

  it("propagates cfg uncertainty from an inline module to its enum", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        fieldType: "modes::WidgetMode",
        enumSource: `#[cfg(any())]
mod modes {
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum WidgetMode { FastMode, SafeMode }
}
`,
      }),
    });

    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous");
  });

  it("fails closed when a field enum exists only inside an unexpanded macro", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `macro_rules! define_widget_mode {
    () => {
        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum WidgetMode { FastMode, SafeMode }
    };
}
define_widget_mode!();
`,
      }),
    });

    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_unresolved");
  });

  it("keeps the outer enum boundary across every supported Rust lexical form", () => {
    const bodies = [
      ["line, doc, and nested block comments", `
    /// docs containing a closing brace } and comma,
    FastMode = 0, // a closing brace } stays inside the comment
    /** docs containing } /* and a nested } comment */ still docs */
    SafeMode,
`],
      ["cooked and byte strings with escapes", `
    FastMode = {
        const COOKED: &str = "\\\" } // comma, /* block */";
        const BYTES: &[u8] = b"\\\" } // comma, /* block */";
        let _ = (COOKED, BYTES);
        0
    },
    SafeMode,
`],
      ["raw and raw byte strings", `
    FastMode = {
        const RAW: &str = r##"quote " then } and "# still raw"##;
        const RAW_BYTES: &[u8] = br##"quote " then } and "# still raw"##;
        let _ = (RAW, RAW_BYTES);
        0
    },
    SafeMode,
`],
      ["escaped characters and nested delimiters", `
    FastMode = {
        const CLOSE: char = '\\u{7d}';
        const BYTE_CLOSE: u8 = b'\\x7d';
        let _ = ([{ (CLOSE, BYTE_CLOSE) }],);
        0
    },
    SafeMode,
`],
    ];

    for (const [label, body] of bodies) {
      const report = evaluateRequestBodyContract({
        repoRoot: enumFixture({
          enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
${body}}
`,
          property: "{ type: string, enum: [fast_mode] }",
        }),
      });

      assert.equal(report.enumResolved, 1, label);
      assert.deepEqual(report.findings, [{
        operation: widgetOperation,
        message: 'Rust-only enum variant "safe_mode" for mode',
      }], label);
    }
  });

  it("does not discover enum items fabricated inside comments or literals", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `const COOKED_FAKE: &str = "pub enum WidgetMode { Phantom, }";
const RAW_FAKE: &str = r##"pub enum WidgetMode { Phantom = '}', }"##;
/* pub enum WidgetMode { Phantom, } */
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
    FastMode,
    SafeMode,
}
`,
        property: "{ type: string, enum: [fast_mode] }",
      }),
    });

    assert.equal(report.enumResolved, 1);
    assert.deepEqual(report.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);
  });

  it("fails closed on lexically invalid or unbalanced Rust item bodies", () => {
    const invalidEnums = [
      ["unterminated block comment", `#[derive(Deserialize)]
enum WidgetMode { FastMode, /* never closed
`],
      ["unterminated cooked string", `#[derive(Deserialize)]
enum WidgetMode { FastMode = "never closed, SafeMode }
`],
      ["mismatched nested delimiter", `#[derive(Deserialize)]
enum WidgetMode { FastMode = (0], SafeMode }
`],
      ["unterminated character literal", `#[derive(Deserialize)]
enum WidgetMode { FastMode = '} as isize, SafeMode }
`],
    ];

    for (const [label, enumSource] of invalidEnums) {
      assert.throws(
        () => evaluateRequestBodyContract({ repoRoot: enumFixture({ enumSource }) }),
        /cannot scan Rust source/,
        label,
      );
    }
  });

  it("keeps variants visible across line, doc, comma-bearing, and nested block comments", () => {
    const bodies = [
      ["line comment after comma", "    FastMode, // comma, /* text */\n    SafeMode,"],
      ["comma on the next line", "    FastMode // comma, /* text */\n    ,\n    SafeMode,"],
      ["outer doc line", "    FastMode,\n    /// docs, with punctuation\n    SafeMode,"],
      ["nested block doc", "    FastMode,\n    /** docs, /* nested, comment */ still docs */\n    SafeMode,"],
    ];
    for (const [label, body] of bodies) {
      const report = evaluateRequestBodyContract({
        repoRoot: enumFixture({
          enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
${body}
}
`,
          property: "{ type: string, enum: [fast_mode] }",
        }),
      });

      assert.equal(report.enumResolved, 1, label);
      assert.deepEqual(report.findings, [{
        operation: widgetOperation,
        message: 'Rust-only enum variant "safe_mode" for mode',
      }], label);
    }
  });

  it("treats comments as whitespace instead of fabricating joined variant names", () => {
    for (const separator of ["/* comment */", "// comment\n    "]) {
      const report = evaluateRequestBodyContract({
        repoRoot: enumFixture({
          enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
    Fast${separator}Mode,
    SafeMode,
}
`,
        }),
      });

      assert.equal(report.enumResolved, 0);
      assert.equal(report.enumSkipped, 1);
      assert.equal(report.observedRegister.enum[0].reason, "rust_enum_unsupported");
    }
  });

  it("does not read comment syntax or commas inside discriminant literals as enum structure", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
    FastMode = {
        const COOKED: &str = "\\\", // comma, /* block */";
        const RAW: &str = r#"}, // comma, /* block */"#;
        let _ = (COOKED, RAW);
        1
    },
    SafeMode = b',' as isize,
}
`,
        property: "{ type: string, enum: [fast_mode] }",
      }),
    });

    assert.equal(report.enumResolved, 1);
    assert.deepEqual(report.findings, [{
      operation: widgetOperation,
      message: 'Rust-only enum variant "safe_mode" for mode',
    }]);
  });

  it("fails closed on syntax-bearing attributes even when their strings look like comments", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
    FastMode,
    #[doc = r##"quote " then ], } and "# still raw"##]
    SafeMode,
}
`,
      }),
    });

    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_unsupported");
  });

  it("reports a serde enum when OpenAPI leaves the string unrestricted", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({ property: "{ type: string }" }),
    });

    assert.equal(report.enumResolved, 1);
    assert.equal(report.findings.length, 1, JSON.stringify(report.findings, null, 2));
    assert.match(report.findings[0].message, /does not constrain.*WidgetMode/);
  });

  it("registers a String-backed spec enum instead of pretending to compare it", () => {
    const report = evaluateRequestBodyContract({
      repoRoot: enumFixture({ fieldType: "String", enumSource: "" }),
    });

    assert.equal(report.enumCandidates, 1);
    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "string_backed_spec_enum");
  });

  it("registers tagged/data enums and unsupported variant attributes instead of guessing", () => {
    const tagged = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WidgetMode {
    FastMode { limit: i64 },
    SafeMode,
}
`,
      }),
    });
    assert.equal(tagged.observedRegister.enum[0].reason, "tagged_or_data_enum");

    const attributed = evaluateRequestBodyContract({
      repoRoot: enumFixture({
        enumSource: `#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode {
    #[serde(alias = "quick")]
    FastMode,
    SafeMode,
}
`,
      }),
    });
    assert.equal(attributed.observedRegister.enum[0].reason, "rust_enum_unsupported");
  });

  it("refuses an ambiguous bare enum name", () => {
    const root = enumFixture({ enumSource: "mod a;\nmod b;\n" });
    writeFileSync(join(root, "backend/crates/widget/rest/src/a.rs"), `#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode { FastMode, SafeMode }
`);
    writeFileSync(join(root, "backend/crates/widget/rest/src/b.rs"), `#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum WidgetMode { FastMode, OtherMode }
`);

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.enumResolved, 0);
    assert.equal(report.enumSkipped, 1);
    assert.equal(report.observedRegister.enum[0].reason, "rust_enum_ambiguous");
  });

  it("normalizes placeholder names without normalizing literal path bytes", () => {
    const root = enumFixture();
    const specPath = join(root, "backend/openapi/openapi.yaml");
    writeFileSync(
      specPath,
      readFileSync(specPath, "utf8").replaceAll("{widget_id}", "{widgetId}"),
    );

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.population, 1);
    assert.equal(report.resolved, 1);
    assert.equal(report.enumResolved, 1);
  });

  it("fails closed when placeholder normalization makes two operations collide", () => {
    const root = enumFixture();
    const specPath = join(root, "backend/openapi/openapi.yaml");
    const duplicate = `  /api/v1/widgets/{other_id}/consumptions:
    post:
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ConsumeWidgetRequest'
      responses:
        '200': { description: ok }
`;
    writeFileSync(
      specPath,
      readFileSync(specPath, "utf8").replace("paths:\n", `paths:\n${duplicate}`),
    );

    const report = evaluateRequestBodyContract({ repoRoot: root });

    assert.equal(report.population, 2);
    assert.ok(report.findings.some((finding) => /normalized OpenAPI operation collision/.test(finding.message)));
  });

  it("makes enum findings fatal in the CLI", () => {
    const root = enumFixture({ property: "{ type: string, enum: [fast_mode, turbo_mode] }" });
    writeObservedRegister(root);

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /spec-only enum variant.*turbo_mode/);
    assert.match(result.stderr, /Rust-only enum variant.*safe_mode/);
  });

  it("enforces the enum floor independently of body resolution", () => {
    const root = widgetFixture({
      derive: camelDeny,
      fields: "    quantity_consumed_milli: i64,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }",
    });
    writeObservedRegister(root);

    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /enum-resolved 0.*below the floor of 20/);
  });
});

describe("undecidable register is exact and fail-closed", () => {
  function undecidableFixture() {
    return widgetFixture({
      derive: '#[serde(rename_all = "camelCase")]',
      fields: "    quantity_consumed_milli: i64,",
      required: ["quantityConsumedMilli"],
      properties: "quantityConsumedMilli: { type: integer }",
    });
  }

  it("rejects a missing, malformed, or duplicate register", () => {
    const missingRoot = undecidableFixture();
    assert.match(
      evaluateRequestBodyContract({ repoRoot: missingRoot }).registerFindings.join("\n"),
      /missing undecidable register/,
    );

    const malformedRoot = undecidableFixture();
    const malformedPath = join(malformedRoot, "scripts/request-body-contract-undecidable.json");
    mkdirSync(dirname(malformedPath), { recursive: true });
    writeFileSync(malformedPath, "{not json\n");
    assert.match(
      evaluateRequestBodyContract({ repoRoot: malformedRoot }).registerFindings.join("\n"),
      /malformed undecidable register/,
    );

    const duplicateRoot = undecidableFixture();
    const duplicateReport = evaluateRequestBodyContract({ repoRoot: duplicateRoot });
    duplicateReport.observedRegister.body.push({ ...duplicateReport.observedRegister.body[0] });
    const duplicatePath = join(duplicateRoot, "scripts/request-body-contract-undecidable.json");
    mkdirSync(dirname(duplicatePath), { recursive: true });
    writeFileSync(duplicatePath, JSON.stringify(duplicateReport.observedRegister));
    assert.match(
      evaluateRequestBodyContract({ repoRoot: duplicateRoot }).registerFindings.join("\n"),
      /duplicate body register entry/,
    );
  });

  it("rejects unknown reasons, reason drift, stale entries, and equal-count substitution", () => {
    const root = undecidableFixture();
    const report = evaluateRequestBodyContract({ repoRoot: root });
    const register = structuredClone(report.observedRegister);
    register.body[0].reason = "invented_reason";
    const registerPath = join(root, "scripts/request-body-contract-undecidable.json");
    mkdirSync(dirname(registerPath), { recursive: true });
    writeFileSync(registerPath, JSON.stringify(register));
    assert.match(
      evaluateRequestBodyContract({ repoRoot: root }).registerFindings.join("\n"),
      /unknown body undecidable reason/,
    );

    register.body[0] = {
      ...report.observedRegister.body[0],
      operation: "POST /api/v1/widgets/{widget_id}/substituted",
    };
    writeFileSync(registerPath, JSON.stringify(register));
    const substitution = evaluateRequestBodyContract({ repoRoot: root }).registerFindings.join("\n");
    assert.match(substitution, /unregistered body undecidable/);
    assert.match(substitution, /stale body register entry/);

    register.body[0] = { ...report.observedRegister.body[0], handler: "wrong_handler" };
    writeFileSync(registerPath, JSON.stringify(register));
    assert.match(
      evaluateRequestBodyContract({ repoRoot: root }).registerFindings.join("\n"),
      /body register metadata drift/,
    );
  });

  it("makes register drift fatal in the CLI", () => {
    const root = undecidableFixture();
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });

    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.match(result.stderr, /missing undecidable register/);
  });
});

describe("live request body census", () => {
  it("binds the exact source-first body and enum populations to the reviewed register", () => {
    const report = evaluateRequestBodyContract({ repoRoot });

    assert.equal(report.population, 299);
    assert.equal(report.resolved, 124);
    assert.equal(report.skipped, 175);
    assert.equal(report.enumCandidates, 50);
    assert.equal(report.enumResolved, 20);
    assert.equal(report.enumSkipped, 30);
    assert.deepEqual(report.findings, []);
    assert.deepEqual(report.unresolvedAnchors, []);
    assert.deepEqual(report.unresolvedLiteralAnchors, []);
    assert.deepEqual(report.unresolvedEnumAnchors, []);
    assert.deepEqual(report.registerFindings, []);
  });
});

describe("native Company raw-byte request contracts", () => {
  const enroll = "POST /api/v2/companies/enroll";
  const cancel = "POST /api/v2/companies/enrollments/{command_id}/cancel";
  const rest = "backend/crates/identity/rest/src/company.rs";
  const restExports = "backend/crates/identity/rest/src/lib.rs";
  const codec = "backend/crates/identity/application/src/company.rs";
  const owner = "backend/crates/identity/application/src/company_enrollment.rs";
  const exports = "backend/crates/identity/application/src/lib.rs";
  const register = "scripts/request-body-contract-undecidable.json";
  const registerDigest = "7be3cba4991191def9f203d96f28be01b6e384e26f2e1487a958281a3e918577";
  let live;

  // Mutate a copy of the actual mounted sources and composed contract, never an
  // invented JSON extractor or DTO. Restore each mutation even when its oracle
  // fails, so a preceding failure cannot supply another test's expected result.
  function withMutation(mutate, verify) {
    live ??= liveSourceFixture();
    const originals = new Map();
    function edit(path, transform) {
      const absolute = join(live, path);
      const original = readFileSync(absolute, "utf8");
      originals.set(absolute, original);
      const changed = transform(original);
      assert.notEqual(changed, original, `fixture no longer matches ${path}`);
      writeFileSync(absolute, changed);
    }
    try {
      mutate(edit);
      verify(live);
    } finally {
      for (const [absolute, original] of originals) writeFileSync(absolute, original);
      assert.equal(createHash("sha256").update(readFileSync(join(live, register))).digest("hex"), registerDigest);
    }
  }

  function replaceOnce(source, before, after) {
    assert.equal(source.split(before).length, 2, "reviewed mutation must match exactly once");
    return source.replace(before, after);
  }

  function census(report) {
    assert.equal(report.population, 299);
    assert.deepEqual([report.enumCandidates, report.enumResolved, report.enumSkipped], [50, 20, 30]);
  }

  function companyUndecidables(report) {
    return report.observedRegister.body.filter((entry) => [enroll, cancel].includes(entry.operation));
  }

  function assertResolved(root) {
    const report = evaluateRequestBodyContract({ repoRoot: root });
    census(report);
    assert.deepEqual([report.resolved, report.skipped], [124, 175]);
    assert.deepEqual(companyUndecidables(report), []);
    assert.deepEqual(report.findings, []);
    assert.deepEqual(report.registerFindings, []);
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
  }

  function assertSourceRefused(root, operations) {
    const report = evaluateRequestBodyContract({ repoRoot: root });
    census(report);
    assert.deepEqual(companyUndecidables(report).map((entry) => entry.operation).sort(), [...operations].sort());
    assert.deepEqual([report.resolved, report.skipped], [124 - operations.length, 175 + operations.length]);
    assert.deepEqual(report.findings, []);
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    for (const operation of operations) {
      assert.ok(report.registerFindings.includes(`unregistered body undecidable: ${operation}`));
      assert.ok(result.stderr.includes(`unregistered body undecidable: ${operation}`));
    }
  }

  function assertSchemaMismatch(root, operation) {
    const report = evaluateRequestBodyContract({ repoRoot: root });
    census(report);
    // Known source remains classified: a contract disagreement is a finding,
    // not a new undecidable entry which conceals the mismatch in a register.
    assert.deepEqual([report.resolved, report.skipped], [124, 175]);
    assert.deepEqual(companyUndecidables(report), []);
    assert.deepEqual(report.registerFindings, []);
    assert.ok(report.findings.length > 0, "Company schema drift must produce a finding");
    assert.ok(report.findings.every((finding) => finding.operation === operation), JSON.stringify(report.findings));
    const result = spawnSync(process.execPath, [cli, root], { encoding: "utf8" });
    assert.equal(result.status, 1, `${result.stdout}${result.stderr}`);
    assert.ok(result.stderr.includes(`${operation}:`), result.stderr);
  }

  function changeDocument(edit, transform) {
    edit("backend/openapi/openapi.yaml", (source) => {
      const document = yaml.load(source);
      const before = structuredClone(document);
      transform(document);
      assert.notDeepEqual(document, before, "OpenAPI fixture mutation must change its semantic structure");
      return yaml.dump(document, { noRefs: true, lineWidth: -1 });
    });
  }

  function deleteOwned(object, key) {
    assert.ok(Object.hasOwn(object, key), `fixture must own ${key} before deletion`);
    delete object[key];
    assert.equal(Object.hasOwn(object, key), false, `fixture must omit ${key} after deletion`);
  }

  it("proves schema mutation controls reject semantic no-ops and missing delete targets", () => {
    assert.throws(() => withMutation((edit) => changeDocument(edit, () => {}),
      () => assert.fail("a semantic no-op must not reach its oracle")),
    /OpenAPI fixture mutation must change its semantic structure/);
    assert.throws(() => deleteOwned({}, "requestBody"), /fixture must own requestBody before deletion/);
  });

  it("resolves both real handlers without changing the reviewed register or enum census", () => {
    withMutation(() => {}, assertResolved);
  });

  it("ignores unmounted nested handler names instead of selecting their bytes", () => {
    withMutation((edit) => edit(rest, (source) => `${source}\nmod unmounted_company_decoy {
    fn enroll() { let _ = "CompanyEnrollmentV1::from_json_slice"; }
    fn cancel() { let _ = "body.is_empty() || body.as_ref() == b\\\"{}\\\""; }
}\n`), assertResolved);
  });

  for (const [name, transform] of [
    ["missing required nullable Group", (schema) => { schema.required = schema.required.filter((name) => name !== "group_id"); }],
    ["nullable Group broadened to a UUID", (schema) => { schema.properties.group_id = { type: ["string", "null"], format: "uuid" }; }],
    ["unknown members allowed", (schema) => { schema.additionalProperties = true; }],
    ["extra declared member", (schema) => { schema.properties.unowned = { type: "string" }; }],
    ["missing actual name property", (schema) => { deleteOwned(schema.properties, "name"); }],
    ["administrative Account UUID wrong type", (schema) => { schema.properties.administrative_account_id.type = "integer"; }],
    ["administrative Account UUID wrong format", (schema) => { schema.properties.administrative_account_id.format = "email"; }],
    ["administrative Account UUID wrong length", (schema) => { schema.properties.administrative_account_id.maxLength = 37; }],
    ["non-object input", (schema) => { schema.type = "array"; }],
    ["weaker name bound", (schema) => { schema.properties.name.maxLength = 257; }],
    ["noncanonical command UUIDs", (schema) => { deleteOwned(schema.properties.command_id, "pattern"); }],
    ["slug character policy drift", (schema) => { schema.properties.slug.pattern = ".*"; }],
  ]) {
    it(`reports enrollment schema mismatch: ${name}`, () => {
      withMutation((edit) => changeDocument(edit, (document) => {
        transform(document.components.schemas.NativeCompanyEnrollmentInput);
      }), (root) => assertSchemaMismatch(root, enroll));
    });
  }

  for (const [name, transform] of [
    ["optional enrollment body", (body) => { body.required = false; }],
    ["omitted enrollment body requirement", (body) => { deleteOwned(body, "required"); }],
  ]) {
    it(`reports ${name} instead of promising a zero-byte command`, () => {
      withMutation((edit) => changeDocument(edit, (document) => {
        transform(document.paths["/api/v2/companies/enroll"].post.requestBody);
      }), (root) => assertSchemaMismatch(root, enroll));
    });
  }

  for (const [name, transform] of [
    ["required body excludes the zero-byte form", (body) => { body.required = true; }],
    ["missing empty-object bound", (body) => { deleteOwned(body.content["application/json"].schema, "maxProperties"); }],
    ["nonzero empty-object bound", (body) => { body.content["application/json"].schema.maxProperties = 1; }],
    ["unknown members allowed", (body) => { body.content["application/json"].schema.additionalProperties = true; }],
    ["declared nonempty body", (body) => { body.content["application/json"].schema.properties = { memo: { type: "string" } }; }],
    ["non-object body", (body) => { body.content["application/json"].schema.type = "string"; }],
  ]) {
    it(`reports cancellation schema mismatch: ${name}`, () => {
      withMutation((edit) => changeDocument(edit, (document) => {
        transform(document.paths["/api/v2/companies/enrollments/{command_id}/cancel"].post.requestBody);
      }), (root) => assertSchemaMismatch(root, cancel));
    });
  }

  for (const [name, path, before, after, operations = [enroll]] of [
    ["codec allows unknown members", codec, "#[serde(deny_unknown_fields)]", "#[serde(default)]"],
    ["codec makes nullable Group optional", codec, '#[serde(deserialize_with = "required_group")]', "#[serde(default)]"],
    ["required Group parser is replaced", codec, "Option::<String>::deserialize(d)", "let _ = d; Ok(None)"],
    ["codec admits non-object forms", codec, ".deserialize_map(InputObject)", ".deserialize_any(InputObject)"],
    ["codec omits trailing-byte refusal", codec, "        parser.end().map_err(|_| invalid())?;", "        // Trailing bytes no longer checked."],
    ["codec body bound changes", codec, "if input.len() > 4096", "if input.len() > 8192"],
    ["codec drops canonical UUID validation", codec, "if id.hyphenated().to_string() != raw", "if false"],
    ["codec drops control-character validation", codec, "        || name.chars().any(char::is_control)", ""],
    ["owner admits existing Group", owner, "    if input.group_id().is_some() {", "    if false {"],
    ["enrollment reads a larger transport body", rest, "let bytes = match to_bytes(request.into_body(), 4096).await", "let bytes = match to_bytes(request.into_body(), 8192).await"],
    ["enrollment validates different bytes", rest, "CompanyEnrollmentV1::from_json_slice(&bytes)", 'CompanyEnrollmentV1::from_json_slice(b"{}")'],
    ["cancellation removes zero-byte admission", rest, 'body.is_empty() || body.as_ref() == b"{}"', 'body.as_ref() == b"{}"', [cancel]],
    ["cancellation admits whitespace bytes", rest, 'body.as_ref() == b"{}"', 'body.as_ref() == b"{ }"', [cancel]],
    ["cancellation broadens byte admission", rest, 'body.is_empty() || body.as_ref() == b"{}"', "body.len() <= 4096", [cancel]],
    ["cancellation reads unrelated body", rest, "    match to_bytes(request.into_body(), 4096).await", '    match to_bytes(axum::body::Body::from("{}"), 4096).await', [cancel]],
    ["cancellation transport bound changes", rest, "    match to_bytes(request.into_body(), 4096).await", "    match to_bytes(request.into_body(), 8192).await", [cancel]],
    ["canonical type export points elsewhere", exports, "pub use company::CompanyEnrollmentV1;", "pub use org::CompanyEnrollmentV1;"],
    ["codec module is remapped", exports, "mod company;", '#[path = "company.rs"]\nmod company;'],
    ["REST Company module is remapped", restExports, "pub mod company;", '#[path = "company.rs"]\npub mod company;', [enroll, cancel]],
    ["codec module is conditional", exports, "mod company;", "#[cfg(any())]\nmod company;"],
    ["enrollment handler is conditional", rest, "async fn enroll<S>", "#[cfg(any())]\nasync fn enroll<S>"],
    ["cancellation handler is conditional", rest, "async fn cancel<S>", "#[cfg(any())]\nasync fn cancel<S>", [cancel]],
    ["application imports point to a different crate", rest, "use console_identity_application::{", "use unrelated_application::{", [enroll, cancel]],
    ["body reader namespace import is conditional", rest, "use axum::{", "#[cfg(any())]\nuse axum::{", [enroll, cancel]],
    ["body reader import is replaced", rest, "    body::to_bytes,", "    body::collect as to_bytes,", [enroll, cancel]],
  ]) {
    it(`refuses source drift: ${name}`, () => {
      withMutation((edit) => edit(path, (source) => replaceOnce(source, before, after)),
        (root) => assertSourceRefused(root, operations));
    });
  }

  it("does not let routes moved into an unused same-file router establish a binding", () => {
    withMutation((edit) => edit(rest, (source) => {
      const start = source.indexOf("pub fn router<S>");
      const end = source.indexOf("\nfn private(", start);
      assert.ok(start >= 0 && end > start, "canonical Company router fixture no longer matches");
      const original = source.slice(start, end);
      const body = original.indexOf("{\n    Router::new()");
      assert.ok(body >= 0, "canonical Company router body no longer matches");
      const empty = `${original.slice(0, body)}{\n    Router::new().with_state(state)\n}\n`;
      const unused = replaceOnce(original, "pub fn router<S>", "fn unused_company_router<S>");
      return replaceOnce(source, original, `${empty}\n${unused}`);
    }), (root) => assertSourceRefused(root, [enroll, cancel]));
  });

  it("refuses ambiguous duplicate enrollment handler declarations", () => {
    withMutation((edit) => edit(rest, (source) => `${source}\nasync fn enroll<S>() {}\n`),
      (root) => assertSourceRefused(root, [enroll]));
  });

  for (const [operation, path] of [
    [enroll, "/api/v2/companies/enroll"],
    [cancel, "/api/v2/companies/enrollments/{command_id}/cancel"],
  ]) {
    for (const [name, transform] of [
      ["requestBody", (document) => { deleteOwned(document.paths[path].post, "requestBody"); }],
      ["whole operation", (document) => { deleteOwned(document.paths[path], "post"); }],
    ]) {
      it(`keeps ${operation} in the source-first census when its ${name} is deleted`, () => {
        withMutation((edit) => changeDocument(edit, transform), (root) => {
          assertSourceRefused(root, [operation]);
          const report = evaluateRequestBodyContract({ repoRoot: root });
          assert.equal(companyUndecidables(report)[0].reason, "no_openapi_request_body");
        });
      });
    }
    it(`keeps ${operation} in the source-first census after body-limit drift and requestBody deletion`, () => {
      withMutation((edit) => {
        const before = operation === enroll
          ? "let bytes = match to_bytes(request.into_body(), 4096).await"
          : "    match to_bytes(request.into_body(), 4096).await";
        edit(rest, (source) => replaceOnce(source, before, before.replace("4096", "8192")));
        changeDocument(edit, (document) => { deleteOwned(document.paths[path].post, "requestBody"); });
      }, (root) => {
        assertSourceRefused(root, [operation]);
        const report = evaluateRequestBodyContract({ repoRoot: root });
        assert.equal(companyUndecidables(report)[0].reason, "no_openapi_request_body");
      });
    });
  }

  it("refuses Group admission moved after durable preparation", () => {
    withMutation((edit) => edit(owner, (source) => replaceOnce(source,
      `    if input.group_id().is_some() {
        return Err(CompanyEnrollmentError::GroupUnavailable);
    }
    store.prepare(credentials, &input).await?;`,
      `    store.prepare(credentials, &input).await?;
    if input.group_id().is_some() {
        return Err(CompanyEnrollmentError::GroupUnavailable);
    }`)), (root) => assertSourceRefused(root, [enroll]));
  });

  for (const [name, path, decoy, operations] of [
    ["REST axum namespace", rest, "mod axum {}", [enroll, cancel]],
    ["REST application namespace", rest, "mod console_identity_application {}", [enroll, cancel]],
    ["unknown REST glob import", rest, "use unrelated::*;", [enroll, cancel]],
    ["ancestor extern crate alias", restExports, "extern crate unrelated as axum;", [enroll, cancel]],
    ["codec serde_json namespace", codec, "mod serde_json {}", [enroll]],
    ["codec serde namespace", codec, "mod serde {}", [enroll]],
    ["owner type shadow", owner, "struct CompanyEnrollmentV1;", [enroll]],
  ]) {
    it(`refuses namespace replacement: ${name}`, () => {
      withMutation((edit) => edit(path, (source) => `${source}\n${decoy}\n`),
        (root) => assertSourceRefused(root, operations));
    });
  }

  it("does not let an unexpanded copy of the strict codec rescue its mounted replacement", () => {
    withMutation((edit) => edit(codec, (source) => {
      const broken = replaceOnce(source, "#[serde(deny_unknown_fields)]", "#[serde(default)]");
      return `${broken}\nmacro_rules! unexpanded_strict_codec { () => { ${source} }; }\n`;
    }), (root) => assertSourceRefused(root, [enroll]));
  });

  it("does not let an unmounted original cancellation rescue its changed byte guard", () => {
    withMutation((edit) => edit(rest, (source) => {
      const broken = replaceOnce(source, 'body.as_ref() == b"{}"', 'body.as_ref() == b"{ }"');
      return `${broken}\nmod unmounted_original_transport { ${source} }\n`;
    }), (root) => assertSourceRefused(root, [cancel]));
  });
});
