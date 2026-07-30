# Payroll statutory source register — the fetch list for release-gate condition 1

> `Status: RESEARCH — discovery layer only. No citation here is an allowed source; asserts no Korean legal conclusion.`
>
> Retrieved **2026-07-30** from the `legalize-kr/legalize-kr` GitHub archive (a discovery aid, never a
> citation — see `docs/ideas/korean-legal-sources.md`) and from unauthenticated `www.law.go.kr` probes.
> This document turns `docs/specs/payroll.md` release gate condition 1 — *"the effective-dated rate table
> version has at least one official source per statutory item in scope"* — into a list of documents to fetch.
> It does not fetch them, and it moves nothing off `HOLD`.

## The headline: the layer is different for every item

The working assumption going in was *"a rate is set in a 시행령 or a 고시, not in the act."* The archive text
disagrees, item by item, and **the archive's text wins**:

- **국민연금 4.75% is set in the act** — in a 부칙 of a 2025 amendment, as a legislated 2026→2032 schedule.
  There is no annual notice to chase; the future values are already citable today.
- **The 간이세액표 is 별표 2 of a 대통령령**, not an NTS 고시. The NTS page the spec cites is a lookup
  service, not the instrument. The official table is a 172 KB HWP file downloadable without any API key.
- **산재보험료율 is delegated twice** — the act says 고용노동부령, and the 고용노동부령 then says 고시.
- **지방소득세 10% is in the act.** Cleanest citation on the list.
- Only **건강보험 / 장기요양 / 고용보험** follow the expected act→시행령 shape.

Four numbers are set by **고시**, and no 고시 is in the archive or resolvable by name on the public portal.
Those four are the whole reason this register needs a second source route.

## 1. The fetch list

`공포번호` and `시행일자` are the **file version** whose text was read, taken verbatim from the archive's YAML
frontmatter. Where a provision's own last amendment differs from the file version, both are given — the
provision date is what a reviewer needs to diff against.

