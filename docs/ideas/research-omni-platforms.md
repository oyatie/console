# Omni / all-in-one platforms — delta over `docs/program/benchmark-matrix/`

Status: RESEARCH — sourced, confidence-labelled
Researched 2026-07-29. Structured as a **delta**: every finding is either new-with-sources, or `EXTENDS <file>:<line>` naming exactly what it adds to the existing corpus. Nothing already in the corpus is paraphrased — covered ground gets a pointer line only.

**Confidence labels.** `CONFIRMED` — vendor primary docs, developer docs, or financial filings I read directly. `LIKELY` — two or more independent reputable sources, or one strong source with no contradiction. `UNCERTAIN` — single secondary source, or a partner/consultancy blog with a commercial interest in the answer. `UNKNOWN` — could not source; stated as a gap.

**Standing caveat on sources.** Most writing about these platforms is by implementation partners selling the implementation. They systematically overstate no-code reach and understate cost. Partner-only claims are `UNCERTAIN` even when plausible — which is why the no-code and cost sections below are hedged harder than the architecture sections. Salesforce's architect docs and Workday's architecture page both blocked automated fetch; those two substrate descriptions lean on Salesforce's own 2008 multitenancy whitepaper and a strong technical secondary source respectively, labelled accordingly.

---

## 0. Already answered in the corpus — pointers, not re-research

| Question | Corpus location |
|---|---|
| Workday `Worker → Position → JobProfile → SupOrg` nesting | `lenses/data-model.md:107` |
| Workday correct-vs-new-effective-dated-change | `lenses/data-model.md:113,126` |
| Workday BP-framework generalisation as one `BusinessProcessDefinition` | `people.md:204`, INDEX below-the-cut |
| Rippling auto re-route on approver vacation; authority follows the org | `appr.md:64` |
| SAP substitution rules + release-code delegation as 전결-like | `appr.md:60` |
| SAP margin / cost-centre / profit-centre as native strength vs our omitted 수익성 | `dashboard.md:80-83` |
| 결재선 / 문서양식 typed forms in 더존 · Naver Works · Hanbiro · Flow | `lenses/data-model.md:328,342` |
| NetSuite bulk-approve as the N-updates-in-1 collapse | `lenses/task-flow.md:47-49,178` |
| NetSuite SuiteFlow as finance automation | `lenses/automation-ext.md:205-207` |
| ServiceNow table-extend, reference fields, GRC control classes, FSM dispatcher | `lenses/data-model.md:159,352,375`, `field.md`, `compliance.md` |
| Salesforce field history (20 fields / 18 months), master-detail cascade, Console workspaces | `lenses/data-model.md:206,244`, INDEX steal #6 |
| Dataverse as "bolt-on and fragmented across four products" | `object-platform.md:192-193` |

Absent from the corpus entirely, verified by grep: **Odoo, Zoho One, WEHAGO/더존 as an ERP substrate (only 결재선 is covered), Dooray, Kakao**. Ecount appears once (`dashboard.md`), Airtable once (`leave.md`), monday once (`automate.md`), all as feature references, none as substrate.

---

## 0b. STALENESS FLAG — stronger than suspected

`CONFIRMED`, checked 2026-07-29 by directory listing only (no source read).

`INDEX.md:5` states every module file is "an evidence-based read of our console (`web/src/console/**` + backend crates + `docs/program/console-program-ledger.md`)".

**`web/` does not exist. There is no frontend tree at the repo root at all, and no replacement** — `find -maxdepth 4 -type d -name console` returns only `scripts/console`, `deploy/apps/console`, `docs/evidence/console`. `backend/` and the 227 KB ledger survive.

So: every "Ours" column, every `[code]` citation into the frontend, and every IA/task-flow finding in all 20 files is evidence-bound to a deleted tree. This is not "may no longer hold" — the cited evidence base is gone. Specifically suspect: the whole of `lenses/ia-layout.md` and `lenses/task-flow.md`, INDEX steal #2 (overview inline decide), #6 (single 22rem panel), #9 (in-panel object page), #17 (⌘K empty), and every "source-observed / source-wired" claim. Backend-cited findings (`period_lock.rs`, migration `0107`, the four-eyes `CHECK`, `ACTION_DISPATCHES`) may still hold; they need re-verification against `backend/`, not assumption. **Vendor columns are unaffected** — they are externally sourced and independently checkable.

---

## 1. The question the corpus does not ask

The corpus compares **feature coverage per module**. It never asks **how a platform covering everything stays usable**. That is the breadth-versus-coherence question, and the answer across the uncovered vendors sorts into exactly three architectures. Naming them is the main analytic contribution here, because which type you are determines which failure modes you get.

**Type A — one object model, one runtime, one ledger.** Odoo, NetSuite, Ecount (and Workday, already in corpus). One schema; a person is one record; all modules post to one ledger with the business object as a dimension. Coherence is structural. Cost: the vendor owns the model, and *your extensions* are what breaks at upgrade.

**Type B — one metadata platform and one workflow engine, no ledger.** Salesforce, ServiceNow, monday, Airtable, Notion. A genuine shared substrate for records, security and workflow — and structurally incapable of being the book of record for money. Excellent at "add a vertical", categorically unable to answer per-contract profitability without a bolt-on.

**Type C — federated apps joined by sync.** Dynamics 365, Zoho One. Marketed as one platform; architecturally several sharing identity, billing and a synchronisation layer.

The corpus's own posture — ontology types + one governed Action verb + Cedar + a real GL with `period_lock` — is Type A with a Type B extension model. That combination is not represented by any vendor surveyed here, which is either the differentiator or the reason no one has done it.

---

## 2. Odoo — absent from corpus

