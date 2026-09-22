"""Declared immutable compiler inputs, native postprocessing, and SSR validation."""

load("@prelude//rust:sources.bzl", "RustSources", "RustSourcesTSet")

def hydration_inputs(package, sources):
    inputs = {package + "/" + source: source for source in sources}
    inputs[package + "/Cargo.toml"] = "Cargo.toml"
    inputs[package + "/BUCK"] = "BUCK"
    for prefix, label, names in [
        ("", "//", ["rust-toolchain.toml", ".buckconfig", "BUCK", "tools/buck2"]),
        ("backend/", "//backend", ["Cargo.toml", "Cargo.lock", "BUCK"]),
        ("tools/ui/", "//tools/ui", ["build-payroll-wasm.sh", "wasm_bundle.py", "hydration.bzl", "BUCK"]),
        ("tools/buck/", "//tools/buck", ["gen_hydrate.py", "gen_first_party.py", "BUCK"]),
        ("toolchains/", "toolchains//", ["BUCK"]),
        ("toolchains/rust/", "toolchains//rust", ["BUCK", "hermetic.bzl", "sysroot.bzl", "wasm.bzl", "lock.bzl"]),
        ("third-party/rust/", "//third-party/rust", ["reindeer.toml", "fixups"]),
        ("third-party/rust/hydrate/", "//third-party/rust/hydrate", ["BUCK", "Cargo.toml", "Cargo.lock", "reindeer.toml", "root-dependencies.json", "input-lock.json", "fixups"]),
    ]:
        for name in names:
            inputs[prefix + name] = label + ":hydration-" + name.replace("/", "-")
    return inputs

def _command(ctx, mode):
    return cmd_args(ctx.attrs._python[RunInfo], ctx.attrs._helper, mode)

def _inputs(ctx):
    return ctx.actions.write_json("inputs.json", ctx.attrs.inputs, with_inputs = True)

def _snapshot_impl(ctx):
    output = ctx.actions.declare_output("snapshot", dir = True)
    ctx.actions.run(
        cmd_args(_command(ctx, "snapshot"), "--inputs", _inputs(ctx), "--out-dir", output.as_output()),
        category = "hydration_snapshot",
    )
    return [
        DefaultInfo(default_output = output),
        RustSources(tset = ctx.actions.tset(RustSourcesTSet, value = output, children = [])),
    ]

def _bundle_impl(ctx):
    output = ctx.actions.declare_output("bundle", dir = True)
    ctx.actions.run(
        cmd_args(_command(ctx, "produce"), "--snapshot", ctx.attrs.snapshot,
                 "--input", ctx.attrs.wasm, "--bindgen", ctx.attrs._bindgen[DefaultInfo].default_outputs[0],
                 "--out-dir", output.as_output()),
        category = "wasm_bindgen",
    )
    return [DefaultInfo(default_output = output)]

def _validate_impl(ctx):
    output = ctx.actions.declare_output("validated", dir = True)
    ctx.actions.run(
        cmd_args(_command(ctx, "validate"), "--inputs", _inputs(ctx), "--manifest", ctx.attrs.manifest,
                 "--js", ctx.attrs.js, "--wasm", ctx.attrs.wasm,
                 "--candidate", ctx.attrs.native_candidate, "--out-dir", output.as_output()),
        category = "hydration_validate",
    )
    return [DefaultInfo(default_output = output)]

_TOOLS = {
    "_python": attrs.exec_dep(default = "toolchains//:cpython", providers = [RunInfo]),
    "_helper": attrs.source(default = "//tools/ui:hydration-wasm_bundle.py"),
}

hydration_snapshot = rule(impl = _snapshot_impl, attrs = _TOOLS | {
    "inputs": attrs.dict(attrs.string(), attrs.source(allow_directory = True)),
})
hydration_bundle = rule(impl = _bundle_impl, attrs = _TOOLS | {
    "snapshot": attrs.source(allow_directory = True),
    "wasm": attrs.source(),
    "_bindgen": attrs.exec_dep(default = "toolchains//rust:wasm-bindgen", providers = [DefaultInfo]),
})
hydration_validate = rule(impl = _validate_impl, attrs = _TOOLS | {
    "inputs": attrs.dict(attrs.string(), attrs.source(allow_directory = True)),
    "manifest": attrs.source(),
    "js": attrs.source(),
    "wasm": attrs.source(),
    "native_candidate": attrs.source(allow_directory = True),
})
