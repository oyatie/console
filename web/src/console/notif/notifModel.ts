import { consoleScreenPath, isExposedScreenKey } from "../shell/nav";
import type { NotificationLink } from "./notifApi";

/** Stable identity for link equality/grouping (mirrors the backend link-equality sweep). */
export function linkKey(link: NotificationLink): string {
  return link.type === "object" ? `object:${link.kind}:${link.id}` : `screen:${link.screen}`;
}

export function sameLink(a: NotificationLink, b: NotificationLink): boolean {
  return linkKey(a) === linkKey(b);
}

export type RowTarget =
  | { type: "screen"; path: string }
  | { type: "object"; kind: string; id: string };

/**
 * Deep-link chain (§4-19 single chokepoint): an object link drills to the
 * source object; a screen link navigates only when the screen is in the
 * evidence-approved exposure manifest (ADR-0025). An unexposed screen yields
 * `undefined` — the row stays ack-able text, never a dead control.
 */
export function rowTarget(link: NotificationLink): RowTarget | undefined {
  if (link.type === "object") return { type: "object", kind: link.kind, id: link.id };
  return isExposedScreenKey(link.screen)
    ? { type: "screen", path: consoleScreenPath(link.screen) }
    : undefined;
}

const TIME_FORMAT = new Intl.DateTimeFormat("ko-KR", {
  month: "numeric",
  day: "numeric",
  hour: "2-digit",
  minute: "2-digit",
});

/** Compact mono timestamp for the row trailing slot; raw value if unparsable. */
export function timeLabel(iso: string): string {
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? iso : TIME_FORMAT.format(date);
}
