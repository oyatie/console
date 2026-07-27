import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { ko } from "../../i18n/ko";
import { PolicyGateProvider, type PolicyGate } from "../policy";
import { WindowManagerProvider } from "../window";
import { EvidenceRecords } from "./EvidenceRecords";
import { evidenceFixtures } from "./evidenceFixtures";

const T = ko.console.evidence;
const [heldWire, plainWire] = evidenceFixtures();
const allowGate: PolicyGate = { can: () => true };

interface ApiResult {
  data?: unknown;
  error?: unknown;
  response: { ok: boolean; status: number };
}

/** Minimal EvidenceObjectView-shaped row for the list endpoint. */
function listRow(detail: typeof heldWire) {
  return {
    id: detail.id,
    code: detail.code,
    title: detail.title,
    classification: detail.classification,
    source: detail.source ? { source_type: "work_order_evidence_media", source_id: detail.source.code, source_code: detail.source.code } : undefined,
    current_custody_stage: detail.custodyStage,
    legal_hold_state: detail.holds.some((h) => h.status === "ACTIVE") ? "ACTIVE" : "CLEAR",
    admissibility_status: detail.admissibility,
    admissibility_reasons: [],
    admissibility_inputs: {},
    created_by: "creator-1",
    updated_by: "creator-1",
    created_at: detail.registeredAt,
    updated_at: detail.registeredAt,
    disposed_at: detail.disposed ? detail.registeredAt : null,
  };
}

/** Full EvidenceObjectDetail wire shape for GET .../{id}. */
function detailWire(detail: typeof heldWire) {
  return {
    object: listRow(detail),
    copies: detail.copies.map((c) => ({
      id: c.id,
      evidence_object_id: detail.id,
      copy_kind: c.kind,
      derivative_kind: c.derivativeKind ?? null,
      parent_copy_id: c.parentCopyId ?? null,
      storage: { provider: "s3", object_id: c.id },
      source_evidence_media_id: c.sourceEvidenceMediaId ?? null,
      digest_sha256: c.digestSha256,
      content_type: c.contentType,
      size_bytes: c.sizeBytes,
      worm_status: c.wormStatus,
      verified_at: null,
      created_by: "creator-1",
      created_at: detail.registeredAt,
    })),
    tsa_proofs: [],
    custody_history: detail.custody.map((event) => ({
      id: event.id,
      evidence_object_id: detail.id,
      stage: event.action,
      actor_user_id: event.actor ?? "system",
      from_custodian: null,
      to_custodian: null,
      location_label: null,
      reason: "",
      source_ref: null,
      audit_event_id: null,
      previous_event_id: null,
      event_digest_sha256: "digest",
      occurred_at: event.occurred_at,
      created_at: event.occurred_at,
    })),
    legal_holds: detail.holds.map((h) => ({
      id: h.id,
      evidence_object_id: detail.id,
      status: h.status,
      case_ref: h.caseRef,
      basis: "basis",
      reason: "reason",
      applied_by: "creator-1",
      applied_at: h.appliedAt,
      released_by: null,
      released_at: h.releasedAt ?? null,
      release_reason: null,
      audit_event_id: null,
    })),
    exports: [],
  };
}

function makeApi() {
  const GET = vi.fn((path: string): Promise<ApiResult> => {
    if (path === "/api/v1/evidence/objects") {
      return Promise.resolve({
        data: { items: [listRow(heldWire), listRow(plainWire)], limit: 200, offset: 0, total: 2 },
        response: { ok: true, status: 200 },
      });
    }
    if (path === "/api/v1/evidence/objects/{id}") {
      // The test only opens the held row.
      return Promise.resolve({ data: detailWire(heldWire), response: { ok: true, status: 200 } });
    }
    if (path === "/api/v1/users") {
      return Promise.resolve({ data: { items: [] }, response: { ok: true, status: 200 } });
    }
    return Promise.resolve({ data: undefined, response: { ok: false, status: 404 } });
  });
  const POST = vi.fn(() => Promise.resolve({ data: undefined, response: { ok: false, status: 404 } }));
  const api = { GET, POST } as unknown as ConsoleApiClient;
  return api;
}

