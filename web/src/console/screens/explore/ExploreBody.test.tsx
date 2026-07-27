import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { StrictMode, type ReactNode } from "react";
import { renderToString } from "react-dom/server";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { ConsoleApiClient } from "../../../api/client";
import { ko } from "../../../i18n/ko";
import {
  detailFixture,
  graphFixture,
  instanceFixture,
  summaryFixture,
} from "../../../test/ontologyFixtures";
import { renderWithWindowManager } from "../../testing";
import { ExploreBody } from "./ExploreBody";

vi.mock("../../../api/ontology");
// Only BulkPolicyGateProvider + PolicyGated are exercised here; stub both as
// open passthroughs so the body renders without the auth/bulk-authorize round-trip.
vi.mock("../../policy", () => {
  const passthrough = ({ children }: { children: ReactNode }) => <>{children}</>;
  // usePolicyGate is exercised by the docked ObjectCard inspector the graph pane
  // now renders; open it so the card mounts without the bulk-authorize round-trip.
  return {
    BulkPolicyGateProvider: passthrough,
    PolicyGated: passthrough,
    usePolicyGate: () => ({ can: () => true }),
  };
});

import * as ont from "../../../api/ontology";

const api = {} as ConsoleApiClient;
const mocked = vi.mocked(ont);

function seedRegistry(): void {
  mocked.listObjectTypes.mockResolvedValue([summaryFixture]);
  mocked.getObjectType.mockResolvedValue(detailFixture);
  mocked.listInstances.mockResolvedValue([instanceFixture]);
  mocked.traverseInstance.mockResolvedValue(graphFixture);
  mocked.getInstance.mockResolvedValue(instanceFixture);
  mocked.getInstanceHistory.mockResolvedValue([]);
  // Supplementary acting read: reload .catch-degrades it, but the auto-mock
  // returns undefined which throws on .catch — seed a resolved empty list.
  mocked.getObjectTypeActing.mockResolvedValue([]);
  // The governed graph card reads the instance dynamic layer directly.
  mocked.getInstanceActing.mockResolvedValue([]);
}

afterEach(() => {
  vi.clearAllMocks();
});

