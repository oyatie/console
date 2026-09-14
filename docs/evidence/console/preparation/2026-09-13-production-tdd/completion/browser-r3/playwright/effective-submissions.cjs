// Test oracle over the browser's actual HTML form-association rules.
function effectiveSubmissions() {
  const result = [];
  for (const form of document.forms)
    result.push({method:form.method.toLowerCase(),action:form.action});
  for (const control of document.querySelectorAll('button,input')) {
    const form = control.form;
    if (!form || !['submit','image'].includes(control.type)) continue;
    result.push({
      method:(control.hasAttribute('formmethod') ? control.formMethod : form.method).toLowerCase(),
      action:control.hasAttribute('formaction') ? control.formAction : form.action,
    });
  }
  return result;
}
function unexpectedMutations(targets, origin) {
  return targets.filter(target => {
    if (target.method !== 'post') return false;
    const action = new URL(target.action);
    return action.origin !== origin || action.pathname !== '/api/v2/auth/logout';
  });
}
module.exports = {effectiveSubmissions,unexpectedMutations};
