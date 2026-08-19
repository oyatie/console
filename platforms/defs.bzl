# Remote-cache-capable execution platform.
#
# prelude//platforms:default carries a Local-only executor, so [buck2_re_client]
# alone can never reach a CAS. This platform reuses that platform's
# ConfigurationInfo (so OS/arch selects still resolve) but swaps in an executor
# with the remote cache enabled. Execution stays local: there are no RE workers
# behind the shared NativeLink, only a cache.

def _remote_cache_platform_impl(ctx):
    configuration = ctx.attrs.base_platform[PlatformInfo].configuration
    platform = ExecutionPlatformInfo(
        label = ctx.label.raw_target(),
        configuration = configuration,
        executor_config = CommandExecutorConfig(
            local_enabled = True,
            remote_enabled = False,
            remote_cache_enabled = True,
            allow_cache_uploads = True,
        ),
    )
    return [
        DefaultInfo(),
        PlatformInfo(label = str(ctx.label.raw_target()), configuration = configuration),
        ExecutionPlatformRegistrationInfo(platforms = [platform]),
    ]

remote_cache_platform = rule(
    impl = _remote_cache_platform_impl,
    attrs = {
        "base_platform": attrs.dep(providers = [PlatformInfo]),
    },
)
