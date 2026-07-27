import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import {
  createWorkflowSchedule,
  listScheduleRuns,
  listWorkflowSchedules,
  previewWorkflowSchedule,
  updateWorkflowSchedule,
} from "./scheduleApi";

const SCHEDULE_ID = "11111111-1111-4111-8111-111111111111";

function apiClient(overrides: Partial<ConsoleApiClient>): ConsoleApiClient {
  return overrides as ConsoleApiClient;
}

describe("workflow schedule API", () => {
  it("uses the generated list, preview, history, and update operations", async () => {
    const GET = vi
      .fn()
      .mockResolvedValueOnce({
        data: { items: [] },
        error: undefined,
        response: new Response(),
      })
      .mockResolvedValueOnce({
        data: { items: [] },
        error: undefined,
        response: new Response(),
      });
    const POST = vi.fn().mockResolvedValue({
      data: {
        cron_expr: "0 9 * * 1-5",
        timezone: "Asia/Seoul",
        fire_times: [],
      },
      error: undefined,
      response: new Response(),
    });
    const PATCH = vi.fn().mockResolvedValue({
      data: {
        id: SCHEDULE_ID,
        label: "평일 점검",
        cron_expr: "0 9 * * 1-5",
        timezone: "Asia/Seoul",
        definition_id: "22222222-2222-4222-8222-222222222222",
        enabled: true,
        created_at: "2026-07-24T00:00:00Z",
        updated_at: "2026-07-24T00:00:00Z",
      },
      error: undefined,
      response: new Response(),
    });
    const api = apiClient({ GET, POST, PATCH });

    await expect(listWorkflowSchedules(api)).resolves.toEqual([]);
    await expect(
      previewWorkflowSchedule(api, "0 9 * * 1-5", "Asia/Seoul"),
    ).resolves.toEqual({
      cron_expr: "0 9 * * 1-5",
      timezone: "Asia/Seoul",
      fire_times: [],
    });
    await expect(listScheduleRuns(api, SCHEDULE_ID)).resolves.toEqual([]);
    await expect(
      updateWorkflowSchedule(api, SCHEDULE_ID, { enabled: true }),
    ).resolves.toMatchObject({
      id: SCHEDULE_ID,
      enabled: true,
    });

    expect(GET).toHaveBeenNthCalledWith(
      1,
      "/api/v1/workflow-studio/schedules",
      { signal: undefined },
    );
    expect(POST).toHaveBeenCalledWith(
      "/api/v1/workflow-studio/schedules/preview-next-runs",
      {
        body: { cron_expr: "0 9 * * 1-5", timezone: "Asia/Seoul" },
        signal: undefined,
      },
    );
    expect(GET).toHaveBeenNthCalledWith(
      2,
      "/api/v1/workflow-studio/schedules/{id}/runs",
      {
        params: { path: { id: SCHEDULE_ID } },
        signal: undefined,
      },
    );
    expect(PATCH).toHaveBeenCalledWith(
      "/api/v1/workflow-studio/schedules/{id}",
      {
        params: { path: { id: SCHEDULE_ID } },
        body: { enabled: true },
      },
    );
  });

  it("preserves the backend status for denied or conflicted writes", async () => {
    const PATCH = vi.fn().mockResolvedValue({
      data: undefined,
      error: { error: { message: "forbidden", code: "forbidden" } },
      response: new Response(undefined, { status: 403 }),
    });

    await expect(
      updateWorkflowSchedule(apiClient({ PATCH }), SCHEDULE_ID, {
        enabled: false,
      }),
    ).rejects.toMatchObject({ status: 403 });
  });

  it("creates a durable schedule through the generated schedule resource", async () => {
    const POST = vi.fn().mockResolvedValue({
      data: {
        id: SCHEDULE_ID,
        label: "평일 점검",
        cron_expr: "0 9 * * 1-5",
        timezone: "Asia/Seoul",
        definition_id: "22222222-2222-4222-8222-222222222222",
        enabled: true,
        created_at: "2026-07-24T00:00:00Z",
        updated_at: "2026-07-24T00:00:00Z",
      },
      response: new Response(),
    });

    await expect(
      createWorkflowSchedule(apiClient({ POST }), {
        label: "평일 점검",
        cron_expr: "0 9 * * 1-5",
        timezone: "Asia/Seoul",
        definition_id: "22222222-2222-4222-8222-222222222222",
        enabled: true,
      }),
    ).resolves.toMatchObject({ id: SCHEDULE_ID });
    expect(POST).toHaveBeenCalledWith("/api/v1/workflow-studio/schedules", {
      body: {
        label: "평일 점검",
        cron_expr: "0 9 * * 1-5",
        timezone: "Asia/Seoul",
        definition_id: "22222222-2222-4222-8222-222222222222",
        enabled: true,
      },
    });
  });
});
