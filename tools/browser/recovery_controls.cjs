'use strict';
// Private structural oracle only. Current authority, canonical input equality,
// proof validation and durable effects MUST come from the real parent owner.
const assert=require('node:assert/strict');
const crypto=require('node:crypto');
const UUID=/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const nil='00000000-0000-0000-0000-000000000000';
const digest=x=>crypto.createHash('sha256').update(JSON.stringify(x)).digest('hex');
function base(e){
  for(const value of [e.company,e.command]){assert.match(value,UUID);assert.notEqual(value,nil);}
  assert.ok(['install','grant','revoke'].includes(e.operation));
  assert.equal(new URL(e.origin).origin,e.origin);
  return `/companies/${e.company}/policy/payroll-read`;
}
function recoveryFormsSafe(forms,state,e){
  try{
    const b=base(e);assert.ok(Array.isArray(forms));
    assert.ok(['uncertain','pending','expired','not-visible'].includes(state));
    assert.ok(['none','accepted','held'].includes(e.recovery));
    if(state==='pending')assert.equal(e.recovery,'accepted');
    if(['expired','not-visible'].includes(state))assert.equal(e.recovery,'none');
    if(e.recovery==='none'){assert.notEqual(state,'pending');assert.equal(forms.length,0);return true;}
    assert.equal(forms.length,1);const f=forms[0];
    assert.equal(f.method,'post');assert.equal(f.enctype,'application/x-www-form-urlencoded');
    assert.equal(f.override,false);assert.equal(f.autosubmit,false);
    assert.equal(f.imageSubmitters,0);assert.equal(f.submitButtons,1);assert.equal(f.submitLabel,'같은 요청 이어서 처리');
    assert.deepEqual(f.submitNames,['']);assert.deepEqual(f.submitDisabled,[false]);
    let fields={};let path;
    if(e.recovery==='accepted'){
      assert.ok(['pending','uncertain'].includes(state));
      path=`${b}/requests/${e.operation}/${e.command}/retry`;
    }else{
      assert.equal(state,'uncertain');fields=e.originalFields;
      assert.ok(fields&&typeof fields==='object');
      const names=['command_id','expected_company_epoch'];
      if(e.operation==='grant')names.push('recipient_account_id','expected_role_revision','assignment_id','expected_assignment_revision','expires_at_local');
      if(e.operation==='revoke')names.push('expected_role_revision','expected_assignment_revision');
      assert.deepEqual(Object.keys(fields).sort(),names.sort());
      assert.equal(fields.command_id,e.command);
      for(const value of Object.values(fields))assert.ok(typeof value==='string'&&value.length>0);
      path=e.operation==='install'?b+'/catalog':e.operation==='grant'?b+'/grants':b+'/grants/'+e.assignment+'/revoke';
      if(e.operation==='revoke'){assert.match(e.assignment,UUID);assert.notEqual(e.assignment,nil);}
    }
    assert.equal(f.action,path);assert.equal(new URL(f.action,e.origin).href,e.origin+path);
    assert.deepEqual(f.controls.map(x=>x.name).sort(),['csrf_proof',...Object.keys(fields)].sort());
    for(const c of f.controls){
      assert.equal(c.type,'hidden');assert.equal(c.disabled,false);
      if(c.name==='csrf_proof'){assert.equal(c.proofPresent,true);assert.equal(c.value,undefined);}
      else assert.equal(c.value,fields[c.name]);
    }
    return true;
  }catch{return false;}
}
function snapshotRecoveryForms(nodes){
  return nodes.map(f=>({
    action:f.getAttribute('action'),method:f.getAttribute('method'),enctype:f.enctype,
    override:[f,...f.querySelectorAll('*'),...f.elements].some(n=>n.hasAttribute('formaction')||n.hasAttribute('formmethod')||n.hasAttribute('formenctype')||n.hasAttribute('formtarget')),
    autosubmit:[f,...f.querySelectorAll('*'),...f.elements].some(n=>[...n.attributes].some(a=>/^on/i.test(a.name))),
    imageSubmitters:[...f.ownerDocument.querySelectorAll('input[type="image" i]')].filter(n=>n.form===f).length,
    submitButtons:[...f.elements].filter(n=>n.type==='submit').length,
    submitNames:[...f.elements].filter(n=>n.type==='submit').map(n=>n.name),
    submitDisabled:[...f.elements].filter(n=>n.type==='submit').map(n=>n.matches(':disabled')),
    submitLabel:[...f.elements].filter(n=>n.type==='submit').map(n=>n.innerText||n.value).join(''),
    controls:[...f.elements].filter(n=>n.type!=='submit').map(n=>({name:n.name,type:n.type,disabled:n.matches(':disabled'),
      ...(n.name==='csrf_proof'?{proofPresent:typeof n.value==='string'&&n.value.length>0}:{value:n.value})})),
  }));
}
async function assertRecoveryForms(page,state,e){
  assert.equal(new URL(page.url()).origin,e.origin);
  assert.equal(typeof e.verifyRecoveryProjection,'function');
  const forms=await page.locator('form').evaluateAll(snapshotRecoveryForms);
  assert.equal(recoveryFormsSafe(forms,state,e),true);
  // No proof/token bytes are retained or passed to this report. The parent uses
  // the genuine current session/owner to validate the issued proof independently.
  const selectors={company:e.company,operation:e.operation,command:e.command,state,recovery:e.recovery};
  const proof=await e.verifyRecoveryProjection({forms,selectors});
  assert.equal(proof.source_projection_verified,true);
  assert.equal(proof.forms_sha256,digest(forms));assert.equal(proof.selectors_sha256,digest(selectors));
  assert.equal(proof.no_get_effects,true);
  if(e.recovery==='accepted'){
    assert.match(proof.intake_receipt_id,UUID);assert.notEqual(proof.intake_receipt_id,nil);
    assert.match(proof.canonical_input_sha256,/^[0-9a-f]{64}$/);
    assert.equal(proof.exact_original_input_verified,true);assert.equal(proof.current_proof_verified,true);
    assert.equal(proof.accepted_input_preserved,true);
    if(state==='pending')assert.equal(proof.current_state,'AcceptedPending');
  }
  if(e.recovery==='held'){
    assert.equal(proof.exact_held_input_verified,true);assert.equal(proof.current_proof_verified,true);
    assert.equal(proof.acceptance_claimed,false);
  }
  if(state==='not-visible')assert.equal(proof.acceptance_claimed,false);
  if(state==='expired'){assert.equal(proof.current_state,'AcceptedExpired');assert.equal(proof.accepted_input_preserved,true);}
}
module.exports={recoveryFormsSafe,assertRecoveryForms,snapshotRecoveryForms};
