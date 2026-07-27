import { render, type RenderResult } from "@testing-library/react";
import type { ReactNode } from "react";

import { WindowManagerProvider } from "../window";

/**
 * FROZEN for wave 4 — name and signature. Module lanes assert through this
 * helper instead of hand-rolling a `<WindowManagerProvider>` per test file.
 *
 * Hand-rolled providers are how the window model stayed green in jsdom for 15
 * module bodies while being absent from `ConsoleShell` in production: each test
 * supplied the host the shell never mounted. This helper mirrors the shell's
 * real composition (rendered tray, in-memory arrangement, optional exact
 * incarnation partition), so a test that passes here exercises what ships.
 *
 * It deliberately does NOT render the shell chrome. A test that needs to prove
 * the shell mounts a host at all must render `ConsoleShell`/`ConsoleApp` — see
 * `console/window/consoleShellWindowHost.test.tsx`.
 */
export function renderWithWindowManager(
  ui: ReactNode,
  options: {
    /** Exact non-secret session/incarnation partition; omit for in-memory only. */
    authorityPartition?: string;
    /** Mirrors the shell: retention follows an owned partition. */
    retentionEnabled?: boolean;
  } = {},
): RenderResult {
  const { authorityPartition, retentionEnabled = authorityPartition !== undefined } =
    options;
  // The `wrapper` option (not explicit nesting) so `rerender` keeps the host —
  // a rerender that dropped it would silently reproduce the very bug this
  // helper exists to stop.
  return render(ui, {
    wrapper: ({ children }: { children: ReactNode }) => (
      <WindowManagerProvider
        authorityPartition={authorityPartition}
        retentionEnabled={retentionEnabled}
      >
        {children}
      </WindowManagerProvider>
    ),
  });
}
