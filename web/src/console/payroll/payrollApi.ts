import type { components } from "@maintenance/api-client-ts";

import type { ConsoleApiClient } from "../../api/client";

// ---------------------------------------------------------------------------
// DTOs. Generated schemas cover the three long-standing read routes; the run
// lifecycle surface is contract-first, recorded in
// docs/evidence/console/CAP-PAYROLL-CONSOLE/frontend/manifests/mount.json and
// derived from the design mirror (docs/design/oyatie-console — pay module +
// HANDOFF) until the integrator lands the openapi paths and regenerates
// clients/ts. The local types below extend the generated ones exactly per that
// contract — any divergence from the regenerated client is a defect to fix
// here.
// ---------------------------------------------------------------------------

type GeneratedRunSummary = components["schemas"]["PayrollRunSummary"];
type GeneratedRunDetail = components["schemas"]["PayrollRunDetail"];
type GeneratedRunPage = components["schemas"]["PayrollRunPage"];

export type PayrollLineSummary = components["schemas"]["PayrollLineSummary"];
export type MyPayrollLinePage = components["schemas"]["MyPayrollLinePage"];

export type PayrollRunStatus =
  | GeneratedRunSummary["status"]
  | "ATTENDANCE_CLOSED"
  | "CALCULATING"
  | "CALCULATED"
  | "SUBMITTED"
  | "REJECTED"
  | "DISBURSEMENT_SCHEDULED"
  | "PAID";

export type PayrollRunSummary = Omit<GeneratedRunSummary, "status"> & {
  status: PayrollRunStatus;
  close_receipt?: unknown;
  submitted_by?: string | null;
  submitted_at?: string | null;
  decided_by?: string | null;
  decided_at?: string | null;
  decision_reason?: string | null;
  approval_ref?: string | null;
};

export type PayrollRunPage = Omit<GeneratedRunPage, "items"> & {
  items: PayrollRunSummary[];
};

export interface PreflightCheck {
  key: string;
  label_ko: string;
  ok: boolean;
  warn: boolean;
  note?: string | null;
  blocking_refs: string[];
}

export interface ClosePreflight {
  checks: PreflightCheck[];
  can_close: boolean;
}

export interface RunCalcSummary {
  version: number;
  calculated_at: string;
  calculated_lines: number;
  blocked_lines: number;
  /** False until the release-gate artifact is registered — never inferred. */
  payable: boolean;
  kernel_rate_table: string;
  /** Null unless every line calculated — never a partial sum shown as a total. */
  total_net_won: number | null;
}

export type PayrollExceptionKind =
  | "OVERTIME_ALLOWANCE"
  | "RETRO_ADJUSTMENT"
  | "ABSENCE_DEDUCTION"
  | "PRORATION"
  | "ACCOUNT_VERIFICATION";

export type PayrollExceptionSeverity = "info" | "warn" | "danger";
export type PayrollExceptionStatus = "OPEN" | "CONFIRMED" | "HELD";

export interface PayrollLinkedRef {
  kind: string;
  code: string;
  id?: string | null;
}

export interface PayrollException {
  id: string;
  run_id: string;
  line_id?: string | null;
  employee_id?: string | null;
  employee_display_name: string;
  kind: PayrollExceptionKind;
  severity: PayrollExceptionSeverity;
  amount_delta_won: number | null;
  summary_ko: string;
  detail?: unknown;
  linked_refs: PayrollLinkedRef[];
  status: PayrollExceptionStatus;
  resolved_by?: string | null;
  resolved_at?: string | null;
  resolved_reason?: string | null;
  carried_from_run_id?: string | null;
}

export interface ExceptionPage {
  items: PayrollException[];
  total: number;
  limit: number;
  offset: number;
}

export type DisbursementStatus = "SCHEDULED" | "SUBMITTED_TO_BANK" | "PAID" | "FAILED";

export interface Disbursement {
  id: string;
  run_id: string;
  scheduled_at: string;
  status: DisbursementStatus;
  attested_by?: string | null;
  attested_at?: string | null;
  reason?: string | null;
}

export interface PayslipDeliveryItem {
  line_id: string;
  employee_id: string;
  inbox_doc_id: string;
  issued_at: string;
  acknowledged_at?: string | null;
}

export interface PayslipDeliverySummary {
  run_id: string;
  issued: number;
  acknowledged: number;
  items: PayslipDeliveryItem[];
  limit: number;
  offset: number;
  total: number;
}

export type PayrollRunDetail = Omit<GeneratedRunDetail, "run"> & {
  run: PayrollRunSummary;
  exceptions_open: number;
  exceptions_total: number;
  calculation: RunCalcSummary | null;
  disbursement: Disbursement | null;
  payslip_delivery: PayslipDeliverySummary | null;
};

export type ResolveExceptionAction = "CONFIRM" | "HOLD";
export type RunDecision = "APPROVE" | "REJECT";

// ---------------------------------------------------------------------------
// Error envelope: `{error: {code, message}}` on every non-2xx.
// ---------------------------------------------------------------------------

export class PayrollApiError extends Error {
  constructor(message: string, readonly status: number, readonly code?: string) {
    super(message);
    this.name = "PayrollApiError";
  }
}

