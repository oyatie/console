> **QUARRY / NON-AUTHORITY.** Idea or draft only. Cannot dispatch work, clear HOLDs, or override product scope. Current authority: repository README + [`docs/current/PRODUCT.md`](../current/PRODUCT.md) / ROADMAP / DELIVERY.

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

Four numbers are set by **고시**, and no 고시 is in the archive. Those four are the whole reason this
register needs a second source route.

## Correction, 2026-07-30 — the 고시 are reachable, and one is now resolved

**What the first version of this document said, and got wrong:** *"no 고시 is … resolvable by name on the
public portal"*, and in §6, that five candidate names *"returned the error page"* — presented as evidence
about the documents. It was evidence about **my names**. Corrected, with the original claim left above so the
change is visible:

- The 고시 **are** indexed, under `target=admrul` — a different target from the `target=law` the earlier probes
  assumed. Hit counts, from searches supplied to me rather than run here: 기준소득월액 → 5, 산재보험료율 → 2,
  최저임금 → 6.
- Given the **exact canonical 행정규칙명**, the public `law.go.kr/행정규칙/{name}` URL resolves
  **credential-free**. Confirmed for the 국민연금 고시: title echoes back, 1,273 B against the 1,196 B error
  control.
- So the API key is needed for **discovery** — learning the exact name — **not for citation, and not for the
  text.** Once the 행정규칙일련번호 is known, `law.go.kr/LSW/admRulInfoR.do?admRulSeq=<seq>` serves the full
  고시 body with no credential at all. That is how item 2 below was read.

**Second correction, same day — the claim was not half-wrong, it was wrong.** The first pass of this
correction said Q3 and Q4 "remain open" because no `LAW_OC` was available here. Both are now **located**:
고용노동부고시 **제2025-91호** (Q3) and **제2025-47호** (Q4), searched by someone with a key and then verified
against the portal from their bodies' own header lines. So of the four 고시 this document originally called
unlocatable, **three are named, and two have been read**:

| 고시 | Status | Figures checked against the kernel? |
|---|---|---|
| 기준소득월액 하한액과 상한액 — 보건복지부고시 제2026-31호 | **Located, body read** | **Yes — match** (§1) |
| 2026년도 사업종류별 산재보험료율 — 고용노동부고시 제2025-91호 | **Located, body read** | N/A — rates live in its 별지; employer-only rests on 징수법 제13조제5항, not on the 고시 (Q3) |
| 2026년 적용 최저임금 고시 — 고용노동부고시 제2025-47호 | **Located; text reached via its PDF attachment, not the record** | **Two of three — match.** `daily_8h_won` is a derivation, not a 고시 figure. Read by hand-decoded CMap: discovery, not evidence (§1, Q4) |
| 기준소득월액, 2025-07 window — 보건복지부고시 제2025-24호 | Number known from 부칙 history only | **No** — portal serves consolidated current text only |

**The residue is smaller and differently shaped than "we need a key".** Discovery needed the key and is now
done. What is left is two *reading* problems — one document served as images, one superseded version not
served at all — plus the standing rule that none of this is an `allowed_source` (§6).

## 1. The fetch list

`공포번호` and `시행일자` are the **file version** whose text was read, taken verbatim from the archive's YAML
frontmatter. Where a provision's own last amendment differs from the file version, both are given — the
provision date is what a reviewer needs to diff against.

