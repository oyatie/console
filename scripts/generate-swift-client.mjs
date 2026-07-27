import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const generatorTag = "1.12.2";
const root = fileURLToPath(new URL("..", import.meta.url));
const cloneDir = resolve(
  root,
  `.cache/swift-openapi-generator-${generatorTag}`,
);
const generatorBin = resolve(
  cloneDir,
  ".build/release/swift-openapi-generator",
);
const outputDir = resolve(
  root,
  "clients/swift/Sources/ConsoleAPIClient/Generated",
);
const config = resolve(root, "clients/swift/openapi-generator-config.yaml");
const inputSpec = resolve(root, "backend/openapi/openapi.yaml");
const stagingRoot = resolve(root, ".cache/generated-clients");

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
    ...options,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(" ")} failed with exit ${result.status}`,
    );
  }
}

function replaceExactlyOnce(text, generated, patched, context) {
  const first = text.indexOf(generated);
  if (first === -1) {
    throw new Error(`${context}: expected generated anchor not found`);
  }
  if (text.indexOf(generated, first + generated.length) !== -1) {
    throw new Error(`${context}: generated anchor is ambiguous`);
  }
  return text.replace(generated, patched);
}

function collectEvaluationRequiredNullableFields(openApiText) {
  const schemasStart = openApiText.indexOf("\n  schemas:\n");
  if (schemasStart === -1) {
    throw new Error("required-nullable discovery: OpenAPI components.schemas anchor not found");
  }

  const lines = openApiText.slice(schemasStart).split("\n");
  const fields = [];
  for (let index = 0; index < lines.length; index += 1) {
    const schemaMatch = lines[index].match(/^    (Evaluation[A-Za-z0-9_]+):\s*$/);
    if (!schemaMatch) {
      continue;
    }

    const schemaName = schemaMatch[1];
    let end = index + 1;
    while (end < lines.length && !/^    [A-Za-z0-9_]+:\s*$/.test(lines[end])) {
      end += 1;
    }
    const schemaLines = lines.slice(index + 1, end);
    const requiredLine = schemaLines.find((line) => /^      required:\s*\[/.test(line));
    if (!requiredLine) {
      index = end - 1;
      continue;
    }
    const required = new Set(
      requiredLine
        .replace(/^      required:\s*\[/, "")
        .replace(/\]\s*$/, "")
        .split(",")
        .map((field) => field.trim().replace(/^['"]|['"]$/g, "")),
    );

    for (let propertyIndex = 0; propertyIndex < schemaLines.length; propertyIndex += 1) {
      const propertyMatch = schemaLines[propertyIndex].match(
        /^        ([A-Za-z0-9_]+):\s*(.*)$/,
      );
      if (!propertyMatch || !required.has(propertyMatch[1])) {
        continue;
      }
      let propertyText = propertyMatch[2];
      let continuation = propertyIndex + 1;
      while (
        continuation < schemaLines.length &&
        !/^        [A-Za-z0-9_]+:\s*/.test(schemaLines[continuation]) &&
        !/^      [A-Za-z0-9_]+:\s*/.test(schemaLines[continuation])
      ) {
        propertyText += `\n${schemaLines[continuation]}`;
        continuation += 1;
      }
      if (
        /nullable:\s*true/.test(propertyText) ||
        /type:\s*\[[^\]]*(?:'null'|"null"|null)[^\]]*\]/.test(propertyText) ||
        /type:\s*(?:'null'|"null")/.test(propertyText)
      ) {
        fields.push(`${schemaName}.${propertyMatch[1]}`);
      }
    }
    index = end - 1;
  }
  return fields.sort();
}

function patchRequiredNullableModel(text, modelName, fields) {
  const structAnchor = `        public struct ${modelName}: Codable, Hashable, Sendable {`;
  const structStart = text.indexOf(structAnchor);
  if (structStart === -1) {
    throw new Error(`required-nullable patch: generated ${modelName} struct not found`);
  }
  const nextSchema = text.indexOf(
    "\n        /// - Remark: Generated from `#/components/schemas/",
    structStart + structAnchor.length,
  );
  if (nextSchema === -1) {
    throw new Error(`required-nullable patch: generated ${modelName} boundary not found`);
  }

  let block = text.slice(structStart, nextSchema);
  const omitted = [];
  for (const field of fields) {
    const property = `            public var ${field.swiftName}: ${field.swiftType}?`;
    const remark = `#/components/schemas/${modelName}/${field.wireName}`;
    if (!block.includes(property)) {
      if (block.includes(remark)) {
        throw new Error(
          `required-nullable patch: ${modelName}.${field.wireName} has an unexpected generated shape`,
        );
      }
      omitted.push(field);
      continue;
    }

    block = replaceExactlyOnce(
      block,
      property,
      `            @RequiredNullable public var ${field.swiftName}: ${field.swiftType}?`,
      `required-nullable patch ${modelName}.${field.wireName} property`,
    );
    block = replaceExactlyOnce(
      block,
      `${field.swiftName}: ${field.swiftType}? = nil`,
      `${field.swiftName}: ${field.swiftType}?`,
      `required-nullable patch ${modelName}.${field.wireName} initializer`,
    );
    block = replaceExactlyOnce(
      block,
      `                self.${field.swiftName} = try container.decodeIfPresent(
                    ${field.swiftType}.self,
                    forKey: .${field.swiftName}
                )`,
      `                self._${field.swiftName} = try container.decode(
                    RequiredNullable<${field.swiftType}>.self,
                    forKey: .${field.swiftName}
                )`,
      `required-nullable patch ${modelName}.${field.wireName} decoder`,
    );
  }

  if (omitted.length > 0) {
    const propertyDeclarations = omitted
      .map(
        (field) =>
          `            /// - Remark: Generated from \`#/components/schemas/${modelName}/${field.wireName}\`.
            @RequiredNullable public var ${field.swiftName}: ${field.swiftType}?`,
      )
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "            /// Creates a new `",
      `${propertyDeclarations}
            /// Creates a new \``,
      `required-nullable patch ${modelName} property insertion`,
    );

    const parameterDocs = omitted
      .map((field) => `            ///   - ${field.swiftName}:`)
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "            public init(\n",
      `${parameterDocs}
            public init(
`,
      `required-nullable patch ${modelName} initializer documentation`,
    );

    const parameters = omitted
      .map((field) => `                ${field.swiftName}: ${field.swiftType}?,`)
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "            public init(\n",
      `            public init(
${parameters}
`,
      `required-nullable patch ${modelName} initializer parameters`,
    );

    const assignments = omitted
      .map((field) => `                self.${field.swiftName} = ${field.swiftName}`)
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "            ) {\n",
      `            ) {
${assignments}
`,
      `required-nullable patch ${modelName} initializer assignments`,
    );

    const codingKeys = omitted
      .map((field) =>
        field.swiftName === field.wireName
          ? `                case ${field.swiftName}`
          : `                case ${field.swiftName} = "${field.wireName}"`,
      )
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "            public enum CodingKeys: String, CodingKey {\n",
      `            public enum CodingKeys: String, CodingKey {
${codingKeys}
`,
      `required-nullable patch ${modelName} coding keys`,
    );

    const decoders = omitted
      .map(
        (field) =>
          `                self._${field.swiftName} = try container.decode(
                    RequiredNullable<${field.swiftType}>.self,
                    forKey: .${field.swiftName}
                )`,
      )
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "                let container = try decoder.container(keyedBy: CodingKeys.self)\n",
      `                let container = try decoder.container(keyedBy: CodingKeys.self)
${decoders}
`,
      `required-nullable patch ${modelName} decoder insertion`,
    );

    const knownKeys = omitted
      .map((field) => `                    "${field.wireName}",`)
      .join("\n");
    block = replaceExactlyOnce(
      block,
      "                try decoder.ensureNoAdditionalProperties(knownKeys: [\n",
      `                try decoder.ensureNoAdditionalProperties(knownKeys: [
${knownKeys}
`,
      `required-nullable patch ${modelName} known keys`,
    );
  }

  return `${text.slice(0, structStart)}${block}${text.slice(nextSchema)}`;
}

