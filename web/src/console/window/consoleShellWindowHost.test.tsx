import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { StrictMode, type ReactNode } from "react";
import { MemoryRouter, useNavigate } from "react-router";
import { afterEach, describe, expect, it, vi } from "vitest";

import { useAuth } from "../../context/auth";
import type { AuthSession } from "../../context/auth";
import { ko } from "../../i18n/ko";
import { AuthTestProvider } from "../../test/AuthTestProvider";
import { ConsoleApp } from "../ConsoleApp";
import type { ConsoleAuthz, ScopeOption } from "../shell/authz";
import { consoleScreenPath, MOUNTED_SCREEN_KEYS } from "../shell/nav";

/**
 * The regression this file exists for: `WindowManagerProvider` was mounted only
 * in the legacy `AppShell`, in per-test wrappers, and as a nested provider
 * inside `OntologyWorkspaceBody`. Every console screen body therefore ran with
 * no window host in production while every module's own test passed, because
 * each test supplied the provider the shell never did.
 *
 * So these assertions render the real `ConsoleApp`/`ConsoleShell` composition —
 * never a hand-wrapped body — and read the host through `[data-window-host]`.
 */

const T = ko.console.window;

const ENTRY_A_TITLE = "작업지시 4102";
const ENTRY_B_TITLE = "계약 C-207";

vi.mock("../rum/rum", () => ({
  initConsoleRum: () => () => {},
  markConsoleRoute: () => {},
}));

// Shell chrome only: keep the async transports (authz projection, nav badge
// inbox, self profile, comms rail) out of a window-host assertion.
vi.mock("../shell/authz", () => ({
  UNION_SCOPE_ID: "__union__",
  useConsoleScopes: (unionLabel: string) => ({
    options: [
      { id: "__union__", label: unionLabel, memberIds: [], isUnion: true },
    ] satisfies ScopeOption[],
    loading: false,
  }),
  ConsoleAuthzProvider: ({ children }: { children: ReactNode }) => <>{children}</>,
  useConsoleAuthz: () => {
    const { session } = useAuth();
    return {
      grants: {
        roles: session?.roles ?? [],
        featureGrants: session?.feature_grants ?? [],
      },
      source: "jwt" as const,
      ready: true,
    } satisfies ConsoleAuthz;
  },
}));
vi.mock("../shell/navBadges", () => ({ useNavBadges: () => ({}) }));
vi.mock("../shell/useSelfProfile", () => ({ useSelfProfile: () => ({}) }));
vi.mock("../shell/CommsRailPanel", () => ({
  CommsRailPanel: () => null,
  CommsRailFallback: () => null,
}));

// Every mounted screen renders the same probe: the question under test is
// whether a screen body — any screen body — sees the shell's window manager.
vi.mock("../screens/registry", async () => {
  const { useOptionalWindowManager } = await import("./windowManagerContext");
  const { MOUNTED_SCREEN_KEYS: keys } = await import("../shell/nav");

  const entryA = {
    id: "obj-a",
    title: "작업지시 4102",
    render: () => <div data-testid="panel-body-a" />,
  };
  const entryB = {
    id: "obj-b",
    title: "계약 C-207",
    render: () => <div data-testid="panel-body-b" />,
  };

  function WindowProbe() {
    const manager = useOptionalWindowManager();
    if (!manager) return <div data-testid="probe-without-manager" />;
    return (
      <div data-testid="probe-with-manager">
        <button
          data-testid="probe-open-a"
          type="button"
          onClick={() => {
            manager.open(entryA);
          }}
        >
          a
        </button>
        <button
          data-testid="probe-open-b"
          type="button"
          onClick={() => {
            manager.open(entryB);
          }}
        >
          b
        </button>
      </div>
    );
  }

  return {
    SCREEN_REGISTRY: Object.fromEntries(keys.map((key) => [key, WindowProbe])),
  };
});

