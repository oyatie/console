# Repository Buck2 root.
#
# Rust targets are generated from the current Cargo workspace. Web, Android,
# and iOS retain their proven native build lanes until equivalent Buck2 rules
# exist; this file must not hide native-tool failures behind wrapper genrules.

# Source-only integrity inputs; no compiler or toolchain selection.
[
    export_file(
        name = "hydration-" + source.replace("/", "-"),
        src = source,
        visibility = ["PUBLIC"],
    )
    for source in ["rust-toolchain.toml", ".buckconfig", "BUCK", "tools/buck2"]
]