| # | Statutory item | Authorising act | Instrument that sets the number | 공포번호 | 시행일자 | Official URL to cite | Annually re-published? | Confidence |
|---|---|---|---|---|---|---|---|---|
| 1 | **국민연금** employee share — 4.75% (2026) | 국민연금법 제88조제3항 | **The act itself.** 제88조제3항 reads 기준소득월액의 1천분의 65; for 2026 it is displaced by **부칙 <법률 제20903호, 2025.4.2> 제4조제1항제1호: 1만분의 475** | operative 부칙: **법률 제20903호** (공포 2025-04-02). File read: 법률 제21689호 | 부칙 제1조: **2026-01-01**. File: 2026-05-26 (in force) | <https://www.law.go.kr/법령/국민연금법> | **No — legislated schedule.** 2026 4.75%, 2027 5.00%, 2028 5.25%, 2029 5.50%, 2030 5.75%, 2031 6.00%, 2032 6.25%, then 제88조제3항's 6.5% | **HIGH** — quoted verbatim from a file whose 시행일자 is in force |
| 2 | **국민연금 기준소득월액** cap and floor | 국민연금법 제3조제1항제5호 | Split: **국민연금법 시행령 제5조** sets the indexation formula and orders publication; the **amounts are a 보건복지부장관 고시** issued by 3월 31일 each year (제5조제3항), applying **해당 연도 7월 ~ 다음 연도 6월** (제5조제4항) | 시행령: **대통령령 제35909호** (2025-12-16). 고시: no 공포번호 | 시행령: 2025-12-16 (in force). Amounts: July→June window | Decree: <https://www.law.go.kr/법령/국민연금법시행령> · **고시: NOT ESTABLISHED** (see Q2) | **YES — annual 고시 by 3월 31일** | **HIGH** for the mechanism and the July–June window; the 고시 document itself **NOT LOCATED** |
| 3 | **건강보험** employee share — 3.595% | 국민건강보험법 제73조제1항 (1천분의 80 범위, 심의위원회 의결 + 대통령령) | **국민건강보험법 시행령 제44조제1항: 1만분의 719** (= 7.19%). The 50/50 split is in the act — **국민건강보험법 제76조제1항: 100분의 50씩** | 시행령: **대통령령 제36116호** (2026-02-19); 제44조 last amended **2025.12.23**. Act file read: 법률 제21687호 | 시행령: 2026-02-19 (in force). **Act file 시행일자 2027-01-01 — not yet in force** | <https://www.law.go.kr/법령/국민건강보험법시행령> · <https://www.law.go.kr/법령/국민건강보험법> | **Effectively yes.** 제44조's 개정 list is 2012·2013·2014·2015·2017·2018·2019·2020·2021·2022·2024·2025 — almost always December | **HIGH** for 7.19%. **MEDIUM** for the 50/50 split — read from a future-effective act file (see Q5) |
| 4 | **장기요양보험** employee share — 0.4724% | 노인장기요양보험법 제9조제2항 (장기요양위원회 심의 + 대통령령) | **노인장기요양보험법 시행령 제4조: 100만분의 9,448** (= 0.9448%). Bearer split arrives **by 준용** — 노인장기요양보험법 제11조 준용s 국민건강보험법 제76조 | 시행령: **대통령령 제36325호** (2026-05-12); 제4조 last amended **2025.12.30**. Act file read: 법률 제21690호 | 시행령: 2026-05-12 (in force). **Act file 시행일자 2026-11-27 — not yet in force** | <https://www.law.go.kr/법령/노인장기요양보험법시행령> · <https://www.law.go.kr/법령/노인장기요양보험법> | **Effectively yes.** 제4조 amended 2008·2009·2017·2018·2019·2020·2021·2022·2023·2025 | **HIGH** for 0.9448%. **MEDIUM-LOW** for modelling it as 0.4724% × 보수월액 (see Q1); **MEDIUM** for the split — a 준용 chain across two future-effective files |
| 5 | **고용보험 실업급여** employee share — 0.9% | 징수법 제14조제1항 (1000분의 30 범위, 고용보험위원회 심의 + 대통령령). Employee half: 징수법 제13조제2항 | **징수법 시행령 제12조제1항제2호: 1천분의 18** (= 1.8%); employee bears ½ | 시행령: **대통령령 제35935호** (2025-12-23); 제12조 last amended **2023.12.26**. Act file read: 법률 제21532호 | 시행령: 2025-12-23 (in force). **Act file 시행일자 2026-10-08 — not yet in force** | <https://www.law.go.kr/법령/고용보험및산업재해보상보험의보험료징수등에관한법률시행령> | **No fixed cadence.** 제12조 amended 2011·2013·2019·2021·2023 — irregular, so it expires on change, not on a calendar | **HIGH** for 1.8%. **MEDIUM** for the ½ share — 제13조제2항 carries `<개정 2026.3.17>` and sits in a future-effective file |
| 6 | **산재보험** — employer-only, industry tariff | 징수법 제14조제3항→제6항 (→ 고용노동부령); employer-only per 징수법 제13조제5항 | **Two-layer delegation.** 징수법 시행규칙 제12조: rates are *"고용노동부장관이 정하여 고시"*; the 시행규칙's **별표 1** supplies only 구성과 산정방법. Then 제13조 개별실적요율 experience-rates the result **per employer** | 시행규칙: **고용노동부령 제473호** (2026-07-01); 제12조 last amended 2017.12.28 | 2026-07-01 (in force) | 시행규칙: <https://www.law.go.kr/법령/고용보험및산업재해보상보험의보험료징수등에관한법률시행규칙> · 별표 1 (HWP): <https://www.law.go.kr/LSW/flDownload.do?flSeq=166487111> · **고시: NOT ESTABLISHED** (Q3) | **YES — per 보험연도**, and 제14조제6항 caps year-on-year movement at ±30% | **HIGH** that there is no employee deduction and that the number is a 고시 + per-employer 요율. The 고시 **NOT LOCATED** |
| 7 | **최저임금** — 2026 guard data | 최저임금법 제10조 | **고용노동부장관 고시.** 제10조제1항 requires 고시; 제10조제2항 makes it effective **1 January of the following year**. 최저임금법 시행령 is 대통령령 제29469호 of **2018-12-31** and does not carry the amount | 고시 has no 공포번호. Act file read: 법률 제21534호 | Effect date fixed by 제10조제2항: **2026-01-01**. **Act file 시행일자 2026-12-08 — not yet in force** | **고시: NOT LOCATED** on law.go.kr by name (Q4). Act: <https://www.law.go.kr/법령/최저임금법> | **YES — every year** | **HIGH** on the instrument type and its effect date; the document **NOT LOCATED** |
| 8 | **소득세** — 근로소득 간이세액표 | 소득세법 제129조제3항 (*"대통령령으로 정하는 근로소득 간이세액표"*) | **소득세법 시행령 별표 2**, per 시행령 제189조제1항. A **대통령령 별표 — not an NTS 고시.** Confirmed fetchable: 172,032 bytes, `application/hwp`, HTTP 200, no auth | **대통령령 제36343호** (공포 2026-05-22) | **2026-07-01** (in force) — note this is *after* 공포 (Q8) | Decree: <https://www.law.go.kr/법령/소득세법시행령> · 별표 2 HWP: <https://www.law.go.kr/LSW/flDownload.do?flSeq=164391981> · PDF: <https://www.law.go.kr/LSW/flDownload.do?flSeq=164391983> | **No annual notice.** It changes when the 별표 is amended | **HIGH** — delegation chain quoted verbatim end to end and the file downloads |
| 9 | **지방소득세** — 근로소득 특별징수 | 지방세법 제103조의13제1항 | **The act itself** — *"원천징수하는 소득세… 의 100분의 10"*, withheld simultaneously with 소득세 | **법률 제21308호** (2025-12-31) | **2026-01-01** (in force) | <https://www.law.go.kr/법령/지방세법> | **No** | **HIGH** |