const evaluationRequiredNullableFields = [
  { model: "EvaluationUnitProgress", wireName: "org_unit", swiftName: "orgUnit", swiftType: "Swift.String" },
  { model: "EvaluationSubjectSummary", wireName: "org_unit", swiftName: "orgUnit", swiftType: "Swift.String" },
  { model: "EvaluationSubjectSummary", wireName: "final_grade", swiftName: "finalGrade", swiftType: "Components.Schemas.EvaluationGrade" },
  { model: "EvaluationSubjectSummary", wireName: "rv_code", swiftName: "rvCode", swiftType: "Swift.String" },
  { model: "EvaluationCycleDetail", wireName: "opened_at", swiftName: "openedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationCycleDetail", wireName: "calibration_started_at", swiftName: "calibrationStartedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationCycleDetail", wireName: "finalized_at", swiftName: "finalizedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationCycleDetail", wireName: "archived_at", swiftName: "archivedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationReview", wireName: "grade", swiftName: "grade", swiftType: "Components.Schemas.EvaluationGrade" },
  { model: "EvaluationReview", wireName: "note", swiftName: "note", swiftType: "Swift.String" },
  { model: "EvaluationReview", wireName: "submitted_at", swiftName: "submittedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationSubjectDetail", wireName: "org_unit", swiftName: "orgUnit", swiftType: "Swift.String" },
  { model: "EvaluationSubjectDetail", wireName: "final_grade", swiftName: "finalGrade", swiftType: "Components.Schemas.EvaluationGrade" },
  { model: "EvaluationSubjectDetail", wireName: "rv_code", swiftName: "rvCode", swiftType: "Swift.String" },
  { model: "EvaluationSubjectDetail", wireName: "calibrated_grade", swiftName: "calibratedGrade", swiftType: "Components.Schemas.EvaluationGrade" },
  { model: "EvaluationSubjectDetail", wireName: "calibration_reason", swiftName: "calibrationReason", swiftType: "Swift.String" },
  { model: "EvaluationSubjectDetail", wireName: "calibrated_by", swiftName: "calibratedBy", swiftType: "Components.Schemas.Uuid" },
  { model: "EvaluationSubjectDetail", wireName: "calibrated_at", swiftName: "calibratedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationSubjectDetail", wireName: "finalized_at", swiftName: "finalizedAt", swiftType: "Components.Schemas.Timestamp" },
  { model: "EvaluationPreflightItem", wireName: "subject_id", swiftName: "subjectId", swiftType: "Components.Schemas.Uuid" },
  { model: "EvaluationPreflightReport", wireName: "next_transition", swiftName: "nextTransition", swiftType: "Components.Schemas.EvaluationCycleTransition" },
  { model: "EvaluationTaskItem", wireName: "review_status", swiftName: "reviewStatus", swiftType: "Components.Schemas.EvaluationReviewStatus" },
];

