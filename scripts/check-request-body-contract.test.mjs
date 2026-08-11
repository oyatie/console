import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { after, describe, it } from "node:test";
import { fileURLToPath } from "node:url";

import {
  evaluateRequestBodyContract,
  jsonRequestSchema,
  renameField,
} from "./check-request-body-contract.mjs";
import {
  hasOwnKey,
  own,
  PROTOTYPE_CHAIN_KEYS,
} from "./own-property.mjs";
import yaml from "js-yaml";

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

// Both wrapped-const and multi-method route forms, because both appear in the real surface and
// both have already broken a resolver silently.
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

function widgetSpec({ required, properties }) {
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
`;
}

function widgetFixture({ derive, fields, required, properties }) {
  return fixture({
    "backend/crates/widget/rest/src/lib.rs": widgetCrate({ derive, fields }),
    "backend/openapi/openapi.yaml": widgetSpec({ required, properties }),
  });
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
    assert.ok(resolved >= 45, `resolver degraded: expected at least 45 resolved operations, got ${resolved}`);
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
  // slot and reads as coverage. The floor of 45 resolved operations means only the real
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
    const handlerFile = widgetCrate({ derive: camelDeny, fields: "    quantity_consumed_milli: i64," })
      .replace(`#[derive(Debug, Deserialize)]\n${camelDeny}\nstruct ConsumeWidgetBody {\n    quantity_consumed_milli: i64,\n}\n`, "");
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