### What this changes about the spec's cited sources

`docs/specs/payroll.md` lists NPS, NHIS, 최저임금위원회 and NTS pages as the sources checked on 2026-06-27.
Every rate in the spec matched what the archive shows — 4.75%, 7.19%/3.595%, 0.9448%/0.4724%, 1.8%/0.9%,
100분의 10. **No arithmetic in the spec is contradicted by anything found here.** What changes is *which
document is the instrument*:

- Items 1, 8 and 9 have a **better** citation available than the one in the spec — the act or the decree
  별표 rather than an agency explainer page.
- Items 3, 4, 5 have a **more precise** citation — the 시행령 article, not the insurer's rate notice page.
- Items 2, 6, 7 genuinely need the agency route, because the instrument is a 고시 (below).

## 2. Items whose instrument is not established — the questions

**These are the fastest things for a 노무사/세무사 or 법제처 to answer. None is a guess below; each is a
gap I could not close from the archive.**

**Q1 — 장기요양: is `보수월액 × 0.4724%` the statutory formula, or only arithmetically close to it?**
The spec models the employee share as 0.4724% of 보수월액. 노인장기요양보험법 제9조제1항 as read does not
describe a rate on 보수월액. It describes: take the 건강보험료액, **subtract 경감/면제 amounts under
국민건강보험법 제74조·제75조**, then multiply by *"건강보험료율 대비 장기요양보험료율의 비율(소수점 이하
다섯째자리에서 반올림한다)"*. Two consequences to confirm:
  1. For an employee with any 경감 or 면제, the base is the reduced health premium — so a direct rate on
     보수월액 would **not** reproduce the statutory result.
  2. The ratio is expressly rounded at the fifth decimal place. 0.9448 ÷ 7.19 = 0.131404…, rounding to
     0.13140, which is not identical to applying 0.9448% directly.
  **Question:** is the direct-rate model acceptable for the in-scope population, and if so under what
  documented assumption? *(I make no claim that either result is correct.)*

**Q2 — which document is the 기준소득월액 상한액·하한액 고시?**
국민연금법 시행령 제5조제3항 requires 보건복지부장관 to 고시 the amounts by 3월 31일 annually. I could not
locate that 고시 as a document: absent from the archive, and not resolvable by name on the public portal.
The spec quotes the 2025-07~2026-06 and 2026-07~2027-06 figures from an NPS page — which matches 제5조제4항's
July–June window exactly. **Question:** what is the 고시's official title, number and citable URL, and is
the NPS page a reproduction of it or an independent publication?

**Q3 — which document is the 사업종류별 산재보험료율 고시?**
징수법 시행규칙 제12조 delegates the actual rates to a 고용노동부장관 고시. Not in the archive; not resolvable
by name. **Question:** the 고시 title/number for 보험연도 2026, and separately — since 제13조 개별실적요율
experience-rates per employer — **is any single 산재 rate ever correct for a tenant, or must it always be a
per-employer input?** The spec already treats 산재 as "industry-tariff-required"; this question asks whether
"industry" is even the right granularity.

