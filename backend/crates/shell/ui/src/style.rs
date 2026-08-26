/// Light-only sheet inlined as `<style>{CSS}</style>`.
///
/// If this text changes, recompute `STYLE_SRC_SHA256` / `style-src` in `csp.rs`
/// (SHA-256 of these bytes, base64). Do not add `prefers-color-scheme`.
pub const CSS: &str = r#"
:root { font-family: "Pretendard", "Apple SD Gothic Neo", sans-serif; color: #111; }
body { margin: 0; }
header { display: flex; gap: 1rem; align-items: baseline; padding: 0.75rem 1rem; border-bottom: 1px solid #ddd; }
nav a { margin-right: 0.85rem; color: inherit; text-decoration: none; }
nav a[aria-current="page"] { font-weight: 600; }
main { padding: 1rem; max-width: 52rem; }
.chip { display: inline-block; padding: 0.1rem 0.5rem; border-radius: 999px; background: #eef2ff; font-size: 0.8rem; }
.lineage { display: block; color: #555; font-size: 0.75rem; }
.editor { display: grid; gap: 0.5rem; margin: 1rem 0; max-width: 24rem; }
.editor label { display: grid; gap: 0.25rem; }
.empty { color: #444; }
.note { color: #444; font-size: 0.9rem; }
.blockers { display: flex; flex-wrap: wrap; gap: 0.35rem; list-style: none; padding: 0; }
"#;
