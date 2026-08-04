# Documentation

Start at the repository [`README.md`](../README.md), then read the three active authorities:

1. [`current/PRODUCT.md`](current/PRODUCT.md)
2. [`current/ROADMAP.md`](current/ROADMAP.md)
3. [`current/DELIVERY.md`](current/DELIVERY.md)

This page is a directory pointer, not a fifth authority. [`documentation-index.json`](documentation-index.json) records the machine-checked authority slice and the future full-manifest field contract. Its `coverage` is deliberately `authority-slice`; the gate rejects a `complete` claim until cross-record semantics and signed archives are implemented. The wider first-party documentation corpus is not yet fully classified, and vendored/generated trees are excluded from that project-owned universe.

ADRs, evidence, executable contracts, historical records, and quarry material remain in place during this slice. They may supply evidence or enforce behavior, but they cannot dispatch work or override the active authorities unless an active authority explicitly delegates to a machine-readable contract.
