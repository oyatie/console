import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const captured = vi.hoisted(() => ({ props: undefined as Record<string, unknown> | undefined }));

vi.mock("../comms-rail", () => ({
  CommsRailContainer: (props: Record<string, unknown>) => {
    captured.props = props;
    return <div data-testid="authenticated-comms-rail" />;
  },
}));

import { CommsRailPanel } from "./CommsRailPanel";

describe("CommsRailPanel", () => {
  it("mounts the authenticated production rail inside the existing shell chrome", () => {
    const onOpenMessengerThread = vi.fn();
    const onOpenMailThread = vi.fn();
    render(<CommsRailPanel accessToken="legacy-shell-token" onOpenMessengerThread={onOpenMessengerThread} onOpenMailThread={onOpenMailThread} />);

    expect(screen.getByTestId("authenticated-comms-rail")).toBeInTheDocument();
    expect(captured.props).toMatchObject({ embedded: true, onOpenMessengerThread });
    expect(captured.props?.onOpenMailThread).toEqual(expect.any(Function));
    expect(captured.props?.copy).toEqual(expect.objectContaining({
      landmark: "커뮤니케이션",
      source: expect.objectContaining({ messenger: "메신저", mail: "메일", notifications: "알림", notices: "공지" }),
    }));
  });

  it("does not invent mail, notice, or full-screen handlers when the shell has not supplied them", () => {
    render(<CommsRailPanel />);

    expect(captured.props).not.toHaveProperty("onOpenMailThread");
    expect(captured.props).not.toHaveProperty("onOpenNotice");
    expect(captured.props).not.toHaveProperty("onOpenFullModule");
  });

  it("passes only canonical notification and notice-list promotions supplied by the shell", () => {
    const onOpenNotificationCenter = vi.fn();
    const onOpenNoticeBoard = vi.fn();
    render(<CommsRailPanel onOpenNotificationCenter={onOpenNotificationCenter} onOpenNoticeBoard={onOpenNoticeBoard} />);

    expect(captured.props?.onViewAll).toEqual(expect.objectContaining({
      notifications: onOpenNotificationCenter,
      notices: onOpenNoticeBoard,
    }));
    expect(captured.props?.onOpenFullModule).toEqual(expect.any(Function));
  });
});
