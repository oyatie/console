import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { recruitingStrings as text } from "../../i18n/recruiting";
import type { RecruitApplicantView, RecruitPostingRow } from "./recruitingApi";
import type { RecruitingCapabilities } from "./recruitingCapabilities";
import { RecruitingScreen } from "./RecruitingScreen";

const manager: RecruitingCapabilities = { canRead: true, canManage: true, canHire: true };
const reader: RecruitingCapabilities = { canRead: true, canManage: false, canHire: false };
const denied: RecruitingCapabilities = { canRead: false, canManage: false, canHire: false };

function posting(over: Partial<RecruitPostingRow> = {}): RecruitPostingRow {
  return {
    id: "post-1", posting_no: "JP-01", role_title: "품질관리 매니저", company: "BESTEC",
    worksite: "안산공장 품질관리팀", employment_type: "REGULAR", scope: "EXTERNAL",
    headcount: 2, hired_count: 0, deadline: "2026-07-20", requirements: ["ISO 9001"],
    status: "PUBLISHED", position_ref: null, published_at: "2026-07-05T00:00:00Z",
    closed_at: null, created_at: "2026-07-01T00:00:00Z", updated_at: "2026-07-23T00:00:00Z",
    stage_counts: { applied: 1, screening: 0, interview: 0, offer: 0 },
    ...over,
  };
}

function applicant(over: Partial<RecruitApplicantView> = {}): RecruitApplicantView {
  return {
    id: "apl-1", applicant_no: "APL-01", posting_id: "post-1", name: "한지원",
    stage: "APPLIED", hold: false, doc_requested: false, rejected_at: null,
    reject_reason: null, reject_note: null, assessment: null,
    profile_lines: ["경력 6년 — 품질관리"],
    source_document: "이력서_한지원.pdf", hired_employee_id: null,
    created_at: "2026-07-01T00:00:00Z", updated_at: "2026-07-23T00:00:00Z",
    ...over,
  };
}

function ok<T>(data: T, status = 200) {
  return { data, response: new Response(null, { status }) };
}

function err(status: number, body?: unknown) {
  return { error: body, response: new Response(null, { status }) };
}

type Reply = { data?: unknown; error?: unknown; response: Response };

function client(gets: Record<string, Reply[]> = {}, posts: Record<string, Reply[]> = {}) {
  const next = (map: Record<string, Reply[]>, path: string): Promise<Reply> => {
    const queueOf = map[path] as Reply[] | undefined;
    if (!queueOf || queueOf.length === 0) return Promise.resolve(err(500));
    return Promise.resolve(queueOf.length > 1 ? (queueOf.shift() as Reply) : queueOf[0]);
  };
  return {
    GET: vi.fn((path: string) => next(gets, path)),
    POST: vi.fn((path: string) => next(posts, path)),
    PUT: vi.fn((path: string) => next(posts, path)),
  } as unknown as ConsoleApiClient;
}

function renderScreen(api: ConsoleApiClient, capabilities = manager, sessionKey = "session-a") {
  return render(
    <RecruitingScreen api={api} actorId="actor-1" capabilities={capabilities} sessionKey={sessionKey} onNavigate={vi.fn()} />,
  );
}

const LIST = "/api/v1/recruiting/postings";
const POOL = "/api/v1/recruiting/talent-pool";
const POSTING = "/api/v1/recruiting/postings/{postingId}";
const APPLICANT = "/api/v1/recruiting/applicants/{applicantId}";

