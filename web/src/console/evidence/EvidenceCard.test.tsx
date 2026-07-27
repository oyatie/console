import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ApiCallError } from "../../api/ontologyActions";
import { ko } from "../../i18n/ko";
import { PolicyGateProvider, type PolicyGate } from "../policy";
import { EvidenceCard, type EvidenceCardProps } from "./EvidenceCard";
import { evidenceFixtures } from "./evidenceFixtures";
import { EvidenceDetailRefreshError, type VerifyEvidence } from "./types";

const T = ko.console.evidence;
const allowGate: PolicyGate = { can: () => true };

const [heldEvidence, plainEvidence] = evidenceFixtures();

function noopHoldProps(): Pick<
  EvidenceCardProps,
  "applyHold" | "requestHoldRelease" | "decideHoldRelease" | "releaseHold"
> {
  return {
    applyHold: vi.fn().mockResolvedValue(undefined),
    requestHoldRelease: vi.fn().mockResolvedValue({ requestRef: "req-1", requestedBy: "user-a" }),
    decideHoldRelease: vi.fn().mockResolvedValue(undefined),
    releaseHold: vi.fn().mockResolvedValue(undefined),
  };
}

function renderCard(
  gate?: PolicyGate,
  verify: VerifyEvidence = vi.fn().mockResolvedValue({ state: "unavailable", copyVerdicts: new Map() }),
  detail = heldEvidence,
  overrides: Partial<EvidenceCardProps> = {},
) {
  const card = (
    <EvidenceCard detail={detail} verify={verify} {...noopHoldProps()} {...overrides} />
  );
  return render(gate ? <PolicyGateProvider gate={gate}>{card}</PolicyGateProvider> : card);
}

describe("EvidenceCard chips", () => {
  it("shows fixity, TSA, admissibility, and legal-hold chips", () => {
    renderCard(allowGate);
    expect(screen.getByText(/^SHA-256 /)).toBeTruthy();
    expect(screen.getByText(T.tsa.VERIFIED)).toBeTruthy();
    expect(screen.getByText(T.admissibility.ADMISSIBLE)).toBeTruthy();
    expect(screen.getByText(T.hold.active)).toBeTruthy();
  });
});

describe("EvidenceCard WORM split", () => {
  it("badges the original as WORM-sealed and lists derivatives as immutable linked copies", () => {
    renderCard(allowGate);
    expect(screen.getByText(T.worm.sealed)).toBeTruthy();
    expect(screen.getByText(T.derivativeKinds.TRANSCODED)).toBeTruthy();
    expect(screen.getByText(T.derivativeKinds.THUMBNAIL)).toBeTruthy();
  });

  it("does not render original, derivative, or ZIP preview controls without a real read endpoint", () => {
    renderCard(allowGate);
    expect(screen.queryByRole("button", { name: T.worm.viewOriginal })).toBeNull();
    expect(screen.queryByRole("button", { name: T.worm.viewDerived })).toBeNull();
    expect(screen.queryByRole("button", { name: T.worm.zip.title })).toBeNull();
    expect(screen.queryByText(T.worm.previewPending)).toBeNull();
  });
});

describe("EvidenceCard custody timeline", () => {
  it("maps wire custody stages to display labels (수집/봉인/열람)", () => {
    renderCard(allowGate);
    expect(screen.getByText(T.custody.stages.REGISTERED)).toBeTruthy();
    expect(screen.getByText(T.custody.stages.WORM_REPLICATED)).toBeTruthy();
    expect(screen.getByText(T.custody.stages.LEGAL_HOLD_APPLIED)).toBeTruthy();
  });
});

