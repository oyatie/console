# Callable minimatch compatibility adapter

`@redocly/openapi-core@1.34.17` loads `minimatch` as a CommonJS callable.
Maintained `minimatch` releases expose that function as the named
`minimatch` export instead.

This private adapter preserves the callable interface while delegating every
operation to the audited, lockfile-pinned modern implementation. It exists
only for the scoped Redocly dependency override in the root package manifest
and can be deleted when `openapi-typescript` supports Redocly 2.
