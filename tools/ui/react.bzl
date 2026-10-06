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
        cmd_args(ctx.attrs.node[RunInfo], workspace.project("node_modules/typescript/lib/tsc.js"),
                 "--project", workspace.project("tsconfig.json"), "--incremental", "--tsBuildInfoFile", output.as_output()),
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
    javascript = ctx.actions.declare_output("people.js")
    stylesheet = ctx.actions.declare_output("people.css")
    guard = ctx.actions.declare_output("people-guard.js")
    common = cmd_args(ctx.attrs.esbuild[RunInfo], "--bundle", "--minify", "--platform=browser",
                      "--target=es2022", "--preserve-symlinks", "--legal-comments=inline",
                      cmd_args("--tsconfig=", workspace.project("tsconfig.json"), delimiter = ""))
    ctx.actions.run(
        cmd_args(common, workspace.project("src/people.tsx"), "--format=esm", "--jsx=automatic",
                 '--define:process.env.NODE_ENV="production"',
                 cmd_args("--outfile=", javascript.as_output(), delimiter = ""), hidden = [stylesheet.as_output()]),
        env = {"NODE_PATH": "", "NODE_OPTIONS": ""},
        category = "react_bundle",
        identifier = "people",
    )
    ctx.actions.run(
        cmd_args(common, workspace.project("src/people-guard.ts"), "--format=iife",
                 cmd_args("--outfile=", guard.as_output(), delimiter = "")),
        env = {"NODE_PATH": "", "NODE_OPTIONS": ""},
        category = "react_bundle",
        identifier = "guard",
    )
    compiled = ctx.actions.symlinked_dir("compiled", {
        "people.js": javascript,
        "people.css": stylesheet,
        "people-guard.js": guard,
        "typecheck.tsbuildinfo": ctx.attrs.typecheck,
    })
    inputs = ctx.actions.write_json("inputs.json", ctx.attrs.inputs, with_inputs = True)
    output = ctx.actions.declare_output("bundle", dir = True)
    ctx.actions.run(
        cmd_args(ctx.attrs._python[RunInfo], ctx.attrs._helper, "record", "--inputs", inputs,
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
    "esbuild": attrs.exec_dep(providers = [RunInfo, DefaultInfo]),
    "inputs": attrs.dict(attrs.string(), attrs.source()),
    "typecheck": attrs.source(),
})

def _validate_impl(ctx):
    inputs = ctx.actions.write_json("inputs.json", ctx.attrs.inputs, with_inputs = True)
    output = ctx.actions.declare_output("validated", dir = True)
    ctx.actions.run(
        cmd_args(ctx.attrs._python[RunInfo], ctx.attrs._helper, "validate", "--inputs", inputs,
                 "--candidate", ctx.attrs.native_candidate, "--committed", ctx.attrs.committed,
                 "--out-dir", output.as_output()),
        category = "react_validate",
    )
    return [DefaultInfo(default_output = output)]

react_validate = rule(impl = _validate_impl, attrs = _TOOLS | {
    "inputs": attrs.dict(attrs.string(), attrs.source()),
    "native_candidate": attrs.source(allow_directory = True),
    "committed": attrs.source(allow_directory = True),
})
