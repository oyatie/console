> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# SAP — Delta over `docs/program/benchmark-matrix/`

Status: RESEARCH — sourced, confidence-labelled
Research date: 2026-07-29
Scope: what SAP offers businesses across the **whole SaaS portfolio**, written as a **delta** over the
existing 20-file / ~590 KB benchmark corpus. External research only; no local repo code read.

---

## 0. How to read this, and what I checked

I read `INDEX.md` and grepped all 20 corpus files before writing. Every finding below is marked:

- **NEW** — zero coverage in the corpus (verified by grep).
- **EXTENDS `<file>:<line>`** — the corpus has the row; this adds to it.
- **COVERED** — already there; §12 is a pointer list, not a paraphrase.

Verified-zero in the corpus (grep count 0 across all 20 files): `Fieldglass`, `Business Partner`,
`Ariba`, `Signavio`, `LeanIX`, `Datasphere`, `Qualtrics`, `Emarsys`, `Business One`, `ByDesign`,
`Extended Warehouse`, `Transportation Management`, `Integrated Business Planning`, `Sales Cloud`,
`Customer Data`, `Commerce`, `Footprint`, `clean core`, `key user`, `company code`,
`controlling area`, `profit cent`, `business place`, `Responsibility Management`, `genealogy`,
`serial number`, `change document`/`CDHDR`, `universal allocation`, `extension ledger`,
`tolerance group`, `position hierarchy`, `Succession`. Also `파견`/`도급`/`용역`/`contingent` = 0.

`MDG` has 30 hits but **only** as a governed-change-request benchmark (`object-platform.md:169-175,222`)
— never as a master-data-single-record story. `Concur` has 15 hits but **only** as a 1-click/glance-card
ergonomics reference (`lenses/task-flow.md:46,48`) — its approval-authority model is absent, and that
turns out to be the most valuable single finding in this document (§6.1).

### Source limitations (bounds every confidence label below)

- `help.sap.com` is a client-rendered SPA in 2026 and returns an empty document to fetchers. Primary
  sourcing leans on **`learning.sap.com`** (SAP's official training portal, static and fetchable),
  SAP's **older static `help.sap.com/doc/...`** pages, **SAP Press** (Rheinwerk, SAP's own publisher),
  and **`news.sap.com`**.
- `community.sap.com` returns HTTP 403 to fetchers; cited from search extracts only, and downgraded.
- Confidence: **CONFIRMED** = read directly in an SAP-official or SAP-publisher source.
  **LIKELY** = consistent across multiple credible secondary sources, primary not read.
  **UNCERTAIN** = single/weak source or conflicting. **UNKNOWN** = could not source.

### Caveat I did not verify

The team lead states the corpus's `OURS` columns are evidence-bound to a frontend tree that has since
been deleted. I did not read the repo, so I neither confirm nor dispute it — but it means the **vendor**
columns are the reusable part of the corpus, and the OURS columns need re-evidencing independently of
anything here.

---

## 1. Portfolio map — **NEW**

The corpus compares SAP per module lens (14 modules × 7 vendors). It never presents SAP as a product
portfolio, so nothing answers "what will we eventually be up against, and is it one product or twenty?"

`Licence` = separately purchased product vs included module. `Shares` = which master data it holds
versus reads from elsewhere.

