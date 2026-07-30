// Resolve a Korean statutory instrument to the fields the jurisdiction register
// requires as evidence, from the official National Legal Information Center API.
//
// WHY THIS EXISTS
//
// docs/program/console-jurisdiction-register.json names four `allowed_sources`, and
// `official_legislation_portal` is the only one reachable without a licensed reviewer.
// This fetches from it. It does NOT move any control off HOLD — it produces the
// source_uri / effective_date / 공포번호 an authority would need in front of them.
//
// TWO THINGS LEARNED BY CALLING IT, both of which contradict what the mirror repos suggest:
//
// 1. The API host is `www.law.go.kr/DRF/`, NOT `open.law.go.kr`. The latter is the
//    registration portal and 404s on the DRF path. Verified: 200 vs 404 on the same query.
//
// 2. **The API echoes your OC key back inside every `법령상세링크` field.** So every raw
//    response contains the credential. This script scrubs it from all output and writes
//    no raw XML. A cache of raw responses — which is what the third-party pipelines
//    keep — is a file full of someone's key. Do not commit response bodies.
//
// The OC is self-designated (the caller picks the value), so it is an identifier rather
// than a bearer secret and a leak is an attribution problem, not a compromise. That is a
// reason to scrub it anyway, not a reason to relax.
//
// Usage:
//   LAW_OC=... node scripts/korean-legal/fetch-statutory-source.mjs 근로기준법 [more names...]
//   LAW_OC=... node scripts/korean-legal/fetch-statutory-source.mjs --json 근로기준법
//   LAW_OC=... node scripts/korean-legal/fetch-statutory-source.mjs --check-kernel
//
// Exit codes: 0 all resolved; 1 any name unresolved, any request failed, or (under
// --check-kernel) any cited source URL that does not resolve.

const API = 'https://www.law.go.kr/DRF/lawSearch.do';
const THROTTLE_MS = 200; // the operators' own documented spacing; do not lower
const RETRY_DELAYS_MS = [2000, 4000, 8000];

const OC = process.env.LAW_OC;
if (!OC) {
  console.error('LAW_OC is not set. Register at open.law.go.kr and export it; never commit it.');
  process.exit(1);
}

const args = process.argv.slice(2);
const asJson = args.includes('--json');
const wantKernelCheck = args.includes('--check-kernel');
const names = args.filter((a) => !a.startsWith('--'));
if (names.length === 0 && !wantKernelCheck) {
  console.error('Usage: LAW_OC=... node scripts/korean-legal/fetch-statutory-source.mjs <법령명> [...]');
  console.error('       LAW_OC=... node scripts/korean-legal/fetch-statutory-source.mjs --check-kernel');
  process.exit(1);
}

// The key appears in responses. Strip it before anything is printed or written.
const scrub = (text) => text.split(OC).join('${LAW_OC}');

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function textOf(xml, tag) {
  // The API returns some fields wrapped in CDATA and some bare.
  const m = xml.match(new RegExp(`<${tag}>(?:<!\\[CDATA\\[)?([\\s\\S]*?)(?:\\]\\]>)?</${tag}>`));
  return m ? m[1].trim() : '';
}

function ymdToIso(ymd) {
  if (!/^\d{8}$/.test(ymd)) return '';
  return `${ymd.slice(0, 4)}-${ymd.slice(4, 6)}-${ymd.slice(6, 8)}`;
}

async function fetchWithRetry(url) {
  for (let attempt = 0; ; attempt += 1) {
    let res;
    try {
      res = await fetch(url, { signal: AbortSignal.timeout(20_000) });
    } catch (err) {
      if (attempt >= RETRY_DELAYS_MS.length) throw new Error(`network: ${err.message}`);
      await sleep(RETRY_DELAYS_MS[attempt]);
      continue;
    }
    if (res.ok) return res.text();
    // 4xx other than 429 will not fix themselves; a bad OC is the common case.
    if (res.status !== 429 && res.status < 500) {
      throw new Error(`HTTP ${res.status} — check LAW_OC is registered and the query is valid`);
    }
    if (attempt >= RETRY_DELAYS_MS.length) throw new Error(`HTTP ${res.status} after retries`);
    await sleep(RETRY_DELAYS_MS[attempt]);
  }
}

