const {test,expect}=require('@playwright/test');
const {effectiveSubmissions,unexpectedMutations}=require('./effective-submissions.cjs');
// Evidence-machinery controls, not product UI tests. No HTTP action is submitted.
const cases=[
  ['ordinary read form', '<form action="/_ui/payroll"><button>read</button></form>', 0],
  ['permitted own logout', '<form method="post" action="/api/v2/auth/logout"><button>logout</button></form>', 0],
  ['renamed business form', '<form aria-label="unrelated" method="post" action="/business/write"><button>write</button></form>', 2],
  ['GET form with POST submit override', '<form id="x" method="get" action="/_ui/payroll"><button formmethod="post" formaction="/business/write">write</button></form>', 1],
  ['external associated submit override', '<form id="x" method="get" action="/_ui/payroll"></form><button form="x" formmethod="post" formaction="/business/write">write</button>', 1],
  ['logout form with hostile submit action', '<form method="post" action="/api/v2/auth/logout"><button formaction="https://example.invalid/leak">write</button></form>', 1],
];
for(const [name,html,count] of cases) test(name,async({page})=>{
  await page.setContent('<base href="http://127.0.0.1/">'+html);
  expect(unexpectedMutations(await page.evaluate(effectiveSubmissions),'http://127.0.0.1')).toHaveLength(count);
});
