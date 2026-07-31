# Korean legal sources — what each of four repositories can and cannot be used for

> `Status: RESEARCH — sourced 2026-07-30. Asserts no Korean legal conclusion and changes no control's HOLD.`
>
> Four repositories the owner supplied. The register governs what may be cited as evidence, so the useful
> question is not "is this good" but **"which register field can this fill, and which can it never fill."**

## The rule these are measured against

`docs/program/console-jurisdiction-register.json` `source_process` names four `allowed_sources`:
**official_legislation_portal**, **official_regulator**, **court_or_authorized_interpretation**,
**qualified_legal_opinion**. Its `required_fields` per source are `source_uri`, `retrieved_at`,
`effective_date`, `applicability_reason`, `product_scope`, `data_scope`, `control_mapping`,
`evidence_path`, `qualified_reviewer`, `review_expiry`. Its `change_rule` invalidates dependent release
evidence on any source, date, scope or interpretation change, and its `uncertainty_rule` is that missing,
stale, conflicting or unqualified authority is `HOLD` — *"agents may not invent certainty."*

**No repository here is itself an allowed source.** All four are third-party work. That does not make them
useless; it makes them **discovery aids** — they tell you which official document to fetch and cite, which
is most of the work. The fourth, `legalize-pipeline`, is the exception that matters: it does not *contain* an
allowed source, it *reaches* one, and §"The official API" below records what that takes.

## 1. `legalize-kr/legalize-kr` — the most useful of the archives

Korean legislation as a **git archive**, one Markdown file per law, amendments recorded as commits dated to
their actual promulgation dates. Data obtained from the **National Legal Information Center OpenAPI**
(law.go.kr) — which *is* an `official_legislation_portal`.

Each file's YAML frontmatter records **공포일자** (promulgation date), **공포번호** (promulgation number),
**시행일자** (effective date), and a **source URL to the official centre**. Legal text is public-domain ROK
government work; the curation and pipeline are the maintainers'.

**What it can fill:** it identifies the exact official document, its promulgation number, and its effective
date — so it supplies `source_uri` and `effective_date` *by pointing at law.go.kr*, and it makes finding
the right instrument fast. That is a real acceleration for release-gate condition 1, which requires at
least one official source per statutory item in scope.

**What it must never be:** the citation itself. Cite law.go.kr, not the mirror.

**And one hard constraint, from its own README:** *"If the pipeline improves, force-push may occur, changing
all commit hashes."* So **a commit hash from this repository can never be an evidence anchor.** Evidence
must be candidate-bound and the `change_rule` invalidates on change; a hash that can be rewritten is the
opposite of that. Anchor on 공포번호 + 시행일자 + the law.go.kr URL, all of which come from the official
API and are stable.

## 2. `legalize-kr/cli-tools` — a research aid, one step further from the source

A Python CLI and **MCP server** querying 법령·판례·행정규칙·자치법규. Emits versioned JSON and Markdown with
frontmatter, MIT code over public-domain text.

**Note carefully:** it queries the **GitHub mirror via the GitHub REST API**, *not* law.go.kr. So it is two
removes from the official source, and its reported provenance capture — explicit retrieval timestamps and
per-document source URLs — is limited relative to what the register requires.

**What it is good for:** looking up statute text quickly while *formulating* questions, and the MCP server
makes that cheap. **What it is not good for:** supplying `retrieved_at`, or any field an auditor would check.

## 3. `jclab-joseph/it-legal` — wrong domain for payroll, right domain for the PII gap

A community technical guide to Korean IT law for SaaS developers, maintained by an individual. Covers
**PIPA/개인정보보호법**, 정보통신망법, 전상법, 전금법, 신정법, cloud, AI transparency and copyright/OSP
liability, translating statute requirements into code-level patterns rather than commentary — retention
scheduling tables, consent event logs with snapshot hashing, tenant isolation, crypto-shredding.

**It explicitly excludes 근로기준법, payroll and 4대보험.** So for the payroll golden-case work it is not
relevant, and should not be reached for there.