async function resolve(name) {
  const url = `${API}?OC=${encodeURIComponent(OC)}&target=law&type=XML`
    + `&query=${encodeURIComponent(name)}&display=20`;
  const xml = await fetchWithRetry(url);

  const code = textOf(xml, 'resultCode');
  if (code && code !== '00') {
    throw new Error(`API resultCode ${code}: ${textOf(xml, 'resultMsg')}`);
  }

  // A rejected OC answers HTTP 200 with <Response><result>사용자 정보 검증에 실패하였습니다.</result>
  // and no resultCode — so without this guard it parses to zero rows and reports
  // "NO EXACT MATCH", which reads as "the document does not exist". That false negative is
  // how docs/ideas/payroll-statutory-sources.md came to say four 고시 were unresolvable.
  // An API refusal must never look like an absent document.
  if (!xml.includes('<law ')) {
    const rejection = textOf(xml, 'result');
    if (rejection) throw new Error(`API refused the call: ${rejection} ${textOf(xml, 'msg')}`.trim());
  }

  const blocks = [...xml.matchAll(/<law id="\d+">([\s\S]*?)<\/law>/g)].map((m) => m[1]);
  const rows = blocks.map((b) => ({
    name: textOf(b, '법령명한글'),
    law_id: textOf(b, '법령ID'),
    mst: textOf(b, '법령일련번호'),
    instrument: textOf(b, '법령구분명'),     // 법률 / 대통령령 / 부령 — which layer sets the rate
    ministry: textOf(b, '소관부처명'),
    status: textOf(b, '현행연혁코드'),
    promulgated_on: ymdToIso(textOf(b, '공포일자')),
    promulgation_no: textOf(b, '공포번호'),
    effective_date: ymdToIso(textOf(b, '시행일자')),
    // A citable URL that carries NO credential. The API's own 법령상세링크 embeds the OC
    // and must never be used as the evidence anchor.
    source_uri: `https://www.law.go.kr/법령/${encodeURIComponent(textOf(b, '법령명한글'))}`,
  }));

  // Exact-name match first; the query is a substring search, so 근로기준법 also returns
  // 근로기준법 시행령 and 시행규칙 — which are usually the instruments that carry the rate.
  const exact = rows.filter((r) => r.name === name);
  return { query: name, exact, related: rows.filter((r) => r.name !== name) };
}

