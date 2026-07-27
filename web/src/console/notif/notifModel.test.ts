import { describe, expect, it } from "vitest";

import { EXPOSED_SCREEN_KEYS, MOUNTED_SCREEN_KEYS, consoleScreenPath, isExposedScreenKey } from "../shell/nav";
import { linkKey, rowTarget, sameLink, timeLabel } from "./notifModel";

const objectLink = { type: "object", kind: "approval_run", id: "run-1" } as const;

describe("notif link model", () => {
  it("keys object and screen links distinctly and symmetrically", () => {
    expect(linkKey(objectLink)).toBe("object:approval_run:run-1");
    expect(linkKey({ type: "screen", screen: "sales" })).toBe("screen:sales");
    expect(sameLink(objectLink, { ...objectLink })).toBe(true);
    expect(sameLink(objectLink, { type: "screen", screen: "sales" })).toBe(false);
  });

  it("drills object links to the source object", () => {
    expect(rowTarget(objectLink)).toEqual({ type: "object", kind: "approval_run", id: "run-1" });
  });

  // Asserted against the live exposure manifest rather than a hardcoded screen:
  // EXPOSED_SCREEN_KEYS is evidence-gated and has legitimately been emptied, so a
  // fixed exemplar would encode a snapshot instead of the ADR-0025 invariant.
  it("navigates only evidence-exposed screens (ADR-0025)", () => {
    for (const key of EXPOSED_SCREEN_KEYS) {
      expect(rowTarget({ type: "screen", screen: key })).toEqual({
        type: "screen",
        path: consoleScreenPath(key),
      });
    }
    // Every mounted-but-dark screen stays an ack-able non-link, as does an unknown key.
    for (const key of MOUNTED_SCREEN_KEYS.filter((k) => !isExposedScreenKey(k))) {
      expect(rowTarget({ type: "screen", screen: key })).toBeUndefined();
    }
    expect(rowTarget({ type: "screen", screen: "not-a-screen" })).toBeUndefined();
  });

  it("formats a compact row timestamp and passes unparsable values through", () => {
    expect(timeLabel("2026-07-24T09:00:00Z")).toMatch(/\d/);
    expect(timeLabel("nonsense")).toBe("nonsense");
  });
});
