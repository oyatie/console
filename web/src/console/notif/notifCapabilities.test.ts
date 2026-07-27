import { describe, expect, it } from "vitest";

import { deriveNotifCapabilities } from "./notifCapabilities";

describe("deriveNotifCapabilities", () => {
  it("grants the all-employee surface only to an authenticated session", () => {
    expect(deriveNotifCapabilities(true)).toEqual({ canRead: true, canAck: true, canMute: true });
    expect(deriveNotifCapabilities(false)).toEqual({ canRead: false, canAck: false, canMute: false });
  });
});
