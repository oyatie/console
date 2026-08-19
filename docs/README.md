# Documentation

Start at the repository [`README.md`](../README.md), then read the three active authorities:

1. [`current/PRODUCT.md`](current/PRODUCT.md)
2. [`current/ROADMAP.md`](current/ROADMAP.md)
3. [`current/DELIVERY.md`](current/DELIVERY.md)

[`documentation-index.json`](documentation-index.json) is the generated schema-v2 index with `coverage: "first-party-manifest"`. Its `documents` records classify every tracked first-party Markdown blob from the exact Git index tree, excluding only the declared generated, dependency, build, and third-party prefixes. The reviewed semantic source is [`documentation-manifest.seed.json`](documentation-manifest.seed.json).

The six classes describe authority and lifecycle, not CI consumption:

- `current` — an active onboarding authority.
- `decision` — an architecture decision, decision note, or maintained decision index.
- `executable-contract` — Markdown constructed as path-stable data, such as a schema, fixture, catalog, or inventory.
- `evidence` — a retained observation, audit, ledger, handoff, release record, or other claim support.
- `historical` — retained prose, reference, specification, design, benchmark, intent, runbook, or surface README that is not current authority.
- `quarry` — an idea, draft, provisional plan, or other context retained for possible reuse.

`path` and `blob_sha` are generated from exact-index-tree custody. `class`, `owner`, `status`, `replacement`, `retention`, and `archive_tag` are reviewed semantics; the generator never invents them. A document's class does not change merely because a gate reads it.

Current vs historical is a label, not a suggestion. Aspirations, frozen plans, and deleted-surface essays are not shipped product. Short pages under `docs/current/` are how-to and explanation for active work; ADRs are decisions; inventories such as [`CI-GATES.md`](CI-GATES.md) are executable contracts.

Run `npm run check:doc-manifest` for the default fail-closed check. Regenerate generated fields and the index with `node scripts/console/generate-documentation-manifest.mjs --write`, then review and classify any semantic skeleton before checking again. A new or edited tracked Markdown blob fails until it is explicitly classified and regenerated. This manifest claims first-party-manifest coverage only; it does not claim complete coverage or archive validation.

This page is a directory pointer, not another authority. ADRs, evidence, executable contracts, historical records, and quarry material may supply evidence or enforce behavior, but they cannot dispatch work or override the three active authorities unless an active authority explicitly delegates to a machine-readable contract.
