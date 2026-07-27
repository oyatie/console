import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { ko } from "../../../i18n/ko";
import type { ConsoleApiClient } from "../../../api/client";
import type { ObjectExplorerModel, ObjectExplorerNode } from "../../explore";
import type { ObjectCardDescriptor } from "../../objectcard";
import type { ObjectRuntimePort } from "../../runtime/objectRuntime";
import { GraphExplorer } from "./GraphExplorer";

// The docked inspector renders a real ObjectCard; open the policy gate so it
// mounts without the bulk-authorize round-trip.
vi.mock("../../policy", () => {
  const passthrough = ({ children }: { children: ReactNode }) => <>{children}</>;
  return {
    PolicyGated: passthrough,
    usePolicyGate: () => ({ can: () => true }),
  };
});

const G = ko.console.explore.graph;
type RuntimeResolve = ObjectRuntimePort["resolve"];
type RuntimeReference = Parameters<RuntimeResolve>[0];

const model: ObjectExplorerModel = {
  nodes: [
    { id: "n1", type: "계약", type_id: "t1", code: "C-207", label: "NK보안 경비용역", lifecycle: { phase: "active" } },
    { id: "n2", type: "구매", type_id: "t2", code: "PO-119", label: "경비 근무 장구", lifecycle: { phase: "active" } },
  ],
  object_links: [{ id: "e1", source_id: "n1", target_id: "n2", relation: "공급" }],
};

function resolvedDescriptor(node: ObjectExplorerNode): ObjectCardDescriptor {
  return {
    id: node.id,
    code: node.code,
    title: node.label,
    objectType: { key: node.type_id ?? node.type, title: node.type },
    lifecycleState: "active",
    properties: [{ key: "amount", title: "월 계약금", type: "money", value: "₩1,860,000" }],
    relations: [],
    lifecycle: [{ state: "active", reached: true, current: true }],
    history: [],
    actions: [{ key: "renew", title: "갱신 검토 기안" }],
  };
}


function runtimeFor(
  resolve: (reference: Parameters<ObjectRuntimePort["resolve"]>[0], options: Parameters<ObjectRuntimePort["resolve"]>[1]) => Promise<ObjectCardDescriptor | undefined>,
): ObjectRuntimePort {
  return { authority: "ontology", resolve };
}

function inspectorApi(): ConsoleApiClient {
  return {
    GET: vi.fn(() =>
      Promise.resolve({
        data: [],
        error: undefined,
        response: { status: 200 },
      }),
    ),
  } as unknown as ConsoleApiClient;
}

function governedProps(runtime: ObjectRuntimePort) {
  return {
    api: inspectorApi(),
    runtime,
    tenantScopeKey: "tenant-a",
    authorityKey: "authority-a",
  };
}

function resolvedRuntime(): ObjectRuntimePort {
  return runtimeFor((reference) => {
    const node = model.nodes.find((candidate) => candidate.id === reference.id) ?? model.nodes[0];
    return Promise.resolve(resolvedDescriptor(node));
  });
}