function envelope(error: unknown): { code?: string; message?: string } {
  if (error && typeof error === "object" && "error" in error) {
    const body = (error as { error?: { code?: unknown; message?: unknown } }).error;
    return {
      code: typeof body?.code === "string" ? body.code : undefined,
      message: typeof body?.message === "string" ? body.message : undefined,
    };
  }
  return {};
}

function requireData<T>(response: { data?: T; error?: unknown; response: Response }): T {
  if (response.data !== undefined) return response.data;
  const { code, message } = envelope(response.error);
  throw new PayrollApiError(
    message ?? `Payroll request failed (${String(response.response.status)})`,
    response.response.status,
    code,
  );
}

// The authenticated openapi-fetch client is typed against the generated path
// map, which does not yet carry the lifecycle routes. One structural view of
// the same runtime client keeps bearer/refresh/caching middleware while the
// contract routes remain client-generation pending (see header note).
/**
 * Whole-collection page size. The workspace sorts, flags, and searches the
 * roster and exception list client-side, so a single truncated server page
 * would silently hide employees; collection reads walk pages to the server
 * total instead.
 */
const PAGE_LIMIT = 500;

/** Payroll run-lifecycle transport bound to the authenticated ConsoleApiClient. */
export function createPayrollApi(api: ConsoleApiClient) {
  return {
    listRuns: async (signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/payroll/runs", {
        params: { query: { limit: PAGE_LIMIT, offset: 0 } },
        signal,
      });
      return requireData(response);
    },
    getRun: async (id: string, signal?: AbortSignal) => {
      const first = requireData(await api.GET("/api/v1/payroll/runs/{id}", {
        params: { path: { id }, query: { limit: PAGE_LIMIT, offset: 0 } },
        signal,
      }));
      const lines = [...first.lines];
      while (lines.length < first.lines_total) {
        const next = requireData(await api.GET("/api/v1/payroll/runs/{id}", {
          params: { path: { id }, query: { limit: PAGE_LIMIT, offset: lines.length } },
          signal,
        }));
        if (next.lines.length === 0) break;
        lines.push(...next.lines);
      }
      return { ...first, lines };
    },
    myPayslips: async (signal?: AbortSignal) => {
      const response = await api.GET("/api/v1/payroll/payslips/me", { signal });
      return requireData(response);
    },
    closePreflight: async (id: string, signal?: AbortSignal) =>
      requireData(await api.GET("/api/v1/payroll/runs/{id}/close-preflight", {
        params: { path: { id } },
        signal,
      })) as ClosePreflight,
    closeAttendance: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/close-attendance", {
        params: { path: { id } },
        body: { attest: true },
        signal,
      })) as PayrollRunDetail,
    calculate: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/calculate", {
        params: { path: { id } },
        signal,
      })) as PayrollRunDetail,
    listExceptions: async (id: string, signal?: AbortSignal) => {
      const first = requireData(await api.GET("/api/v1/payroll/runs/{id}/exceptions", {
        params: { path: { id }, query: { limit: PAGE_LIMIT, offset: 0 } },
        signal,
      })) as ExceptionPage;
      const items = [...first.items];
      while (items.length < first.total) {
        const next = requireData(await api.GET("/api/v1/payroll/runs/{id}/exceptions", {
          params: { path: { id }, query: { limit: PAGE_LIMIT, offset: items.length } },
          signal,
        })) as ExceptionPage;
        if (next.items.length === 0) break;
        items.push(...next.items);
      }
      return { ...first, items };
    },
    resolveException: async (
      id: string,
      exId: string,
      input: { action: ResolveExceptionAction; reason?: string },
      signal?: AbortSignal,
    ) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/exceptions/{exceptionId}/resolve", {
          params: { path: { id, exceptionId: exId } },
          body: input,
          signal,
        })) as PayrollException,
    submit: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/submit", {
        params: { path: { id } },
        signal,
      })) as PayrollRunDetail,
    decide: async (
      id: string,
      input: { decision: RunDecision; reason?: string },
      signal?: AbortSignal,
    ) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/decision", {
        params: { path: { id } },
        body: input,
        signal,
      })) as PayrollRunDetail,
    withdraw: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/withdraw", {
        params: { path: { id } },
        signal,
      })) as PayrollRunDetail,
    scheduleDisbursement: async (id: string, scheduledAt: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/schedule-disbursement", {
          params: { path: { id } },
          body: { scheduled_at: scheduledAt },
          signal,
        })) as Disbursement,
    attestDisbursement: async (
      id: string,
      input: { status: Exclude<DisbursementStatus, "SCHEDULED">; reason?: string },
      signal?: AbortSignal,
    ) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/disbursement/attest", {
        params: { path: { id } },
        body: input,
        signal,
      })) as Disbursement,
    issuePayslips: async (id: string, signal?: AbortSignal) =>
      requireData(await api.POST("/api/v1/payroll/runs/{id}/issue-payslips", {
          params: { path: { id } },
          signal,
        })) as PayslipDeliverySummary,
    payslipDelivery: async (id: string, signal?: AbortSignal) =>
      requireData(await api.GET("/api/v1/payroll/runs/{id}/payslip-delivery", {
          params: { path: { id } },
          signal,
        })) as PayslipDeliverySummary,
  };
}

export type PayrollApi = ReturnType<typeof createPayrollApi>;
