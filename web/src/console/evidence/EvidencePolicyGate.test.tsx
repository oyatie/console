import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { clearAuthorizeBulkCache } from "../../api/authorizeBulk";
import type { ConsoleApiClient } from "../../api/client";
import type { AuthSession } from "../../context/auth";
import { ko } from "../../i18n/ko";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { BulkPolicyGateProvider } from "../policy";
import { EvidenceCard } from "./EvidenceCard";
import { evidenceFixtures } from "./evidenceFixtures";
import { EVIDENCE_ACTIONS, type VerifyEvidence } from "./types";

const T = ko.console.evidence;
const session: AuthSession = {
  access_token: "t",
  org_id: "11111111-1111-4111-8111-111111111111",
  user_id: "u1",
  // A local role must not bypass the server-backed bulk decision.
  roles: ["SUPER_ADMIN"],
};
const [heldEvidence] = evidenceFixtures();

function effects(...effect: ("allow" | "deny")[]) {
  return {
    data: {
      decisions: effect.map((entry) => ({
        effect: entry,
        determining_policies: [],
        errors: [],
        reason: "",
      })),
    },
  };
}

function mount(post: ReturnType<typeof vi.fn>) {
  const api = { POST: post } as unknown as ConsoleApiClient;
  const verify: VerifyEvidence = vi
    .fn()
    .mockResolvedValue({ state: "unavailable", copyVerdicts: new Map() });
  return render(
    <AuthTestProvider session={session} overrides={{ api }}>
      <BulkPolicyGateProvider actions={Object.values(EVIDENCE_ACTIONS)}>
        <EvidenceCard
          detail={heldEvidence}
          verify={verify}
          applyHold={vi.fn().mockResolvedValue(undefined)}
          requestHoldRelease={vi
            .fn()
            .mockResolvedValue({ requestRef: "request-1", requestedBy: "u2" })}
          decideHoldRelease={vi.fn().mockResolvedValue(undefined)}
          releaseHold={vi.fn().mockResolvedValue(undefined)}
        />
      </BulkPolicyGateProvider>
    </AuthTestProvider>,
  );
}

beforeEach(() => {
  clearAuthorizeBulkCache();
});

describe("Evidence legal-hold server authorization", () => {
  it("omits legal-hold controls while the server decision is pending", () => {
    mount(vi.fn().mockReturnValue(new Promise(() => {})));
    expect(
      screen.queryByRole("button", { name: T.hold.requestRelease }),
    ).toBeNull();
  });

  it("omits legal-hold controls when the server denies even a SUPER_ADMIN role", async () => {
    const post = vi
      .fn()
      .mockResolvedValue(effects("deny", "deny", "deny", "deny"));
    mount(post);
    await waitFor(() => {
      expect(post).toHaveBeenCalledTimes(1);
    });
    expect(
      screen.queryByRole("button", { name: T.hold.requestRelease }),
    ).toBeNull();
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it("fails closed on an authorization error and omits legal-hold controls", async () => {
    mount(vi.fn().mockResolvedValue({ error: { message: "unavailable" } }));
    await waitFor(() => {
      expect(screen.getByRole("alert")).toBeTruthy();
    });
    expect(
      screen.queryByRole("button", { name: T.hold.requestRelease }),
    ).toBeNull();
  });
});