| Product | What the business gets | Current / legacy | Separate licence? | Shares which master data | Conf | Source + date |
|---|---|---|---|---|---|---|
| **S/4HANA** (Cloud Public / Cloud Private / on-prem) | The ERP core: FI, CO, MM, SD, PP, PM, QM, PS, Asset Accounting on one journal | **Current**. 2025 grouped Public+Private under the umbrella **SAP Cloud ERP** | Core licence, per-user tiers | **Owns** Business Partner, material, cost/profit centre, G/L, asset | LIKELY | [NBS — SAP Cloud ERP renaming](https://blog.nbs-us.com/difference-between-sap-cloud-erp-and-sap-s/4hana-cloud-public-edition) — accessed 2026-07-29 |
| **SAP ECC** (ERP 6.0) | The legacy ERP | **Legacy**. EHP 6–8 mainstream maintenance ends **2027-12-31**, extended to **2030-12-31** at ~+2pp; EHP 0–5 ended 2025-12-31 | n/a | Separate customer + vendor masters (pre-Business-Partner) | LIKELY | [Rimini Street — no ECC extension](https://www.riministreet.com/blog/no-extension-to-ecc-support-2027-deadline/) — accessed 2026-07-29 |
| **Business One** | SMB ERP, separate codebase | **Current** | Separate product | Own masters; no BP model | LIKELY | [ERP Research — SAP ERP products](https://www.erpresearch.com/en-us/sap-erp) — accessed 2026-07-29 |
| **Business ByDesign** | SMB cloud ERP | **Sunsetting**: net-new sales stop **2026-04-20**; existing customers supported, no announced maintenance end. Successor = SAP Cloud ERP / Business One | Separate product | Own masters | LIKELY | [Walldorf Consulting — The End of ByDesign](https://walldorf.consulting/en/blog/the-end-of-sap-business-bydesign); [Navigator via PRNewswire](https://www.prnewswire.com/news-releases/navigator-business-solutions-reaffirms-commitment-to-sap-business-bydesign-customers-amid-sap-pricelist-update-302554651.html) — accessed 2026-07-29 |
| **SuccessFactors Employee Central** | Cloud HRIS: the employee/position system of record | **Current** | Separate PEPM; effectively all-employee | **Owns** the employee + position record; feeds every other SF module | LIKELY | [SAP Licensing Experts — EC licensing](https://saplicensingexperts.com/blog/sap-successfactors-employee-central-licensing) — accessed 2026-07-29 |
| **SF Employee Central Payroll** | Localised payroll incl. **South Korea** | **Current** | Separate PEPM | Reads EC | LIKELY (locale counts differ: 50 / 53 / 60) | [SAP News — EC Payroll 50 locales](https://news.sap.com/2023/12/sap-successfactors-employee-central-payroll-supports-50-locales/) — 2023-12 |
| **SF Recruiting / Onboarding / Performance & Goals / Compensation / Learning / Succession & Development / Time Tracking** | Each a talent or time function | **Current** | **Each its own PEPM**; can be licensed to a *subset* of users (unlike EC) | All read EC as system of record | LIKELY | [Redress Compliance — SuccessFactors modules 2026](https://redresscompliance.com/successfactors-modules-breakdown-which-hr-modules-do-you-need.html) — accessed 2026-07-29 |
| **Fieldglass** | External workforce: contingent, SOW services, profile workers | **Current** | Separate product | **Owns** the external-worker record; pushes a mirror into EC | CONFIRMED | [learning.sap.com — Contingent Staffing and Services Procurement](https://learning.sap.com/courses/introducing-sap-fieldglass/introducing-contingent-staffing-and-services-procurement) — accessed 2026-07-29 |
| **Ariba** (Sourcing, Contracts, SLP, Guided Buying, Buying & Invoicing) | Source-to-contract, compliant buying, supplier lifecycle | **Current** | Separate products per module | **Contends** for the supplier record with S/4HANA and MDG (§10.1) | LIKELY | [SAP Press — What Is SAP Ariba](https://blog.sap-press.com/what-is-sap-ariba); [SAP — strategic sourcing & contracts](https://www.sap.com/products/spend-management/strategic-sourcing-and-contracts.html) — accessed 2026-07-29 |
| **SAP Business Network** | The supplier-facing trading network Ariba transacts over | **Current** | Separate | Supplier-side accounts, not buyer master data | LIKELY | [SAP — What is Ariba](https://www.sap.com/products/acquired-brands/what-is-ariba.html) — accessed 2026-07-29 |
| **Concur** (Travel, Expense, Invoice) | Employee spend with **per-person approval limits** (§6.1) | **Current** | Separate | Own employee + approver-limit records, synced from HR | CONFIRMED | [learning.sap.com — Maintaining Authorized Approvers](https://learning.sap.com/courses/working-with-primary-configuration-in-concur-expense-professional-edition/maintaining-authorized-approvers-1) — accessed 2026-07-29 |
| **Extended Warehouse Management (EWM)** | Warehouse execution | **Current**; **embedded** in S/4HANA since 1610, or decentralised for high throughput | Basic included; **advanced needs an extra licence**, often metered on transactions | Reads S/4 material/plant | LIKELY | [ProExcellency — EWM vs embedded EWM](https://www.proexcellency.com/blogs/sap-online-training/sap-ewm-vs-s-4hana-embedded-ewm-key-differences-explained-in-2025); [SAP Licensing Experts](https://saplicensingexperts.com/sap-s-4hana-licensing-overview/) — accessed 2026-07-29 |
| **Transportation Management (TM)** | Freight planning and execution | **Current**; embedded or standalone | Metered (e.g. freight orders) | Reads S/4 | LIKELY | [SAP Press — S/4HANA TM and WM](https://blog.sap-press.com/discover-sap-s/4hanas-transportation-management-system-and-warehouse-management-system) — accessed 2026-07-29 |
| **Integrated Business Planning (IBP)** | Demand and supply planning; advanced planning moved to cloud (successor to APO) | **Current** | Separate cloud product | Reads S/4 | LIKELY | [ERP Research — S/4HANA SCM](https://www.erpresearch.com/en-us/sap-s4-hana-supply-chain-management) — accessed 2026-07-29 |
| **Sales Cloud / Service Cloud / Commerce Cloud / Customer Data Cloud** | CRM, service, e-commerce, consent + identity | **Current** — four of the five CX pillars | Separate products | **Own** customer/contact records; integrate to the S/4 BP | LIKELY | [learning.sap.com — Identifying the Solutions in the SAP CX Portfolio](https://learning.sap.com/courses/sap-customer-experience-lead-to-cash/identifying-the-solutions-in-the-sap-customer-experience-portfolio) — accessed 2026-07-29 |
| **SAP Engagement Cloud** (was **Emarsys**) | Omnichannel marketing | **Current, renamed 2026-02-19**. Emarsys Edition (unchanged) + new Enterprise Edition (cross-cloud orchestration, real-time ERP signals, Joule) | Separate | Own contact/consent data | LIKELY | [Spadoom — Emarsys now SAP Engagement Cloud](https://www.spadoom.com/en/blog/sap-emarsys-now-sap-engagement-cloud/); [SAP News — What's New in SAP CX Q2 2026](https://news.sap.com/2026/07/new-in-sap-cx-q2-2026-when-insight-is-not-enough/) — 2026-07 |
| **Qualtrics** | Experience management | **NOT AN SAP PRODUCT.** SAP sold its entire stake; Silver Lake + CPP closed **2023-06-28**. SAP is a go-to-market partner only | n/a | n/a | CONFIRMED | [SAP News — sale completed](https://news.sap.com/2023/06/sap-completes-sale-of-its-stake-in-qualtrics-as-silver-lake-and-cpp-investments-complete-qualtrics-acquisition/) — 2023-06 |
| **GRC / Access Control** + **Process Control** | SoD ruleset, access request, firefighter, UAR recertification | **Current** | Separate product | Reads roles/users from every connected system | CONFIRMED | [learning.sap.com — Describing SAP Access Control](https://learning.sap.com/courses/exploring-the-fundamentals-of-sap-system-security/describing-sap-access-control) — accessed 2026-07-29 |
| **Master Data Governance (MDG)** | Governed authoring of BP/customer/supplier/material/finance masters + a **legal ownership hierarchy** (§3.3) | **Current** | Separate | **Aims to be** the authoring source of truth ahead of S/4 | LIKELY | [SAP — Master Data Governance](https://www.sap.com/products/data-cloud/master-data-governance.html); [SAP Community — MDG innovations, S/4HANA PCE 2025](https://community.sap.com/t5/technology-blog-posts-by-sap/innovations-in-master-data-governance-with-sap-s-4hana-cloud-private/ba-p/14146957) — accessed 2026-07-29 |
| **SAP Build** (Build Apps / **Build Process Automation** / Build Work Zone) | Low-code apps, workflows + **Decisions**, and a portal | **Current** (SBPA = merged SAP Workflow Management + Intelligent RPA) | BTP consumption | No master data of its own | LIKELY | [SAP — SBPA features](https://www.sap.com/products/technology-platform/process-automation/features.html) — accessed 2026-07-29 |
| **Signavio** | Process mining + process management; ships out-of-box Datasphere/SAC content | **Current** | Separate | Reads process event data | LIKELY | [SAP News — LeanIX, Signavio, WalkMe](https://news.sap.com/2025/11/leanix-signavio-walkme-sap-transformation-excellence-summit/) — 2025-11 |
| **LeanIX** | Enterprise architecture / application-portfolio management, integrated with Signavio | **Current** | Separate | Application inventory | LIKELY | same as above — 2025-11 |
| **Analytics Cloud (SAC)** + **Datasphere** | BI and planning, and the data fabric feeding it | **Current** | Separate | Reads; SAC planning writes back to CO/FI | LIKELY | [SAP Community — Signavio Process Insights with Datasphere + SAC](https://community.sap.com/t5/technology-blog-posts-by-sap/sap-signavio-process-insights-sap-datasphere-and-sap-analytics-cloud/ba-p/13620415) — accessed 2026-07-29 |
| **Business Data Cloud (BDC)** | The 2025/26 unification layer over SAP + non-SAP data, feeding Joule | **Current, early** — announced 2025; hyperscaler integrations landing H1–Q3 2026 | Separate | A governed layer, not a master | UNCERTAIN (vendor-stage) | [BARC — SAP data & analytics 2026](https://barc.com/sap-data-analytics-2026/) — accessed 2026-07-29 |
| **BTP** (Integration Suite, extension model, Event Mesh) | The sanctioned place for custom code and integration; carries the **clean core** doctrine (§9) | **Current** | Consumption-based | None | CONFIRMED | [learning.sap.com — Clean Core Extensibility Best Practices](https://learning.sap.com/courses/practicing-clean-core-extensibility-for-sap-s-4hana-cloud/explaining-extensibility-model-best-practices_e290f382-800e-40ef-a203-85a13115f487) — accessed 2026-07-29 |
| **Sustainability Control Tower** + **Sustainability Footprint Management** | ESG metrics and reporting (ESRS, EU Taxonomy) and GHG footprint | **Current**; 2026 roadmap adds evidence management + validation rules, and imports SFM emissions "preserving native source granularity" | Separate | Reads ERP + third-party | LIKELY | [SAP Community — Sustainability Control Tower Q1–Q2 2026 roadmap](https://community.sap.com/t5/technology-blog-posts-by-sap/sap-sustainability-control-tower-q1-q2-2026-updates-amp-roadmap-highlights/ba-p/14394486) — 2026 |
| **Joule** / Business AI | Agent + assistant layer over the same transactional core; 224 agents + 51 assistants claimed at Sapphire 2026 | **Current, unproven** | Bundled / consumption | None | UNCERTAIN (vendor announcement) | [SAP News — Business AI Q1 2026](https://news.sap.com/2026/04/sap-business-ai-release-highlights-q1-2026/) — 2026-04 |
| **Fiori** | The UI layer over S/4HANA; **SAP GUI is still shipped and still strategy** | **Current, partial** | Included | n/a | UNCERTAIN on the coverage figure | [heflo — Fiori for power users](https://www.heflo.com/blog/sap-fiori-for-power-users) — accessed 2026-07-29 |

**Portfolio read, in one line:** there is **one ERP core with one journal**, and around it roughly twenty
separately-licensed products, at least five of which hold their own copy of a person or a company and
synchronise it. The suite is not one model; it is one good model plus a synchronisation estate.

---

## 2. Fieldglass — external workforce — **NEW, and the highest-value item here**

Grep: `Fieldglass` = 0, and `파견`/`도급`/`용역`/`contingent`/`contractor` = 0 (one incidental
"staffing" at `lenses/automation-ext.md:182`, about Workday's BP framework). The corpus has **no**
external-workforce coverage, and employer ≠ worksite is a live design requirement.

### 2.1 The three worker kinds, split on who directs and who pays

SAP separates external workers on exactly the axis Korean law splits 파견 from 도급/용역 on:

- **Contingent** — "workers who are procured from a supplier using a job posting and are placed into an
  existing team and/or report directly to a manager within the organization." The buyer directs
  day-to-day work; the **supplier employs, manages and pays**. Time-based. → 파견 shape.
- **Services Procurement (SOW)** — "procuring a specific service provided by a supplier, to fulfill
  contractually defined projects," with "milestones, deliverables, fixed fees, and with or without
  labor." Decisively: SOW workers "do not report directly to the organizations that contracted them, but
  to the organizations that were hired to perform the contracted service." → 도급/용역 shape, and the
  reporting-line difference is **in the object model**, not a clause in a contract document.
- **Profile worker** — a worker "who is not paid by nor reports to the organization." The least
  integrated category (e.g. a supplier's on-site staff you must still badge and track).

`CONFIRMED` — [learning.sap.com — Introducing Contingent Staffing and Services
Procurement](https://learning.sap.com/courses/introducing-sap-fieldglass/introducing-contingent-staffing-and-services-procurement), accessed 2026-07-29.

**Carry this forward.** The corpus's people module models employees. Nothing in it models "a person who
works here, is directed by our manager, and is employed and paid by someone else" — and SAP has a
shipped, three-way typed answer keyed on direction and payment.

### 2.2 The document chain: the work order is the authority object

Job Posting → Job Seeker (candidate) → **Work Order** → Worker. "A work order is a binding document and
dataset that contains all necessary information for a specific contract between the buyer and supplier,"
and most of its content "is pulled directly from both the Job Posting and the Job Seeker records." The
supplier "submits candidates against the submitted Job Posting and is ultimately responsible for the
payment of the contingent worker for time and expenses incurred." The Worker record "is the result of
the candidate being selected for hire."

Note the shape: **the work order is the authority object, not the worker.** Right-to-work, rate and cost
coding live on a contract between two legal entities; the worker is downstream of it.

`LIKELY` (learning.sap.com course indexes + search extracts; the *Creating Work Orders* and *Engaging
the Worker* lesson pages 404'd to fetchers) —
[learning.sap.com — Creating Work Orders](https://learning.sap.com/courses/procuring-contingent-workers-using-sap-fieldglass-es/creating-work-orders);
[learning.sap.com — Introducing the Contingent Workflow](https://learning.sap.com/courses/introducing-sap-fieldglass/introducing-the-contingent-workflow_c152a633-e63f-4109-89f9-f3168818fa5b) — accessed 2026-07-29.

### 2.3 How the external worker relates to the client's own HR records

- Fieldglass "is a master system for external workers," and the integration "allows you to view
  contingent worker records taken from SAP Fieldglass in Employee Central" — the client's HRIS gets a
  contingent worker profile "similar to an employee profile for an internal worker."
- The two stay distinguishable by a flag: "Worker data exported from SAP Fieldglass includes a flag
  indicating the source system, which allows SAP SuccessFactors to correctly identify contingent workers
  and differentiate between SAP Fieldglass and Employee Central contingent workers, eliminating the need
  for dual licensing."
- SAP markets the pair as **Total Workforce Management** — one headcount view over employees and
  non-employees.

`LIKELY` — [SAP Help — Fieldglass ↔ EC Integration Business Synopsis](https://help.sap.com/docs/SAP_FIELDGLASS_INTEGRATION/31c37f12fb734bd5a3b6039108a9c3ad/43347255fcb745aaa4f52764441fa50b.html) (index only, SPA);
[SAP Press — Total Workforce Management](https://blog.sap-press.com/total-workforce-management-with-sap-fieldglass-sap-successfactors);
[SAP — Workforce Management](https://www.sap.com/products/hcm/workforce-management.html) — accessed 2026-07-29.

**The seam, stated plainly:** this is two systems of record for "a person who works here", reconciled by
a **source-system flag**. The flag exists precisely because the copy exists. A platform that models
engagement type as a property of one worker object never needs it.

### 2.4 Classification risk

Fieldglass reportedly includes a **decision wizard** walking a hiring manager through classifying a
worker, because misclassification carries co-employment liability, back taxes and penalties.
`UNCERTAIN` — sourced only from a partner page, not SAP directly.
[LeverX — Fieldglass contingent workforce management](https://leverx.com/solutions/fieldglass-contingent-workforce-management);
regulatory context: [US DOL — misclassification under the FLSA](https://www.dol.gov/agencies/whd/flsa/misclassification) — accessed 2026-07-29.

Korean equivalent (불법파견 / 위장도급 exposure, 파견 duration limits, 직접고용 obligation triggers):
**UNKNOWN.** I found no SAP-published Korean classification content. Flagging rather than guessing.

| finding | delta | what the business gets | conf |
|---|---|---|---|
| Contingent vs SOW vs profile, split on **who directs** and **who pays** | **NEW** | A typed answer to 파견 vs 도급/용역 enforced by the object model, not a contract PDF | CONFIRMED |
| Work order as the binding buyer↔supplier authority object; worker downstream | **NEW** | Rate, cost coding and right-to-work live on an inter-entity contract | LIKELY |
| External worker mirrored into EC with a **source-system flag** | **NEW** | One headcount view — at the cost of two masters | LIKELY |
| Classification decision wizard | **NEW** | Structured classification at hire time | UNCERTAIN |
| Korean 파견법 / 불법파견 content | **NEW gap** | — | UNKNOWN |

---

## 3. Business Partner unification + MDG — **NEW / EXTENDS `object-platform.md:169-175,222`**

The corpus cites MDG only as a *governed change-request* benchmark. The single-record story is absent,
and it is the most instructive architecture decision in the suite.

### 3.1 What Business Partner is and what it replaced

In ECC there were **separate customer (debtor) and vendor (creditor) masters**, with their own
transactions, numbers and address data. In S/4HANA:

- "The strategic object model in SAP S/4HANA is the Business Partner (BP)," and BP is "the single point
  of entry to create, edit and display master data for Business Partners, Customers, and Vendors."
- "In SAP S/4HANA those [customer and vendor master transactions] are completely replaced by the
  Business Partner."
- One party holds **several roles**: "If a company is both a customer and a supplier, CVI merges them
  into a single Business Partner, assigning both a customer role and a supplier role to that one entry."

`LIKELY` (SAP-authored community posts, 403 to fetchers; consistent across four independent sources) —
[SAP Community — BP conversion: merge customer and vendor into a single BP](https://community.sap.com/t5/enterprise-resource-planning-blog-posts-by-sap/sap-s-4hana-business-partner-conversion-merge-customer-and-vendor-into/ba-p/13484058);
[Eursap — intro to Business Partners and CVI](https://eursap.eu/blog/an-intro-to-business-partners-and-cvi-in-s-4hana-conversions) — accessed 2026-07-29.

### 3.2 What it cost — read this twice

The merge was not optional. **BP is mandatory in S/4HANA**, so "all Customer and Vendor master data in
SAP ECC needs to be converted before your SAP S/4HANA migration." The conversion mechanism is **CVI
(Customer/Vendor Integration)**, which keeps the superseded tables alive behind the new object: "CVI
ensures that Customer and Vendor master data tables are updated automatically after a BP is
created/changed." Old identities persist as mapping tables (`BD001`/`BC001`) rather than disappearing.

So the price of unifying two masters into one was: a **mandatory pre-migration data project**, a
**permanent synchronisation layer** keeping the old tables current for downstream code, and **key-mapping
tables preserving the old identities indefinitely**. CVI is routinely described as a gating workstream of
a brownfield conversion, and duplicate or inconsistent party data is what breaks it.

`LIKELY` — [SAP Community — FAQ: CVI for system conversion to S/4HANA](https://community.sap.com/t5/enterprise-resource-planning-blog-posts-by-sap/faq-cvi-customer-vendor-integration-for-system-conversion-to-sap-s-4hana/ba-p/13740757);
[SAP Community — handling old customer/vendor↔BP mapping with BD001/BC001](https://community.sap.com/t5/enterprise-resource-planning-blog-posts-by-sap/sap-s-4hana-business-partner-conversion-handle-old-customer-vendor-business/ba-p/13562692);
[CDQ — the role of CVI](https://www.cdq.com/blog/sap-customer-vendor-integration-cvi) — accessed 2026-07-29.

**The lesson:** SAP concluded a party is **one entity with roles**, not several records — and it was
right. But because it reached that conclusion *after* shipping the split model, it paid with a mandatory
migration, a permanent compatibility shim, and eternal key mapping. **A platform that starts with
party-plus-roles gets the entire benefit and pays none of that cost.** This is the strongest single
argument in the suite for getting the party model right on day one.

*(Cross-reference, not a repo read: `docs/ideas/authority-and-approval-model.md` — surfaced to me by the
harness, not opened by me — independently reaches "'employee' is not an entity… it is a role a person
plays relative to a legal entity" and "name it `party`, not `employee`". SAP's CVI history is external
corroboration of that direction, and a price tag for deferring it.)*

### 3.3 MDG — and a correction to `compliance.md:248`

MDG governs authoring of master data (BP, customer, supplier, material, finance) across SAP and non-SAP
sources with consolidation into a central repository. That aligns with the corpus's framing.

**What the corpus misses:** MDG ships an app providing "governance of an aligned, application-agnostic
**legal business partner hierarchy representing the company ownership**", to "achieve a single source of
truth for legal ownership data across all applications and support for compliance with regulatory and
audit requirements."

`compliance.md:248` says a "그룹 obligation cascading to 계열사 — **none of the 7 vendors model** a
Korean conglomerate hierarchy natively." That must be split:

- **A legal ownership hierarchy across entities: SAP MDG does model this.** The claim as written is now
  too strong.
- **Cascading an *obligation* down that hierarchy: still unmodelled** as far as I can source. MDG governs
  the ownership graph; it does not propagate compliance duties along it.

The differentiator survives, but narrower and sharper — and stating it precisely is what makes it
defensible under review. `LIKELY` — [SAP — Master Data Governance](https://www.sap.com/products/data-cloud/master-data-governance.html);
[SAP Community — MDG innovations with S/4HANA PCE 2025](https://community.sap.com/t5/technology-blog-posts-by-sap/innovations-in-master-data-governance-with-sap-s-4hana-cloud-private/ba-p/14146957) — accessed 2026-07-29.

---

## 4. GRC / Access Control — **EXTENDS `lenses/governance.md:155`**

`lenses/governance.md:155` already names the SoD ruleset as "**L, highest governance ROI**" and
identifies that SAP's value is a predefined library of conflicting-permission pairs plus mitigation
controls; `policy.md:121` already covers MSMP, Firefighter and UAR. Three additions only.

**4.1 The five components, from SAP's own training** (upgrades `policy.md:121` from secondary `[V]` to
SAP-official). Access Control "enables an organization to control access, identify risk, and document
compliance" via: **risk analysis and remediation**; **access request management** ("a self-service portal
for users to request access… and automated workflows for access request approval"); **business role
management** with "a best practice template methodology"; **emergency access management** where
"firefighters can be granted extra system security and temporary access" with activity logged and
reviewed by controllers; and **user access review / SoD review**, both "workflow-driven process[es]".
Plus **mitigating controls** as compensating controls applied to users, roles and groups.
`CONFIRMED` — [learning.sap.com — Describing SAP Access Control](https://learning.sap.com/courses/exploring-the-fundamentals-of-sap-system-security/describing-sap-access-control) — accessed 2026-07-29.

**4.2 Why the product has to exist — NEW, and the part worth stealing.** SAP's authorization model is
**positive-only**: everything not explicitly granted is denied, and there is no deny rule. Roles are
therefore purely additive, and nothing in the role tool can express "these two capabilities must not
co-occur in one person." **SoD is an emergent property of an additive model, not a feature of it** —
which is why detection had to be sold as a second product running after the fact. That reframes
`lenses/governance.md:155` from "SAP has a ruleset we should copy" to "**a model that can express mutual
exclusion natively does not need the second product.**" `LIKELY` on positive-only —
[keyusertraining — SAP roles and authorizations](https://keyusertraining.com/en/sap-roles-and-authorizations/) — accessed 2026-07-29.

**4.3 Cross-system SoD is a distinct, harder problem**, following directly from the portfolio shape in
§1: with ~20 products each holding roles, a conflict pair can span two of them, and SAP publishes
separate 2025-era guidance on it. `UNCERTAIN` (search extract only) —
[SAP Community — How to Manage Cross-System SoD Risk](https://community.sap.com/t5/financial-management-blog-posts-by-sap/how-to-manage-cross-system-segregation-of-duties-sod-risk/ba-p/14083887) — accessed 2026-07-29.

**Also worth knowing for how a role is authored** (extends `policy.md:49`'s "heavy consultant tooling"):
an authorization **object** is a developed artefact whose check must be written into the code, but role
**content** is data an administrator authors in `PFCG`; **organisational levels** (company code `BUKRS`,
plant `WERKS`, sales org `VKORG`) are set once and inherited by every object in the role; and **derived
roles** inherit "the menu structure and the functions included… from the referenced role" while user
assignments are "**not** inherited" and already-maintained org levels are not overwritten. SAP's own
example is two warehouse staff with identical jobs authorised for plants 1000 and 2000 — i.e. "same job,
different 법인/사업장" solved as inheritance. `CONFIRMED` —
[learning.sap.com — Implementing a Derived Role Strategy](https://learning.sap.com/courses/exploring-the-authorization-concept-for-sap-s-4hana-and-sap-business-suite/implementing-a-derived-role-strategy);
[learning.sap.com — Implementing the ABAP Roles](https://learning.sap.com/courses/exploring-the-fundamentals-of-sap-system-security/implementing-the-abap-roles-for-on-premise-private-cloud) — accessed 2026-07-29.

---

## 5. The enterprise-structure ladder — **NEW**

Grep: `company code` = 0, `controlling area` = 0, `profit cent` = 0, `business place` = 0. The corpus has
**our** `Group→법인→branch→worksite` ladder in several places (`dashboard.md:160`, `policy.md:69`,
`field.md:183`) but never SAP's, so there is nothing to compare against. SAP has thirty years of
modelling this exact shape.

SAP's insight: a business is not one hierarchy but **several independent dimensions over the same
transaction**, each answering a different question, with a posting carrying a coordinate in every one.

| Unit | SAP's definition / role | Maps to | Conf |
|---|---|---|---|
| **Client** | "Each client is an independent unit that contains separate master records, a set of tables, and data" — business-wise, a corporate group | The tenant = the 그룹 | CONFIRMED |
| **Company code** | "An independent legal accounting entity, such as a company, with independent accounts within a corporate group"; "financial statements required by law are created at the company code level"; the **only mandatory** FI org unit — "the minimum structure necessary" | **법인**, one-to-one | CONFIRMED |
| **Company** (≠ company code) | "FI records are consolidated at the level of the company. One or more company codes… can be assigned to a company"; doubles as **trading partner** on intercompany postings | Consolidation parent + counterparty ID for elimination | CONFIRMED |
| **Controlling area** | "The organizational unit within a company for which complete, closed cost controlling can be carried out" — a **closed** system; cost cannot be allocated across its boundary. 1:n over company codes enables cross-entity allocation | The management-accounting boundary | CONFIRMED |
| **Plant** | Site / logistics unit inside exactly one company code; normally also the **valuation area**, so inventory is valued per site | 사업장 as a physical site | LIKELY |
| **Profit centre** | "Evaluates the success of independent areas that are responsible for costs and revenues" — internal responsibility, deliberately independent of legal entity | 본부 / 사업부 | CONFIRMED |
| **Cost centre** | "Separate areas within a controlling area at which costs are incurred" | 부서 as a cost home | CONFIRMED |
| **Segment** | "A division of a company for which you can create financial statements for external reporting" (IFRS 8 / US-GAAP) | External reporting division | CONFIRMED |
| **Business area** | Legacy. **Not available at all** in S/4HANA Cloud Public Edition; on-prem retains it for migration compatibility only; SAP steers new builds to profit centre + segment | — | LIKELY |
| **Business place** | "Used in countries that by law require returns for taxes on sales/purchases to be submitted at a level below the company code"; assigned to a company code; "the unit responsible for tax reporting"; assigns official document numbers to outgoing documents. **South Korea explicitly named** | **사업장 as a tax-filing unit** | CONFIRMED |

Sources: [learning.sap.com — Managing Organizational Units in FI](https://learning.sap.com/courses/customizing-core-settings-in-financial-accounting-in-sap-s4hana/managing-organizational-units-in-financial-accounting-fi);
[learning.sap.com — Describing the Components of Management Accounting](https://learning.sap.com/courses/cost-center-and-internal-order-accounting-in-sap-s-4hana-fr/describing-the-components-of-management-accounting);
[learning.sap.com — Creating Profit Centers and Segments](https://learning.sap.com/courses/customizing-core-settings-in-financial-accounting-in-sap-s4hana/creating-profit-centers-and-segments);
[help.sap.com static doc — Business Place](https://help.sap.com/doc/8248d953189a424de10000000a174cb4/700_SFIN3E%20006/en-US/6f81d0531d8b4208e10000000a174cb4.html);
[SAP KBA 2760863 — Business Area not in Public Cloud](https://userapps.support.sap.com/sap/support/knowledge/en/2760863) — all accessed 2026-07-29.

### 5.1 Four constraints that are the actual lessons

1. **Cross-entity cost allocation is bought with a shared chart of accounts.** One controlling area over
   many company codes requires all of them to use the **same operational chart of accounts**, and "the
   number of posting periods must be the same for company code and controlling area." So group-wide cost
   allocation forces a global CoA decision before the first posting — and an acquisition violates it
   immediately. `CONFIRMED`.
2. **Segment is derived, not entered.** "When you post to a profit center, the segment is posted
   automatically," and "segments can only be derived automatically using profit centers"; manual
   assignment needs a BAdI or substitution rule. One reporting dimension is hard-wired as a function of
   another. `CONFIRMED`.
3. **Cost-centre standard-hierarchy assignment is not time-dependent.** Change it and both historical and
   current reporting restate under the current assignment. Consolidation hierarchies *do* support
   time-dependent nodes and leaves. **Reorganisation silently rewrites the past unless hierarchy edges are
   versioned** — a concrete requirement, not a nicety. `LIKELY` —
   [SAP Press — What Are Cost Centers in S/4HANA](https://blog.sap-press.com/what-are-cost-centers-in-sap-s4hana);
   [learning.sap.com — Outlining Global Hierarchies](https://learning.sap.com/learning-journeys/performing-consolidation-with-sap-s-4hana-cloud-for-group-reporting/outlining-global-hierarchies_ccd92686-9f4a-4416-a38b-f0db60119cd8).
4. **Consolidation is a separate application.** Company codes and profit centres map to **consolidation
   units**; data flows from `ACDOCA` to `ACDOCU`; currency translation is a configured method per FS item
   with defined handling of translation differences; then eliminations run over consolidation groups.
   `LIKELY` — [learning.sap.com — Describing Consolidation Processes](https://learning.sap.com/learning-journeys/performing-consolidation-with-sap-s-4hana-cloud-for-group-reporting/describing-consolidation-processes_e0a49f1c-8d52-4a07-94dd-b1e932b46b5e).

### 5.2 Korea, beyond the org unit

S/4HANA generates Korean **electronic tax invoices (전자세금계산서)** as NTS-format XML — but "you need an
external forwarding system to act as a bridge between SAP S/4HANA and the NTS." Korea-specific tax codes
and tax-invoice handling ship in the local version. `LIKELY` —
[help.sap.com static doc — Creation and Sending of Electronic Customer Tax Invoices in XML](https://help.sap.com/doc/474a13c5e9964c849c3a14d6c04339b5/100/en-US/6db365bbd7d24d54ab8e213905114aa0.html) — accessed 2026-07-29.

---

## 6. Approvals — **EXTENDS `appr.md:60`, `appr.md:160`, `lenses/task-flow.md:46`**

### 6.1 The single most valuable finding: Concur has the readable authority object; S/4HANA does not

The corpus cites Concur only for 1-click ergonomics. Its **authority model** is the interesting part, and
it is precisely the artefact `appr.md:160` says SAP lacks.

In Concur Expense, an **Authorized Approver** is "a user who holds the standard Expense Approver role
plus additional permissions that allow them to approve expense reports under specific conditions." Two
kinds, combinable — "An Authorized Approver may have one or both types":

- **Limit approval** — permission to "approve an expense report when the total amount is equal to or less
  than their assigned approval limit," configured as **currency + amount, per approver**.
- **Exception approval** — approve reports "that contain exceptions within a defined minimum and maximum
  exception level."

Plus **Cost Object Approvers**, defined with levels or limits, so the limit binds to *what is being
charged*, not only to the org chart. And the traversal escalates: "if no approvers are found at the
initial level, the system searches the next level up in the authorized approver hierarchy and continues
this process until a level that has approvers is found."

`CONFIRMED` — [learning.sap.com — Maintaining Authorized Approvers](https://learning.sap.com/courses/working-with-primary-configuration-in-concur-expense-professional-edition/maintaining-authorized-approvers-1);
`LIKELY` on cost-object approvers and escalation — [learning.sap.com — Managing a Limit-Based Approval](https://learning.sap.com/learning-journeys/getting-started-with-concur-expense-standard-for-administrators/managing-a-limit-based-approval) (404 to fetcher; search extract);
[help.sap.com — Add Expense Authorised Approver or Cost Object Approvers](https://help.sap.com/doc/a42953e63bbd46e9a503f6bebbaf7083/2022_04/en-GB/5aac4e653dea40f295589c3dfccb1b5f.html) — accessed 2026-07-29.

**Contrast with the ERP core — this is the delta.** In S/4HANA "this person may approve up to X in company
code Y" is **not one object**. It is the intersection of:

- a **threshold in process configuration** — value bands as characteristics of a release strategy
  (typically over `CEKKO-GNETW`), or a "Total net amount" condition on a flexible-workflow step; and
- a **right to act in a role** — `M_EINK_FRG` (fields `FRGGR` release group, `FRGCO` release code)
  determines "which purchasing documents the user may release (approve) and which release codes he or
  she may use when doing so"; `F_BKPF_BUK` scopes accounting postings by company code.

The **only** genuine per-person monetary ceiling in the ERP core is FI **tolerance groups** (`OBA4`),
setting per group per company code the maximum "amount per document", the maximum amount per open-item
line, and the permitted payment difference — FI-posting-scoped, not general approval.

**So SAP built the readable authority object in the acquired expense product and never back-ported it to
the ERP core.** Two products in the same suite answer "what is this person's approval authority?"
completely differently, and only one can answer it at all. `LIKELY` on the authorization-object field
semantics — [tcodesearch — M_EINK_FRG](https://www.tcodesearch.com/sap-authorization-objects/M_EINK_FRG);
[tcodesearch — F_BKPF_BUK](https://www.tcodesearch.com/sap-authorization-objects/F_BKPF_BUK);
`LIKELY` on tolerance groups — [learning.sap.com — Managing Posting Authorizations](https://learning.sap.com/courses/basics-of-customizing-for-financial-accounting-gl-ap-ar-in-sap-s-4hana/managing-posting-authorizations) — accessed 2026-07-29.

### 6.2 Correction: `appr.md:160` is stale for S/4HANA

`appr.md:160` reads: "SAP: localizable but 전결규정 must be modeled as release-code hierarchy; heavy
consulting; 합의/협조 line types not native. [I]".

The verdict is right; the mechanism is out of date. Release strategies are the **ECC-era** path. In
S/4HANA the current mechanism is **Flexible Workflow**, activated per document type in customising and
then **configured by a business user in a Fiori app** (`Manage Workflows for Purchase Requisitions` /
`… Orders`). Available conditions include "Account assignment category Cost Center, Account assignment
category Project, Company code, Material group in at least one purchase order item, Currency, Document
type, Purchasing group, Purchasing organization, Total net amount", combinable with "and/or linkage";
"You can assign one or several possible approvers to a workflow. The approvers can be users or roles";
"Approvals can have one or several steps". SAP states it "can replace the release procedure with
classification or be used additionally," chosen per document type.

Also: **amount bands never lived in the release code.** The code is the authority *point*; the band is a
characteristic of the strategy. `appr.md:60`'s "release-code delegation encodes 전결-like
authority-by-level" is directionally right but conflates the two — and a release code is not delegated;
delegation is workflow substitution (§6.4).

For completeness on the classic path, since the corpus's `appr.md` covers 결재선 depth: PRs release
**item-wise** ("every item is checked… as you enter the data") or as an **overall release** ("all items
must fulfill these criteria… the check is made when you save"), decided per requisition document type;
sequence is expressed as **release prerequisites**; and standard SAP allows **8 release levels**.

`CONFIRMED` on flexible workflow and item-wise/overall — [learning.sap.com — Setting Up Flexible Workflows in Purchasing](https://learning.sap.com/courses/purchasing-in-sap-s-4hana/setting-up-flexible-workflows-in-purchasing);
[learning.sap.com — Processing Release Procedures for PRs](https://learning.sap.com/courses/exploring-operational-procurement-in-sap-s-4hana/processing-release-procedures-for-purchase-requisitions);
`LIKELY` on the 8-level ceiling and prerequisites — [Guru99 — release procedures](https://www.guru99.com/release-procedures-for-purchasing-documents.html) — accessed 2026-07-29.

### 6.3 Responsibility Management — **NEW**, and SAP conceding the org-chart point

Grep = 0. This is SAP's current answer to "who approves?", and the framing is the valuable part.

Responsibility Management is a framework to "uniformly and centrally determine the person or entity who
could be held responsible for completing a particular task or activity." You define **teams** (a **team
category** represents a business process), **members with member functions** describing what each does,
and **responsibility rules** matching attributes such as company code and plant against functions like
"Operational Purchaser" or "Cost Center Manager". Three agent-determination shapes: a whole team as
agent; role-based via member function; rule-based (e.g. "Manager of an employee"). Standard rules ship
and are copied in `Manage Responsibility Rules`; a BAdI exists for custom logic.

SAP's own stated motivation: to "simplify approval process to reflect real business needs" and to avoid
"the cumbersome process to change an existing organization structure to adopt dynamic approval
processes."

**The vendor of the world's most elaborate org-management model is saying approval authority should not
be derived from the org chart.** For a 전결규정 design that is strong external corroboration.

`CONFIRMED` — [learning.sap.com — Explaining Responsibility Management](https://learning.sap.com/courses/sap-workflow-overview-basics-strategy-and-extensibility/explaining-responsibility-management) — accessed 2026-07-29.

### 6.4 Substitution detail — **EXTENDS `appr.md:60`**

`appr.md:60` correctly notes planned/unplanned substitution. Three mechanics worth copying: it is
**self-service** (the approver manages their own substitutions from Fiori My Inbox); **planned**
substitution is dated and the substitute simply sees the tasks for that window, while **unplanned**
substitution requires the substitute to **actively accept** before seeing them; and scope narrows by
assigning a **task group**, so a substitute handles only certain work-item types (no task group =
everything). `LIKELY` — [blogs.sap.com — SAP Fiori My Inbox 2.0](https://blogs.sap.com/2016/03/04/sap-fiori-my-inbox-20/) (2016-03-04);
[help.sap.com — Create and Manage Substitution Rules](https://help.sap.com/docs/SAP_FIORI/d2c296c4f32d4f2a9e3752f58d5ef222/e4fa99f64eae4c5a9b189d035ec8adc9.html) (index only) — accessed 2026-07-29.

### 6.5 Approval is bound to document *state*, not document ID — **NEW**

If a released purchasing document changes such that a configured threshold is crossed — notably net value
rising past the release indicator's changeability setting — the release strategy is **reset** and
re-approval is required (reported: decreases do not retrigger). Revoking a release is permitted only **if
the release indicator allows it**; otherwise the indicator's configuration must change or the document
must be recreated. This structurally closes the "approved at ₩10M, then edited to ₩100M" hole instead of
relying on policy. `LIKELY` (community sources only) —
[SAP Community — reverse release strategy after changes](https://community.sap.com/t5/enterprise-resource-planning-q-a/how-to-reverse-release-strategy-of-purchase-order-after-making-changes/qaq-p/9299196) — accessed 2026-07-29.

---

## 7. Economics spine — **EXTENDS `finance.md:41,49,113`, `dashboard.md:80`**

### 7.1 `finance.md:41` is correct and can be upgraded from `[I]` to sourced

"Universal Journal: header in BKPF, lines in ACDOCA" is accurate: `BKPF` remains the accounting document
header; `ACDOCA` replaced `BSEG` as line items. Citable rather than inferred: `ACDOCA` supersedes `BSEG`
(GL line items), `COEP` (CO line items), `FAGLFLEXA`/`FAGLFLEXT` (new-GL) and the CO-PA `CE1*` tables,
with compatibility views retained; it merges GL, Asset Accounting, Material Ledger, Controlling and
account-based CO-PA. `CONFIRMED` —
[cbs — SAP Universal Journal (ACDOCA)](https://www.cbs-consulting.com/us/sap-universal-journal-acdoca/) (2022-11-19).

### 7.2 The business object is a *dimension on the posting* — **NEW as an explicit claim**

`finance.md:49` registers "source-to-posting lineage" as a row. The stronger statement is available:
`ACDOCA` carries **one field per account-assignment object** — cost centre, profit centre, segment,
functional area, order number and order category, **WBS element**, project definition, network and
activity, business process — plus `ACCASTY_N1..N3` distinguishing **real from statistical** assignments.
Market-segment / profitability characteristics ride the same line item.

Three consequences worth naming:

- "What did project P cost" is a **filter on the journal**, not a join to a project system.
- FI↔CO reconciliation does not exist as a concept, because there is one line item, not two. (ECC needed
  a reconciliation ledger.)
- **Real vs statistical assignment** is the underrated one: the same cost may be *reported* against
  several objects while being *owned* by exactly one. That resolves double-counting declaratively rather
  than by convention.

`LIKELY` (field lists from an SAP-authored community post, 403 to fetchers, plus a field dump) —
[SAP Community — CO Account Assignment and Attribution with S/4HANA](https://community.sap.com/t5/enterprise-resource-planning-blog-posts-by-sap/co-account-assignment-and-attribution-with-s-4hana/ba-p/13575394) — accessed 2026-07-29.

### 7.3 `dashboard.md:80` — upgrade and qualify

`dashboard.md:80` says "SAP: native strength — margin/cost-center/profit-center analytics… **[I]**".
Sourced version: this is **Margin Analysis** (account-based CO-PA), which "is fully integrated in the
Universal Journal. It's therefore reconciled by design." Revenue and COGS postings get profitability
dimensions at posting time; project, WBS element, customer and product are derived onto the same journal
line. Costing-based CO-PA is replaced by Margin Analysis in Public Cloud. **Qualify with §5.1's
constraint**: the segment dimension is only as good as the profit-centre model, because segment can only
be derived from profit centre. `CONFIRMED` —
[SAP Press — Account-Based CO-PA in S/4HANA](https://blog.sap-press.com/account-based-co-pa-in-sap-s4hana-how-margin-analysis-works-in-the-universal-journal) (2026-02).

### 7.4 Universal Allocation and the saved simulation — **NEW**

Grep = 0. One engine, one Fiori app (`Manage Allocations`), replacing a scatter of transaction codes.
Contexts: cost centre, profit centre, margin analysis (costing-based profitability **not** supported).
Two mechanics: **distribution** reallocates while crediting/debiting the *original* G/L account, so the
nature of the cost is preserved; **overhead allocation / assessment** posts through a **secondary cost
account** (cost element category 42) so the allocation is visible as its own cost type. Intercompany
allocation is supported.

The part worth stealing: the test run is "**a full simulation mode that is saved into the system**" — a
retained, inspectable dry-run document produced before anything posts. **An allocation is a posting, not
a report**: it has a document, a sender, a receiver, a basis and a reversal path. Indirect-cost arguments
are political; a durable pre-posting simulation is what makes them settleable.

`CONFIRMED` — [SAP Press — Universal Allocation in SAP S/4HANA](https://blog.sap-press.com/universal-allocation-in-sap-s4hana) (2025-09-29);
[learning.sap.com — Differentiating Distribution and Overhead Allocation](https://learning.sap.com/courses/detailing-overhead-cost-accounting/differentiating-between-distribution-and-overhead-allocation_d3bcfcf4-62f7-40b7-a05c-feaf4ff542c6).

### 7.5 Extension ledgers — **NEW**

Extension ledgers layer on top of an underlying ledger **without copying its rows**, giving a management
view or a local-GAAP adjustment layer over the same postings; parallel ledgers carry IFRS / local GAAP /
US GAAP concurrently; ledger `0L` is the leading ledger and the one integrated with Controlling. The
generalisable idea: **an alternative view is a thin overlay of deltas on a base, not a second copy of the
base.** `LIKELY` — [SAP Press — 11 Features of General Ledger Accounting in S/4HANA](https://blog.sap-press.com/11-features-of-general-ledger-accounting-in-sap-s4hana) — accessed 2026-07-29.

### 7.6 Time-driven events emit ordinary documents — **NEW**

Depreciation is **always periodic** — program `FAA_DEPRECIATION_POST` (replacing `RAPOSTxxxx`),
transaction `AFBP`, typically monthly; there is no real-time depreciation, and unplanned/special
depreciation needs its own run. Each asset is valued in multiple **depreciation areas**, flexibly
assigned to **ledgers**, which is how one physical asset carries several simultaneous valuations.
Clock-driven economics produce journal entries indistinguishable downstream from human-initiated ones —
one document model, one audit trail, one reversal path for both. `LIKELY` —
[SAP Press — Asset Depreciation in S/4HANA](https://blog.sap-press.com/asset-depreciation-in-sap-s4hana);
[SAP Press — Depreciation Areas in S/4HANA](https://blog.sap-press.com/depreciation-areas-in-sap-s4hana).

### 7.7 Cost collector + settlement rule — **NEW as a named pattern**

A maintenance work order accumulates confirmations (labour), goods issues (material) and overhead; then
**settlement** "credits the order with the actual costs" and transfers them to the receiver named in the
order's **settlement rule** — typically the cost centre of the maintained equipment or the org unit that
requested the work. Settlement is periodic or full and repeatable after further actuals. Projects use the
**WBS element** as the collector, with planned costs aggregating up the structure. The pattern:
**work accumulates cost on itself, then declares who bears it** — separating "what did this job cost" from
"whose budget takes it", which are different questions. `LIKELY` —
[learning.sap.com — Settlement](https://learning.sap.com/courses/customizing-in-sap-s-4hana-asset-management/settlement);
`CONFIRMED` on WBS — [learning.sap.com — Planning Costs for WBS](https://learning.sap.com/learning-journeys/discovering-the-basics-of-sap-s-4hana-project-system/planning-costs-for-wbs-elements_d9f22bd8-1e3f-4b71-88e7-b2b52e6bb6c7).

---

## 8. Traceability — **EXTENDS `finance.md:49`; batch / serial / change-documents are NEW**

Grep: `lineage` (38 hits) is entirely **Foundry data lineage**; `genealogy`, `serial number`,
`change document` / `CDHDR` = 0. SAP's transactional traceability stack is absent.

- **Change documents (`CDHDR`/`CDPOS`)** — on save, one header row to `CDHDR` (object class, change
  number, user, date, time) and **one item per changed field** to `CDPOS` (old value, new value), across
  master data and many transactional documents in FI/MM/SD/HR. Field-level, user-attributed, timestamped
  before/after. `LIKELY` — [learntosap — CDHDR/CDPOS](https://www.learntosap.com/SAP-CDHDR-CDPOS-Tables.html).
- **Document flow (`VBFA`)** — stores the relation between all sales documents and the *type* of relation
  between each preceding and subsequent document. Backward navigation is trivial (a document stores its
  predecessor); `VBFA` exists specifically to make **forward** traversal and fan-out (one order → many
  deliveries) possible. `LIKELY` — [tcodesearch — VBFA](https://www.tcodesearch.com/sap-tables/VBFA).
- **Material documents** — header `MKPF`, items `MSEG`; movement types encode intent (101 = GR against
  PO, 102 = its reversal); the reversal's `MSEG-SMBLN`/`SMBLP` point back to the reversed document/item.
  Nothing mutates; quantities are added and subtracted by documents. `LIKELY` —
  [leanx — MSEG](https://leanx.eu/en/sap/table/mseg.html).
- **Financial reversal** — `FB08` posts a reversal document and the original becomes a "reversed
  document"; both retained. `LIKELY` — [help.sap.com — Reversal and Reversed document in FI](https://help.sap.com/docs/SUPPORT_CONTENT/fiaccounting/3361878485.html).
- **Batch genealogy** — batch where-used gives **top-down** (which batches were consumed to make this
  one, including intermediate stages) and **bottom-up** (which came from it). Batch identity is configured
  at one of three **batch levels**; attributes are classification **characteristics**; **batch
  determination** selects which lot to consume via the condition technique. Cross-system genealogy
  requires the separately licensed **Global Batch Traceability (GBT)**. `LIKELY` —
  [SAPinsider — GBT for genealogy reporting](https://sapinsider.org/how-to-use-sap-global-batch-traceability-to-meet-your-genealogy-reporting-requirements/);
  [learning.sap.com — Basics of Batch Management](https://learning.sap.com/courses/implementing-sap-s-4hana-cloud-public-edition-manufacturing/explaining-the-basics-of-batch-management).
- **Serial numbers** — per-unit identity, switched on per material by a **serial number profile**, which
  also decides whether an **equipment master** is created at serial creation or later. `LIKELY` —
  [learning.sap.com — Working with Serial Numbers](https://learning.sap.com/courses/managing-technical-objects-in-sap-s-4hana-asset-management/working-with-serial-numbers).

**The limits, stated honestly — these are the design requirements:**

1. **Lineage bottoms out at the batch, not the unit.** Serial numbers give unit identity but are a
   separate mechanism, not a general provenance graph.
2. **Proportional attribution across a split is not modelled.** The graph says "B came from A1, A2, A3";
   it does not say what fraction of B is attributable to each. Adequate for recall scope — its design
   purpose — and weak for mass balance or per-unit cost attribution. `UNCERTAIN` — inferred from absence
   across every source read, not positively confirmed.
3. **Batch derivation degrades on unplanned usage** — documented limitation when "unplanned material
   usage is a common occurrence." `LIKELY` — [Clarkston — batch release functionality](https://clarkstonconsulting.com/insights/batch-release-functionality-in-sap/).
4. **Cross-system lineage is a separate product**, and **GBT's 2026 status is UNCERTAIN** — 3.0 dates to
   2017; a `GBT on S/4HANA` variant exists and Batch Release Hub integrates with it, but I found no
   current roadmap statement. Do not assume it is a growth area.

---

## 9. Extensibility and the developer boundary — **NEW**

Grep: `clean core` = 0, `key user` = 0. The corpus repeatedly notes SAP is "consultant-heavy"
(`leave.md:182`, `finance.md:113`, `policy.md:121`, `dashboard.md:170`) but never says where the line
falls, so there is no basis for "how much could a business admin actually do?"

Three extension types: **key user (in-app)**, **developer (on-stack ABAP Cloud)**, **side-by-side on
BTP**. SAP formalised a compliance ladder **A–D**: **A** = released APIs only, "highest standard in SAP
extensibility", maximum upgrade stability; **B** = classic APIs, "well-established, broadly
recommended"; **C** = internal objects, upgrade risk; **D** = "highest risk category" requiring
remediation, explicitly including modifications, implicit enhancements, objects marked `noAPI`, and
"unsupported write operations on SAP tables". Guidance: "extend SAP S/4HANA Cloud with the highest level
possible."

**What a business user genuinely can do without a developer:** add **custom fields** to standard objects
and apps via `Custom Fields and Logic` (the scope enabled for key-user custom fields "is much broader
than the scope for developer extensibility"); write **custom logic** (the cloud name for BAdIs) at
published extension points; author **Flexible Workflow** conditions and steps; define **responsibility
teams and rules**; define **allocation cycles** and run saved simulations; author **roles and derived
roles**; maintain **global hierarchies**.

**Hard ceilings:** "There is no direct access allowed to database tables, instead you can use released CDS
views in select statements"; "any extension is still limited by the input and output parameters provided
in the BAdI interface"; and "if you make changes to the code that are not supported, the editor simply
won't allow you to save or publish."

**Net, and this is the transferable judgement: a business can change a great deal of *policy* without a
developer, and almost none of the *model*.** Thresholds, approvers, teams, allocation rules, hierarchies,
roles, extra attributes — yes. New object types, new relationship semantics, new protected authorization
dimensions, new integrations — no. SAP itself frames A–D as "an actionable roadmap" rather than a state,
which is a fair indicator of how much real customer code sits at C/D.

`CONFIRMED` — [learning.sap.com — Clean Core Extensibility Best Practices](https://learning.sap.com/courses/practicing-clean-core-extensibility-for-sap-s-4hana-cloud/explaining-extensibility-model-best-practices_e290f382-800e-40ef-a203-85a13115f487);
`LIKELY` — [blogs.sap.com — Key User Extensibility: Adding Custom Business Logic](https://blogs.sap.com/2017/05/28/key-user-extensibility-on-sap-s4hana-cloud-adding-custom-business-logic/) (2017-05-28);
[SAP Community — Dos and Don'ts: Key User Custom Fields](https://community.sap.com/t5/enterprise-resource-planning-blog-posts-by-sap/dos-and-don-ts-key-user-custom-fields-in-add-ons-based-on-sap-s-4hana-cloud/ba-p/14340833) — accessed 2026-07-29.

**SAP Build Process Automation boundary** (asked specifically): SBPA is the BTP-side low-code option,
merging the former SAP Workflow Management and Intelligent RPA. Its **Decisions** artefact "decouple[s]
complex rules from your code, making them easier to maintain and evolve," is authorable by business
users, and "can be built and consumed independently without the need to create a process." So business
users author processes, decision tables and bots; developers are still needed for anything crossing a
system boundary that lacks a released API. `LIKELY` —
[SAP Press — Central Decision Management in SBPA](https://blog.sap-press.com/central-decision-management-in-sap-build-process-automation);
[SAP — SBPA features](https://www.sap.com/products/technology-platform/process-automation/features.html) — accessed 2026-07-29.

---

## 10. Cross-suite questions the redirect asked

### 10.1 What is genuinely shared vs separately modelled — **NEW**

The load-bearing table for a platform intending breadth.

| Concept | Exists once? | Reality | Conf |
|---|---|---|---|
| **A company (party: customer / supplier)** | **Once inside S/4HANA** — Business Partner with roles, mandatory | But **Ariba SLP** is often the entry system and **MDG** the governed authoring source, so the same supplier is authored in one place, governed in another, operational in a third. Duplicate/inconsistent party data is the routine failure | LIKELY |
| **An employee** | **Once in SuccessFactors EC**, which "serves as the system of record that feeds data to every other SuccessFactors module" | But EC↔S/4HANA needs replication (employee one way, cost centre the other) over Cloud Integration / Master Data Integration; and **Concur** holds its own employee + approver-limit record | LIKELY |
| **An external worker** | **No — twice by design.** Fieldglass is master; EC holds a mirror distinguished by a **source-system flag** | The flag is the tell (§2.3) | LIKELY |
| **A cost centre** | Owned by S/4HANA CO | Replicated *into* EC; with the S/4HANA 2025 release the `ODTFINCC` component is deprecated and **Master Data Integration becomes required** | LIKELY |
| **A customer / contact for CRM** | **No** — Sales Cloud, Service Cloud, Commerce Cloud, Customer Data Cloud and Engagement Cloud each hold contact data and integrate to the S/4 BP | Five CX products, several contact stores | LIKELY |
| **A financial posting** | **Once, genuinely** — `ACDOCA` | The one place the suite fully delivers "modelled once" | CONFIRMED |
| **An approval authority** | **No** — Concur has a readable per-person limit object (§6.1); S/4HANA has strategies + roles; SBPA has decision tables; GRC has its own request workflow | Four mechanisms, one concept | LIKELY / CONFIRMED |
| **A role / permission** | **No** — every product has its own; GRC exists partly to see across them | Hence cross-system SoD as a distinct problem (§4.3) | LIKELY |

Sources: [SAP Licensing Experts — EC licensing](https://saplicensingexperts.com/blog/sap-successfactors-employee-central-licensing);
[SAP Press — How to Integrate EC with S/4HANA](https://blog.sap-press.com/how-to-integrate-sap-successfactors-employee-central-with-sap-s4hana);
[SAP Help PDF — Replicating Cost Centers from S/4HANA to EC](https://help.sap.com/doc/d6f4318b83de4b06bcc53e92cb50c42a/2511/en-US/SF_S4_EC_CC_HCI_en-US.pdf);
[blogs.sap.com — Seamless Supplier Integration with Ariba powered by MDG](https://blogs.sap.com/2020/10/06/experience-seamless-supplier-integration-with-sap-ariba-powered-by-master-data-governance/) — accessed 2026-07-29.

### 10.2 Seams that hurt — **NEW**, sourced

1. **EC ↔ S/4HANA cost-centre replication.** SAP's own KBAs and community threads document: successive
   cost-centre changes replicating incorrectly (the employee reverting to the original cost centre);
   `externalCode` field-length mismatches between EC and ERP; `USE_EXTERNAL_COST_CENTER` left False; MDI
   consumer activation showing Inactive; integrations failing on metadata constraints. Plus a moving
   target: `ODTFINCC` deprecated at S/4HANA 2025 with MDI becoming required. `LIKELY` —
   [SAP KBA 3196423](https://userapps.support.sap.com/sap/support/knowledge/en/3196423);
   [SAP Community — incorrect replication of successive cost-centre changes](https://community.sap.com/t5/human-capital-management-q-a/incorrect-replication-of-successive-cost-centre-changes-from-sap/qaq-p/14309972).
2. **Ariba ↔ S/4HANA supplier master.** "Master data mismatches, timing issues and approval conflicts
   cause most failures," with duplicated supplier data, missing banking fields and inconsistent naming
   across systems. Compounding it: "the integration of SAP Ariba with ERPs is very standardized, which
   creates a conflict when organizations rely on customized processes, fields, and local extensions" —
   i.e. the integration is brittle exactly where a business is distinctive. `LIKELY` —
   [LeverX — Ariba integration with ECC and S/4HANA in 2026](https://leverx.com/newsroom/how-to-integrate-sap-ariba-with-sap-erp-and-saps4hana);
   [STL Digital — Ariba and S/4HANA integration](https://www.stldigital.tech/blog/accelerating-strategic-procurement-fast-tracking-sap-ariba-alongside-s-4hana-transformation/).
3. **The CVI shim (§3.2)** — a permanent synchronisation layer plus `BD001`/`BC001` key mapping, kept
   alive indefinitely so downstream code expecting the old tables keeps working.
4. **Fieldglass ↔ EC source-system flag (§2.3)** — the mirror is the design, not a defect, and the flag
   is how the suite admits it.

**The pattern:** every seam is a place where **two products each believe they own the same noun**. None
are integration-technology problems; they are model-ownership problems that integration technology is
asked to paper over.

### 10.3 What SAP charges for separately that one platform could offer as one thing — **NEW**

Read as a signal of where the architecture forced a split, not as pricing commentary.

| Separately sold | The single concept underneath | What the split reveals |
|---|---|---|
| **GRC Access Control** | "May this person hold these two capabilities together?" | The core model is additive and positive-only, so mutual exclusion had to be an external detector (§4.2) |
| **MDG** | "What is the authoritative version of this party / material?" | The ERP was never the sole authoring surface once the portfolio grew |
| **Fieldglass** + **SuccessFactors EC** | "Who works here?" | Employment status was baked into the record *type* instead of being a property of one worker object |
| **Concur** | "Who may commit how much money?" | Expense grew its own authority object because the ERP core had none (§6.1) |
| **Ariba** + S/4HANA MM | "Who do we buy from, under what agreement?" | Supplier collaboration and supplier posting were built in different eras |
| **Global Batch Traceability** | "Which inputs produced this output?" | Lineage stops at a system boundary, so cross-system lineage became a product |
| **Group Reporting** | "What does the group look like?" | Consolidation is an application over the journal rather than a property of the entity ladder |
| **Signavio** + **LeanIX** | "What are our processes and which systems run them?" | Process and application knowledge live outside the system that executes them |
| **Sustainability Control Tower** | "What did this activity emit?" | Emissions were never a dimension on the posting the way cost is (§7.2) — the instructive near-miss |
| **EWM / TM advanced tiers** | "Where is the stock and how does it move?" | Metered add-ons over an embedded base |

The last substantive row is the sharpest: SAP got "cost is a dimension of the transaction" exactly right,
then built emissions as a **separate reporting product** importing data "preserving native source
granularity" — re-deriving downstream what could have been a coordinate on the posting.

### 10.4 Converging / retiring as of 2026 — **NEW**

| Item | Status | Conf |
|---|---|---|
| **ECC** | Mainstream maintenance ends 2027-12-31 (EHP 6–8); extended to 2030-12-31 at ~+2pp; a reported "SAP ERP, private edition, transition option" extends select complex customers beyond 2030 | LIKELY |
| **Business ByDesign** | Net-new sales stop **2026-04-20**; existing customers supported with no announced end; successor = SAP Cloud ERP / Business One | LIKELY |
| **Emarsys → SAP Engagement Cloud** | Renamed **2026-02-19**; Emarsys Edition (unchanged) + new Enterprise Edition | LIKELY |
| **Public + Private editions → "SAP Cloud ERP"** | Umbrella renaming, 2025 | LIKELY |
| **Qualtrics** | **Divested 2023-06-28** — not an SAP product | CONFIRMED |
| **Costing-based CO-PA → Margin Analysis** | Replaced in Public Cloud | CONFIRMED |
| **Business area** | Absent from Public Cloud; compatibility-only on-prem | LIKELY |
| **`ODTFINCC` cost-centre replication** | Deprecated at S/4HANA 2025; MDI required | LIKELY |
| **Classic release strategy → Flexible Workflow** | Both shipped; flexible workflow is the direction (§6.2) | CONFIRMED |
| **Global Batch Traceability** | Roadmap position unclear; 3.0 dates to 2017 | UNCERTAIN |
| **SAP GUI** | Still shipped, still part of S/4HANA strategy | LIKELY |

(Sources for this table are the same as the corresponding rows in §1 and §8.)

---

## 11. What it costs to get — **NEW (numbers); extends the corpus's qualitative "consultant-heavy"**

The corpus says "consultant-heavy" in at least four places and carries no figures and no failure modes.

- **Duration.** Full ECC→S/4HANA migration commonly 18–36 months; brownfield 12–24; phased 18–36.
  `LIKELY` — [SAVIC roadmap 2026](https://www.savictech.com/insights/sap-s4hana-implementation-roadmap-enterprise-guide-2026/).
- **Cost.** Panorama's 2025 *Clash of the Titans* puts the **median project cost in the SAP cohort at
  $2.5M** — median, so half sit below; large programmes are quoted $3M–$12M+. `LIKELY` — cited via
  [top10erp pricing guide](https://www.top10erp.org/products/sap-s-4hana/pricing).
- **Subscription.** Public Cloud published list from **~$180/user/month**, to ~$400 for
  private/extended-functionality users, with the analyst caveat that no large US enterprise pays list.
  `LIKELY` — [erpimplementationcost](https://erpimplementationcost.com/vendor/sap-s4hana/);
  [Redress Compliance benchmarks](https://redresscompliance.com/sap-s-4hana-cloud-pricing-benchmarks-for-u-s-enterprises.html).
- **Where the money goes — the most useful ratio here.** Systems-integrator fees are commonly cited at
  **40–60% of total implementation cost**, and implementation overall at **1×–3× the first-year licence
  or subscription fee**. **The software is a minority of the spend.** `LIKELY` —
  [ERP Research — implementation cost breakdown](https://www.erpresearch.com/en-us/erp-implementation-cost-breakdown).
- **Consultant dependency.** UK median SAP consultant contract day rate **£581** (six months to
  2026-01-16), S/4HANA specialists ~**£610/day**; European freelance **€350–€1,500/day**. Demand has
  exceeded supply since late 2025 and is expected to through the 2027 deadline. Structural, not cyclical:
  a fixed pool who know both ECC and S/4HANA against a deadline-driven spike. `LIKELY` —
  [ITJobsWatch](https://www.itjobswatch.co.uk/contracts/uk/sap%20consultant.do);
  [keyusertraining — freelance day rates 2026](https://keyusertraining.com/en/freelance-sap-consultant/).
- **UI friction.** Nearly 90% of SAP customers have deployed / are deploying / plan Fiori, yet a reported
  **~40% of critical workflows still run on classic SAP GUI or third-party apps after migration** —
  because some GUI transactions have no Fiori equivalent, Fiori can be slower, and one deep transaction
  gets split across several apps. `UNCERTAIN` on the 40% (single secondary source) —
  [heflo](https://www.heflo.com/blog/sap-fiori-for-power-users).
- **Published failure modes** (all secondary-sourced). **Lidl** — ~€500M (some sources $600M) over seven
  years, then cancelled and reverted to legacy; the reported root cause is the instructive part: Lidl
  valued inventory at **purchase price**, SAP's retail model at **retail price**, and the programme
  customised around a fundamental modelling mismatch rather than changing either side. **Revlon** — ~$64M
  of net sales unshippable after go-live and a rare *shareholder* class action (2019). **National Grid** —
  ~$585M including remediation; Wipro settled for $75M. **Waste Management** — $500M lawsuit against SAP,
  settled. **HP** — a $30M migration reportedly costing $160M in lost sales. **Hershey** — >$100M
  attributed to poor planning and inadequate testing. `LIKELY` —
  [Henrico Dolfing — Lidl case study](https://www.henricodolfing.ch/en/case-study-12-lidls-e500-million-sap-debacle/);
  [Consultancy.uk](https://www.consultancy.uk/news/18243/lidl-cancels-sap-introduction-having-sunk-e500-million-into-it);
  [Computer Weekly — Revlon](https://www.computerweekly.com/news/252464278/SAP-disruption-leads-to-Revlon-class-action-lawsuit);
  [TechTarget — 12 ERP failures](https://www.techtarget.com/searcherp/feature/7-reasons-for-ERP-implementation-failure).

**The pattern across the failures is not "SAP is broken."** It is: *the business's model and SAP's model
disagreed, and the programme tried to resolve the disagreement in software.* Lidl is the purest case.

---

## 12. Already covered — pointers only, no paraphrase

| Topic | Existing row |
|---|---|
| Substitution planned/unplanned; release-code authority-by-level as the 전결 analogue | `appr.md:60` (corrections + detail in §6.2, §6.4) |
| 전결규정 as release-code hierarchy; 합의/협조 not native | `appr.md:160` (**stale** — §6.2) |
| 전결 / 대결 / 후결 / out-of-office across vendors, incl. the KR legal caveat (Commercial Act, 89DaKa3677) and the "configurable authority semantics, not blanket legal requirement" framing | `appr.md:58-67` |
| SAP approve/reject with reason; configurable decline routing | `appr.md:71` |
| SoD ruleset as highest-governance-ROI steal; mitigation-control library | `lenses/governance.md:155`, `INDEX.md:21`, `compliance.md:149` |
| GRC MSMP, Firefighter, UAR recertification, consultant-heavy authoring | `policy.md:121`, `policy.md:49`, `policy.md:148` |
| Org-level / role / system scoping and cross-system risk | `policy.md:69` |
| SuccessFactors first-class positions / jobs / org units / cost centres; "steal position/org-management rigor" | `people.md:74`, `people.md:159`, `INDEX.md:45` (**precision fix** — §13.4) |
| EC Payroll with official Korea localisation | `people.md:81` |
| SF MDF effective-dated; Workday `Worker→Position→JobProfile→SupOrg`; correct-vs-new-effective-change | `lenses/data-model.md:107-108,112-113,126` (**extension** — §13.5) |
| Workday BP-framework generalisation as the reusable approval abstraction | `people.md:204`, `INDEX.md:38` |
| SAP margin / cost-centre / profit-centre analytics as native strength | `dashboard.md:80` (**upgrade** — §7.3) |
| Universal Journal, BKPF header + ACDOCA lines, park→post, Dr=Cr, OB52 period control | `finance.md:41,113` (**upgrade** — §7.1) |
| Document flow / source-to-posting lineage as a benchmark row | `finance.md:49` (**extension** — §7.2, §8) |
| Real-time TB / P&L / BS off ACDOCA, no subledger reconciliation at close | `finance.md:107` |
| MDG as governed-CR / branch-proposal benchmark; MDG data model = entity types + attributes + relationships; USMD/MDF no-code authoring | `object-platform.md:70,78,84,169-175,222`, `INDEX.md:29` (**single-record story** — §3) |
| SBPA decision-table approver determination | `INDEX.md:45` (**boundary** — §9) |
| SAP Overview Page / Fiori smart-filter bar and honest counts | `dashboard.md:208`, `INDEX.md:30` |
| SAP Object Page anchored sections | `INDEX.md:25` |
| Concur 1-click approve + glance card; NetSuite bulk approve | `lenses/task-flow.md:46,48,49`, `finance.md:167`, `INDEX.md:18` (**authority model** — §6.1) |
| SAP FSM AI dispatch scheduler + Crowd Service | `INDEX.md:45` |
| 그룹 obligation cascading to 계열사 unmodelled by all 7 | `compliance.md:248` (**needs qualifying** — §3.3) |

---

## 13. Where the corpus is wrong or stale — the high-value flags

1. **`compliance.md:248` overstates the differentiator.** "None of the 7 vendors model a Korean
   conglomerate hierarchy natively" — but **SAP MDG ships a governed, application-agnostic legal
   business-partner hierarchy representing company ownership**, explicitly for a single source of truth
   on legal ownership across applications. Split the claim: *the ownership hierarchy is modelled by SAP;
   cascading obligations down it is not.* The narrower claim is defensible; the current one is not.
   `LIKELY`, §3.3.
2. **`appr.md:160` is stale for S/4HANA.** Release strategies are the ECC-era mechanism. Since S/4HANA
   the current path is **Flexible Workflow** — configured by a business user in a Fiori app with
   company-code / total-net-amount / account-assignment conditions and and/or linkage — plus
   **Responsibility Management** for approver determination. `CONFIRMED`, §6.2, §6.3.
3. **`appr.md:60` conflates the authority point with the amount band.** The release *code* is the signing
   point; the *threshold* is a characteristic of the release strategy (or a workflow condition). And a
   release code is not delegated — delegation is workflow substitution. `CONFIRMED`, §6.1, §6.2.
4. **`people.md:74` conflates two different SAP products.** "SAP SF: **Organizational Management** =
   first-class positions, jobs, org units, cost centers" — *Organizational Management* is the
   **on-premise SAP HCM** module, whose model is a typed relationship graph: object types (`O` org unit,
   `S` position, `C` job, `P` person), existence in **infotype 1000** ("infotype 1000 defines the
   existence of an object in the system"), **every relationship a record in infotype 1001**, reciprocal
   with automatic inverses, carrying **validity periods**, and with **customer-definable object *and
   relationship* types**. **SuccessFactors EC Position Management** is a different, MDF-object-based
   model. The steal-item should name which one — and the on-prem graph is the more interesting of the
   two. `CONFIRMED` — [learning.sap.com — Modifying the Data Model](https://learning.sap.com/courses/organizational-management-in-sap-hcm-for-s-4hana/modifying-the-data-model).
5. **`lenses/data-model.md:108` under-describes SuccessFactors.** Add: Position is an **MDF object** (so
   it takes custom fields, associations, business rules and workflows like any other MDF object) and is
   effective-dated; and — load-bearing — "**by default, Position Hierarchy is the leading hierarchy**",
   distinct from the employee→manager reporting hierarchy. Also **incumbent tracking** ("whether an
   employee on a global assignment or a leave of absence have a right to return into their position") and
   **position-based permissions**. In a job-based structure job details live on the employee record and
   are lost when they leave; in a position-based structure they live on the position, which survives
   vacancy — which is what makes headcount and budget planning possible. `CONFIRMED` —
   [learning.sap.com — Describing EC Position Management](https://learning.sap.com/courses/sap-successfactors-employee-central-position-management-academy/describing-sap-successfactors-employee-central-position-management_bcd65377-bc96-400d-9df9-c31c228a8ecf).
6. **`finance.md:41` and `dashboard.md:80` are correct but marked `[I]`** — both can be upgraded to
   sourced (§7.1, §7.3). Not errors; cheap credibility.
7. **Portfolio-level facts a corpus reader could get wrong if they extend it:** `Qualtrics` was
   **divested in 2023**; `Emarsys` is now **SAP Engagement Cloud** (2026-02-19); `Business ByDesign`
   stops net-new sales **2026-04-20**; and `business area` is gone from Public Cloud.

---

## 14. Concepts worth adopting — whole suite

New or newly-sharpened only; the corpus's existing 20-item steal-list stands.

1. **Party = one entity with roles, from day one.** *(§3, NEW.)* SAP concluded a customer and a supplier
   are one Business Partner with two roles — and paid for reaching that conclusion late with a mandatory
   migration, a permanent CVI sync shim, and eternal `BD001`/`BC001` key mapping. **Getting this right at
   the start captures the entire benefit at none of the cost.** The strongest single argument in the suite.
2. **Engagement type as a property of one worker, not a separate record type.** *(§2, NEW.)* Steal SAP's
   three-way split — contingent / SOW-services / profile — because it keys on **who directs** and **who
   pays**, exactly the 파견 vs 도급/용역 line. Reject the two-masters-plus-a-flag implementation: model it
   as a property and the source-system flag never needs to exist.
3. **The work order as the authority object for external labour.** *(§2.2, NEW.)* A binding
   buyer↔supplier contract carrying rate, cost coding and right-to-work, with the worker downstream of it.
   Authority between two legal entities is a first-class document, not an attribute on a person.
4. **A readable per-person approval-authority object, bound to a cost object.** *(§6.1, NEW.)* Concur's
   Authorized Approver — currency + amount limit, optional exception level, cost-object variant, with
   escalation up the hierarchy when no approver is found at a level. **This is the 전결규정 artefact, and
   it already exists inside the suite.** It answers "what is this person's authority?" as a lookup, and
   turns `appr.md:160` from "SAP can't do this" into "SAP's expense product can; its ERP can't."
5. **Approval bound to document *state*, not document ID.** *(§6.5, NEW.)* A value change past a threshold
   resets the release, closing the edit-after-approval hole structurally.
6. **Mutual exclusion in the model, so SoD needs no second product.** *(§4.2, extends
   `lenses/governance.md:155`.)* Reframes the existing steal-item: the point is not to copy GRC's ruleset,
   it is that **a model able to express "these two may not co-occur" does not need a detector running
   after the fact.**
7. **The multi-dimension entity ladder, with the four constraints as requirements.** *(§5, NEW.)* Legal
   entity / management responsibility / external-reporting division / site / cost home as independent
   coordinates on one transaction. Take the constraints as design rules: don't derive one reporting
   dimension from another (segment←profit centre); **version hierarchy edges**; don't charge a shared
   chart of accounts for cross-entity allocation.
8. **A tax-filing organisational level below the legal entity.** *(§5, NEW.)* SAP's **business place**
   exists because some jurisdictions require sub-entity returns, and it names South Korea. 사업장 is a
   **level** with obligations (VAT registry number, official document numbering), not an attribute.
9. **The business object as a dimension of the posting, plus real-vs-statistical assignment.** *(§7.2,
   extends `finance.md:49`.)* One field per account-assignment object on the journal line, so "what did
   project P cost" is a filter. **Real vs statistical** resolves double-counting declaratively: report
   against many, own by exactly one.
10. **Allocation as a posting with a durable pre-posting simulation.** *(§7.4, NEW.)* Universal
    Allocation's test run "is saved into the system." Distribution preserves the original account;
    assessment posts through a secondary cost account so the allocation is visible as its own cost type.
11. **Alternative views as thin overlays, not copies.** *(§7.5, NEW.)* Extension ledgers layer deltas on a
    base ledger without duplicating its rows.
12. **Clock-driven events emit ordinary documents.** *(§7.6, NEW.)* Depreciation is a periodic run
    producing journal entries indistinguishable downstream — one document model, one audit trail, one
    reversal path for both time-driven and action-driven economics.
13. **Cost collector + settlement rule.** *(§7.7, NEW.)* Work accumulates cost on itself, then declares
    who bears it — separating "what did this job cost" from "whose budget takes it".
14. **Organisation as a typed, time-bounded, extensible relationship graph.** *(§13.4, corrects
    `people.md:74`.)* SAP HCM OM: existence in infotype 1000, every relationship a record in infotype 1001
    with validity periods and automatic inverses, and **customer-definable relationship types**. 25 years
    old and still the most sophisticated organisational model in this survey.
15. **Position-based, with the position hierarchy leading.** *(§13.5, extends `lenses/data-model.md:108`.)*
    The seat survives vacancy; right-to-return is modelled; permissions can derive from the position and
    hierarchy rather than the person.
16. **A published extension-risk ladder (A–D).** *(§9, NEW.)* Turns "is this customisation safe?" from an
    argument into a lookup, and makes the cost of a shortcut explicit when it is taken.
17. **Emissions as a coordinate on the posting, not a downstream report.** *(§10.3, NEW — inverting SAP.)*
    SAP got cost right as a dimension and then built sustainability as a separate product that re-imports
    granularity. The near-miss is the lesson: if a quantity will ever need attribution to an entity, a
    period and a responsibility, it belongs on the posting.

---

## 15. Concepts to reject — whole suite

1. **Two masters for one noun, reconciled by a source-system flag.** *(§2.3, §10.1.)* Fieldglass+EC for
   workers; Ariba+MDG+S/4 for suppliers; five CX products for contacts. **Every seam in §10.2 is a
   model-ownership problem that integration technology was asked to paper over.** The flag is the
   confession.
2. **Unifying a model after shipping the split.** *(§3.2.)* Mandatory migration + permanent compatibility
   shim + eternal key mapping. If two things are one thing, say so before there is data.
3. **SoD as a bolt-on product.** *(§4.2.)* Detection after the fact means the conflict existed.
4. **Approval authority as an intersection of process config and role assignment, with no readable
   artefact.** *(§6.1.)* You cannot print "what is Kim's authority as of today" from S/4HANA. A 전결규정
   has legal force; if the system cannot render it, a spreadsheet becomes the source of truth.
5. **One concept with four mechanisms across the suite.** *(§10.1.)* Concur limits vs S/4 release
   strategies vs SBPA decision tables vs GRC request workflows.
6. **Thresholds living in module-local configuration.** Purchasing has release strategies; FI has
   tolerance groups (`OBA4`); expense has approver limits. Every new document type reinvents them.
7. **One reporting dimension hard-wired as a function of another.** *(§5.1.)* Segment derivable *only*
   from profit centre, with a BAdI as the escape hatch.
8. **Non-versioned hierarchy edges.** *(§5.1.)* Cost-centre standard-hierarchy reassignment silently
   restates history. Reorganisation must not rewrite the past.
9. **A shared chart of accounts as the price of cross-entity cost allocation.** *(§5.1.)* Forces a global
   decision before the first posting; an acquisition breaks it immediately.
10. **A fixed structural ceiling on approval depth.** Standard SAP allows 8 release levels. Discovering a
    hard limit in configuration is a bad way to learn a design constraint.
11. **Two-character identifier namespaces** (release codes `FRGCO`, OM object types). 1990s identifier
    economy producing unreadable configuration and namespace exhaustion. Readable identifiers are free.
12. **Batch-granularity lineage with no proportional attribution.** *(§8.)* The graph gives adjacency, not
    fractions. If quantity provenance matters, put quantities on the split edge.
13. **Lineage that stops at a system boundary, sold as a separate product.** *(§8 — GBT, whose 2026
    roadmap position is unclear.)*
14. **A dimension re-derived downstream instead of captured at posting.** *(§10.3.)* Sustainability
    Control Tower importing footprint data "preserving native source granularity" is the tell.
15. **Business area.** A dimension SAP itself abandoned, kept alive only by installed base.
16. **Two UIs, permanently.** *(§11.)* ~40% of critical workflows reportedly still on SAP GUI after
    migration, with GUI retained as strategy. The clearest case of "merely entrenched" rather than
    "genuinely good" — and a consequence of not having a UI-independent model to begin with.
17. **Customising around a fundamental model disagreement.** *(§11, Lidl.)* Purchase-price vs
    retail-price inventory valuation was a business-model question answered in software for ~€500M.

---

## 16. Open questions

1. **Korean 파견법 handling in Fieldglass.** Does SAP publish any 불법파견 / 위장도급 classification
   content, or is the decision wizard US/EU-shaped only? `UNKNOWN` — the one place SAP's
   external-workforce model could fail exactly where we need it.
2. **Can Fieldglass express 파견 duration limits and 직접고용 obligation triggers** (the "contractor is now
   technically an employee" case), or is that a partner / EOR overlay? `UNKNOWN`.
3. **Is there any SAP artefact that renders the effective approval matrix for the ERP core?** Concur has
   one; I found nothing equivalent for S/4HANA. Absence of evidence is weak here.
4. **Parallel (합의) vs sequential (순차) semantics in Flexible Workflow.** Multiple approvers per step is
   confirmed; all-must-approve vs any-one vs configurable is not.
5. **Can an ERP-core approval limit be a per-person attribute at all**, or is the strategy/workflow
   indirection unavoidable? Concur's existence suggests it was a choice, not a constraint.
6. **Does MDG's legal-ownership hierarchy support obligation propagation** in any form, or purely
   ownership representation? This determines exactly how much of `compliance.md:248` survives.
7. **Does batch genealogy carry quantities on split/merge edges anywhere** in SAP, including GBT? Sources
   describe adjacency only.
8. **GBT's 2026 status** — maintained, superseded by Batch Release Hub, or maintenance-only?
9. **Korean payroll completeness in EC Payroll** — which of 4대보험 / 연말정산 / 퇴직금 are standard vs
   partner add-on? Locale counts differ across sources (50 / 53 / 60), and SAP's published 연말정산 tooling
   is a **prototype**, which suggests gaps. Extends `people.md:81`.
10. **Does business place exist in S/4HANA Cloud Public Edition** with the same semantics? Confirmed
    on-premise only, and it matters given business area's absence there.
11. **Do key-user custom fields on `ACDOCA` become genuine reporting *dimensions* or only attributes?**
    Sources assert they flow to reporting; the distinction is load-bearing for §7.2.
12. **What fraction of real S/4HANA customers reach clean-core level A?** SAP frames A–D as "an actionable
    roadmap", implying most do not. No figures found.
13. **Are profit-centre hierarchy nodes versioned** the way consolidation hierarchy nodes are, given
    cost-centre standard-hierarchy assignment is not? Needs primary confirmation.
