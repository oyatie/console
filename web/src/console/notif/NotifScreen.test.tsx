import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, useLocation } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";

import type * as NavModule from "../shell/nav";

// The exposure manifest is evidence-gated and legitimately empties as screens
// await their ADR-0025 evidence, so this suite pins one exposed screen rather
// than depending on whatever the live manifest happens to hold.
vi.mock("../shell/nav", async () => {
  const actual = await vi.importActual<typeof NavModule>("../shell/nav");
  return { ...actual, EXPOSED_SCREEN_KEYS: ["sales"], isExposedScreenKey: (s: string) => s === "sales" };
});

import { createConsoleApiClient } from "../../api/client";
import { notifStrings as text } from "../../i18n/notif";
import { NotifScreen } from "./NotifScreen";
import type { NotifCapabilities } from "./notifCapabilities";
import type { NotificationObjectGroup, NotificationSummary } from "./notifApi";

const all: NotifCapabilities = { canRead: true, canAck: true, canMute: true };
const denied: NotifCapabilities = { canRead: false, canAck: false, canMute: false };

const RUN_ID = "9a8b7c6d-5e4f-4a3b-8c1d-0e9f8a7b6c5d";
const USER_ID = "0a1b2c3d-4e5f-4a6b-8c7d-8e9f0a1b2c3d";

const row = (id: string, over: Partial<NotificationSummary>): NotificationSummary => ({
  id,
  recipient_user_id: USER_ID,
  category: "결재",
  kind: "info",
  text: "",
  link: { type: "object", kind: "approval_run", id: RUN_ID },
  unread: true,
  created_at: "2026-07-24T09:00:00Z",
  read_at: null,
  resolved_at: null,
  ...over,
});

const approvalRow = row("11111111-1111-4111-8111-111111111111", { text: "AP-3121 결재 대기" });
const salesRow = row("22222222-2222-4222-8222-222222222222", {
  category: "문서",
  text: "판매 문의 3건",
  link: { type: "screen", screen: "sales" },
  unread: false,
  read_at: "2026-07-24T08:00:00Z",
});
const darkRow = row("33333333-3333-4333-8333-333333333333", {
  category: "멘션",
  text: "새 결재 도착",
  link: { type: "screen", screen: "mywork" },
});

// English producer key on purpose: the chip must localize AND tone it as 결재.
const approvalGroup: NotificationObjectGroup = {
  link: approvalRow.link,
  total: 3,
  unread: 2,
  categories: [{ category: "approval", unread: 2 }],
  latest: approvalRow,
  muted: false,
};

const mutePolicy = {
  id: "p-1",
  scope: "object" as const,
  link: approvalRow.link,
  action: "mute",
  created_at: "2026-07-24T09:00:00Z",
};

function json(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), { status, headers: { "content-type": "application/json" } });
}

type Handler = (request: Request, url: URL) => Response | Promise<Response>;

function stubFetch(handlers: Partial<Record<string, Handler>>) {
  const calls: { method: string; url: URL; body: string }[] = [];
  vi.stubGlobal("fetch", vi.fn(async (input: Request | string, init?: RequestInit) => {
    const request = input instanceof Request ? input : new Request(input, init);
    const url = new URL(request.url);
    calls.push({ method: request.method, url, body: await request.clone().text() });
    const handler = handlers[`${request.method} ${url.pathname}`];
    if (!handler) throw new Error(`unhandled ${request.method} ${url.pathname}`);
    return handler(request, url);
  }));
  return calls;
}

function happyHandlers(): Record<string, Handler> {
  return {
    "GET /api/v1/me/notifications": () => json({ items: [approvalRow, salesRow, darkRow], next_cursor: null }),
    "GET /api/v1/me/notifications/summary": () =>
      json({ total_unread: 2, muted_unread: 1, by_category: [{ category: "결재", unread: 2 }] }),
    [`GET /api/objects/approval_run/${RUN_ID}`]: () =>
      json({ kind: "approval_run", id: RUN_ID, code: "AP-3121", title: "긴급 결재", status: "in_progress", exists: true }),
  };
}

function LocationProbe() {
  const location = useLocation();
  return <output data-testid="location">{location.pathname}</output>;
}

