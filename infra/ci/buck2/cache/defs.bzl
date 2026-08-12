# Cache-only execution platform for shared NativeLink CAS (console warm canary).
#
# Lives under infra/ci/buck2/cache — NOT toolchains/ (toolchains cell is a reorg
# target; do not deepen canary coupling there). Mirrors prelude//platforms:default
# (local execution only) and adds remote_cache_enabled / allow_cache_uploads knobs
# the prelude hardcodes off. Remote *execution* stays False.
#
# Dark-by-default: root .buckconfig keeps prelude//platforms:default and never
# sets [cas_cache]. Only opt-in overlays under infra/ci/buckconfig/warm-cache-*
# select this platform.

load("@prelude//cfg/exec_platform:marker.bzl", "get_exec_platform_marker")

def _cache_execution_platform_impl(ctx: AnalysisContext) -> list[Provider]:
    constraints = dict()
    constraints.update(ctx.attrs.cpu_configuration[ConfigurationInfo].constraints)
    constraints.update(ctx.attrs.os_configuration[ConfigurationInfo].constraints)
    cfg = ConfigurationInfo(constraints = constraints, values = {})

    name = ctx.label.raw_target()
    platform = ExecutionPlatformInfo(
        label = name,
        configuration = cfg,
        executor_config = CommandExecutorConfig(
            local_enabled = True,
            remote_enabled = False,
            remote_cache_enabled = ctx.attrs.remote_cache_enabled,
            allow_cache_uploads = ctx.attrs.allow_cache_uploads,
            use_windows_path_separators = ctx.attrs.use_windows_path_separators,
        ),
    )

    return [
        DefaultInfo(),
        platform,
        PlatformInfo(label = str(name), configuration = cfg),
        ExecutionPlatformRegistrationInfo(
            platforms = [platform],
            exec_marker_constraint = get_exec_platform_marker(),
        ),
    ]

cache_execution_platform = rule(
    impl = _cache_execution_platform_impl,
    attrs = {
        "allow_cache_uploads": attrs.bool(default = False),
        "cpu_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "os_configuration": attrs.dep(providers = [ConfigurationInfo]),
        "remote_cache_enabled": attrs.bool(default = False),
        "use_windows_path_separators": attrs.bool(default = False),
    },
)