| # | Statutory item | Authorising act | Instrument that sets the number | 공포번호 | 시행일자 | Official URL to cite | Annually re-published? | Confidence |
|---|---|---|---|---|---|---|---|---|
| 1 | **국민연금** employee share — 4.75% (2026) | 국민연금법 제88조제3항 | **The act itself.** 제88조제3항 reads 기준소득월액의 1천분의 65; for 2026 it is displaced by **부칙 <법률 제20903호, 2025.4.2> 제4조제1항제1호: 1만분의 475** | operative 부칙: **법률 제20903호** (공포 2025-04-02). File read: 법률 제21689호 | 부칙 제1조: **2026-01-01**. File: 2026-05-26 (in force) | <https://www.law.go.kr/법령/국민연금법> | **No — legislated schedule.** 2026 4.75%, 2027 5.00%, 2028 5.25%, 2029 5.50%, 2030 5.75%, 2031 6.00%, 2032 6.25%, then 제88조제3항's 6.5% | **HIGH** — quoted verbatim from a file whose 시행일자 is in force |
| 2 | **국민연금 기준소득월액** cap and floor | 국민연금법 제3조제1항제5호 | Split: **국민연금법 시행령 제5조** sets the indexation formula and orders publication; the amounts are set by **「국민연금 기준소득월액 하한액과 상한액」, 보건복지부고시 제2026-31호** (일부개정), issued by 3월 31일 each year per 제5조제3항, applying **해당 연도 7월 ~ 다음 연도 6월** per 제5조제4항 | 시행령: **대통령령 제35909호** (2025-12-16). 고시: **발령번호 제2026-31호**, 발령일자 **2026-02-02**, 소관 보건복지부(국민연금정책과) | 시행령: 2025-12-16 (in force). **고시 시행일자 2026-07-01** | Decree: <https://www.law.go.kr/법령/국민연금법시행령> · 고시: <https://www.law.go.kr/행정규칙/국민연금%20기준소득월액%20하한액과%20상한액> · body (no credential): `law.go.kr/LSW/admRulInfoR.do?admRulSeq=2100000274228` | **YES — annual 고시 by 3월 31일.** Its own 부칙 history is one amendment per year, 제2012-31호 through 제2026-31호 — 15 consecutive years | **HIGH** — 고시 named, fetched and read; amounts compared against the kernel below |
| 3 | **건강보험** employee share — 3.595% | 국민건강보험법 제73조제1항 (1천분의 80 범위, 심의위원회 의결 + 대통령령) | **국민건강보험법 시행령 제44조제1항: 1만분의 719** (= 7.19%). The 50/50 split is in the act — **국민건강보험법 제76조제1항: 100분의 50씩** | 시행령: **대통령령 제36116호** (2026-02-19); 제44조 last amended **2025.12.23**. Act file read: 법률 제21687호 | 시행령: 2026-02-19 (in force). **Act file 시행일자 2027-01-01 — not yet in force** | <https://www.law.go.kr/법령/국민건강보험법시행령> · <https://www.law.go.kr/법령/국민건강보험법> | **Effectively yes.** 제44조's 개정 list is 2012·2013·2014·2015·2017·2018·2019·2020·2021·2022·2024·2025 — almost always December | **HIGH** for 7.19%. **MEDIUM** for the 50/50 split — read from a future-effective act file (see Q5) |
| 4 | **장기요양보험** employee share — 0.4724% | 노인장기요양보험법 제9조제2항 (장기요양위원회 심의 + 대통령령) | **노인장기요양보험법 시행령 제4조: 100만분의 9,448** (= 0.9448%). Bearer split arrives **by 준용** — 노인장기요양보험법 제11조 준용s 국민건강보험법 제76조 | 시행령: **대통령령 제36325호** (2026-05-12); 제4조 last amended **2025.12.30**. Act file read: 법률 제21690호 | 시행령: 2026-05-12 (in force). **Act file 시행일자 2026-11-27 — not yet in force** | <https://www.law.go.kr/법령/노인장기요양보험법시행령> · <https://www.law.go.kr/법령/노인장기요양보험법> | **Effectively yes.** 제4조 amended 2008·2009·2017·2018·2019·2020·2021·2022·2023·2025 | **HIGH** for 0.9448%. **MEDIUM-LOW** for modelling it as 0.4724% × 보수월액 (see Q1); **MEDIUM** for the split — a 준용 chain across two future-effective files |
| 5 | **고용보험 실업급여** employee share — 0.9% | 징수법 제14조제1항 (1000분의 30 범위, 고용보험위원회 심의 + 대통령령). Employee half: 징수법 제13조제2항 | **징수법 시행령 제12조제1항제2호: 1천분의 18** (= 1.8%); employee bears ½ | 시행령: **대통령령 제35935호** (2025-12-23); 제12조 last amended **2023.12.26**. Act file read: 법률 제21532호 | 시행령: 2025-12-23 (in force). **Act file 시행일자 2026-10-08 — not yet in force** | <https://www.law.go.kr/법령/고용보험및산업재해보상보험의보험료징수등에관한법률시행령> | **No fixed cadence.** 제12조 amended 2011·2013·2019·2021·2023 — irregular, so it expires on change, not on a calendar | **HIGH** for 1.8%. **MEDIUM** for the ½ share — 제13조제2항 carries `<개정 2026.3.17>` and sits in a future-effective file |
| 6 | **산재보험** — employer-only, industry tariff | 징수법 제14조제3항→제6항 (→ 고용노동부령); employer-only per 징수법 제13조제5항 | **Two-layer delegation.** 징수법 시행규칙 제12조: rates are *"고용노동부장관이 정하여 고시"*; the 시행규칙's **별표 1** supplies only 구성과 산정방법. Then 제13조 개별실적요율 experience-rates the result **per employer** | 시행규칙: **고용노동부령 제473호** (2026-07-01); 제12조 last amended 2017.12.28. 고시: **고용노동부고시 제2025-91호**, 발령 **2025-12-31**, 소관 고용노동부(산재보상정책과) | 시행규칙 2026-07-01 (in force). **고시 시행 2026-01-01, and it expires by its own terms 2026-12-31** | 시행규칙: <https://www.law.go.kr/법령/고용보험및산업재해보상보험의보험료징수등에관한법률시행규칙> · 별표 1 (HWP): <https://www.law.go.kr/LSW/flDownload.do?flSeq=166487111> · 고시: <https://www.law.go.kr/행정규칙/2026년도%20사업종류별%20산재보험료율> · body `admRulInfoR.do?admRulSeq=2100000271450` · **the rate table itself is the 고시's 별지**: PDF <https://www.law.go.kr/LSW/flDownload.do?flSeq=160837293> (114,447 B), HWPX <https://www.law.go.kr/LSW/flDownload.do?flSeq=160837289> | **YES — per 보험연도, with an explicit 유효기간 ending 2026-12-31.** 제14조제6항 caps year-on-year movement at ±30% | **HIGH** — 고시 named, fetched and read. Employee share still rests on 징수법 제13조제5항, not on the 고시, which is silent on who bears it |
| 7 | **최저임금** — 2026 guard data | 최저임금법 제10조 | **고용노동부장관 고시.** 제10조제1항 requires 고시; 제10조제2항 makes it effective **1 January of the following year**. 최저임금법 시행령 is 대통령령 제29469호 of **2018-12-31** and does not carry the amount | **고용노동부고시 제2025-47호** (제정), 발령 **2025-08-05**, 소관 고용노동부(근로기준정책과). Act file read: 법률 제21534호 | 고시 **시행 2026-01-01**, matching 제10조제2항. **Act file 시행일자 2026-12-08 — not yet in force** | 고시: <https://www.law.go.kr/행정규칙/2026년%20적용%20최저임금%20고시> · **the operative text is the attachment, not the record**: PDF <https://www.law.go.kr/flDownload.do?flSeq=155278071> (84,498 B, 1 page, verified 200 `application/pdf`). Act: <https://www.law.go.kr/법령/최저임금법> | **YES — every year**, and issued ~5 months ahead of effect | **HIGH** that this is the instrument. Two of three kernel figures **read and matching**; the third is a derivation, not a 고시 figure — and the read came from a hand-decoded PDF, so it is discovery, not evidence (Q4) |
| 8 | **소득세** — 근로소득 간이세액표 | 소득세법 제129조제3항 (*"대통령령으로 정하는 근로소득 간이세액표"*) | **소득세법 시행령 별표 2**, per 시행령 제189조제1항. A **대통령령 별표 — not an NTS 고시.** Confirmed fetchable: 172,032 bytes, `application/hwp`, HTTP 200, no auth | **대통령령 제36343호** (공포 2026-05-22) | **2026-07-01** (in force) — note this is *after* 공포 (Q8) | Decree: <https://www.law.go.kr/법령/소득세법시행령> · 별표 2 HWP: <https://www.law.go.kr/LSW/flDownload.do?flSeq=164391981> · PDF: <https://www.law.go.kr/LSW/flDownload.do?flSeq=164391983> | **No annual notice.** It changes when the 별표 is amended | **HIGH** — delegation chain quoted verbatim end to end and the file downloads |
| 9 | **지방소득세** — 근로소득 특별징수 | 지방세법 제103조의13제1항 | **The act itself** — *"원천징수하는 소득세… 의 100분의 10"*, withheld simultaneously with 소득세 | **법률 제21308호** (2025-12-31) | **2026-01-01** (in force) | <https://www.law.go.kr/법령/지방세법> | **No** | **HIGH** |