describe("EvidenceCard verify affordance", () => {
  it("calls the verify hook and surfaces the verified outcome + per-copy verdicts", async () => {
    const verify = vi
      .fn<VerifyEvidence>()
      .mockResolvedValue({ state: "verified", processedAt: null, copyVerdicts: new Map([["cp-12-orig", "MATCH"]]) });
    renderCard(allowGate, verify);
    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(screen.getByText(T.actions.verifyOk)).toBeTruthy();
    });
    expect(screen.getByText(T.copyVerdict.MATCH)).toBeTruthy();
    expect(verify).toHaveBeenCalledTimes(1);
  });

  it("clears a prior MATCH verdict chip when a later verify comes back unavailable", async () => {
    const verify = vi
      .fn<VerifyEvidence>()
      .mockResolvedValueOnce({ state: "verified", processedAt: null, copyVerdicts: new Map([["cp-13-orig", "MATCH"]]) })
      .mockResolvedValueOnce({ state: "unavailable", copyVerdicts: new Map() });
    renderCard(allowGate, verify, plainEvidence);
    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(screen.getByText(T.copyVerdict.MATCH)).toBeTruthy();
    });
    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(screen.getByText(T.actions.verifyPending)).toBeTruthy();
    });
    // The stale green MATCH chip must be gone, not lingering as fake fixity.
    expect(screen.queryByText(T.copyVerdict.MATCH)).toBeNull();
  });

  it("keeps indeterminate copy-level storage evidence visible without claiming integrity failure", async () => {
    const verify = vi.fn<VerifyEvidence>().mockResolvedValue({
      state: "unavailable",
      copyVerdicts: new Map([["cp-13-orig", "CHECKSUM_UNAVAILABLE"]]),
    });
    renderCard(allowGate, verify, plainEvidence);

    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(screen.getByText(T.actions.verifyPending)).toBeTruthy();
    });
    expect(screen.getByText(T.copyVerdict.CHECKSUM_UNAVAILABLE)).toBeTruthy();
    expect(screen.queryByText(T.actions.verifyFail)).toBeNull();
  });

  it("shows an authorization denial without falsely claiming a fixity failure or offering a futile retry", async () => {
    const verify = vi.fn<VerifyEvidence>().mockRejectedValue(new ApiCallError(403));
    renderCard(allowGate, verify);

    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));

    await waitFor(() => {
      expect(screen.getByText(ko.page.permissionDenied)).toBeTruthy();
    });
    expect(screen.queryByText(T.actions.verifyFail)).toBeNull();
    expect(screen.getByRole("button", { name: T.actions.verify })).toBeDisabled();
  });

  it("keeps a transient verification failure retryable and replaces it with the next authoritative verdict", async () => {
    const verify = vi
      .fn<VerifyEvidence>()
      .mockRejectedValueOnce(new ApiCallError(500))
      .mockResolvedValueOnce({ state: "verified", processedAt: null, copyVerdicts: new Map([["cp-13-orig", "MATCH"]]) });
    renderCard(allowGate, verify, plainEvidence);

    const action = screen.getByRole("button", { name: T.actions.verify });
    fireEvent.click(action);
    await waitFor(() => {
      expect(screen.getByText(T.actions.verifyFail)).toBeTruthy();
    });
    expect(action).not.toBeDisabled();

    fireEvent.click(action);
    await waitFor(() => {
      expect(screen.getByText(T.actions.verifyOk)).toBeTruthy();
    });
    expect(screen.getByText(T.copyVerdict.MATCH)).toBeTruthy();
    expect(verify).toHaveBeenCalledTimes(2);
  });

  it("clears a prior MATCH verdict chip when a later verify throws", async () => {
    const verify = vi
      .fn<VerifyEvidence>()
      .mockResolvedValueOnce({ state: "verified", processedAt: null, copyVerdicts: new Map([["cp-13-orig", "MATCH"]]) })
      .mockRejectedValueOnce(new Error("network"));
    renderCard(allowGate, verify, plainEvidence);
    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(screen.getByText(T.copyVerdict.MATCH)).toBeTruthy();
    });
    fireEvent.click(screen.getByRole("button", { name: T.actions.verify }));
    await waitFor(() => {
      expect(screen.getByText(T.actions.verifyFail)).toBeTruthy();
    });
    expect(screen.queryByText(T.copyVerdict.MATCH)).toBeNull();
  });
});

describe("EvidenceCard PBAC (deny-by-omission)", () => {
  it("hides mutable legal-hold controls without a gate while retaining the real verify action", () => {
    renderCard(undefined);
    expect(screen.queryByRole("button", { name: T.hold.requestRelease })).toBeNull();
    expect(screen.getByRole("button", { name: T.actions.verify })).toBeTruthy();
  });

  it("shows only backed legal-hold controls for the compliance persona", () => {
    renderCard(allowGate);
    expect(screen.getByRole("button", { name: T.hold.requestRelease })).toBeTruthy();
    expect(screen.queryByRole("button", { name: T.actions.transfer })).toBeNull();
    expect(screen.queryByRole("button", { name: T.actions.dispose })).toBeNull();
  });
});

