// cargo metadata `target.kind[0]` is "lib" for the default crate-type and "rlib"
// once `[lib] crate-type` lists rlib (wasm cdylib crates are `["rlib","cdylib"]`).
// Both are `cargo test --lib`. kind[0]==="lib" alone reports those unit tests as dark.

const LIB_KINDS = new Set(["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"]);

export function cargoTestKind(kind) {
  const kinds = Array.isArray(kind) ? kind : [kind];
  if (kinds.includes("test")) return "test";
  if (kinds.some((entry) => LIB_KINDS.has(entry))) return "lib";
  return null;
}

// Match the generator's direct local-feature model; unsupported defaults fail closed.
export function cargoDefaultFeatures(definitions, requested = []) {
  if (!definitions || typeof definitions !== "object" || Array.isArray(definitions)) {
    throw new Error("missing Cargo feature definitions");
  }
  const defaults = Object.hasOwn(definitions, "default") ? definitions.default : [];
  if (!Array.isArray(defaults) || !Array.isArray(requested)
    || [...defaults, ...requested].some(feature => typeof feature !== "string"
      || !feature || feature === "default" || !Object.hasOwn(definitions, feature)
      || !Array.isArray(definitions[feature]))) {
    throw new Error("malformed defaults or unknown local Cargo feature");
  }
  return [...new Set([...defaults, ...requested])].sort();
}
