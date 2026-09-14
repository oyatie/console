const { test, expect } = require('@playwright/test');
const fs = require('node:fs');
const {effectiveSubmissions,unexpectedMutations} = require('./effective-submissions.cjs');
const crypto = require('node:crypto');

// Presentation/transport acceptance against the real mounted Leptos application.
// Fixture construction is a dependency: this file never installs fake routes,
// synthesizes session cookies, fulfills requests or supplies owner responses.
// Each project/case needs an independent owner-produced fixture.
function fixture(testInfo) {
  const name = process.env.CONSOLE_BROWSER_FIXTURE;
  if (!name) throw new Error('PREREQUISITE: CONSOLE_BROWSER_FIXTURE must identify the independently reviewed owner-produced fixture; this is not a product RED');
  const bytes = fs.readFileSync(name);
  if (crypto.createHash('sha256').update(bytes).digest('hex') !== process.env.CONSOLE_BROWSER_FIXTURE_SHA256)
    throw new Error('PREREQUISITE: fixture custody digest mismatch');
  const all = JSON.parse(bytes);
  if (all.kind !== 'SYNTHETIC_OWNER_PRODUCED_BROWSER_FIXTURE' || !/^[a-f0-9]{40}$/.test(all.candidate_sha) || all.candidate_sha !== process.env.CONSOLE_BROWSER_CANDIDATE_SHA)
    throw new Error('PREREQUISITE: missing exact candidate and synthetic fixture custody');
  const f = all.cases[`${testInfo.project.name}/${testInfo.title}`];
  if (!f) throw new Error('PREREQUISITE: separate fixture missing for this project/case');
  const origin = new URL(all.origin);
  if (!['http:', 'https:'].includes(origin.protocol) || !['127.0.0.1','[::1]','localhost'].includes(origin.hostname) || origin.username || origin.password)
    throw new Error('PREREQUISITE: test destination must be owned loopback application');
  const path = f.path;
  if (typeof path !== 'string' || !path.startsWith('/_ui/') || path.startsWith('//') || path.includes('\\')) throw new Error('PREREQUISITE: invalid read target');
  const target = new URL(path, origin.origin);
  if (target.origin !== origin.origin || !target.pathname.startsWith('/_ui/') || target.pathname !== path.split('?')[0]) throw new Error('PREREQUISITE: noncanonical read target');
  return { ...f, origin: origin.origin, url: target.href, pathname: target.pathname };
}
async function open(page, f) {
  await page.context().addCookies(f.cookies); // Real auth-owner-issued synthetic session.
  const response = await page.goto(f.url);
  expect(response.status(), 'actual authorized GET must succeed').toBe(200);
  await expect(page.getByRole('main')).toBeVisible();
}
function composer(page) { return page.getByRole('form', {name:'업무 입력',exact:true}); }
async function absentEverywhere(page, response, canaries) {
  const html = await page.content();
  const initial = await response.text();
  const headers = JSON.stringify(response.headers());
  for (const value of canaries) {
    expect(value).toMatch(/^CONSOLEPRIVATE[A-Z0-9]{24}$/);
    expect(initial).not.toContain(value);
    expect(html).not.toContain(value);
    expect(headers).not.toContain(value);
  }
}

test('draft save preserves exact acknowledged input across reload', async ({ page }, info) => {
  const f = fixture(info); await open(page, f);
  await expect(composer(page)).toHaveCount(1);
  await expect(page.getByLabel('기본급', {exact:true})).toHaveValue('1200000');
  await page.getByLabel('기본급', {exact:true}).fill('1234567');
  const response = page.waitForResponse(r => r.request().method() === 'POST' && new URL(r.url()).pathname === f.pathname);
  await page.getByRole('button', {name:'저장',exact:true}).click();
  expect((await response).status()).toBe(303);
  await expect(page.getByText('서버에 저장됨',{exact:true})).toBeVisible();
  await page.reload();
  await expect(page.getByLabel('기본급',{exact:true})).toHaveValue('1234567');
  await expect(page.getByText(f.company_label,{exact:true})).toBeVisible();
});