describe("EvidenceRecords list (real-wired)", () => {
  it("fetches the real EV- list and renders every row as an objDrag source", async () => {
    const api = makeApi();
    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await waitFor(() => {
      expect(screen.getByText(heldWire.code)).toBeTruthy();
    });
    const code = screen.getByText(heldWire.code);
    expect(code.getAttribute("data-obj-code")).toBe(heldWire.code);
    expect(code.getAttribute("draggable")).toBe("true");
  });

  it("shows the compact stat bar with per-status counts", async () => {
    const api = makeApi();
    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await waitFor(() => {
      expect(screen.getByText(heldWire.code)).toBeTruthy();
    });
    const bar = screen.getByRole("group", { name: T.records.statBar });
    expect(within(bar).getByRole("button", { name: new RegExp(T.records.all) })).toBeTruthy();
  });

  it("owns and aborts the paged register request on unmount", async () => {
    let listSignal: AbortSignal | undefined;
    const GET = vi.fn((path: string, options?: { signal?: AbortSignal }) => {
      if (path === "/api/v1/evidence/objects") {
        listSignal = options?.signal;
        return new Promise(() => undefined);
      }
      return Promise.resolve({ data: { items: [] }, response: { ok: true, status: 200 } });
    });
    const api = { GET, POST: vi.fn() } as unknown as ConsoleApiClient;
    const { unmount } = render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);

    await waitFor(() => {
      expect(listSignal).toBeDefined();
    });
    unmount();
    expect(listSignal?.aborted).toBe(true);
  });

  it("synchronously clears prior-session rows while the replacement session is pending", async () => {
    let resolveB: ((value: ApiResult) => void) | undefined;
    const pendingB = new Promise<ApiResult>((resolve) => {
      resolveB = resolve;
    });
    const apiA = {
      GET: vi.fn((path: string) => {
        if (path === "/api/v1/evidence/objects") {
          return Promise.resolve({
            data: { items: [listRow(heldWire)], limit: 200, offset: 0, total: 1 },
            response: { ok: true, status: 200 },
          });
        }
        return Promise.resolve({ data: { items: [] }, response: { ok: true, status: 200 } });
      }),
      POST: vi.fn(),
    } as unknown as ConsoleApiClient;
    const apiB = {
      GET: vi.fn((path: string) =>
        path === "/api/v1/evidence/objects"
          ? pendingB
          : Promise.resolve({ data: { items: [] }, response: { ok: true, status: 200 } }),
      ),
      POST: vi.fn(),
    } as unknown as ConsoleApiClient;

    const view = render(<EvidenceRecords api={apiA} sessionIncarnation="session-a" />);
    expect(await screen.findByText(heldWire.code)).toBeVisible();

    view.rerender(<EvidenceRecords api={apiB} sessionIncarnation="session-b" />);
    expect(screen.queryByText(heldWire.code)).toBeNull();
    expect(screen.getByText(T.records.loading)).toBeVisible();

    resolveB?.({
      data: { items: [], limit: 200, offset: 0, total: 0 },
      response: { ok: true, status: 200 },
    });
  });

  it("shows a retry affordance when the list fails to load", async () => {
    const GET = vi.fn(() => Promise.resolve({ data: undefined, response: { ok: false, status: 500 } }));
    const api = { GET, POST: vi.fn() } as unknown as ConsoleApiClient;
    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await waitFor(() => {
      expect(screen.getByText(T.records.loadFailed)).toBeTruthy();
    });
    expect(screen.getByRole("button", { name: T.records.retry })).toBeTruthy();
  });
});

