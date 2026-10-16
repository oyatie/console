from pathlib import Path
P=Path('/private/tmp/console-company-entry-read-tests-20260920');R=Path('/private/tmp/console-client-release-20260919')
source=Path('/private/tmp/console-native-company-setup-tests-retained-v4/company-journey.cjs').read_text()
s=source.replace("const MUTATION_PATHS=['/api/v2/auth/registration/start','/api/v2/auth/registration/finish','/api/v2/companies/enroll'];","const MUTATION_PATHS=['/api/v2/auth/registration/start','/api/v2/auth/registration/finish'];")
a=s.index('function documentPaths(');b=s.index('function completeDocuments(',a)
s=s[:a]+"function documentPaths(result){return ['/','/account/register','/account','/account','/account/companies/new'];}\n"+s[b:]
a=s.index('function completeObservations(');b=s.index('function leafStatus(',a)
s=s[:a]+'''function completeObservations(r){return completeDocuments(r)&&completeMutations(r)&&r.browser_version==='151.0.7922.34'&&r.relay_failure!==true&&r.tls_client_error!==true&&r.root_status===200&&r.registration_wire===true&&r.resident===true&&r.cookie_security===true&&r.literal_secret_absent===true&&r.external_requests===0&&r.checkpoints?.join(',')==='ENROLLED,PREVIEW_PRESERVED'&&['reflow_root_320','reflow_register_320','reflow_account_320','reflow_setup_320','keyboard_skip','keyboard_registration','keyboard_terms','keyboard_registration_submit','keyboard_company_entry','preview_has_no_form','preview_enter_preserved','business_input_not_stored'].every(key=>r[key]===true)&&Array.isArray(r.preview_enters)&&r.preview_enters.length===2&&r.preview_enters.every((e,i)=>e.field===['name','slug'][i]&&e.no_requests===true&&e.url_unchanged===true&&e.history_unchanged===true&&e.inputs_preserved===true);}
'''+s[b:]
s=s.replace('REAL_NATIVE_COMPANY_UI_BROWSER_LEAF','REAL_NATIVE_COMPANY_PREVIEW_BROWSER_DEPENDENCY').replace('Three independent DB acknowledgements plus actual parent-owned designation are required; parent remains final acceptance owner. Grant/revoke and dropped-response recovery are separate acceptance leaves.','Two independent DB checkpoints plus actual parent-owned designation required. Preview is unreleased; Company command, grant/revoke and recovery remain separate requirements.')
needle="  observeMutations(context,origin,result);";s=s.replace(needle,needle+"\n  const observedRequests=[];context.on('request',request=>observedRequests.push({method:request.method(),url:request.url()}));")
a=s.index("  result.recipient_consequence=");b=s.index("  requireFact(!result.relay_failure",a)
s=s[:a]+'''  result.preview_has_no_form=await page.locator('form,[type="submit"],[data-native-action]').count()===0;requireFact(result.preview_has_no_form,'COMPANY_FORM');
  result.reflow_setup_320=await reflow320();requireFact(result.reflow_setup_320,'REFLOW_320');
  await nameInput.fill(companyName);await slugInput.fill(slug);await page.waitForLoadState('networkidle');
  result.preview_enters=[];
  for(const [field,input] of [['name',nameInput],['slug',slugInput]]){
   await input.focus();const before={url:page.url(),requests:observedRequests.length,history:await page.evaluate(()=>history.length)};
   await page.keyboard.press('Enter');await page.waitForTimeout(350);
   const evidence={field,no_requests:observedRequests.length===before.requests,url_unchanged:page.url()===before.url,history_unchanged:await page.evaluate(()=>history.length)===before.history,inputs_preserved:await nameInput.inputValue()===companyName&&await slugInput.inputValue()===slug};
   result.preview_enters.push(evidence);requireFact(Object.entries(evidence).filter(([key])=>key!=='field').every(([,value])=>value===true),'INPUT_PRESERVATION');
  }
  result.preview_enter_preserved=true;
  result.business_input_not_stored=await page.evaluate(({name,slug})=>!JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)}).includes(name)&&!JSON.stringify({local:Object.entries(localStorage),session:Object.entries(sessionStorage)}).includes(slug),{name:companyName,slug});requireFact(result.business_input_not_stored,'BUSINESS_STORAGE');
  await secretFree();await capture('04-company-preview-input-preserved.png');
  await checkpoint('PREVIEW_PRESERVED',account);
'''+s[b:]
(P/'company-preview.cjs').write_text(s)
original=(R/'backend/app/tests/auth_rest/native_company_browser.rs').read_text();rust=original.replace('async fn native_company_real_browser_create_reopen_and_workspace','async fn native_company_real_browser_preview_preserves_enter_without_commands').replace('CONSOLE_COMPANY_BROWSER_','CONSOLE_COMPANY_PREVIEW_BROWSER_')
a=rust.index('        let created = browser_owner_event');b=rust.index('        let final_event = browser_owner_event',a)
rust=rust[:a]+'''        let observed = browser_owner_event(&mut events).await;
        assert_eq!(browser_checkpoint(&observed, "PREVIEW_PRESERVED"), account);
        assert!(before_company == all_rows(&pool).await, "preview typing/Enter made durable effects");
        assert_no_company_identity(&pool, account).await;
        checkpoint_receipts.push("PREVIEW_PRESERVED");
        browser_owner_continue(&mut input, "PREVIEW_PRESERVED").await;
'''+rust[b:]
rust=rust.replace('INDEPENDENT_NATIVE_COMPANY_UI_DATABASE_CHECKPOINTS','INDEPENDENT_NATIVE_COMPANY_PREVIEW_DATABASE_CHECKPOINTS').replace('actual native enrollment/designation/Company route; does not prove grant/revoke, lost-response, human usability, WCAG or production exposure','actual native enrollment/designation/preview only; no Company command or release acceptance; no human usability/WCAG/production exposure')
# Retained full Company browser test unchanged; only append this distinct dependency.
(P/'native_company_browser.rs').write_text(original+'\n'+rust)
