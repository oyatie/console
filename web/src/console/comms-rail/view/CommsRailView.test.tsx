import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import type { CommsRailItem, CommsRailLoadState, CommsRailSnapshot, CommsRailSource } from "../model";
import { CommsRailView, type CommsRailCopy, type CommsRailViewProps } from "./CommsRailView";

const copy: CommsRailCopy = {
  landmark: "Communications",
  drawerTitle: "Communications drawer",
  close: "Close communications",
  open: "Open communications",
  source: { messenger: "Messenger", mail: "Mail", notifications: "Notifications", notices: "Notices" },
  state: {
    loading: "Loading", empty: "Nothing here", denied: "Access denied", malformed: "Invalid response",
    error: "Unavailable", retry: "Retry", retrying: "Retrying",
  },
  viewAll: (source) => `View all ${source}`,
  unread: (count) => `${String(count)} unread`,
  collapse: (source) => `Collapse ${source}`,
  expand: (source) => `Expand ${source}`,
  detail: "Detail",
  occurredAt: (iso) => `At ${iso}`,
};

const items: Record<CommsRailSource, CommsRailItem> = {
  messenger: {
    id: "thread-1", source: "messenger", occurredAt: "2026-07-22T10:00:00+09:00", code: "thread",
    unread: true, target: { kind: "inline", source: "messenger", id: "thread-1" }, action: { kind: "mark-messenger-read", threadId: "thread-1", lastMessageId: "message-1" },
    title: "Operations", branchId: "branch-1", visibility: "channel", unreadCount: 2, muted: false, memberCount: 4,
  },
  mail: {
    id: "mail-1", source: "mail", occurredAt: "2026-07-22T10:00:00+09:00", code: "mail",
    unread: true, target: { kind: "inline", source: "mail", id: "mail-1" }, action: { kind: "mark-mail-read", threadId: "mail-1" },
    subject: "Invoice review", unreadCount: 1, messageCount: 1, hasAttachments: true, flagged: false,
  },
  notifications: {
    id: "notification-1", source: "notifications", occurredAt: "2026-07-22T10:00:00+09:00", code: "notification",
    unread: true, target: { kind: "full-screen", source: "notifications", id: "notification-1", route: "/console/overview" }, action: { kind: "mark-notification-read", notificationId: "notification-1" },
    category: "approval", notificationKind: "approval_requested", text: "Approval requested",
  },
  notices: {
    id: "notice-1", source: "notices", occurredAt: "2026-07-22T10:00:00+09:00", code: "notice",
    unread: true, target: { kind: "inline", source: "notices", id: "notice-1" },
    title: "Policy notice", status: "published", acknowledged: false,
  },
};

function snapshot(state: CommsRailLoadState = { kind: "ready", items: Object.values(items), loadedAt: "2026-07-22T10:00:00+09:00" }): CommsRailSnapshot {
  return {
    messenger: state.kind === "ready" ? { kind: "ready", items: [items.messenger], loadedAt: state.loadedAt } : state,
    mail: state.kind === "ready" ? { kind: "ready", items: [items.mail], loadedAt: state.loadedAt } : state,
    notifications: state.kind === "ready" ? { kind: "ready", items: [items.notifications], loadedAt: state.loadedAt } : state,
    notices: state.kind === "ready" ? { kind: "ready", items: [items.notices], loadedAt: state.loadedAt } : state,
  };
}

type PersistentProps = Extract<CommsRailViewProps, { presentation?: "persistent" }>;

function view(props: Omit<Partial<PersistentProps>, "snapshot" | "copy"> = {}) {
  return <CommsRailView presentation="persistent" snapshot={snapshot()} copy={copy} {...props} />;
}