function renderScreen(capabilities: NotifCapabilities = all) {
  const api = createConsoleApiClient("bearer-token");
  return render(
    <MemoryRouter initialEntries={["/console/notif"]}>
      <NotifScreen api={api} actorId={USER_ID} capabilities={capabilities} sessionKey="session-a" />
      <LocationProbe />
    </MemoryRouter>,
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("NotifScreen", () => {
  it("denies an unauthenticated session before any fetch", () => {
    const calls = stubFetch({});
    renderScreen(denied);
    expect(screen.getByText(text.denied)).toBeVisible();
    expect(calls).toHaveLength(0);
  });

  it("renders backend rows, the mute-aware unread chip, and a resolved token link", async () => {
    stubFetch(happyHandlers());
    renderScreen();
    expect(await screen.findByRole("button", { name: approvalRow.text })).toBeVisible();
    expect(screen.getByLabelText(text.unreadBadge)).toHaveTextContent("2");
    expect(screen.getByLabelText(text.mutedBadge)).toHaveTextContent("1");
    expect(screen.getAllByText("결재").length).toBeGreaterThan(0);
    // Source-object head resolved -> the in-text code is a live token chip.
    expect(await screen.findByRole("button", { name: /AP-3121 긴급 결재/ })).toBeVisible();
  });

  it("shows a truthful empty state", async () => {
    stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications": () => json({ items: [], next_cursor: null }),
    });
    renderScreen();
    expect(await screen.findByText(text.empty)).toBeVisible();
  });

  it("recovers from a server error through retry", async () => {
    let failures = 1;
    stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications": () => {
        if (failures > 0) {
          failures -= 1;
          return json({ error: { code: "internal", message: "boom" } }, 500);
        }
        return json({ items: [approvalRow], next_cursor: null });
      },
    });
    renderScreen();
    expect(await screen.findByRole("alert")).toHaveTextContent(text.loadError);
    await userEvent.click(screen.getByRole("button", { name: text.retry }));
    expect(await screen.findByRole("button", { name: approvalRow.text })).toBeVisible();
  });

  it("treats a backend denial as denied, not an error with retry", async () => {
    stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications": () =>
        json({ error: { code: "unauthorized", message: "denied" } }, 401),
    });
    renderScreen();
    expect(await screen.findByText(text.denied)).toBeVisible();
    expect(screen.queryByRole("button", { name: text.retry })).toBeNull();
  });

  it("refetches with unread=true when the 미확인 filter is picked", async () => {
    const calls = stubFetch(happyHandlers());
    renderScreen();
    await screen.findByRole("button", { name: approvalRow.text });
    await userEvent.click(screen.getByRole("button", { name: text.filterUnread }));
    await waitFor(() => {
      expect(calls.some((call) =>
        call.url.pathname === "/api/v1/me/notifications" && call.url.searchParams.get("unread") === "true",
      )).toBe(true);
    });
  });

  it("toggles read state in both directions from the authoritative response", async () => {
    const calls = stubFetch({
      ...happyHandlers(),
      [`POST /api/v1/me/notifications/${approvalRow.id}/read`]: () =>
        json({ ...approvalRow, unread: false, read_at: "2026-07-24T09:05:00Z" }),
      [`POST /api/v1/me/notifications/${approvalRow.id}/unread`]: () =>
        json({ ...approvalRow, unread: true, read_at: "2026-07-24T09:05:00Z" }),
    });
    renderScreen();
    const primary = await screen.findByRole("button", { name: approvalRow.text });
    const item = primary.closest("li");
    if (!item) throw new Error("row list item missing");
    await userEvent.click(within(item).getByRole("button", { name: text.markRead }));
    await userEvent.click(await within(item).findByRole("button", { name: text.markUnread }));
    expect(await within(item).findByRole("button", { name: text.markRead })).toBeVisible();
    expect(calls.some((call) => call.url.pathname.endsWith("/unread"))).toBe(true);
  });

  it("acks a row via keyboard activation on its primary control", async () => {
    const calls = stubFetch({
      ...happyHandlers(),
      [`POST /api/v1/me/notifications/${approvalRow.id}/read`]: () =>
        json({ ...approvalRow, unread: false, read_at: "2026-07-24T09:05:00Z" }),
    });
    renderScreen();
    const primary = await screen.findByRole("button", { name: approvalRow.text });
    primary.focus();
    await userEvent.keyboard("{Enter}");
    await waitFor(() => {
      expect(calls.some((call) => call.method === "POST" && call.url.pathname.endsWith(`/${approvalRow.id}/read`))).toBe(true);
    });
  });

  it("marks everything read through the backend and reloads", async () => {
    let marked = false;
    stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications": () =>
        json(marked
          ? { items: [{ ...approvalRow, unread: false, read_at: "2026-07-24T09:05:00Z" }], next_cursor: null }
          : { items: [approvalRow], next_cursor: null }),
      "POST /api/v1/me/notifications/read-all": () => {
        marked = true;
        return json({ marked: 1 });
      },
    });
    renderScreen();
    await screen.findByRole("button", { name: approvalRow.text });
    await userEvent.click(screen.getByRole("button", { name: text.markAllRead }));
    expect(await screen.findByRole("button", { name: text.markUnread })).toBeVisible();
  });

  it("navigates an exposed screen link and keeps a dark screen link ack-only", async () => {
    const calls = stubFetch({
      ...happyHandlers(),
      [`POST /api/v1/me/notifications/${darkRow.id}/read`]: () =>
        json({ ...darkRow, unread: false, read_at: "2026-07-24T09:05:00Z" }),
    });
    renderScreen();
    await screen.findByRole("button", { name: darkRow.text });
    await userEvent.click(screen.getByRole("button", { name: darkRow.text }));
    await waitFor(() => {
      expect(calls.some((call) => call.method === "POST" && call.url.pathname.endsWith(`/${darkRow.id}/read`))).toBe(true);
    });
    expect(screen.getByTestId("location")).toHaveTextContent("/console/notif");
    await userEvent.click(screen.getByRole("button", { name: salesRow.text }));
    await waitFor(() => {
      expect(screen.getByTestId("location")).toHaveTextContent("/console/sales");
    });
  });

  it("aggregates by object with counts and category chips, and mutes per policy", async () => {
    let muted = false;
    const calls = stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications/by-object": () =>
        json({ items: [{ ...approvalGroup, muted }], next_cursor: null }),
      "GET /api/v1/me/notification-policies": () => json({ items: muted ? [mutePolicy] : [] }),
      "PUT /api/v1/me/notification-policies": () => {
        muted = true;
        return json(mutePolicy);
      },
      [`DELETE /api/v1/me/notification-policies/${mutePolicy.id}`]: () => {
        muted = false;
        return new Response(null, { status: 204 });
      },
    });
    renderScreen();
    await screen.findByRole("button", { name: approvalRow.text });
    await userEvent.click(screen.getByRole("button", { name: text.viewByObject }));
    expect(await screen.findByText("2/3")).toBeVisible();
    const categoryChip = screen.getByText("결재 2");
    expect(categoryChip).toBeVisible();
    expect(categoryChip).toHaveAttribute("style", expect.stringContaining("var(--accent-bg)"));
    await userEvent.click(screen.getByRole("button", { name: text.mute }));
    const unmute = await screen.findByRole("button", { name: text.unmute });
    expect(unmute).toHaveAttribute("aria-pressed", "true");
    await userEvent.click(unmute);
    expect(await screen.findByRole("button", { name: text.mute })).toBeVisible();
    expect(calls.some((call) => call.method === "PUT" && call.body.includes("\"scope\":\"object\""))).toBe(true);
    expect(calls.some((call) => call.method === "DELETE" && call.url.pathname.endsWith(mutePolicy.id))).toBe(true);
  });

  it("keeps paging available when a row action interrupts an in-flight page load", async () => {
    let listCalls = 0;
    stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications": () => {
        listCalls += 1;
        if (listCalls === 1) return json({ items: [approvalRow], next_cursor: "c1" });
        return new Promise<Response>(() => undefined); // interrupted page load never settles
      },
      [`POST /api/v1/me/notifications/${approvalRow.id}/read`]: () =>
        json({ ...approvalRow, unread: false, read_at: "2026-07-24T09:05:00Z" }),
    });
    renderScreen();
    const item = (await screen.findByRole("button", { name: approvalRow.text })).closest("li");
    if (!item) throw new Error("row list item missing");
    await userEvent.click(screen.getByRole("button", { name: text.loadMore }));
    await userEvent.click(within(item).getByRole("button", { name: text.markRead }));
    expect(await within(item).findByRole("button", { name: text.markUnread })).toBeVisible();
    expect(screen.getByRole("button", { name: text.loadMore })).toBeVisible();
  });

  it("drills a group to its object-filtered timeline and clears the filter", async () => {
    stubFetch({
      ...happyHandlers(),
      "GET /api/v1/me/notifications/by-object": () => json({ items: [approvalGroup], next_cursor: null }),
      "GET /api/v1/me/notification-policies": () => json({ items: [] }),
    });
    renderScreen();
    await screen.findByRole("button", { name: salesRow.text });
    await userEvent.click(screen.getByRole("button", { name: text.viewByObject }));
    await screen.findByText("2/3");
    // Group overlay carries the latest text; activate the group's primary control.
    await userEvent.click(screen.getByRole("button", { name: approvalRow.text }));
    expect(screen.queryByRole("button", { name: salesRow.text })).toBeNull();
    const clear = screen.getByRole("button", { name: text.objectFilterClear });
    expect(clear).toHaveTextContent("AP-3121");
    await userEvent.click(clear);
    expect(await screen.findByRole("button", { name: salesRow.text })).toBeVisible();
  });
});
