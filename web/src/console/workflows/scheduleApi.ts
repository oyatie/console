/**
 * Typed Workflow Studio schedule transport.
 *
 * These endpoints are intentionally separate from the legacy definition JSON
 * envelope: schedule lifecycle, chronology, and tenant authorization belong to
 * the durable `workflow_schedules` resource the backend owns.
 */
import type { components } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";
import { ApiCallError } from "../../api/ontologyActions";

export type WorkflowSchedule =
  components["schemas"]["WorkflowScheduleResponse"];
export type WorkflowScheduleUpdate =
  components["schemas"]["UpdateWorkflowScheduleRequest"];
export type WorkflowScheduleCreate =
  components["schemas"]["CreateWorkflowScheduleRequest"];
export type WorkflowScheduleRun = components["schemas"]["ScheduleRunItem"];
export type WorkflowSchedulePreview =
  components["schemas"]["PreviewScheduleResponse"];

export async function listWorkflowSchedules(
  api: ConsoleApiClient,
  signal?: AbortSignal,
): Promise<WorkflowSchedule[]> {
  const { data, error, response } = await api.GET(
    "/api/v1/workflow-studio/schedules",
    { signal },
  );
  if (!data) throw new ApiCallError(response.status, error);
  return data.items;
}

export async function createWorkflowSchedule(
  api: ConsoleApiClient,
  input: WorkflowScheduleCreate,
): Promise<WorkflowSchedule> {
  const { data, error, response } = await api.POST(
    "/api/v1/workflow-studio/schedules",
    { body: input },
  );
  if (!data) throw new ApiCallError(response.status, error);
  return data;
}

export async function previewWorkflowSchedule(
  api: ConsoleApiClient,
  cronExpr: string,
  timezone: string,
  signal?: AbortSignal,
): Promise<WorkflowSchedulePreview> {
  const { data, error, response } = await api.POST(
    "/api/v1/workflow-studio/schedules/preview-next-runs",
    { body: { cron_expr: cronExpr, timezone }, signal },
  );
  if (!data) throw new ApiCallError(response.status, error);
  return data;
}

export async function listScheduleRuns(
  api: ConsoleApiClient,
  scheduleId: string,
  signal?: AbortSignal,
): Promise<WorkflowScheduleRun[]> {
  const { data, error, response } = await api.GET(
    "/api/v1/workflow-studio/schedules/{id}/runs",
    { params: { path: { id: scheduleId } }, signal },
  );
  if (!data) throw new ApiCallError(response.status, error);
  return data.items;
}

export async function updateWorkflowSchedule(
  api: ConsoleApiClient,
  scheduleId: string,
  update: WorkflowScheduleUpdate,
): Promise<WorkflowSchedule> {
  const { data, error, response } = await api.PATCH(
    "/api/v1/workflow-studio/schedules/{id}",
    {
      params: { path: { id: scheduleId } },
      body: update,
    },
  );
  if (!data) throw new ApiCallError(response.status, error);
  return data;
}