function Navigator() {
  const navigate = useNavigate();
  return (
    <button
      data-testid="goto-explorer"
      type="button"
      onClick={() => {
        void navigate(consoleScreenPath("objectExplorer"));
      }}
    >
      go
    </button>
  );
}

/** A session with the owned org/user/incarnation the layout partition requires. */
function ownedSession(incarnation: string): AuthSession {
  return {
    access_token: "t",
    display_name: "전성진",
    roles: ["SUPER_ADMIN"],
    org_id: "org-1",
    user_id: "user-1",
    client_session_incarnation: incarnation,
  };
}

const UNOWNED_SESSION: AuthSession = {
  access_token: "t",
  display_name: "전성진",
  roles: ["SUPER_ADMIN"],
  org_id: "org-1",
};

function renderConsole(session: AuthSession) {
  return render(
    <MemoryRouter initialEntries={["/console"]}>
      <AuthTestProvider session={session}>
        <ConsoleApp screenKeys={MOUNTED_SCREEN_KEYS} />
      </AuthTestProvider>
      <Navigator />
    </MemoryRouter>,
  );
}

function windowHosts(): HTMLElement[] {
  return Array.from(document.querySelectorAll<HTMLElement>("[data-window-host]"));
}

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("console shell window host", () => {
  it("mounts exactly one window host, and it wraps every screen body", () => {
    renderConsole(UNOWNED_SESSION);

    const hosts = windowHosts();
    expect(hosts).toHaveLength(1);
    // Wrapping the shell root (not sitting inside the screen section) is what
    // makes the split padding move the sidebar and comms rail out of the pinned
    // panel's way, and what keeps one arrangement across navigation.
    expect(hosts[0]).toContainElement(
      document.querySelector<HTMLElement>("[data-cshell-root]"),
    );
    expect(screen.getByTestId("probe-with-manager")).toBeInTheDocument();
    expect(screen.queryByTestId("probe-without-manager")).not.toBeInTheDocument();
  });

  it("keeps the previously pinned object recoverable when a second one is opened", () => {
    renderConsole(UNOWNED_SESSION);

    fireEvent.click(screen.getByTestId("probe-open-a"));
    expect(screen.getByRole("region", { name: ENTRY_A_TITLE })).toBeVisible();

    // Single-pin: the second open replaces the panel. The first must land in the
    // tray with a restore affordance — silently dropping it is data loss.
    fireEvent.click(screen.getByTestId("probe-open-b"));
    expect(screen.getByRole("region", { name: ENTRY_B_TITLE })).toBeVisible();
    expect(
      screen.queryByRole("region", { name: ENTRY_A_TITLE }),
    ).not.toBeInTheDocument();

    const tray = screen.getByRole("group", { name: T.tray });
    const chip = within(tray).getByRole("button", {
      name: T.restoreItem(ENTRY_A_TITLE),
    });
    fireEvent.click(chip);

    expect(screen.getByRole("region", { name: ENTRY_A_TITLE })).toBeVisible();
    // …and the demoted one is recoverable in turn: neither direction loses state.
    expect(
      within(screen.getByRole("group", { name: T.tray })).getByRole("button", {
        name: T.restoreItem(ENTRY_B_TITLE),
      }),
    ).toBeVisible();
  });

  it("keeps the pinned panel and the tray across screen navigation", () => {
    renderConsole(UNOWNED_SESSION);
    const firstScreen = document
      .querySelector("[data-cshell-screen]")
      ?.getAttribute("data-cshell-screen");

    fireEvent.click(screen.getByTestId("probe-open-a"));
    fireEvent.click(screen.getByTestId("probe-open-b"));
    fireEvent.click(screen.getByTestId("goto-explorer"));

    expect(
      document.querySelector("[data-cshell-screen]")?.getAttribute("data-cshell-screen"),
    ).toBe("objectExplorer");
    expect(firstScreen).not.toBe("objectExplorer");
    expect(windowHosts()).toHaveLength(1);
    expect(screen.getByRole("region", { name: ENTRY_B_TITLE })).toBeVisible();
    expect(
      within(screen.getByRole("group", { name: T.tray })).getByRole("button", {
        name: T.restoreItem(ENTRY_A_TITLE),
      }),
    ).toBeVisible();
  });

  it("gives a session with no owned incarnation the full in-memory model and no storage", () => {
    const getItem = vi.spyOn(Storage.prototype, "getItem");
    const setItem = vi.spyOn(Storage.prototype, "setItem");
    renderConsole(UNOWNED_SESSION);

    fireEvent.click(screen.getByTestId("probe-open-a"));

    expect(screen.getByRole("region", { name: ENTRY_A_TITLE })).toBeVisible();
    for (const call of [...getItem.mock.calls, ...setItem.mock.calls]) {
      expect(call[0]).not.toContain("oyatie.console.window.layout");
    }
  });

  // Named for what it proves: the layout key is READ under a per-incarnation
  // partition. It deliberately does not claim layouts persist — see the
  // inert-storage tripwire below.
  it("reads the layout key under a partition scoped to the exact session incarnation", async () => {
    const getItem = vi.spyOn(Storage.prototype, "getItem");

    const first = render(
      <StrictMode>
        <MemoryRouter initialEntries={["/console"]}>
          <AuthTestProvider session={ownedSession("incarnation-a")}>
            <ConsoleApp screenKeys={MOUNTED_SCREEN_KEYS} />
          </AuthTestProvider>
        </MemoryRouter>
      </StrictMode>,
    );
    const second = render(
      <StrictMode>
        <MemoryRouter initialEntries={["/console"]}>
          <AuthTestProvider session={ownedSession("incarnation-b")}>
            <ConsoleApp screenKeys={MOUNTED_SCREEN_KEYS} />
          </AuthTestProvider>
        </MemoryRouter>
      </StrictMode>,
    );

    const readKeys = () => getItem.mock.calls.map(([key]) => key);
    await waitFor(() => {
      const layoutKeys = new Set(
        readKeys().filter((key) =>
          key.startsWith("oyatie.console.window.layout.v2."),
        ),
      );
      expect(layoutKeys.size).toBe(2);
    });
    // The unpartitioned key is the cross-tenant leak this partitioning exists to
    // prevent; it must never be read.
    expect(readKeys()).not.toContain("oyatie.console.window.layout");

    first.unmount();
    second.unmount();
  });

  /**
   * TRIPWIRE, not an endorsement. Layout retention is inert today: `saveLayout()`
   * is the only writer and nothing calls it, and `register()` is the only
   * restorer and nothing calls it either. Mounting the host must not be mistaken
   * for shipping persistence, so pin an object under an OWNED incarnation — the
   * case that would persist if anything did — and require that no layout write
   * happens. When someone wires a real writer this goes red, which is the point:
   * the partitioning, the do-not-ship-ban-#9 localStorage ceiling and the orphan
   * `ko.console.window.{saveLayout,restoreDefault}` strings all have to be
   * re-decided in that same change.
   */
  it("does not persist a layout — retention is inert until a writer exists", () => {
    const setItem = vi.spyOn(Storage.prototype, "setItem");
    render(
      <MemoryRouter initialEntries={["/console"]}>
        <AuthTestProvider session={ownedSession("incarnation-owned")}>
          <ConsoleApp screenKeys={MOUNTED_SCREEN_KEYS} />
        </AuthTestProvider>
      </MemoryRouter>,
    );

    fireEvent.click(screen.getByTestId("probe-open-a"));
    fireEvent.click(screen.getByTestId("probe-open-b"));
    expect(screen.getByRole("region", { name: ENTRY_B_TITLE })).toBeVisible();

    for (const call of setItem.mock.calls) {
      expect(call[0]).not.toContain("oyatie.console.window.layout");
    }
  });
});