### The 고시 text against the kernel constant — 기준소득월액

The only item on this list whose 고시 I could read. Quoted in full because it is four lines, and stated as a
factual comparison of two figures — not as a conclusion that either is legally correct.

**보건복지부고시 제2026-31호** `[시행 2026. 7. 1.] [보건복지부고시 제2026-31호, 2026. 2. 2., 일부개정]`,
보건복지부(국민연금정책과), retrieved 2026-07-30 from `admRulInfoR.do?admRulSeq=2100000274228`:

> 1. 국민연금 기준소득월액 — 가. 하한액 : **410천원** / 나. 상한액 : **6,590천원**
> 2. 적용기간 : **2026년도 7월분부터 2027년도 6월분까지**

`backend/crates/payroll/domain/src/lib.rs:575-580`, second `MonthlyBaseLimit`:
period `2026-07-01 .. 2027-07-01`, `minimum_won: 410_000`, `maximum_won: 6_590_000`.

**They match.** 410천원 = 410,000원 and 6,590천원 = 6,590,000원; and the 고시's 적용기간 — July 2026 through
June 2027 — is the same span as the kernel's half-open `2026-07-01 .. 2027-07-01`. The kernel's cited
`authority`/`url` for this row is an NPS page; the **instrument** is this 고시, so the citation can now be
upgraded even though the numbers need no change.

