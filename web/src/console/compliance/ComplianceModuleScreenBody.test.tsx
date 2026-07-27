import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { AuthContext, type AuthContextValue } from "../../context/auth";
import { ComplianceModuleScreenBody } from "./ComplianceModuleScreenBody";

function renderBody(roles: string[], featureGrants: string[] = []) {
  const api = {
    GET: vi.fn((path: string) => {
      if (path === "/api/v1/compliance/obligations")
        return Promise.resolve({
          data: {
            items: [
              {
                id: "cp-1",
                code: "CP-0001",
                title: "근로시간 준수",
                description: "설명",
                obligation_type: "LEGAL",
                scope: {
                  kind: "ORG",
                  scope_ref: null,
                  branch_id: null,
                  site_id: null,
                },
                owner_user_id: null,
                severity: "HIGH",
                status: "ACTIVE",
                effective_from: null,
                effective_to: null,
                review_cadence: null,
                next_review_on: null,
                metadata: {},
                created_by: "user",
                updated_by: "user",
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-01T00:00:00Z",
              },
            ],
            limit: 100,
            offset: 0,
            total: 1,
          },
        });
      if (path === "/api/v1/compliance/regulations")
        return Promise.resolve({
          data: {
            items: [
              {
                id: "rg-1",
                code: "RG-0001",
                title: "근로기준법",
                jurisdiction: "대한민국",
                citation: "제50조",
                impact_area: "인사",
                impact_summary: "규정",
                risk_level: "HIGH",
                status: "ACTIVE",
                owner_user_id: null,
                effective_from: null,
                effective_to: null,
                review_due_on: null,
                metadata: {},
                created_by: "user",
                updated_by: "user",
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-01T00:00:00Z",
              },
            ],
            limit: 100,
            offset: 0,
            total: 1,
          },
        });
      if (path === "/api/v1/compliance/evidence-bindings")
        return Promise.resolve({
          data: {
            items: [
              {
                id: "eb-1",
                control_id: "control-1",
                obligation_id: null,
                evidence_target_type: "external_document",
                evidence_target_id: "POL-MANAGE",
                source_audit_event_id: null,
                status: "PROPOSED",
                confidence: "HIGH",
                collected_at: null,
                collected_by: null,
                valid_from: null,
                valid_to: null,
                hash_sha256: null,
                metadata: {},
                created_by: "user",
                updated_by: "user",
                created_at: "2026-01-01T00:00:00Z",
                updated_at: "2026-01-01T00:00:00Z",
              },
            ],
            limit: 100,
            offset: 0,
            total: 1,
          },
        });
      return Promise.resolve({
        data: { items: [], limit: 100, offset: 0, total: 0 },
      });
    }),
  } as unknown as ConsoleApiClient;
  const authValue = {
    session: {
      access_token: "cp-token",
      org_id: "00000000-0000-0000-0000-0000000000a1",
      user_id: "00000000-0000-0000-0000-0000000000b1",
      client_session_incarnation: "compliance-test-incarnation",
      roles,
      feature_grants: featureGrants,
    },
    restoring: false,
    login: vi.fn(),
    logout: vi.fn(),
    refresh: vi.fn(),
    acceptTokens: vi.fn(),
    clearPasskeySetup: vi.fn(),
    api,
    viewAs: undefined,
    enterViewAs: vi.fn(),
    exitViewAs: vi.fn(),
  } as unknown as AuthContextValue;
  return render(
    <AuthContext.Provider value={authValue}>
      <ComplianceModuleScreenBody />
    </AuthContext.Provider>,
  );
}

describe("ComplianceModuleScreenBody", () => {
  it("renders the CP-/RG-/FW- catalog for an integrity-role reader", async () => {
    renderBody(["EXECUTIVE"]);
    // CP-0001 is the auto-selected first row, so it renders in both the list
    // cell and the detail pane.
    expect((await screen.findAllByText("CP-0001")).length).toBeGreaterThan(0);
    // Regulation + framework kinds share the same catalog surface.
    expect(screen.getByText("RG-0001")).toBeInTheDocument();
  });

  it("renders the catalog for a holder of the compliance-domain-read feature grant", async () => {
    renderBody(["MECHANIC"], ["compliance_domain_read"]);
    expect((await screen.findAllByText("CP-0001")).length).toBeGreaterThan(0);
  });

  it("shows the audit trail to a built-in SUPER_ADMIN without custom grants", async () => {
    renderBody(["SUPER_ADMIN"]);
    expect(await screen.findByText("감사")).toBeInTheDocument();
  });

  it("keeps audit trail hidden from a non-admin reader without the audit grant", async () => {
    renderBody(["EXECUTIVE"]);
    await screen.findAllByText("CP-0001");
    expect(screen.queryByText("감사")).not.toBeInTheDocument();
  });

  it("denies the whole surface by omission for an unauthorized role", async () => {
    renderBody(["MEMBER"]);
    // GenericModuleScreen gates its entire content plane on the read action;
    // an unauthorized session never sees a catalog row (no disabled ghost).
    await waitFor(() => {
      expect(screen.queryByText("CP-0001")).not.toBeInTheDocument();
    });
  });
  it("permits proposal for an exact read-plus-link session but omits acceptance", async () => {
    renderBody(
      ["MEMBER"],
      ["compliance_domain_read", "compliance_evidence_link"],
    );
    expect(
      await screen.findByRole("button", { name: "Propose binding" }),
    ).toBeInTheDocument();
    await screen.findByRole("button", { name: /POL-MANAGE details/ });
    const details = await screen.findByRole("button", {
      name: /POL-MANAGE details/,
    });
    details.click();
    expect(
      screen.queryByRole("button", { name: "Accept evidence" }),
    ).not.toBeInTheDocument();
  });

  it("permits acceptance for an exact read-plus-manage session but omits proposal", async () => {
    renderBody(
      ["MEMBER"],
      ["compliance_domain_read", "compliance_domain_manage"],
    );
    expect(
      await screen.findByRole("button", { name: /POL-MANAGE details/ }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Propose binding" }),
    ).not.toBeInTheDocument();
    screen.getByRole("button", { name: /POL-MANAGE details/ }).click();
    expect(
      await screen.findByRole("button", { name: "Accept evidence" }),
    ).toBeInTheDocument();
  });

  it.each(["compliance_evidence_link", "compliance_domain_manage"])(
    "fails closed when %s is present without compliance-domain-read",
    async (actionGrant) => {
      renderBody(["MEMBER"], [actionGrant]);
      await waitFor(() => {
        expect(screen.queryByText("CP-0001")).not.toBeInTheDocument();
        expect(
          screen.queryByRole("region", { name: "Evidence bindings" }),
        ).not.toBeInTheDocument();
      });
    },
  );
});