### Substrate
`CONFIRMED`. One PostgreSQL database, one Python ORM, one model registry. Modules are not applications; they are additive layers on a shared registry. Three named mechanisms — "Classical inheritance", "Extension", "Delegation", "Fields Incremental Definition" ([Odoo 19 ORM API](https://www.odoo.com/documentation/19.0/developer/reference/backend/orm.html)):

- `_inherit` — a module extends an existing model **in place**. Installing `hr` does not create a parallel person table; it adds fields to models that already exist.
- `_inherits` — delegation. Child holds a FK to the parent and exposes its fields without storing them. Canonical case: `res.users` delegates to `res.partner`, so a user's name/email/phone physically live on the partner record.

### Shared entities — a genuine single party
`CONFIRMED`. `res.partner` is one party concept. Companies, individuals, customers, suppliers, contacts and users all resolve to partner records; roles are booleans and related data, not duplicate records.

`CONFIRMED`, and the mechanism is the interesting part. **Sharing is a null on the owning dimension.** Shared records are those with no `company_id`/`company_ids`; the record rule is "null company OR company ∈ my active companies". Integrity is enforced by `_check_company_auto = True` plus `check_company=True` on relational fields, so "a sales order and its invoice should not belong to different companies" ([Odoo 19 multi-company developer guidelines](https://www.odoo.com/documentation/19.0/developer/howtos/company.html)). One mechanism covers global master data, per-entity data, and the integrity rule between them.

### Approvals — attached to buttons
`CONFIRMED`. `EXTENDS people.md:204` and `lenses/data-model.md:342`. The corpus has Workday's BP framework as the reusable-approval reference and 결재선 as the Korean shape. Odoo adds a third, cheaper generalisation: **approval rules attach to buttons in views**, not to modules or document types ([Odoo 19 Studio approval rules](https://www.odoo.com/documentation/19.0/applications/studio/approval_rules.html)). Anything that is an action can be gated, so a new module inherits approvals for free — it has buttons. Native support: ordered multi-step, a domain filter per step, approvers as users and/or groups, "Exclusive Approval" (one person cannot satisfy two steps on one record), escalation from higher steps, audit in chatter plus an approval-entries log.

`CONFIRMED`, and the delegation granularity is the weakness worth recording: approvers delegate to another user **for all records**, with no time bound. Compare Workday (per-person, per-period — §13.3) and Salesforce (cannot select approvers mid-process at all).

`LIKELY`. A healthy third-party market for approval engines on 19.0 offering SLA deadlines, escalation and any-model mixins ([apps.odoo.com 19.0 listings](https://apps.odoo.com/apps/modules/19.0/approval_workflow_engine)) — evidence the native capability is a floor, not a ceiling.

### Economics — the cleanest "object as ledger dimension"
`CONFIRMED`. `EXTENDS dashboard.md:80-83`. The corpus has SAP CO/PA as the margin reference. Odoo reaches the same outcome far cheaper: one ledger, and from v17 **analytic plans** replaced single analytic accounts — a journal line carries several independent analytic dimensions simultaneously (project, department, cost centre), and one amount splits by percentage across analytic accounts, automatically via distribution models ([Odoo 19 analytic accounting](https://www.odoo.com/documentation/19.0/applications/finance/accounting/reporting/analytic_accounting.html)). Per-project profitability is a query over the ledger, not a sub-ledger.

### Org model
`CONFIRMED`. Company as a record with parent/child hierarchy; multi-company as a per-user active-companies set; Inter-Company Transactions auto-generate the counterpart order or invoice ([Odoo 19 multi-company](https://www.odoo.com/documentation/19.0/applications/general/companies/multi_company.html)). `LIKELY`: no generic org-unit abstraction — departments (HR), projects, warehouses/locations are separate models, so org shapes are per-domain.

### Config/code boundary and packaging
`CONFIRMED`. A vertical is a Python module with a `__manifest__.py`; the manifest carries commerce (`'price': 49.99, 'currency': 'EUR'`, EUR/USD only) and the author owns bug fixes in paid apps. Version numbers must embed the Odoo major version, `major-minor-bugfix` e.g. `10.0.1.1.3`, incremented for schema updates ([Odoo Apps vendor guidelines](https://apps.odoo.com/apps/vendor-guidelines)). A module is pinned to a platform major version by contract.

`CONFIRMED` and sharp: **you cannot install third-party or custom modules on Odoo Online** — custom code requires Odoo.sh or self-hosting ([Odoo 19 apps and modules](https://www.odoo.com/documentation/19.0/applications/general/apps_modules.html)). The extension model and the cheapest hosting tier are mutually exclusive.

`UNCERTAIN`, three independent partner blogs agreeing against their own interest: Studio does not cover complex business logic, external integrations, or specific workflows; crons and arbitrary method automation need backend Python ([Silent Infotech](https://silentinfotech.com/blog/odoo-1/odoo-studio-limitations-21), [Dixmit](https://www.dixmit.com/en/blog/our-blog-1/odoo-studio-risks-and-benefits-36), [LegionsSoft](https://legionssoft.com/odoo-19-studio-customization-guide-for-developers/)).

### Failure mode
`LIKELY`, the most concrete in the survey: **custom modules do not survive major upgrades.** "Custom modules built for Odoo 18 will not work in Odoo 19 due to changes in the API and underlying code" ([Cybrosys](https://www.cybrosys.com/blog/how-to-migrate-custom-modules-from-odoo-18-to-odoo-19)). Studio changes have no migration scripts (`UNCERTAIN`, [Dixmit](https://www.dixmit.com/en/blog/our-blog-1/odoo-studio-risks-and-benefits-36)). Customer complaint on Odoo's own forum, November 2025, unrebutted: "a continuous cycle of lack of proper support when customizations are involved and upgrade difficulties whenever even small customizations are present" ([forum 291246](https://www.odoo.com/forum/help-1/serious-concerns-about-odoo-support-quality-customization-handling-upgrade-limitations-requesting-management-attention-291246)).

---

## 3. NetSuite — corpus has bulk-approve and SuiteFlow only

### Shared entities — the best single-party model found
`CONFIRMED`. `EXTENDS lenses/data-model.md:83`, which records NetSuite as "✅ record types | ✅ | ◐ | ◐ | [I]" and nothing about identity.

**"Records as Multiple Types":** one entity record saved as several types at once — a vendor can also be a customer or partner. Available on customer, partner, vendor and other-name records; updating one updates all related; **the contact record shares the same internal ID as the original** ([Oracle NetSuite Help, Records as Multiple Types](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N1099012.html)).

`CONFIRMED`, and the exceptions matter more than the rule. Exactly two fields deliberately do **not** propagate: **Category**, because the valid list differs by role (Accounting Lists for vendors, CRM Lists for partners), and **Print Check As**, deliberately distinct to identify payees for accounting (same source).

That is the answer to the shared-master-data question, and it is neither "one record" nor "one per role": **one identity, role-scoped attributes, and a short explicit divergence list.** The design work is enumerating the list. NetSuite enumerated two.

`CONFIRMED` limit: internal and external IDs are **not unique across record types**, so every operation needs the pair (record type, id) ([Oracle NetSuite Help, using internal/external IDs](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N3432681.html)). Identity is scoped by type, not global.

### Org model — directly corrects a corpus claim
`CONFIRMED`. `EXTENDS compliance.md:248`, which states a 그룹 obligation cascading to 계열사 is something "none of the 7 vendors model". True of those seven — but **NetSuite OneWorld models a group of legal entities properly**, and so does Dynamics F&O (§4). Subsidiary is a first-class dimension: one account, many subsidiaries, international or domestic, each a separate company; consolidated reports translate child amounts to parent via a **Consolidated Exchange Rates** table ([OneWorld overview](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N266701.html), [subsidiaries in OneWorld](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N268563.html), [consolidated reporting](https://docs.oracle.com/en/cloud/saas/netsuite/ns-online-help/section_N278654.html)). The corpus's conclusion should narrow to: none of the *sampled* vendors, and the two that do it are both Type A ERPs.

### Economics — packageable ledger dimensions
`LIKELY`. `EXTENDS dashboard.md:80-83`. **Custom segments** are customer-definable GL-impacting dimensions: subsidiary is a standard dimension, segment values can be scoped to specific subsidiaries, and custom segments cut across subsidiaries (a global Business Unit code). Fully supported in SuiteCloud Development Framework and **bundleable into SuiteApps with predefined values** ([HouseBlend, April 2026](https://houseblend.io/articles/pdfs/netsuite-custom-segments-setup-gl-impact.pdf)).

Customer-definable, GL-impacting, *packageable* dimensions is the strongest economics-packaging story found: a vertical ships its own profitability axis.

### Pricing / failure
`LIKELY`, wide error bars: ~$999/month base plus $129–199/user/month; OneWorld a paid add-on reported at $500–1,000/month per additional subsidiary or $1,900–2,500+/month for the module; SuiteSuccess implementations from ~$25,000 to $750,000 for large multi-country deployments ([ERP Research](https://www.erpresearch.com/pricing/oracle-netsuite), [BrokenRubik, July 2026](https://www.brokenrubik.com/blog/netsuite-pricing-the-definitive-guide), [HouseBlend, May 2026](https://www.houseblend.io/articles/pdfs/netsuite-pricing-2026-license-costs.pdf)). Note the shape: **multi-entity is a paid feature priced per entity** — group structure is the monetisation axis. `UNKNOWN`: no rigorous independent study of NetSuite customer dissatisfaction found.

---

## 4. Dynamics 365 + Dataverse — corpus has one assessment line

### The retrofit — the highest-value finding in this document
`CONFIRMED`. `EXTENDS object-platform.md:192-193` ("Dataverse tables as the object store… capable but bolt-on and fragmented across four products") and **explains the mechanism behind** `lenses/data-model.md:205` (Dataverse "Eff-dated ✗").

It is not four products loosely joined. It is **two stores joined by sync, and the party concept was retrofitted by package.** Microsoft's own docs:

- Sales, Customer Service and Human Resources store data in Dataverse — then, on the same page: *"Finance and Operations apps currently require the configuration of the Data Integrator to make your business data from Finance and Operations apps available in Dataverse."* ([Microsoft Learn, What is Dataverse, updated 2026-03-31](https://learn.microsoft.com/en-us/power-apps/maker/data-platform/data-platform-intro)). **The ledger half of D365 is not on Dataverse.**
- Dual-write is "tightly coupled, bidirectional integration" — synchronous, with play/pause/catchup and offline modes ([dual-write overview, updated 2026-01-15](https://learn.microsoft.com/en-us/dynamics365/fin-ops-core/dev-itpro/data-entities/dual-write/dual-write-overview)). A good integration. An integration.
- Dual-write ships as **two marketplace solutions installed onto Dataverse**, and installing them changes the Dataverse schema: *"Dataverse includes new concepts such as company and party"*; activities and notes unified to cover both C1s (users) and C2s (customers); **date effectivity is added to Dataverse** to support past/present/future on one table; currency precision extended from 4 to 10 decimals via an opt-in metadata migration converting `money` to `decimal`, to prevent data loss in transmission (same source).

Read that list as a specification of what a substrate needs **before** modules are built on it: a party concept, a legal-entity concept, effective dating as a platform primitive, and adequate monetary precision. Microsoft shipped Dataverse without all four and added them by package. This is the strongest external evidence that the corpus's effective-dating-first posture is the right order of construction — and that `lenses/data-model.md:205`'s "✗" is not a missing feature but an unrecoverable substrate decision.

`CONFIRMED` and decisive on shared master data: Microsoft states the concepts do not line up — Dataverse business unit "is primarily a security and visibility boundary… doesn't have the same legal or business implications as the company concept", and *"Because business unit and company aren't equivalent concepts, you can't force a one-to-one mapping between them"* ([company concept in Dataverse](https://learn.microsoft.com/en-us/dynamics365/fin-ops-core/fin-ops/data-entities/company-data), [organization hierarchy in Dataverse](https://learn.microsoft.com/en-us/dynamics365/fin-ops-core/dev-itpro/data-entities/dual-write/organization-mapping)). Dual-write's own scenario list says "integrated customer master", "unified product mastering", "integrated vendor master" — the language of two records kept in agreement.

`CONFIRMED`, do not confuse: Dataverse's `ActivityParty` is older and unrelated — 12 party types in an integer bitmask `ActivityParty.ParticipationTypeMask`, about who was on a phone call ([ActivityParty table](https://learn.microsoft.com/en-us/power-apps/developer/data-platform/activityparty-entity)).

### Org model — purpose-typed hierarchies
`CONFIRMED`. `EXTENDS compliance.md:248` alongside NetSuite. F&O has a formal typology — legal entities, operating units, teams — and **each organisation hierarchy carries a "purpose" that determines which organisation types may appear in it and which application scenarios may use it** ([organizations and organizational hierarchies](https://learn.microsoft.com/en-us/dynamics365/fin-ops-core/fin-ops/organization-administration/organizations-organizational-hierarchies), [plan your organizational hierarchy](https://learn.microsoft.com/en-us/dynamics365/fin-ops-core/fin-ops/organization-administration/plan-organizational-hierarchy)). Each legal entity requires a ledger with its own chart of accounts, accounting currency, reporting currency and fiscal calendar.

**Purpose-typed hierarchies are the best org-modelling idea found.** One company sits simultaneously in a legal-consolidation hierarchy, a procurement-authority hierarchy and a sales-territory hierarchy, without three org models — and "which hierarchy governs this decision" becomes answerable. Directly applicable to 그룹 → 법인 → branch → worksite, where the same entities need different trees for statutory consolidation versus 결재 authority.

### Approvals — two engines
`LIKELY`. F&O has its own workflow engine scoped to F&O ([workflow FAQ](https://learn.microsoft.com/en-us/dynamics365/fin-ops-core/fin-ops/organization-administration/workflow-FAQ)); Power Automate is the cross-application engine with 300+ connectors and approval via email/Teams/push ([Nalashaa](https://nalashaadigital.com/blog/microsoft-flow-and-workflows/), [Avantiico](https://avantiico.com/enhancing-d365-fscm-with-power-automate-approval-workflows/), [Arctic IT](https://arcticit.com/automating-financial-workflows-in-dynamics-365-finance/) — partner sources). The recommended pattern is F&O workflow for in-ERP approval plus Power Automate on top so approvers never enter the ERP. Two places to express one policy: the cost of Type C.

`UNCERTAIN` / `UNKNOWN`: dual-write lists "project-to-cash" as supported, which says the scenario exists, not how clean the margin number is or what latency sits between project and ledger. Gap.

---

## 5. Salesforce Platform — corpus has field history and Console IA

### Substrate — why custom objects are cheap
`LIKELY`. `EXTENDS lenses/data-model.md:206`. The corpus records *what* Salesforce's model does; the mechanism explains what it costs.

Metadata-driven multitenancy: org-specific objects, fields, procedures and triggers are virtual constructs described by metadata held in a few physical tables — the **Universal Data Dictionary** — and a runtime engine materialises all application data from metadata ([Force.com Multitenant Architecture whitepaper, salesforce.com](https://www.developerforce.com/media/ForcedotcomBookLibrary/Force.com_Multitenancy_WP_101508.pdf); vendor primary but dated 2008, principle unchanged per [Salesforce Architects](https://architect.salesforce.com/docs/architect/fundamentals/guide/platform-multitenant-architecture.html), which 403'd on fetch). Creating a custom field runs no DDL — a metadata row is inserted and the value lands in a pre-existing generic slot column ([Cirra](https://cirra.ai/articles/salesforce-database-architecture-explained), [Connexxia](https://connexxia.ca/deeper-dive-into-the-salesforce-platforms-data-layer-architecture-part-2/)).

**Schema change is a data write.** Salesforce states the consequence: the platform performs online multitenant schema maintenance without blocking other orgs' concurrent activity (whitepaper). That is what makes customer-added verticals economically possible at all — and it is why governor limits exist. Shared physical resources need per-tenant quotas; the limits are the invoice for the trick, not a design flaw.

### The extension-model honesty test
`CONFIRMED`. Salesforce's own Industry Clouds are built with the customer's mechanism. Financial Services Cloud "uses new custom fields on the Account and Contact standard objects to model clients, along with new custom objects to model client financials, relationship groups, and more" ([Salesforce Help, FSC data models](https://help.salesforce.com/s/articleView?id=ind.fsc_admin_data_model.htm&type=5), [Trailhead](https://trailhead.salesforce.com/content/learn/modules/fsc_data_model/fsc_data_model_unit_1)). OmniStudio is the declarative industry layer above: FlexCards, OmniScripts, Data Mapper, Integration Procedures ([Salesforce Developers, OmniStudio overview](https://developer.salesforce.com/docs/atlas.en-us.industries_reference.meta/industries_reference/omnistudio_overview.htm)).

**The vendor building its own verticals with the customer's extension mechanism is the falsifiable test of whether an extension model is real.** Salesforce and ServiceNow pass. It is also a forcing function: if the vendor's own vertical needs a privileged path, the vendor learns exactly where the model is inadequate.

### Shared entities — the weakest, and instructive
`CONFIRMED`. No party model. Account (organisation) and Contact (person at an organisation), one-to-many. An individual as a customer requires **Person Accounts**, fusing Account and Contact fields into one conceptual record ([FSC data models](https://help.salesforce.com/s/articleView?id=ind.fsc_admin_data_model.htm&type=5)).

`CONFIRMED`, and this is the tell: **Person Accounts cannot be disabled once enabled** ([Trailblazer Community](https://trailhead.salesforce.com/trailblazer-community/feed/0D5KX00000bGFGx0AO), corroborated [Robert Setiadi](https://www.robertsetiadi.com/salesforce-tips-enabling-person-accounts-b2c-solution/)). An irreversible org-wide switch is what a retrofitted party model looks like — the platform cannot safely reason about un-merging the two shapes. `LIKELY`: 50,000 person accounts per owning user, doubleable by Support, forcing large B2C orgs to mint owner users purely as a sharding device ([release notes](https://help.salesforce.com/s/articleView?id=release-notes.rn_experiences_person_account_limits.htm&language=en_US&release=242&type=5)).

Salesforce does have party concepts — Household, `Applicant`, `Party Profile` — **per industry cloud, above the platform, not in it** (FSC data models). If you have a party model in the substrate, this is the specific thing you have that Salesforce does not.

### Approvals
`CONFIRMED`. Flow Approval Orchestrations supersede legacy Approval Processes: all objects, stages containing Approval Steps (interactive screen flows) and Background Steps (autolaunched), decision-element routing, conditional stage completion ([Salesforce Help](https://help.salesforce.com/s/articleView?id=platform.automate_automated_approvals.htm&type=5), walkthrough [Salesforce Ben, Winter '26](https://www.salesforceben.com/salesforce-spring-25-release-new-flow-approval-process-capabilities/)).

`CONFIRMED` and commercially significant: **as of Spring '26 (week of 16 February 2026) Flow Orchestration runs are included with no usage-based caps** ([Salesforce Break, Summer '26 Flow updates](https://salesforcebreak.com/2026/04/25/summer-26-flow-updates/)). Until then the general workflow engine was metered per run.

`LIKELY` remaining limits: 50 versions per orchestration; **manual approver selection mid-process unsupported — all approvers determined upfront**; external approvers need specific config; Partner Community licences face work-item visibility restrictions (Salesforce Ben). The upfront constraint is the one that bites: "route to whoever the requester's manager is at the time of the third rejection" is inexpressible — and neither is 후결.

### Org model, economics
`LIKELY`, by absence. **No legal-entity or group-consolidation concept in the platform**; org structure is role hierarchy, sharing rules, territories, or separate orgs. No general ledger — quote-to-cash exists, the book of record is elsewhere. Absence is hard to prove; verify directly if a plan depends on it.

### Failure mode — the best-quantified low-code warning available
`LIKELY`. **56.3% of Salesforce admins name technical debt their biggest 2026 challenge**; 31% call it severe enough to slow daily work; **2% describe their org as clean**. Only 18.5% say executives clearly understand the consequences; 10.9% say they do not understand the risks at all ([Salesforce Ben, State of the Salesforce Admin Role in 2026](https://www.salesforceben.com/the-state-of-the-salesforce-admin-role-in-2026/)).

The mechanism: "low-code tools that were supposed to reduce complexity have… created a different kind of complexity that's harder to see and harder to fix" (same source; developed in [Digital Mass](https://digitalmass.com/how-we-think/salesforce-flow-technical-debt-automation-sprawl-2026/)). No-code does not remove debt, it removes the *tooling* for managing it — no diff, no test suite, no review, no owner. Directly relevant to INDEX steal #3 and #8: every no-code capability shipped needs a governance surface shipped with it.

---

## 6. ServiceNow — corpus has features, not the substrate economics

### Substrate and the convention layer
`CONFIRMED`. `EXTENDS lenses/data-model.md:159` ("tables, extend base"). Above the table hierarchy sits **CSDM** — "a capability built into the Now Platform, the single cloud application platform that all ServiceNow products run on", a "standard and shared set of service-related definitions across ServiceNow products and platform" plus prescriptive modelling guidance ([ServiceNow, Common Services Data Model](https://www.servicenow.com/platform/common-services-data-model.html), [CSDM solution brief PDF](https://www.servicenow.com/content/dam/servicenow-assets/public/en-us/doc-type/resource-center/solution-brief/sbr-servicenow-common-service-data-model.pdf)).

CSDM is not a schema — it is **a published opinion about how to use a schema that permits anything**. That is what a platform must ship when its substrate is too flexible to be coherent alone. A registry of 27 seeded types has the same problem: without a prescribed model, every tenant invents a different one and breadth becomes the customer's integration problem.

### The extension mechanism is metered — anti-pattern
`CONFIRMED`, and directly relevant to INDEX steal #3. A custom table is "any non-ServiceNow provided table created or installed by or on behalf of Customer on the ServiceNow platform". Non-production creates freely; **production requires purchased custom-table entitlement.** Certain OOTB tables are exempt and extendable — `cmdb_*`, `cmn_location`, `cmn_schedule_condition`, `kb_knowledge`, `sc_cat_item_delivery_task`, `sys_choice`, `sys_dictionary`, `sys_filter`, `sysauto`, `syslog` — and **each exempt table may be extended up to 1,000 times**; beyond that needs App Engine or a product carrying App Engine Starter ([ServiceNow Community architect article](https://www.servicenow.com/community/architect-articles/custom-table-avoid-subscription-consumption/ta-p/2330639), corroborated [custom application licensing thread](https://www.servicenow.com/community/developer-forum/new-custom-application-licensing-interaction-with-custom-table/m-p/1752326)). Custom apps license either via a Creator licence or as an app-based subscription with its own metric ([Admodum](https://admodumcompliance.com/blog/servicenow/custom-app-licensing)).

The documented consequence is the lesson: the recommended technique in that architect article is to **reuse exempt tables — particularly `sys_choice` extensions — with reference qualifiers instead of creating proper tables**, purely to avoid consuming entitlement. **Commercial metering of the data model deforms the data model.** If a type registry is ever monetised per type, this is what customers will do to it.

### Shared entities — two person records at the best-case vendor
`CONFIRMED`. `sys_user` is the one user table across every product. But HRSD requires **two** profiles: the User Profile (`sys_user`, needed for platform access) and a separate **HR Profile** populated from the personnel system ([ServiceNow Community, HRSD basics](https://www.servicenow.com/community/hrsd-articles/hr-service-delivery-basics-overview/ta-p/2311268); reconciliation failure when created out of order: [HRSD forum 1336815](https://www.servicenow.com/community/hrsd-forum/how-to-handle-a-user-and-hr-profile-created-during-on-boarding/m-p/1336815)).

The vendor with the most genuinely unified substrate in the survey still has two records for a person, and a documented ordering bug between them. This is the shared-party failure in the wild, at the best case.

### Workflow, and 2026 direction
`LIKELY`. Flow Designer is one engine across ITSM, ITOM, HRSD, CSM, SecOps and custom apps, with reusable Actions and Subflows ([ServiceNow, Flow Designer](https://www.servicenow.com/products/platform-flow-designer.html), cross-module use per [NowBen](https://nowben.com/a-beginners-guide-to-flow-designer-in-servicenow/)) — while the legacy Workflow engine persists alongside it ([Flow vs Workflow](https://www.servicenow.com/community/workflow-data-fabric-forum/some-major-difference-between-flow-and-workflow/m-p/3507335)). `UNKNOWN`: how delegated multi-step conditional approval is expressed declaratively versus in script. Not sourced, not guessed.

`LIKELY`. 2026 is **Workflow Data Fabric**: zero-copy data fabric tables connecting directly to external sources without ETL, plus Context Engine and Autonomous Data Governance ([ServiceNow](https://www.servicenow.com/platform/workflow-data-fabric.html), [newsroom 2026](https://newsroom.servicenow.com/press-releases/details/2026/ServiceNow-launches-the-real-time-data-foundation-that-puts-autonomous-AI-to-work-across-the-enterprise/default.aspx)). Strategically an admission that not all enterprise data will enter the CMDB, so the substrate must **reference** what it does not own.

### The platform play that worked
`LIKELY`. Technology workflows — ITSM, ITOM, ITAM, SecOps, the original business — were **47% of 2025 revenue**, so non-IT is now the majority ([Cyntexa aggregation of vendor disclosures](https://cyntexa.com/blog/servicenow-statistics/)). `CONFIRMED` from the filing: Q2 2026 subscription revenue $3,877m, +24.5% YoY (23% cc); 658 customers over $5m ACV, ~23% YoY; 123 transactions over $1m net-new ACV, +40% YoY; ServiceNow AI past $1bn ACV ([Q2 2026 results](https://newsroom.servicenow.com/press-releases/details/2026/ServiceNow-Reports-Second-Quarter-2026-Financial-Results/default.aspx)). `LIKELY` on cross-sell: EmployeeWorks deal volume +150% QoQ; CRM and Industry Workflows in 16 of the top 20 deals; CRM a $2bn ACV business ([Futurum](https://futurumgroup.com/insights/servicenow-q2-fy-2026-ai-security-and-workflow-expansion-fuel-growth/)).

ServiceNow entered HR and CSM against better-featured incumbents and won on substrate — same tables, same workflow engine, one user record. **The substrate was the product.** It beat better modules. That is the strongest external evidence that an ontology-first bet can win a module it does not yet lead on features.

### Failure mode
`UNCERTAIN` on specifics, `LIKELY` on pattern: upgrades break customised workflows; customisation makes each cycle costlier; organisations defer upgrades and fall behind the release schedule ([Dyna Software](https://dynasoftwareinc.com/why-technical-debt-has-become-the-silent-growth-killer-in-servicenow-environments/), [Iconica](https://www.iconica.co/post/servicenow-technical-debt)). One quantified case, **single-vendor-sourced, illustrative anecdote only**: 127 custom workflows over 18 months, +$95,000/year maintenance, seven-month Xanadu migration delay (Dyna Software).

The better formulation, `LIKELY`: the indirect cost is "platform upgrades delayed because the estate is too fragile to move, and new module deployments scoped smaller than planned because nobody is confident the foundation will hold" ([Iconica](https://www.iconica.co/post/servicenow-technical-debt)). **Breadth dies from declining confidence in the substrate, not from a failure.**

---

## 7. Zoho One — absent from corpus

`UNCERTAIN` claim, and I believe it is false: "All Zoho One apps share a single unified database… a contact record in Zoho CRM is not copied to Zoho Books — it is the same record" ([Codroid IT Labs](https://codroiditlabs.com/zoho-one-app-integration/)). Partner blog, no technical detail.

`CONFIRMED`, vendor docs say the opposite. Books and CRM are joined by a configurable **sync**: "new accounts, contacts, vendors, or products added in Zoho CRM will sync into Zoho Books **every two hours** after the initial sync, while transaction modules sync instantly." You choose a sync type (Contacts only, or Accounts & their Contacts, with a checkbox to include contacts with no account, which arrive as Customers with Customer Type = Individual). Contacts created via API "are treated differently in the sync" from UI-created ones ([Zoho Books help, CRM integration](https://www.zoho.com/us/books/help/integrations/crm-integration.html), [Books API v3](https://www.zoho.com/books/api/v3/integration/)). Two-hour polling, user-selected mapping policy, UI-vs-API divergence, and forums of "force sync" threads ([force sync](https://help.zoho.com/portal/en/community/topic/force-sync-contact-account-from-crm-to-books), [sync issues](https://help.zoho.com/portal/en/community/topic/sync-issues-between-zoho-books-and-crm)) are two records reconciled, not one shared.

`LIKELY`. What Zoho genuinely shares is real and worth naming precisely: **one subscription, one admin console with cross-app role permissions, one AI assistant, cross-app analytics, and Blueprint workflows spanning apps** ([Zoho One what's new](https://www.zoho.com/one/whats-new.html), [SiliconANGLE, ZohoDay 2026](https://siliconangle.com/2026/02/21/hitting-stride-appos-ai-low-code-automation-zohoday-2026/)); 50+ apps ([CRM-Masters](https://crm-masters.com/zoho-one-applications/)). A legitimate architecture — identity, permissions, analytics, automation shared; data model synchronised — just not the advertised one.

`LIKELY`, candid for a partner review: "integration gaps require configuration that doesn't happen automatically, the learning curve is steeper than marketed, and some apps are capped below their standalone counterparts' top tiers" ([Ravenlabs, tested all 45 apps](https://www.theravenlabs.com/zoho-one-review-2026-tested-all-45-apps-heres-what-actually-works/)).

---

## 8. Korea — corpus has 결재선 only; ERP substrate is a real gap

The corpus covers Korean **approval semantics** well (`lenses/data-model.md:328,342`; `lenses/ia-layout.md:183`; `appr.md:205`). It does not cover any Korean vendor as an **all-in-one substrate**. Sourcing here is materially weaker than for Western vendors: Korean vendors publish little architectural documentation in any language. Directional only.

### Douzone — WEHAGO / Amaranth 10 (closest Korean analogue)
`LIKELY`. WEHAGO is a cloud/big-data business platform integrating what a company needs to operate: ERP/management functions plus messenger, video conferencing, mail, calendar, **전자결재**, boards, web office and cloud storage in one platform ([Douzone WEHAGO](https://www.douzone.com/product/wehago.jsp), [groupware](https://www.douzone.com/product/groupware.jsp)). Baseline coverage: accounting, HR/payroll, logistics/inventory, purchasing/sales, tax, 전자세금계산서, closing/adjustment ([Korea-ERP review](https://korea-erp.com/wehago/)).

`LIKELY`, architecturally interesting: **Amaranth 10 is described as the first solution providing cloud-native Kubernetes-based architecture simultaneously in both SaaS and on-premise package form**, integrating ERP with groupware, office and document centralisation ([Douzone Amaranth 10](https://www.douzone.com/product/amaranth10.jsp), [FKII Digital 365 Vol.13, March 2022](https://www.fkii.org/webzine/FKII_2203/FKII_sub33.php), [F Today, 2022](http://www.ftoday.co.kr/news/articleView.html?idxno=242754)). Note the 2022 dating — not a 2026 announcement.

`LIKELY`. Verticalisation is **by edition, not by extension**: WEHAGO for SMEs, WEHAGO T for tax agents, WEHAGO H for healthcare, WEHAGO V for government; up-market iCube (mid-market) and ERP 10 (enterprise) ([WEHAGO V](https://www.douzone.com/product/wehagov.jsp), [Korea-ERP comparison 2025](https://korea-erp.com/douzone-program/)). Core solutions integrated with AWS for a global SaaS platform effort ([ETNews, 2023-10-16](https://www.etnews.com/20231016000192)).

`UNKNOWN`, and these are exactly the questions that matter: whether WEHAGO/Amaranth has one shared party model across ERP and groupware; whether 전자결재 is one engine used by all modules or per-module approval; whether any customer-facing extension model exists at all. No technical documentation found addressing any of them. **Do not assume — the domestic incumbent's substrate is unknown, and if it is per-module approval over a federated model, that is the competitive opening.** Needs primary research: Korean-language technical docs or Douzone partner conversations.

### NHN Dooray!
`LIKELY`. All-in-one collaboration built on project collaboration, providing messenger, mail and **전자결재** as one product, plus video conferencing, calendar, drive, co-editing, translation ([Dooray](https://dooray.com/main/en/), [Namuwiki](https://namu.wiki/w/NHN%20Dooray!), [NHN Cloud](https://www.nhncloud.com/jp/service/collaboration/dooray)); separate government edition ([gov-dooray](https://gov-dooray.com/main/), [NHN Cloud public-sector docs](https://docs.gov-nhncloud.com/ko/Collaboration/Dooray/ko/overview/)). `UNKNOWN`: data model, extension model, pricing figures.

### Naver Works / Kakao
`LIKELY`. Naver Works is messaging, mail on your own domain, drive, calendar — positioned as cheaper than SMEs building groupware themselves, priced per account plus domain ([Namuwiki](https://en.namu.wiki/w/%EB%84%A4%EC%9D%B4%EB%B2%84%EC%9B%8D%EC%8A%A4)). Kakao's Agit is free to 30 people with functional limits ([Seoulstart, 2026](https://seoulstart.com/guides/naver-vs-kakao)). **These are collaboration suites, not business platforms** — no ERP, no ledger, no party model. `lenses/data-model.md:328` correctly cites Naver Works for 결재선 semantics; do not extrapolate that to substrate comparison.

### Ecount — the pricing outlier
`LIKELY`. `EXTENDS dashboard.md` (single passing mention). Korean cloud ERP, founded 1999, offices across SE Asia; all-in-one inventory/sales/purchasing/accounting at **a flat $55/month or $600/year for the whole company, unlimited users and IDs, all features, no additional cost for implementation, upgrades or maintenance** ([Ecount pricing policy](https://www.ecount.com/us/ecount/join/pricing), [SoftwareSuggest](https://www.softwaresuggest.com/ecount-erp), [SelectHub](https://www.selecthub.com/p/erp-software/ecount/), [Capterra](https://www.capterra.com/p/130047/Ecount-ERP/)).

Against Workday at $720k–960k/year for 1,000 employees, this is the most aggressive possible answer to occasional-user pricing: stop counting. `UNKNOWN` what it costs Ecount in per-tenant capability, extension model or scale ceiling — that trade is the interesting part and I could not source it. It also sets the domestic SME price floor, which constrains any Korean-market pricing plan.

---

## 9. The flexible-substrate end — monday, Airtable, Notion

The honest inverse of the ERP vendors: no domain model, so no coherence problem — and no answer to economics, legal entities, or the ledger. Corpus mentions are single feature references (`automate.md`, `leave.md`, `lenses/ia-layout.md`), not substrate.

`CONFIRMED`. **mondayDB** is **schemaless**, storage and compute separated ([monday.com](https://monday.com/w/mondaydb), [monday engineering](https://engineering.monday.com/nice-to-meet-you-mondaydb-architecture/)); the whole suite — work platform, CRM, service, dev — runs on it as "a single source of truth" (`LIKELY`, [CXLABS](https://www.cxlabs.digital/blog/from-work-os-to-ai-work-platform-how-monday-com-is-leading-the-ai-revolution)).

`LIKELY`, and useful as scale calibration for schemaless substrates: the DB 3.1 "Products at Scale" roadmap for 2026 H1 targets **1M items in a data set, 5M items in a report, 5k projects in a portfolio** — all live-beta ([Fruition Services, 2026](https://www.fruitionservices.io/post/what-is-monday-db-for-enterprises)). Those are the ceilings a generic-item substrate reaches: two to three orders of magnitude below a relational ERP in one table. **Flexibility is paid for in scale.** Airtable claims far higher — HyperDB "scales up to hundreds of millions of records in a single table" (`LIKELY`, [Airtable](https://www.airtable.com/platform/app-building)), alongside Omni generating tables, interfaces and automations from a description.

`LIKELY`, and the strategic item: on **13 May 2026** Notion shipped a developer platform with Workers, **database sync** and agent APIs, positioning as an orchestration layer rather than a system of record — pulling operational data from Salesforce, Zendesk and Postgres into Notion databases without one-off integrations ([Notion releases 2026-05-13](https://www.notion.com/releases/2026-05-13), [TechCrunch](https://techcrunch.com/2026/05/13/notion-just-turned-its-workspace-into-a-hub-for-ai-agents/), [InfoWorld](https://www.infoworld.com/article/4171166/notion-courts-developers-with-platform-for-ai-agents-and-workflow-automation/)).

Note the convergence: Notion's database sync and ServiceNow's zero-copy data fabric are the same 2026 bet from opposite ends of the market — the substrate should **reference** records it does not own. Two vendors with no reason to agree, agreeing.

---

## 10. Packaging a vertical for reuse — cross-vendor mechanics

Not covered anywhere in the corpus. Four mechanisms, and the differences are instructive.

**Salesforce 2GP.** `CONFIRMED`. Source-driven development, package versioning, CI/CD. A namespace is a 1–15 char identifier fixed at package creation that "can't be changed" — but unlike 1GP, **multiple packages can share one namespace**. A Dev Hub org owns every package; version skipping, rollback and branching supported ([2GP Developer Guide](https://developer.salesforce.com/docs/atlas.en-us.pkg2_dev.meta/pkg2_dev/sfdx_dev_dev2gp.htm), [namespaces](https://developer.salesforce.com/docs/atlas.en-us.pkg2_dev.meta/pkg2_dev/sfdx_dev_dev2gp_plan_namespaces.htm)). Namespace-per-package was the 1GP constraint making a vertical an all-or-nothing monolith; sharing it lets a vertical be decomposed and released in pieces.

**ServiceNow Store.** `CONFIRMED`. Develop and test on a Build Partner vendor instance, upload to the Store via the Publisher Portal, request certification, then price and publish. Certification is "a rigorous vetting process" for stability and architectural soundness, hands-on with continuous contact with the certification team. Publishers must **stay current within n-3 of ServiceNow releases** ([uploading your application](https://www.servicenow.com/community/app-publisher-blog/uploading-your-application-to-the-store-publisher-portal/ba-p/2477377), [certification guide](https://www.servicenow.com/community/app-publisher-blog/guide-to-getting-your-app-certified-and-certification/ba-p/2477630), [KB0813336](https://support.servicenow.com/kb?id=kb_article_view&sysparm_article=KB0813336)). **Mandatory version currency for publishers caps how far the ecosystem can lag the platform** — which is what preserves the platform's freedom to change.

**Odoo Apps.** §2 above — manifest-declared price, version pinned to platform major, author owns bug fixes, and not installable on the cheapest hosting tier.

**NetSuite SuiteApps.** §3 above — the only one that can package **GL-impacting dimensions** with predefined values.

---

## 11. Pricing under breadth

Not covered in the corpus. Five patterns, four of which work.

`CONFIRMED` mechanism, `LIKELY` numbers. **Dynamics base + attach**: the first application for a named user must be the highest-priced (base); further apps are "attach" licences, "identical in their core capabilities and only differentiated in price" ([Dynamics 365 Licensing Guide, March 2026](https://cdn-dynmedia-1.microsoft.com/is/content/microsoftcorp/microsoft/bade/documents/products-and-services/en-us/bizapps/Dynamics-365-Licensing-Guide-March-2026-PUB.pdf)). Attach at ~20–50% of base; the guide's worked example is Business Central Premium $100 base with Customer Service / Field Service / Sales Enterprise attached at $20. Plus a **Team Member licence at ~$8/user/month with read access to data from any D365 app and limited write on specific entities** ([Redress](https://redresscompliance.com/dynamics-365-licensing), [Microsoft Negotiations](https://microsoftnegotiations.com/white-papers/dynamics-365-licensing-guide)). Three tiers of engagement — deep, second-app, read-mostly — is the right shape for a platform most employees touch rarely.

`LIKELY`. **Zoho charges for the option to exclude people**: All Employee $37/user/month annually **requiring every person on payroll be licensed with no exceptions**, versus Flexible User $90 for any subset — a ~143% premium for a small denominator ([Zenatta](https://zenatta.com/zoho-pricing-guide-2025/), [HouseBlend](https://www.houseblend.io/articles/zoho-one-pricing-models-explained), [Capterra](https://www.capterra.com/p/166175/Zoho-One/pricing/)).

`LIKELY`. **Odoo: breadth free, depth charged.** Every paid seat gets every app; two tiers (Standard, Custom), Custom ~50–70% more and adding Studio, multi-company, external API, and self/Odoo.sh hosting. Regional pricing varies enormously — the same Standard plan reported from $8.95 to $76.20/user/month by country ([ERP Research](https://www.erpresearch.com/pricing/odoo), [OEC.sh 179-country comparison](https://oec.sh/odoo-pricing)). Odoo does not price modules; it prices **platform capabilities**.

`LIKELY`. **Ecount: stop counting** — flat $600/year, unlimited users (§8).

`LIKELY`, and the anti-pattern: **monday sells CRM / service / dev as separate subscriptions, each with its own 3-seat minimum.** Work Management and dev from $9/seat/month annually, CRM from $12, service from $31; "if you want both Work Management and CRM, you're paying for two subscriptions, and each product requires its own 3-seat minimum" ([monday plans and pricing](https://support.monday.com/hc/en-us/articles/4405633151634-Plans-and-pricing-for-monday-com), [UseCarly](https://www.usecarly.com/blog/monday-pricing/), [Vendr](https://www.vendr.com/marketplace/monday-com)). One substrate, re-fragmented at the invoice.

Reference points for scale: Salesforce Enterprise from $165/user/month, Unlimited $330; Agentforce add-ons $125–150; Agentforce 1 Editions from $550; Flex Credits $500/100k credits at ~20 credits/action (`LIKELY`, [Enterprise Dreamin'](https://enterprisedreamin.org/articles/agentforce-pricing-explained-2026/), [MagicFuse](https://magicfuse.co/blog/agentforce-cost)). Workday `UNCERTAIN`: core HCM ~$34–42 PEPM at scale, $55–150 full-suite; 1,000 employees $720k–960k/year for HCM+Financials ([ERP Research](https://www.erpresearch.com/pricing/workday), [VendorBenchmark](https://vendorbenchmark.com/blog/workday-pricing-benchmark-per-employee)).

---

## 12. What consistently fails

**Customisation survives no upgrade, in every architecture.** Odoo 18 modules do not run on 19; Workday's mandatory biannual updates break Extend apps via changed APIs, deprecated objects and altered business-process behaviour ([WorkdayNegotiations](https://workdaynegotiations.com/blog/workday-extend-vs-custom-development/)); ServiceNow customers defer upgrades because the estate is too fragile. Metadata-driven, object-graph and source-module architectures all fail here identically. Only two mitigations found anywhere: SAP's tiered released-API-only doctrine with forward certification (§13.6), and ServiceNow's n-3 publisher currency rule.

**Low-code creates invisible debt, not less debt.** §5. The only quantified data in the survey, and it is damning.

**Implementation cost and duration overwhelm licence cost.** Panorama's 2026 study, compiled September 2025–January 2026 across 2,400+ discrete-manufacturing ERP implementations: **215% average budget overrun** in discrete manufacturing, 26 points above the cross-industry average, and **73% failing to meet objectives** ([Panorama Consulting](https://www.panorama-consulting.com/panorama-consulting-group-releases-latest-study-of-erp-implementation-outcomes-across-the-globe/), summarised in [Godlan](https://godlan.com/erp-implementation-failure-statistics/)) — worst-case segment, not the average. Workday implementation runs 100–200% of annual subscription; NetSuite $25k–750k.

**Consultant dependency is a function of ecosystem size, and small ecosystems are worse.** Workday Extend expertise "is concentrated in a small number of consultancies and individuals, which drives up both hiring costs and implementation partner rates and creates project delivery risk", explicitly contrasted with thousands of certified Salesforce and ServiceNow developers ([WorkdayNegotiations](https://workdaynegotiations.com/blog/workday-extend-vs-custom-development/)). **A better extension model with a smaller talent pool is worse in practice than a worse one with a large pool** — relevant to any plan whose extension surface is novel.

**Breadth loses at scale, wins below it.** The cleanest quantification found, from higher education but structurally general: among institutions under 2,500 students only **15%** go multi-vendor; above 30,000 students, **59%** do ([ListEdTech](https://listedtech.com/blog/best-of-breed-versus-best-of-suite-the-rise-of-decoupled-ecosystems/)). Stated cause: APIs and cloud-native integration dissolved the technical barrier that used to force single-vendor compromise, and large organisations stopped tolerating "mediocre functionality in a critical department simply because it happens to come bundled". Implication, uncomfortable and specific: **the addressable market is organisations below the scale at which integrating best-of-breed becomes cheaper than tolerating the weakest module.**

**The module the vendor deprioritises becomes the reason customers leave.** "Businesses may be very dependent on a critical part of a software suite that the vendor has decided not to prioritise" (ListEdTech); Zoho's own reviewers note apps "capped below their standalone counterparts' top tiers" (Ravenlabs). Breadth is a promise to maintain N modules forever; the weakest sets the churn rate.

**You cannot acquire your way into a single object model.** `EXTENDS lenses/data-model.md:107` — the corpus treats Workday as the effective-dating reference; the strategic counterweight belongs with it. Analysts pressed Workday to reconcile "Power of One" with acquisitions (Adaptive Insights, Scout RFP, Zimit) that were not fully integrated; **Workday changed the claim, shifting to "Power to Adapt"** rather than re-model them ([diginomica](https://diginomica.com/workday-exit-power-one-enter-power-adapt)). Every Type A platform faces the same fork on every acquisition: multi-year re-modelling, or a permanent exception to the coherence story.

**Two engines for one job is the standard end state.** Dynamics has F&O workflow *and* Power Automate; Salesforce legacy Approval Processes *and* Flow Approvals; ServiceNow Workflow *and* Flow Designer. In each case the second exists because the first could not span the platform, and the first cannot be removed. If you build one governed Action verb (INDEX steal #8), the thing to defend is not its feature set but its **universality** — the moment one module needs its own, you have two forever.

---

## 13. Design decisions worth stealing — delta only

Ranked by leverage per unit of implementation cost. Items already in the corpus steal-list are not repeated.

1. **Attach approval to transitions, not documents.** Odoo gates buttons. `EXTENDS people.md:204` — the corpus wants one reusable `BusinessProcessDefinition` for HR events; button-level attachment is the cheaper generalisation that covers *every* module without per-module enrolment, because every module has actions. Also the natural carrier for 결재선 as an ordered link-set (`lenses/data-model.md:342`).

2. **One identity, role-scoped attributes, explicit divergence list.** NetSuite, including the discipline of naming the two fields that must *not* propagate. `EXTENDS lenses/data-model.md:83`. Neither one flat record nor one per role. Directly applicable to a party/person type that must be employee, customer, vendor and 거래처 contact at once.

3. **Delegation as a time-boxed, self-service authority transfer.** `EXTENDS appr.md:64`. The corpus has Rippling's auto re-route on vacation as closest-to-대결 and SAP's substitution rules. Workday adds the missing shape: the approver themselves routes inbox tasks to a named substitute **for a defined period** ([UVA Finance delegation QRG, updated 2023-09-25](https://uvafinance.virginia.edu/sites/uvafinance/files/2023-09/UVAFST_QRG_DelegateWorkdayTransactions_Final_R2%20(1).pdf)). Compare Odoo (all records, no time bound) and Salesforce (approvers fixed upfront, so 후결 is inexpressible). Time-boxing is the part everyone skips and the part 대결 needs.

4. **Purpose-typed hierarchies over one organisation record set.** Dynamics F&O: each hierarchy declares a purpose constraining valid member types and consuming scenarios. Workday's analogue: workers sit in many org types but are hired into exactly one Supervisory Org, and **approval authority derives from which hierarchy the transaction belongs to** — Supervisory Org managers approve HR transactions, Cost Center managers approve financial ones ([Workday Navigator](https://workdaynavigator.com/blog/organization-types-in-workday/)). `EXTENDS compliance.md:248`: this is the mechanism that makes a 그룹 obligation cascade to 계열사 without inventing a second org model — statutory consolidation and 결재 authority as two purpose-typed trees over the same 법인 records.

5. **One ledger, business object as a multi-dimensional distribution — and make the dimensions packageable.** Odoo analytic plans + NetSuite custom segments. `EXTENDS dashboard.md:80-83`: the corpus has SAP CO/PA as the margin reference and 계약 수익성 as omitted-until-backed; these two vendors reach the same outcome without SAP's weight, and NetSuite shows the dimension itself shipping inside a package.

6. **Tier extensions by upgrade safety and certify forward.** SAP clean core: extensions must use only officially released extension points, in a three-type model — on-stack key-user (1), on-stack developer (2), side-by-side on BTP (3), with type 3 preferred where loose coupling suits ([SAP News Center, August 2025](https://news.sap.com/2025/08/extend-sap-s4hana-cloud-right-way-clean-clear/), [SAP Community ABAP extensibility guide](https://community.sap.com/t5/technology-blog-posts-by-sap/abap-extensibility-guide-clean-core-for-sap-s-4hana-cloud-august-2025/ba-p/14175399)). `UNCERTAIN`, partner-sourced but consistent: a 2026 A–D maturity model and a **Clean Core Certification Programme announced at Sapphire 2026** certifying BTP extensions compatible across **a minimum of three consecutive release cycles** ([ERP Implementation EU](https://www.erpimplementation.eu/en/sap-clean-core-strategy-btp-extensions-s4hana-2026/), [SAVIC](https://www.savictech.com/insights/sap-clean-core-strategy-2026/), [KPS](https://kps.com/insights/blog/2026/sap-clean-core/)). Three transferable ideas: name the tiers and rank them by upgrade safety; certify against *future* releases so upgrade risk becomes a vendor commitment; make "released API only" a **binary status per extension** so upgrade impact is computable rather than discoverable.

7. **Build your own verticals with the customer's extension mechanism.** §5. Simultaneously the honesty test for an extension model and a forcing function that tells the vendor where it is inadequate.

8. **Ship a prescribed model on top of the flexible one.** ServiceNow CSDM. A registry of 27 types permits anything; without a published opinion on how to use it, every tenant diverges and breadth becomes the customer's integration problem.

9. **Require marketplace publishers to stay current.** ServiceNow n-3 (§10).

10. **Namespaces shared across packages.** Salesforce 2GP (§10) — lets a vertical be decomposed instead of shipped monolithically.

11. **Reference data you do not own.** ServiceNow zero-copy fabric + Notion database sync (§9). Two vendors with no reason to agree, agreeing in 2026.

**Two anti-patterns, stated as such:**

- **Do not meter the data model.** ServiceNow's custom-table entitlements produce a documented practice of abusing `sys_choice` extensions with reference qualifiers to dodge licence consumption (§6). Charging per type deforms the type system. Direct warning for any commercial model layered on a type registry.
- **Do not charge per workflow run.** Salesforce metered Flow Orchestration until Spring '26, then made runs uncapped (§5). A general engine that costs per execution will not be used generally, which defeats having one.

---

## 14. Open questions

1. **The corpus's OURS evidence base is deleted (§0b).** Before any of its self-assessments are used in a plan, the backend-cited findings need re-verification against `backend/` and the frontend-cited ones need re-derivation or retirement. This outranks everything below.

2. **Is a single party record actually right, or is NetSuite's "one identity + role-scoped attributes + divergence list" the real answer?** Every vendor claiming one party record has exceptions (NetSuite two named fields; ServiceNow a separate HR Profile; Salesforce an irreversible Account/Contact fusion). `UNKNOWN`: whether anyone has published a principled account of which attributes must be role-scoped. This looks unsolved, not solved-badly.

3. **Douzone's substrate is unknown and it is the domestic incumbent (§8).** Whether WEHAGO/Amaranth shares a party model across ERP and groupware, and whether 전자결재 is one engine or per-module, is unsourced. Needs Korean-language primary research. If it is per-module approval over a federated model, that is the competitive opening — and if it is not, that changes the plan.

4. **What does Ecount give up for $600/year unlimited seats?** `UNKNOWN`, and it sets the Korean SME price floor.

5. **Can no-code approvals express real policy?** Salesforce cannot select approvers mid-process; Odoo delegation is all-records; ServiceNow's declarative delegation depth is unsourced. `UNKNOWN`: whether any platform lets a business configure "escalate to the requester's manager's manager after 48 hours unless the requester is themselves a cost-centre owner" without code. 후결 is a live instance of this.

6. **How does "one ledger, many dimensions" behave at volume?** Odoo analytic plans and NetSuite custom segments are architecturally right; `UNKNOWN` what happens to posting and reporting performance with high-cardinality dimensions and percentage distributions across millions of lines. No vendor publishes it, no benchmark surfaced.

7. **Dynamics project profitability across the Dataverse/F&O boundary.** `UNKNOWN` (§4).

8. **Salesforce and legal entities.** `LIKELY` absent, inferred from absence of documentation. Verify directly if a plan depends on it.

9. **Do published no-code claims survive contact with a deployment?** The only quantified counter-evidence found measures *debt*, not capability ceiling. `UNKNOWN`: what fraction of a vertical a business user can actually build without a developer or certified partner. Every source on this question has a commercial interest in the answer.
