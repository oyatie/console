# KNL one-stop platform — frozen pre-pivot notes (non-authority)

> **STOP. HISTORICAL / NON-AUTHORITY. DO NOT DISPATCH WORK FROM THIS PAGE.**
> A skim of headings below is not a work order.
> **Current ordered roadmap:** [`docs/current/ROADMAP.md`](current/ROADMAP.md), constrained by [`docs/current/PRODUCT.md`](current/PRODUCT.md).
> This file is a **frozen pre-pivot plan**. It cannot clear HOLDs. Frontend, storefront, FSM, and live-hostname claims below are historical.
> ERP, communications, MES, sales storefront, and public-web items contradict current PRODUCT scope and remain quarry/history only.

## Historical snapshot (pre-pivot claims; not current verification)

- **FSM operations** — dispatch board, work orders (= the per-asset maintenance record), daily plans, evidence upload, mobile offline sync. State machine RECEIVED→…→FINAL_COMPLETED.
- **Approvals / 기안서 chain** — purchase-request approval incl. executive final-approve (`admin-13-financial`, `exec-03-purchase-final-approve`; `PurchaseFinalApprove` feature).
- **CX / customer relations** — REAL support tickets (`/api/v1/support/intake` → support_tickets) + sales inquiries. Not a board.
- **Governance & observability** — multi-tenant platform admin (onboard/suspend/reactivate), cross-tenant ops-health rollup, KPI + ops dashboards, audit_events.
- **Sales** — sales_listings + public storefront + inquiries.
- **Platform** — multi-tenant org isolation + RLS, passkey/WebAuthn auth (cold-start OTP → enroll → usernameless login), org-scoped roles (RECEPTIONIST/MECHANIC/ADMIN/EXECUTIVE/SUPER_ADMIN).
- **Live** — knllogistic.com (storefront) + console.knllogistic.com (console; legacy fsm.knllogistic.com 301-redirects here) on the OCI/Talos cluster.

## Historical "in progress" claims at freeze — not current work

These bullets were written as present-tense before the 2026-07-28 pivot. They do not authorize implementation.

- **Web front door redesign** — unified one-stop site (corporate + FSM/CX-SaaS). The `web/` surface this described was deleted. Do not rebuild it from this page.

## Historical domain sequence — superseded, do not implement from this page

The numbered list below is a frozen pre-pivot sequence. Current ordered work is only in [`docs/current/ROADMAP.md`](current/ROADMAP.md).

1. Asset lifecycle & cost analytics (historical first pick)
2. Procurement price governance
3. Payroll as a net-new domain (current PRODUCT instead projects existing payroll truth as PayRun)
4. Bookkeeping / accounting (out of current PRODUCT scope)
5. Employee / HR cycle (current PRODUCT has a narrower HR writer path)
6. Manufacturing execution / MES (out of current PRODUCT scope; [`docs/specs/mes.md`](./specs/mes.md) is historical/quarry)
7. Customer accounts / portal (open historical decision; not current work)

## Historical cross-cutting notes — not current requirements

- Unified sign-in, ticket system, cost/governance, and RLS-arming notes below described the pre-pivot product.
- The `console-gate-rls-arming` name remains a live CI binary; citing it here does not revive storefront or FSM scope.

## Historical verification notes — not a current test plan

- Browser-E2E, `e2e/run.sh`, `landing.spec.ts`, and `console-web` image rebuilds refer to deleted or out-of-repo surfaces.
- Live deploy language is not a go-live grant. Production remains HOLD in [`docs/current/PRODUCT.md`](current/PRODUCT.md).
