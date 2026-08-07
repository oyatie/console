> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

# Palantir Foundry — delta and disagreements over `docs/program/benchmark-matrix/`

**Status: RESEARCH — sourced, confidence-labelled**
Researched 2026-07-29. External research only. Read: `docs/program/benchmark-matrix/**` (read-only) to avoid duplicating it. No backend source read, no build commands.

This replaces a standalone 557-line Foundry reference. The benchmark corpus already describes Foundry accurately across most of its capability matrix, and a second parallel description would drift from the first. What remains is: **pointers**, **additions**, **disagreements**.

Confidence labels: `CONFIRMED` = official Palantir page, quoted or closely paraphrased + URL · `LIKELY` = credible sources agree / strongly implied · `UNCERTAIN` = conflicting or thin · `UNKNOWN` = could not establish.

Method ceiling, stated so nothing is built on a false floor: pages were fetched from `palantir.com/docs/foundry/*` and summarised by an extraction pass. Quoted text was returned as a quotation by that pass — accurate in substance, not verified character-by-character. Public Foundry docs describe the *current* release and carry no systematic version history, so "when did this change" is usually `UNKNOWN`. Several pages 404'd on the URLs tried (`/object-link-types/datasources`, `/ontologies/scenarios-overview`, `/data-lineage/explore-artifacts-and-ontology-entities`); facts depending only on those are `UNKNOWN`, not inferred. `community.palantir.com` is Palantir-hosted but user-written — treated as secondary.

---

## 1. Already covered by the corpus — pointers only

Checked against my sources and **found accurate**. Do not re-describe these; where the corpus marks a cell `[I]` that I can now confirm, the confidence upgrade is noted.

