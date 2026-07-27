import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../api/client";
import { PolicyGateProvider, type PolicyGate } from "../policy";
import { GenericModuleScreen } from "./GenericModuleScreen";
import { CANONICAL_ONTOLOGY_LIST } from "./moduleScreens";
import type { ModuleScreenConfig } from "./types";

const allowGate: PolicyGate = { can: () => true };
const typeVersionId = "00000000-0000-4000-8000-000000000701";

const config: ModuleScreenConfig = {
  id: "widget",
  screen: "widget",
  route: "/modules?screen=widget",
  navLabelKey: "Widget",
  titleKey: "Widget",
  objectNameKey: "Widget",
  objectKind: "widget",
  typeKey: "widget",
  codePrefix: "WD-",
  emptyMode: "live",
  policy: { read: "object.view" },
  data: { list: CANONICAL_ONTOLOGY_LIST },
  statbar: [{ key: "instances", labelKey: "Instances", tone: "neutral", source: "canonical" }],
  search: { labelKey: "Search", placeholderKey: "Search", fields: [] },
  list: { keyboard: [], sharedTrack: "widgetTrack", columns: [{ key: "code" }] },
  detail: { fields: [], linkChips: [], actions: [] },
  rows: [],
};

function apiForCanonicalWidget(): ConsoleApiClient {
  const GET = vi.fn()
    .mockResolvedValueOnce({
      data: {
        object_type: {
          id: typeVersionId,
          stable_key: "widget",
          title: "Real Widget",
          backing_kind: "instance",
          schema_version: 7,
          lifecycle_state: "published",
          key_write_revision: 7,
          key_write_etag: '"widget-r7"',
        },
        title_property_key: "label",
        backing_table: null,
        primary_key_property: null,
        properties: [{ id: "p1", key: "label", title: "Governed label", field_type: "text", config: {}, backing_column: null, required: true, in_property_policy: false }],
        links: [], actions: [], analytics: [],
      },
      response: new Response(),
    })
    .mockResolvedValueOnce({
      data: [{
        instance: { id: "00000000-0000-4000-8000-000000000703", object_type_id: typeVersionId, title: "Real widget instance", current_revision_id: "r1", lifecycle_state: "active" },
        revision: { id: "r1", instance_id: "00000000-0000-4000-8000-000000000703", version: 1, attributes: { label: "Real governed value" }, valid_from: "2026-07-25T00:00:00Z", valid_to: null, action_type_id: null, actor: null, reason: null, prev_hash: "0".repeat(64), row_hash: "a".repeat(64) },
      }],
      response: new Response(),
    });
  return { GET } as unknown as ConsoleApiClient;
}

describe("GenericModuleScreen canonical dynamic type", () => {
  it("renders columns and values from real canonical ontology detail, not OT-KIND/code-title fallback", async () => {
    render(
      <PolicyGateProvider gate={allowGate}>
        <GenericModuleScreen api={apiForCanonicalWidget()} authorityKey="tenant-a:session-1" config={config} />
      </PolicyGateProvider>,
    );

    expect(await screen.findByRole("columnheader", { name: "Governed label" })).toBeVisible();
    expect(screen.getAllByText("Real governed value").length).toBeGreaterThan(0);
    expect(screen.queryByText("OT-WIDGET")).not.toBeInTheDocument();
  });

  it("clears previously governed rows, selection, and stats when a canonical refresh fails", async () => {
    const api = apiForCanonicalWidget();
    (api.GET as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      data: undefined,
      error: { error: { code: "DENIED", message: "denied" } },
      response: new Response(null, { status: 403 }),
    });
    render(
      <PolicyGateProvider gate={allowGate}>
        <GenericModuleScreen api={api} authorityKey="tenant-b:session-1" config={config} />
      </PolicyGateProvider>,
    );

    expect((await screen.findAllByText("Real governed value")).length).toBeGreaterThan(0);
    expect(screen.getByText("Instances 1")).toBeVisible();
    fireEvent.change(screen.getByRole("searchbox", { name: "Search" }), { target: { value: "real" } });

    expect(await screen.findByRole("alert")).toBeVisible();
    expect(screen.queryByText("Real governed value")).not.toBeInTheDocument();
    expect(screen.queryByText("Instances 1")).not.toBeInTheDocument();
    expect(screen.queryByText("40008000")).not.toBeInTheDocument();
  });
});
