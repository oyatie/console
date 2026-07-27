# Wave-4 Lens D — Korean statutory parameters, LIVE-verified

**Accessed: 2026-07-25** (every URL below fetched/searched on this date)
**Method:** WebSearch + WebFetch against 국가법령정보센터 (law.go.kr), 고용노동부 (moel.go.kr), 보건복지부 (mohw.go.kr), 국민연금공단 (nps.or.kr), 국민건강보험공단 (nhis.or.kr), 국세청 (nts.go.kr), 법제처 찾기쉬운 생활법령정보 (easylaw.go.kr).
**Not legal advice.** This is an engineering parameter register. `docs/specs/payroll.md` §"Release gate" still binds: production payroll stays disabled until a licensed 노무사/세무사 signs golden cases.

## 0. What already existed (do not re-derive)

| Prior artifact | What it already established |
|---|---|
| `/Users/jasonlee/Developer/maintenance/docs/specs/payroll.md` (lines 23–45) | Source-of-truth order, 2026-06-27 checked rates (최저임금 10,320 / 국민연금 9.5% / 건강 7.19% / 장기요양 0.9448% / 고용 실업급여 0.9% / 산재 employer-only), effective-dated rate-table design, golden-case + professional-validation release gate. **This wave re-verified all of them live; all held.** |
| `/Users/jasonlee/Developer/maintenance/docs/specs/korean-legal-boundaries.md` | PIPA/근로기준법/위치정보법/퇴직급여법 source anchors (2026-06-28), 12 product guardrails, intra-group employment-episode model. Lens D inherits guardrails 1, 3, 4, 5, 8 verbatim — do not restate them in lane charters, cite them. |
| `/Users/jasonlee/Developer/maintenance/docs/specs/hr-payroll-readiness.md`, `hr-core.md` | Readiness gating for HR/payroll surfaces. |
| `.../pr488-design-mirror-sync/docs/program/benchmark-matrix/{people,leave,compliance}.md` | Vendor benchmark lenses for people/leave/compliance modules (data, not statutory). |

**Delta this wave adds:** working-time (주52 구성, 가산율 + overlap), 휴게, 주휴수당 conditions, 연차 accrual + 촉진 timings, 간이세액표 consumption mechanism, 채용절차법 timings, PIPA talent-pool retention — none of which existed in the prior specs.

---

## 1. 최저임금 (Minimum wage) — 2026

**Rule.** 최저임금액에 미치지 못하는 금액을 임금으로 정한 부분은 무효이고, 무효 부분은 최저임금액과 동일한 임금을 지급하기로 한 것으로 본다 (최저임금법 제6조제3항).

| Parameter | Value | Source | Effective |
|---|---|---|---|
| 시간급 | **10,320원** | https://www.moel.go.kr/news/enews/report/enewsView.do?news_seq=18144 (고시 2025-08-05) | 2026-01-01 |
| 일급 (8h) | **82,560원** | same | 2026-01-01 |
| 월 환산액 | **2,156,880원** | same | 2026-01-01 |
| 월 환산 기준시간 | **209시간** (주40h 소정 + 주8h 유급주휴 = 48h × 4.345주) | same + https://www.moel.go.kr/news/enews/report/enewsView.do?news_seq=18054 | — |
| 인상폭 | +290원 (+2.9%) vs 2025 (10,030원) | same | — |

**Landing:** payroll (minimum-wage guard on every draft payslip: 통상시급 vs 10,320; monthly-salaried compare against 2,156,880 at 209h), attendance (209h divisor is the same constant the 통상시급 derivation uses — one shared constant, effective-dated, not two).

**needs-verification:** 산입범위. 최저임금법 제6조제4항 statute text still carries the *pre-phase-down* limits (상여금 25% 한도, 복리후생비 7% 한도) — https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=최저임금법&joNo=000600000&mode=4 (시행일 2020-05-26). The 2018 개정 부칙 phases both to 0% (full inclusion) by 2024; I could **not** fetch the 부칙 text live. Secondary sources assert full 100% inclusion of 매월 정기·일률 지급 상여금·복리후생비 for 2024+. **Do not hardcode a 산입범위 percentage until the 부칙 is read.** Model 산입범위 as an effective-dated table with a `source_url` NOT NULL and let it fail closed.

---

## 2. 4대보험 — 2026 rates, splits, caps

### 2.1 국민연금

| Parameter | Value | Source | Effective |
|---|---|---|---|
| 보험료율 (총) | **9.5%** (9% → 9.5%, +0.5%p) | https://www.mohw.go.kr/board.es?mid=a10503000000&bid=0027&list_no=1488390&act=view (보도자료 2025-12-29): "보험료율이 9% → 9.5%(+0.5%p)로 조정된다" | 2026-01-01 |
| Escalation | +0.5%p/yr → **13% from 2033** | https://www.nps.or.kr/pnsinfo/ntpsklg/getOHAF0038M0.do : "2026년부터는 매년 0.5%p씩 보험료율을 8년간 인상하여 2033년부터는 13%의 보험료율이 적용됩니다" | 2026–2033 |
| Split (사업장가입자) | **50 / 50** → employee **4.75%**, employer **4.75%** | https://www.nps.or.kr/pnsinfo/ntpsklg/getOHAF0038M0.do : "보험료율에 해당하는 금액을 본인과 사업장의 사용자가 각각 절반씩 부담" | 2026 |
| 기준소득월액 상한 | **6,370,000원** | same NPS page | 2025-07-01 ~ 2026-06-30 |
| 기준소득월액 하한 | **400,000원** | same | 2025-07-01 ~ 2026-06-30 |
| 기준소득월액 상한 | **6,590,000원** | same NPS page | **2026-07-01 ~ 2027-06-30** |
| 기준소득월액 하한 | **410,000원** | same | **2026-07-01 ~ 2027-06-30** |