| Concept | Corpus location | My check |
|---|---|---|
| Object type / property / PK / title key | `object-platform.md:72`, `:78` | Accurate |
| Link types 1-1 / 1-many / many-many, m-m needs join dataset | `object-platform.md:84`, `lenses/data-model.md:204` | `CONFIRMED` — and for m-m "datasources back the link types themselves" ([link-types](https://www.palantir.com/docs/foundry/object-link-types/link-types-overview)) |
| Action type = parameters + submission criteria + rules + side effects → writeback | `object-platform.md:102` | `CONFIRMED` ([actions](https://www.palantir.com/docs/foundry/action-types/overview)) |
| Actions-only is a *configurable lockdown*, not an absolute mandate | `object-platform.md:102`, `:258` (adjudication), `:161` | `CONFIRMED` — the corpus adjudication is right. See §3-A for the path it still misses |
| Object statuses + lifecycle via criteria/functions; no mandated FSM | `object-platform.md:108` | `CONFIRMED` ([statuses](https://www.palantir.com/docs/foundry/object-link-types/metadata-statuses)) |
| Proposals = PRs: branch → merge-check → reviewer → merge; protection forces branch+proposal; changelog | `object-platform.md:120`, `lenses/data-model.md:204`, `:227` | `CONFIRMED` ([branching](https://www.palantir.com/docs/foundry/foundry-branching/core-concepts)). One documented limit missing — §2-E |
| Object policy (row → instance hidden) + property policy (field → null) = cell-level, independent of dataset perms | `object-platform.md:126` — marked `[I]` | **Upgrade `[I]` → `CONFIRMED`.** Property policies "are identical to object security policies, except they only apply to a selection of properties"; for cell-level "the user must pass both"; denial yields null ([object-security-policies](https://www.palantir.com/docs/foundry/object-permissioning/object-security-policies)) |
| Automate: condition over object sets → effect (action/function/notify/webhook), same verbs humans use | `object-platform.md:132`, `lenses/governance.md:117` | `CONFIRMED` — effects are "Submit Foundry actions, Trigger AIP Logic functions, Execute Foundry functions, Send platform and email notifications" ([Automate](https://www.palantir.com/docs/foundry/automate/overview)) |
| Granular policy = operators over user attributes / columns / values; restricted views row-level | `policy.md:41`, `:49`, `:119` | `CONFIRMED` ([restricted-views](https://www.palantir.com/docs/foundry/security/restricted-views)) |
| Mandatory control property = required-marking **row/datasource** gate, *not* a per-field classifier | `policy.md:63`, `:143`, `:160` (adjudication) | **`CONFIRMED` — the corpus adjudication is exactly right.** "A mandatory control property secures all other properties in the same datasource"; base type is Mandatory Control, supporting markings, organizations or classifications; "must be mapped to a marking column on a restricted view" ([mandatory-control-properties](https://www.palantir.com/docs/foundry/object-link-types/mandatory-control-properties)). This contradicts `lenses/governance.md:34` — §3-B |
| Submission criteria = conditions over object/user/param; no approval-line concept; sequencing via chained action types | `appr.md:55`, `:106` | `CONFIRMED`. Terminology: criteria were "formerly known as validations" ([submission-criteria](https://www.palantir.com/docs/foundry/action-types/submission-criteria)) |
| Workshop = declarative widgets + typed variables + event bindings + actions; custom widgets as escape hatch | `object-platform.md:138`, `overview.md:59` | Accurate |
| Purpose-based access | `lenses/governance.md:34`, `:103` | `LIKELY` ([blog](https://blog.palantir.com/purpose-based-access-controls-at-palantir-f419faa400b3)) |
| Base-type richness incl. geospatial + time-series | `lenses/data-model.md:204`, `:237` | `CONFIRMED`, and richer than stated — §2-F |
| AIP adds LLM over a deterministic base object grammar | `support.md:192` | `CONFIRMED` — §2-G |
| Action type defined once → available in Workshop/Automate | `lenses/task-flow.md:97`, `object-platform.md:252` | Accurate |
| Marketplace packages and versions ontology + apps | `dashboard.md:112` (passing mention only) | `CONFIRMED`, under-covered — §2-H |

---

## 2. What my sources add — the corpus is silent here

### A. The write path has a **third** route, and it bypasses Actions entirely

`object-platform.md:102` and `:258` frame it as a binary: locked to actions-only, **or** reopened to Forms / direct Object-Explorer edit / API. Both are *user/application* paths. The corpus does not mention the pipeline path.

Object types are populated by the **Object Data Funnel**, which "reads data from Foundry datasources (such as datasets, restricted views, and streaming datasources) **and** user edits (from Actions)". A transform that rewrites a backing dataset changes object property values with no Action submitted, no submission criteria evaluated, and no action-log entry. `CONFIRMED` — [object-backend/overview](https://www.palantir.com/docs/foundry/object-backend/overview).

So "Actions are the only sanctioned mutation verb" holds only for the human/app surface. Data-plane writes are a separate channel by design.

Corollary the corpus also lacks: user edits land in a **writeback dataset**, not the backing dataset — "edits are written to the writeback dataset and not the dataset backing an object type or link type. This ensures that users have access to both the original data and the edited data in their analyses." `CONFIRMED` — [allow-editing](https://www.palantir.com/docs/foundry/object-link-types/allow-editing). The object a user sees is a **merge** of pipeline output and accumulated edits — the ontology is neither a live view over datasets nor an independent store.

| Claim | Confidence | Source |
|---|---|---|
| Pipeline/datasource writes reach objects without Actions | CONFIRMED | object-backend/overview |
| Edits go to a separate writeback dataset; both versions stay available | CONFIRMED | allow-editing |
| Objects are indexed materialisations (OSv1 "Phonograph" legacy; OSv2 current, separates indexing from querying, supports streaming) | CONFIRMED | object-backend/overview |
| Reconciliation policy when a rebuilt backing dataset contradicts a user edit | UNKNOWN | — |

### B. How "who may submit this action" is expressed — precisely, and why the shape matters

`appr.md:106` and `object-platform.md:102` name submission criteria but not their grammar. The grammar is what bears on a no-code role canvas.

Criteria "allow for fine-grained control over who can run an action. Simple submission criteria can require a specific user ID or group ID and can be combined with information from parameters." Two condition templates, combined with AND/OR/NOT:

- **Current User** — user ID, "group memberships via group IDs, **or any other multipass attribute**", organization.
- **Parameter** — the values the submitter entered in *this* submission.

Operators: single-value `is`, `is not`, `matches`, `is less than`, `is greater than or equals`; multi-value `includes`, `includes any`, `is included in`, `each is`, `each is not`. Per-criterion failure messages are configurable. "Actions can only be submitted if all the submission criteria are met." Authored in Ontology Manager — no code. `CONFIRMED` — [submission-criteria](https://www.palantir.com/docs/foundry/action-types/submission-criteria).

**The caveat the corpus does not carry, worth inheriting inverted:** authorization is a *conjunction*, and the no-code surface shows only half of it. "the user submitting the action must be able to view the edited object types and link types and their datasources, and pass the submission criteria" — and "an action type's configuration does not display permission settings on affected underlying object types; the person configuring the action type must ensure that these permissions are correct." Where non-action edits are permitted, the submitter "also needs edit permissions on the writeback dataset." `CONFIRMED` — [action-types/permissions](https://www.palantir.com/docs/foundry/action-types/permissions).

**Rule vocabulary** — 12 ontology rule types: create object; modify object(s); create-or-modify; delete object(s); create link(s) (m-m); delete link; **function rule** ("can be used to reference an Ontology edit function"); plus five interface-scoped variants. Rule inputs map from a parameter, an object parameter's property, a static value, or **Current User / Current Time**. Plus notification and webhook rules. Primary keys cannot be edited via actions. `CONFIRMED` — [rules](https://www.palantir.com/docs/foundry/action-types/rules), [scale-property-limits](https://www.palantir.com/docs/foundry/action-types/scale-property-limits).

**Answering the `policy.md:20` question directly.** `policy.md:20` records our canvas as fixed **Principal → Resource → Action → Effect**, `actions ∈ {view, edit, read_field, console:configure, console:deploy}`, resource attrs whitelisted to `{resource_type, owner, branch, legal_hold}`. Foundry's write-authorization model is **different in shape, not merely richer**, in four ways:

1. **The verb set is open and grows from the canvas.** Foundry has no fixed action enum. Every action type *is* a verb, created in a no-code UI. A five-element action enum has nowhere to put the verb a configurator just invented — this is the structural mismatch, and it is what blocks the no-code requirement.
2. **Authorization is co-located with the verb, not centralised.** Criteria live *on* the action type. There is no central policy document listing `(principal, action, resource)` for writes.
3. **Predicates range over the action's own parameters** — values typed at submit time. A grammar whose right-hand side is limited to resource attributes cannot express "only if the amount the submitter entered is under X", which is most of what real approval rules say.
4. **Two layers, two shapes.** Reads use object/property policies — attribute-vs-column conditions, structurally close to P→R→A→E. Writes use per-action-type criteria. Foundry does not force one grammar to serve both.

Not a difference: Foundry criteria are also *additive* to resource permissions rather than the whole decision — the same conjunction issue as (2).

### C. Per-object provenance exists and is **runtime-queryable** — via Action Log, not lineage

`object-platform.md:114` and `overview.md:272` place Foundry history in "writeback dataset transactions + Ontology Changelog + branch snapshots". `evidence.md:25` credits "immutable transactional datasets + data lineage". **Action Log appears nowhere in the corpus** — zero hits across all 20 files. It is what answers "which decision produced this edit".

The action log "models all action submissions as object types to be analyzed and displayed in object-aware Foundry tooling", answering "what changed, by whom, and when?". Default fields: action RID, action type RID, UTC timestamp, submitting Multipass user ID, primary keys of all edited objects, optionally summary, parameter values, property values. Log object types map 1:1 with action types, are prefixed `[LOG]`, and each submission produces one log object "automatically linked to all objects edited by the submitted action" — the docs' example: closing 10 Alerts yields one log object "with foreign key links to all 10 Alert objects". Function-backed action types require the edit function to have `Edits` provenance configured. `CONFIRMED` — [action-log](https://www.palantir.com/docs/foundry/action-types/action-log).

Because the log **is** an object type, decision→effect is a link traversal at runtime, not a log search. Separately, per-object **edit history** can be enabled per object type ("Track user edit history"), surfaced by a Workshop widget, with changelog records end users cannot delete or modify. `LIKELY` — [user-edit-history](https://www.palantir.com/docs/foundry/object-edits/user-edit-history), [widgets-edits-history](https://www.palantir.com/docs/foundry/workshop/widgets-edits-history).

**Lineage granularity — this decides whether Foundry's model transfers:**

| Question | Answer | Confidence | Source |
|---|---|---|---|
| Dataset lineage granularity | Resource-level (per dataset/transaction). Nodes: datasets, media sets, virtual tables, streams, transforms, schedules, artifacts, ontology entities | LIKELY | [data-lineage/overview](https://www.palantir.com/docs/foundry/data-lineage/overview) |
| A second, ontology-spanning graph exists | **Workflow Lineage** — "explore workflows to see details on objects, actions, functions, large language models, and applications" | CONFIRMED | [workflow-lineage/overview](https://www.palantir.com/docs/foundry/workflow-lineage/overview) |
| Per-row provenance inside a dataset | No mechanism found | UNKNOWN | — |
| Lineage queryable at runtime by an application | Appears not to be; a public-API request is open | LIKELY (negative) | [community request](https://community.palantir.com/t/expose-resource-lineage-as-a-public-api/6496) **[secondary]** |
| Object-level provenance queryable at runtime | **Yes** — Action Log is an object type | CONFIRMED | action-log |

Consequence for split/merge quantity genealogy: it does **not** come from lineage. In Foundry it would be modelled explicitly as object types + link types, with the Action Log supplying the causal edge. `LIKELY` by absence of any named alternative.

### D. Scenarios — a real simulate-before-commit primitive, absent from the corpus

Zero hits for "scenario" in a Foundry sense across all 20 corpus files. This is the corpus's largest silence and the closest match to a dry-run requirement.

A Scenario forks **data**, not schema: "a fork of the data in the Ontology created by applying a set of Actions and evaluating a set of Models", storing only the delta — "only the edits or changes from the base Ontology including modified Object properties, created Objects, deleted Objects, created link types, and deleted link types."

Sandbox and commit semantics: "When working within a scenario, all edits (creating objects, modifying properties, deleting objects, and adding or removing links) exist only within that scenario's isolated sandbox. These edits do not affect the main Ontology. Applying the scenario commits all those staged edits to the Ontology as a single transaction via the merge action."

The commit is itself governed — the transferable insight: "Scenarios are merged through action types that provide granular control over the scenario edits and enforce fine-grained execution permissions." The merge action declares in-scope object and link types plus its own Security & Submission Criteria, and "at minimum… requires the single **Scenario** parameter, which is the RID of the scenario". **Simulation therefore creates no permissions bypass.**

Documented caps: 30,000 edits per scenario · 50 actions · 10,000 objects loadable from an object set · attachment properties unsupported · inherits all Action limits and Function limits when function-backed. `CONFIRMED` — [scenarios-concepts](https://www.palantir.com/docs/foundry/workshop/scenarios-concepts), [merge-scenario](https://www.palantir.com/docs/foundry/ontology/merge-scenario).

**Flagged conflict, not smoothed:** one page states "A Scenario is immutable once created" (modify = create a new one with a different Action set); the merge/sandbox pages describe a mutable workspace accumulating staged edits. These may be different flavours (Vertex model-driven vs ontology/Workshop editing) or a terminology shift. `UNCERTAIN` — resolve before copying, because immutable-action-set and mutable-workspace are different products.

Distinct third mechanism: **Vertex Scenarios** — model-driven what-if over a system graph, chaining models, requiring Functions-on-models "published as actions", with **Domains** ("describes the valid set over which Model can be evaluated") required to be decomposable. A much heavier commitment. `CONFIRMED` — [vertex/scenarios-overview](https://www.palantir.com/docs/foundry/vertex/scenarios-overview).

**Three mechanisms, three layers** — the corpus treats branching as the whole story:

| Layer | Mechanism | Governance on the commit |
|---|---|---|
| Definitions (schema) | Global Branching → proposal → checks → required approvals | Branch `Owner` role or space `Administrator` to open a proposal |
| Data | Scenario → merge action | Submission criteria on the merge action |
| Modelled outcomes | Vertex Scenario | Functions-on-models published as actions |

### E. Global Branching has a documented leak

`lenses/data-model.md:204`, `:227` and `object-platform.md:120` treat Global Branching as clean. Ontology entities branch fully — "you can create, modify, or delete entities on the branch without affecting the `main` branch" — but **non-ontology resources do not**: "creating or deleting Foundry resources on a branch will affect `main`." Rebase "auto-resolves any non-conflicting changes"; true conflicts need a manual pick. `CONFIRMED` — [foundry-branching/core-concepts](https://www.palantir.com/docs/foundry/foundry-branching/core-concepts).

Terminology: a page titled **"Ontology branches [Legacy]"** exists — the older ontology-specific mechanism was superseded by the Global Branching integration. Current term: ontology branching via Global Branching, merged by **proposals**. `CONFIRMED` — [ontology-branches-legacy](https://www.palantir.com/docs/foundry/ontologies/ontology-branches-legacy).

### F. Type-system parts the corpus omits

| Concept | Detail | Confidence | Source |
|---|---|---|---|
| **Interfaces** | Zero corpus hits. "An interface is an Ontology type that describes the shape of an object type and its capabilities" — abstract, "not backed by datasets", not instantiable; composed of interface properties + link type constraints + action type constraints + metadata; can extend other interfaces. Foundry's mechanism for one application over many object types | CONFIRMED | [interfaces](https://www.palantir.com/docs/foundry/interfaces/interface-overview) |
| Interface support is uneven | "fully supported in Ontology Manager, Marketplace, and TypeScript v2 Functions, with **partial support in Actions and Object Set Service**" — a warning about half-plumbed polymorphism | CONFIRMED | same |
| **Shared properties** | "a property that can be used on multiple object types in your ontology" — what interfaces are built from | CONFIRMED | [core-concepts](https://www.palantir.com/docs/foundry/ontology/core-concepts) |
| **Object sets** | First-class ("a collection of multiple object instances"), with a dedicated Object Set Service — the currency Workshop, functions and OSDK pass around, not just a query result | CONFIRMED | core-concepts |
| Advanced base types (fuller than `lenses/data-model.md:237`) | Vector (semantic search), Geopoint, Geoshape, Attachment, Time series, **Geotemporal series**, **Media reference**, **Cipher text**, **Struct**, **Mandatory Control**. "all field types are valid base types except for `Map` and `Binary`". Vector and Time series cannot appear in arrays | CONFIRMED | [base-types](https://www.palantir.com/docs/foundry/object-link-types/base-types), mandatory-control-properties |
| Time/geo/media are *reference* properties | A time series property "stores a history of timestamped values" where a conventional property holds one value; media reference points at a media set. Pattern: reference-typed property into a specialised store | CONFIRMED | [time-series-properties](https://www.palantir.com/docs/foundry/time-series/time-series-properties), base-types |

### G. AIP separability — usable as a hard constraint

`support.md:192` says it in one line; here is the enumeration, because **the boundary is per-feature, not per-application** — "we use Workshop" says nothing about AI exposure.

AIP is separately enabled: administrators "may govern usage of these capabilities via Control Panel under **AIP settings**", with a distinct enable flow and per-LLM enablement per user group. Foundry is described independently as "the foundational data operations platform… data management, logic authoring, Ontology development, analytics, and workflow development". `CONFIRMED` — [aip-features](https://www.palantir.com/docs/foundry/aip/aip-features), [platforms](https://www.palantir.com/docs/foundry/architecture-center/platforms).

**LLM-dependent:** AIP Assist + sidebar, AIP Logic, AIP Chatbot Studio (**formerly AIP Agent Studio** — renamed), AIP Evals, AIP Threads, Palantir MCP, Ontology MCP, **Pilot** (NL app builder), Workshop's AIP Agent widget, Pipeline Builder's Use-LLM and Text-to-embeddings nodes, LLM effects inside Automate, Notepad, Scheduler. **Machinery** is described in Palantir's own application reference as using AIP capabilities — `UNCERTAIN`, treat as AI-linked until verified.

**Deterministic:** the ontology itself, submission criteria, object/property policies, markings/CBAC/purposes, restricted views, Global Branching and proposals, Scenarios and merge actions, Action Log and edit history, datasets/transforms/Code Repositories, Pipeline Builder minus its LLM nodes, data expectations and health checks, DevOps/Marketplace, Workshop minus AIP widgets, Object Views, Object Explorer, Contour, Quiver, Insight, Map, Fusion, OSDK, Automate minus its AIP effects.

Caveat: `Vector` properties exist "for use in a semantic search" — a deterministic property type, but the documented population path is an LLM node. Whether a non-LLM embedding path exists: `UNCERTAIN`.

### H. Packaging, and quality-gate timing

**Packaging** (`dashboard.md:112` mentions it once, in a dashboard context). A **product** is a "collections of Foundry resources that a product builder has made available to install" — named examples include ontologies, use cases combining Workshop applications and functions, pipelines, containerised models, Carbon workspaces. Versioned, with manual or automatic upgrade; installed multiple times with per-environment inputs. **[secondary]** evidence records an "Always install source Ontology API names" option so API names survive the move and application code keeps working. `CONFIRMED` — [devops/core-concepts](https://www.palantir.com/docs/foundry/devops/core-concepts); `LIKELY` — [marketplace-installation](https://www.palantir.com/docs/foundry/developer-console/marketplace-installation), [marketplace-ontology-types](https://www.palantir.com/docs/foundry/object-link-types/marketplace-ontology-types). Whether object *instances* ship with a product or only definitions plus required inputs: `UNKNOWN`.

**Quality gates** (zero corpus hits for expectations or health checks). The useful distinction is *when* they run. **Data expectations** run during the build and can abort it (`on_error` default `FAIL`; Pipeline Builder supports primary-key and row-count expectations and fails the build) — `LIKELY`, [data-expectations](https://www.palantir.com/docs/foundry/transforms-python/data-expectations-getting-started), [pipeline-builder expectations](https://www.palantir.com/docs/foundry/pipeline-builder/dataexpectations-overview). **Health checks** run after builds and alert — "monitoring and alerting on common issues across datasets and other resource types", on datasets, schedules and tables. `CONFIRMED` — [data-health](https://www.palantir.com/docs/foundry/data-health/overview).

**Apollo** (zero corpus hits) is *not* the content-promotion mechanism: it "is the continuous delivery platform that manages the underlying infrastructure that hosts both Foundry and AIP services". It deploys the platform; content is promoted by DevOps + Marketplace. `CONFIRMED` — [platforms](https://www.palantir.com/docs/foundry/architecture-center/platforms).

### I. Documented ceilings the corpus does not record

| Limit | Value | Source |
|---|---|---|
| Action submission | 50 object types · 10,000 objects · 32 KB (OSv1) / 3 MB (OSv2) per object edit | [scale-property-limits](https://www.palantir.com/docs/foundry/action-types/scale-property-limits) |
| Batch | 10,000 calls; 20 for non-batched function-backed actions | same, [apply-action-batch](https://www.palantir.com/docs/foundry/api/ontologies-v2-resources/actions/apply-action-batch) |
| Notifications | 500 recipients (50 if function-rendered) | scale-property-limits |
| OSv2 data | 12 MB strings · 100,000-element arrays · **no empty strings** · no `NaN`/`±infinity` · no nested arrays · PKs cannot be geopoint/geoshape/array/time-series/real-number | [data-restrictions](https://www.palantir.com/docs/foundry/object-indexing/data-restrictions) |
| Properties per object type | ~2000 (OSv2) | [object-backend/overview](https://www.palantir.com/docs/foundry/object-backend/overview) |
| OSv1 consistency hazard | "changes to objects or links stored in Object Storage V1 are eventually consistent and may take some time to be visible" (OSv2 is immediate) | [apply-action](https://www.palantir.com/docs/foundry/api/ontologies-v2-resources/actions/apply-action) |
| Ontology volume metric | GB / GB-Month, "size of all objects… including their properties and links"; **no published caps** on object/type/link counts | [volume-usage](https://www.palantir.com/docs/foundry/ontologies/volume-usage) |
| Write throughput / ingestion latency SLOs | **UNKNOWN** — not published. Action limits are per-submission caps, not throughput | — |

---

## 3. Where the corpus and my sources disagree

### 3-A. `[minor]` "Actions-only or reopened" is a binary; there is a third path

- **Corpus:** `object-platform.md:102`, `:258` — object edits are "locked to actions-only **OR** reopened to Forms / direct Object-Explorer edit / API".
- **Mine:** correct as far as it goes, but both branches are *user-surface* paths. Pipeline writes through the Object Data Funnel mutate object property values with no Action at all. `CONFIRMED` — [object-backend/overview](https://www.palantir.com/docs/foundry/object-backend/overview).
- **Believe: both.** The corpus adjudication was right to soften the original claim; it is incomplete rather than wrong. Practical effect: "all mutation flows through one audited verb" is not achievable by locking actions alone if a pipeline also writes the same object type.

### 3-B. `[MATERIAL — affects a cost-L steal]` Markings are not cell-level, and their propagation follows derivation, not links

- **Corpus:** `lenses/governance.md:34` — "markings (cell-level, propagate down lineage) `[V]`". `:103` — "markings propagate cell→downstream via lineage". `:161` — "Foundry marks a source cell and every derived dataset inherits the eligibility gate. **Marking = property + forbid-policy; propagate along link-types.** `[V Foundry]` — **L.**" Carried as governance steal ①, "highest-value governance feature we lack".
- **Mine:** three separable mechanisms are merged here.
  1. **Markings** are mandatory controls on **files, folders and Projects** — resource granularity, not cells. "Markings travel with the data. A file may inherit a Marking in two ways: via the file hierarchy and/or via data dependencies", and "All resources derived from a marked file, folder, or Project will assume a Marking unless the Marking is explicitly removed." Conjunctive (boolean AND). Removal only at the point of original application, which "will **immediately** remove the Marking from downstream files and data dependencies". `CONFIRMED` — [markings](https://www.palantir.com/docs/foundry/security/markings).
  2. **Row-granular marking enforcement** is the **mandatory control property** — explicitly *not* per-field: "A mandatory control property secures all other properties in the same datasource", must be required, and "must be mapped to a marking column on a restricted view". `CONFIRMED` — [mandatory-control-properties](https://www.palantir.com/docs/foundry/object-link-types/mandatory-control-properties).
  3. **Cell-level** comes from object policy × property policy, configured "on the object type, **independently of the permissions on the backing data source**" — and these can *drop* inherited mandatory controls ("By default, policies inherit mandatory controls (markings, organizations, classifications) from backing data sources… can be removed if no longer necessary"). `CONFIRMED` — [object-security-policies](https://www.palantir.com/docs/foundry/object-permissioning/object-security-policies).

  And the load-bearing negative: **no documented propagation of markings along ontology link types.** Inheritance is via file hierarchy and *transform/data dependencies*. A search aimed specifically at link-following propagation surfaced only the hierarchy-and-data-dependency language. Pipelines opt out with a `stop_propagating` input property, and the lineage graph flags nodes that do. `LIKELY (negative)` — [remove-inherited-markings](https://www.palantir.com/docs/foundry/building-pipelines/remove-inherited-markings).
- **Believe: mine.** The corpus cell cites the markings overview page but asserts a granularity and a propagation axis that page does not support; my claims are direct quotes from three official pages, including one (`mandatory-control-properties`) the corpus itself cites *correctly* elsewhere.
- **The corpus already contains its own correction, unpropagated.** `policy.md:63` and the `policy.md:160` adjudication get this exactly right — "a required-marking row/datasource-level gate, not a per-field classifier; object/property policies give field grain". `lenses/governance.md:34` was never updated to match. Same-corpus contradiction, one file behind the other.
- **Why it changes the plan, not just the prose:** steal ① is sized **L** on the belief that it copies a shipped Foundry feature. What Foundry ships is marking inheritance along a **derivation graph between stored resources**. Propagation along **ontology link types** is a different traversal with different semantics — a link is a relationship, not a derivation, so "employee → payroll record" would propagate a marking where no data was derived. That design is novel, not a copy: it needs its own answers (which link directions propagate? both sides of a m-m? what is the opt-out?) and it cannot inherit Foundry's correctness argument. Two named Foundry capabilities *are* directly copyable and cheaper: **propagation-impact simulation** before applying a marking (`LIKELY` — [see-impact-marking-changes](https://www.palantir.com/docs/foundry/data-lineage/see-impact-marking-changes)) and **removal only at the point of origin**.

### 3-C. `[MATERIAL]` "Lineage is inherently audit-grade" overstates what lineage answers

- **Corpus:** `evidence.md:25`, `:94` — "Immutable transactional datasets + data lineage… strong on custody/provenance"; "Transaction log + lineage is inherently audit-grade. `[V]`" citing Data Lineage. `object-platform.md:114` and `overview.md:272` locate object history in writeback transactions + changelog.
- **Mine:** dataset lineage is resource-level, appears to have no public API, and answers "which datasets fed this dataset" — not "which decision changed this object's field, by whom, under which parameters". That is answered by the **Action Log**, which the corpus never mentions and which is queryable at runtime precisely because it is an object type. `CONFIRMED` — [action-log](https://www.palantir.com/docs/foundry/action-types/action-log); `LIKELY (negative)` on the API — [community request](https://community.palantir.com/t/expose-resource-lineage-as-a-public-api/6496) **[secondary]**.
- **Believe: mine**, with the caveat that the corpus claim is directionally fine for dataset custody and only wrong if read as per-object provenance.
- **Effect:** for traceable production inputs the transferable design is Action-Log-shaped (a submission object linked to every affected object), not lineage-shaped. Genealogy must be modelled as object types + link types.

### 3-D. `[minor]` Action side effects: "build" is unverified

- **Corpus:** `object-platform.md:102` — side effects "(notify/webhook/build)".
- **Mine:** the rules page enumerates exactly two non-ontology rule types — notification rules and webhook rules. No build side effect appeared. `UNCERTAIN` — [rules](https://www.palantir.com/docs/foundry/action-types/rules).
- **Believe: unresolved.** A build-triggering side effect may exist under Automate or schedules rather than as an action rule. Low stakes.

### 3-E. `[minor]` "No approval inbox primitive" — Foundry has an Approvals application

- **Corpus:** `appr.md:35` — "Palantir: no approval inbox primitive; pending edits surface as Action submissions inside Workshop apps / Object Explorer".
- **Mine:** a native **Approvals** application exists — "manages workflow of requesting, approving, and invoking changes in Foundry", consolidating "compliance, governance, and peer-review workflows". Documented request types: add a user to a group, add a user to a Marking, add a Project reference, and "Make changes to the Ontology". `CONFIRMED` — [approvals](https://www.palantir.com/docs/foundry/approvals/overview).
- **Believe: the corpus, in substance.** Approvals governs *platform* requests (permissions, markings, ontology proposals), not business records, so "no approval inbox for domain objects" holds. The correction is factual scope, not verdict — and it is informative: Foundry built a native approvals surface for governance while leaving domain approvals to action types + Workshop. How Approvals workflows are *configured* (stages, N-of-M, sequencing): `UNKNOWN`.

### 3-F. `[no disagreement — hedge confirmed]` Object-level as-of

`lenses/data-model.md:219-221` and `INDEX.md:62` hedge deliberately: Foundry's temporal story centres on time-series properties and edit-history transactions, and they explicitly avoid asserting Foundry *lacks* object-level as-of. I found no point-in-time whole-object reconstruction feature. That is consistent with the hedge and does not upgrade it — absence of evidence in the pages I read, not evidence of absence. **Keep the hedge as written.** `UNKNOWN`.

---

## 4. Open questions still `UNCERTAIN` or `UNKNOWN`

Ordered by how much a decision hinges on the answer.

1. **Scenario immutability — sources conflict.** "A Scenario is immutable once created" vs a mutable sandbox merged by a merge action. Resolve before any dry-run design copies "Foundry Scenarios". `UNCERTAIN`.
2. **Writeback-vs-rebuild reconciliation.** The ontology merges pipeline output with writeback edits; the policy when a rebuild contradicts an edit is undocumented. The correctness question under "the ontology holds its own state". `UNKNOWN`.
3. **Per-row provenance.** No dataset-level per-row lineage found. Object-level provenance covers *edits* only, not "which input rows produced this property value". Decisive for production-input traceability. `UNKNOWN`.
4. **Ontology write throughput and ingestion latency.** Nothing published; any sizing decision needs it and cannot get it from public docs. `UNKNOWN`.
5. **Can a submission criterion invoke a function?** Function-backed *rules* are documented; function-backed *criteria* are not. Sets the hard ceiling on declarative write authorization. `UNKNOWN`.
6. **Approvals workflow configuration model** — stages, N-of-M, sequencing, whether it reaches past platform permissions. Directly relevant to multi-party sign-off. `UNKNOWN`.
7. **Marking propagation along ontology links.** Asserted absent above on a `LIKELY (negative)`. If any official page documents link-following propagation, governance steal ① reverts to a copy rather than a novel design — worth one targeted check before sizing. `UNCERTAIN`.
8. **Column-level security on restricted views.** Row-level is explicit; column-level was not stated on that page (property policies cover it at the ontology layer). Matters only if the dataset layer must carry column security independently. `UNCERTAIN`.
9. **Authoritative list of backing datasource types.** Datasets, restricted views, streams confirmed; virtual tables and media sets appear in adjacent docs; the canonical page 404'd. `UNCERTAIN`.
10. **CBAC mechanics vs plain markings.** Named as a distinct control, not read. Relevant if a classification lattice is needed rather than flat conjunctive markings. `UNKNOWN`.
11. **Do Marketplace products ship object instances or only definitions + inputs?** Decides whether a vertical package can carry seed data. `UNKNOWN`.
12. **Action side-effect "build"** (§3-D). `UNCERTAIN`.
13. **Machinery's AI dependence**, and **whether a non-LLM path populates `Vector` properties.** Both matter under a no-LLM product constraint. `UNCERTAIN`.
14. **Terminology drift generally.** Two renames confirmed (validations → submission criteria; AIP Agent Studio → AIP Chatbot Studio); three legacy markers visible (Ontology branches [Legacy], Code Workbook [Legacy], a Legacy Object Views section). Public docs carry no systematic version history, so other renames are plausible and undetectable from here. `UNKNOWN`.
