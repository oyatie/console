# Oyatie Console — Canonical Design-Intent Register

> Synthesis (2026-07-24) of seven per-source intent extracts over the Claude Design authority mirror at
> `docs/design/oyatie-console/` (worktree `pr488-design-mirror-sync`):
> DESIGN.md (101), HANDOFF.md (90), AGENTS.md §1–4 + change-log 1–120 (109), AGENTS.md change-log 121–193 + 「다음」 (59),
> TODO.md (105), ROADMAP.md (88), README/CLAUDE/BENCHMARK/DEMO (60) — 612 source intents merged and deduplicated here.

## 0. How to read this register

- **Authority boundary.** Every 「완료/시행」 in the design mirror certifies a *mockup/simulation contract*, never runtime behavior. Presumed real state: legacy server authz + PostgreSQL RLS, Cedar target/shadow only. Implementation authority is `docs/program/console-enterprise-roadmap.md` (ADR-0025: screen states DECLARED→MOUNTED/DARK→EXPOSED, `EXPOSED_SCREEN_KEYS` currently empty; `/console/*` opens only when a server-owned rollout response AND evidence-approval manifest both allow, else fail-closed to legacy `/overview`). *(AGENTS.md preamble; ROADMAP.md preamble L3-13; TODO.md preamble)*
- **Intent ≠ visuals.** Each entry states the WHY/operating principle, its exact anchor, and — where the authority reveals what the prototype simulation stands in for — the **Wiring** obligation: what FULL wiring requires (every datum from a real authorized backend response or a truthful empty/denied/error state; stubs and filler banned).
- **`[>190]`** marks intents that entered the authority after change-log 190 — the build lanes never saw them (entries 191/191a, 192/192a, 193; TODO item-41 execution charter; TODO #39 designed slice).
- Prototype seed rows, mock counts, and 완료 checkboxes are ignored as evidence throughout.

---

## 1. Cross-cutting charter (64 intents)

### 1.1 Ontology first

- **C-1** Screens are object views, not pages: every rendered noun (person, team, 법인, contract, posting, applicant, task, date, card) is referenceable, linkable, movable, traceable from anywhere. *(DESIGN §1; HANDOFF §0)* — **Wiring:** every noun resolves to a real backend object with stable code/ID and a fetchable, authorized card endpoint.
- **C-2** One object simultaneously carries three layers — semantic (type/attrs/relations), kinetic (actions/events/transitions), dynamic (policies/workflows/derived analytics/simulation) — and the UI always exposes all three together. *(DESIGN §1 3계층; HANDOFF §0; CLAUDE.md 핵심 원칙 1)* — **Wiring:** object detail API returns attributes + event history + acting policies/automations in one authorized response.
- **C-3** Relations ARE the workflow: the standard chain `C- → Position → PolicyPreset → Posting → Applicant → Employee → Timetable ⇄ Attendance ⇄ Substitution/OT(AP-) → PayrollRun → Payslip → LaborCost → ContractProfitability → (feedback) C-` is 1-click traversable up/downstream from any node; gates render as pipeline steps with exactly ONE next-action CTA. *(DESIGN §3; ROADMAP §2 표준 관계 체인; DESIGN §2 계획)* — **Wiring:** chain links from persisted relations; gate state + single permitted next action computed server-side per principal.
- **C-4** Goal state: connections already made without the user asking — record finely, analyze easily; back-references are written automatically on every mutation. *(DESIGN §1, §5)* — **Wiring:** server writes back-references and events on the real mutation path.
- **C-5** Single engine, multiple consumers: one type registry (`ONT_TYPES` in the prototype) defines each type's typed attribute schema + link types (cardinality) + writeback actions + derived analytics, and ALL surfaces — explore graph, type cards, policy principal/resource pickers, workflow blocks, module surfaces, dashboards — reference it; defining the same ontology twice is a violation. *(DESIGN §4-20; HANDOFF §18; AGENTS 56-58, 65)* — **Wiring:** a real backend ontology/metadata service as single source of truth; per-module schema redefinition forbidden.
- **C-6** Full ontology coverage — "literally everything is ontology": every on-screen noun (approvals, docs, ingest, policy, compliance CP-/RG-/FW-, audit, analytics AN-/FC-, automation, comms, HR, ERP, field, series, workforce pool…) has an engine type definition, and user-proposed types are first-class immediately with zero hardcoding. *(HANDOFF §18.1; AGENTS 58)* — **Wiring:** registry auto-merge of curated + user types in the real schema service.
- **C-7** Actions, incidents and phenomena are objects too: every event is `(type, actor, timestamp, targets, reason, result)`, linked; the audit log IS this event-object stream feeding timelines/graphs/analytics. *(DESIGN §2 액션·이벤트도 개체; HANDOFF §0)* — **Wiring:** unified event store queryable per object; audit entries first-class linkable records.

### 1.2 Lifecycle & governance

- **C-8** Universal lifecycle standard: Draft → self-attestation → submit → peer four-eyes → approve (effective-dated or immediate) → active (sensitive=passkey publish) → notice → revision (sandbox v+1) → archive → dispose; pure-event objects collapse create=close; every transition = audit event + PBAC gate + notification; versioned with non-destructive rollback. *(DESIGN §3.9; HANDOFF §15; ROADMAP DoD-9)* — **Wiring:** server-side lifecycle state machine per type.
- **C-9** Direct save is banned (§3.9.0): every save/apply/confirm names its pipeline stage; whitelist only ① personal workspace settings ② pure event objects ③ draft-stage edits ④ explicitly delegated audited ops toggles; editing an active object = pendingRev staging (live version stays; four-eyes "apply approval" effects v+1; discard withdraws). *(DESIGN §3.9.0; AGENTS 16, 18)* — **Wiring:** mutations route through stage-aware endpoints; no lifecycle-bypassing generic save.
- **C-10** Effective-dating everywhere: changes carry effective dates, drafts are sandbox proposal versions never touching live, and any version is reconstructible as-of (read-only, itself audited) — Workday/SAP SF/Oracle HCM model. *(DESIGN §3.9.1; HANDOFF §15; AGENTS 36①)* — **Wiring:** temporal/versioned data model with as-of queries server-side.
- **C-11** Pre-change impact analysis: before effect/abolition, an automatic dependency + compliance scan (people, reporting lines, positions, postings, budget, in-flight approvals, payroll linkage, span-of-control, orphan/cycle) distinguishes blockers from warnings. *(DESIGN §3.9.1; HANDOFF §15)* — **Wiring:** server-side dependency scan over real relations.
- **C-12** Maker-checker/SoD (SOX): drafter ≠ approver, approval matrix + 전결/DoA routing; 법인-level/sensitive changes require passkey. *(DESIGN §3.9.1; HANDOFF §15)*
- **C-13** Referential-integrity settlement gate + change freeze windows: no disposal while dependents or statutory settlement (transfers, consent/notice, works-council, payroll·4대보험·severance) are unsettled; org changes locked during payroll/accounting close. *(DESIGN §3.9.1; HANDOFF §15)* — **Wiring:** disposal endpoints verify dependents; freeze calendar evaluated server-side.
- **C-14** Archive ≠ delete: hidden from active views, immutably retained (audit/legal retention), as-of reconstructible; HARD DELETE FORBIDDEN — a governance anti-pattern alongside live direct edit, orphaned dependents, effective-date-less changes, drafter=approver, destructive ops without impact analysis, unnotified changes. *(DESIGN §3.9.1, §3.9.3)* — **Wiring:** soft-archive semantics + retention clocks in storage.

### 1.3 Preventive controls & PBAC

- **C-15** Prevention (gate) > detection (audit/alert) > correction, fail-closed default-deny per COSO/SOX ITGC/three-lines/four-eyes: every action passes a preflight of {authority (Cedar perms + clearance + DoA), checklist (self/peer), approval (SoD/DoA), egress (state + export perm + classification)}. *(DESIGN §3.10; HANDOFF §16; CLAUDE.md 핵심 원칙 4)* — **Wiring:** gates enforced server-side; UI reflects real denial states.
- **C-16** Attestation checklists are objects: signed+timestamped self-attestation before submit and peer four-eyes before approval; self-review system-blocked; incomplete required items BLOCK the transition. *(DESIGN §3.10-②③; HANDOFF §16)* — **Wiring:** attestation records persisted with signature+time.
- **C-17** Egress gate: external send/export/share/print only when resource.state ∈ {approved, published} AND sender has export permission AND classification allows; drafts/unreviewed/unauthorized blocked; the only escape is a reasoned DoA exception-approval request that keeps the block and audits the ask; an ungated export path must not exist. *(DESIGN §3.10-⑤; HANDOFF §16, §13.1)* — **Wiring:** egress checks enforced at the send/export API.
- **C-18** Detective layer + override taxonomy: unauthorized attempts, unapproved egress, SoD-violation attempts = anomaly audit + compliance alert; permission/SoD/egress = hard blockers, best practice = advisory; override = reason + higher approval + audit. *(DESIGN §3.10-⑥; HANDOFF §16)* — **Wiring:** denial events recorded and alert-routed server-side.
- **C-19** Everything rendered is a Cedar policy-evaluation result — screens, cards, rows, actions, search/palette results, drag candidates, aggregates, badges, notifications; card sections are the finest resource grain; deny-by-omission; never over-transmit and filter client-side. *(DESIGN §4.5; HANDOFF §0, §2; CLAUDE.md 핵심 원칙 5)* — **Wiring:** server-side Cedar evaluation on every read path including search/aggregation; placeholder chips must flip to REAL evaluation results — chips never remain decorative.
- **C-20** Covert clearance (비밀 구역): CEO-designated only; to unauthorized principals no section/menu/search hit/chip renders at all (no lock icons — existence is information); the holder roster and the covert role itself are covert; covert access logs to a CEO-only audit stream. *(DESIGN §4.5 비밀 구역; HANDOFF §2; AGENTS 141)* — **Wiring:** server never returns covert data or counts to unauthorized principals; separate CEO-only-readable audit stream.
- **C-21** "전체" (group-wide) is principal-relative: the union of the user's authorized 법인; ALL aggregates (KPIs, sums, rates, drill roots, search results, badge counts) computed only within authorized scope; unauthorized corps appear nowhere. *(DESIGN §4.5 "전체"; HANDOFF §2)* — **Wiring:** aggregation scoped by authorization server-side; never client filtering of a global dataset.
- **C-22** Even authorized viewing is recorded — (actor, target, category, timestamp) persisted as an audit object with a "view recorded" badge; own-payroll self-viewing is a right and audit-exempt via a policy flag, never hardcode. *(DESIGN §4.5, §4.8 셀프서비스; HANDOFF §2)* — **Wiring:** read-audit written on the backend read path.
- **C-23** Security/policy logic evaluates at single chokepoints only (screen guard, person visibility, mention autocomplete); components never query people/member data directly; every denial audited — silent regressions banned. *(DESIGN §4-19 단일 chokepoint)* — **Wiring:** one authorization evaluation path shared by every UI entry (palette/token/deep-link), backed by real Cedar.
- **C-24** Production identity = the real principal of an SSO/passkey session decided by Cedar; there is NO role-switch UI; admin sudo/impersonation is a separate authorization + audit procedure, never a dropdown. *(HANDOFF §0-①)* — **Wiring:** view-as switcher is demo scaffold; production = real sessions with RLS/PBAC scoping.

### 1.4 Audit & traceability

- **C-25** Audit backbone: every state transition emits who/what/when/where/how/on-what/decision/integrity + dataClass 4-tier (일반/대외비/민감/비밀) + device/geo/browser/authMethod + the driving Cedar decision + seq+prevHash chain (NIST 800-53 AU, ISO 27001, CADF/OCSF); no unaudited write path exists; viewing the audit log is itself audited. *(HANDOFF §0, §7, §9; AGENTS §3)* — **Wiring:** server-side append-only hash-chained audit store with signing + external TSA anchoring; the client-side hash chain is production-banned scaffold.
- **C-26** One token grammar app-wide: `@`=mention (notifies), `#`=object link (silent), `!`=direct code, barcode, date — in EVERY composer; candidate lists and resolved links are PBAC-gated at resolution time (covert/above-clearance deny-by-omission; unauthorized `!CODE` does not link); explicit confirmation only. *(DESIGN §4.7-7; HANDOFF §9; AGENTS 151)* — **Wiring:** candidate search + link resolution run against authorization-filtered backend queries per requesting principal.
- **C-27** Complete traceability (§4.7-10): object codes and person names in bodies/messages/notifications are token links, never plain text; claim/request drafts steer recipient rosters, evidence, and DUPLICATE-CLAIM AUTO-CHECK into structured fields — prose alone cannot process them. *(DESIGN §4.7-10)* — **Wiring:** duplicate-claim checks execute against real records.
- **C-28** Integration checklist = definition of done for every feature: (a) reference tokens/link chips to related objects, (b) back-references + events on connected objects at state transitions, (c) one-click up/downstream navigation; a feature that cannot answer "which objects does this connect to" is incomplete. *(HANDOFF §8; DESIGN §5 통합 원칙; ROADMAP DoD-3)*

### 1.5 UI grammar

- **C-29** No explanatory UI: no subtitles, tech-stack captions, meta notices, prose repeating chips; only action-driving copy; Korean-first formal-operational tone (합니다체 toasts, terse noun labels), status=chips, numbers/times/codes=mono, units and comparison bases explicit; background principles live only in the docs. *(DESIGN §4-12, §4-8; README §CONTENT FUNDAMENTALS; AGENTS 10, 28, 41)* — **Wiring:** all copy derives from real object state; bases computed from real prior-period data.
- **C-30** Exception-first + truthful empty states: normal is quiet, only exceptions get color/chips; every empty state = reason + next action in one line — the sanctioned truthful-empty pattern; fail-closed blocks state reason + resolution path. *(DESIGN §4-3, §4-10; TODO lane 20)* — **Wiring:** real empty/denied/error responses map to actionable empty states; never filler rows.
- **C-31** Files are boundary formats, objects first-class: inbound xlsx/pdf/docx ingested to structured objects with the original linked as provenance/WORM evidence; outbound = egress-gated export; the structured view is primary, the original file demoted to a secondary chip. *(DESIGN §4-13; CLAUDE.md 2026-07-08 추가)* — **Wiring:** ingest pipeline + WORM store + export generation are real backend services.
- **C-32** Module = object surface, never text wrapper: every row a typed object joining the graph (codes = auto edges), 3-layer card, governed CRUD (create=draft/register/ingest, read=audited, update=v+1 sandbox, delete=archive/dispose gate), relation drawing (code/drag = audited removable edges); for every on-screen element "how do I create, change, remove this in the UI" must be answerable — post-draft data change = OVERRIDE (reason + four-eyes, prior value audit-preserved), uniform across ALL objects. *(DESIGN §4-14; HANDOFF §20)* — **Wiring:** real CRUD+detail endpoints per type; module rows are backend records.
- **C-33** Recurring things are parent series objects (SR-): rule, instance history, trend, next occurrence; instances (monthly AP-, WO-, payroll runs) link to series so past context is never lost; instance card = series mini-timeline. *(DESIGN §4-15)* — **Wiring:** series objects + recurrence rules persisted with real instance links.
- **C-34** Regulations/parameters (minimum wage, 주52h cap, checkup cycles) are executable ledger objects (RG-) driving reference/flagging/simulation/derived actions — impact assessment, prefilled amendment drafts, effective-date automation, watch models (FC-). *(DESIGN §4-16)* — **Wiring:** regulation values stored as versioned parameters actually consumed by computations.
- **C-35** Typed fields: attributes/conditions/qualifications/reasons default to curated enums or structured types; free text only supplementary — a text field signals the object cannot participate in the dynamics layer; enum chips are live filters; no new field without a type verdict. *(DESIGN §4-19 typed fields; §4.8 사유 enum)*
- **C-36** Every object chip/row/code label/nav item is a drag source with the standard `[code title]` payload via one helper (objDrag); modules themselves are MD- objects; payload is code-only so drag can never exfiltrate content; an undraggable object representation is a violation. *(DESIGN §4-20 드래그 소스, §4-23; AGENTS 38③)*
- **C-37** Component reuse default, new patterns chartered exceptions: single module-surface cfg grammar, single object-card modal, single TONE palette, shared chip/mono/drop-zone/Esc/panel grammar; hand-drawing the same shape twice = the violation signal; a pattern introduced once auto-propagates to all same-shaped UI and the catalog updates with each directive. *(DESIGN §4-18, §4.7; ROADMAP DoD-6)*
- **C-38** Single window model for all cards/panels: 4 states (grid / pinned split reserving real body space / free-float popout / minimized tray-as-Dock), header drag=popout, dblclick=true split pin, pinned floats survive navigation; detail defaults to PINNED PANEL; every new surface declares its supported states — "unsupported by default" is a violation; approvals deliberately keep a tab workspace (intentional exception). *(DESIGN §4.7-2·3, §4-23; AGENTS 91-93)*
- **C-39** Universal list grammar: J/K/Enter nav + selection highlight, multi-attribute search over visible attributes, column drag with readability floor, shared-track alignment (per-row max-content banned), end padding + overscroll block + bottom fade, numeric/₩/%-aware sorting, no unnamed icon buttons (aria). *(DESIGN §4.7-1, §4-4, §4-5; AGENTS 44①, 97-98, 111)* — **Wiring:** grammar operates on real result sets (keyboard nav over server-paginated lists).
- **C-40** Analysis = drill invariant: every number/bar/row on an aggregate screen is a button to source objects; unclickable numbers and decorative charts banned; honest chart scaling — zero baseline unless variance <~1/3, truncation ALWAYS disclosed ("axis truncated — base ₩xxx"). *(DESIGN §4.7-9, §4-24)* — **Wiring:** aggregates carry drill queries resolving to the same authorized backend data.
- **C-41** Every element serves ≥2 user stories and supports adding a new item in place (list=new row, enum=proposed value, table=new column, stat bar=new stat, type=new attribute/relation/action/analytic, filter=savable preset) — end-to-end WITHOUT placeholders, through §3.9/§3.9.0 governance (Foundry/Airtable/Notion/ServiceNow benchmark). *(DESIGN §4-22)*
- **C-42** Viewport/density truth: default layout fits one screen, overflow scrolls inside panes, horizontal body scroll is a defect class; densityZoom keeps information density qHD→8K; every module passes the 960×540 overflow sweep; tables auto-switch full/compact by real available width. *(DESIGN §4-6, §4-19 레이아웃; AGENTS 147)*

### 1.6 Truthful wiring, honesty & scale

- **C-43** Mockup independence — THE no-simulated-data clause: the module and the whole console must work today without stubs/mock data; every visible datum is state-derived (changeable via UI) or a seed with a UI creation path; hardcoded-only data or stub behavior is a registered gap; backend-required items become explicit HANDOFF contracts — "it's backend-y" never excuses a missing UI path. *(DESIGN §4-25-6; README §WORKING PROTOCOL ⑥)* — **Wiring:** full wiring replaces every seed with real authorized backend data while preserving the UI creation paths.
- **C-44** Truthfulness audit doctrine — "express only what the console workflow can actually do": hardcoded counts/badges replaced with state-derived values, prose notifications replaced with real object links, ambiguous stats scope-labeled, cross-surface count consistency verified; false/asserted figures are defects. *(AGENTS 156, 159, 165; ROADMAP §7)* — **Wiring:** every displayed number comes from an authorized backend aggregate or is absent.
- **C-45** nav stub = 0: every nav item, CTA and button performs a real object transition or navigation; toast-only/redirect stubs are defects swept to zero; every "+" opens a full structured builder — text-input/stub creation prohibited. *(AGENTS §2, 116, 119, 120, 123-124; TODO items 24, 26)* — **Wiring:** every control maps to a real authorized API mutation or a truthful denied/empty state.
- **C-46** From-scratch reconstructability is a hard property: the console can be emptied and rebuilt purely through its own UI (creation-path inventory maintained; seed-empty/restore demo proves truthful 0-data behavior; DEMO.md scripts 7 scenarios). *(AGENTS 160, 167; DEMO.md; TODO item 28/34)* — **Wiring:** bootstrap requires no seed fixtures — real creation flows suffice; every create path posts to real APIs.
- **C-47** Scale honesty: enumeration UI is banned at production scale (3,000 people, hundreds of objects/day, multi-year) — selectors are typeahead (top-n + suggestions), lists cap with "N shown / 전체" + search, N+1 chips keep presets open; audit year partitioning and roster virtual scroll backlogged, not ignored. *(DESIGN §4-27-4; AGENTS 169; BENCHMARK §구조적 격차)* — **Wiring:** server-side indexed search + pagination endpoints per selectable type are a named backend contract.
- **C-48** DLP honest threat model — security theater banned: a pure web app (incl. WASM) cannot block screenshot/OS-copy/print; the console deters (user-select, in-app clipboard, canvas render, dynamic watermark, blur-on-blur, print intercept), tracks (audit every view/copy/export/print attempt), gates (covert/sensitive render only on managed device + trusted network + passkey); complete prevention is a layer-3 DEPLOYMENT requirement (enterprise browser/VDI/endpoint DLP/MDM); NO UI may ever promise "complete blocking"; graduated intervention keeps approved-object daily viewing frictionless. *(DESIGN §4.5 DLP; HANDOFF §13, §13.1; AGENTS 87-90, 159)* — **Wiring:** the server transmits sensitive data only AFTER policy evaluation (no over-transmission); real enforcement = server-side token revoke + minimal-transmission API.
- **C-49** Enterprise trust = working features as evidence, not documents: standards/certifications are FW- objects and each control maps to a live console feature as its evidence (Vanta/Drata benchmark) atop the HANDOFF §17 backend contract (SOC 2, ISO 27001/27017/27018, SSO SAML/OIDC + SCIM, tenancy isolation, KMS envelope encryption, TLS 1.3/mTLS, STRIDE gates, NIST 800-61 IR, OTel, SLO/error budget, OCSF/SIEM export). *(HANDOFF §17; AGENTS 19)* — **Wiring:** §17 is real infrastructure, not UI.

### 1.7 Determinism, no-code, configuration

- **C-50** Deterministic no-AI identity charter: algorithmic/mechanical/programmatic/no-code-first replaces Palantir's AI-first; all "smart" behavior is rules, templates, typed predicate evaluation, and deterministic simulation. *(DESIGN §4-20 정체성; AGENTS 65c)* — **Wiring:** no AI dependencies; all derivations deterministic and reproducible.
- **C-51** Automation is deterministic or manual, never AI (§4-28): every automated decision's basis is a deterministic rule (same input = same output) named in the audit record; what rules cannot answer goes to a human queue with four-eyes — automation removes tracking/handoff burden, never replaces human decisions, and is never a policy-bypass lane. *(DESIGN §4-28; AGENTS 178; HANDOFF §6)*
- **C-52** No-code first: Cedar policies AND workflows are edited on visual block canvases by non-developers (no code/JSON/policy syntax in UI), always shipping preview + simulation ("who sees what", "how objects move") + undo/version history; decorative simulation is banned — simulation EXECUTES real predicates on real samples; the no-code audit bar: discoverable without docs, mistakes self-recoverable. *(DESIGN §4.6, §4-20 configurable; HANDOFF §2; TODO line 307)* — **Wiring:** bidirectional mapping between canvas blocks and real Cedar/workflow definitions; simulation runs real evaluation.
- **C-53** Configuration is functional typed config: thresholds/routing/mappings/protocols are versioned no-code policy/config objects with revision staging (HANDOVER_POLICY, APPR_ROUTING, TEAM_MIN_POLICY, DoA-E1, auto-Lost policy…); hardcoded constants and hardcoded protocols are banned; logic objects require full CRUD. *(DESIGN §4-20 configurable; AGENTS 52-55, 180/180a)* — **Wiring:** config objects stored and consumed by the real execution engine.
- **C-54** Escape-proof forms (§4-27, six invariants): keystroke normalization masks so malformed values CANNOT EXIST (phone/amount/사업자번호/date/code/ratio), required structured fields = fail-closed submit gates steering to the first violation, N+1 enums ("+ 직접 입력"), typeahead scale selectors, practical copy, one-line downstream-effect disclosure ("where this record joins"). *(DESIGN §4-27-1…6; AGENTS 170/170a)*
- **C-55** Pre/postflight checklists on critical actions (publish/activate/submit/deploy/close): preflight auto-checks machine-verifiable items + manual signed attest ONLY for human judgment, fail-closed showing what remains and why; postflight auto-records verification (run log, produced objects, anomalies) with alert + rollback paths; soft items honestly soft (warn, never fake-blocking). *(DESIGN §4-29; AGENTS 182/183/183a)* — **Wiring:** preflight verdicts computed from real system state; postflight from real run results.
- **C-56** SLA ≠ SLO, never conflated: SLA = contractual external commitment (contract-bound, breach=penalty, egress-grade severity); SLO = internal target (breach=alert+improvement loop); labels always distinguish; BOTH are configurable settings objects. *(DESIGN §4-26)* — **Wiring:** SLA/SLO definitions stored as config objects driving real breach computation.
- **C-57** Dynamics surfacing is bidirectional: object surfaces show acting automations/policies, automation rules show the object chains they touch, 1-click both ways; automation/policy edit blocks MUST bind to ontology object types (free-text blocks banned). *(DESIGN §4.7-8; AGENTS 63b, 65b)*

### 1.8 Scope, personas, mobile

- **C-58** The console is an all-employee system (field/production/maintenance included), not an admin tool: universal minimum = e-approval, OWN payroll self-service, messenger/mail/notifications/directory/board; the PBAC principal determines the screen; "admin-only" is the exception. *(DESIGN §4.8; CLAUDE.md 핵심 원칙 11)*
- **C-59** One console, two renderings: the mobile employee app is the same console self-adapting under 768px (7 comms/approval surfaces, bottom tab bar, sheets, swipe grammar, keyboard-safe composers, 44px+ targets) — same objects, same state, not a forked app. *(AGENTS §1, 27, 29; DESIGN §4.8 모바일; TODO mobile lane)* — **Wiring:** shared state/APIs across desktop console and mobile.
- **C-60** Comms modules are dual surfaces: rail summary and full main view share the same objects/state so they always agree; opening from the sidebar promotes rail→main, navigation returns to rail. *(DESIGN §4.8 rail↔main; AGENTS §2)* — **Wiring:** one data source per comms domain feeding both surfaces.
- **C-61** Personas are deny-by-omission end to end: nav, palette, data, people and record existence are not exposed beyond scope; inboxes/submissions owner-scoped for every persona; the 10-persona matrix (incl. external applicant, field technician, payroll, compliance, CX/sales) is walked e2e — a role's real daily flow is the design criterion, core task within 3 clicks. *(AGENTS 126; DESIGN §4-25-7; ROADMAP §8)* — **Wiring:** nav and screen sets derived from real per-principal PBAC entitlements.

### 1.9 Standing protocols & maturity

- **C-62** Closed-loop review is permanent operating mode: every user-visible page cycles the §4-25 eight questions (best-in-class, workflow coverage incl. edge cases, friction, benchmark, integration/reuse, mockup independence, persona coverage, layout) plus the §4-21 three-question benchmark ("what would Palantir/SAP/Slack — or the domain's best — do better?") feeding a governed benchmark-gap register that selects the next deep slice; "done" without the loop is banned; the console also lints itself (Foundry Linter parity: rule-based live sweeps producing non-destructive Fix Proposals routed through staging/approval gates). *(DESIGN §4-21, §4-25; AGENTS 104-106; TODO 실행 큐 header)*
- **C-63** Maturity is graded: all 33 modules judged against six L2 criteria (full CRUD, e2e-proven flows, §4-27/28/29 compliance, persona coverage, drill completeness, production-scale handling); L3 deficits (real ledgers, statutory outputs, real calc engines) tracked solely in the ROADMAP grade table. *(AGENTS 187-190; TODO item 42)*
- **C-64** `[>190]` One-module-at-a-time full maturity: each new suite module passes the §4-25 loop before the next starts; thin fan-out banned; order CRM→WMS→MES; Palantir flavor mandatory (typed objects, transitions=audit events, rules=settings objects, drill chains). *(TODO item 41 header; AGENTS 193 context)*

---

## 2. Per-module intents

Modules ranked by centrality within each section. `[>190]` = entered the authority after change-log 190.

### payroll (7)

- **PAY-1** The payroll round (회차) is a gated, reproducible chain proven e2e: attendance exceptions (reason required fail-closed; overtime requires linked work scope) → per-법인 close 4/4 → calculation → roster → payroll-exception confirmations → "ready to submit" → submission as AP- → transfer scheduled only on approval; every gate real, every step audited; Workday close-cockpit shape where every blocking message is a fix-link drilling to the exact unresolved item. *(AGENTS 118, 165-166; TODO item 25)* — **Wiring:** real payroll computation and transfer scheduling behind approvals; blocked-on: real statutory payroll calc engine (BENCHMARK row 4).
- **PAY-2** PayrollRun · PayItem · PayslipDoc PS- are real objects; runs belong to an SR-205 series with trend/next-run linkage. *(HANDOFF §1; AGENTS 33⑨; TODO line 176)*
- **PAY-3** Payslips deliver via the personal inbox (self-service right, audit-exempt by policy flag); viewing others'/aggregate rosters is a sensitive-class audited event. *(DESIGN §4.8 셀프서비스; AGENTS 97, 139)* — **Wiring:** read-audit on the real read path.
- **PAY-4** Overtime closes the loop with no manual re-entry: AT- approval → wf3 automation (⚙ actor audit naming the AT→PS chain) → payroll exception auto-update with recomputed amount surfacing at calculation stage. *(AGENTS 150; TODO line 313)* — **Wiring:** automation executes server-side; payroll delta from the real engine.
- **PAY-5** Substitute pay settles as a series (SR-206) feeding labor cost and contract margin recomputation live. *(AGENTS 112; TODO lanes 4/9)*
- **PAY-6** Payroll roster obeys the universal list grammar (column-width drag as personal view config §3.9.0-①, J/K); roster opens as sheet only after generation (progressive disclosure aligned with the computation gate), the opening itself a sensitive audited view. *(AGENTS 139, 144; ROADMAP §7)*
- **PAY-7** Payroll persona flow: attendance-closing gate → run creation (scheduled job) → exception review (deductions·substitution pay) → transfer approval → payslip distribution to inbox. *(ROADMAP §8 급여 담당)*

### attendance (12)

- **ATT-1** Attendance is plan-vs-actual truth everywhere: daily 2-track hourly timeline (planned dashed / actual solid / now line) and monthly view with typed status vocabulary (lv 유급휴가, pa 승인 결근, ab 무단, abc 대체 커버), precise aggregation semantics (approved absences excluded from absence aggregates; covered absences still count against the absentee), and weekly-52h + month-close gates. *(AGENTS §3, 2026-07-04 (4); HANDOFF §1, §9; TODO lines 255·274)* — **Wiring:** real roster/attendance data with server-computed aggregates.
- **ATT-2** The employee-day is an object (person×date): plan/actual timeline, that day's AuditEvents, payroll impact, and communications, with SECTION-level dataClass gates (health = 비밀, health manager/CEO only, viewing audited). *(HANDOFF §9; AGENTS 3f; TODO line 256)*
- **ATT-3** Check-in/out is a verified state transition: registered-device × geofence gates, unregistered = forbid fail-closed + audit, reflected live in the actual track; shift swaps = mutual consent → foreman approval → timetable exchange as AP- objects. *(AGENTS 44④, 101③; TODO item 10)* — **Wiring:** real device registration + geofence verification server-side.
- **ATT-4** The substitution (대근) loop closes end-to-end through one chokepoint: vacancy → pool assignment → AP- + timeline fill-in → per-case labor contract auto-issued to the substitute's inbox (passkey receipt = call acceptance) → SLO breach alert auto-resolves → substitute-pay series settles → labor cost and contract margin recompute. *(AGENTS 44③, 47③, 79, 99, 112; HANDOFF §9 Substitution; DEMO §4)* — **Wiring:** each hop is a real transaction: assignment, contract issuance, notification, cost rollup.
- **ATT-5** Cover planning is forward-looking: leave approval for a cover-required position fires the substitute-needed alert at approval time (lead time secured); a D+7–30 cover planner crosses approved absences × cover-required positions × assignment status; future-dated assignments affect only their date; half-day = partial time-band coverage; weekly cover-check scheduled job. *(AGENTS 82, 101⑤; TODO items 13/93)*
- **ATT-6** Team minimum staffing is a fail-closed approval gate (TM-01): overlapping approved absences breaching a per-team minimum (TEAM_MIN_POLICY settings object) block approval with reason + resolution path; substitute assignment offsets the count → re-approval closed loop. *(AGENTS 102①; ROADMAP §7)*
- **ATT-7** Vacancy handover is policy-driven automation: HANDOVER_POLICY settings object (auto-redistribute default, dept-head four-eyes escalation, fit floor, org-chart-resolved head mapping — no hardcoded names) governs idempotent redistribution with per-case audit citing the policy; absence auto-fires wf7 with a time-limited TK- token scoped to only the relevant objects (least privilege, auto-revoked), leave approval auto-installs delegation + vacation protection (right-to-disconnect); only judgment cases stay in the manual four-eyes queue; the auto/manual boundary itself is policy-configurable (HO-01). *(AGENTS 52-55, 177; ROADMAP §7)* — **Wiring:** triggers from real HR events; tokens are real authz artifacts; plans derive from real work items (the seed plan is explicitly a stand-in).
- **ATT-8** Separation/leave-of-absence is a lifecycle state machine: draft (reason enum·effective date) → precheck (ongoing work·cover·freeze window) → 4-step SoD approval → effectuation (status auto-excludes from aggregates, triggers backfill alerts) → 6-item fail-closed recovery settlement checklist (assets·accounts·handover·leave·pay·insurance) blocking archive; reinstatement restores active. *(AGENTS 101②; ROADMAP §7)*
- **ATT-9** Off-site/business-trip/remote work are typed approval objects in the same leave decision queue — work mode is governed. *(AGENTS 51②)*
- **ATT-10** Attendance close-confirm passes a §4-29 preflight (auto checks + attest; soft leave items marked "retroactively applied on approval"). *(AGENTS 185/186)*
- **ATT-11** Today view is hierarchical: 법인 filter seg, site-group fold, site name = object/map drill. *(AGENTS 125)*
- **ATT-12** Statutory leave promotion (근로기준법 §61) rounds 1·2 and work-refusal-right notices are AP- objects flowing through approval lines to personal-inbox passkey receipt confirmation — legal compliance executed as workflow; unused-leave remainder is DERIVED from Attendance/Leave, never hand-entered; paper fallback continues from the same object. *(DESIGN §4.8 연차 촉진; HANDOFF §4; AGENTS 2026-07-04 (1))*

### workforce (인력풀) (4)

- **WF-1** WorkforcePool is a Person subtype for non-regular labor (daily/part-time/freelance/dispatch) with contractType, rates, availability, skills, clearance, rating, rehireHistory, distance; pool intake ONLY via recruiting with posting provenance and NO employee record created. *(HANDOFF §9; ROADMAP §8 인력풀 체인)*
- **WF-2** Employment states gate scheduling/aggregation via a single filter chokepoint (hrMatch/wpAll); the pool never inflates headcount; Cedar p9-p11 scope pool PII, deny internal modules to non-regulars, exclude inactives from staffing. *(AGENTS 43①②; ROADMAP §8)*
- **WF-3** Call-out monetary loop: per-call contract → passkey receipt → SR-206 settlement series auto-enrollment → payroll run note; unfilled vacancy = SLO alert auto-resolving on assignment. *(TODO workforce lanes 4/9; AGENTS 99)*
- **WF-4** Talent-pool → workforce-pool conversion requires the person's consent (proposal pending agreement, provenance retained). *(AGENTS 44⑤)*

### recruit (8)

- **REC-1** Recruiting is a bidirectional guarded pipeline (Greenhouse/Lever/Ashby benchmark): scorecards are a fail-closed precondition for offers; offer withdrawal returns to interview preserving history; rejections are typed reason enums feeding a talent pool with reversal; hire confirmation creates the real employee object (position presets inherited, contract dispatched to inbox, audited). *(AGENTS 2026-07-08 (5); ROADMAP §4 recruit)* — **Wiring:** hire = real employee record + contract issuance transaction.
- **REC-2** Posting visibility is typed policy: internal/external scope enum; external personas must not even see internal postings exist (deny-by-omission + Cedar p12); internal postings double as a company-wide career board gated by can(). *(AGENTS 39, 45; ROADMAP §8 채용공고 스코프)*
- **REC-3** Posting creation separates draft (invisible to applicants) from publish — a distinct audited authority act (§3.9.0-④) with fail-closed required fields and a §4-29 publish preflight; daily-hire postings route hires into the workforce pool, not employee creation. *(AGENTS 43①, 46, 185; ROADMAP §8 공고 등록)*
- **REC-4** External candidates get true scoped self-service: own applications/offers/documents only, evaluations invisible, offers as passkey-gated legal inbox docs, recruiting-scoped DM, application joins the real pipeline with duplicate block, audited. *(AGENTS 36⑦, 38⑤; ROADMAP §8 지원자)*
- **REC-5** Depth backlog: posting auto-fills position conditions from the contract→position→preset chain; hire auto-creates employee+timetable; interview schedules link to calendar objects. *(DESIGN §6-1·2·5)*
- **REC-6** Application/resume viewing is audited; candidate dignity is a design property of the whole chain. *(TODO recruit lines 317, 183)*
- **REC-7** New-posting ingest can auto-generate bid/pricing decisions via automation (sovereign pricing feed). *(TODO line 120)*
- **REC-8** HR persona flow: pipeline → hire confirmation → labor contract (inbox passkey) → onboarding check → HR card. *(ROADMAP §8 HR 담당)*

### org (8)

- **ORG-1** Organization change is the lifecycle reference implementation: draft (신설/개편/폐지 + effective date) → impact precheck (affected people·dependents·freeze windows) → 4-party SoD approval → atomic versioned effectuation (org version N+1, prior kept) → abolition settlement under a referential-integrity gate (transfers, positions, cost centers, postings, assets, payroll/insurance + statutory: consent/notice/노사협의) → dispose precheck → archive (hidden, history preserved). *(DESIGN §3.9.2; HANDOFF §15; AGENTS 2026-07-04 (13))* — **Wiring:** org versions and settlement checklists are backend entities.
- **ORG-2** Org = Group→법인→Site→Team hierarchy with `entity.visible` policy so private entities are deny-by-omission invisible. *(HANDOFF §1)*
- **ORG-3** Inline org-chart edits are sandbox proposals accumulating into a reorg approval banner ("N건 제안 · 개편 승인"); team delete is fail-closed while members remain (forbid audit + relocation guidance); ending edit with a diff auto-opens the reorganization approval. *(DESIGN §3.9.0 전수 audit; AGENTS 140)* — **Wiring:** diff against real org data; approval a real workflow instance.
- **ORG-4** New-법인 creation is a progressive 3-step wizard (basics → org with mandatory first worksite fail-closed → integrations seeding channel/alerts/주52h monitoring) whose confirmation is a reorganization approval; one submit cascades real object creation across org/messenger/notification/automation. *(AGENTS 128; DEMO §1)* — **Wiring:** atomic multi-module creation transaction.
- **ORG-5** Entity-card finance summary exists only for sensitive/secret clearance (deny-by-omission), collapsed by default with the §3.10-⑥ "expanding is recorded" pre-notice; expansion emits an ENT-FIN view audit and drills to the entity-scoped dashboard. *(AGENTS 142)* — **Wiring:** real audit on expand; figures from real finance aggregates.
- **ORG-6** Every org designation anywhere is an object drill: team → team card (derived headcount, auto-detected leader, path), 법인 → entity card. *(AGENTS 143)*
- **ORG-7** Lifecycle stage determines edit governance: draft-stage entities inline-edit directly (§3.9.0-③ with audit + §4-27 masks); confirmed entities change only via approval. *(AGENTS 172)*
- **ORG-8** Entity wizard captures representative/사업자번호 (blank allowed pre-issuance, rendered honestly as "발급 대기")/address with truthful fallback rendering. *(AGENTS 168)*

### directory (3)

- **DIR-1** The roster derives from live people data with employment-status enums: inactive statuses dim and are excluded from aggregates via a single filter helper; Workday-People-grade with message/mail/card actions. *(AGENTS 43②; ROADMAP §4 directory)* — **Wiring:** dynamic PEOPLE source = real people records.
- **DIR-2** Person cards support a persistent photo slot and a compact "access card" view (photo+basics, access-zone chips, recent access events) as the access-control-integration surface. *(AGENTS 152)* — **Wiring:** real photo storage; access events from the real access-control system.
- **DIR-3** Client directory tab: persistent CL- clients (grade/terms/main-site) with transaction-chain graph edges spanning contracts/sites/mail/CS. *(TODO line 181)*

### hr (person card) (5)

- **HR-1** Person is the principal-attribute source (position, grade, job, affiliation, hire date, self-relation); working conditions derive from preset inheritance (법인←site←job←position), not per-person copies. *(HANDOFF §1)*
- **HR-2** The person card is the employee LEDGER: hire, evaluations (sensitivity-gated), attendance exceptions, leave, payroll assignments (amounts masked, view audited), own approvals merged chronologically with type chips, object codes, and row drill to source modules (Workday worker profile benchmark), all live-derived. *(AGENTS 174)* — **Wiring:** ledger aggregates real cross-module history per person with per-category PBAC and view audits.
- **HR-3** Field-level governance split is explicit and taught by the card: contact info = direct-edit whitelist (§3.9.0-①); rank/affiliation = appointment approval. *(AGENTS 173)*
- **HR-4** Employee registration is a from-scratch real flow: fail-closed modal (required fields, duplicate active-employee block) whose submit joins real registries, rebuilds the graph, fires onboarding notification + contract inbox dispatch with audit — same contract as the recruiting hire hook. *(AGENTS 116, 168)* — **Wiring:** registration writes real people records feeding every downstream module.
- **HR-5** Personnel-card category access chips must become real Cedar-evaluated enforcement (currently labels — a registered gap). *(TODO line 315)*

### evaluation (3)

- **EVAL-1** Scorecards auto-attach decision context (attendance / recent work / KPI, each drillable) so grading is evidence-adjacent; submission mints an RV- object, clears the to-do, and audits. *(AGENTS 149)* — **Wiring:** context from real per-subject data; RV- persisted.
- **EVAL-2** Evaluation history on the person card renders only for sensitive/secret clearance with per-row view audit. *(AGENTS 149; TODO line 314)*
- **EVAL-3** Target bar: Lattice/15Five-grade reviews tied to KPI and attendance data. *(ROADMAP §4 review)*

### leave / benefit (4)

- **LV-1** Leave = grant/use/promotion as objects; LeavePromotion {round 1|2, leaveRemaining derived, ap, receiptDoc, deadline} and LaborRefusal both notify the entire approval line + audit, with paper fallback linked on the SAME object. *(HANDOFF §4)*
- **LV-2** Leave status/queue/promotion render as window-model card zones (pattern-setter for multi-section cards); promotion card always present with empty-state CTA. *(AGENTS 91; TODO item 1)*
- **BEN-1** Benefits (복리후생) are typed linked objects with a policy lifecycle (draft→pending→finalized w/ effective date→implemented→retiring→retired); transitions notify participants + audit; tiered by site/grade/title with Cedar no-code conditions on principal attributes; the lifecycle pattern propagates to other policy-like objects. *(DESIGN §4.8 복리후생; HANDOFF §1)*
- **LV-3** Compose-time object linking is enforced per draft type: welfare=recipients (REQUIRED), leave=roster, cover-shift=site/vacancy — links remain reference tokens for chain tracing/analytics. *(DESIGN §4.8 기안 유형별 연결)*

### appr (approvals / drafting) (7)

- **APR-1** Drafting is structured and guarded: subject person chips, automatic duplicate/similar-case checks with links (same target×type×period vs existing AP-), evidence as structured lines (amount·item·vendor) with originals demoted to provenance chips, expenditure types requiring evidence fail-closed, computed amount-projection panel with derived-chain chips. *(AGENTS 2026-07-08 (2b)(11a), 95; TODO item 4)* — **Wiring:** duplicate checks + projections computed from real records.
- **APR-2** Contract drafting is the guardrail template: permission preflight audit, timestamped self-attestation checklist, four-eyes reviewer excluding drafter, SoD approval line — incomplete = fail-closed block. *(AGENTS 2026-07-08 (1a); DESIGN §3.10.1)*
- **APR-3** Final approval ≠ closure: the drafter (else assignee) confirms the outcome and closes; only closed docs move to the archive box (24h grey-out); Cedar-authorized override closure and post-final rejection (audit/compliance/CEO) exist — always with reason + audit + all-party notification. *(DESIGN §2 종결 규칙; HANDOFF §5)* — **Wiring:** closure is a distinct backend state with Cedar-checked override endpoints and notification fan-out.
- **APR-4** History is transparent, overrides governed: completed items drill into the approval line's audit chain; 대결 (decision-on-behalf) is a Cedar override modal requiring reason (fail-closed), showing DoA basis, executing for real, audited at sensitive grade, notifying the owner. *(AGENTS 119)*
- **APR-5** Bulk approval never bypasses judgment: urgent, high-value (₩5M+), and self-drafted (SoD) rows get no checkbox at all (with stated reason); bulk approve = per-item state change + individual audit + notification clearing. *(AGENTS 163)* — **Wiring:** guard classes computed from real item attributes.
- **APR-6** Inline approve/reject is deliberately rejected — approval must pass the review panel; approvals keep a tab workspace (intentional window-model exception). *(AGENTS 102④, 48)*
- **APR-7** Console changes are themselves approvable objects: screen-config/ontology/automation changes have a dedicated draft template; support tickets escalate into it; team layout deployments route through it. *(AGENTS 60c, 73)*

### inbox (personal inbox) (5)

- **INB-1** The personal inbox generalizes the payslip: delivers payslips + legally receipt-required documents (labor contracts, leave-promotion notices, work-refusal rights, rules-of-employment changes); each doc carries receipt/read-confirmation state — the confirmation IS the legal evidence. *(DESIGN §4.8 개인 Inbox; HANDOFF §1/§3)* — **Wiring:** receipt confirmations persisted as evidentiary records; first backend-ization target.
- **INB-2** Legal friction only where legally required: `legal && !confirmed` docs reveal body ONLY after WebAuthn/FIDO2 passkey; auth success = receipt/reading proof → immutable confirmed{actor,ts} + audit; everyday docs (payslips) stay frictionless. *(HANDOFF §3; DESIGN §4.8 passkey)* — **Wiring:** server challenge issuance, signature verification, RFC3161-grade receipt-timestamp notarization; the front's 1.05s scan is UX simulation only (§0-②).
- **INB-3** Sender side: company→individual legal notices go AP- approval → on approval create target InboxDoc → await receipt; the AP-'s final 수령확인 stage closes via InboxDoc.confirmed with the bidirectional link AP.receiptDoc ↔ InboxDoc. *(HANDOFF §3)*
- **INB-4** Inbox is strictly owner-scoped for every persona; nav badge = unconfirmed legal docs. *(AGENTS 36⑦; TODO lines 262-264)*
- **INB-5** Offers to external candidates are passkey-gated legal inbox documents. *(AGENTS 38⑤)*

### docs / records / evidence (10)

- **DOC-1** Every approval/record artifact is an object without exception: drafts gain AP- on submit, intake IN-, journals JL- (day×site×author, cross-linked to attendance/maintenance); any new record type is born with code + reference token + card + audit grammar. *(DESIGN §2 결재물·기록물)* — **Wiring:** server-side code issuance + audit trail at creation for every type.
- **DOC-2** Records registration is governed intake: file drop + metadata → IN- pending → records-manager approval → archive confirmation with original integrity hash + registration audit; registered records join the generic lifecycle with disposal gated on retention and legal hold (disposal destroys content, preserves metadata+audit per PIPA). *(HANDOFF §9 ArchiveRegister; AGENTS 2026-07-08 (10); TODO line 122)*
- **DOC-3** Evidence is court-grade: EV- EvidenceRecord{SHA-256 originalHash, RFC-3161 tsaToken, sig, custody[], derivatives[]}; originals WORM write-once; derivatives (transcodes, PDF/A, WebP, thumbnails, OCR text) are linked "viewing copies, non-evidentiary"; archive optimization applies to derivatives ONLY; standards with per-jurisdiction branching (ISO 15489, OAIS, eIDAS QTS, FRE 901/902, NIST SP 800-86, SEC 17a-4, 전자문서법, 전자서명법). *(HANDOFF §11; DESIGN §6-8; TODO lines 331-337)* — **Wiring:** real WORM object store + hash chain + trusted timestamping + re-verification API.
- **DOC-4** Chain-of-custody: collector, time, device, IP + every view/download/transfer audited; the viewer renders the sealed original pane fail-closed (any access attempt = forbid audit), derivative previews as audited views with tiled identity watermark; scoped download is an egress gate requiring approval. *(HANDOFF §11; AGENTS 34, 100, 145)* — **Wiring:** real streaming/transcoding backend; server-enforced original/derivative separation.
- **DOC-5** In-console media/ZIP handling: photos preserve EXIF/geo/device + in-image OCR; video keeps original + H.265 derivative + keyframe index; ZIP originals kept WORM with server-side safe extraction (zip-bomb, path-traversal, nested-recursion defenses) → read-only entry tree with per-entry hashes. *(HANDOFF §11; DESIGN §6-8)*
- **DOC-6** The in-console office editor is a governed shell over a HEAVY ONLYOFFICE/Euro-Office fork: the host console owns storage/ontology/PBAC/versioning/sharing/audit/approval, the editor is only a canvas; every save = immutable version, rollback = non-destructive restore AS a new version; per-capability PBAC (edit/review/comment/fillForms/download/print/copy each Cedar-evaluated); fork internals: per-edit-operation audit hooks, server-side covert-section render blocking, DLP, classification labels, passkey session integration; AGPL-3.0 source-disclosure compliance review required. *(HANDOFF §12; DESIGN §6-9; AGENTS 34②)* — **Wiring:** actual DocumentServer fork; the current canvas is the shell contract only.
- **DOC-7** Collaborative sheets close the loop internal DB (live query) → pipeline → sheet edit (inline cells, live Σ, presence) → immutable version save → object load through the expectation gate → back into internal data; "open as sheet" generalizes to every module list header. *(AGENTS 132, 136; TODO item 33)* — **Wiring:** real collaborative-editing backend (concurrency/codec = HANDOFF §12).
- **DOC-8** Sensitive/legal document viewing requires passkey identity verification because the view itself is receipt evidence. *(DESIGN §4.8 passkey)*
- **DOC-9** Approval evidence leads with structure: structured lines (item/amount/vendor/date) with the raw file as an audited auxiliary chip; records lead with the object card. *(TODO line 207; DESIGN §4-13)*
- **DOC-10** Target bar: Foundry-Docs/M-Files/iManage-grade records extended to media + ZIP archiving. *(ROADMAP §4 docs)*

### mail (7)

- **MAIL-1** Mail backend = mox (Go, MIT — deliberately favorable license vs the office editor's AGPL): all-in-one secure mail server (IMAP4rev2+extensions, SMTP, SPF/DKIM/DMARC, MTA-STS, DANE, DNSSEC, ACME TLS, per-account encrypted storage, webapi/webhooks) with our own console mail UI (rail↔main, Gmail benchmark); access via webapi/webhooks first + IMAP4 + SMTP submission, JMAP when ready. *(HANDOFF §14; DESIGN §6-10)* — **Wiring:** real mox server integration, not a mail mock.
- **MAIL-2** mox is modified to internalize enterprise governance: console §7 audit events on view/send/delete/move/forward/export (with deviceCtx); Cedar PBAC on mailboxes/shared mailboxes/delegation with covert deny-by-omission + passkey for sensitive reads; compliance (retention, litigation hold, journaling as immutable archive copies, e-discovery → §11 WORM); DLP (outbound content scan, attachment blocking, S/MIME, sensitivity labels, watermark); ontology integration (Mail = CommObject with token grammar, attachments → DX- ingest / EvidenceRecord, mail ↔ AP-, thread state = object). *(HANDOFF §14; AGENTS 2026-07-04 (8)(9))*
- **MAIL-3** Outbound mail passes the egress gate: attachments are registered objects with lifecycle chips; external × unapproved/sensitive = blocked panel + anomaly audit + compliance alert; the only escape is a reasoned DoA exception request that keeps the block. *(AGENTS 2026-07-08 (1), 49①; TODO line 148)*
- **MAIL-4** Gmail parity without losing governance: conversation threading (subject normalization, collapsed priors), j/k/Enter/e keys scoped to folder×filter×search, scheduled send passing the identical egress gate with cancel-back-to-drafts, mobile swipe archive — everything audited. *(AGENTS 33⑦, 102②, 111)*
- **MAIL-5** Sender authentication (SPF/DKIM/DMARC), TLS and storage encryption are surfaced truthfully as security panels — real verdicts, not badges. *(AGENTS 2026-07-04 (8); ROADMAP §7 메일)* — **Wiring:** real auth verdicts and DLP evaluation.
- **MAIL-6** Mail attachment primary CTA = "인제스트로 구조화" / "증거 등재" creating real DX-/EV- objects. *(TODO line 123; DEMO §4)*
- **MAIL-7** Empty folders hidden (Gmail parity); composer target typeahead searches graph + people. *(TODO item 12; AGENTS 175)*

### messenger (6)

- **MSG-1** Messenger is Slack/Teams parity as one stateful surface: 3-tier sidebar (# channels semi-permanent / meetings LIVE·ending into preserved read-only MT- objects / DMs with presence and DND-linked status), grouping, unread dividers, ack toggles, quotes, per-thread mute (personal setting), reply-in-thread (replies belong to the message object), huddles whose end leaves an audited system message. *(AGENTS 2026-07-08 (7)(8), 50, 103, 110④; ROADMAP §8 메신저 패리티)* — **Wiring:** real messaging backend; simulateReplies is production-banned; real-time multi-user (WebSocket) + presence + push are named structural gaps.
- **MSG-2** Messages are ontology-aware: object codes auto-link, first code unfurls a mini object card (single resolver, Teams grammar), @mentions carry a notification+audit contract, messages convert to todos, composer autocomplete draws from the single policy-filtered source. *(AGENTS 2026-07-08 (2a)(8), 44②)*
- **MSG-3** Meetings are MT- objects with structured notes/decisions/actions→todos, preserved read-only after ending. *(AGENTS 50; ROADMAP §8)*
- **MSG-4** DND is a real toggle (mute-all) in settings; presence status is self-set and linked to it; thread-mute excludes badges including mobile totals. *(AGENTS 71, 102②)*
- **MSG-5** Typing WO-/AP- codes in channels produces live lifecycle links (DEMO cross-module chain). *(DEMO §4)*
- **MSG-6** Recruiting-scoped DM for external candidates; no internal comms rail exposure. *(AGENTS 38⑤)*

### notif (4)

- **NOTIF-1** Notifications are pointers to objects, never standalone text: rows resolve codes into real content (title·requester·deadline·evidence·direct action panel) under the viewer's authorization. *(DESIGN §2; AGENTS 32①; TODO my-work lines)* — **Wiring:** notification payloads carry real object references resolving per-principal.
- **NOTIF-2** Condition-based SLO alerts auto-resolve when the underlying condition is fixed (e.g. vacancy filled). *(AGENTS 47②, 99)*
- **NOTIF-3** Object watch (Foundry subscribe parity): watching an object pushes a notification with a live link on any transition the watcher didn't perform; subscribing/unsubscribing is itself audited. *(AGENTS 162)* — **Wiring:** server-side subscriptions on real event streams.
- **NOTIF-4** Swipe toggles read state on mobile; personal automation may act on own notifications only. *(AGENTS 32①; HANDOFF §6)*

### board (2)

- **BRD-1** Notices carry receipt accountability: NT- rows show confirmation progress (done/total) via a generic prog field reusable by any module (100%=ok, else warn). *(AGENTS 30; TODO line 170)* — **Wiring:** real read-receipt counts.
- **BRD-2** Target bar: Confluence/Slack-grade notice board joined to the object graph. *(ROADMAP §4 board)*

### mywork (6)

- **MW-1** "My work" is a live personal aggregation, never a copy: my-turn approvals/dispatch, in-progress submissions, receipt confirmations, unread notifications, assigned WOs derive from the same state as their source screens via single definitions, owner-scoped per persona. *(AGENTS 14, 25, 42)*
- **MW-2** The calendar is an ontology view: date-keyed engine with list/week/month tabs; derived events (paydays, approval deadlines) come from objects and link back; todos target dates with completion toggles; click = full edit modal. *(AGENTS 36③, 40, 133)*
- **MW-3** Tasks are structured objects (priority, defer-to-tomorrow, promote-to-approval), not text lines; recurring tasks deliberately link to scheduled jobs (no duplicate mechanism). *(AGENTS 130, 133)*
- **MW-4** One global authoring entry: 「+ 만들기」 type-select modal (10 tiles) replaces scattered text boxes; every tile chains into a real structured builder; progressive disclosure = quick create with optional detail. *(AGENTS 134/135)* — **Wiring:** each tile creates real objects via real APIs.
- **MW-5** All six staging families (workflow, schedule, ontology definition, type schema, mapping template, data override) converge into ONE central proposal review queue whose approve/withdraw call existing chokepoint methods (inheriting audit/version/gates), scoped to sensitive+ reviewers, self-approval double-blocked. *(AGENTS 109)*
- **MW-6** Field check-in/out lives in my-work for field personas. *(ROADMAP §8 어고노믹)*

### ontology (explore / engine) (10)

- **ONT-1** Type definitions (OT-) and relation types are lifecycle objects: proposal sandbox → data-governance/executive SoD review → active schema v+1 → revision → deprecate; archive gate = migrate referencing instances + rebind automations/policies; attribute/relation removal = deprecated marker + 30-day sunset then archive; as-of reconstructs past definitions; breaking changes cannot bypass approval. *(DESIGN §4-17; HANDOFF §18.2; AGENTS 101④)* — **Wiring:** versioned schema registry with migration/rebinding enforcement server-side.
- **ONT-2** ObjectType registry schema: {typeId, label, version, stage, propSchema[dataType text|money|percent|date|enum|lifecycle|person|number], linkTypes[{fromType, toType, cardinality 1:1|1:N|N:1|N:N}], actions, analytics}; linkType cardinality is the backend basis for referential integrity and join validation; links are TYPED (free-string rels are a migration target). *(HANDOFF §18, §18.2)*
- **ONT-3** Actions are writeback functions (Palantir Actions): server-side functions bound to types that mutate or create derived objects (contract "갱신 검토 draft" → AP-; attendance "대근 편성" → Substitution+AP-), ALL policy-evaluated + audited + guardrailed; action forms declare params + submission criteria (live-judged clearance predicates, unmet = forbid fail-closed) + side effects, edited via staged revision. *(HANDOFF §18; DESIGN §4-20 개체; AGENTS 110①)* — **Wiring:** action invocation endpoints with full audit per action.
- **ONT-4** Analytics are derived properties: expr compiles to aggregate queries (margin = contract amount − labor − overhead; labor = Σ PayItem), exposed read-only on dashboards and cards. *(HANDOFF §18)* — **Wiring:** real-data derivation from payroll/attendance aggregates.
- **ONT-5** The explorer is a working graph: radial typed traversal with recenter/trail, always-visible authoring strip (new-object wizard, relation by code/drag, monitor rule), layer-grouped collapsible legend doubling as type-card entry, live coloring lenses (type/lifecycle/audit-activity) that recolor without moving structure and audit the switch; full type-manager IDE workspace (props, relations+cardinality, actions, analytics, live instances, automations) under revision-staging banners. *(AGENTS 63, 66, 108; TODO lines 123·129)*
- **ONT-6** Object mutation is stage-governed: drafts edit directly; post-draft objects change only via data override — reason required fail-closed, auto approval line, pending state with current values live, four-eyes apply (self-approval blocked) merging while audit-preserving prior values. *(AGENTS 61, 79; HANDOFF §20)*
- **ONT-7** Creation closes the loop: typed creation wizards (schema fields, initial relations ≥1 mandatory so every new type joins the chain, fail-closed naming) cover the full type registry (35 types); wizard-created objects auto-join their type-matching module surface from the single instance store. *(AGENTS 114, 124, 171; HANDOFF §20)* — **Wiring:** real object store backs module lists; MOD_SCREENS seed derivation → engine queries is the named backend contract (lane 15).
- **ONT-8** Series promote from repetition: recurring instances promote into SR- with attach/trend recomputation and auto-detection proposals from repeated similar drafts. *(AGENTS 84, 36⑥)*
- **ONT-9** Typed creation wizards show quantitative projection: money/percent inputs yield deterministic point estimate·CI95·CVaR95 with stated assumptions (no AI). *(AGENTS 68)* — **Wiring:** backend Monte Carlo·EVT/GPD tail fitting; recomputable from real inputs.
- **ONT-10** Field-level classification labels (general→personal→sensitive) are ontology metadata propagating to masking/egress/context gates. *(AGENTS 157/157a)* — **Wiring:** classifications drive real masking/egress enforcement.

### policy (Cedar PBAC surface) (8)

- **POL-1** Policy authoring is no-code but real: who→what→action blocks with permit/forbid and condition toggles, auto-generated natural-language rule text, simulation that actually evaluates typed predicates over sample access events producing real permit/forbid counts; draft/revision v+1 saves audited; policy vocabularies auto-derive from the ontology so extending the registry extends what policy can govern. *(AGENTS 2026-07-08 (4), 77, 56b; HANDOFF §2)* — **Wiring:** rule compiler + simulation over the real entity store; demo §5-5 requires persona-switch render changes from real Cedar evaluation.
- **POL-2** Two access axes: semi-permanent attribute-based Cedar rules + one-shot TTL/single-use grants as first-class TK- token objects (issuer/scope/expiry/reason/uses/state), fail-closed issue form, revoke = immediate effect with finalize audit and non-destructive history (PAM JIT / break-glass). *(AGENTS 153; TODO line 287)* — **Wiring:** issuance/revocation are real authz artifacts consumed by enforcement.
- **POL-3** Context-driven access control (ctxGate): unmanaged devices are blocked fail-closed (forbid CTX audit) from sensitive person detail, entity finance, and covert entry — orthogonal to clearance (secret clearance on an unmanaged device is still denied); device/geo/network/time rules + current-context card. *(AGENTS 154; HANDOFF §9 Device/Context)* — **Wiring:** real MDM/device-management and network signals.
- **POL-4** Covert zone is a nav-level deny-by-omission surface (item exists only for secret clearance), entry itself audited and Cedar-guarded fail-closed; contains executive compensation ledger (per-click view audit) and a live sensitive-access/forbid stream; deliberately absent from search. *(AGENTS 141)*
- **POL-5** Console configuration and layout deployment are themselves policy scopes (console:configure internal-only, console:deploy sensitive+) with deny-by-omission hiding. *(AGENTS 86)*
- **POL-6** Welfare/DoA/handover decision parameters are editable versioned policy objects whose rule labels derive from parameters, changed only via four-eyes approval bumping the version. *(AGENTS 180/180a)*
- **POL-7** `[>190]` Designed next slice (TODO #39): rule builder "policy link" row, new DoA clause authoring (clone seed, E2 numbering, draft→four-eyes apply=active), settings-object ledger on the policy screen (TK- ledger grammar), non-destructive archive — closes the automation logic-object L2 deficit. *(TODO item 39 + line 74)*
- **POL-8** Pre-policy UI contract: chips/gates placed before policies exist must flip to REAL Cedar evaluation results — never remain decorative. *(DESIGN §4.5)*

### workflow / automation / scheduled jobs (9)

- **WFL-1** The workflow studio is typed·actionable config: parameterized triggers/actions bound to ontology types, conditions as field·operator·value predicates over a typed field registry (every number/money/percent/enum property of active types is a condition candidate — schema additions instantly extend the vocabulary, N+1 closed loop), simulation performing REAL evaluation with audited pass counts, cfg persisted and rehydrated on revision. *(AGENTS 55, 65a, 181; ROADMAP §8 워크플로 스튜디오)* — **Wiring:** simulation and execution run against real object data server-side.
- **WFL-2** Automation scope is governed: personal rules activate immediately (whitelist), owner-only visible (deny-by-omission), act only on the owner's objects — policy-evaluated within the owner's resource scope so automation cannot privilege-escalate; company rules require draft→publish approval; active-rule edits go through pending-revision staging; every execution/creation is audited. *(HANDOFF §6; AGENTS 16, 32②)*
- **WFL-3** Runs produce an n8n-style execution timeline with created-object chips and error retry; execution creates REAL objects (AP-, notifications) subject to the same policy evaluation and audit — automation is not a bypass lane. *(AGENTS 59c; HANDOFF §6; ROADMAP §3 자동화)*
- **WFL-4** Cron scheduled jobs are first-class governed objects: natural-language schedule, next-run preview, execution log, failure retry, draft→cadence-edit→activate(=publish)→revise v+1→archive; covers attendance-close reminders, monthly payroll-run creation, retention expiry, leave-promotion batch, periodic reports (Airflow/Temporal benchmark). *(HANDOFF §6; AGENTS 161, 166)* — **Wiring:** real scheduler; DAG dependencies/backfill/SLA-miss alerts are named gaps.
- **WFL-5** Named protocols are reassemblable from generic blocks: wf7 handover, wf8 after-hours P0 break-glass (deterministic severity split — only P0 pages; compensated on-call rotation "no arbitrary unpaid calls"; timed no-response auto-escalation; DoA emergency pre-action within a monetary limit with automatic post-hoc approval + next-business-day four-eyes), every judgment value a configurable parameter. *(AGENTS 179, 181)* — **Wiring:** real on-call rosters, escalation timers, break-glass approval records.
- **WFL-6** Workflow ↔ explore are bidirectional: rules expose object chains into the explorer; the explorer offers type-trigger monitor rules; every module detail shows the automations touching its type with "+ monitor". *(AGENTS 2026-07-08 (1c), 63b, 65b)*
- **WFL-7** Example canonical rules: 3 unexcused absences → HR alert + explanation draft auto-created; leave usage <20% & Jul 1 → promotion round 1 auto-send (Workato/ServiceNow Flow benchmark). *(HANDOFF §6)*
- **WFL-8** Every new module's events become trigger candidates automatically. *(ROADMAP §3 자동화)*
- **WFL-9** `[>190]` wf10 new-lead round-robin: trigger = deal created without owner, condition = inflow ≠ renewal (renewals belong deterministically to wf9 owner-succession — no rule overlap), action = fixed rotation (same input = same output) + first-activity deadline. *(AGENTS 193)* — **Wiring:** assignment executed server-side against a real rotation roster; run log real.

### ingest / pipeline (9)

- **ING-1** Ingest is a deterministic, provenance-first, no-AI pipeline (Rust): sources (11 file types + photo/video/ZIP/arbitrary + external APIs) → parse/OCR (calamine, quick-xml, pdf-extract/lopdf, docx-rs, leptess/Tesseract, EXIF, ffmpeg) → sanitize (normalization, Great-Expectations-style validation, PII regex/dictionary) → classify (structural signature + keyword rules) → map (regex/anchors, gazetteer, fuzzy, type coercion, confidence) → human review of low confidence → commit (typed ontology object + provenance/lineage + back-refs + audit + classification); all templates/rules/statistics. *(HANDOFF §10; DESIGN §6-7; AGENTS 2026-07-04 (5)(6))* — **Wiring:** real parsers/OCR/connectors; loads are real ontology writes; the UI's pipeline progress is simulation (§0-⑦) — real stage events required.
- **ING-2** IngestJob DX- has a staged lifecycle (uploaded→parse→sanitize→classify→map→review→committed|failed) with per-field {label, raw, val, conf, tgt, status, pii, provenance}; every stage transition is an audit event; provenance/lineage is per VALUE (source doc, region/cell/path, transform step — Foundry Data Lineage). *(HANDOFF §10)*
- **ING-3** Mapping templates TP- are reusable versioned no-code objects: src→tgt rows, transform enums, usage drill, draft→publish fail-closed, active revisions four-eyes v+1, archive blocked while in use; every job shows a 4-node lineage strip (source→transform/template→validation→object), all drillable. *(AGENTS 101①; HANDOFF §10)*
- **ING-4** Data expectations gate the load: Severe failures (mapping completeness, required values) block commit fail-closed with forbid audit naming reason+resolution; Moderate failures warn-and-proceed with audit (Foundry Data Health parity). *(AGENTS 107)*
- **ING-5** Source connectors are objects {kind file|api|db|sftp|queue, auth, cadence, status} with auth, rate-limit, schema-drift and retry handling; source builder creates pending-connection sources — credentials/approval explicitly backend. *(HANDOFF §10; AGENTS 124)*
- **ING-6** Pipeline Builder follows Foundry grammar (Inputs→Transform→Preview→Deliver): preview and build share the same code path (reproducibility), 0-row builds fail-closed; inputs include live internal ontology queries (employees, attendance issues, contracts, audit events) reacting to state changes; dual output (external ingests → review queue; internal analyses → AN- objects); branches = non-destructive draft saves; 6 deterministic expression types. *(AGENTS 130-131, 137-138)* — **Wiring:** transforms execute on real inputs server-side.
- **ING-7** Ingest is exposed as a workflow trigger ("new ingest record"), supports scheduled polls, auto-commits above confidence threshold. *(HANDOFF §10)*
- **ING-8** Signature demo: bank transactions(API DX-)→Voucher→Ledger; procurement notice(DX-)→Bid→Contract candidate; contract scan(DX-)→auto-mapped C- fields. *(ROADMAP §5-2)*
- **ING-9** P1 phasing: the data spine (ingest→explore→automation) builds first because it most strongly proves ontology/correlation/workflow. *(ROADMAP §6)*

### dashboard (7)

- **DASH-1** The dashboard is display-only over live queries: widgets are ontology QUERY BINDINGS ({count|trend|dist, bind}) that self-maintain; current period computed live from single sources, closed periods are legitimate archived snapshots, KPI deltas compare to closed snapshots; computation is owned by ontology/automation, never the dashboard. *(AGENTS 67, 70, 94, 96; ROADMAP §7 2026-07-10 ②)* — **Wiring:** dashboards read the same real stores as source modules — no dashboard-local data; deltas vs a real closed snapshot.
- **DASH-2** No dead numbers: every figure drills into source objects; scope×period is PBAC-segmented with as-of and honest empty states; the collapsed pipeline strip renders recruit→attendance/cover→leave→approval→pay-round→analysis-feedback as six live-computed drillable stages. *(AGENTS 32⑤, 60a, 102④)*
- **DASH-3** The console itself is a reconfigurable canvas over the ontology (§19): component add/edit, column add/reorder from the type's propSchema, behavior selection reusing the pin/window model; the CONFIG is a versioned/approved/audited ontology object (vs Retool's vendor-locked JSON) — personal views direct (whitelist ①), shared/deployed layouts draft → approval → effective with rollback + as-of; PBAC evaluates components/columns (sensitive columns deny-by-omission). *(HANDOFF §19; AGENTS 59-60, 83, 113)* — **Wiring:** component store persistence, shared-layout deployment approval, generic chart/timeline/kanban binding are named residual backend.
- **DASH-4** Ad-hoc chart builder (Quiver parity): only types with instances offered, aggregation runs live off the single store, bars drill to instances, chart saveable as an AN- object with evidence chain. *(AGENTS 164)* — **Wiring:** aggregation = real authorized queries.
- **DASH-5** Config-mode widgets (distribution bars, kanban) derive from the current list query with every segment/card a row drill. *(AGENTS 113)*
- **DASH-6** Executive persona: dashboard → contract profitability drill → labor cost → DoA approvals → audit stream, every figure drillable. *(ROADMAP §8 임원)*
- **DASH-7** Honest chart scaling and empty-state truth apply to every widget (see C-40, C-30). *(DESIGN §4-24)*

### analysis / forecast (6)

- **AN-1** Analysis, monitoring and automation are one body: the analysis screen authors threshold monitor rules directly, monitor rows derive live from active workflow definitions (drill to studio, instant simulation on current data), AN-/FC- insights are derived objects with evidence chains, prescriptive actions, and "rule-derived, recomputable" provenance. *(AGENTS 62a, 67b, 88, 122)* — **Wiring:** watch rows from real active workflow defs; simulation evaluates real data server-side.
- **AN-2** All authoring is fully interactive structured builders — never free-text stubs: AN- = evidence-object multi-select × deterministic formula × threshold with live preview and fail-closed insufficiency notice; FC- = deterministic model (EWMA/Poisson/M/M/c) × input series × window with model-derived preview; creations are typed objects joining lists/graph/lifecycle. *(AGENTS 123; DEMO §6)* — **Wiring:** previews computed from real series; objects persisted as typed ontology objects.
- **AN-3** Quantitative judgment is deterministic and honest: stated models (EWMA·student-t·CVaR95·logit·Poisson·M/M/c) with visible assumptions; verdict chips on the face, methodology in kv; what-if sliders reproject live (generic sim field); portfolio margin×risk scatter from the single contract source. *(AGENTS 68, 69, 75, 102③)* — **Wiring:** backend Monte Carlo/EVT; every figure recomputable from real inputs.
- **AN-4** Sovereign pricing: "we set our own prices and choose which fights to take — quantitative data always"; bid decision = proposal band, win curve, cost stack, CVaR gate, fail-closed unit-price floor; renewal = raise band, churn threshold, evidence chain, negotiation draft; verdicts legible to non-technical operators. *(TODO line 120; AGENTS 69)*
- **AN-5** Forecasting is RULE-BASED scenario forecasting — no AI (Anaplan/Foundry benchmark). *(ROADMAP §4 forecast)*
- **AN-6** laborcost: profitability table is a live-derived closed loop — substitution staffing→labor cost→margin recomputes live with vacancy chips; per-contract breakdown and forecast (Foundry-Contour/Adaptive grade). *(AGENTS 112; ROADMAP §4 laborcost, §5-1)* — **Wiring:** derived, never hardcoded, profitability over real joins.

### finance / purchase / inventory (ERP) (7)

- **FIN-1** Ledger integrity is enforced by document flows: voucher entry→debit/credit verification→approve→post with imbalance blocking posting fail-closed; account drill back to source objects; period-close checklist per run lifecycle; auto journal entries from payroll·workflow·AP-. *(AGENTS 63c; ROADMAP §4 finance)* — **Wiring:** real double-entry ledger; multi-currency/period close/tax filing are named gaps.
- **FIN-2** Purchase follows the SAP flow PR→PO→GR→IR→pay with 3-way match blocking payment on invoice/PO mismatch; partial GR/IR matching pays only min(GR,IR) with remainder open — each with exception-approval CTAs; flow/match/recon are generic reusable fields. *(AGENTS 64a, 75b, 110②; TODO line 131)*
- **INV-1** Stock is derived, never asserted: current stock = sum of movement documents (receipts, issues, reservations, scheduled) linked to source PO/WO; rendered as quantity-bar matrices with safety-stock ticks and shortage danger states. *(AGENTS 26, 64c; TODO lines 134·180)* — **Wiring:** document-derived stock from real movements.
- **INV-2** MRP is a deterministic requirements formula (stock×incoming×reserved×usage×coverage) proposing order quantities consistent with existing objects (single source). *(AGENTS 110③)*
- **INV-3** Target bar: SAP-MM/Fishbowl-grade IV- with safety stock, recon matrix, MRP. *(ROADMAP §4 inventory)*
- **FIN-3** Registered SAP gaps to burn down: valuation methods, payment terms, partial GR/IR reconciliation depth, auto double-entry, MRP optimization. *(TODO register; BENCHMARK row 9)*
- **FIN-4** Palantir-flavor end state: every business document is a typed graph node with writeback actions, automation triggers, and a profitability feedback chain. *(TODO line 136)*

### maintenance (4)

- **MNT-1** Maintenance is an order cycle on every WO-: intake→plan (parts reserved from inventory, PO drill)→execute (journal JL-)→settle (cost settlement→voucher) with document-chain round trips into purchase/inventory. *(AGENTS 32③, 63, 64b; TODO lines 135·172)* — **Wiring:** real reservation/settlement transactions across modules.
- **MNT-2** Shortage auto-proposes a PO via automation; SLA queue kanban synced to row selection. *(TODO line 104; AGENTS 77-78)*
- **MNT-3** Target bar: UpKeep/Fiix/SAP-PM-grade WO- orders reusing the shared WO- object + processing panel. *(ROADMAP §4 maintenance)*
- **MNT-4** Evidence chain demo: field photo/video (EvidenceRecord WORM) → WO- → contract fulfillment proof → audit. *(ROADMAP §5-6)*

### equipment / asset (3)

- **EQ-1** Assets (FL-) render as lifecycle timelines: acquisition→maintenance events→planned return/replacement dashed, every referenced code drills; assets revise through the standard lifecycle like every object (proven e2e). *(AGENTS 26, 118; TODO line 180)*
- **EQ-2** Target bar: ServiceNow-ITAM/EAM-grade FL- assets (GPU, rental) linked to WO- and C-. *(ROADMAP §4 asset)*
- **EQ-3** Purchase drafts link asset/inventory/contract objects at compose time (typed link enforcement). *(DESIGN §4.8 기안 유형별 연결)*

### dispatch / map (5)

- **DSP-1** Dispatch is a matching surface: WO- queue × candidate drivers × SLA with processing panel and bidirectional map round-trip ("view on map" ↔ marker = dispatch panel); stats derived, not hand-fed. *(AGENTS 14-15; TODO lines 195-196)*
- **DSP-2** Dispatch persona flow: WO- queue with SLA chips → available-driver matching → assignment approval → tracking → settlement linkage. *(ROADMAP §8 배차 담당)*
- **MAP-1** The operations map is an interactive control surface: overlays (coverage/issues/contracts/maintenance/dispatch) driving queue panels with action CTAs, pulsing site markers, unit layers (forklifts/drivers/bus), site summary cards, right-click quick actions; schematic terrain from tokens — no external tile dependency. *(AGENTS 15, 20, 35)* — **Wiring:** real coordinates + live unit telemetry are backend.
- **MAP-2** Map authoring is governed: marker drag placement and "+site" proposals confirm via reorg approval. *(TODO lines 194-195)*
- **DSP-3** Target bar: Samsara/Geotab/Onfleet-grade queue with map round-trip. *(ROADMAP §4 dispatch)*

### field (4)

- **FLD-1** Field ops tie sites×contracts×SLA with client CL- objects in the trade chain and per-row map presets (ServiceNow-FSM grade, linked to contract, attendance, CS-). *(AGENTS 24, 47①; ROADMAP §4 field)*
- **FLD-2** Field persona (mobile forklift driver): check-in → receive WO- → work journal JL- → overtime AP- → own payslip/inbox — owner-scoped data, viewer-gated mobile tabs. *(ROADMAP §8 현장직)*
- **FLD-3** Incoming WO-/CS- orders are honestly external-event-sourced (automation/ingest), never console-authored fiction. *(AGENTS 167)* — **Wiring:** external orders arrive via real event sources.
- **FLD-4** Field SLAs are contract-bound SLA (not SLO) — breach severity is egress-grade (C-56 applies). *(DESIGN §4-26)*

### logistics (WMS) (5)

- **WMS-1** Zone▸bin typed locations with capacity, "1 lot per bin" commingling rules, and bin lifecycle (available/damaged/QA-Hold); WMS = bin-level truth EXTENDING IV- movement documents with bin coordinates (ERP stays warehouse-level — no parallel stock truth). *(TODO item 41-②; AGENTS 184)*
- **WMS-2** Directed putaway is a deterministic ruleset settings object (product family, ABC velocity, FIFO) per §4-28. *(TODO item 41-②)*
- **WMS-3** Scan confirm = §4-29 postflight; system≠physical mismatch = exception object. *(TODO item 41-②)*
- **WMS-4** Zone-by-zone cycle counts run without stopping operations (scheduled job). *(TODO item 41-②)*
- **WMS-5** Vendor-suite depth is translated into existing console grammar — typed objects, transition audits, settings objects, drills — never bolted on as foreign patterns; `[>190]` execution order places WMS after CRM under the one-module-at-a-time charter. *(AGENTS 184; TODO item 41 header)*

### manufacturing (MES) (4)

- **MES-1** Process-step objects distinct from WO- (input→process→inspect→output) with genealogy (ISA-95). *(TODO item 41-③; AGENTS 184)*
- **MES-2** Quality Hold/Release syncs to bin/lot status; release = four-eyes. *(TODO item 41-③)*
- **MES-3** OEE = availability×performance×quality as a derived stat, every number drillable. *(TODO item 41-③)*
- **MES-4** Andon events P0-P3 join the wf8 incident protocol. *(TODO item 41-③)*

### crm (6 — all `[>190]`)

- **CRM-1** `[>190]` CRM is a typed-object sales pipeline: DL- deal type joins the ontology (relations to account/converted contract/forecast evidence; weighted-amount and cycle analytics defined); stages are lifecycle flow lanes with Lost explicitly NOT a stage; all stats derived (weighted pipeline = Σ size×win-rate); inflows carry provenance links to source events. *(AGENTS 191/191a)* — **Wiring:** deals/stages/stats from real deal records and real event provenance.
- **CRM-2** `[>190]` Activity-based discipline: "no next activity" is a danger state with inactivity stats; a deterministic auto-Lost policy is a settings object; Closed-Lost requires a reason enum feeding monthly AN- pattern analysis. *(AGENTS 191)*
- **CRM-3** `[>190]` Stage transitions are actions requiring a per-stage evidence enum (fail-closed, audited, really transitioning state). *(AGENTS 192)*
- **CRM-4** `[>190]` The contract lifecycle closes its loop: expiry D-90 auto-creates a renewal deal (duplicate-prevented, profitability evidence attached, owner succeeded via wf9, activity deadline set) so sign→active→expiring→renewal→re-sign is continuous. *(AGENTS 192/192a)* — **Wiring:** renewal automation on real contract expiry events with real dedupe.
- **CRM-5** `[>190]` New-lead assignment is deterministic round-robin wf10 (see WFL-9); renewals deterministically belong to wf9 — no rule overlap. *(AGENTS 193)*
- **CRM-6** `[>190]` Won deal auto-converts to contract C- through the guarded composer, joining the Bid- chain; large-deal manager alerts with the threshold as a settings object. *(TODO item 41-①)*

### contract (5)

- **CON-1** Full value chain from the contract: C- → positions (site×job×title×TO) → policy presets (inheritance corp←site←job←position; presets are reusable objects, not settings) → one-click posting → hire → attendance → payroll → labor-cost analysis → contract profitability feedback. *(DESIGN §2 계획; ROADMAP §5-1)* — **Wiring:** each arrow is a real server-side derivation/creation hook.
- **CON-2** The contract→position→preset chain editor visualizes inheritance (muted) vs override (accent) vs pending revision (warn dashed); chip click edits the override, staged via §3.9 so current values stay effective until approval. *(AGENTS 148)* — **Wiring:** presets/overrides are real config objects with approval-gated effectivity.
- **CON-3** Upstream chain backlog: government grants (discovery→eligibility matching→application-as-AP-→disbursement→settlement→post-management→obligation tracking) + bidding/procurement (나라장터/SAM.gov benchmark) — all coded, tokened, audited, PBAC'd objects with jurisdiction branches and subsidy-audit readiness. *(TODO line 310; ROADMAP §4 contract)*
- **CON-4** Contract profitability feedback is the signature demo: ContractProfitability→LaborCost→PayrollRun→Attendance/Substitution→WorkforcePool drillable up/downstream on one screen. *(ROADMAP §5-1)* — **Wiring:** real derived-metric chain over real joins.
- **CON-5** CX/sales persona: external mail (CS- quote) → guardrailed contract drafting → posting/staffing chain. *(ROADMAP §8 CX/영업)*

### compliance (8)

- **CMP-1** Compliance is obligations-as-objects: CP- items cross obligation×legal-basis×target-objects×fulfillment-drafts×evidence×monitoring-workflows as typed ontology links, with the Vanta-style control→evidence matrix drilling into live features; backend-only requirements labeled honestly. *(AGENTS 33⑧, 58b; HANDOFF §18.1)*
- **CMP-2** Data-subject rights (PIPA §35/§36/§37/§35-2) are self-service first-class DSR- objects with statutory D-10 deadlines flowing through the approval pipeline; passkey identity verification reused. *(AGENTS 121; TODO line 282)* — **Wiring:** real DSR request store; resolution actually changes processing.
- **CMP-3** The consent ledger keeps per-item version/time/scope; voluntary consents withdraw with immediate effect and preserved history; legally-mandated processing shows a statutory-basis chip instead of a fake consent toggle. *(AGENTS 121)* — **Wiring:** withdrawal must actually change processing effect; external-agency integration is backend.
- **CMP-4** Live transparency registers: CP-016 processing register (PIPA §30 — purposes × items/retention/processors/cross-border, each row drilling to the owning module, legal review → passkey publication) and CP-017 safeguard checklist mapping each statutory requirement to the console mechanism satisfying it, honest about unmet ones ("backend pending" naming the dependency). *(AGENTS 146, 155)* — **Wiring:** register from real processing metadata; compliance posture truthful, never aspirational.
- **CMP-5** Jurisdiction is an object: per-법인 jurisdiction chains (local ordinance→national→international) drive applicable regulation codes (real codes only — invented codes were purged); Cedar evaluates jurisdiction as context choosing the most-protective rule; cross-border transfer denied without adequacy/SCC; audit records jurisdiction+residency. *(AGENTS 158; TODO line 280)* — **Wiring:** jurisdiction→regulation mapping is real reference data.
- **CMP-6** Regulation parameters RG- are versioned executable objects with revision approval (개정 시작 → v+1 draft, current stays effective → effective date → four-eyes → takes effect). *(DESIGN §4-16; DEMO §3)* — **Wiring:** effective-dating actually switches the enforced parameter.
- **CMP-7** Standards frameworks FW- map controls to working features (see C-49). *(HANDOFF §17)*
- **CMP-8** Data classification (sensitive/unique-identifier/PII) extends to object/field level, mapped per jurisdiction. *(TODO line 281; AGENTS 157)*

### audit (module) (4)

- **AUD-1** The audit screen is a live forensic surface: date-grouped feed, per-object/person/policy filters, full-text search, time range, expanded detail (policy denials, before→after, session/IP), correlation drill (session/chain), anomaly/denial coloring, stats, export as governed egress; unreviewed denials feed back into governance via the linter. *(AGENTS 2026-07-04 (2), 106⑦; HANDOFF §7)* — **Wiring:** reads the real audit store; Splunk/CloudTrail/Workday benchmark.
- **AUD-2** Anomaly detection targets sensitive-view spikes, privilege escalation, abnormal-hours access; the anomaly chip is a UI contract that must be driven by real detection. *(HANDOFF §7, §17)*
- **AUD-3** Viewing the audit log is itself audited; the CEO-only covert stream is separate. *(HANDOFF §7)*
- **AUD-4** Audit correlation demo: AuditEvent → "go to object" + "related events (session·chain)" → explore graph. *(ROADMAP §5-4)*

### support (3)

- **SUP-1** Support is an SLO-governed module for all personas including external: typed/severity SUP- tickets with SLO timers, related-screen links, FAQ→ticket escalation deep-linking into features, fail-closed submission, escalation into console-change drafts. *(AGENTS 48③, 73, 81b; ROADMAP §8 지원 센터)*
- **SUP-2** Ticket SLOs are internal targets (breach=alert+improvement), never conflated with contract SLA. *(DESIGN §4-26)*
- **SUP-3** The bottom "system" nav group holds support/settings/info as universally-allowed full screens with real toggles (DND, passkey management, audit hash-chain anchor, shortcuts). *(AGENTS 71)*

---

## 3. Do-not-ship — the authority's own stub/sim/filler bans

The authority itself enumerates what may NEVER reach production. Consolidated:

1. **HANDOFF §0 scaffold inventory (CRITICAL, feature-flag-protected removal targets):**
   ① view-as role-switch card (VIEWERS personas) → real SSO/passkey sessions + Cedar (§0-①);
   ② pkAuth 1.05s scan simulation → real WebAuthn/FIDO2 + receipt notarization (§3);
   ③ ALL seed data (EMPLOYEES, rcData, threads, mails, …) → real DB-backed data;
   ④ fixed deviceCtx ip/geo → real device telemetry;
   ⑤ client-side hash chain → server-side tamper-evident chain (§7);
   ⑥ simulateReplies messenger auto-responses → real message delivery;
   ⑦ ingest pipeline progress simulation → real pipeline stage events. *(HANDOFF §0; AGENTS 38⑥)*
2. **Mockup-independence clause:** hardcoded-only data or stub behavior is a registered gap; every visible datum must be state-derived or have a UI creation path; "backend-y" never excuses a missing path. *(DESIGN §4-25-6; README ⑥)*
3. **Filler-content ban (§4.6 안티패턴 / §4-12):** explanatory captions/subtexts, functionless labels/badges/numbers/stats/icons, dummy sections, space fillers, big KPI cards, AI-slop visuals (gradients, non-brand emoji, left-border accent cards, Inter/Roboto/Arial) — the visual face of the no-simulated-data rule. *(DESIGN §4.6 안티패턴, §4-12; README §ANTI-PATTERNS)*
4. **Truthfulness doctrine:** hardcoded counts/badges, scope-ambiguous stats, prose notifications, cross-surface count inconsistencies, invented regulation codes = defects; "coming soon" text replaced by the real capability. *(AGENTS 156, 158-159; TODO item 32b, line 362)*
5. **Decorative simulation ban:** policy/workflow simulation must EXECUTE predicates on real samples — decorative toasts banned. *(DESIGN §4-20 configurable; AGENTS 77)*
6. **Security theater ban:** no UI may promise "complete blocking"; DLP claims limited to deter+trace+gate; layer-3 prevention documented as a deployment requirement. *(HANDOFF §13; DESIGN §4.5 DLP)*
7. **nav stub = 0:** toast-only/redirect-only handlers, text-input creation stubs, unimplemented "+" buttons are defects. *(AGENTS 116, 119-120, 123-124)*
8. **Enumeration UI at scale:** full-roster dropdowns, uncapped lists, client-side search over global datasets banned — typeahead + server pagination. *(DESIGN §4-27-4; AGENTS 169)*
9. **Persistence honesty:** localStorage is not a store; session state must become real persistence per HANDOFF; client-computed aggregates over over-transmitted data banned. *(BENCHMARK §구조적 격차; HANDOFF §0/§2)*
10. **Prototype 완료 labels:** every checkbox/완료 in the mirror is prototype-layer contract completion only — claiming runtime behavior from them is itself a banned move; screen exposure is server-gated (ADR-0025, EXPOSED_SCREEN_KEYS empty, fail-closed to legacy). *(AGENTS preamble; ROADMAP preamble; TODO preamble)*

---

## 4. Blocked-on-missing-backend index

Wiring implications that name backend surfaces which do not yet exist (or exist only dark/shadow). Grouped by contract; module refs in parentheses.

| # | Missing backend surface | Named by | Blocks |
|---|---|---|---|
| B-1 | **Real server-side Cedar PBAC evaluation** on every read/write path incl. search + aggregation (currently legacy authz + RLS, Cedar target/shadow) | HANDOFF §0/§2; DESIGN §4.5 | C-19…C-24, POL-1…8, all deny-by-omission + covert intents |
| B-2 | **Ontology service**: type registry + typed instance store + engine queries replacing MOD_SCREENS hardcoding (lane 15 ⓐⓑ) | HANDOFF §18/§18.1/§18.2; AGENTS 114; 「다음」 | C-5…C-7, ONT-1…10, every module-surface row |
| B-3 | **Server-side lifecycle engine** with effective-dating, as-of temporal queries, versioning, settlement gates, freeze windows | HANDOFF §15 | C-8…C-14, ORG-1, CMP-6 |
| B-4 | **Guardrail engine**: preflight evaluation, checklist objects, egress gate, detective alerts at the API layer | HANDOFF §16 | C-15…C-18, MAIL-3, DOC-4 |
| B-5 | **WebAuthn/FIDO2 passkey** + RFC-3161 receipt-timestamp notarization for InboxDoc receipt evidence | HANDOFF §3 | INB-1…3, ATT-12, REC-4, DOC-8 |
| B-6 | **Tamper-evident audit chain**: server-side append-only seq+hash store, signing, external TSA anchoring, SIEM/OCSF export, real anomaly detection | HANDOFF §7/§9/§17 | C-25, AUD-1…3 |
| B-7 | **Rust ingest runtime**: parsers/OCR, connector auth/rate-limit/schema-drift, per-value provenance, real stage events, credential/approval flow | HANDOFF §10; AGENTS 124 | ING-1…9 |
| B-8 | **WORM evidence store**: SHA-256 + TSA + custody, transcoding/streaming, safe ZIP extraction, re-verification API | HANDOFF §11 | DOC-3…5, MNT-4 |
| B-9 | **ONLYOFFICE/Euro-Office heavy fork** (DocumentServer + audit/PBAC/DLP/covert inside the editor) + collaborative sheet concurrency/codec; AGPL compliance review | HANDOFF §12 | DOC-6…7, PAY-6 |
| B-10 | **mox mail server integration** (webapi/webhooks, IMAP4, SMTP; enterprise modifications: audit, PBAC, retention/journaling/e-discovery, DLP) | HANDOFF §14 | MAIL-1…7 |
| B-11 | **DLP layer-3 deployment** (enterprise browser/VDI/endpoint DLP/MDM) + real managed-device/network signals for ctxGate | HANDOFF §13; AGENTS 154 | C-48, POL-3 |
| B-12 | **Workflow/automation execution engine** (server-side triggers/conditions/actions over real objects, run logs, retries, queues) + **real scheduler** (DAG deps, backfill, SLA-miss alerts) | HANDOFF §6; ROADMAP §3; BENCHMARK rows 2-3 | WFL-1…9, ATT-7, PAY-4 |
| B-13 | **Statutory payroll calculation engine** (tax law, retro rulesets, multi-country) | BENCHMARK row 4 | PAY-1, L3 grade table |
| B-14 | **Quant engine**: Monte Carlo, EVT/GPD tail fitting for CI95/CVaR95 projections | AGENTS 68; HANDOFF §18 | ONT-9, AN-3…4 |
| B-15 | **Real-time multi-user backend**: WebSocket messaging, presence, huddle media, push notifications | BENCHMARK §구조적 격차 | MSG-1…5, NOTIF-3, C-59 |
| B-16 | **Scale substrate**: server pagination, indexed search/typeahead APIs, virtual scroll, audit year partitioning | DESIGN §4-27-4; AGENTS 169; BENCHMARK | C-47, C-39 |
| B-17 | **Enterprise SaaS substrate**: SSO (SAML/OIDC) + SCIM provisioning, tenancy isolation, KMS envelope encryption, OTel pipeline, SLO/error budget, IR runbooks, SBOM | HANDOFF §17 | C-24, C-49 |
| B-18 | **DSR/consent backend**: request store, consent ledger with effect-on-processing, external-agency integration | AGENTS 121 | CMP-2…3 |
| B-19 | **Access-control system integration** (photo storage, access-zone events) | AGENTS 152 | DIR-2 |
| B-20 | **Live field telemetry**: real site coordinates, unit (forklift/driver) positions | TODO map lines | MAP-1, DSP-1 |
| B-21 | **Device registration + geofence verification** for check-in/out | AGENTS 44④; TODO item 10 | ATT-3 |
| B-22 | **Real external event sources** for inbound WO-/CS- orders (automation/ingest wired to customers) | AGENTS 167 | FLD-3 |
| B-23 | **Dashboard/component config store** persistence + shared-layout deployment approval + generic chart/timeline/kanban binding | HANDOFF §19 residual | DASH-3 |
| B-24 | **Double-entry ledger + period close + multi-currency + 3-way-match transactional core** | BENCHMARK row 9 | FIN-1…2, INV-1 |
| B-25 | **TK- token issuance/revocation as real enforced authz artifacts** (TTL, single-use, break-glass) | AGENTS 153, 177 | POL-2, ATT-7, WFL-5 |
| B-26 | `[>190]` **Contract-expiry renewal automation** (real expiry events, dedupe) + **round-robin assignment roster** (wf10) | AGENTS 192-193 | CRM-4…5, WFL-9 |
| B-27 | **Read-receipt counting** for notices; object-watch subscriptions on real event streams | AGENTS 30, 162 | BRD-1, NOTIF-3 |
| B-28 | **Server-owned rollout gating + evidence-approval manifest** for screen exposure (ADR-0025) | ROADMAP preamble | Every screen's EXPOSED transition |

---

*Register totals: 64 cross-cutting + 208 per-module intents across 35 module sections (payroll 7, attendance 12, workforce 4, recruit 8, org 8, directory 3, hr 5, evaluation 3, leave/benefit 4, appr 7, inbox 5, docs/evidence 10, mail 7, messenger 6, notif 4, board 2, mywork 6, ontology 10, policy 8, workflow 9, ingest 9, dashboard 7, analysis/forecast 6, finance/purchase/inventory 7, maintenance 4, equipment 3, dispatch/map 5, field 4, logistics/WMS 5, manufacturing/MES 4, crm 6, contract 5, compliance 8, audit 4, support 3) + 10 do-not-ship bans + 28 blocked-backend contracts. Intents newer than change-log 190: C-64, POL-7, WFL-9, CRM-1…6, WMS-5 (charter framing), B-26.*