test('invalid submission preserves input and exposes linked errors', async ({ page }, info) => {
  const f = fixture(info); await open(page, f);
  await page.getByLabel('기본급',{exact:true}).fill('1..2');
  const response = page.waitForResponse(r => r.request().method() === 'POST' && new URL(r.url()).pathname === f.pathname);
  await page.getByRole('button',{name:'상신',exact:true}).click();
  expect((await response).status()).toBe(422);
  const input = page.getByLabel('기본급',{exact:true});
  await expect(input).toHaveValue('1..2');
  await expect(input).toHaveAttribute('aria-invalid','true');
  const summary = page.getByRole('alert');
  await expect(summary).toBeVisible();
  const link = summary.getByRole('link',{name:/기본급/});
  await expect(link).toHaveCount(1);
  const target = await link.getAttribute('href');
  expect(target).toMatch(/^#[a-zA-Z][a-zA-Z0-9_-]*$/);
  expect(await input.getAttribute('id')).toBe(target.slice(1));
  await link.click();
  await expect(input).toBeFocused();
});

test('stale save preserves acknowledged inputs and offers recovery', async ({page},info) => {
  const f = fixture(info); await open(page,f);
  const other = await page.context().newPage();
  await open(other,f); // Both pages now hold the same acknowledged revision.
  await other.getByLabel('기본급',{exact:true}).fill('1250000');
  const otherResponse = other.waitForResponse(r => r.request().method() === 'POST' && new URL(r.url()).pathname === f.pathname);
  await other.getByRole('button',{name:'저장',exact:true}).click();
  expect((await otherResponse).status()).toBe(303);
  await expect(other.getByText('서버에 저장됨',{exact:true})).toBeVisible();
  await other.close();
  await page.getByLabel('기본급',{exact:true}).fill('1234567');
  const response = page.waitForResponse(r => r.request().method() === 'POST' && new URL(r.url()).pathname === f.pathname);
  await page.getByRole('button',{name:'저장',exact:true}).click();
  expect((await response).status()).toBe(409);
  await expect(page.getByLabel('기본급',{exact:true})).toHaveValue('1234567');
  await expect(page.getByRole('alert')).toBeVisible();
  const recovery = page.getByRole('link',{name:'현재 내용 확인',exact:true});
  await expect(recovery).toBeVisible();
  await recovery.click();
  await expect(page.getByLabel('기본급',{exact:true})).toHaveValue('1250000');
});

test('scoped review presents permitted evidence without hidden coverage', async ({page},info) => {
  const f=fixture(info); await page.context().addCookies(f.cookies);
  const response=await page.goto(f.url); expect(response.status()).toBe(200);
  for(const text of ['1200000',f.visible_evidence_narrative,f.visible_subject_label])
    await expect(page.getByText(text,{exact:true})).toBeVisible();
  expect(f.canaries).toHaveLength(4); // Hidden employee, wage, narrative, global coverage marker.
  await absentEverywhere(page,response,f.canaries);
  await expect(page.getByRole('button',{name:'결정',exact:true})).toBeVisible();
});

test('historical own publication remains nonpayable with correction entry', async ({page},info) => {
  // Employment ended; the Account remains ACTIVE and current own-field policy permits this publication.
  const f=fixture(info); await open(page,f);
  for(const text of ['1200000','NONPAYABLE_REVIEW',f.historical_period])
    await expect(page.getByText(text,{exact:true})).toBeVisible();
  await expect(page.getByRole('button',{name:'지급',exact:true})).toHaveCount(0);
  await expect(page.getByRole('link',{name:'정정 요청',exact:true})).toBeVisible();
});

test('read-only projection has no state-changing form or concealed private fields', async ({page},info) => {
  const f=fixture(info); await page.context().addCookies(f.cookies);
  const response=await page.goto(f.url); expect(response.status()).toBe(200);
  await expect(page.getByText(f.visible_subject_label,{exact:true})).toBeVisible();
  expect(f.canaries).toHaveLength(3);
  await absentEverywhere(page,response,f.canaries);
  await expect(composer(page)).toHaveCount(0);
  // This fixture grants no business mutation; its only separately permitted
  // global mutation is the initial BW31 own-session logout registration.
  // An arbitrarily renamed business form cannot evade the scan.
  expect(unexpectedMutations(await page.evaluate(effectiveSubmissions),f.origin)).toEqual([]);
  for (const control of await page.locator('[data-action-key]').all())
    expect(await control.getAttribute('data-action-key')).toBe('account.session.logout');
});
