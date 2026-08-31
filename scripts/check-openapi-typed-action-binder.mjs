// Typed-action execute-binder generation gate (ADR-0031 slice).
//
// The hole this closes: #999 generates the thirteen DispatchTarget codecs and
// `decode_dispatch_target` into typed_action_generated.rs, but
// `bind_canonical_action_params` (roster match, deny-unknown via those codecs,
// action_key/target reject, non-roster object-or-null catch-all) stayed
// hand-written in typed_action.rs. PRODUCT requires one source that generates
// OpenAPI and validators. A hand-copied binder is a second contract.
//
// Chesterton: unknown-field policy lives on the generated Input structs
// (`deny_unknown_fields`). Non-roster keys (`set_priority`, `create`, …) must
// stay object-or-null — do not type-decode them and do not accept arrays.
// `reject_caller_action_key` is the path-vs-body trust boundary; keep it on
// the generated binder rather than a new runtime. Extend console-openapi-gen.
// Do not add a second OpenAPI writer.
//
// Totality: own-property reads of the manifest + text scans of the emitter,
// generator, binder host, and generated Rust. A walker that visits nothing
// reports nothing, so ACTION_FLOOR locks examined-zero.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  ACTION_FLOOR,
  CANONICAL_ACTIONS,
} from "./check-openapi-semantic-contract.mjs";
import {
  DTO_RS_REL,
  MANIFEST_REL,
  OPENAPI_GEN_REL,
  SEMANTIC_RS_REL,
} from "./check-openapi-semantic-generate.mjs";
import {
  TYPED_ACTION_GENERATED_REL,
  TYPED_ACTION_RS_REL,
} from "./check-openapi-typed-action-codecs.mjs";
import { isPlainObject, own } from "./own-property.mjs";

function push(findings, location, message) {
  findings.push({ location, message });
}

function loadJson(repoRoot, rel) {
  const path = join(repoRoot, rel);
  if (!existsSync(path)) return { path, missing: true, value: null };
  try {
    return { path, missing: false, value: JSON.parse(readFileSync(path, "utf8")) };
  } catch (error) {
    return { path, missing: false, value: null, error: error.message };
  }
}

function readText(repoRoot, rel) {
  const path = join(repoRoot, rel);
  if (!existsSync(path)) return { path, missing: true, text: "" };
  return { path, missing: false, text: readFileSync(path, "utf8") };
}

function hasFn(source, name) {
  const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return new RegExp(
    String.raw`(?:^|\n)\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+${escaped}\b`,
  ).test(source);
}

