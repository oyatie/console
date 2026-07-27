// Test-only canvas fixtures. Production consumers supply governed registries,
// samples, and persisted documents through their own adapters.

import type { CanvasDoc, FieldRegistry, SampleRow } from "./types";

/** Reference field registry (mirrors ont_property_defs {key,title,type,config}). */
export const TEST_FIELD_REGISTRY: FieldRegistry = [
  { key: "absence_count", label: "absence_count", type: "number" },
  { key: "days_open", label: "days_open", type: "number" },
  { key: "opened_at", label: "opened_at", type: "date" },
  { key: "is_active", label: "is_active", type: "bool" },
  {
    key: "priority",
    label: "priority",
    type: "enum",
    choices: [
      { id: "low", name: "low" },
      { id: "med", name: "med" },
      { id: "high", name: "high" },
    ],
  },
  { key: "work_order", label: "work_order", type: "code" },
];

/** Seed sample object-set the simulation panel evaluates over. */
export const TEST_SAMPLES: readonly SampleRow[] = [
  { absence_count: 3, days_open: 2, opened_at: "2026-07-01", is_active: true, priority: "high", work_order: "WO-2643" },
  { absence_count: 1, days_open: 9, opened_at: "2026-06-20", is_active: true, priority: "med", work_order: "WO-2650" },
  { absence_count: 4, days_open: 1, opened_at: "2026-07-08", is_active: false, priority: "low", work_order: "WO-2661" },
];

/** Reference Trigger→Condition→Branch→Action document for tests. */
export function testCanvasDoc(): CanvasDoc {
  return {
    version: 1,
    nodes: [
      { id: "n-trigger", kind: "trigger", title: "trigger", x: 40, y: 32 },
      { id: "n-condition", kind: "condition", title: "condition", x: 40, y: 208 },
      {
        id: "n-branch",
        kind: "branch",
        title: "branch",
        x: 40,
        y: 384,
        outputs: [
          { port: "yes", label: "yes" },
          { port: "no", label: "no" },
        ],
      },
      { id: "n-action", kind: "action", title: "action", x: 360, y: 384 },
    ],
    edges: [
      { id: "e-trigger-condition", from: "n-trigger", to: "n-condition" },
      { id: "e-condition-branch", from: "n-condition", to: "n-branch" },
      { id: "e-branch-action", from: "n-branch", fromPort: "yes", to: "n-action" },
    ],
    vars: [{ key: "absence_count", type: "number", label: "absence_count" }],
  };
}