**Q4 — which document is the 2026 최저임금 고시, and is the 최저임금위원회 table the instrument?**
최저임금법 제10조 makes the 고용노동부장관 고시 the operative instrument. The spec cites
`minimumwage.go.kr`, which is the **Commission** — a 심의 body whose 최저임금안 goes to the Minister under
제8조·제9조. **Question:** confirm the 고시 is the instrument and the Commission table is a summary, and
supply the 고시's citable identifier. Also confirm whether 제10조제2항's proviso (the Minister may set a
different effective date per business type) is inactive for 2026.

**Q5 — for a payroll pay date, which *version* of a law governs, and how do we know we read that one?**
This is the most dangerous gap and it is procedural, not legal. **6 of the 27 archive files I sampled carry a
`시행일자` later than today (2026-07-30) while `상태` still reads `시행`** — 국민건강보험법 법률 (2027-01-01),
국민건강보험법 시행규칙 (2026-08-11), 노인장기요양보험법 법률 (2026-11-27), 고용보험법 법률 (2026-11-27),
징수법 법률 (2026-10-08), 최저임금법 법률 (2026-12-08). The archive's `main` file is the latest **promulgated**
consolidated text, which is not necessarily the text **in force** on a pay date. Three of the four
"who bears what share" provisions in this register (items 3, 4, 5) were read from such files, which is why
their confidence is MEDIUM rather than HIGH. **Question:** what is the retrieval procedure that guarantees
the version in force on a given pay date? *(This is answerable without counsel — the OpenAPI's
현행법령(시행일) endpoint exists for it. See §3.)*

**Q6 — 건강보험 scope: is the 사립학교 교원 case in scope?**
국민건강보험법 제76조제1항's proviso splits 50/30/20 between employee, 사용자 and 국가 for 사립학교 교원, not
50/50. **Question:** is that population in scope for this slice? If not, say so in the spec so the 50/50
model is bounded rather than assumed universal.

**Q7 — does the kernel encode the 국민연금 2026→2032 schedule now?**
Item 1 is not an annually refreshed number; it is a seven-step legislated ramp ending at 6.5%. **Question:**
should the rate table carry all eight effective-dated rows today, or only 2026? Encoding the schedule removes
seven future refresh events; encoding only 2026 makes the kernel fail closed each January. Either is
defensible — but note that item 1's effective-dating is **calendar-year** while item 2's cap/floor is
**July–June**, so a single "2026 rate table version" spans two different effective-date axes.

**Q8 — which 간이세액표 applies to a pay date between 2026-05-22 and 2026-07-01?**
소득세법 시행령 제36343호 was 공포 2026-05-22 with 시행일자 2026-07-01. The 별표 2 link recorded in the
archive frontmatter resolves to a file today, but I cannot tell from the link alone which version it is.
**Question:** does the recorded `flSeq` identify a version-bound file, and what is the prior version's link
for pay dates before 2026-07-01? *(`flSeq` is an opaque file-sequence id, not a semantic version — its
stability across amendments is unverified and it should not be treated as an evidence anchor without that
check.)*

## 3. What needs a `LAW_OC` key — the registration ask

**One line:** designate an `open.law.go.kr` `OC` value so payroll can call the **행정규칙 목록 조회 /
행정규칙 본문 조회** endpoints — the 고시 layer that items 2, 6 and 7 depend on and that no other route we
have reaches — and the **현행법령(시행일) 목록·본문 조회** endpoints, which answer Q5 by retrieving the text
in force on a given date rather than the latest promulgated text.

`docs/ideas/korean-legal-sources.md` already records the registration mechanics and I do not restate them:
the `OC` is **self-designated** (the caller picks the value), so it is an identifier passed as a request
parameter rather than a bearer secret, sourced from `LAW_OC`. What this register adds is **which endpoints
payroll specifically needs it for** — the two families above, named for the ask.

**One narrowing of that doc's "what is NOT known".** It records that endpoint names require a logged-in
account. They do not: `open.law.go.kr/LSO/openApi/guideList.do` is **publicly readable without any key**
(fetched 2026-07-30, "총 191 건") and enumerates the endpoint families by name — including
`행정규칙 목록 조회` / `행정규칙 본문 조회` and `현행법령(시행일) 목록 조회` / `현행법령(시행일) 본문 조회`.
So the **endpoint names are now known**; the **base URL path and parameter set are still not**, and those are
what remain behind the login. This narrows the gap, it does not close it — and it does not change that
gate condition 1 needs the key.