describe("RecruitingScreen", () => {
  it("denies an unauthorized session before any fetch", () => {
    const api = client();
    renderScreen(api, denied);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(api.GET).not.toHaveBeenCalled();
  });

  it("renders the live stat line and posting grammar from the server list", async () => {
    const api = client({
      [LIST]: [ok({ items: [posting(), posting({ id: "post-2", posting_no: "JP-02", role_title: "경비원 · 야간 상주", scope: "INTERNAL", status: "DRAFT", deadline: null, stage_counts: { applied: 0, screening: 0, interview: 1, offer: 0 } })] })],
      [POOL]: [ok({ items: [] })],
    });
    renderScreen(api);
    expect(await screen.findByText(text.headStat(2, 2, 1))).toBeVisible();
    expect(screen.getByRole("button", { name: "품질관리 매니저" })).toBeVisible();
    expect(screen.getByText(text.internalChip)).toBeVisible();
    expect(screen.getByRole("button", { name: text.draftPublish })).toBeVisible();
    expect(screen.getByText(text.always)).toBeVisible();
    expect(screen.getByText("7/20")).toBeVisible();
  });

  it("shows the truthful empty state", async () => {
    const api = client({ [LIST]: [ok({ items: [] })], [POOL]: [ok({ items: [] })] });
    renderScreen(api);
    expect(await screen.findByText(text.empty)).toBeVisible();
  });

  it("treats a server denial as denied, not as an error", async () => {
    const api = client({ [LIST]: [err(403, { error: { message: "denied" } })], [POOL]: [ok({ items: [] })] });
    renderScreen(api);
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("recovers from a load error through the retry alert", async () => {
    const api = client({
      [LIST]: [err(500), ok({ items: [posting()] })],
      [POOL]: [ok({ items: [] })],
    });
    renderScreen(api);
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(text.loadError);
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await screen.findByRole("button", { name: "품질관리 매니저" })).toBeVisible();
  });

  it("roves rows with J/K and opens the pipeline with Enter", async () => {
    const api = client({
      [LIST]: [ok({ items: [posting(), posting({ id: "post-2", posting_no: "JP-02", role_title: "지게차 정비 기사" })] })],
      [POOL]: [ok({ items: [] })],
      [POSTING]: [ok({ posting: posting({ id: "post-2", posting_no: "JP-02", role_title: "지게차 정비 기사" }), applicants: [applicant({ posting_id: "post-2" })] })],
    });
    renderScreen(api);
    const first = await screen.findByRole("button", { name: "품질관리 매니저" });
    first.focus();
    await userEvent.keyboard("j");
    const second = screen.getByRole("button", { name: "지게차 정비 기사" });
    expect(second).toHaveFocus();
    await userEvent.keyboard("{Enter}");
    expect(await screen.findByRole("button", { name: "한지원" })).toBeVisible();
    expect(second).toHaveAttribute("aria-expanded", "true");
    await userEvent.keyboard("k");
    expect(first).toHaveFocus();
  });

  it("omits every manage affordance for a read-only grant", async () => {
    const api = client({
      [LIST]: [ok({ items: [posting({ status: "DRAFT" })] })],
      [POOL]: [ok({ items: [] })],
      [POSTING]: [ok({ posting: posting({ status: "DRAFT" }), applicants: [applicant()] })],
    });
    renderScreen(api, reader);
    await screen.findByRole("button", { name: "품질관리 매니저" });
    expect(screen.queryByRole("button", { name: text.newPosting })).toBeNull();
    expect(screen.queryByRole("button", { name: text.draftPublish })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: "품질관리 매니저" }));
    expect(await screen.findByRole("button", { name: "한지원" })).toBeVisible();
    expect(screen.queryByRole("button", { name: text.advanceTo(text.stage.SCREENING) })).toBeNull();
    expect(screen.queryByRole("button", { name: text.applicantForm.open })).toBeNull();
  });

  it("reconciles an advance from the server read, not local state", async () => {
    const advanced = applicant({ stage: "SCREENING" });
    const api = client(
      {
        [LIST]: [ok({ items: [posting()] })],
        [POOL]: [ok({ items: [] })],
        [POSTING]: [
          ok({ posting: posting(), applicants: [applicant()] }),
          ok({ posting: posting({ stage_counts: { applied: 0, screening: 1, interview: 0, offer: 0 } }), applicants: [advanced] }),
        ],
      },
      { "/api/v1/recruiting/applicants/{applicantId}/advance": [ok(undefined)] },
    );
    renderScreen(api);
    await userEvent.click(await screen.findByRole("button", { name: "품질관리 매니저" }));
    await userEvent.click(await screen.findByRole("button", { name: text.advanceTo(text.stage.SCREENING) }));
    expect(api.POST).toHaveBeenCalledWith("/api/v1/recruiting/applicants/{applicantId}/advance", expect.objectContaining({
      params: { path: { applicantId: "apl-1" } },
      body: { expected_updated_at: "2026-07-23T00:00:00Z" },
    }));
    expect(await screen.findByRole("button", { name: text.advanceTo(text.stage.INTERVIEW) })).toBeVisible();
    expect(await screen.findByText(text.toast.advanced("한지원", text.stage.SCREENING))).toBeVisible();
  });

  it("surfaces a mutation failure and keeps the server truth", async () => {
    const api = client(
      {
        [LIST]: [ok({ items: [posting()] })],
        [POOL]: [ok({ items: [] })],
        [POSTING]: [ok({ posting: posting(), applicants: [applicant()] })],
      },
      { "/api/v1/recruiting/applicants/{applicantId}/advance": [err(409, { error: { message: "conflict" } })] },
    );
    renderScreen(api);
    await userEvent.click(await screen.findByRole("button", { name: "품질관리 매니저" }));
    await userEvent.click(await screen.findByRole("button", { name: text.advanceTo(text.stage.SCREENING) }));
    expect(await screen.findByText(text.conflict)).toBeVisible();
    expect(screen.getByRole("button", { name: text.advanceTo(text.stage.SCREENING) })).toBeVisible();
  });

  it("fail-closes the offer CTA behind a recorded assessment", async () => {
    const api = client({
      [LIST]: [ok({ items: [posting()] })],
      [POOL]: [ok({ items: [] })],
      [POSTING]: [ok({ posting: posting(), applicants: [applicant({ stage: "INTERVIEW" })] })],
      [APPLICANT]: [ok({ applicant: applicant({ stage: "INTERVIEW" }), offers: [], events: [{ id: "ev-1", action: "ADVANCED", occurred_at: "2026-07-22T09:00:00Z" }] })],
    });
    renderScreen(api);
    await userEvent.click(await screen.findByRole("button", { name: "품질관리 매니저" }));
    await userEvent.click(await screen.findByRole("button", { name: "한지원" }));
    expect(await screen.findByText("경력 6년 — 품질관리")).toBeVisible();
    expect(screen.getByText(text.card.event.ADVANCED)).toBeVisible();
    await userEvent.click(screen.getByRole("button", { name: text.card.ctaOffer }));
    expect(await screen.findByText(text.card.assessmentRequired)).toBeVisible();
    expect(api.POST).not.toHaveBeenCalled();
  });

  it("publishes a draft only through the attested preflight gate", async () => {
    const draft = posting({ status: "DRAFT" });
    const api = client(
      {
        [LIST]: [ok({ items: [draft] }), ok({ items: [posting()] })],
        [POOL]: [ok({ items: [] })],
      },
      {
        "/api/v1/recruiting/postings/{postingId}/preflight": [ok({ checks: [{ key: "position_worksite", ok: true, note: "품질관리 매니저" }], publishable: true })],
        "/api/v1/recruiting/postings/{postingId}/publish": [ok(undefined)],
      },
    );
    renderScreen(api);
    await userEvent.click(await screen.findByRole("button", { name: text.draftPublish }));
    expect(await screen.findByText(text.preflight.checkLabel.position_worksite)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.preflight.publish })).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: text.preflight.attest }));
    await userEvent.click(screen.getByRole("button", { name: text.preflight.publish }));
    expect(api.POST).toHaveBeenCalledWith("/api/v1/recruiting/postings/{postingId}/publish", expect.objectContaining({
      body: { attest_exposure_scope: true, expected_updated_at: draft.updated_at },
    }));
    expect(await screen.findByText(text.toast.published("품질관리 매니저"))).toBeVisible();
    await waitFor(() => { expect(screen.queryByText(text.preflight.title)).toBeNull(); });
  });

  it("fail-closes composer save on missing typed fields", async () => {
    const api = client({ [LIST]: [ok({ items: [] })], [POOL]: [ok({ items: [] })] });
    renderScreen(api);
    await userEvent.click(await screen.findByRole("button", { name: text.newPosting }));
    await userEvent.click(screen.getByRole("button", { name: text.composer.saveDraft }));
    expect(await screen.findByText(text.composer.validationError)).toBeVisible();
    expect(api.POST).not.toHaveBeenCalled();
  });

  it("fences a stale API client's late response out of the replacement view", async () => {
    let resolveOld: ((value: Reply) => void) | undefined;
    const oldPromise = new Promise<Reply>((resolve) => { resolveOld = resolve; });
    const apiA = { GET: vi.fn(() => oldPromise), POST: vi.fn(), PUT: vi.fn() } as unknown as ConsoleApiClient;
    const apiB = client({
      [LIST]: [ok({ items: [posting({ role_title: "지게차 정비 기사" })] })],
      [POOL]: [ok({ items: [] })],
    });
    const view = renderScreen(apiA);
    await waitFor(() => { expect(apiA.GET).toHaveBeenCalled(); });
    view.rerender(
      <RecruitingScreen api={apiB} actorId="actor-1" capabilities={manager} sessionKey="session-b" onNavigate={vi.fn()} />,
    );
    expect(await screen.findByRole("button", { name: "지게차 정비 기사" })).toBeVisible();
    resolveOld?.(ok({ items: [posting({ role_title: "품질관리 매니저" })] }));
    await waitFor(() => { expect(screen.queryByRole("button", { name: "품질관리 매니저" })).toBeNull(); });
  });

  it("moves focus into the candidate card so Escape closes it", async () => {
    const api = client({
      [LIST]: [ok({ items: [posting()] })],
      [POOL]: [ok({ items: [] })],
      [POSTING]: [ok({ posting: posting(), applicants: [applicant()] })],
      [APPLICANT]: [ok({ applicant: applicant(), offers: [], events: [] })],
    });
    renderScreen(api);
    await userEvent.click(await screen.findByRole("button", { name: "품질관리 매니저" }));
    await userEvent.click(await screen.findByRole("button", { name: "한지원" }));
    const dialog = await screen.findByRole("dialog");
    expect(dialog.contains(document.activeElement)).toBe(true);
    await userEvent.keyboard("{Escape}");
    await waitFor(() => { expect(screen.queryByRole("dialog")).toBeNull(); });
  });

  it("hires an accepted-offer applicant through the handshake and links the created employee", async () => {
    const offered = applicant({ stage: "OFFER" });
    const hired = applicant({ stage: "HIRED", hired_employee_id: "emp-7" });
    const acceptedOffer = {
      id: "ofr-1", version: 2, amount: "3400000", amount_period: "MONTHLY" as const,
      reply_deadline: null, status: "ACCEPTED" as const, created_at: "2026-07-22T00:00:00Z",
    };
    const onNavigate = vi.fn();
    const api = client(
      {
        [LIST]: [ok({ items: [posting()] })],
        [POOL]: [ok({ items: [] })],
        [POSTING]: [
          ok({ posting: posting(), applicants: [offered] }),
          ok({ posting: posting({ hired_count: 1 }), applicants: [hired] }),
        ],
        [APPLICANT]: [
          ok({ applicant: offered, offers: [acceptedOffer], events: [] }),
          ok({ applicant: hired, offers: [acceptedOffer], events: [] }),
        ],
        "/api/v1/branches": [ok([{ id: "br-1", name: "안산지점" }])],
      },
      { "/api/v1/recruiting/applicants/{applicantId}/hire": [ok({ employee_id: "emp-7", applicant: hired, posting: posting({ hired_count: 1 }) })] },
    );
    render(
      <RecruitingScreen api={api} actorId="actor-1" capabilities={manager} sessionKey="session-a" onNavigate={onNavigate} />,
    );
    await userEvent.click(await screen.findByRole("button", { name: "품질관리 매니저" }));
    await userEvent.click(await screen.findByRole("button", { name: "한지원" }));
    const dialog = await screen.findByRole("dialog");
    await userEvent.click(within(dialog).getByRole("button", { name: text.card.ctaHire }));
    const form = await screen.findByRole("region", { name: text.hire.title });
    await userEvent.type(within(form).getByLabelText(text.hire.employeeNumber), "24-1187");
    await userEvent.type(within(form).getByLabelText(text.hire.phone), "010-1234-5678");
    await userEvent.type(within(form).getByLabelText(text.hire.orgUnit), "품질관리팀");
    await userEvent.selectOptions(
      within(form).getByLabelText(text.hire.homeBranch),
      await within(form).findByRole("option", { name: "안산지점" }),
    );
    await userEvent.click(within(form).getByRole("button", { name: text.hire.submit }));
    expect(api.POST).toHaveBeenCalledWith("/api/v1/recruiting/applicants/{applicantId}/hire", expect.objectContaining({
      params: { path: { applicantId: "apl-1" } },
      body: {
        employee_number: "24-1187", phone: "010-1234-5678", org_unit: "품질관리팀",
        position: "품질관리 매니저", site: "안산공장 품질관리팀", home_branch_id: "br-1", base_pay: "3400000",
      },
    }));
    expect(await screen.findByText(text.toast.hired("한지원", "1 / 2"))).toBeVisible();
    await userEvent.click(await screen.findByRole("button", { name: text.hire.openEmployee }));
    expect(onNavigate).toHaveBeenCalledWith("/console/people");
  });

  it("lists the talent pool as the rejection downstream", async () => {
    const api = client({
      [LIST]: [ok({ items: [] })],
      [POOL]: [ok({ items: [{ applicant_no: "APL-09", name: "강태오", role_title: "품질관리 매니저", reason: "ROLE_MISMATCH", rejected_at: "2026-07-20T02:00:00Z" }] })],
    });
    renderScreen(api);
    expect(await screen.findByText("강태오")).toBeVisible();
    expect(screen.getByText(`${text.talentPool.reasonPrefix}${text.card.rejectReasons.ROLE_MISMATCH}`)).toBeVisible();
  });
});
