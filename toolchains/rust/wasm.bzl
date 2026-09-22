"""Pinned host executables for the single supported hydration target."""

load("@prelude//cxx:cxx_toolchain_types.bzl", "CxxPlatformInfo", "CxxToolchainInfo", "LinkerInfo", "LinkerType")
load("@prelude//toolchains:cxx.bzl", "CxxToolsInfo")

def _wasm_cxx_toolchain_impl(ctx):
    base = ctx.attrs.base[CxxToolchainInfo]
    linker = base.linker_info
    # The bundled generic toolchain treats os=none as Linux and adds a C-driver
    # option. This target invokes raw rust-lld; preserve every other field.
    if linker.type != LinkerType("wasm") or linker.linker_flags != ["-fuse-ld=lld"]:
        fail("review the WASM toolchain adapter after a prelude linker change")
    linker_fields = {field: getattr(linker, field) for field in dir(linker)}
    linker_fields["linker_flags"] = []
    fields = {field: getattr(base, field) for field in dir(base)}
    fields["linker_info"] = LinkerInfo(**linker_fields)
    return [ctx.attrs.base[DefaultInfo], CxxToolchainInfo(**fields), ctx.attrs.base[CxxPlatformInfo]]

wasm_cxx_toolchain = rule(
    impl = _wasm_cxx_toolchain_impl,
    attrs = {"base": attrs.toolchain_dep(providers = [CxxToolchainInfo, CxxPlatformInfo])},
    is_toolchain_rule = True,
)

def _wasm_cxx_tools_impl(ctx):
    sysroot = ctx.attrs.sysroot[DefaultInfo].default_outputs[0]
    linker = sysroot.project("lib/rustlib/{}/bin/rust-lld".format(ctx.attrs.host))
    return [
        DefaultInfo(),
        CxxToolsInfo(
            compiler = cmd_args("/usr/bin/clang", "--target=wasm32-unknown-unknown"),
            cxx_compiler = cmd_args("/usr/bin/clang++", "--target=wasm32-unknown-unknown"),
            compiler_type = "clang",
            archiver = "/usr/bin/ar",
            archiver_type = "gnu",
            linker = linker,
            linker_type = LinkerType("wasm"),
        ),
    ]

wasm_cxx_tools = rule(
    impl = _wasm_cxx_tools_impl,
    attrs = {
        "sysroot": attrs.dep(providers = [DefaultInfo]),
        "host": attrs.string(),
    },
)

def _bindgen_impl(ctx):
    executable = ctx.attrs.archive[DefaultInfo].default_outputs[0].project("wasm-bindgen")
    return [DefaultInfo(default_output = executable), RunInfo(args = [executable])]

wasm_bindgen_tool = rule(
    impl = _bindgen_impl,
    attrs = {"archive": attrs.dep(providers = [DefaultInfo])},
)