describe("EvidenceCard hold-release four-eyes flow (fail-closed)", () => {
  it("opens a pending approval, blocks a self-decide, and never releases on decide alone", async () => {
    const requestHoldRelease = vi
      .fn()
      .mockResolvedValue({ requestRef: "req-1", requestedBy: "user-a" });
    const decideHoldRelease = vi.fn().mockResolvedValue(undefined);
    const releaseHold = vi.fn().mockResolvedValue(undefined);
    renderCard(allowGate, vi.fn().mockResolvedValue({ state: "unavailable", copyVerdicts: new Map() }), heldEvidence, {
      currentUserId: "user-a",
      requestHoldRelease,
      decideHoldRelease,
      releaseHold,
    });

    fireEvent.click(screen.getByRole("button", { name: T.hold.requestRelease }));
    await waitFor(() => {
      expect(screen.getByText(T.hold.releasePending)).toBeTruthy();
    });
    // requestedBy === currentUserId → self-decide is blocked in the UI.
    expect(screen.getByText(T.hold.selfDecideBlocked)).toBeTruthy();
    expect(screen.queryByRole("button", { name: T.hold.decideApprove })).toBeNull();
    expect(decideHoldRelease).not.toHaveBeenCalled();
    expect(releaseHold).not.toHaveBeenCalled();
  });

  it("lets a distinct approver decide, then finalizes the real release call", async () => {
    const requestHoldRelease = vi
      .fn()
      .mockResolvedValue({ requestRef: "req-1", requestedBy: "user-a" });
    const decideHoldRelease = vi.fn().mockResolvedValue(undefined);
    const releaseHold = vi.fn().mockResolvedValue(undefined);
    renderCard(allowGate, vi.fn().mockResolvedValue({ state: "unavailable", copyVerdicts: new Map() }), heldEvidence, {
      currentUserId: "user-b",
      requestHoldRelease,
      decideHoldRelease,
      releaseHold,
    });

    fireEvent.click(screen.getByRole("button", { name: T.hold.requestRelease }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: T.hold.decideApprove })).toBeTruthy();
    });

    fireEvent.click(screen.getByRole("button", { name: T.hold.decideApprove }));
    await waitFor(() => {
      expect(decideHoldRelease).toHaveBeenCalledWith("req-1", "user-a", "approved");
    });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: T.hold.release })).toBeTruthy();
    });

    fireEvent.click(screen.getByRole("button", { name: T.hold.release }));
    await waitFor(() => {
      expect(releaseHold).toHaveBeenCalledWith(
        expect.objectContaining({ holdId: "hold-12-1", fourEyesRequestRef: "req-1" }),
      );
    });
  });

  it("keeps hold-apply inputs and exposes an authoritative reread retry after a successful mutation refresh fails", async () => {
    const retry = vi.fn().mockResolvedValue(undefined);
    const applyHold = vi.fn().mockRejectedValue(new EvidenceDetailRefreshError(retry));
    renderCard(allowGate, vi.fn().mockResolvedValue({ state: "unavailable", copyVerdicts: new Map() }), plainEvidence, {
      applyHold,
    });

    fireEvent.change(screen.getByRole("textbox", { name: T.hold.caseRef }), { target: { value: "CASE-1" } });
    fireEvent.change(screen.getByRole("textbox", { name: T.hold.basisLabel }), { target: { value: "law" } });
    fireEvent.change(screen.getByRole("textbox", { name: T.hold.reasonLabel }), { target: { value: "preserve" } });
    fireEvent.click(screen.getByRole("button", { name: T.hold.apply }));

    await waitFor(() => {
      expect(screen.getByText(T.hold.applyRefreshFailed)).toBeTruthy();
    });
    expect(screen.getByRole<HTMLInputElement>("textbox", { name: T.hold.caseRef }).value).toBe("CASE-1");
    fireEvent.click(screen.getByRole("button", { name: T.hold.refreshRetry }));
    await waitFor(() => {
      expect(retry).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(screen.getByRole<HTMLInputElement>("textbox", { name: T.hold.caseRef }).value).toBe("");
    });
  });

  it("keeps a successful hold release explicit and retryable when its authoritative reread fails", async () => {
    const retry = vi.fn().mockResolvedValue(undefined);
    const requestHoldRelease = vi.fn().mockResolvedValue({ requestRef: "req-1", requestedBy: "user-a" });
    const decideHoldRelease = vi.fn().mockResolvedValue(undefined);
    const releaseHold = vi.fn().mockRejectedValue(new EvidenceDetailRefreshError(retry));
    renderCard(allowGate, vi.fn().mockResolvedValue({ state: "unavailable", copyVerdicts: new Map() }), heldEvidence, {
      currentUserId: "user-b",
      requestHoldRelease,
      decideHoldRelease,
      releaseHold,
    });

    fireEvent.click(screen.getByRole("button", { name: T.hold.requestRelease }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: T.hold.decideApprove })).toBeTruthy();
    });
    fireEvent.click(screen.getByRole("button", { name: T.hold.decideApprove }));
    await waitFor(() => {
      expect(screen.getByRole("button", { name: T.hold.release })).toBeTruthy();
    });
    fireEvent.change(screen.getByRole("textbox", { name: T.hold.reasonLabel }), { target: { value: "approved release" } });
    fireEvent.click(screen.getByRole("button", { name: T.hold.release }));

    await waitFor(() => {
      expect(screen.getByText(T.hold.releaseRefreshFailed)).toBeTruthy();
    });
    expect(screen.queryByText(T.hold.active)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: T.hold.refreshRetry }));
    await waitFor(() => {
      expect(retry).toHaveBeenCalledTimes(1);
    });
  });

});