describe("CommsRailView", () => {
  it.each<CommsRailLoadState>([
    { kind: "loading" }, { kind: "empty" }, { kind: "denied", status: 403 },
    { kind: "malformed", code: "malformed_response" }, { kind: "error", code: "server_error" },
  ])("renders the %s source state for all four categories", (state) => {
    const { container } = render(<CommsRailView snapshot={snapshot(state)} copy={copy} onRetry={vi.fn()} />);
    expect(container.querySelectorAll(`[data-comms-state="${state.kind}"]`)).toHaveLength(4);
    if (state.kind === "error") expect(screen.getAllByRole("button", { name: "Retry" })).toHaveLength(4);
  });

  it("renders retry as an explicit source state and issues a source-scoped retry", async () => {
    const user = userEvent.setup();
    const onRetry = vi.fn();
    render(<CommsRailView snapshot={snapshot({ kind: "error", code: "network_error" })} copy={copy} retryingSource="mail" onRetry={onRetry} />);
    expect(screen.getAllByText("Retrying")).toHaveLength(1);
    await user.click(screen.getAllByRole("button", { name: "Retry" })[0]);
    expect(onRetry).toHaveBeenCalledWith("messenger");
  });

  it("supports independently collapsible categories and status text beyond color", async () => {
    const user = userEvent.setup();
    render(view());
    await user.click(screen.getByRole("button", { name: "Collapse Mail" }));
    expect(screen.getByRole("button", { name: "Expand Mail" })).toHaveAttribute("aria-expanded", "false");
    expect(screen.getByLabelText("2 unread")).toBeInTheDocument();
  });

  it("omits compose and typed-row actions without real handlers or targets", () => {
    const withoutTarget: CommsRailSnapshot = {
      ...snapshot(),
      mail: { kind: "ready", items: [{ ...items.mail, target: undefined, action: undefined }], loadedAt: "now" },
    };
    render(<CommsRailView snapshot={withoutTarget} copy={copy} />);
    expect(screen.queryByRole("button", { name: "Compose" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Invoice review/ })).not.toBeInTheDocument();
  });

  it("keeps target-only rows static until an actual drill handler or detail renderer exists", () => {
    render(view());
    expect(screen.queryByRole("button", { name: /Operations/ })).not.toBeInTheDocument();
  });

  it("opens authorized mail detail without declaring the workspace replaced", async () => {
    const user = userEvent.setup();
    const onOpenMailThread = vi.fn();
    const { container } = render(view({ onOpenMailThread, workspacePreservedId: "overview", renderInlineDetail: (item) => <p>{`Authoritative ${item.id}`}</p> }));
    await user.click(screen.getByRole("button", { name: /Invoice review/ }));
    expect(container.querySelector('[data-comms-detail="mail-1"]')).toBeInTheDocument();
    expect(screen.getByText("Authoritative mail-1")).toBeInTheDocument();
    expect(onOpenMailThread).toHaveBeenCalledWith(items.mail, items.mail.target);
    expect(container.querySelector('[data-comms-preserves-workspace="overview"]')).toBeInTheDocument();
  });

  it("uses native button keyboard activation for an authorized notice drill", async () => {
    const user = userEvent.setup();
    const onOpenNotice = vi.fn();
    render(view({ onOpenNotice }));
    const row = screen.getByRole("button", { name: /Policy notice/ });
    row.focus();
    await user.keyboard("{Enter}");
    expect(onOpenNotice).toHaveBeenCalledWith(items.notices, items.notices.target);
    expect(screen.queryByRole("region", { name: "Detail" })).not.toBeInTheDocument();
  });

  it("makes only rows with an exact source-and-target handler actionable", async () => {
    const user = userEvent.setup();
    const onOpenMessengerThread = vi.fn();
    render(view({ onOpenMessengerThread }));

    const messenger = screen.getByRole("button", { name: /Operations/ });
    expect(screen.queryByRole("button", { name: /Invoice review/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Approval requested/ })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Policy notice/ })).not.toBeInTheDocument();

    await user.click(messenger);
    expect(onOpenMessengerThread).toHaveBeenCalledWith("thread-1");
  });

  it("keeps a full-screen row static when the embedding shell rejects its route contract", () => {
    render(view({ onOpenFullModule: vi.fn(), canOpenFullModule: () => false }));
    expect(screen.queryByRole("button", { name: /Approval requested/ })).not.toBeInTheDocument();
  });

  it("offers source promotion only when the embedding shell supplied an exact destination", async () => {
    const user = userEvent.setup();
    const openNotifications = vi.fn();
    render(view({ onViewAll: { notifications: openNotifications } }));

    await user.click(screen.getByRole("button", { name: "View all Notifications" }));
    expect(openNotifications).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("button", { name: "View all Notices" })).not.toBeInTheDocument();
  });

  it("opens an explicitly typed notification-origin messenger thread", async () => {
    const user = userEvent.setup();
    const onOpenMessengerThread = vi.fn();
    const messengerMention: CommsRailSnapshot = {
      ...snapshot(),
      notifications: {
        kind: "ready",
        items: [{
          ...items.notifications,
          target: {
            kind: "messenger-thread",
            source: "notifications",
            id: "thread-2",
          },
        }],
        loadedAt: "now",
      },
    };

    render(
      <CommsRailView
        presentation="persistent"
        snapshot={messengerMention}
        copy={copy}
        onOpenMessengerThread={onOpenMessengerThread}
      />,
    );
    await user.click(screen.getByRole("button", { name: /Approval requested/ }));
    expect(onOpenMessengerThread).toHaveBeenCalledWith("thread-2");
  });

  it("fails closed when a mail row carries a forged messenger target", async () => {
    const user = userEvent.setup();
    const onOpenMessengerThread = vi.fn();
    const hostile: CommsRailSnapshot = {
      ...snapshot(),
      mail: {
        kind: "ready",
        items: [{ ...items.mail, target: { kind: "inline", source: "messenger", id: "thread-1" } }],
        loadedAt: "now",
      },
    };

    render(<CommsRailView presentation="persistent" snapshot={hostile} copy={copy} onOpenMessengerThread={onOpenMessengerThread} />);
    expect(screen.queryByRole("button", { name: /Invoice review/ })).not.toBeInTheDocument();
    await user.click(screen.getByText("Invoice review"));
    expect(onOpenMessengerThread).not.toHaveBeenCalled();
  });

  function ControlledDrawer({ trigger }: { trigger: HTMLButtonElement }) {
    const [open, setOpen] = useState(true);
    return <CommsRailView
      snapshot={snapshot()}
      copy={copy}
      presentation="drawer"
      drawerOpen={open}
      onRequestClose={() => { setOpen(false); }}
      returnFocusRef={{ current: trigger }}
    />;
  }

  it("renders a controlled accessible drawer, traps Tab boundaries, closes on Escape, and restores its trigger", async () => {
    const user = userEvent.setup();
    const trigger = document.createElement("button");
    trigger.textContent = "Open rail";
    document.body.append(trigger);
    trigger.focus();
    render(<ControlledDrawer trigger={trigger} />);
    const drawer = screen.getByRole("dialog", { name: "Communications drawer" });
    expect(drawer).toHaveAttribute("aria-modal", "true");
    const buttons = within(drawer).getAllByRole("button");
    const first = buttons[0];
    const last = buttons[buttons.length - 1];
    expect(first).toHaveFocus();
    last.focus();
    await user.keyboard("{Tab}");
    expect(first).toHaveFocus();
    first.focus();
    await user.keyboard("{Shift>}{Tab}{/Shift}");
    expect(last).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Communications drawer" })).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
    trigger.remove();
  });

  it("fails closed when a category receives a row for another source", () => {
    const invalid: CommsRailSnapshot = {
      ...snapshot(),
      messenger: { kind: "ready", items: [items.mail], loadedAt: "now" },
    };
    const { container } = render(<CommsRailView snapshot={invalid} copy={copy} />);
    const messenger = container.querySelector('[data-comms-source="messenger"]');
    expect(messenger?.querySelector('[data-comms-state="malformed"]')).toBeInTheDocument();
    expect(messenger).not.toHaveTextContent("Invoice review");
  });
});