What does **not** need a key, confirmed by fetch today:
- **별표/서식 file downloads** — `www.law.go.kr/LSW/flDownload.do?flSeq=…` served the 간이세액표 as a
  172,032-byte `application/hwp` with HTTP 200 and no auth. So **the single most important payroll table on
  this list is already reachable.** (Parsing HWP is a separate problem; it is not an access problem.)
- **Human-readable 법령 URLs** — `www.law.go.kr/법령/{name}` resolves, but returns a ~1.3 KB frameset that
  loads content in an iframe. Fine as a `source_uri` for a human reviewer; useless for automated text
  retrieval. That is the second reason to want the key.

## 4. What is not in a statute archive at all, and where it comes from instead

The archive's own structure explains this: it stores 법률·대통령령·부령 only. Across the 27 files sampled the
`법령구분` values were **9 법률, 9 대통령령, 4 고용노동부령, 3 보건복지부령, 1 재정경제부령, 1 행정안전부령 —
zero 고시, zero 행정규칙.** Five direct probes for 고시-shaped directories (`사업종류별산재보험료율`,
`국민연금기준소득월액하한액과상한액`, `최저임금고시`, `근로소득간이세액표`, `장기요양보험료율고시`) all
returned **404**. This is a finding, not a failure: it tells us exactly which items need a second route.

| Not in the archive | Instrument | Where it must come from instead |
|---|---|---|
| **기준소득월액 상한액·하한액** (item 2) | 보건복지부장관 고시, annually by 3월 31일 | 보건복지부 / NPS as `official_regulator`, **or** law.go.kr 행정규칙 via the OC key. The NPS page in the spec is the practical starting point — but establish whether it reproduces the 고시 (Q2) |
| **사업종류별 산재보험료율** (item 6) | 고용노동부장관 고시, per 보험연도 | 고용노동부 / 근로복지공단 as `official_regulator`, **or** law.go.kr 행정규칙 via OC. The 시행규칙 별표 1 (already fetchable) gives only the calculation method, not the rates |
| **2026 최저임금** (item 7) | 고용노동부장관 고시 | 고용노동부 as `official_regulator`. 최저임금위원회 is a 심의 body — treat its table as corroboration, not as the instrument (Q4) |
| **The version in force on a pay date** (Q5) | n/a — a retrieval capability, not a document | OpenAPI 현행법령(시행일) endpoints. Nothing in the archive can supply this |

One item moves the *other* way, and it is the most useful correction in this register: the
**근로소득 간이세액표 is in the archive's metadata**, as 별표 2 of 소득세법 시행령 with a working official
download link. The spec's source-of-truth list points at 국세청 for it; the *instrument* is a 대통령령 별표
published by 법제처. NTS supplies the lookup service and the HomeTax download path — useful, but the
citable instrument is the decree.

## 5. Refresh cadence — what expires, and the `change_rule` consequence

`docs/program/console-jurisdiction-register.json` `source_process.change_rule`: *"A source, effective-date,
scope, or interpretation change invalidates dependent release evidence and requires a new candidate-bound
review."* Read against this list, the payroll rate table is **not a one-time fetch and cannot be**. Each row
below is a scheduled invalidation of every golden case and professional-validation artifact bound to it.

| When | What changes | `change_rule` consequence |
|---|---|---|
| **Every 3월 31일** | 기준소득월액 상한액·하한액 고시 (item 2) | New effective-date → dependent evidence invalid. Note the new amounts apply from **1 July**, so the 고시 and its effect are ~3 months apart |
| **Every 1 July** | The July→June cap/floor window turns over (item 2) | An effective-date change even in a year with no rate change |
| **Every 1 January, 2027 through 2033** | 국민연금 employee share steps 5.00 → 5.25 → 5.50 → 5.75 → 6.00 → 6.25 → 6.5% (item 1) | Seven scheduled invalidations. **Already knowable and citable today** — the only expiry on this list that can be pre-empted rather than watched |
| **Every 1 January** | 최저임금 고시 takes effect (item 7) | 최저임금법 제10조제2항. New source document each year |
| **Every 보험연도** | 사업종류별 산재보험료율 고시 (item 6) | Plus per-employer 개별실적요율, which can change without any 고시 change at all |
| **Most Decembers** | 건강보험 시행령 제44조 and 장기요양 시행령 제4조 (items 3, 4) | Not legally guaranteed annual, but empirically near-annual — treat as annual and verify |
| **Irregular** | 고용보험 실업급여 요율 (item 5) | 2011·2013·2019·2021·2023. No cadence to schedule — needs change detection, not a calendar |
| **On amendment** | 소득세법 시행령 별표 2 (item 8) | Watch the 별표, not the article. The article text (제189조) has been stable since 2010.2.18 while the table underneath it changes |