**For the PII gap it is directly relevant**, and that gap is real: searched across all 205 migrations,
there is no table whose name contains `pii`, `personal_data`, `retention_polic*`, `erasure`, `dsr`,
`subject_request`, `data_subject`, `anonymization`, `redaction` or `purge`; the only consent table is
location-tracking consent under 위치정보법; and the reserved evidence-retention migration slot was released
with a body of `SELECT 1;`.

### The conflict this repository surfaces that nobody here had raised

Its **crypto-shredding** pattern exists to resolve *"destruction mandates conflicting with backup
retention."* We have exactly that conflict and have not named it:

- **ADR-0015** mandates continuous WAL archiving and PITR, with RPO ≤5min and RTO ≤1h proven by restore to
  an arbitrary timestamp.
- A PIPA erasure obligation requires personal data to be destroyed.
- **A backup you can restore to an arbitrary past timestamp still contains the data you destroyed.**

Deleting a row does not satisfy an erasure obligation while a PITR window can reconstruct it. Crypto-
shredding — encrypting per-subject and destroying the key — is the documented resolution, and it is an
architectural decision with an ADR-0015 interaction, not a feature.

**This is a question for qualified counsel, not a conclusion here.** What this document asserts is only
that the *conflict exists in our architecture* and that no record addresses it. Note also that the guide is
2026-baseline and says many provisions it cites are **not yet in force**, so its timing claims need checking
against 시행일자 for each instrument — which is exactly what repository 1 is good for.

## How to use these together

1. **Discovery** — `legalize-kr` to find the instrument, its 공포번호 and its 시행일자.
2. **Citation** — fetch and cite **law.go.kr** directly. Never the mirror, never a commit hash.
3. **Question formulation** — `cli-tools`' MCP server to read text while drafting the questions a
   professional must answer.
4. **PII architecture only** — `it-legal` for code-level patterns, remembering it is a community work and
   excludes labour and payroll entirely.

None of this moves a control off `HOLD`. Only a qualified Korea legal or compliance authority may do that,
with a candidate-bound I2/I3 receipt in independent custody.

## Follow-up worth doing

**Name the erasure-versus-PITR conflict in a record.** It sits between ADR-0015 and the absent PII
infrastructure, and it is the kind of architectural tension that is cheap to resolve before a person's data
is in the system and expensive afterwards. It needs a decision, and the decision needs counsel — but the
*question* can be written now.

## The official API, and what is known about calling it

`legalize-pipeline` calls **`open.law.go.kr`** — the National Legal Information Center OpenAPI, which **is**
an `official_legislation_portal` and therefore the one allowed source among everything here. That makes the
registration step the single thing standing between us and citable evidence for release-gate condition 1.

**The OC key is self-designated.** Per the registration form: *"국가법령정보 OPEN API의 인증키는 사용자가
직접 지정하여 사용할 수 있습니다"* — the caller chooses the value rather than receiving a generated one, and
it is passed as the `OC` request parameter. So it is an **identifier, not a bearer secret**: a leak is an
attribution problem, not a credential compromise. It still comes from the environment (`LAW_OC`) and is
never committed.

**Operational facts from the pipeline's README, worth not rediscovering:**

| | |
|---|---|
| Env var | `LAW_OC` |
| Throttle | 0.2 s default spacing between calls |
| Retry | exponential backoff, 2 / 4 / 8 s |
| Law identifier | **MST** |
| Detail response | XML, cached as `.cache/detail/{MST}.xml` |
| Revision history | JSON, cached as `.cache/history/{법령명}.json` |
| Licence | Apache-2.0 / MIT, so the approach is reusable |

**What is NOT known and must come from the official docs:** the base URL path, endpoint names, and the full
parameter set. The pipeline's README explicitly does not document them, and `open.law.go.kr`'s own
documentation requires a logged-in account.

### Why no fetcher has been written yet

Deliberate. Writing one now would mean guessing the endpoint spec and shipping code that has never made a
real call — which is the exact defect shape this repository has hit five times in a week: correct-looking
code that executes nowhere. The moment an OC value exists, the fetcher gets built **and verified against a
live call in the same pass**, so its first green is evidence rather than a hope.

What it should produce when it is written: for each statutory item, the `source_uri`, `retrieved_at`,
`effective_date` (시행일자) and 공포번호 the jurisdiction register requires as `required_fields` — fetched
from the official portal, not from the mirror.
