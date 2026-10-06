import type {Page, PeopleRequest, PersonRecord, Registration, Scope} from "./people-projection";

const directory = (company: string) => `/companies/${company}/people`;
function recordName(value: string | null): string {
  return value === null ? "이름 미등록" : value.trim() === "" ? "공백으로 저장된 이름" : value;
}
function recordNumber(value: string | null): string {
  return value === null ? "사번 미등록" : value.trim() === "" ? "공백으로 저장된 사번" : value;
}
function Header({page}: {page: Page}) {
  const scope = "scope" in page ? page.scope : null;
  const peopleCurrent = page.kind === "directory" ? "page" : page.kind === "detail" ? "location" : undefined;
  const registrationCurrent = page.kind === "registration" ? "page" : page.kind === "registration_conflict" || page.kind === "request" ? "location" : undefined;
  const navigation = (desktop: boolean) => <>
    {scope && (scope.directory_link || scope.can_create || (scope.payroll_link && desktop)) && <>
      <p className="nav-group">사람과 조직</p>
      <nav aria-label="사람과 조직 탐색">
        {scope.directory_link && <a href={directory(scope.company)} aria-current={peopleCurrent}>사람</a>}
        {scope.can_create && <a href={`${directory(scope.company)}/new`} aria-current={registrationCurrent}>사람 등록</a>}
        {scope.payroll_link && desktop && <a href={`/companies/${scope.company}/payroll`}>급여</a>}
      </nav>
    </>}
    {scope && (scope.company_link || scope.policy_link) && <>
      <p className="nav-group">관리</p><nav aria-label="회사 관리">
        {scope.company_link && <a href={`/companies/${scope.company}`}>회사 업무 공간</a>}
        {scope.policy_link && <a href={`/companies/${scope.company}/policy`}>권한 관리</a>}
      </nav>
    </>}
    <nav className="native-account-nav" aria-label="계정 탐색"><a href="/account">내 계정 · 업무 공간 선택</a></nav>
  </>;
  return <header className="app native-workspace-header">
    <a className="brand" href="/"><span className="brand-mark" aria-hidden="true">C</span>Console</a>
    <div data-native-navigation="desktop">{navigation(true)}</div>
    <div data-native-navigation="mobile">
      {scope?.payroll_link && <a className="native-payroll-shortcut" data-native-payroll-shortcut="" href={`/companies/${scope.company}/payroll`}>급여</a>}
      <details className="native-mobile-menu"><summary>업무 탐색</summary>
        <div className="native-mobile-menu-body">{navigation(false)}</div>
      </details>
    </div>
  </header>;
}
function Consequences() {
  return <section className="panel people-consequences" aria-labelledby="people-consequence-heading">
    <h2 id="people-consequence-heading">등록하면 무엇이 달라지나요?</h2>
    <div className="policy-panel-body">
      <p>사람 목록에 등록합니다. 고용과 발령은 별도로 승인해야 합니다.</p>
      <p className="supporting">이름과 사번으로 사람을 구분합니다. 이름이 같아도 다른 사람의 기록과 합치지 않습니다.</p>
    </div>
  </section>;
}
function Directory({page}: {page: Extract<Page, {kind: "directory"}>}) {
  const {scope, records, search_number, next_href, after_cursor} = page;
  const path = directory(scope.company), searched = search_number !== null;
  return <section className="panel" aria-labelledby="people-directory-heading">
    <div className="people-list-heading"><h2 id="people-directory-heading">{searched ? "사번 검색 결과" : "사람 목록"}</h2>
      {records.length > 0 && scope.can_create && <a className="policy-button" href={`${path}/new`}>사람 등록</a>}
    </div>
    <form className="people-search" method="get" action={path} role="search">
      <label htmlFor="people-search-number">전체 사번으로 정확히 찾기</label>
      <div className="people-search-controls">
        <input id="people-search-number" name="employee_number" type="search" defaultValue={search_number ?? ""} autoComplete="off" aria-describedby="people-search-help"/>
        <button className="policy-button" type="submit">찾기</button>
        {searched && <a href={path}>검색 지우기</a>}
      </div>
      <p id="people-search-help" className="supporting">현재 회사의 사번 전체를 입력하세요. 이름이나 일부 사번은 검색되지 않습니다.</p>
    </form>
    {records.length === 0 ? <div className="people-empty">
      <span className="people-empty-mark" aria-hidden="true">＋</span>
      <h3>{searched ? "일치하는 사번이 없습니다" : "표시할 사람이 없습니다"}</h3>
      <p>{searched ? (after_cursor ? "현재 회사에서 더 이상 일치하는 사람이 없습니다." : "현재 회사에서 이 사번과 일치하는 사람이 없습니다.") : "현재 목록에 표시할 등록 정보가 없습니다."}</p>
      {!searched && scope.can_create && <a className="policy-button" href={`${path}/new`}>사람 등록</a>}
    </div> : <ul className="directory-list">{records.map(record => {
      const name = recordName(record.legal_name), needsIdentity = record.legal_name === null || record.legal_name.trim() === "";
      return <li className="directory-entry" key={record.employee_id} data-people-record={record.employee_id} data-people-person={record.person_id}>
        <a className="directory-name" href={`${path}/${record.employee_id}`} aria-label={needsIdentity ? `${name} · 목록 기록 ${record.employee_id}` : undefined}>{name}</a>
        <dl><dt>사번</dt><dd>{recordNumber(record.employee_number)}</dd>
          {needsIdentity && <><dt>목록 기록</dt><dd>{record.employee_id}</dd></>}
        </dl><span className="directory-state">목록에 등록됨</span>
      </li>;
    })}</ul>}
    {next_href && <nav className="people-pagination" aria-label="사람 목록 페이지"><a className="policy-button secondary" href={next_href}>다음 사람 보기</a></nav>}
  </section>;
}
function Hidden({name, value}: {name: string; value: string}) {
  return <input type="hidden" name={name} value={value}/>;
}
function Validation({form}: {form: Registration}) {
  if (form.name_error === null && form.number_error === null && form.form_error === null) return null;
  return <section id="people-input-errors" className="people-validation" role="alert" tabIndex={-1}>
    <h3>입력한 내용을 확인해 주세요</h3>
    <p>이번 제출로 새 요청을 접수하지 않았습니다. 작성한 내용은 아래에 남아 있습니다.</p>
    {form.form_error !== null && <p>{form.form_error}</p>}
    <ul>
      {form.name_error !== null && <li><a href="#people-name">이름: {form.name_error}</a></li>}
      {form.number_error !== null && <li><a href="#people-number">사번: {form.number_error}</a></li>}
    </ul>
  </section>;
}
function Register({scope, form}: {scope: Scope; form: Registration}) {
  const path = directory(scope.company), expected = form.expected;
  return <div className="people-task-layout">
    <aside aria-label="등록의 영향"> <Consequences/> </aside>
    <section className="panel people-input-panel"><h2>등록할 사람</h2><div className="policy-panel-body">
      <Validation form={form}/>
      <form method="post" action={`${path}/requests`} data-people-operation="prepare" autoComplete="off">
        <Hidden name="csrf_proof" value={form.proof}/><Hidden name="command_id" value={form.command}/>
        <Hidden name="expected_company_epoch" value={expected.company_epoch}/>
        <Hidden name="object_type_id" value={expected.object_type_id}/><Hidden name="action_type_id" value={expected.action_type_id}/>
        <Hidden name="expected_action_revision" value={expected.action_revision}/><Hidden name="expected_schema_revision" value={expected.schema_revision}/>
        <Hidden name="legal_name_property_id" value={expected.legal_name_property_id}/><Hidden name="employee_number_property_id" value={expected.employee_number_property_id}/>
        <label htmlFor="people-name">이름</label>
        <input id="people-name" name="legal_name" type="text" required defaultValue={form.legal_name}
          aria-invalid={form.name_error !== null ? "true" : undefined}
          aria-describedby={form.name_error !== null ? "people-name-help people-name-error" : "people-name-help"}/>
        <p id="people-name-help" className="supporting">공식 기록에 사용할 이름을 입력하세요. 최대 200자입니다.</p>
        {form.name_error !== null && <p id="people-name-error" className="people-field-error">{form.name_error}</p>}
        <label htmlFor="people-number">사번</label>
        <input id="people-number" name="employee_number" type="text" required defaultValue={form.employee_number}
          aria-invalid={form.number_error !== null ? "true" : undefined}
          aria-describedby={form.number_error !== null ? "people-number-help people-number-error" : "people-number-help"}/>
        <p id="people-number-help" className="supporting">이 회사에서 사람을 구분할 고유한 번호입니다. 최대 64자이며, 등록된 번호와 중복될 수 없습니다.</p>
        {form.number_error !== null && <p id="people-number-error" className="people-field-error">{form.number_error}</p>}
        <div className="policy-actions"><button className="policy-button" type="submit">등록 내용 확인</button>
          {scope.directory_link && <a href={path}>목록으로 돌아가기</a>}
        </div>
      </form>
    </div></section>
  </div>;
}
function SubmittedValues({name, number, label}: {name: string; number: string; label?: string}) {
  return <dl className="people-accepted" aria-label={label}><dt>이름</dt><dd>{name}</dd><dt>사번</dt><dd>{number}</dd></dl>;
}
function RequestView({scope, request}: {scope: Scope; request: PeopleRequest}) {
  const root = directory(scope.company), path = `${root}/requests/${request.command}`, outcome = request.outcome;
  let title: string, description: string;
  switch (outcome.kind) {
    case "pending": title = "등록 내용을 확인하세요"; description = "요청을 접수했습니다. 등록 확정을 선택하면 사람 목록에 반영됩니다."; break;
    case "committed": title = "사람 등록을 완료했습니다"; description = "사람 목록에 등록했습니다. 이 등록으로 고용이나 발령이 생성되지는 않았습니다."; break;
    case "rejected": title = "등록하지 못했습니다"; description = outcome.reason; break;
    case "conflicting": title = "변경된 내용을 확인해 주세요"; description = outcome.reason; break;
    case "cancelled": title = "등록 요청을 취소했습니다"; description = "이 요청으로 사람을 등록하지 않았습니다."; break;
    case "expired": title = "등록 요청의 처리 기한이 지났습니다"; description = "이 요청으로 사람을 등록하지 않았습니다. 현재 내용을 확인하고 새 등록 요청을 작성하세요."; break;
  }
  return <section className="panel people-request" data-people-outcome={outcome.kind} data-people-command={request.command}
    data-people-employee={outcome.kind === "committed" ? outcome.employee_id : undefined}
    data-people-person={outcome.kind === "committed" ? outcome.person_id : undefined}>
    <h2>{title}</h2><div className="policy-panel-body">
      <p className="people-outcome-description">{description}</p>
      <SubmittedValues name={request.legal_name} number={request.employee_number}/>
      <p className="supporting">내가 제출한 등록 요청의 내용입니다.</p>
      {outcome.kind === "pending" && <>
        <p className="supporting">처리 기한: {request.deadline}</p>
        <div className="policy-actions">
          <form method="post" action={`${path}/execute`} data-people-operation="execute">
            <Hidden name="command_id" value={request.command}/><Hidden name="csrf_proof" value={outcome.proof}/>
            <button className="policy-button" type="submit">등록 확정</button>
          </form>
          <form method="post" action={`${path}/cancel`} data-people-operation="cancel">
            <Hidden name="command_id" value={request.command}/><Hidden name="csrf_proof" value={outcome.proof}/>
            <button className="policy-button secondary" type="submit">등록 요청 취소</button>
          </form>
        </div>
      </>}
      {outcome.kind === "committed" && scope.directory_link && <a className="policy-button" href={`${root}/${outcome.employee_id}`}>등록한 사람 보기</a>}
      {outcome.kind !== "pending" && outcome.kind !== "committed" && scope.can_create && <div className="policy-actions"><a className="policy-button" href={`${root}/new`}>새 등록 요청 작성</a></div>}
      <details className="policy-provenance"><summary>이 요청의 처리 기록</summary>
        <dl className="record-meta"><dt>요청</dt><dd>{request.command}</dd>
          <dt>접수 시각</dt><dd>{request.accepted_at}</dd><dt>처리 기한</dt><dd>{request.deadline}</dd>
          <dt>접수 기록</dt><dd>{request.intake_receipt}</dd><dt>요청 당시 회사 버전</dt><dd>{request.expected_company_epoch}</dd>
          {outcome.kind === "committed" && <><dt>완료 기록</dt><dd>{outcome.receipt}</dd><dt>등록 시각</dt><dd>{outcome.registered_at}</dd></>}
        </dl><a href={path}>이 요청 다시 열기</a>
      </details>
    </div>
  </section>;
}
function Detail({record}: {record: PersonRecord}) {
  return <section className="panel people-detail-card" data-people-record={record.employee_id} data-people-person={record.person_id}>
    <h2>사람 목록 등록 정보</h2><div className="policy-panel-body">
      <span className="directory-state">목록에 등록됨</span>
      <dl className="people-accepted"><dt>사번</dt><dd>{recordNumber(record.employee_number)}</dd>
        <dt>등록 시각</dt><dd>{record.registered_at}</dd><dt>사람 기록 버전</dt><dd>{record.person_version}</dd>
      </dl><p className="supporting">사람 목록의 정보입니다. 이 화면의 등록 상태는 고용이나 발령 상태를 나타내지 않습니다.</p>
      <details className="policy-provenance"><summary>기록 식별 정보</summary>
        <dl className="record-meta"><dt>목록 기록</dt><dd>{record.employee_id}</dd><dt>사람 기록</dt><dd>{record.person_id}</dd></dl>
      </details>
    </div>
  </section>;
}
function Content({page}: {page: Page}) {
  switch (page.kind) {
    case "directory": return <Directory page={page}/>;
    case "registration": return <Register scope={page.scope} form={page.form}/>;
    case "request": return <RequestView scope={page.scope} request={page.request}/>;
    case "detail": return <Detail record={page.record}/>;
    case "registration_conflict": return <section className="panel" data-people-outcome="prepare-conflict" role="alert" tabIndex={-1}>
      <h2>등록 내용을 다시 확인하세요</h2><div className="policy-panel-body">
        <p>회사 설정이나 제출 내용이 현재 기록과 일치하지 않습니다. 요청 상태를 확인한 뒤 새 요청을 작성하세요.</p>
        <SubmittedValues name={page.legal_name} number={page.employee_number} label="제출한 내용"/>
        <div className="policy-actions"><a className="policy-button" href={`${directory(page.scope.company)}/requests/${page.command}`}>요청 상태 확인</a>
          {page.scope.can_create && <a href={`${directory(page.scope.company)}/new`}>새 등록 요청 작성</a>}
        </div>
      </div>
    </section>;
    case "request_not_visible": return <section className="panel" data-people-outcome="not-visible"><h2>요청 상태를 확인할 수 없습니다</h2>
      <div className="policy-panel-body">
        <p>현재 권한으로 이 요청의 처리 결과를 확인할 수 없습니다. 이 화면은 등록이 실패했거나 취소되었다는 뜻이 아닙니다.</p>
        <p>새 요청을 작성하기 전에 기존 요청의 처리 여부를 확인하세요.</p>
        <div className="policy-actions"><a className="policy-button" href={`${directory(page.scope.company)}/requests/${page.command}`}>같은 요청 상태 다시 확인</a>
          {page.scope.can_create && <a href={`${directory(page.scope.company)}/new`}>새 등록 요청 작성</a>}
        </div>
      </div>
    </section>;
    case "uncertain": return <section className="panel" data-people-outcome="uncertain"><h2>처리 결과를 아직 확인하지 못했습니다</h2>
      <div className="policy-panel-body"><p>요청이 처리되었을 수 있습니다. 새로 등록하기 전에 같은 요청의 결과를 확인하세요.</p>
        <a className="policy-button" href={`${directory(page.company)}/requests/${page.command}`}>같은 요청 결과 다시 확인</a>
      </div>
    </section>;
  }
}
function title(page: Page): string {
  switch (page.kind) {
    case "directory": return "사람";
    case "registration": return "사람 등록";
    case "registration_conflict": return "등록 내용을 다시 확인하세요";
    case "request": return "사람 등록 요청";
    case "request_not_visible": return "요청 상태를 확인할 수 없습니다";
    case "detail": return recordName(page.record.legal_name);
    case "uncertain": return "등록 결과 확인";
  }
}
export function PeopleView({page}: {page: Page}) {
  const scope = "scope" in page ? page.scope : null;
  return <div data-console-react-people="mounted" className="react-people-workspace">
    <a className="skip-link" href="#main-content">본문 바로가기</a>
    <Header page={page}/>
    <main id="main-content" tabIndex={0}>
      <div className="page-heading">
        {scope && <p className="page-eyebrow">{scope.company_name ?? `회사 ID ${scope.company}`}</p>}
        <h1>{title(page)}</h1>
      </div><Content page={page}/>
    </main>
  </div>;
}