// --check-kernel: compare what the payroll kernel CITES against what is currently in
// force. This is a staleness report, not a legal check — it asserts nothing about Korean
// law, only about whether our own citations still point at the live instrument.
//
// It found one real defect on its first run: **total.comwel.or.kr, the 산재보험 anchor,
// answers 400** — a dead evidence anchor.
//
// IT ALSO PRODUCED ONE FALSE POSITIVE, WHICH IS THE MORE USEFUL LESSON.
//
// I read "고용보험법 시행령, effective 2026-07-01, promulgated 2026-06-30" against
// `payroll_sources_verified_on() == 2026-06-27` and concluded the kernel's 고용보험 citation
// was stale. It is not. The 고용보험 employee rate is set in a DIFFERENT decree — 징수법
// 시행령 제12조제1항제2호 (1천분의 18, employee bears ½) — and the kernel cites exactly that:
// `lsiSeq=280527` resolves to 「고용보험 및 산업재해보상보험의 보험료징수 등에 관한 법률 시행령」,
// 대통령령 제35935호, 공포 2025-12-23, 시행 2025-10-01 — which matches the pinned
// efYd=20251001 and predates 2026-06-27. The citation is correct and fresh.
//
// The error was inferring a delegation chain from a NAME MATCH: 고용보험법 sounds like it
// governs 고용보험 rates, so I assumed it did. It does not. See
// docs/ideas/payroll-statutory-sources.md, which read the chains and found the layer is
// different for every single item — 국민연금 4.75% is in an act 부칙, the 간이세액표 is 별표 2 of a
// 대통령령, 산재 is delegated twice.
//
// So this mode cannot conclude staleness from dates alone, and does not try to. It reports
// what resolves and what each URL pins; deciding whether a pin is the RIGHT instrument
// requires reading the delegation chain, which is a human job.
async function checkKernel() {
  const { readFileSync } = await import('node:fs');
  const KERNEL = 'backend/crates/payroll/domain/src/lib.rs';
  const src = readFileSync(KERNEL, 'utf8');

  const verifiedOn = src.match(/payroll_sources_verified_on\(\)\s*->\s*Date\s*\{\s*date!\((\d{4})\s*-\s*(\d{2})\s*-\s*(\d{2})\)/);
  const retrievedOn = verifiedOn ? `${verifiedOn[1]}-${verifiedOn[2]}-${verifiedOn[3]}` : null;
  console.log(`kernel payroll_sources_verified_on(): ${retrievedOn ?? 'NOT FOUND'}\n`);

  const urls = [...new Set([...src.matchAll(/url:\s*"(https?:\/\/[^"]+)"/g)].map((m) => m[1]))];
  let bad = 0;

  for (const url of urls) {
    let code = '000';
    try {
      const res = await fetch(url, { method: 'GET', redirect: 'follow', signal: AbortSignal.timeout(15_000) });
      code = String(res.status);
    } catch { /* leave 000 */ }
    const dead = code === '000' || Number(code) >= 400;
    if (dead) bad += 1;
    console.log(`  [${code}]${dead ? ' DEAD  ' : '       '}${url}`);

    // A law.go.kr permalink pins a version via efYd. If the instrument has since been
    // re-issued, the pin is stale even though the URL still answers 200.
    const efYd = url.match(/efYd=(\d{8})/)?.[1];
    if (efYd) {
      console.log(`         pinned to efYd=${ymdToIso(efYd)} — resolve the instrument to compare`);
    }
    await sleep(THROTTLE_MS);
  }

  if (retrievedOn) {
    console.log(`\nFreshness needs the instrument's 공포일자 <= ${retrievedOn} — but ONLY for the`);
    console.log('instrument that actually sets the number. A name-similar law is not it: see the');
    console.log('false positive in this file\'s header. Read the delegation chain, do not infer it.');
  }
  if (bad > 0) {
    console.error(`\n${bad} of ${urls.length} cited source URLs do not resolve.`);
    process.exit(1);
  }
  console.log(`\nAll ${urls.length} cited source URLs resolve. Freshness still needs the comparison above.`);
}

if (wantKernelCheck) {
  await checkKernel();
  if (names.length === 0) process.exit(0);
}

const retrieved_at = new Date().toISOString();
const results = [];
let failed = 0;

for (const [i, name] of names.entries()) {
  if (i > 0) await sleep(THROTTLE_MS);
  try {
    const r = await resolve(name);
    if (r.exact.length === 0) failed += 1;
    results.push({ ...r, retrieved_at });
  } catch (err) {
    failed += 1;
    results.push({ query: name, error: scrub(String(err.message)), retrieved_at });
  }
}

if (asJson) {
  console.log(scrub(JSON.stringify(results, null, 2)));
} else {
  for (const r of results) {
    console.log(`\n### ${r.query}`);
    if (r.error) { console.log(`  ERROR: ${r.error}`); continue; }
    if (r.exact.length === 0) console.log('  NO EXACT MATCH — do not guess; refine the name');
    for (const row of [...r.exact, ...r.related]) {
      const mark = row.name === r.query ? '*' : ' ';
      console.log(`  ${mark} ${row.name}  [${row.instrument}] ${row.status}`);
      console.log(`      시행 ${row.effective_date}  공포 ${row.promulgated_on} 제${row.promulgation_no}호  ${row.ministry}`);
      console.log(`      ${row.source_uri}`);
    }
  }
  console.log(`\nretrieved_at ${retrieved_at}`);
}

if (failed > 0) {
  console.error(`\n${failed} of ${names.length} unresolved.`);
  process.exit(1);
}