**Trap:** the cap year runs **July→June**, not calendar. A single "2026 rate row" is wrong. The rate table needs `(effective_from, effective_to)` per *parameter*, not per year — 국민연금 rate is calendar-scoped, its caps are July-scoped, and both change inside 2026.

### 2.2 건강보험 + 장기요양

| Parameter | Value | Source | Effective |
|---|---|---|---|
| 건강보험료율 (직장) | **7.19%** (+0.1%p, +1.48%) | https://www.mohw.go.kr/board.es?mid=a10503010100&bid=0027&act=view&list_no=1487279 (발표 2025-08-28); https://edi.nhis.or.kr/portal/images/popup/20251204_pop01longdesc.html : "7.19%(전년대비1.48%인상)" | 2026-01-01 |
| Split | **50 / 50** → employee **3.595%** | NHIS notice above | 2026 |
| 장기요양보험료율 (소득 대비) | **0.9448%** (+2.90%) | https://www.mohw.go.kr/board.es?act=view&bid=0027&list_no=1487817&mid=a10503000000 ; NHIS notice: "0.9448%(전년대비2.90%인상)" | 2026-01-01 |
| 장기요양 계산식 | **건강보험료 × 0.9448% ÷ 7.19%** (= ×13.1405…%) | NHIS notice (formula stated verbatim) | 2026 |
| 장기요양 split | 50 / 50 → employee **0.4724%** of 보수월액 | derived from NHIS 50/50 health split | 2026 |

**Trap:** 장기요양 is officially defined **two ways** — as a rate on 소득 (0.9448%) *and* as a multiplier on the already-computed 건강보험료 (0.9448/7.19). Compute it the NHIS way (multiplier on the rounded 건강보험료), not the income way, or you will differ by rounding cents from the NHIS EDI 고지서 and every reconciliation will show a diff.

### 2.3 고용보험

Statutory source: 고용보험 및 산업재해보상보험의 보험료징수 등에 관한 법률 **시행령 제12조** — https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=고용보험%20및%20산업재해보상보험의%20보험료징수%20등에%20관한%20법률%20시행령&joNo=001200000&mode=4 (시행일 **2025-12-23**, 대통령령 제35935호).

| Parameter | Value (verbatim) | Bearer |
|---|---|---|
| 실업급여 보험료율 | **"1천분의 18"** = 1.8% | split 50/50 → **0.9% employee / 0.9% employer** |
| 고용안정·직업능력개발, 상시 150명 미만 | **"1만분의 25"** = 0.25% | employer only |
| … 150명 이상 우선지원대상기업 | **"1만분의 45"** = 0.45% | employer only |
| … 150명 이상 1천명 미만 | **"1만분의 65"** = 0.65% | employer only |
| … 1천명 이상 / 국가·지자체 | **"1만분의 85"** = 0.85% | employer only |

Also in 제12조: 상시근로자수 counts **all domestic 사업 of the same 사업주 combined** (공동주택 관리사업 counted per-사업); if a headcount increase pushes the band up, the **prior rate is held for 3 보험연도** from the following 보험연도.

**needs-verification:** the exact statutory basis for "실업급여 50/50" is 보험료징수법 제13조제2항 (근로자 부담 = 실업급여 요율의 1/2), which I did not fetch article-by-article. The 0.9% figure itself is consistent across MOEL-derived sources and matches `docs/specs/payroll.md`. Fetch 제13조 before hardcoding.

### 2.4 산재보험

| Parameter | Value | Source | Effective |
|---|---|---|---|
| 고시 | **고용노동부고시 제2025-91호**, 공고 **2025-12-31** | https://www.moel.go.kr/info/lawinfo/instruction/view.do?bbs_seq=20251201757 | 2026-01-01 |
| 평균 산재보험료율 | **1.47%** (동결, 3년 연속) | https://eiec.kdi.re.kr/policy/materialView.do?num=275553 (KDI mirror of MOEL release; 심의·의결 2025-12-12) | 2026 |
| Composition | 28개 사업종류별 요율 + 전 업종 동일 **출퇴근재해요율**; the 1.47% average **includes** 출퇴근재해요율 | same | 2026 |
| Bearer | **employer only**, no employee deduction | 산재보험법 / `docs/specs/payroll.md` line 41 | — |

**needs-verification:** the numeric **출퇴근재해요율** and the per-업종 table. The MOEL 고시 page publishes them only inside `.hwp/.hwpx` attachments — the HTML carries no numbers. Machine-readable per-업종 rates exist at 공공데이터포털 https://www.data.go.kr/data/15068737/fileData.do ("고용노동부_사업종류별 산재보험요율_20260101"). **Ingest that dataset; never type an industry rate by hand.**

**Landing (all of §2):** payroll (employee deduction lines + employer-cost lines), and the org record needs `상시근로자수` + `우선지원대상기업` flag + `업종코드` because three of the four insurances band on them.

