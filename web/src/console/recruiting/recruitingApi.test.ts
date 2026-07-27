import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { createRecruitingApi, isConflict, isDenied, RecruitingApiError } from "./recruitingApi";

function client() {
  return { GET: vi.fn(), POST: vi.fn(), PUT: vi.fn() } as unknown as ConsoleApiClient;
}

function ok<T>(data: T, status = 200) {
  return { data, response: new Response(null, { status }) };
}

function err(status: number, body?: unknown) {
  return { error: body, response: new Response(null, { status }) };
}

describe("createRecruitingApi", () => {
  it("binds the recruiting contract paths and bodies", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(ok({ items: [] }));
    vi.mocked(api.POST).mockResolvedValue(ok(undefined, 200));
    const recruiting = createRecruitingApi(api);

    await recruiting.listPostings({ status: "PUBLISHED" });
    expect(api.GET).toHaveBeenCalledWith("/api/v1/recruiting/postings", expect.objectContaining({
      params: { query: { status: "PUBLISHED", scope: undefined } },
    }));

    await recruiting.listTalentPool();
    expect(api.GET).toHaveBeenCalledWith("/api/v1/recruiting/talent-pool", expect.anything());

    await recruiting.advanceApplicant("apl-1", { expected_updated_at: "t1" });
    expect(api.POST).toHaveBeenCalledWith("/api/v1/recruiting/applicants/{applicantId}/advance", expect.objectContaining({
      params: { path: { applicantId: "apl-1" } },
      body: { expected_updated_at: "t1" },
    }));

    await recruiting.publishPosting("post-1", { attest_exposure_scope: true, expected_updated_at: "t2" });
    expect(api.POST).toHaveBeenCalledWith("/api/v1/recruiting/postings/{postingId}/publish", expect.objectContaining({
      body: { attest_exposure_scope: true, expected_updated_at: "t2" },
    }));
  });

  it("surfaces the canonical error envelope message with status", async () => {
    const api = client();
    vi.mocked(api.GET).mockResolvedValue(err(422, { error: { message: "면접 평가 기록 후 오퍼 제안 가능" } }));
    const recruiting = createRecruitingApi(api);
    const failure = await recruiting.listPostings().catch((cause: unknown) => cause);
    expect(failure).toBeInstanceOf(RecruitingApiError);
    expect((failure as RecruitingApiError).message).toBe("면접 평가 기록 후 오퍼 제안 가능");
    expect((failure as RecruitingApiError).status).toBe(422);
  });

  it("carries the 422 publish check vector for the fail-closed preflight", async () => {
    const api = client();
    vi.mocked(api.POST).mockResolvedValue(err(422, {
      error: { message: "게시할 수 없습니다" },
      checks: [{ key: "no_duplicate_open", ok: false, note: "중복 존재" }],
    }));
    const recruiting = createRecruitingApi(api);
    const failure = await recruiting
      .publishPosting("post-1", { attest_exposure_scope: true, expected_updated_at: "t" })
      .catch((cause: unknown) => cause);
    expect((failure as RecruitingApiError).checks).toEqual([
      { key: "no_duplicate_open", ok: false, note: "중복 존재" },
    ]);
  });

  it("classifies denial and conflict without leaking them into generic errors", () => {
    expect(isDenied(new RecruitingApiError("x", 403))).toBe(true);
    expect(isDenied(new RecruitingApiError("x", 401))).toBe(true);
    expect(isDenied(new RecruitingApiError("x", 404))).toBe(false);
    expect(isConflict(new RecruitingApiError("x", 409))).toBe(true);
    expect(isConflict(new Error("x"))).toBe(false);
  });
});