describe("ExploreBody", () => {
  beforeEach(() => {
    seedRegistry();
  });

  it("shows the loading state before the registry resolves", () => {
    render(<ExploreBody api={api} />);
    expect(screen.getByRole("status")).toHaveTextContent(ko.page.loading);
  });

  it("renders the graph and a drillable stat strip once loaded", async () => {
    render(<ExploreBody api={api} />);
    // The focus node title appears in the graph.
    expect((await screen.findAllByText("NK보안 경비용역")).length).toBeGreaterThan(0);

    // Stat strip: three drillable stats (타입 / 인스턴스 / 관계) — each a button.
    const strip = screen.getByRole("group", { name: ko.console.explore.title });
    const typeStat = within(strip).getByRole("button", {
      name: `${ko.console.ontology.typeList.title} ${ko.console.ontology.count(1)}`,
    });
    expect(typeStat).toHaveTextContent("1");
    expect(
      within(strip).getByRole("button", {
        name: `${ko.console.ontology.subtabs.links} ${ko.console.ontology.count(1)}`,
      }),
    ).toBeInTheDocument();

    // The explorer is read-only: no 타입·매니저 authoring tablist.
    expect(screen.queryByRole("tablist")).not.toBeInTheDocument();
  });

  it("runs the graph card through server preflight, executes with exact ids, and reads back history", async () => {
    const gates = {
      gates: [
        { gate: "authority", status: { status: "satisfied" } },
        { gate: "self_checklist", status: { status: "not_required" } },
        { gate: "four_eyes", status: { status: "not_required" } },
        { gate: "egress_dlp", status: { status: "not_required" } },
      ],
      allow: true,
    };
    const post = vi.fn((path: string) => ({
        data: path.endsWith("/preflight")
          ? { gates, criteria_ok: true, would_execute: true }
          : { instance: instanceFixture, gates },
        response: { status: 200 },
      }));
    const get = vi.fn(() => ({ data: readback, response: { status: 200 } }));
    const actionApi = {
      POST: post,
      GET: get,
    } as unknown as ConsoleApiClient;
    const readback = [
      {
        id: "revision-2",
        instance_id: instanceFixture.instance.id,
        version: 2,
        attributes: {},
        valid_from: "2026-07-23T12:00:00Z",
        valid_to: null,
        action_type_id: detailFixture.actions[0].id,
        actor: "user-1",
        reason: "graph readback",
        prev_hash: "0".repeat(64),
        row_hash: "1".repeat(64),
      },
    ];
    render(<ExploreBody api={actionApi} authorityKey="tenant-1" />);

    await screen.findByRole("button", {
      name: ko.console.objectcard.actionAria(detailFixture.actions[0].title),
    });
    fireEvent.click(
      screen.getByRole("button", {
        name: ko.console.objectcard.actionAria(detailFixture.actions[0].title),
      }),
    );
    fireEvent.click(
      await screen.findByRole("button", {
        name: ko.console.objectcardGov.preflight.execute,
      }),
    );

    await waitFor(() => {
      expect(post).toHaveBeenCalledTimes(2);
      expect(get).toHaveBeenCalledWith(
        "/api/v1/ontology/instances/{id}/history",
        {
          params: { path: { id: instanceFixture.instance.id } },
        },
      );
    });
    const [, preflightOptions] = post.mock.calls[0] ?? [];
    expect(preflightOptions).toMatchObject({
      body: {
        object_type_id: detailFixture.object_type.id,
        instance_id: instanceFixture.instance.id,
      },
    });
    expect(
      await screen.findByText(
        ko.console.objectcardGov.executedToast(
          detailFixture.actions[0].title,
          instanceFixture.revision.version,
          "AAAAAAAA",
        ),
      ),
    ).toBeInTheDocument();
    expect(screen.getAllByText(ko.console.objectcard.lifecycle.active).length).toBeGreaterThan(0);
    expect(await screen.findByText("graph readback")).toBeInTheDocument();
  });

  it("drills from a stat without crashing (graph stays mounted)", async () => {
    render(<ExploreBody api={api} />);
    const nodes = await screen.findAllByText("NK보안 경비용역");
    const strip = screen.getByRole("group", { name: ko.console.explore.title });
    fireEvent.click(
      within(strip).getByRole("button", {
        name: `${ko.console.ontology.typeList.title} ${ko.console.ontology.count(1)}`,
      }),
    );
    expect(nodes[0]).toBeInTheDocument();
  });

  it("opens governed analytics and drills the exact returned instance into the graph", async () => {
    render(
      <ExploreBody api={api} authorityKey="tenant-a:incarnation-a" />,
    );
    await screen.findAllByText(instanceFixture.instance.title);
    fireEvent.click(screen.getByRole("button", { name: ko.console.ontology.analysis.open }));

    const groupButton = await screen.findByRole("button", {
      name: ko.console.ontology.analysis.openGroup(
        instanceFixture.instance.lifecycle_state,
        1,
      ),
    });
    fireEvent.click(groupButton);

    expect(
      screen.getByText(ko.console.ontology.analysis.result.title),
    ).toBeInTheDocument();
    const instanceCode = instanceFixture.instance.id
      .replaceAll("-", "")
      .slice(0, 8)
      .toUpperCase();
    const instanceLabel = `${instanceCode} · ${instanceFixture.instance.title}`;
    fireEvent.click(
      screen.getByRole("button", {
        name: ko.console.ontology.analysis.result.openInstance(instanceLabel),
      }),
    );
    await waitFor(() => {
      expect(mocked.traverseInstance).toHaveBeenCalledWith(
        api,
        instanceFixture.instance.id,
      );
    });
  });

  it("hides analytics drill state synchronously when authority changes", async () => {
    const view = render(
      <ExploreBody api={api} authorityKey="tenant-a:incarnation-a" />,
    );
    await screen.findAllByText(instanceFixture.instance.title);
    fireEvent.click(screen.getByRole("button", { name: ko.console.ontology.analysis.open }));
    fireEvent.click(
      await screen.findByRole("button", {
        name: ko.console.ontology.analysis.openGroup(
          instanceFixture.instance.lifecycle_state,
          1,
        ),
      }),
    );
    expect(
      screen.getByText(ko.console.ontology.analysis.result.title),
    ).toBeInTheDocument();

    view.rerender(
      <ExploreBody api={api} authorityKey="tenant-b:incarnation-b" />,
    );
    expect(
      screen.queryByText(ko.console.ontology.analysis.result.title),
    ).not.toBeInTheDocument();
  });

  it("renders the honest empty state when the registry is empty", async () => {
    mocked.listObjectTypes.mockResolvedValue([]);
    render(<ExploreBody api={api} />);
    expect(await screen.findByText(ko.console.explore.labels.empty)).toBeInTheDocument();
  });

  it("renders the error state with retry when the read fails", async () => {
    mocked.listObjectTypes.mockRejectedValue(new Error("boom"));
    render(<ExploreBody api={api} />);
    expect(await screen.findByText(ko.page.loadFailed)).toBeInTheDocument();
    const retry = screen.getByRole("button", { name: ko.page.retry });

    // Retry re-reads: point the mock at data and click.
    seedRegistry();
    fireEvent.click(retry);
    expect((await screen.findAllByText("NK보안 경비용역")).length).toBeGreaterThan(0);
  });
  it("has no render-time API identity inference and mounts no window host of its own", () => {
    const source = readFileSync(
      "src/console/screens/_ontology/OntologyWorkspaceBody.tsx",
      "utf8",
    );
    expect(source).not.toMatch(
      /readOnlyApiAuthorityIds|nextReadOnlyApiAuthorityId/,
    );
    expect(source).not.toMatch(/new WeakMap|api as object/);
    // The window host is the console shell's, mounted once above every screen
    // body (see console/window/consoleShellWindowHost.test.tsx). A provider here
    // forked the arrangement into two trays and two layout partitions.
    expect(source).not.toContain("WindowManagerProvider");
    expect(() =>
      renderToString(
        <ExploreBody api={api} authorityKey="tenant-a:incarnation-a" />,
      ),
    ).not.toThrow();
  });

  it("adds no second window host inside the shell's, in either StrictMode root", async () => {
    const first = renderWithWindowManager(
      <StrictMode>
        <ExploreBody api={api} authorityKey="tenant-a:incarnation-a" />
      </StrictMode>,
      { authorityPartition: "tenant-a:incarnation-a" },
    );
    const second = renderWithWindowManager(
      <StrictMode>
        <ExploreBody api={api} authorityKey="tenant-a:incarnation-b" />
      </StrictMode>,
      { authorityPartition: "tenant-a:incarnation-b" },
    );

    // One host per root — the wrapper's — and nothing nested underneath it.
    await waitFor(() => {
      expect(
        first.container.querySelectorAll("[data-window-host]"),
      ).toHaveLength(1);
      expect(
        second.container.querySelectorAll("[data-window-host]"),
      ).toHaveLength(1);
    });
    await screen.findAllByText(instanceFixture.instance.title);
    expect(first.container.querySelectorAll("[data-window-host]")).toHaveLength(1);
    expect(second.container.querySelectorAll("[data-window-host]")).toHaveLength(1);

    first.unmount();
    second.unmount();
  });
});