**The kernel's other row is not verified.** `lib.rs:570-573` holds `2025-07-01 .. 2026-07-01` with
400,000 / 6,370,000. The 부칙 history dates the governing instrument for that window to
**보건복지부고시 제2025-24호**, but the portal serves only the consolidated current text, so **I did not read
its amounts.** That row's figures remain unverified here — matching the spec is not the same as matching the
고시. It needs the same treatment once 제2025-24호's text is in hand.

### The 고시 text against the kernel constant — 최저임금

Second comparison, and it needs its caveat read first because the caveat is load-bearing.

**How the text was obtained.** `admRulInfoR.do` gave 101 characters, all header — the 조문내용 is **empty
despite 조문형식여부 = Y**, which is why the first pass here concluded the body was unreadable. The general
route past that: **`lawService.do?target=admrul&ID=<행정규칙일련번호>&type=XML` returns a `<첨부파일>` block**,
and for 제2025-47호 it names `2026년 적용 최저임금 고시(고용노동부 고시 제2025-47호).pdf` at
`flDownload.do?flSeq=155278071` — verified here as **HTTP 200, `application/pdf`, 84,498 bytes, 1 page**,
credential-free. (`flSeq=155278071` is the same id as the viewer's `key` parameter, which independently
confirms it is this 고시's attachment. A second attachment, 최저임금 고시 재개정 이유서, is at `…flSeq=155278073`,
88,968 B.) **This is the technique to reach for whenever a 고시's 조문내용 is empty** — the same shape as the
산재 별지 in item 6, so for 고시 the instrument's text is frequently the attachment rather than the record.

**How the PDF was read, and why that limits what follows.** No `pdftotext` or poppler is available in this
environment (confirmed), and the PDF uses subset fonts with glyph-index text, so it was decoded by extracting
the embedded ToUnicode CMaps and mapping content-stream runs by hand. **The reading order comes out
scrambled** — font switches interleave the runs. So only exact numeric strings are treated as evidence here,
and the surrounding prose is not quoted at all.

On that basis, against `minimum_wage_rates()` at `backend/crates/payroll/domain/src/lib.rs:585-593`:

| Kernel field | Kernel value | In the 고시 PDF? |
|---|---|---|
| `hourly_won` | 10,320 | **Appears exactly once — matches** |
| `monthly_209h_won` | 2,156,880 | **Appears exactly once — matches** |
| `daily_8h_won` | 82,560 | **Zero occurrences.** Not a 고시 figure — it is 10,320 × 8 |
| (209-hour basis) | — | Not cleanly recoverable; the digit stream is scrambled around it, so nothing is claimed |

**The `daily_8h_won` finding is an attribution nuance, not a defect.** The field name discloses the
derivation, and 82,560 is arithmetically 10,320 × 8. What the register records is that **all three fields
share one `source`, and that source supplies two of them.** A reviewer signing this row should know the daily
figure is ours, not the Minister's.

**And this does not make the amount established.** It is an agent's read of an official PDF through a
scrambled extractor. It raises confidence that the kernel is not wrong; it is not the evidence release-gate
condition 1 asks for. **A licensed reviewer must open the PDF themselves** before this row is relied on — the
URL and byte size above are there so they can confirm they have the same file.

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

**Q2 — which document is the 기준소득월액 상한액·하한액 고시? — ANSWERED 2026-07-30.**
*Original question, left as asked:* 국민연금법 시행령 제5조제3항 requires 보건복지부장관 to 고시 the amounts by
3월 31일 annually. I could not locate that 고시 as a document: absent from the archive, and not resolvable by
name on the public portal. **Question:** what is the 고시's official title, number and citable URL, and is
the NPS page a reproduction of it or an independent publication?

**What answered it:** a `target=admrul` search of the official API — the target the earlier probes did not try
— returning 「**국민연금 기준소득월액 하한액과 상한액**」, 종류 **고시**, 소관부처 **보건복지부**, 발령일자
**2026-02-02**, 발령번호 **제2026-31호**, 시행일자 **2026-07-01**, 행정규칙ID 2035959, 행정규칙일련번호
2100000274228, 현행연혁구분 현행.