describe("GraphExplorer", () => {
  it("renders typed nodes, relation-labelled edges and a legend by type", () => {
    render(<GraphExplorer model={model} />);
    // Typed node code chips (C-207 also appears in the docked focus-node card).
    expect(screen.getAllByText("C-207").length).toBeGreaterThan(0);
    expect(screen.getByText("PO-119")).toBeInTheDocument();
    // Edge carries its relation label.
    expect(screen.getByText("공급")).toBeInTheDocument();
    // Legend counts the two distinct types.
    const legend = screen.getByRole("group", { name: G.legend });
    expect(within(legend).getByText(G.legendCount(2))).toBeInTheDocument();
  });

  it("zooms in from the 100% baseline", () => {
    render(<GraphExplorer model={model} />);
    expect(screen.getByText(G.zoomLevel(100))).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: G.zoomIn }));
    expect(screen.getByText(G.zoomLevel(110))).toBeInTheDocument();
  });

  it("does not restart governed object reads while zooming or panning the same entity", async () => {
    const resolve = vi.fn((reference: RuntimeReference) => {
      const node =
        model.nodes.find((candidate) => candidate.id === reference.id) ??
        model.nodes[0];
      return Promise.resolve(resolvedDescriptor(node));
    });
    const view = render(
      <GraphExplorer model={model} {...governedProps(runtimeFor(resolve))} />,
    );
    await waitFor(() => {
      expect(resolve).toHaveBeenCalledTimes(1);
    });

    fireEvent.click(screen.getByRole("button", { name: G.zoomIn }));
    const pane = screen.getByLabelText(G.pane);
    const panSurface = pane.querySelector<HTMLDivElement>(
      'div[aria-hidden="true"]',
    );
    if (!panSurface) throw new Error("expected graph pan surface");
    fireEvent.pointerDown(panSurface, { clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 40, clientY: 25 });
    fireEvent.pointerUp(window);

    await waitFor(() => {
      expect(screen.getByText(G.zoomLevel(110))).toBeInTheDocument();
      expect(resolve).toHaveBeenCalledTimes(1);
    });
    view.unmount();
  });

  it("stacks the graph and contextual rail below the narrow viewport breakpoint", () => {
    const originalWidth = window.innerWidth;
    Object.defineProperty(window, "innerWidth", { configurable: true, value: 800 });
    render(<GraphExplorer model={model} />);

    expect(screen.getByLabelText(G.pane)).toHaveStyle({
      gridTemplateColumns: "minmax(0, 1fr)",
    });
    Object.defineProperty(window, "innerWidth", { configurable: true, value: originalWidth });
  });

  it("resolves the focus-node inspector card on mount, before any click", async () => {
    const resolve = vi.fn((reference: RuntimeReference) => {
      const node = model.nodes.find((candidate) => candidate.id === reference.id) ?? model.nodes[0];
      return Promise.resolve(resolvedDescriptor(node));
    });
    render(<GraphExplorer model={model} {...governedProps(runtimeFor(resolve))} />);
    // The docked inspector opens with the focus node's real card — no click, so
    // the relation/property rows are never falsely empty (관계 0개) on landing.
    await waitFor(() => {
      expect(screen.getByText("월 계약금")).toBeInTheDocument();
    });
    expect(resolve.mock.calls.some(([reference]) => reference.id === "n1" && reference.tenantScopeKey === "tenant-a" && reference.authorityKey === "authority-a")).toBe(true);
  });

  it("resolves a second node's card when it is activated", async () => {
    const resolve = vi.fn((reference: RuntimeReference) => {
      const node = model.nodes.find((candidate) => candidate.id === reference.id) ?? model.nodes[0];
      return Promise.resolve(resolvedDescriptor(node));
    });
    render(<GraphExplorer model={model} {...governedProps(runtimeFor(resolve))} />);
    await waitFor(() => {
      expect(resolve.mock.calls.some(([reference]) => reference.id === "n1" && reference.tenantScopeKey === "tenant-a" && reference.authorityKey === "authority-a")).toBe(true);
    });
    fireEvent.click(screen.getByRole("button", { name: ko.console.explore.actions.recenter("경비 근무 장구") }));
    await waitFor(() => {
      expect(resolve.mock.calls.some(([reference]) => reference.id === "n2")).toBe(true);
    });
  });

  it("focuses and resolves an exact host-requested instance once it is in the graph", async () => {
    const resolve = vi.fn((reference: RuntimeReference) => {
      const node = model.nodes.find((candidate) => candidate.id === reference.id) ?? model.nodes[0];
      return Promise.resolve(resolvedDescriptor(node));
    });
    const view = render(
      <GraphExplorer
        model={{ ...model, nodes: [model.nodes[0]] }}
        requestedFocusId="n2"
        {...governedProps(runtimeFor(resolve))}
      />,
    );
    expect(
      screen.queryByRole("button", {
        name: ko.console.explore.actions.recenter("경비 근무 장구"),
      }),
    ).not.toBeInTheDocument();

    view.rerender(
      <GraphExplorer
        model={model}
        requestedFocusId="n2"
        {...governedProps(runtimeFor(resolve))}
      />,
    );

    await waitFor(() => {
      expect(resolve.mock.calls.some(([reference]) => reference.id === "n2")).toBe(true);
      expect(
        screen.getByRole("complementary", {
          name: ko.console.objectcard.panel("경비 근무 장구"),
        }),
      ).toBeInTheDocument();
    });
  });

  it("ignores authority cancellation and permits the current resolver to retry", async () => {
    const cancelled = vi.fn<RuntimeResolve>().mockResolvedValue(undefined);
    const view = render(<GraphExplorer model={model} {...governedProps(runtimeFor(cancelled))} />);
    await waitFor(() => {
      expect(cancelled.mock.calls.some(([reference]) => reference.id === "n1")).toBe(true);
    });

    const current = vi.fn((reference: RuntimeReference) => {
      const node = model.nodes.find((candidate) => candidate.id === reference.id) ?? model.nodes[0];
      return Promise.resolve(resolvedDescriptor(node));
    });
    view.rerender(<GraphExplorer model={model} {...governedProps(runtimeFor(current))} />);
    await waitFor(() => {
      expect(current.mock.calls.some(([reference]) => reference.id === "n1")).toBe(true);
      expect(screen.getByText("월 계약금")).toBeInTheDocument();
    });
  });

  it("shows the honest 조회 전용 state for a projected node and never resolves it", () => {
    const resolve = vi.fn((reference: RuntimeReference) => {
      const node = model.nodes.find((candidate) => candidate.id === reference.id) ?? model.nodes[0];
      return Promise.resolve(resolvedDescriptor(node));
    });
    render(
      <GraphExplorer
        model={model}
        {...governedProps(runtimeFor(resolve))}
        projectedTypeIds={new Set(["t1"])}
      />,
    );
    // Focus node (t1) is projected → the notice shows without a resolve attempt.
    expect(screen.getByText(G.projectedNotice)).toBeInTheDocument();
    expect(resolve).not.toHaveBeenCalled();
  });

  it("moves graph focus with arrow keys and exposes the same typed relations as a list", async () => {
    const onFocusChange = vi.fn();
    render(<GraphExplorer model={model} onFocusChange={onFocusChange} />);

    const first = screen.getByRole("button", {
      name: ko.console.explore.actions.recenter("NK보안 경비용역"),
    });
    first.focus();
    fireEvent.keyDown(first, { key: "ArrowRight" });

    await waitFor(() => {
      expect(onFocusChange).toHaveBeenCalledWith("n2");
      expect(document.activeElement).toHaveAttribute(
        "aria-label",
        ko.console.explore.actions.recenter("경비 근무 장구"),
      );
    });
    expect(
      screen.getByRole("list", { name: G.relationList }),
    ).toHaveTextContent("NK보안 경비용역 공급 경비 근무 장구");
  });
});