function dispatchVariant(actionKey) {
  return actionKey
    .split(/[._]/)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

/**
 * @param {{ repoRoot: string }} options
 * @returns {{
 *   actions: number,
 *   findings: { location: string, message: string }[],
 * }}
 */
export function evaluateTypedActionBinder({ repoRoot }) {
  const findings = [];
  const manifestLoad = loadJson(repoRoot, MANIFEST_REL);
  if (manifestLoad.missing) {
    push(
      findings,
      MANIFEST_REL,
      "semantic manifest is absent; hand-written typed_action.rs binder is not a generator",
    );
    return { actions: 0, findings };
  }
  if (manifestLoad.error || !isPlainObject(manifestLoad.value)) {
    push(
      findings,
      MANIFEST_REL,
      `semantic manifest is not a JSON object${manifestLoad.error ? `: ${manifestLoad.error}` : ""}`,
    );
    return { actions: 0, findings };
  }

  const actions = own(manifestLoad.value, "actions");
  let actionCount = 0;
  if (!Array.isArray(actions)) {
    const dto = readText(repoRoot, DTO_RS_REL);
    if (!dto.missing && dto.text.includes("fn dto_actions")) {
      actionCount = CANONICAL_ACTIONS.filter((spec) =>
        dto.text.includes(`"${spec.action_key}"`),
      ).length;
    } else {
      push(
        findings,
        `${MANIFEST_REL}#/actions`,
        "actions must be an array of thirteen DispatchTarget contracts",
      );
    }
  } else {
    for (const spec of CANONICAL_ACTIONS) {
      const found = actions.find(
        (item) => isPlainObject(item) && own(item, "action_key") === spec.action_key,
      );
      if (!found) {
        push(
          findings,
          `${MANIFEST_REL}#/actions/${spec.action_key}`,
          "dispatch target is missing from the semantic manifest",
        );
        continue;
      }
      actionCount += 1;
    }
  }

  const semanticRs = readText(repoRoot, SEMANTIC_RS_REL);
  if (semanticRs.missing) {
    push(
      findings,
      SEMANTIC_RS_REL,
      "contracts crate has no semantic emitter; the execute binder cannot be generated",
    );
  } else {
    if (!semanticRs.text.includes("generated_typed_action_rs")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must export generated_typed_action_rs so the binder is not dual-maintained",
      );
    }
    if (!semanticRs.text.includes("fn bind_canonical_action_params")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must emit bind_canonical_action_params from the same DTO/manifest source as the codecs",
      );
    }
    if (!semanticRs.text.includes("fn reject_caller_action_key")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must emit reject_caller_action_key (path-vs-body action_key reject is not a second runtime)",
      );
    }
    if (!semanticRs.text.includes("emit_typed_action_binder")) {
      push(
        findings,
        SEMANTIC_RS_REL,
        "semantic emitter must call emit_typed_action_binder so the binder is generated, not hand-copied match arms",
      );
    }
  }

  const genBin = readText(repoRoot, OPENAPI_GEN_REL);
  if (genBin.missing) {
    push(findings, OPENAPI_GEN_REL, "console-openapi-gen is missing");
  } else {
    if (!genBin.text.includes("generated_typed_action_rs")) {
      push(
        findings,
        OPENAPI_GEN_REL,
        "console-openapi-gen must write generated typed-action rust; a second binder writer is refused",
      );
    }
    if (!genBin.text.includes("fn bind_canonical_action_params")) {
      push(
        findings,
        OPENAPI_GEN_REL,
        "console-openapi-gen must fail closed when generated rust omits bind_canonical_action_params",
      );
    }
  }

  const binder = readText(repoRoot, TYPED_ACTION_RS_REL);
  if (binder.missing) {
    push(
      findings,
      TYPED_ACTION_RS_REL,
      "typed_action.rs host is missing; do not invent a second runtime binder",
    );
  } else {
    if (!binder.text.includes("typed_action_generated.rs")) {
      push(
        findings,
        TYPED_ACTION_RS_REL,
        "typed_action.rs must include! the generated binder; a parallel codec is refused",
      );
    }
    if (hasFn(binder.text, "bind_canonical_action_params")) {
      push(
        findings,
        `${TYPED_ACTION_RS_REL}:bind_canonical_action_params`,
        "typed_action.rs still hand-defines bind_canonical_action_params — dual-maintained binder is not generation",
      );
    }
    if (hasFn(binder.text, "reject_caller_action_key")) {
      push(
        findings,
        `${TYPED_ACTION_RS_REL}:reject_caller_action_key`,
        "typed_action.rs still hand-defines reject_caller_action_key — action_key reject must be generated",
      );
    }
    if (hasFn(binder.text, "decode")) {
      push(
        findings,
        `${TYPED_ACTION_RS_REL}:decode`,
        "typed_action.rs still hand-defines decode — codec decode must be generated with the binder",
      );
    }
  }

  const generated = readText(repoRoot, TYPED_ACTION_GENERATED_REL);
  if (generated.missing) {
    push(
      findings,
      TYPED_ACTION_GENERATED_REL,
      "generated typed-action rust is absent; the execute binder is still hand-written",
    );
  } else {
    const text = generated.text;
    if (!text.includes("semantic_manifest.json") && !text.includes("@generated")) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated binder file must declare it is emitted from semantic_manifest.json",
      );
    }
    if (!hasFn(text, "bind_canonical_action_params")) {
      push(
        findings,
        `${TYPED_ACTION_GENERATED_REL}:bind_canonical_action_params`,
        "generated rust must define bind_canonical_action_params; hand-copied match arms in typed_action.rs are not this contract",
      );
    }
    if (!hasFn(text, "reject_caller_action_key")) {
      push(
        findings,
        `${TYPED_ACTION_GENERATED_REL}:reject_caller_action_key`,
        "generated binder must reject caller target/action_key fields",
      );
    }
    if (!text.includes("decode_dispatch_target")) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated binder must call decode_dispatch_target so the 13 arms are not dual-maintained",
      );
    }
    if (!text.includes("DispatchTarget::from_str")) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated binder must use DispatchTarget::from_str for roster membership (do not hand-copy match arms)",
      );
    }
    if (!text.includes('["target", "action_key"]') && !(text.includes('"target"') && text.includes('"action_key"'))) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated binder must reject params.target and params.action_key",
      );
    }
    if (!text.includes("Value::Null | Value::Object(_)")) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated binder must keep the non-roster object-or-null catch-all (do not loosen)",
      );
    }
    if (/Value::Array/.test(text) && /Ok\(params\.clone\(\)\)/.test(text)) {
      push(
        findings,
        TYPED_ACTION_GENERATED_REL,
        "generated binder must not accept arrays on the non-roster catch-all",
      );
    }
    for (const spec of CANONICAL_ACTIONS) {
      const variant = dispatchVariant(spec.action_key);
      if (!text.includes(`DispatchTarget::${variant}`)) {
        push(
          findings,
          `${TYPED_ACTION_GENERATED_REL}:${spec.action_key}`,
          `generated decode_dispatch_target must arm DispatchTarget::${variant} from the manifest action_key`,
        );
      }
    }
  }

  const belowFloor = actionCount < ACTION_FLOOR;
  if (belowFloor && findings.length === 0) {
    push(
      findings,
      MANIFEST_REL,
      `generated binder covers ${actionCount}/${ACTION_FLOOR} actions — below the floor`,
    );
  }

  return { actions: actionCount, findings };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const repoRoot = process.argv[2] ?? fileURLToPath(new URL("..", import.meta.url));
  const result = evaluateTypedActionBinder({ repoRoot });
  const { actions, findings } = result;
  for (const finding of findings) console.error(`${finding.location}: ${finding.message}`);
  const belowFloor = actions < ACTION_FLOOR;
  if (belowFloor) {
    console.error(
      `saw ${actions}/${ACTION_FLOOR} actions — below the floor; `
        + `hand-written bind_canonical_action_params is not this contract`,
    );
  }
  if (findings.length > 0 || belowFloor) {
    console.error(
      `openapi typed-action-binder gate FAILED: ${findings.length} finding(s), `
        + `${actions}/${ACTION_FLOOR} actions`,
    );
    process.exit(1);
  }
  console.log(
    `openapi typed-action-binder gate passed `
      + `(${actions}/${ACTION_FLOOR} actions, generated bind_canonical_action_params, 0 findings)`,
  );
}
