# Overlay rust-std into rustc's tree to make a usable sysroot.
#
# The two dist tarballs are deliberately separate upstream: `rustc` carries the
# compiler and its own rustlib, `rust-std` carries the standard library for one
# target triple. rustc finds std under <sysroot>/lib/rustlib/<triple>, so a
# sysroot is the union of the two. Assembling it as a build ARTIFACT -- rather
# than installing it somewhere and pointing a flag at a path -- is what puts the
# compiler and the library inside the action's input tree, and therefore inside
# the action digest. That is the whole point: a different compiler is then a
# different action, so the cache misses instead of serving an unlinkable rlib.

def _rust_sysroot_impl(ctx):
    out = ctx.actions.declare_output("sysroot", dir = True)

    # `cp -a` preserves modes and any links rather than dereferencing them.
    # The macOS linker runtime link below is assembled explicitly from the
    # pinned compiler archive; it never resolves outside the resulting tree.
    script = ctx.actions.write(
        "assemble.sh",
        [
            "#!/usr/bin/env bash",
            "set -euo pipefail",
            'out="$1"; shift',
            'rustc_dir="$1"; shift',
            'rm -rf "$out"; mkdir -p "$out"',
            'cp -a "$rustc_dir"/. "$out"/',
            'mkdir -p "$out/lib/rustlib"',
            '# Remaining args are: std trees, then `--`, then component trees.',
            '# The rule already knows which is which -- an earlier revision threw',
            '# that away and sniffed directory shape to rediscover it, which works',
            '# only while no component ships a lib/rustlib without a bin/.',
            '# llvm-tools-preview would have been the first one to break it.',
            'while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do',
            '  cp -a "$1"/lib/rustlib/. "$out"/lib/rustlib/',
            '  shift',
            'done',
            '# The separator is always emitted by the rule, so a missing one is a',
            '# bug in the caller -- and a silent one: without it the loop above',
            '# consumes the component trees as stds, and the script SUCCEEDS having',
            '# overlaid nothing. clippy-driver would simply be absent, surfacing',
            '# much later inside a clippy action as a missing binary.',
            '[ "${1-}" = "--" ] || { echo "assemble.sh: expected -- after the std trees" >&2; exit 2; }',
            'shift',
            'for tree in "$@"; do',
            '  cp -a "$tree"/. "$out"/',
            'done',
            '# macOS rust-lld expects libLLVM beside its host rustlib bin directory.',
            'if [ -f "$out/lib/libLLVM.dylib" ] && [ ! -e "$out/lib/rustlib/{0}/lib/libLLVM.dylib" ] && [ ! -L "$out/lib/rustlib/{0}/lib/libLLVM.dylib" ]; then'.format(ctx.attrs.triple),
            '  mkdir -p "$out/lib/rustlib/{}/lib"'.format(ctx.attrs.triple),
            '  ln -s ../../../libLLVM.dylib "$out/lib/rustlib/{}/lib/libLLVM.dylib"'.format(ctx.attrs.triple),
            'fi',
        ],
        is_executable = True,
    )

    ctx.actions.run(
        cmd_args(
            script,
            out.as_output(),
            ctx.attrs.rustc[DefaultInfo].default_outputs[0],
            [dep[DefaultInfo].default_outputs[0] for dep in ctx.attrs.stds],
            "--",
            [dep[DefaultInfo].default_outputs[0] for dep in ctx.attrs.components],
        ),
        category = "rust_sysroot",
        identifier = ctx.attrs.triple,
        # Measured at 534 MB assembled, not the "~1 GB of symlinks" an earlier
        # revision of this comment claimed -- the same revision whose symlink
        # story is corrected 30 lines above. Copying that locally is cheaper
        # than shipping it to a remote executor and back; the INPUTS stay
        # content-addressed, which is what the digest cares about.
        local_only = True,
    )
    return [DefaultInfo(default_output = out)]

rust_sysroot = rule(
    impl = _rust_sysroot_impl,
    attrs = {
        "rustc": attrs.dep(providers = [DefaultInfo]),
        # clippy-driver lives in its own archive, overlaid so the sysroot's bin/
        # holds every binary buck2 invokes and nothing falls back to PATH.
        # NOT rustfmt: the prelude never invokes it (no RustToolchainInfo field,
        # zero references in prelude/rust/), so it stays in the pin for
        # `cargo fmt` and is excluded here -- see `_SYSROOT_SKIP` in ./BUCK.
        "components": attrs.list(attrs.dep(providers = [DefaultInfo]), default = []),
        "stds": attrs.list(attrs.dep(providers = [DefaultInfo])),
        "triple": attrs.string(),
    },
)