describe("GraphExplorer runtime authority fencing", () => {
  it("aborts a superseded authority read and never renders the retired descriptor", async () => {
    let resolveA!: (value: ObjectCardDescriptor | undefined) => void;
    const pendingA = new Promise<ObjectCardDescriptor | undefined>((resolve) => {
      resolveA = resolve;
    });
    const runtimeA = runtimeFor(() => pendingA);
    const runtimeB = resolvedRuntime();
    const view = render(<GraphExplorer model={model} {...governedProps(runtimeA)} />);

    view.rerender(
      <GraphExplorer
        model={model}
        api={inspectorApi()}
        runtime={runtimeB}
        tenantScopeKey="tenant-b"
        authorityKey="authority-b"
      />,
    );
    resolveA({ ...resolvedDescriptor(model.nodes[0]), title: "A secret object" });

    await waitFor(() => {
      expect(screen.getByText("NK보안 경비용역")).toBeInTheDocument();
    });
    expect(screen.queryByText("A secret object")).not.toBeInTheDocument();
  });

  it("does not render a card or object-count surface when the scoped runtime omits a denied reference", async () => {
    const denied = runtimeFor(() => Promise.resolve(undefined));
    render(<GraphExplorer model={model} {...governedProps(denied)} />);
    await waitFor(() => {
      expect(screen.queryByText("Loading object")).not.toBeInTheDocument();
    });
    expect(screen.queryByRole("heading", { name: "NK보안 경비용역" })).not.toBeInTheDocument();
  });
});
