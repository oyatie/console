# DESIGN — Post-pivot Console

## Object-first interaction model

The product exposes governed company objects and their authorized actions, not a generic work hub:

`Company → OrgUnit → JobPosition → Person/Employment → HR action → PayRun`

Every surface reads real backend contracts, omits unauthorized data server-side, displays stable human labels instead of raw identifiers, and includes loading, empty, partial-failure, full-failure, recovery, responsive, and accessible states.

## Frontend hold

There is currently no frontend. Leptos SSR work is HOLD until all ADR-0030 gates are freshly green, an ADR-0001 amendment defines `Layer::Ui`, contracts and the SSR shell are stable, and real E2E evidence exists. UI crates may depend only on Contracts and UI crates; client-side business rules, fixtures, placeholder routes, comms rails, and unrelated navigation are forbidden.

The current design authority and backend reference status are recorded in [`docs/PIVOT-2026-07-28.md`](docs/PIVOT-2026-07-28.md).