function patchEvaluationRequiredNullableFields(text) {
  const discovered = collectEvaluationRequiredNullableFields(readFileSync(inputSpec, "utf8"));
  const declared = evaluationRequiredNullableFields
    .map((field) => `${field.model}.${field.wireName}`)
    .sort();
  if (JSON.stringify(discovered) !== JSON.stringify(declared)) {
    throw new Error(
      "required-nullable patch: Evaluation OpenAPI fields drifted; " +
        `discovered=${JSON.stringify(discovered)} declared=${JSON.stringify(declared)}`,
    );
  }

  const models = new Map();
  for (const field of evaluationRequiredNullableFields) {
    const fields = models.get(field.model) ?? [];
    fields.push(field);
    models.set(field.model, fields);
  }
  for (const [modelName, fields] of models) {
    text = patchRequiredNullableModel(text, modelName, fields);
  }
  return text;
}

function patchKnownGeneratorGaps(stagingDir) {
  // swift-openapi-generator 1.12 skips the `null` branch for a required
  // oneOf[$ref, null] property and emits an empty struct. The OpenAPI contract
  // intentionally models SeriesByInstanceResponse.series as a present key whose
  // value may be null, so patch this generated shape until the generator supports
  // that schema form natively.
  const typesFile = resolve(stagingDir, "Types.swift");
  let text = readFileSync(typesFile, "utf8");

  // Swift's synthesized Codable treats an Optional property as both nullable
  // and omittable. OpenAPI distinguishes those states: a required-nullable
  // property must be present on the wire even when its value is null. Keep one
  // generated support wrapper at schema scope so every patched model gets
  // missing-key rejection and explicit-null encoding.
  const schemasAnchor = `    public enum Schemas {
`;
  const requiredNullableSupport = `    public enum Schemas {
        @propertyWrapper
        public struct RequiredNullable<Value: Codable & Hashable & Sendable>: Codable, Hashable, Sendable {
            public var wrappedValue: Value?
            public init(wrappedValue: Value?) {
                self.wrappedValue = wrappedValue
            }
            public init(from decoder: any Swift.Decoder) throws {
                let container = try decoder.singleValueContainer()
                self.wrappedValue = container.decodeNil() ? nil : try container.decode(Value.self)
            }
            public func encode(to encoder: any Swift.Encoder) throws {
                var container = encoder.singleValueContainer()
                if let wrappedValue {
                    try container.encode(wrappedValue)
                } else {
                    try container.encodeNil()
                }
            }
        }
`;
  if (text.includes(schemasAnchor)) {
    text = text.replace(schemasAnchor, requiredNullableSupport);
  } else if (!text.includes(requiredNullableSupport)) {
    throw new Error(
      "patchKnownGeneratorGaps: expected Components.Schemas anchor not found; " +
        "swift-openapi-generator output may have changed, update the patch.",
    );
  }
  const emptySeriesByInstance = `        /// The series an instance belongs to, or null.
        ///
        /// - Remark: Generated from \`#/components/schemas/SeriesByInstanceResponse\`.
        public struct SeriesByInstanceResponse: Codable, Hashable, Sendable {
            /// Creates a new \`SeriesByInstanceResponse\`.
            public init() {}
        }`;
  const patchedSeriesByInstance = `        /// The series an instance belongs to, or null.
        ///
        /// - Remark: Generated from \`#/components/schemas/SeriesByInstanceResponse\`.
        public struct SeriesByInstanceResponse: Codable, Hashable, Sendable {
            /// - Remark: Generated from \`#/components/schemas/SeriesByInstanceResponse/series\`.
            public var series: Components.Schemas.SeriesHead?
            /// Creates a new \`SeriesByInstanceResponse\`.
            ///
            /// - Parameters:
            ///   - series:
            public init(series: Components.Schemas.SeriesHead? = nil) {
                self.series = series
            }
            public enum CodingKeys: String, CodingKey {
                case series
            }
        }`;
  if (text.includes(emptySeriesByInstance)) {
    text = text.replace(emptySeriesByInstance, patchedSeriesByInstance);
  } else if (!text.includes(patchedSeriesByInstance)) {
    throw new Error(
      "patchKnownGeneratorGaps: expected generated SeriesByInstanceResponse shape not found; " +
        "swift-openapi-generator output may have changed, update the patch.",
    );
  }

  // swift-openapi-generator 1.12 also drops a required oneOf[$ref, null]
  // property entirely. LeaveRequestV2View.charge_units is required on the wire
  // even though its value can be null, so preserve both halves of that contract:
  // a missing key must fail decoding and nil must encode as an explicit null.
  const chargePropertyAnchor = `            @available(*, deprecated)
            public var days: Swift.Double
            /// - Remark: Generated from \`#/components/schemas/LeaveRequestV2View/charge_state\`.`;
  const patchedChargeProperty = `            @available(*, deprecated)
            public var days: Swift.Double
            /// Exact resolved charge; null while review is required or no charge applies.
            ///
            /// - Remark: Generated from \`#/components/schemas/LeaveRequestV2View/charge_units\`.
            @RequiredNullable public var chargeUnits: Components.Schemas.LeaveUnits?
            /// - Remark: Generated from \`#/components/schemas/LeaveRequestV2View/charge_state\`.`;
  const chargeInitAnchor = `                days: Swift.Double,
                chargeState: Components.Schemas.LeaveRequestV2View.ChargeStatePayload,`;
  const patchedChargeInit = `                days: Swift.Double,
                chargeUnits: Components.Schemas.LeaveUnits?,
                chargeState: Components.Schemas.LeaveRequestV2View.ChargeStatePayload,`;
  const chargeAssignmentAnchor = `                self.days = days
                self.chargeState = chargeState`;
  const patchedChargeAssignment = `                self.days = days
                self.chargeUnits = chargeUnits
                self.chargeState = chargeState`;
  const chargeCodingKeyAnchor = `                case days
                case chargeState = "charge_state"`;
  const patchedChargeCodingKey = `                case days
                case chargeUnits = "charge_units"
                case chargeState = "charge_state"`;

  const chargePatchPairs = [
    [chargePropertyAnchor, patchedChargeProperty],
    [chargeInitAnchor, patchedChargeInit],
    [chargeAssignmentAnchor, patchedChargeAssignment],
    [chargeCodingKeyAnchor, patchedChargeCodingKey],
  ];
  for (const [generated, patched] of chargePatchPairs) {
    if (text.includes(generated)) {
      text = text.replace(generated, patched);
    } else if (!text.includes(patched)) {
      throw new Error(
        "patchKnownGeneratorGaps: expected generated LeaveRequestV2View charge_units anchor not found; " +
          "swift-openapi-generator output may have changed, update the patch.",
      );
    }
  }

  const requiredNullablePagePatchPairs = [
    [
      `            /// - Remark: Generated from \`#/components/schemas/LeaveRequestV2Page/next_cursor\`.
            public var nextCursor: Swift.String?`,
      `            /// - Remark: Generated from \`#/components/schemas/LeaveRequestV2Page/next_cursor\`.
            @RequiredNullable public var nextCursor: Swift.String?`,
    ],
    [
      `                items: [Components.Schemas.LeaveRequestV2View],
                nextCursor: Swift.String? = nil`,
      `                items: [Components.Schemas.LeaveRequestV2View],
                nextCursor: Swift.String?`,
    ],
    [
      `            /// - Remark: Generated from \`#/components/schemas/ActionInboxResponse/next_cursor\`.
            public var nextCursor: Swift.String?`,
      `            /// - Remark: Generated from \`#/components/schemas/ActionInboxResponse/next_cursor\`.
            @RequiredNullable public var nextCursor: Swift.String?`,
    ],
    [
      `                total: Swift.Int,
                totalIsExact: Swift.Bool,
                nextCursor: Swift.String? = nil`,
      `                total: Swift.Int,
                totalIsExact: Swift.Bool,
                nextCursor: Swift.String?`,
    ],
    [
      `            /// - Remark: Generated from \`#/components/schemas/EvidenceObjectPage/next_cursor\`.
            public var nextCursor: Swift.String?`,
      `            /// - Remark: Generated from \`#/components/schemas/EvidenceObjectPage/next_cursor\`.
            @RequiredNullable public var nextCursor: Swift.String?`,
    ],
    [
      `                total: Swift.Int64,
                asOf: Swift.Int64,
                nextCursor: Swift.String? = nil`,
      `                total: Swift.Int64,
                asOf: Swift.Int64,
                nextCursor: Swift.String?`,
    ],
  ];
  for (const [generated, patched] of requiredNullablePagePatchPairs) {
    if (text.includes(generated)) {
      text = text.replace(generated, patched);
    } else if (!text.includes(patched)) {
      throw new Error(
        "patchKnownGeneratorGaps: expected generated required-nullable pagination anchor not found; " +
          "swift-openapi-generator output may have changed, update the patch.",
      );
    }
  }

  text = patchEvaluationRequiredNullableFields(text);
  writeFileSync(typesFile, text, "utf8");
}