**Provenance, stated exactly because it matters.** That search row was **supplied to me, not run by me** — I
had no `LAW_OC`. So I treated it as a lead and confirmed it against the portal directly. Independently
verified, credential-free: the 행정규칙명 (echoed in the page `<title>`), the 행정규칙일련번호 (appears as
`admRulSeq=2100000274228` in the portal's own frameset `src`), and — from the 고시 body's own header line —
`[시행 2026. 7. 1.] [보건복지부고시 제2026-31호, 2026. 2. 2., 일부개정]`, which confirms 종류, 소관부처,
발령번호, 발령일자 and 시행일자 from the document itself. **Only 행정규칙ID 2035959 and 현행연혁구분 rest on
the supplied row alone.** Nothing load-bearing here depends on an unverified hand-off.

Two things the answer settled beyond what was asked. **The July–June window is confirmed by the document
itself** — 적용기간 2026년 7월분 ~ 2027년 6월분 — so it is no longer only an inference from 시행령 제5조제4항.
And **발령 2026-02-02 is before the 3월 31일 deadline** 제5조제3항 sets, so the statutory cadence and the
document's actual behaviour agree.

**Still open, narrowly:** whether the NPS page the spec cites reproduces this 고시 or publishes
independently. Nothing here establishes that, and the 고시 is the better citation either way.

**Q3 — which document is the 사업종류별 산재보험료율 고시? — STILL OPEN, and now cheap to close.**
징수법 시행규칙 제12조 delegates the actual rates to a 고용노동부장관 고시. **Question:** the 고시 title/number
for 보험연도 2026, and separately — since 제13조 개별실적요율 experience-rates per employer — **is any single
산재 rate ever correct for a tenant, or must it always be a per-employer input?** The spec already treats
산재 as "industry-tariff-required"; this question asks whether "industry" is even the right granularity.

**ANSWERED 2026-07-30.** 「**2026년도 사업종류별 산재보험료율**」, 종류 **고시**, 소관 **고용노동부(산재보상정책과)**,
발령일자 **2025-12-31**, 발령번호 **제2025-91호**, 시행일자 **2026-01-01**, 일부개정, 현행,
행정규칙일련번호 2100000271450. Search row supplied; the name, number, dates and 소관부처 are all confirmed
from the body's own header line, read credential-free.

**What the body says**, quoted because three of its four operative lines matter:

> Ⅰ. 1. 2026년도 "사업종류별 산재보험료율"은 **별지와 같다.**
> Ⅰ. 2. 2026년도 "통상적인 경로와 방법으로 출퇴근하는 중 발생한 재해에 관한 산재보험료율"은 **사업의 종류를
> 구분하지 아니하고 0.6/1,000로 한다.**
> Ⅱ. 2. 유효기간 — 이 고시는 **2026년 12월 31일까지** 효력을 가진다.

Three findings, in order of consequence:

1. **The 고시 does not set the industry rates — its 별지 does.** So the fetch target is the attachment, not
   the 고시 page: PDF `flDownload.do?flSeq=160837293` (114,447 B) or HWPX `…flSeq=160837289` (12,578 B), both
   HTTP 200 credential-free. And **사업종류 selection is therefore a per-entity input keyed to that
   classification table** — which answers the granularity half of the original question in the direction the
   spec already assumed, without settling the 개별실적요율 part.
2. **An explicit sunset.** 유효기간 2026-12-31 is a hard expiry written into the instrument, not an inferred
   annual cadence. See §5 — this is the only item on the list that expires by its own terms.
3. **The 고시 is silent on who bears the premium.** It sets rates and nothing else. The employer-only rule
   stays where §1 already puts it — 징수법 제13조제5항 — so the kernel's `employee_ppm: Some(0)` is
   **not** corroborated by this document, and does not need to be: it never rested on it.

**One trap this search surfaced.** A sibling instrument exists with the same 발령·시행 dates and the adjacent
number: 「**노무제공자 직종별 산재보험료율**」, 고용노동부고시 **제2025-92호**, seq 2100000271454 — confirmed by
fetching its header. That one is **직종별 for 노무제공자**; 제2025-91호 is **사업종류별**. A name-match on
"산재보험료율" hits both. **Which applies to a given worker is a scope question, not a lookup**, and this
document does not decide it.

**And the replacement for a dead anchor.** `backend/crates/payroll/domain/src/lib.rs:555-560` cites
`https://total.comwel.or.kr/` as the 산재보험 source. **That URL answers HTTP 400** — verified 2026-07-30, as
does `https://www.comwel.or.kr/`. It is an agency root, not an instrument, and it is dead. **Its correct
replacement is 고용노동부고시 제2025-91호** — a named instrument with a stable credential-free URL and a
downloadable rate table. Recorded here because the kernel edit is not mine to make.

**Q4 — which document is the 2026 최저임금 고시, and is the 최저임금위원회 table the instrument? — STILL OPEN,
and now cheap to close.**
최저임금법 제10조 makes the 고용노동부장관 고시 the operative instrument. The spec cites
`minimumwage.go.kr`, which is the **Commission** — a 심의 body whose 최저임금안 goes to the Minister under
제8조·제9조. **Question:** confirm the 고시 is the instrument and the Commission table is a summary, and
supply the 고시's citable identifier. Also confirm whether 제10조제2항's proviso (the Minister may set a
different effective date per business type) is inactive for 2026.

**INSTRUMENT ANSWERED 2026-07-30; ITS FIGURES ARE NOT.** 「**2026년 적용 최저임금 고시**」, 종류 **고시**,
소관 **고용노동부(근로기준정책과)**, 발령일자 **2025-08-05**, 발령번호 **제2025-47호**, **제정**, 시행일자
**2026-01-01**, 현행, 행정규칙일련번호 2100000262710. Name, number, dates and 소관부처 confirmed from the
body's own header line. 시행 2026-01-01 matches 최저임금법 제10조제2항 exactly, and 발령 2025-08-05 puts issue
~5 months ahead of effect.

**Its figures are now read — via the attachment, and with a caveat.** The first pass here concluded the body
was unreadable: `admRulInfoR.do` returns 101 characters, all header, because the **조문내용 is empty despite
조문형식여부 = Y**, and probes at the Synap viewer's data paths returned the JS shell or 404. That was right
about the HTML and wrong to stop there. The API's detail endpoint —
**`lawService.do?target=admrul&ID=2100000262710&type=XML`** — exposes a `<첨부파일>` block naming the
고시 PDF at `flDownload.do?flSeq=155278071` (**84,498 B, 1 page, HTTP 200 `application/pdf`**, verified here,
credential-free). §1 carries the comparison, the extraction caveat and the general technique.

**Result in one line:** `hourly_won` 10,320 and `monthly_209h_won` 2,156,880 both appear in the 고시 and match
the kernel; `daily_8h_won` 82,560 **does not appear** and is a 10,320 × 8 derivation the field name already
discloses. The 209-hour basis was not cleanly recoverable and nothing is claimed about it. The read came from
a hand-decoded CMap with scrambled reading order, so **it is discovery, not evidence** — see §1.

Also from that detail response, recorded for the register's fields: 담당부서 고용노동부(근로기준정책과),
제개정구분 **제정**, 현행여부 **Y**, 시행일자 **20260101**.

**Two forward-dated traps this search surfaced**, both of which a name-match would walk into:

- 「**2026년 선원 최저임금 고시**」 — 해양수산부, 제2025-200호, 발령 2025-12-08, 시행 2026-01-01. A separate
  instrument for **선원**. Not the general minimum wage; do not cite it.
- 「**2027년 적용 최저임금안 고시**」 — 제2026-55호, 발령 2026-07-16, 시행 2027-01-01, and marked **연혁, not
  현행**. It is an **안** — a proposal under 제9조, not a 결정 under 제10조. It exists *today* and is
  forward-dated, so a "latest match" search finds it and a careless fetch would pin the rate table to a
  proposal. It must not displace 제2025-47호. This is precisely the shape `change_rule` exists to catch, and
  it is also Q5 in miniature: newest ≠ operative.

**Still open:** whether 제10조제2항's proviso (different effect dates per business type) is inactive for 2026.
The 고시 body would answer it; see above for why it is unread.

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

**And the base URL is no longer unknown either** — `scripts/korean-legal/fetch-statutory-source.mjs` was
built and live-tested after the first version of this document, and its header records the two things that
cost the most to learn: **the API host is `www.law.go.kr/DRF/`, not `open.law.go.kr`** (the latter is the
registration portal and 404s on the DRF path), and **the API echoes the OC back inside every
법령상세링크/행정규칙상세링크**, so raw responses contain the credential and must never be cached or committed.
The 고시 layer is reached by the same `lawSearch.do` with **`target=admrul`** instead of `target=law`, whose
row fields are 행정규칙명 / 행정규칙종류 / 발령일자 / 발령번호 / 시행일자 / 행정규칙ID / 소관부처명, with detail
at `lawService.do?target=admrul&ID=<행정규칙일련번호>`.

**Two operational facts worth having before the next attempt.** First, the `OC` is **bound to registered
IP/domain**, not just to an account: a rejected call answers with *"사용자 정보 검증에 실패하였습니다 …
서버장비의 IP주소 및 도메인주소를 등록해 주세요"*, so a key that works from one machine can refuse from
another. Second — and this is why §6's "not found" results deserved less trust than they got — **that refusal
arrives as HTTP 200 with no `resultCode`**, so it used to parse to zero rows and print
`NO EXACT MATCH — do not guess; refine the name`. An API refusal read exactly like an absent document. Guarded
now, in the same script, with the refusal surfaced as an error instead.

**What needs no key at all**, confirmed 2026-07-30: citing a 고시 (`law.go.kr/행정규칙/{exact name}`) and
**reading its full text** (`law.go.kr/LSW/admRulInfoR.do?admRulSeq=<행정규칙일련번호>`). Item 2 was read that
way. So the key buys **discovery** — the exact name and 일련번호 — and nothing downstream of it.

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
| **기준소득월액 상한액·하한액** (item 2) — **located 2026-07-30** | 보건복지부고시 제2026-31호, annually by 3월 31일 | **Resolved:** law.go.kr 행정규칙. Discovery needed the OC key (`target=admrul`); citation and body did not. Still open: whether the NPS page reproduces it (Q2) |
| **사업종류별 산재보험료율** (item 6) — **located 2026-07-30** | 고용노동부고시 제2025-91호, per 보험연도, 유효기간 to 2026-12-31 | **Resolved:** law.go.kr 행정규칙, body read. The rates are in the 고시's **별지**, downloadable credential-free (§1) — the 시행규칙 별표 1 gives only the calculation method. Replaces the dead `total.comwel.or.kr` anchor (Q3) |
| **2026 최저임금** (item 7) — **located 2026-07-30** | 고용노동부고시 제2025-47호 | **Instrument resolved**, body **not readable as text** — viewer-rendered attachment (Q4). 최저임금위원회 remains a 심의 body: corroboration, not the instrument |
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
| **Every 3월 31일** | 기준소득월액 상한액·하한액 고시 (item 2) | New effective-date → dependent evidence invalid. **No longer an inference:** the 고시's own 부칙 history runs 제2012-31호, 제2013-47호 … 제2024-6호, 제2025-24호, 제2026-31호 — **15 consecutive annual amendments.** 제2026-31호 was 발령 2026-02-02 for effect 2026-07-01, so issue and effect sit ~5 months apart |
| **Every 1 July** | The July→June cap/floor window turns over (item 2) | An effective-date change even in a year with no rate change |
| **Every 1 January, 2027 through 2033** | 국민연금 employee share steps 5.00 → 5.25 → 5.50 → 5.75 → 6.00 → 6.25 → 6.5% (item 1) | Seven scheduled invalidations. **Already knowable and citable today** — the only expiry on this list that can be pre-empted rather than watched |
| **Every 1 January** | 최저임금 고시 takes effect (item 7) | 최저임금법 제10조제2항. New source document each year |
| **2026-12-31, by the instrument's own terms** | 사업종류별 산재보험료율 고시 (item 6) | **Not an inferred cadence — a written sunset:** 제2025-91호 Ⅱ.2 states *"이 고시는 2026년 12월 31일까지 효력을 가진다."* The only item here that expires by its own text, so the expiry date needs no watching, only honouring. Plus per-employer 개별실적요율, which can change without any 고시 change at all |
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

- **No `LAW_OC` value in this session** — neither in the environment nor, correctly, in the tree. Cost: could
  not run the `target=admrul` search that yields the exact 행정규칙명 for items 6 and 7. This is the *only*
  reason discovery could not be done here. Items 6 and 7 were resolved by someone who had one; I verified
  both against the portal rather than taking the hand-off on trust.
- **~~No 고시 located for items 2, 6, 7~~ — superseded twice, see Correction.** All three are now named, and
  two are read. The original claim — *"not resolvable by name through the `/행정규칙/{name}` URL pattern"* —
  was wrong about the pattern: it resolves fine given the exact canonical name. What the failed probes
  actually showed is that **the name cannot be guessed** — 5 candidates in the first pass and 12 in the
  second all returned the error page, and the true names (`2026년도 사업종류별 산재보험료율`,
  `2026년 적용 최저임금 고시`) were not among them. **I guessed no 고시 title, number or URL at any point.**
- **~~Item 7's body is not extractable as text~~ — superseded.** True of the HTML record, false of the
  instrument: its 조문내용 is empty and the text lives in a PDF attachment the API's `<첨부파일>` block names
  (§1). Two of `minimum_wage_rates()`'s three constants are now read and matching. **But the read used a
  hand-built ToUnicode CMap decode with scrambled reading order**, so only exact numeric strings were taken as
  evidence and no prose from that PDF is quoted anywhere in this document. A licensed reviewer must open it.
- **Still unparsed: the 간이세액표 별표 2 HWP and the 산재 별지.** Both located, both reachable, contents not
  read. Unlike item 7 nobody has attempted a decode, so nothing about their contents is claimed at all.
- **Added 2026-07-30 — how the one 고시 was read, credential-free and repeatable.** Given the exact
  행정규칙명: `GET law.go.kr/행정규칙/{name}` → confirm via `<title>` → extract the frameset's
  `src="/LSW//admRulInfoP.do?admRulSeq=…"`, whose `admRulSeq` **is** the 행정규칙일련번호 → then
  `GET law.go.kr/LSW/admRulInfoR.do?admRulSeq=<seq>`, which returns the full body (13,983 B for item 2). The
  outer `admRulInfoP.do` page is chrome only and contains none of the text; `admRulInfoR.do` is the one that
  carries it. **So the 일련번호 alone unlocks any 고시 body without a key.**
- **Could not reach the 고시 for the kernel's earlier 기준소득월액 row.** The portal serves consolidated
  current text, so 제2025-24호's amounts (kernel `400_000` / `6_370_000`) are **unverified** — only its
  existence and number are established, from the 부칙 history.
- **Did not extend the fetch script to `admrul`.** Deliberate: with no key I could not make a single live
  call, and this repo's own rule — recorded in that script's header and in
  `docs/ideas/korean-legal-sources.md` — is that a fetcher gets built and verified against a real call in the
  same pass. Shipping an untested admrul path would be the exact defect that rule exists to prevent. What I
  did add is a guard for a defect I *could* reproduce: a refused OC returned HTTP 200 with no `resultCode`
  and printed `NO EXACT MATCH — do not guess; refine the name`, making an API refusal indistinguishable from
  an absent document. Verified before and after.
- **Did not open the 별표 files.** I confirmed the 간이세액표 link serves 172 KB of `application/hwp` with
  HTTP 200; I did not parse the HWP, so **nothing in this register asserts anything about the table's
  contents** — only about where it lives and that it is reachable.
- **Still have not verified law.go.kr against the archive, item by item.** Every 공포번호 and 시행일자 in §1
  comes from the archive's frontmatter — derived from the official OpenAPI, but a third-party copy. Each must
  be re-verified against law.go.kr before it becomes evidence. The earlier version blamed the frame-loaded
  page body; that excuse is gone, since the frame's inner `lsInfoP.do` URL is extractable the same way
  `admRulInfoR.do` was. It simply was not done for all 27, and remains outstanding.
- **Six sampled files are future-effective** (Q5). Items 3, 4 and 5 inherit MEDIUM confidence from this.
  **Corroborated 2026-07-30 from the portal's own choice of version:** for 최저임금법 the public page loads
  `lsInfoP.do?lsiSeq=218303&efYd=20200526` — the text **in force** — while the archive's `main` file for the
  same law is 법률 제21534호 with 시행일자 **2026-12-08**. The two disagree about what "the current 최저임금법"
  means, and for a pay date the portal's answer is the relevant one. That is Q5 restated as an observation
  rather than a worry.
- **No commit hash anywhere in this document.** The archive's README warns force-push may rewrite every
  hash; anchoring is on 공포번호 + 시행일자 + the law.go.kr URL, per `docs/ideas/korean-legal-sources.md`.

**What this document is not.** It is a fetch list. Nothing here is an `allowed_source`; every citation is a
third-party-derived pointer at a document someone must still retrieve from law.go.kr or the issuing
regulator, record with `retrieved_at`, and have reviewed by a qualified reviewer. Per the register's
`uncertainty_rule`, the eight questions in §2 are `HOLD`, not defaults.

**Being named is not being verified, and being read is not being evidenced.** Three 고시 are named and all
three read — real progress on *discovery*, none at all on *authority*. Two figure comparisons come out
matching: 기준소득월액 (clean HTML text) and 최저임금's hourly and monthly (hand-decoded PDF, caveat in §1).
Neither is evidence in the register's sense: one is an agent reading a web page, the other an agent reading a
PDF with a scrambled extractor.

Still unread: **the 산재 별지 rate table, the 간이세액표 별표 2, and 기준소득월액 제2025-24호 for the 2025-07
window** — the last of which stays blocked because the portal serves only consolidated current text, and is
recorded as unverified rather than inferred from the spec. And one kernel figure is ours rather than an
instrument's: `daily_8h_won`.

A qualified reviewer still has to retrieve each source, record it with `retrieved_at`, and sign it. Nothing
here moves any control off `HOLD`.
