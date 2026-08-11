//! Compose every face fragment + shared preamble into `backend/openapi/openapi.yaml`.
//!
//! Run from the repository root (or any cwd): the output path is resolved from
//! this crate's manifest directory so CI and local runs agree.
//!
//! ```text
//! cargo run --locked --manifest-path backend/Cargo.toml -p console-contracts --bin console-openapi-gen
//! git diff --exit-code -- backend/openapi/openapi.yaml
//! ```

#[path = "../gen_registry.rs"]
mod gen_registry;

use std::path::PathBuf;
use std::{env, fs, process};

use console_contracts::compose_document;

fn main() {
    // Examined-zero must fail: a registry that forgot the faces would "regen"
    // an empty/partial document and `git diff` would not catch the omission if
    // openapi.yaml was deleted in the same commit. Require the shared fragment
    // plus every REST face.
    const EXPECTED_FRAGMENTS: usize = 35; // 1 shared + 34 faces
    if gen_registry::ALL_FRAGMENTS.len() != EXPECTED_FRAGMENTS {
        eprintln!(
            "console-openapi-gen: expected {EXPECTED_FRAGMENTS} fragments, found {}",
            gen_registry::ALL_FRAGMENTS.len()
        );
        process::exit(1);
    }

    let doc = match compose_document(gen_registry::ALL_FRAGMENTS, &gen_registry::PREAMBLE) {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!("console-openapi-gen: composition failed:\n{err}");
            process::exit(1);
        }
    };

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = manifest_dir
        .join("../../openapi/openapi.yaml")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../openapi/openapi.yaml"));

    // canonicalize fails before the file exists on first run; fall back to lexically cleaned path.
    let out = if out.exists() {
        out
    } else {
        manifest_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|backend| backend.join("openapi/openapi.yaml"))
            .expect("contracts crate must live at backend/crates/contracts")
    };

    if let Err(err) = fs::write(&out, &doc) {
        eprintln!(
            "console-openapi-gen: failed to write {}: {err}",
            out.display()
        );
        process::exit(1);
    }
    println!("wrote {} ({} bytes)", out.display(), doc.len());
}
