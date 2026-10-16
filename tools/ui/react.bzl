"""Direct pinned React compilation, strict checking and independent publication proof."""

def _tool_impl(ctx):
    executable = ctx.attrs.archive[DefaultInfo].default_outputs[0].project(ctx.attrs.path)
    return [DefaultInfo(default_output = executable), RunInfo(args = [executable])]

react_tool = rule(impl = _tool_impl, attrs = {
    "archive": attrs.dep(providers = [DefaultInfo]),
    "path": attrs.string(),
})

def _workspace(ctx):
    entries = {"src/" + name: source for name, source in ctx.attrs.sources.items()}
    entries.update({"node_modules/" + name: dep[DefaultInfo].default_outputs[0] for name, dep in ctx.attrs.packages.items()})
    entries["tsconfig.json"] = ctx.attrs.tsconfig
    entries["package.json"] = ctx.attrs.package
    return ctx.actions.symlinked_dir("workspace", entries)

def _typecheck_impl(ctx):
    workspace = _workspace(ctx)
    output = ctx.actions.declare_output("typecheck.tsbuildinfo")
    ctx.actions.run(
        cmd_args(ctx.attrs.node[RunInfo], cmd_args(workspace, format = "{}/node_modules/typescript/lib/tsc.js"),
                 "--project", cmd_args(workspace, format = "{}/tsconfig.json"), "--incremental", "--tsBuildInfoFile", output.as_output()),
        env = {"NODE_PATH": "", "NODE_OPTIONS": ""},
        category = "react_typecheck",
    )
    return [DefaultInfo(default_output = output)]

_WORKSPACE = {
    "sources": attrs.dict(attrs.string(), attrs.source()),
    "packages": attrs.dict(attrs.string(), attrs.dep(providers = [DefaultInfo])),
    "tsconfig": attrs.source(),
    "package": attrs.source(),
    "node": attrs.exec_dep(providers = [RunInfo, DefaultInfo]),
}

react_typecheck = rule(impl = _typecheck_impl, attrs = _WORKSPACE)

def _bundle_impl(ctx):
    workspace = _workspace(ctx)
    profile = ctx.attrs.profile
    javascript = ctx.actions.declare_output(profile + ".js")
    stylesheet = ctx.actions.declare_output(profile + ".css")
    guard = ctx.actions.declare_output(profile + "-guard.js")
    runtime_metafile = ctx.actions.declare_output(profile + ".meta.json")
    guard_metafile = ctx.actions.declare_output(profile + "-guard.meta.json")
    common = cmd_args(ctx.attrs.esbuild[RunInfo], "--bundle", "--minify", "--platform=browser",
                      "--target=es2022", "--preserve-symlinks", "--legal-comments=inline",
                      cmd_args(workspace, format = "--tsconfig={}/tsconfig.json"))
    ctx.actions.run(
        cmd_args(common, cmd_args(workspace, format = "{}/src/" + profile + ".tsx"), "--format=esm", "--jsx=automatic",
                 '--define:process.env.NODE_ENV="production"',
                 cmd_args("--metafile=", runtime_metafile.as_output(), delimiter = ""),
                 cmd_args("--outfile=", javascript.as_output(), delimiter = ""), hidden = [stylesheet.as_output()]),
        env = {"NODE_PATH": "", "NODE_OPTIONS": ""},
        category = "react_bundle",
        identifier = profile,
    )
    ctx.actions.run(
        cmd_args(common, cmd_args(workspace, format = "{}/src/" + profile + "-guard.ts"), "--format=iife",
                 cmd_args("--metafile=", guard_metafile.as_output(), delimiter = ""),
                 cmd_args("--outfile=", guard.as_output(), delimiter = "")),
        env = {"NODE_PATH": "", "NODE_OPTIONS": ""},
        category = "react_bundle",
        identifier = "guard",
    )
    compiled = ctx.actions.symlinked_dir("compiled", {
        profile + ".js": javascript,
        profile + ".css": stylesheet,
        profile + "-guard.js": guard,
        "typecheck.tsbuildinfo": ctx.attrs.typecheck,
    })
    inputs = ctx.actions.write_json("inputs.json", ctx.attrs.inputs, with_inputs = True)
    packages = ctx.actions.write_json("packages.json", {
        name: dep[DefaultInfo].default_outputs[0]
        for name, dep in ctx.attrs.packages.items()
    }, with_inputs = True)
    output = ctx.actions.declare_output("bundle", dir = True)
    ctx.actions.run(
        cmd_args(ctx.attrs._python[RunInfo], ctx.attrs._helper, "record", "--profile", profile, "--inputs", inputs,
                 "--workspace", workspace, "--packages", packages,
                 "--runtime-metafile", runtime_metafile, "--guard-metafile", guard_metafile,
                 "--compiled", compiled, "--node", ctx.attrs.node[DefaultInfo].default_outputs[0],
                 "--esbuild", ctx.attrs.esbuild[DefaultInfo].default_outputs[0], "--out-dir", output.as_output()),
        category = "react_record",
    )
    return [DefaultInfo(default_output = output)]

_TOOLS = {
    "_python": attrs.exec_dep(default = "toolchains//:cpython", providers = [RunInfo]),
    "_helper": attrs.source(default = "//tools/ui:react-react_bundle.py"),
}

react_bundle = rule(impl = _bundle_impl, attrs = _WORKSPACE | _TOOLS | {
    "profile": attrs.enum(["people", "account"], default = "people"),
    "esbuild": attrs.exec_dep(providers = [RunInfo, DefaultInfo]),
    "inputs": attrs.dict(attrs.string(), attrs.source()),
    "typecheck": attrs.source(),
})

def _validate_impl(ctx):
    inputs = ctx.actions.write_json("inputs.json", ctx.attrs.inputs, with_inputs = True)
    output = ctx.actions.declare_output("validated", dir = True)
    ctx.actions.run(
        cmd_args(ctx.attrs._python[RunInfo], ctx.attrs._helper, "validate", "--profile", ctx.attrs.profile, "--inputs", inputs,
                 "--candidate", ctx.attrs.native_candidate, "--committed", ctx.attrs.committed,
                 "--out-dir", output.as_output()),
        category = "react_validate",
    )
    return [DefaultInfo(default_output = output)]

react_validate = rule(impl = _validate_impl, attrs = _TOOLS | {
    "profile": attrs.enum(["people", "account"], default = "people"),
    "inputs": attrs.dict(attrs.string(), attrs.source()),
    "native_candidate": attrs.source(allow_directory = True),
    "committed": attrs.source(allow_directory = True),
})
