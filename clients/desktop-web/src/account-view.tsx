import type {Page, Projection} from "./account-projection";

function AttemptStatus() {
  return <>
    <p id="native-status" className="status" role="status" aria-live="polite" tabIndex={-1}/>
    <p id="native-error" className="error" role="alert" hidden/>
    <button id="native-cancel" className="button secondary" type="button" hidden>취소</button>
    <button id="native-recheck" className="button secondary" type="button" hidden>결과 다시 확인</button>
    <button id="native-retry-finish" className="button secondary" type="button" hidden>같은 요청 다시 제출</button>
    <a id="native-continue" className="text-link" href="/account" hidden>로그인으로 확인</a>
    <a id="native-reload-terms" className="text-link" href="/account/register" hidden>새 약관 확인</a>
    <noscript><p className="notice">패스키를 사용하려면 이 브라우저에서 JavaScript를 허용해 주세요. 약관과 안내는 그대로 읽을 수 있습니다.</p></noscript>
  </>;
}
function CompanyStatus() {
  return <>
    <p id="company-status" className="status" role="status" aria-live="polite" tabIndex={-1}/>
    <p id="company-error" className="error" role="alert" hidden/>
    <a id="company-result" className="text-link" hidden>요청 결과 확인</a>
  </>;
}
function Reference({account}: {account: string}) {
  return <section className="workspace-state account-reference" aria-labelledby="account-reference-title">
    <h2 id="account-reference-title">내 계정 참조</h2>
    <p id="account-reference-help">이 참조를 공유하면 권한이 있는 운영자가 관리할 계정으로 선택할 수 있습니다. 로그인 비밀이나 사람의 신원 증명, 업무 권한이 아닙니다.</p>
    <label htmlFor="own-account-reference">내 계정 참조</label>
    <input id="own-account-reference" type="text" data-account-reference="" defaultValue={account} readOnly aria-describedby="account-reference-help" spellCheck={false}/>
    <button className="button secondary" type="button" data-account-copy="">계정 참조 복사</button>
    <p className="status" role="status" aria-live="polite" data-account-copy-status=""/>
  </section>;
}
function Recipient({account}: {account: string}) {
  return <fieldset className="account-recipient">
    <legend>관리할 계정</legend>
    <label className="recipient-choice"><input type="radio" name="administrator_mode" value="self" data-company-recipient-mode="" defaultChecked/>내 계정</label>
    <label className="recipient-choice"><input type="radio" name="administrator_mode" value="other" data-company-recipient-mode=""/>다른 계정</label>
    <div data-company-other="" hidden>
      <label htmlFor="company-administrator-reference">관리할 계정 참조</label>
      <input id="company-administrator-reference" name="administrative_account_reference" type="text" data-company-administrator-reference="" disabled autoComplete="off" autoCapitalize="none" spellCheck={false} minLength={36} maxLength={36} aria-describedby="company-recipient-help"/>
    </div>
    <section className="workspace-state" aria-labelledby="company-recipient-title">
      <h2 id="company-recipient-title">선택한 관리 계정</h2>
      <p className="account-id" data-company-selected-account="">{account}</p>
      <p id="company-recipient-help">이 계정에 이 회사의 정보 열람과 제한된 권한 관리 기능을 연결합니다. 급여·인사 권한과 운영자 권한은 포함되지 않습니다.</p>
    </section>
  </fieldset>;
}
function Content({page, groups}: {page: Page; groups: Projection["groups"]}) {
  switch (page.kind) {
    case "public": return <section className="entry-card welcome">
      <p className="eyebrow">CONSOLE</p><h1>하나의 계정으로,<br/>업무를 이어가세요.</h1>
      <p className="lead">로그인하면 지금 사용할 수 있는 업무 공간을 확인할 수 있습니다.</p>
      <div className="actions"><a className="button primary" href="/account">로그인</a><a className="button secondary" href="/account/register">계정 만들기</a></div>
      <p className="supporting">아직 연결된 회사나 업무가 없어도 계정을 만들 수 있습니다.</p>
    </section>;
    case "sign_in": return <section className="entry-card" data-account-state="anonymous">
      <p className="eyebrow">다시 만나서 반갑습니다</p><h1>Console에 로그인</h1>
      <p className="lead">기기에 저장된 패스키로 본인의 계정에 로그인하세요.</p>
      <form data-native-action="login">
        <label htmlFor="native-account-choice">패스키 계정 선택</label>
        <input id="native-account-choice" type="text" autoComplete="username webauthn" aria-describedby="native-choice-help"/>
        <p id="native-choice-help" className="supporting">아래 버튼을 누른 뒤 이 입력란에서 브라우저가 제안하는 패스키를 선택하세요. 이름이나 이메일을 입력할 필요는 없습니다.</p>
        <button className="button primary wide" type="button" data-native-submit="" disabled>패스키로 로그인</button><AttemptStatus/>
      </form>
      <p className="other-path">처음 방문하셨나요? <a href="/account/register">계정 만들기</a></p>
    </section>;
    case "register": return <section className="entry-card registration" data-account-state="anonymous">
      <p className="eyebrow">내 계정의 시작</p><h1>계정 만들기</h1>
      <p className="lead">약관을 확인하고 패스키를 만드세요. 회사 소속이나 직책은 지금 입력하지 않습니다.</p>
      <form data-native-action="register" data-terms-version={page.version}>
        <fieldset><legend>필수 약관을 각각 확인해 주세요</legend>{page.items.map((item, index) => <article className="terms-item" key={item.kind}>
          <label className="check-label" htmlFor={`native-terms-${index}`}><input id={`native-terms-${index}`} type="checkbox" data-terms-kind={item.kind} required/><span>{item.title}</span></label>
          <details><summary>약관 내용 보기</summary><div className="terms-content">{item.content}</div><a href={item.content_url} target="_blank" rel="noopener noreferrer">문서 새 창으로 열기</a></details>
        </article>)}</fieldset>
        <p className="supporting">다음 단계에서 기기의 화면 잠금이나 보안 키로 패스키를 만듭니다.</p>
        <button className="button primary wide" type="button" data-native-submit="" disabled>패스키로 계정 만들기</button><AttemptStatus/>
      </form>
      <p className="other-path">이미 계정이 있나요? <a href="/account">로그인</a></p>
    </section>;
    case "account": return <section className="entry-card" data-account-state="active">
      <p className="eyebrow">내 CONSOLE</p><h1>계정에 로그인했습니다</h1><Reference account={page.account_id}/>
      {page.context.kind === "companies" ? <section className="workspace-state" data-context-state="populated"><h2>회사 업무 공간</h2><ul>{page.context.companies.map(([identifier, name]) => <li key={identifier}><a href={`/companies/${identifier}`}>{name}</a></li>)}</ul></section>
        : page.context.kind === "empty" ? <section className="workspace-state" data-context-state="empty"><span className="state-label">연결된 회사 없음</span><h2>계정은 준비되었습니다</h2><p>현재 이 계정으로 접근할 수 있는 회사 업무 공간이 없습니다. 업무 접근 권한이 연결되면 이곳에서 이어갈 수 있습니다.</p><a href="/account">다시 확인</a></section>
          : <section className="workspace-state" data-context-state="unavailable" role="alert"><span className="state-label">다시 확인이 필요합니다</span><h2>회사 업무 공간을 확인할 수 없습니다</h2><p>계정 로그인은 확인했지만 회사 업무 접근 정보를 불러오지 못했습니다. 잠시 후 다시 확인해 주세요.</p><a href="/account">다시 확인</a></section>}
      {!!groups.length && <section className="workspace-state" aria-labelledby="account-group-work-title"><h2 id="account-group-work-title">그룹 관리</h2><ul>{groups.map(group => <li key={group.group}><p>{group.label}</p><a href={`/groups/${group.group}/identity`}>그룹 신원 확인</a></li>)}</ul></section>}
      {page.company_setup === "eligible" && <a className="button primary" href="/account/companies/new">회사 업무 공간 만들기</a>}
      {page.company_setup === "unavailable" && <section className="workspace-state" role="alert"><h2>업무 공간 등록을 확인할 수 없습니다</h2><p>잠시 후 다시 확인해 주세요.</p><a href="">다시 확인</a></section>}
      {page.can_logout && <form data-native-action="logout" className="logout-form"><button className="button secondary" type="button" data-native-submit="" disabled>로그아웃</button><AttemptStatus/></form>}
    </section>;
    case "company_setup": return <section className="entry-card" data-company-setup="" aria-labelledby="company-setup-title">
      <p className="eyebrow">내 CONSOLE</p><h1 id="company-setup-title">회사 업무 공간 만들기</h1><p className="lead">기존 회사가 사용할 콘솔 업무 공간을 등록합니다.</p>
      {page.command_id && <p className="notice">이 계정에 저장된 요청을 찾지 못했습니다. 입력 내용을 다시 확인하고 제출하면 현재 주소의 같은 요청 번호를 사용합니다.</p>}
      <form data-company-enrollment="" data-account-id={page.account_id} data-command-id={page.command_id ?? undefined}>
        <fieldset><legend>회사 업무 공간 정보</legend><label htmlFor="company-name">회사 이름</label><input id="company-name" name="name" type="text" autoComplete="organization" required maxLength={256} aria-describedby="company-name-help"/>
          <p id="company-name-help" className="supporting">업무에서 사용하는 회사 이름을 입력하세요.</p>
          <label htmlFor="company-slug">업무 공간 식별자</label><input id="company-slug" name="slug" type="text" autoComplete="off" autoCapitalize="none" spellCheck={false} required maxLength={63} pattern="[a-z0-9](?:[a-z0-9\-]{0,61}[a-z0-9])?" aria-describedby="company-slug-help"/>
          <p id="company-slug-help" className="supporting">영문 소문자와 숫자, 하이픈(-)으로 입력하세요. 처음과 끝에는 하이픈을 사용할 수 없습니다.</p></fieldset>
        <Recipient account={page.account_id}/><button className="button primary wide company-submit" type="submit" disabled>회사 업무 공간 만들기</button><CompanyStatus/>
        <noscript><p className="notice">업무 공간을 등록하려면 이 브라우저에서 JavaScript를 허용해 주세요.</p></noscript>
      </form><a href="/account">내 계정으로</a>
    </section>;
    case "company_created": return <section className="entry-card" data-company-outcome="committed">
      <p className="eyebrow">회사 업무 공간</p><h1>생성 완료</h1><p className="lead">요청이 처리되었습니다. 이 주소에서 결과를 다시 확인할 수 있습니다.</p>
      <dl className="request-reference"><dt>관리할 계정 참조</dt><dd>{page.administrative_account_id}</dd><dt>요청 번호</dt><dd>{page.command_id}</dd><dt>처리 기록</dt><dd>{page.receipt_id}</dd></dl>
      {page.company && <section className="workspace-state"><h2>{page.company[1]}</h2><a className="button primary" href={`/companies/${page.company[0]}`}>업무 공간 열기</a></section>}
      <a href="/account">내 계정으로</a>
    </section>;
    case "company_pending": return <section className="entry-card" data-company-outcome="pending">
      <p className="eyebrow">저장된 등록 요청</p><h1>아직 생성이 완료되지 않았습니다</h1><p className="lead">입력한 내용이 저장되어 있습니다. 같은 요청으로 다시 시도하면 중복으로 만들지 않습니다.</p>
      <form data-company-enrollment="" data-account-id={page.account_id} data-command-id={page.command_id} data-group-id={page.group_id ?? undefined}>
        <label htmlFor="company-name">회사 이름</label><input id="company-name" name="name" defaultValue={page.name} readOnly/>
        <label htmlFor="company-slug">업무 공간 식별자</label><input id="company-slug" name="slug" defaultValue={page.slug} readOnly/>
        <label htmlFor="company-retained-administrator">관리할 계정 참조</label><input id="company-retained-administrator" type="text" defaultValue={page.account_id} readOnly/>
        <dl className="request-reference"><dt>요청 번호</dt><dd>{page.command_id}</dd></dl>
        <button className="button primary wide company-submit" type="submit" disabled>같은 요청으로 다시 시도</button>
        <p className="supporting">등록 요청을 취소하면 다시 제출할 수 없습니다. 이미 생성이 완료된 업무 공간은 취소되지 않습니다.</p>
        <button className="button secondary" type="button" data-company-cancel="" disabled>등록 요청 취소</button><CompanyStatus/>
        <noscript><p className="notice">다시 제출하려면 이 브라우저에서 JavaScript를 허용해 주세요.</p></noscript>
      </form><a href="/account">내 계정으로</a>
    </section>;
    case "company_uncertain": return <section className="entry-card" data-company-outcome="uncertain"><p className="eyebrow">원래 요청 확인</p><h1>아직 결과를 확인할 수 없습니다</h1><p className="lead">요청이 처리 중일 수 있습니다. 새로 등록하지 말고 이 주소에서 같은 요청의 결과를 다시 확인해 주세요.</p><a className="button primary" href="">같은 요청 결과 다시 확인</a><a className="text-link" href="/account">내 계정으로</a></section>;
    case "company_terminal": return <section className="entry-card" data-company-outcome={page.expired ? "expired" : "cancelled"}><p className="eyebrow">회사 등록 요청</p><h1>{page.expired ? "요청이 만료되었습니다" : "요청이 취소되었습니다"}</h1><p className="lead">이 요청으로 생성된 회사 업무 공간은 없습니다.</p><dl className="request-reference"><dt>요청 번호</dt><dd>{page.command_id}</dd></dl><a className="button primary" href="/account">내 계정으로</a></section>;
    case "refused": return <section className="entry-card"><p className="eyebrow">요청 확인</p><h1>이 요청을 열 수 없습니다</h1><p className="lead">Console의 로그인 화면에서 다시 시작해 주세요.</p><a className="button primary" href="/account">로그인 화면으로</a></section>;
    case "unavailable": return <section className="entry-card" role="alert"><p className="eyebrow">잠시 기다려 주세요</p><h1>지금은 계정을 확인할 수 없습니다</h1><p className="lead">계정이나 약관 정보를 불러오지 못했습니다. 잠시 후 다시 시도해 주세요.</p><a className="button primary" href="">다시 시도</a><a className="text-link" href="/">시작 화면으로</a></section>;
  }
}
export function AccountView({projection}: {projection: Projection}) {
  return <div data-console-react-account="mounted">
    <a className="skip-link" href="#main-content">본문 바로가기</a>
    <header className="entry-header"><a className="brand" href="/" aria-label="Console 시작 화면">Console<span className="brand-dot" aria-hidden="true"/></a></header>
    <main id="main-content" tabIndex={-1}><Content page={projection.page} groups={projection.groups}/></main>
    <footer className="entry-footer">필요한 업무와 정보가 한곳에.</footer>
  </div>;
}