**The practical consequence:** the earliest expiry after any 2026 fetch is **2027-01-01** (four items at once:
국민연금 step, 최저임금 고시, 산재 보험연도, and likely the December decree amendments). A rate-table version
fetched now should carry a `review_expiry` no later than that, and item 8's watch target must be the 별표
file, not the decree article.

## 6. Method and limits

**How I fetched.** Repeatable without credentials:

1. **Structure discovery** — `api.github.com/repos/legalize-kr/legalize-kr/contents/kr/{법령명}` per law, to
   confirm which of `법률.md` / `시행령.md` / `시행규칙.md` exist. The directory name is the 법령명 with
   spaces removed, matching law.go.kr's URL convention (per the archive README).
2. **Text retrieval** — `raw.githubusercontent.com/legalize-kr/legalize-kr/main/kr/{법령명}/{file}.md`,
   27 files, 6.8 MB total, for the 9 laws in scope.
3. **Frontmatter extraction** — parsed the YAML block for 제목, 법령구분, 공포일자, 공포번호, 시행일자, 상태,
   출처, 법령MST, and the `첨부파일` list (which carries 별표 titles and official `flDownload` links —
   the single most useful field for this task and the one I had not expected to exist).
4. **Provision location** — regex for the rate keyword, then walked back to the nearest `##### 제N조 (…)`
   heading to read the whole article rather than a fragment. For item 1 the same walk over `부칙 <…>`
   headings located the operative transitional provision.
5. **Existence probes** — `www.law.go.kr` returns **HTTP 200 for nonexistent paths**, so status code proves
   nothing. A bogus 법령 path returns 1,196 bytes with `<title>국가법령정보센터 | 오류페이지</title>`; a real
   one returns ~1,273 bytes with the law's name as `<title>`. I used the `<title>` as the existence test.

**What I sampled.** 9 laws × 3 instrument levels = 27 files: 국민연금법, 국민건강보험법, 노인장기요양보험법,
고용보험법, 산업재해보상보험법, 고용보험및산업재해보상보험의보험료징수등에관한법률, 최저임금법, 소득세법,
지방세법. Every rate figure in this register was read verbatim from that text, not recalled.

**What I could not reach, and what that costs.**

- **No `open.law.go.kr` OpenAPI access** (no `LAW_OC`). Cost: cannot reach 행정규칙 (the 고시 layer), and
  cannot request the text in force on a specific date. These are exactly the two gaps in §2 and §3.
- **No 고시 located for items 2, 6, 7.** Absent from the archive (5/5 probes 404) and not resolvable by name
  through the `/행정규칙/{name}` URL pattern — all five candidate names returned the error page, identically
  to the bogus control. **I did not guess a 고시 title, number or URL**, which is why those cells read
  NOT ESTABLISHED rather than carrying a plausible-looking citation.
- **Did not open the 별표 files.** I confirmed the 간이세액표 link serves 172 KB of `application/hwp` with
  HTTP 200; I did not parse the HWP, so **nothing in this register asserts anything about the table's
  contents** — only about where it lives and that it is reachable.
- **Did not verify law.go.kr against the archive.** Every 공포번호 and 시행일자 here comes from the archive's
  frontmatter, which is derived from the official OpenAPI but is a third-party copy. Each one must be
  re-verified against law.go.kr before it becomes evidence. The frame-loaded page body (§3) is why I could
  not do that with curl today.
- **Six sampled files are future-effective** (Q5). Items 3, 4 and 5 inherit MEDIUM confidence from this.
- **No commit hash anywhere in this document.** The archive's README warns force-push may rewrite every
  hash; anchoring is on 공포번호 + 시행일자 + the law.go.kr URL, per `docs/ideas/korean-legal-sources.md`.

**What this document is not.** It is a fetch list. Nothing here is an `allowed_source`; every citation is a
third-party-derived pointer at a document someone must still retrieve from law.go.kr or the issuing
regulator, record with `retrieved_at`, and have reviewed by a qualified reviewer. Per the register's
`uncertainty_rule`, the eight questions in §2 are `HOLD`, not defaults — and the three NOT ESTABLISHED cells
in §1 are the reason release-gate condition 1 cannot be satisfied for 기준소득월액, 산재보험료율 and 최저임금
by anything in this document alone.