function replaceDirectoryFromStaging(stagingDir, targetDir) {
  const backupDir = mkdtempSync(resolve(stagingRoot, "swift-previous-"));
  rmSync(backupDir, { recursive: true, force: true });

  try {
    if (existsSync(targetDir)) {
      renameSync(targetDir, backupDir);
    }
    renameSync(stagingDir, targetDir);
    rmSync(backupDir, { recursive: true, force: true });
  } catch (error) {
    if (!existsSync(targetDir) && existsSync(backupDir)) {
      renameSync(backupDir, targetDir);
    }
    throw error;
  }
}

if (!existsSync(cloneDir)) {
  mkdirSync(resolve(root, ".cache"), { recursive: true });
  run("git", [
    "-c",
    "advice.detachedHead=false",
    "clone",
    "--branch",
    generatorTag,
    "--depth",
    "1",
    "https://github.com/apple/swift-openapi-generator",
    cloneDir,
  ]);
}

if (!existsSync(generatorBin)) {
  run("swift", [
    "build",
    "--package-path",
    cloneDir,
    "--configuration",
    "release",
    "--product",
    "swift-openapi-generator",
  ]);
}

mkdirSync(stagingRoot, { recursive: true });
const stagingDir = mkdtempSync(resolve(stagingRoot, "swift-"));

try {
  run(generatorBin, [
    "generate",
    "--config",
    config,
    "--output-directory",
    stagingDir,
    inputSpec,
  ]);
  patchKnownGeneratorGaps(stagingDir);
  replaceDirectoryFromStaging(stagingDir, outputDir);
} catch (error) {
  rmSync(stagingDir, { recursive: true, force: true });
  throw error;
}
