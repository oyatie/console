import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { SimulationPanel } from "./SimulationPanel";
import { DEFAULT_CANVAS_STRINGS } from "./strings";
import { TEST_FIELD_REGISTRY, TEST_SAMPLES } from "./testFixtures";
import type { PredicateGroup } from "./types";

const S = DEFAULT_CANVAS_STRINGS;

describe("SimulationPanel", () => {
  it("runs a real eval over the seed samples and shows pass/total", () => {
    const group: PredicateGroup = {
      join: "and",
      predicates: [{ id: "r1", field: "absence_count", op: "gte", value: { kind: "number", value: 3 } }],
    };
    render(<SimulationPanel group={group} registry={TEST_FIELD_REGISTRY} strings={S} samples={TEST_SAMPLES} />);

    // No result until the action runs (not a decorative always-on toast).
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText(S.runSimulation));
    // absence_count 3,1,4 ≥ 3 → 2 of 3 pass.
    expect(screen.getByRole("status")).toHaveTextContent(S.simulationResult(2, 3));
  });
});
