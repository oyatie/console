import type { components, operations } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

export type WorkOrderStatus = components["schemas"]["WorkOrderStatus"];
export type PriorityLevel = components["schemas"]["PriorityLevel"];
export type WorkResultType = components["schemas"]["WorkResultType"];
export type AttachmentStage = components["schemas"]["AttachmentStage"];
export type WorkOrderSummary = components["schemas"]["WorkOrderSummary"];
export type CreateWorkOrder = components["schemas"]["CreateWorkOrderRequest"];
export type AssignWorkOrder = components["schemas"]["AssignWorkOrderRequest"];
export type SubmitReport = components["schemas"]["SubmitReportRequest"];
export type EvidencePresign = components["schemas"]["EvidencePresignRequest"];

export type MaintenanceType = components["schemas"]["MaintenanceType"];
export type MaintenanceCause = components["schemas"]["MaintenanceCause"];

export type WorkOrderRow = components["schemas"]["WorkOrderListItem"];
export type WorkOrderDetail = components["schemas"]["WorkOrderDetail"];
export type WorkOrderLens = components["schemas"]["WorkOrderObjectSetLens"];

export interface WorkOrderListPage {
  items: WorkOrderRow[];
  limit: number;
  offset: number;
  total: number;
  lens?: WorkOrderLens;
}

export type WorkOrderListQuery = NonNullable<operations["listWorkOrders"]["parameters"]["query"]>;

export type SettlementStatus = components["schemas"]["SettlementStatus"];
export type SettlementLineKind = components["schemas"]["SettlementLineKind"];
export type SettlementLine = components["schemas"]["SettlementLineRequest"];
export type SettlementLineView = components["schemas"]["SettlementLineSummary"];
export type WorkOrderSettlement = components["schemas"]["SettlementSummary"];
export type ReviewSettlementInput = components["schemas"]["ReviewSettlementRequest"];

export class MaintenanceApiError extends Error {
  constructor(message: string, readonly status: number) {
    super(message);
    this.name = "MaintenanceApiError";
  }
}

function message(error: unknown, status: number): string {
  if (error && typeof error === "object" && "error" in error) {
    const body = error as { error?: { message?: unknown } };
    if (typeof body.error?.message === "string") return body.error.message;
  }
  return `Maintenance request failed (${String(status)})`;
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  throw new MaintenanceApiError(message(response.error, response.response.status), response.response.status);
}

/** Work-order transport bound to the authenticated ConsoleApiClient. */
export function createMaintenanceApi(api: ConsoleApiClient) {
  return {
    list: async (query?: WorkOrderListQuery, signal?: AbortSignal): Promise<WorkOrderListPage> => {
      const response = await api.GET("/api/v1/work-orders", { params: { query: query ?? {} }, signal });
      return requireData(response);
    },
    detail: async (id: string, signal?: AbortSignal): Promise<WorkOrderDetail> => {
      const response = await api.GET("/api/v1/work-orders/{workOrderId}", {
        params: { path: { workOrderId: id } },
        signal,
      });
      return requireData(response);
    },
    create: async (input: CreateWorkOrder, signal?: AbortSignal) => {
      const response = await api.POST("/api/work-orders", { body: input, signal });
      return requireData(response);
    },
    assign: async (id: string, input: AssignWorkOrder, signal?: AbortSignal) => {
      const response = await api.PUT("/api/work-orders/{workOrderId}/assignments", {
        params: { path: { workOrderId: id } },
        body: input,
        signal,
      });
      return requireData(response);
    },
    start: async (id: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/work-orders/{workOrderId}/start", {
        params: { path: { workOrderId: id } },
        signal,
      });
      return requireData(response);
    },
    report: async (id: string, input: SubmitReport, signal?: AbortSignal) => {
      const response = await api.POST("/api/work-orders/{workOrderId}/report", {
        params: { path: { workOrderId: id } },
        body: input,
        signal,
      });
      return requireData(response);
    },
    approve: async (id: string, comment: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/work-orders/{workOrderId}/approve", {
        params: { path: { workOrderId: id } },
        body: { comment },
        signal,
      });
      return requireData(response);
    },
    reject: async (id: string, memo: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/v1/work-orders/{workOrderId}/reject", {
        params: { path: { workOrderId: id } },
        body: { memo },
        signal,
      });
      return requireData(response);
    },
    setPriority: async (id: string, priority: PriorityLevel, signal?: AbortSignal) => {
      const response = await api.PATCH("/api/work-orders/{workOrderId}/priority", {
        params: { path: { workOrderId: id } },
        body: { priority },
        signal,
      });
      return requireData(response);
    },
    presignEvidence: async (input: EvidencePresign, signal?: AbortSignal) => {
      const response = await api.POST("/api/v1/evidence/presign", { body: input, signal });
      return requireData(response);
    },
    confirmEvidence: async (evidenceId: string, signal?: AbortSignal) => {
      const response = await api.POST("/api/v1/evidence/{evidenceId}/confirm", {
        params: { path: { evidenceId } },
        signal,
      });
      return requireData(response);
    },
    settlement: async (workOrderId: string, signal?: AbortSignal): Promise<WorkOrderSettlement | undefined> => {
      const result = await api.GET("/api/v1/work-orders/{workOrderId}/settlement", {
        params: { path: { workOrderId } },
        signal,
      });
      // A work order with no live settlement is a 404, not an error state.
      if (result.data === undefined && result.response.status === 404) return undefined;
      return requireData(result);
    },
    createSettlement: async (workOrderId: string, lines: SettlementLine[], signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/work-orders/{workOrderId}/settlement", {
        // The server dedupes on this key: a replay with an identical body
        // returns the existing settlement instead of opening a second one.
        params: { path: { workOrderId }, header: { "Idempotency-Key": crypto.randomUUID() } },
        body: { lines },
        signal,
      });
      return requireData(result);
    },
    submitSettlement: async (settlementId: string, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/settlements/{settlementId}/submit", {
        params: { path: { settlementId } },
        signal,
      });
      return requireData(result);
    },
    reviewSettlement: async (settlementId: string, input: ReviewSettlementInput, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/settlements/{settlementId}/review", {
        params: { path: { settlementId } },
        body: input,
        signal,
      });
      return requireData(result);
    },
    voidSettlement: async (settlementId: string, reason: string, signal?: AbortSignal) => {
      const result = await api.POST("/api/v1/settlements/{settlementId}/void", {
        params: { path: { settlementId } },
        body: { reason },
        signal,
      });
      return requireData(result);
    },
  };
}

export type MaintenanceApi = ReturnType<typeof createMaintenanceApi>;
