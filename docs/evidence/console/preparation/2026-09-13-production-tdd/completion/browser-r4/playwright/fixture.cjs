const {expect}=require('@playwright/test');
const fs=require('node:fs');
const crypto=require('node:crypto');
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

module.exports={fixture,open};

function submittedCommand(request){
 const type=request.headers()['content-type']||'';const body=request.postData();expect(body).not.toBeNull();
 const input=type.includes('application/json')?JSON.parse(body):Object.fromEntries(new URLSearchParams(body));
 expect(input.operation).toBe('save');
 const proof=request.headers()['x-console-csrf']||input.csrf_proof;
 expect(typeof proof).toBe('string');expect(proof.length).toBeGreaterThan(0);
 expect(input.command_id).toMatch(/^[0-9a-f-]{36}$/i);return input.command_id;
}
function writeObservation(observation){
 const file=process.env.CONSOLE_BROWSER_OBSERVATION;
 if(!file)throw new Error('PREREQUISITE: private browser observation path missing');
 fs.writeFileSync(file,JSON.stringify(observation),{flag:'wx',mode:0o600});
}
module.exports.submittedCommand=submittedCommand;module.exports.writeObservation=writeObservation;