---

## 3. 근로기준법 working time

All articles fetched from 국가법령정보센터 조문 endpoint pattern
`https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=근로기준법&lsId=2031481&joNo=<6-digit>&mode=4`.
Current 시행일 for the fetched articles: **2025-10-23** (법률 제20520호, 2024-10-22 일부개정).

### 3.1 주52 = 40 + 12

| Rule | Verbatim | Article | joNo |
|---|---|---|---|
| "1주" definition | **"1주란 휴일을 포함한 7일을 말한다."** | 제2조제1항제7호 | 000200000 |
| 소정근로시간 definition | "제50조… 근로시간의 범위에서 근로자와 사용자 사이에 정한 근로시간" | 제2조제1항제8호 | 000200000 |
| 평균임금 | "산정하여야 할 사유가 발생한 날 이전 3개월 동안에 지급된 임금의 총액을 그 기간의 총일수로 나눈 금액" | 제2조제1항제6호 | 000200000 |
| 주 기본 | **"1주 간의 근로시간은 휴게시간을 제외하고 40시간을 초과할 수 없다."** | 제50조제1항 | (lsJoLnkSeq=900552087) |
| 일 기본 | **"1일의 근로시간은 휴게시간을 제외하고 8시간을 초과할 수 없다."** | 제50조제2항 | same |
| 대기시간 | "사용자의 지휘ㆍ감독 아래에 있는 대기시간 등은 근로시간으로 본다" | 제50조제3항 | same |
| 연장 한도 | **"당사자 간에 합의하면 1주 간에 12시간을 한도로 제50조의 근로시간을 연장할 수 있다."** | 제53조제1항 | 005300000 |
| 탄력·선택 | 제51조·제52조 정산기간 **평균 주 12시간 초과 불가** | 제53조제2항 | 005300000 |
| 특별연장근로 | "고용노동부장관의 인가와 근로자의 동의를 받아" 추가 연장 (긴급 시 사후승인) | 제53조제4항 | 005300000 |

→ **40 + 12 = 52**, computed over **휴일을 포함한 7일**, i.e. holiday work counts toward the 12h 연장 cap.