describe("EvidenceRecords list truthfulness", () => {
  it("does not synthesize a hold from a list-row flag before detail is fetched", async () => {
    const api = makeApi();
    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await waitFor(() => {
      expect(screen.getByText(heldWire.code)).toBeTruthy();
    });
    expect(screen.queryByText(T.hold.active)).toBeNull();
    expect(screen.queryByRole("button", { name: new RegExp(T.hold.active) })).toBeNull();
  });

  it("keeps an opened object's authoritative detail when the user directory resolves late", async () => {
    let resolveUsers: ((value: ApiResult) => void) | undefined;
    const pendingUsers = new Promise<ApiResult>((resolve) => {
      resolveUsers = resolve;
    });
    const wired = makeApi();
    const wiredGet = wired.GET as unknown as (path: string, options?: unknown) => Promise<ApiResult>;
    const GET = vi.fn((path: string, options?: unknown) =>
      path === "/api/v1/users" ? pendingUsers : wiredGet(path, options),
    );
    const api = { GET, POST: vi.fn() } as unknown as ConsoleApiClient;

    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await screen.findByText(heldWire.code);
    fireEvent.click(
      screen.getAllByRole("button", { name: T.records.open(heldWire.code, heldWire.title) })[0],
    );
    const detail = await screen.findByLabelText(T.detailAria(heldWire.code));
    expect(within(detail).getByText(T.hold.active)).toBeTruthy();

    // The directory lands after the object is already open. It may only rename
    // actors. It must not re-seed the register: the list endpoint carries no
    // holds, custody or copies, so re-seeding redraws a held object as unheld.
    resolveUsers?.({
      data: { items: [{ id: "creator-1", display_name: "김보관" }] },
      response: { ok: true, status: 200 },
    });
    await within(detail).findByText("김보관");

    expect(GET.mock.calls.filter(([path]) => path === "/api/v1/evidence/objects")).toHaveLength(1);
    expect(within(detail).getByText(T.hold.active)).toBeTruthy();
  });
});

