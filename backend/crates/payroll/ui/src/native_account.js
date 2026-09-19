// Native browser transport only. The server owns identity, terms and every action.
// No cookie inspection, storage, telemetry payloads or client authority.
const form = document.querySelector('form[data-native-action]');
if (form) {
  const action = form.dataset.nativeAction;
  const submit = form.querySelector('[data-native-submit]');
  const status = document.getElementById('native-status');
  const error = document.getElementById('native-error');
  const cancel = document.getElementById('native-cancel');
  const recheck = document.getElementById('native-recheck');
  const retryFinish = document.getElementById('native-retry-finish');
  const continuation = document.getElementById('native-continue');
  const reloadTerms = document.getElementById('native-reload-terms');
  const choice = document.getElementById('native-account-choice');
  const consentControls = Array.from(form.querySelectorAll('[data-terms-kind]'));
  let returnFocus = false;
  let busy = false;
  let blocked = false;
  let controller;
  let pendingFinish;
  const announce = text => { status.textContent = text; };
  const problem = text => { error.hidden = false; error.textContent = text; };
  const clearProblem = () => { error.textContent = ''; error.hidden = true; };
  const failure = (code, httpStatus = 0) => Object.assign(new Error(code), {code, httpStatus});

  async function request(path, body, proof) {
    const headers = {Accept: 'application/json'};
    if (body !== undefined) headers['Content-Type'] = 'application/json';
    if (proof !== undefined) headers['X-Console-CSRF'] = proof;
    let response;
    try {
      response = await fetch(path, {
        method: body === undefined ? 'GET' : 'POST', headers,
        body: body === undefined ? undefined : JSON.stringify(body),
        credentials: 'same-origin', cache: 'no-store', redirect: 'error',
        signal: controller ? AbortSignal.any([AbortSignal.timeout(15000), controller.signal]) : AbortSignal.timeout(15000),
      });
    } catch { throw failure('transport_unconfirmed'); }
    let value;
    try { value = await response.json(); }
    catch { throw failure('transport_unconfirmed', response.status); }
    if (!response.ok) throw failure(value?.error?.code ?? 'request_failed', response.status);
    return value;
  }

  function accountFromHandle(value) {
    if (typeof value !== 'string' || !/^[A-Za-z0-9_-]+$/.test(value)) throw failure('identity_unconfirmed');
    const bytes = Uint8Array.from(atob(value.replace(/-/g, '+').replace(/_/g, '/')), c => c.charCodeAt(0));
    if (bytes.length !== 16) throw failure('identity_unconfirmed');
    const hex = Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('');
    return `${hex.slice(0,8)}-${hex.slice(8,12)}-${hex.slice(12,16)}-${hex.slice(16,20)}-${hex.slice(20)}`;
  }

  function explain(cause) {
    const code = cause?.code;
    if (cause?.name === 'AbortError' || cause?.name === 'NotAllowedError') {
      returnFocus = true;
      announce('패스키 확인을 완료하지 않았습니다. 준비되면 다시 시도해 주세요.');
    } else if (code === 'terms_changed') {
      blocked = true;
      pendingFinish = undefined;
      for (const box of form.querySelectorAll('[data-terms-kind]')) box.checked = false;
      reloadTerms.hidden = false;
      problem('약관이 변경되었습니다. 새 약관을 확인하고 각각 다시 동의해 주세요.');
    } else if (code === 'already_authenticated') {
      blocked = true; continuation.hidden = false;
      problem('이미 로그인한 계정이 있습니다. 계정 화면에서 이어가세요.');
    } else if (code === 'terms_acceptance_required') {
      blocked = true;
      continuation.hidden = false;
      problem('이 계정의 약관 동의 정보를 확인하지 못해 로그인할 수 없습니다. 이 화면에서는 동의 정보를 변경할 수 없습니다. 새 계정을 만들지 말고 로그인 화면으로 돌아가 주세요.');
    } else if (code === 'rate_limited' || cause?.httpStatus === 429) {
      problem('요청이 많아 잠시 기다려야 합니다. 잠시 후 직접 다시 시도해 주세요.');
    } else if (code === 'identity_unconfirmed') {
      blocked = true; continuation.hidden = false;
      problem('이번에 선택한 계정과 응답이 일치하는지 확인할 수 없습니다. 로그인 화면에서 계정을 다시 선택해 주세요.');
    } else if (cause?.httpStatus === 401 || code === 'csrf_invalid') {
      continuation.hidden = false;
      problem('로그인 상태 또는 요청의 유효 시간이 바뀌었습니다. 로그인 화면에서 다시 확인해 주세요.');
    } else if (code === 'authority_unavailable' || code === 'navigation_unavailable' || cause?.httpStatus >= 500) {
      problem('계정 서비스를 확인할 수 없습니다. 잠시 후 다시 시도해 주세요.');
    } else {
      problem('요청을 완료했는지 확인할 수 없습니다. 안내를 확인한 뒤 다시 시도해 주세요.');
    }
  }

  async function observeFinish(record) {
    try {
      const current = await request('/api/v2/accounts/me');
      if (current.account_id !== record.account) throw failure('identity_unconfirmed');
      blocked = true;
      window.location.assign('/account');
      return true;
    } catch (cause) {
      if (cause?.code === 'identity_unconfirmed') throw cause;
      return false;
    }
  }

  function offerRecovery(record) {
    pendingFinish = record;
    blocked = true;
    recheck.hidden = false;
    retryFinish.hidden = record.retried;
    continuation.hidden = false;
    problem('완료 응답을 확인하지 못했습니다. 새 계정을 만들거나 새 요청을 시작하지 말고, 결과를 확인하거나 같은 요청을 한 번 다시 제출해 주세요.');
  }

  async function finish(record) {
    try {
      const completed = await request(record.path, record.body);
      if (completed.account?.account_id !== record.account) throw failure('identity_unconfirmed');
      pendingFinish = undefined;
      announce('확인했습니다. 계정으로 이동합니다.');
      blocked = true;
      window.location.assign('/account');
    } catch (cause) {
      if (cause?.code === 'transport_unconfirmed' || cause?.httpStatus >= 500) {
        if (!(await observeFinish(record))) offerRecovery(record);
        return;
      }
      throw cause;
    }
  }

  async function createAccount() {
    if (!form.reportValidity()) return;
    for (const box of consentControls) box.disabled = true;
    const version = form.dataset.termsVersion;
    const acknowledgements = Array.from(form.querySelectorAll('[data-terms-kind]'), box => ({terms_kind: box.dataset.termsKind, accepted: box.checked}));
    announce('패스키를 만들 준비를 하고 있습니다.');
    const started = await request('/api/v2/auth/registration/start', {terms_version: version});
    const wrapper = started.public_key_options;
    const account = accountFromHandle(wrapper?.publicKey?.user?.id);
    // Convert only the publicKey member; all wrapper behavior remains intact.
    const options = {...wrapper, publicKey: PublicKeyCredential.parseCreationOptionsFromJSON(wrapper.publicKey), signal: controller.signal};
    cancel.hidden = false;
    announce('기기의 안내에 따라 패스키를 만들어 주세요.');
    const credential = await navigator.credentials.create(options);
    cancel.hidden = true;
    if (!credential) throw failure('identity_unconfirmed');
    // Reload after a possibly committed finish must return to sign-in, not
    // silently offer a new Account when the in-memory expected UUID is lost.
    window.history.replaceState(null, '', '/account');
    await finish({path:'/api/v2/auth/registration/finish', account, retried:false,
      body:{ceremony_id:started.ceremony_id, credential:credential.toJSON(), accept_terms_version:version, accept_items:acknowledgements}});
  }

  async function signIn() {
    announce('패스키를 선택할 준비를 하고 있습니다.');
    const started = await request('/api/v2/auth/passkey/login/start', {});
    const wrapper = started.public_key_options;
    // In particular, preserve the owner's conditional mediation and empty allow-list.
    const options = {...wrapper, publicKey: PublicKeyCredential.parseRequestOptionsFromJSON(wrapper.publicKey), signal:controller.signal};
    cancel.hidden = false;
    announce('패스키 계정 선택 입력란에서 사용할 계정을 선택해 주세요.');
    const operation = navigator.credentials.get(options);
    choice.focus();
    const credential = await operation;
    cancel.hidden = true;
    if (!credential) throw failure('identity_unconfirmed');
    const assertion = credential.toJSON();
    const account = accountFromHandle(assertion.response?.userHandle);
    await finish({path:'/api/v2/auth/passkey/login/finish', account, retried:false,
      body:{ceremony_id:started.ceremony_id, assertion}});
  }

  async function logout() {
    announce('로그아웃을 확인하고 있습니다.');
    const proof = await request('/api/v2/auth/csrf', undefined, 'fetch');
    const result = await request('/api/v2/auth/logout', {}, proof.csrf_proof);
    if (result.outcome !== 'COMMITTED') throw failure('transport_unconfirmed');
    blocked = true;
    window.location.assign('/');
  }

  async function exclusive(operation) {
    if (busy) return;
    busy = true; submit.disabled = true; recheck.disabled = true; retryFinish.disabled = true;
    clearProblem(); controller = new AbortController();
    try {
      // Account cookies are shared between tabs. Never queue a stale attempt.
      await navigator.locks.request('console.native-account.browser-action', {ifAvailable:true}, async lock => {
        if (!lock) { problem('다른 탭에서 계정 요청을 진행 중입니다. 그 탭에서 완료하거나 취소한 뒤 다시 시도해 주세요.'); return; }
        await operation();
      });
    } catch (cause) { explain(cause); }
    finally {
      busy = false; cancel.hidden = true; submit.disabled = blocked;
      recheck.disabled = false; retryFinish.disabled = false;
      if (action === 'register' && !pendingFinish && !blocked) {
        for (const box of consentControls) box.disabled = false;
      }
      if (controller?.signal.aborted || returnFocus) submit.focus();
      returnFocus = false;
      controller = undefined;
    }
  }

  const start = () => { if (!blocked) void exclusive(action === 'register' ? createAccount : action === 'login' ? signIn : logout); };
  submit.addEventListener('click', start);
  form.addEventListener('submit', event => { event.preventDefault(); start(); });
  cancel.addEventListener('click', () => controller?.abort());
  recheck.addEventListener('click', () => { if (pendingFinish) void exclusive(async () => { if (!(await observeFinish(pendingFinish))) offerRecovery(pendingFinish); }); });
  retryFinish.addEventListener('click', () => { if (pendingFinish && !pendingFinish.retried) void exclusive(async () => {
    const record = pendingFinish;
    if (await observeFinish(record)) return;
    record.retried = true; retryFinish.hidden = true;
    await finish(record);
  }); });
  window.addEventListener('pagehide', () => controller?.abort());
  window.addEventListener('pageshow', event => { if (event.persisted) window.location.reload(); });

  async function initialize() {
    const nativeAvailable = window.isSecureContext && navigator.locks?.request &&
      (action === 'logout' || (window.PublicKeyCredential && navigator.credentials &&
        typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'function' &&
        typeof PublicKeyCredential.parseRequestOptionsFromJSON === 'function' &&
        typeof PublicKeyCredential.prototype.toJSON === 'function'));
    if (!nativeAvailable || (action === 'login' && !(await PublicKeyCredential.isConditionalMediationAvailable?.()))) {
      blocked = true;
      problem('이 브라우저에서는 필요한 패스키 기능을 사용할 수 없습니다. 최신 브라우저의 보안 연결에서 다시 열어 주세요.');
      return;
    }
    submit.disabled = false;
  }
  void initialize().catch(() => { blocked = true; problem('브라우저의 계정 기능을 확인할 수 없습니다. 다른 최신 브라우저에서 다시 시도해 주세요.'); });
}