**Sunset trap — do not implement 제53조제3항 as live.** The statute text still shows "상시 30명 미만 … 근로자대표와 서면합의 시 1주 8시간 추가 연장". MOEL 행정해석 **임금근로시간과-545 (2022-03-07)** (https://www.law.go.kr/LSW/cgmExpcInfoP.do?cgmExpcDatSeq=16804&mode=2&ofiClsCd=350101) states it was 한시적, **valid only through 2022-12-31**, and that post-expiry use is punished as a 제53조제1항·제2항 violation. A 60h/week cap for <30인 orgs is **wrong in 2026**.

### 3.2 가산율 (제56조, joNo=005600000, 시행 2025-10-23)

| Kind | 가산 | Verbatim |
|---|---|---|
| 연장근로 | **+50%** | "연장근로에 대하여는 통상임금의 100분의 50 이상을 가산하여 지급" |
| 휴일근로 ≤ 8h | **+50%** | 제56조제2항제1호 |
| 휴일근로 > 8h (초과분) | **+100%** | 제56조제2항제2호 |
| 야간근로 (**22:00–06:00**) | **+50%** | "야간근로(오후 10시부터 다음 날 오전 6시 사이)에 대하여는 통상임금의 100분의 50 이상" |

### 3.3 OVERLAP / stacking — the rule most implementations get wrong

**Night premium stacks; it is never absorbed.** MOEL 빠른인터넷상담 (https://www.moel.go.kr/minwon/fastcounsel/fastcounselView.do?inetDcssMngId=202512011020569010365):

> **"야간근로가 휴일·연장근로와 중복될 경우 야간근로가산수당은 추가지급되어야 합니다."**

Resulting multipliers on 통상시급 (5인 이상 사업장):

| Situation | Multiplier |
|---|---|
| 소정 within 40h | 1.0 |
| 연장 (평일, 주간) | 1.0 + 0.5 = **1.5** |
| 야간 only (소정시간 내, 22:00–06:00) | 1.0 + 0.5 = **1.5** |
| 연장 + 야간 | 1.0 + 0.5 + 0.5 = **2.0** |
| 휴일 ≤8h | 1.0 + 0.5 = **1.5** |
| 휴일 ≤8h + 야간 | 1.0 + 0.5 + 0.5 = **2.0** |
| 휴일 >8h (초과분) | 1.0 + 1.0 = **2.0** |
| 휴일 >8h + 야간 | 1.0 + 1.0 + 0.5 = **2.5** |

**Engineering shape:** premiums are **additive flags on a time segment**, not a `enum PremiumKind`. Segment a shift at 22:00, 06:00, the 8h-holiday boundary, and the 40h-week boundary; each segment carries an independent `{overtime: bool, night: bool, holiday: bool, holiday_over_8h: bool}` and the multiplier is `1.0 + Σ`. Any design with a single "premium type" per hour will silently underpay 연장+야간, which is a wage-underpayment (임금체불) exposure, not a rounding bug.

**Do NOT stack 휴일 and 연장 for the same hour.** 휴일근로 8시간 초과분 is already 100%; adding a separate 연장 50% on top is the well-known double-count that the 2018 개정 settled. Model 휴일근로 as its own ladder (50 / 100) that *excludes* 연장 가산, with 야간 as the only orthogonal add-on.

### 3.4 휴게 (제54조, joNo=005400000)

> ① **"사용자는 근로시간이 4시간인 경우에는 30분 이상, 8시간인 경우에는 1시간 이상의 휴게시간을 근로시간 도중에 주어야 한다."**
> ② **"휴게시간은 근로자가 자유롭게 이용할 수 있다."**

**Landing:** attendance. "근로시간 도중" is load-bearing — a break appended to the shift end does not satisfy the article. Validation rule: for any shift with worked-time ≥ 4h, require ≥30m of break strictly interior to the shift; ≥8h → ≥60m interior. Break time is **excluded** from 근로시간 for the 40/12 caps (제50조제1항·제2항 say "휴게시간을 제외하고").

### 3.5 휴일 / 주휴수당 (제55조 + 시행령 제30조)

| Rule | Verbatim | Source |
|---|---|---|
| 주휴일 | **"사용자는 근로자에게 1주에 평균 1회 이상의 유급휴일을 보장하여야 한다."** | 제55조제1항, joNo=005500000 |
| 공휴일 유급화 | "대통령령으로 정하는 휴일을 유급으로 보장하여야 한다. 다만, 근로자대표와 서면으로 합의한 경우 특정한 근로일로 대체할 수 있다." | 제55조제2항, same |
| 주휴 지급 조건 | **"법 제55조에 따른 유급휴일은 1주 동안의 소정근로일을 개근한 자에게 주어야 한다."** | 근로기준법 시행령 제30조 — http://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=근로기준법+시행령&lsId=prec20110421&joNo=003000&efYd=20110421&mode=11 |
| 초단시간 제외 | **"4주 동안(4주 미만으로 근로하는 경우에는 그 기간)을 평균하여 1주 동안의 소정근로시간이 15시간 미만인 근로자에 대하여는 제55조와 제60조를 적용하지 아니한다."** | 제18조제3항, joNo=001800000 |

**주휴수당 conditions (implementable predicate):**
1. 4주 평균 주 소정근로시간 **≥ 15h** (제18조제3항), AND
2. 그 주의 **소정근로일 개근** (시행령 제30조), AND
3. 상시 5명 이상? — **No.** 제55조 applies to <5인 사업장 too (unlike 제56조/제60조). Confirm against 시행령 별표1 before coding the exclusion set.

**needs-verification:** the *amount* formula for part-timers (`소정근로시간/40 × 8 × 통상시급`) is 행정해석/지침, not statute. Also unresolved live: whether 지각·조퇴 break 개근 (secondary sources say no, they do not count as 결근). Get a MOEL 지침 citation before implementing the proration.

### 3.6 적용범위 gate (제11조, joNo=001100000, 시행 2025-10-23)

> ① **"이 법은 상시 5명 이상의 근로자를 사용하는 모든 사업 또는 사업장에 적용한다."** 다만 동거하는 친족만을 사용하는 사업과 가사 사용인은 제외.
> ② 상시 4명 이하 사업장에는 대통령령으로 정하는 바에 따라 **일부 규정만** 적용.
> ③ 상시근로자수 산정 방법은 대통령령으로 정한다.

**제56조 (가산수당) and 제60조 (연차) do not apply to <5인 사업장** (per 시행령 별표1). **Landing:** every payroll/attendance/leave rule must be gated on the org's `상시근로자수` at the time of the pay period, not "today".

**needs-verification (policy watch, NOT law):** MOEL's 2026 업무보고 reportedly proposes phased extension of 근로기준법 to <5인 사업장 (stage 1: 연차, 괴롭힘 금지, 여성 야간·휴일 제한; stage 3: 가산수당). **Only secondary/blog sources found live — no gov URL.** Treat as unenacted. Mitigation is cheap: keep the exclusion set a **data table** keyed by `(article, min_headcount, effective_from)`, not `if headcount < 5` branches.

---

## 4. 연차 유급휴가

### 4.1 Accrual — 제60조 (joNo=006000000)

| Rule | Verbatim / value |
|---|---|
| 제1항 | **"사용자는 1년간 80퍼센트 이상 출근한 근로자에게 15일의 유급휴가를 주어야 한다."** |
| 제2항 | **"계속하여 근로한 기간이 1년 미만인 근로자 또는 1년간 80퍼센트 미만 출근한 근로자에게 1개월 개근 시 1일의 유급휴가"** → max **11일** in the first year (11 completed months before the 1-year mark) |
| 제4항 | 3년 이상 계속근로 시 **최초 1년 초과 매 2년마다 1일 가산**, 총 **한도 25일** |
| 제7항 | **"1년간 행사하지 아니하면 소멸된다"** — 다만 사용자의 귀책사유로 사용하지 못한 경우 제외 |
| 초단시간 제외 | 제18조제3항 — 4주 평균 주 15시간 미만이면 제60조 미적용 |
| <5인 제외 | 제11조 + 시행령 별표1 |

Accrual formula for year *n* of 계속근로 (n ≥ 1, 80%+ attendance): `min(15 + floor((n - 1) / 2), 25)`. Reaches the 25-day cap at n = 21.

### 4.2 촉진 (제61조) — exact timings

Fetched: https://www.law.go.kr/lsLinkProc.do?ancYd=20160302&lsClsCd=L&lsNm=근로기준법&lsId=2031481&joNo=006100000&mode=4

Both 제1항 and 제2항 open with the same consequence: 사용자가 촉진 조치를 하였음에도 근로자가 휴가를 사용하지 아니하여 제60조제7항 본문에 따라 소멸된 경우 → **"사용자는 그 사용하지 아니한 휴가에 대하여 보상할 의무가 없다."**

**제1항 — 1년 이상 근로자 (제60조제1항·제2항·제4항 휴가):**
| Step | Timing (verbatim) | Actor |
|---|---|---|
| 1. 미사용 일수 통보 + 사용시기 지정 서면 촉구 | **"기간이 끝나기 6개월 전을 기준으로 10일 이내에"** | 사용자 |
| 2. 사용시기 통보 | **"촉구를 받은 때부터 10일 이내에"** | 근로자 |
| 3. 근로자 미통보 시 사용자가 시기 지정·서면 통보 | **"기간이 끝나기 2개월 전까지"** | 사용자 |

**제2항 — 계속근로 1년 미만 근로자 (제60조제2항 휴가):**
| Step | Timing (verbatim) | Actor |
|---|---|---|
| 1. 1차 촉구 | **"최초 1년의 근로기간이 끝나기 3개월 전을 기준으로 10일 이내에"** | 사용자 |
| 2. 1차 사용자 지정·통보 | **"최초 1년의 근로기간이 끝나기 1개월 전까지"** | 사용자 |
| 3. 2차 촉구 (마지막 달 발생분) | **"최초 1년의 근로기간이 끝나기 1개월 전을 기준으로 5일 이내에"** | 사용자 |
| 4. 2차 사용자 지정·통보 | **"최초 1년의 근로기간이 끝나기 10일 전까지"** | 사용자 |

**Form requirement:** 서면. MOEL 행정해석 (surfaced live via 노무법인 secondary summaries — cite needs-verification): "서면" means paper; electronic documents qualify **only** where the company runs a complete 전자결재체계 managing 기안/결재/시행 for all business. **Landing:** the leave module's 촉진 must emit a signed, immutable, per-employee document with a 발송 timestamp, not an in-app toast.

### 4.3 미사용수당

Statute gives only the **negative**: 제61조 removes the 보상 의무 *when* 촉진 was properly done. Therefore the affirmative rule the system must implement is: **unused leave that lapses under 제60조제7항 must be paid out as 임금 UNLESS a complete, timely, written 촉진 sequence is on file.** Payout base = 통상임금 (or 평균임금 per 취업규칙) for the 소멸 일수.

**Engineering shape:** `leave_promotion_evidence` rows keyed to `(employee, leave_year, step)` with `sent_at`, `channel`, `document_hash`. The payout job's default is **pay**; it withholds only when all required steps exist and every timing window was met. Fail-open-to-paying is the safe direction — the reverse is 임금체불.

**needs-verification:** 미사용수당 산정 기준임금 (통상 vs 평균) — 행정해석-driven and 취업규칙-dependent; get a 노무사 line before coding a default.

**Landing:** leave (accrual engine, 촉진 scheduler, 소멸 job), payroll (미사용수당 line item), attendance (80% 출근율 computation — needs the 소정근로일 denominator and the statutory 출근 간주 days: 업무상 부상·질병 휴업기간, 출산전후휴가, 육아휴직 — **needs-verification**, 제60조제6항 not fetched).

---

## 5. 소득세 간이세액표 — mechanism

Source: 국세청 «근로소득 원천징수방법(간이세액표)» — https://www.nts.go.kr/nts/cm/cntnts/cntntsView.do?mi=6583&cntntsId=7862 (accessed 2026-07-25). Statutory basis: 소득세법 제134조 + 소득세법 시행령 **제194조** (근로소득 간이세액표의 적용) + 별표2 — https://www.law.go.kr/법령/소득세법시행령/제194조

**How the table is consumed (this is what we implement; the rows themselves stay gated):**

1. **Trigger:** applied "근로자에게 매월 급여(상여금 포함)를 지급할 때" — monthly withholding, separate from 연말정산.
2. **Row lookup — two axes only:**
   - **월 급여액** bucket, computed **excluding 비과세 and 자녀 학자금 지원금액** (e.g. "3,500천원 이상 3,520천원 미만"). Buckets are 20,000원 wide in that range — bucket width is not uniform across the table, so it must be an interval lookup, not arithmetic.
   - **공제대상가족의 수** — 본인 + 배우자 + 8세 이상 20세 이하 자녀 등 (NTS example: "본인, 배우자, 8세이상 20세 이하 자녀 2명인 경우 공제대상가족의 수는 4명").
3. **자녀 추가공제** subtracted from the looked-up amount: 1명 **12,500원**, 2명 **29,160원**, 3명 이상 **29,160원 + (초과 자녀 1명당 25,000원)**.
4. **Election (소득세법 시행령 제194조):** the employee may elect **80% / 100% / 120%** of the table amount via 「소득세 원천징수세액 조정신청서」 or by entering the ratio on the 소득·세액공제신고서. **Default = 100%.** (Election mechanism introduced 2015-06-30.)
5. **지방소득세:** withheld separately at 10% of 소득세 — **needs-verification**, not stated on the fetched NTS page; 지방세법 제103조의? not fetched. `docs/specs/payroll.md` line 43 already requires it come from the supplied row/golden case rather than a hidden approximation. Keep that.
6. **Official artifact:** 홈택스 → 세금신고 → 원천세 신고 → 근로소득 간이세액표 (한글/excel/pdf). This is the ingestion target.

**Landing:** payroll. The kernel implements steps 1–4 as a **table-driven lookup with a required `tax_table_version`**; it must refuse to compute when no row is supplied (already the rule in `docs/specs/payroll.md` line 42: "The kernel must not synthesize tax brackets from memory"). The 80/100/120 election is a **per-employee payroll attribute**, effective-dated, audit-logged (it changes take-home pay).

**Explicitly out of scope per charter:** the 2026 간이세액표 *rows* are NTS-gated. Ingest the Excel; do not transcribe.

---

## 6. 채용절차의 공정화에 관한 법률 (채용절차법)

Law lsId **011990**. Articles fetched 2026-07-25; the fetched 조문 report **시행일 2020-05-26** (no amendment in force since).

### 6.1 결과 통지 의무

> **제10조 (채용 여부의 고지):** "구인자는 채용대상자를 확정한 경우에는 지체 없이 구직자에게 채용 여부를 알려야 한다."
> https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=채용절차의%20공정화에%20관한%20법률&lsId=011990&joNo=001000000&mode=4

Trigger is **확정 of the 채용대상자** — the duty fires for *every* 구직자 in the requisition the moment the hire is decided, not per-candidate as each is rejected. "지체 없이" ≈ without delay (PIPA guidance treats 지체 없이 as ~5일; see §7).

### 6.2 서류 반환 / 파기

> **제11조 (채용서류의 반환 등):** 구직자의 채용 여부가 확정된 이후 구직자(확정된 채용대상자는 제외)가 반환을 청구하면 **본인 확인 후 반환하여야 한다**. 반환 청구기간이 지난 경우 및 반환하지 아니한 경우에는 **「개인정보 보호법」에 따라 채용서류를 파기하여야 한다.** 반환 비용은 원칙적으로 구인자 부담. 구인자는 반환 관련 사항을 **채용 여부가 확정되기 전까지** 구직자에게 알려야 한다.
> https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=채용절차의%20공정화에%20관한%20법률&lsId=011990&joNo=001100000&mode=4

**시행령 timings (the numbers live here, not in the Act):**

| Parameter | Verbatim | Source |
|---|---|---|
| 반환 청구기간 | **"채용서류의 반환 청구기간은 구직자의 채용 여부가 확정된 날 이후 14일부터 180일까지의 기간의 범위에서 구인자가 정한 기간으로 한다."** → employer picks a window, **min 14일, max 180일** from 확정일, and must pre-announce it | 시행령 제4조 — https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=채용절차의%20공정화에%20관한%20법률%20시행령&joNo=000400000&mode=4 |
| 반환 실행 기한 | **"구직자가 반환 청구를 한 날부터 14일 이내에 구직자에게 해당 채용서류를 발송하거나 전달하여야 한다."** 원칙적으로 우편법상 **특수취급우편물**; 합의 시 다른 방식 가능. 발송지 = 구직자 주소지 또는 청구 시 지정 장소 | 시행령 제2조 — https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=채용절차의%20공정화에%20관한%20법률%20시행령&joNo=000200000&mode=4 |
| 보관 기간 | 반환 미청구 시 → 제4조 청구기간 만료까지; 반환 청구 시 → 특수취급우편물 발송/전달 시점까지 | 시행령 제3조 — joNo=000300000 |
| 파기 | 청구기간 경과 또는 미반환 → PIPA에 따라 파기 (see §7) | 법 제11조 |

**needs-verification:** 적용범위 (상시 30명 이상 사업장 — 법 제3조) and 과태료 amounts. The MOEL «채용절차법 조항별 Q&A» page (https://www.moel.go.kr/policy/policydata/view.do?bbs_seq=20230900682) exists but publishes only a `.hwp` attachment; the HTML has no numbers. Also: a 전부개정 **「공정채용법」** was proposed (2023 발의 → 임기만료 폐기; re-filed) — **no in-force 2026 version found live**; treat 채용절차법 as current and keep the retention windows configurable.

**Landing:** recruiting. Concrete obligations to build:
- `requisition.retention_window_days ∈ [14, 180]`, set per requisition, **shown to applicants before 확정** (제11조 사전 고지).
- On 확정: fan-out a 결과 통지 to every non-selected applicant, audited, "지체 없이".
- Return request → SLA timer, **14일**, evidencing 특수취급우편물 dispatch (tracking number) or agreed alternate delivery.
- On window expiry with no request → **automatic PIPA-compliant destruction job**, evidenced.

---

## 7. PIPA — talent-pool consent & retention

Law: 개인정보 보호법. Articles fetched 2026-07-25 report **시행일 2025-10-02** (법률 제20897호, 2025-04-01 일부개정).

### 7.1 Lawful basis + what consent must disclose

**제15조제1항** lawful bases (verbatim list): ① 정보주체의 동의 ② 법률에 특별한 규정 / 법령상 의무 준수 불가피 ③ 공공기관 소관 업무 불가피 ④ **계약 이행 또는 계약 체결 과정에서 정보주체의 요청에 따른 조치 이행에 필요** ⑤ 급박한 생명·신체·재산 이익 ⑥ **개인정보처리자의 정당한 이익 (명백히 정보주체 권리보다 우선)** ⑦ 공중위생 등 긴급 필요.
https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=개인정보%20보호법&joNo=001500000&mode=4

**제15조제2항** — when relying on consent, four items must be disclosed and consented to:
1. 개인정보의 수집·이용 목적
2. 수집하려는 개인정보의 항목
3. **개인정보의 보유 및 이용 기간**
4. 동의를 거부할 권리가 있다는 사실 및 거부에 따른 불이익의 내용

**제15조제3항** — additional use within a scope reasonably related to the original purpose, without fresh consent, weighing 불이익 발생 여부 and 안전성 확보 조치.

**Applied to a talent pool:** the *application* is 제15조제1항제4호 territory (계약 체결 과정의 요청에 따른 조치). **Retaining the profile after the requisition closes, for future openings, is a new purpose** → requires a **separate, refusable 인재풀 consent** carrying its own 보유기간, and refusal must not disadvantage the current application. Two consents, two retention clocks, two destruction jobs.

### 7.2 파기

**제21조** (https://www.law.go.kr/lsLinkProc.do?lsClsCd=L&lsNm=개인정보%20보호법&joNo=002100000&mode=4):
> ① "보유기간의 경과, 개인정보의 처리 목적 달성, 가명정보의 처리 기간 경과 등 그 개인정보가 불필요하게 되었을 때에는 **지체 없이** 그 개인정보를 파기하여야 한다." (다른 법령에 따라 보존하여야 하는 경우 제외)
> ② "복구 또는 재생되지 아니하도록 조치하여야 한다."
> ③ 법령상 보존 시 → **"다른 개인정보와 분리하여서 저장ㆍ관리하여야 한다."**
> ④ 파기 방법·절차는 대통령령.

법제처 easylaw (https://easylaw.go.kr/CSP/CnpClsMain.laf?popMenu=ov&csmSeq=1257&ccfNo=2&cciNo=2&cnpClsNo=3):
- No statutory day-count; the standard is **"지체 없이"**. (The "5일 이내" figure comes from the 2023 개인정보위·고용노동부 «개인정보보호 가이드라인(인사·노무편)» — **needs-verification**, guideline not fetched.)
- 파기 방법: electronic → 전용 소자장비 / 복원 불가 초기화 / 덮어쓰기; 인쇄물 → 파쇄 또는 소각 (완전 파기 곤란 시 마스킹·천공).
- 과태료: 파기 불이행 **3천만원 이하**; 분리보관 미이행 **1천만원 이하**.

**Landing:** recruiting (candidate profile lifecycle), and the platform-wide retention machinery.
- Every candidate record carries `(purpose, lawful_basis, consent_id, retention_until)`; destruction is a **scheduled job**, not a manual button, and it writes an audit event (`what/when/method`) without dumping the payload — consistent with `korean-legal-boundaries.md` guardrail 8.
- Legal-hold path (제21조③): records kept under another statute must be **physically/logically separated** from the live pool, not just flagged.
- Deletion must be **irrecoverable** (제21조②) — soft-delete alone violates it. Backups/RustFS objects must be in scope of the destruction job.

---

## 8. Consolidated parameter table for the rate-table migration

Effective-dated rows lens D should seed (every row NOT NULL on `source_url` + `retrieved_at`):

| key | value | unit | effective_from | effective_to | source |
|---|---|---|---|---|---|
| `minimum_wage.hourly` | 10320 | KRW | 2026-01-01 | 2026-12-31 | moel enews 18144 |
| `minimum_wage.monthly_hours` | 209 | h | 2026-01-01 | — | moel enews 18144 |
| `nps.rate_total` | 0.095 | ratio | 2026-01-01 | 2026-12-31 | mohw 1488390 |
| `nps.employee_share` | 0.5 | ratio | 2026-01-01 | — | nps OHAF0038M0 |
| `nps.base_cap_max` | 6370000 | KRW | 2025-07-01 | 2026-06-30 | nps OHAF0038M0 |
| `nps.base_cap_min` | 400000 | KRW | 2025-07-01 | 2026-06-30 | nps OHAF0038M0 |
| `nps.base_cap_max` | 6590000 | KRW | **2026-07-01** | 2027-06-30 | nps OHAF0038M0 |
| `nps.base_cap_min` | 410000 | KRW | **2026-07-01** | 2027-06-30 | nps OHAF0038M0 |
| `nhis.health_rate` | 0.0719 | ratio | 2026-01-01 | 2026-12-31 | mohw 1487279 / nhis edi |
| `nhis.health_employee_share` | 0.5 | ratio | 2026-01-01 | — | nhis edi |
| `nhis.ltc_rate_on_income` | 0.009448 | ratio | 2026-01-01 | 2026-12-31 | mohw 1487817 / nhis edi |
| `nhis.ltc_multiplier_on_health_premium` | 0.009448 / 0.0719 | ratio | 2026-01-01 | 2026-12-31 | nhis edi (formula) |
| `ei.unemployment_rate_total` | 0.018 | ratio | 2025-12-23 | — | 징수법 시행령 제12조 |
| `ei.stabilization_lt150` | 0.0025 | ratio | 2025-12-23 | — | 징수법 시행령 제12조 |
| `ei.stabilization_ge150_priority` | 0.0045 | ratio | 2025-12-23 | — | 징수법 시행령 제12조 |
| `ei.stabilization_150_999` | 0.0065 | ratio | 2025-12-23 | — | 징수법 시행령 제12조 |
| `ei.stabilization_ge1000` | 0.0085 | ratio | 2025-12-23 | — | 징수법 시행령 제12조 |
| `wc.average_rate` | 0.0147 | ratio (informational) | 2026-01-01 | 2026-12-31 | 고용노동부고시 2025-91호 |
| `wc.by_industry.*` | — | ratio | 2026-01-01 | 2026-12-31 | **needs-verification** → data.go.kr 15068737 |
| `lsa.overtime_premium` | 0.5 | ratio | 2025-10-23 | — | 근기법 제56조① |
| `lsa.night_premium` | 0.5 | ratio | 2025-10-23 | — | 근기법 제56조③ |
| `lsa.night_window` | 22:00–06:00 | time | 2025-10-23 | — | 근기법 제56조③ |
| `lsa.holiday_premium_le8h` | 0.5 | ratio | 2025-10-23 | — | 근기법 제56조②1 |
| `lsa.holiday_premium_gt8h` | 1.0 | ratio | 2025-10-23 | — | 근기법 제56조②2 |
| `lsa.weekly_base_hours` | 40 | h | — | — | 근기법 제50조① |
| `lsa.weekly_overtime_cap` | 12 | h | — | — | 근기법 제53조① |
| `lsa.break_4h` / `lsa.break_8h` | 30 / 60 | min | — | — | 근기법 제54조① |
| `lsa.short_time_exclusion_hours` | 15 | h/week (4wk avg) | — | — | 근기법 제18조③ |
| `lsa.annual_leave_base` / `_cap` | 15 / 25 | days | — | — | 근기법 제60조①④ |
| `lsa.annual_leave_first_year_max` | 11 | days | — | — | 근기법 제60조② |
| `lsa.attendance_threshold` | 0.80 | ratio | — | — | 근기법 제60조① |
| `lsa.promotion.*` | 6m/10d, 10d, 2m; 3m/10d, 1m, 1m/5d, 10d | — | — | — | 근기법 제61조①② |
| `fhp.return_window_min` / `_max` | 14 / 180 | days | — | — | 채용절차법 시행령 제4조 |
| `fhp.return_dispatch_days` | 14 | days | — | — | 채용절차법 시행령 제2조 |
| `lsa.min_headcount_for_articles` | 5 | headcount | — | — | 근기법 제11조① |

---

## 9. needs-verification register (blocking items for lens D)

| # | Item | Why it matters | Where to get it |
|---|---|---|---|
| V1 | 최저임금법 제6조제4항 **부칙** phase-down of 상여금 25% / 복리후생 7% to 0% | Determines 산입범위; wrong = false minimum-wage violations or missed ones | law.go.kr 최저임금법 부칙 (2018 개정) |
| V2 | 산재보험 **출퇴근재해요율** + 28개 업종별 요율 | Employer cost line cannot be computed | data.go.kr 15068737 (machine-readable); MOEL 고시 hwpx |
| V3 | 보험료징수법 **제13조제2항** (실업급여 근로자 1/2 부담) | Statutory basis for the 0.9% employee split | law.go.kr 징수법 제13조 |
| V4 | 근로기준법 **시행령 별표1** (4인 이하 사업장 적용 조문 목록) | The exclusion set for 제56조/제60조 and whether 제55조 is in it | law.go.kr 근기법 시행령 별표1 |
| V5 | 근로기준법 **제60조제6항** (출근 간주 기간: 업무상 부상·질병, 출산전후휴가, 육아휴직) | 80% 출근율 denominator/numerator | law.go.kr 제60조 full text |
| V6 | 주휴수당 **금액** 산정 for part-timers; 지각·조퇴 vs 개근 | Payout amount | MOEL 지침/행정해석 |
| V7 | 연차 미사용수당 **기준임금** (통상 vs 평균) | Payout amount | 노무사 sign-off (already required by payroll.md gate) |
| V8 | 지방소득세 10% withholding basis | Payslip line | 지방세법; NTS 원천징수 안내 |
| V9 | 채용절차법 **제3조 적용범위** (상시 30명 이상?) + 과태료 | Whether the module's obligations even fire for small orgs | law.go.kr 채용절차법 제3조; MOEL Q&A hwp |
| V10 | 「인사·노무 개인정보보호 가이드라인」 (2023, 개인정보위·고용노동부) — "지체 없이 = 5일" | Destruction SLA | pipc.go.kr 자료실 |
| V11 | 제61조 "서면" — electronic acceptability conditions | Whether in-app 촉진 notices are valid | MOEL 행정해석 |
| V12 | 「공정채용법」 전부개정 status | Retention windows may change | 국회 의안정보 / MOEL |
| V13 | 2026 NTS **간이세액표 rows** | Withholding amounts | 홈택스 → 원천세 → 간이세액표 (Excel) — **ingest, do not transcribe** |
| V14 | <5인 사업장 단계적 확대 roadmap | Only blog sources found live | MOEL 2026 업무보고 (gov URL not located) |