describe("EvidenceRecords detail opening", () => {
  it("fetches the full detail and opens the EvidenceCard as the right pin (§4.7-3)", async () => {
    const api = makeApi();
    render(
      <WindowManagerProvider>
        <EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />
      </WindowManagerProvider>,
    );
    await waitFor(() => {
      expect(screen.getByText(heldWire.code)).toBeTruthy();
    });
    fireEvent.click(
      screen.getAllByRole("button", { name: T.records.open(heldWire.code, heldWire.title) })[0],
    );
    await waitFor(() => {
      expect(screen.getByLabelText(T.detailAria(heldWire.code))).toBeTruthy();
    });
    const detail = screen.getByLabelText(T.detailAria(heldWire.code));
    expect(within(detail).getByText(T.worm.sealed)).toBeTruthy();
  });

  it("loads authoritative custody and hold detail, then invokes the real verify endpoint", async () => {
    const api = makeApi();
    const POST = api.POST as unknown as ReturnType<typeof vi.fn>;
    POST.mockResolvedValue({
      data: {
        evidence_object_id: heldWire.id,
        verified_at: "2026-07-24T00:00:00Z",
        outcome: "VERIFIED",
        copies: heldWire.copies.map((copy) => ({
          copy_id: copy.id,
          copy_kind: copy.kind,
          status: "MATCH",
        })),
      },
      response: { ok: true, status: 200 },
    });

    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await screen.findByText(heldWire.code);
    expect(screen.queryByText(T.hold.active)).toBeNull();

    fireEvent.click(
      screen.getAllByRole("button", { name: T.records.open(heldWire.code, heldWire.title) })[0],
    );
    const detail = await screen.findByLabelText(T.detailAria(heldWire.code));
    expect(within(detail).getByText(T.hold.active)).toBeTruthy();
    expect(within(detail).getByText(T.custody.stages.LEGAL_HOLD_APPLIED)).toBeTruthy();

    fireEvent.click(within(detail).getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(POST).toHaveBeenCalledWith("/api/v1/evidence/objects/{id}/verify", {
        params: { path: { id: heldWire.id } },
      });
    });
    expect(await within(detail).findByText(T.actions.verifyOk)).toBeTruthy();
  });

  it("opens the EvidenceCard inline when no window shell is mounted", async () => {
    const api = makeApi();
    render(<EvidenceRecords api={api} sessionIncarnation="evidence-records-test" />);
    await waitFor(() => {
      expect(screen.getByText(heldWire.code)).toBeTruthy();
    });
    fireEvent.click(
      screen.getAllByRole("button", { name: T.records.open(heldWire.code, heldWire.title) })[1],
    );
    await waitFor(() => {
      expect(screen.getByLabelText(T.detailAria(heldWire.code))).toBeTruthy();
    });

    fireEvent.click(screen.getByRole("button", { name: T.records.close }));
    expect(screen.queryByLabelText(T.detailAria(heldWire.code))).toBeNull();
  });

  it("reports a successful release with a failed authoritative reread and retries that reread", async () => {
    let detailReads = 0;
    const releasedWire = {
      ...heldWire,
      holds: heldWire.holds.map((hold) => ({
        ...hold,
        status: "RELEASED" as const,
        releasedAt: "2026-07-24T01:00:00Z",
      })),
    };
    const GET = vi.fn((path: string): Promise<ApiResult> => {
      if (path === "/api/v1/evidence/objects") {
        return Promise.resolve({
          data: { items: [listRow(heldWire)], limit: 200, offset: 0, total: 1 },
          response: { ok: true, status: 200 },
        });
      }
      if (path === "/api/v1/evidence/objects/{id}") {
        detailReads += 1;
        if (detailReads === 2) {
          return Promise.resolve({ data: undefined, response: { ok: false, status: 503 } });
        }
        return Promise.resolve({
          data: detailWire(detailReads === 1 ? heldWire : releasedWire),
          response: { ok: true, status: 200 },
        });
      }
      if (path === "/api/v1/users") {
        return Promise.resolve({ data: { items: [] }, response: { ok: true, status: 200 } });
      }
      return Promise.resolve({ data: undefined, response: { ok: false, status: 404 } });
    });
    const POST = vi.fn((path: string): Promise<ApiResult> => {
      if (path === "/api/v1/governance/approvals") {
        return Promise.resolve({ data: { request_ref: "req-1", requested_by: "user-a" }, response: { ok: true, status: 200 } });
      }
      if (path === "/api/v1/governance/approvals/decide") {
        return Promise.resolve({ data: {}, response: { ok: true, status: 200 } });
      }
      if (path === "/api/v1/evidence/objects/{id}/hold") {
        return Promise.resolve({
          data: { id: heldWire.holds[0]?.id, case_ref: heldWire.holds[0]?.caseRef, status: "RELEASED", applied_at: heldWire.holds[0]?.appliedAt },
          response: { ok: true, status: 200 },
        });
      }
      return Promise.resolve({ data: undefined, response: { ok: false, status: 404 } });
    });
    const api = { GET, POST } as unknown as ConsoleApiClient;

    render(
      <PolicyGateProvider gate={allowGate}>
        <EvidenceRecords api={api} currentUserId="user-b" sessionIncarnation="release-refresh-failure" />
      </PolicyGateProvider>,
    );
    await screen.findByText(heldWire.code);
    fireEvent.click(screen.getAllByRole("button", { name: T.records.open(heldWire.code, heldWire.title) })[0]);
    const detail = await screen.findByLabelText(T.detailAria(heldWire.code));
    fireEvent.click(within(detail).getByRole("button", { name: T.hold.requestRelease }));
    await waitFor(() => {
      expect(within(detail).getByRole("button", { name: T.hold.decideApprove })).toBeTruthy();
    });
    fireEvent.click(within(detail).getByRole("button", { name: T.hold.decideApprove }));
    await waitFor(() => {
      expect(within(detail).getByRole("button", { name: T.hold.release })).toBeTruthy();
    });
    fireEvent.click(within(detail).getByRole("button", { name: T.hold.release }));

    await waitFor(() => {
      expect(within(detail).getByText(T.hold.releaseRefreshFailed)).toBeTruthy();
    });
    expect(within(detail).getByText(T.hold.active)).toBeTruthy();
    fireEvent.click(within(detail).getByRole("button", { name: T.hold.refreshRetry }));
    await waitFor(() => {
      expect(detailReads).toBe(3);
    });
    await waitFor(() => {
      expect(screen.queryByText(T.hold.active)).toBeNull();
    });
  });
});
