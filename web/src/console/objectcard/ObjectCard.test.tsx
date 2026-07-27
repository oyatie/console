import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { ko } from "../../i18n/ko";
import { PolicyGateProvider, type PolicyGate } from "../policy";
import { objectRefToken, type WindowEntry } from "../window";
import { ObjectCard, ObjectCardModal, objectCardWindowEntry } from "./ObjectCard";
import { createObjectCardStub } from "./stub";
import { objectCardA11yStrings, objectCardDynStrings } from "./strings";
import {
  OBJECT_CARD_ACTIONS,
  type ObjectCardDescriptor,
  type ObjectCardHandlers,
} from "./types";

const T = ko.console.objectcard;
const DYN = objectCardDynStrings();
const A11Y = objectCardA11yStrings();
const allowGate: PolicyGate = { can: () => true };

function renderCard(gate: PolicyGate, handlers?: ObjectCardHandlers, descriptor = createObjectCardStub()) {
  return render(
    <PolicyGateProvider gate={gate}>
      <ObjectCard descriptor={descriptor} handlers={handlers} />
    </PolicyGateProvider>,
  );
}

describe("ObjectCard three-layer structure", () => {
  it("renders the semantic, kinetic, and dynamic layer headings", () => {
    renderCard(allowGate);
    expect(screen.getByText(T.layers.semantic)).toBeTruthy();
    expect(screen.getByText(T.layers.kinetic)).toBeTruthy();
    expect(screen.getByText(T.layers.dynamic)).toBeTruthy();
  });
});

describe("ObjectCard property-policy deny-by-omission", () => {
  it("hides a property-policy field when the subject cannot read it", () => {
    // deny only the policy-gated 'cost' property; everything else allowed.
    const gate: PolicyGate = {
      can: (action, resource) =>
        !(
          action === OBJECT_CARD_ACTIONS.propertyRead &&
          typeof resource === "object" &&
          resource.id === "cost"
        ),
    };
    renderCard(gate);
    expect(screen.getByText(T.samples.props.priority)).toBeTruthy();
    expect(screen.queryByText(T.samples.props.cost)).toBeNull();
  });

  it("shows the property-policy field when read is allowed", () => {
    renderCard(allowGate);
    expect(screen.getByText(T.samples.props.cost)).toBeTruthy();
  });
});

describe("ObjectCard §20 override vs draft-direct chip", () => {
  it("shows the override chip on a non-draft object", () => {
    renderCard(allowGate, undefined, createObjectCardStub({ lifecycleState: "active" }));
    expect(screen.getAllByText(T.edit.override).length).toBeGreaterThan(0);
  });

  it("shows the direct-edit chip on a draft object", () => {
    renderCard(allowGate, undefined, createObjectCardStub({ lifecycleState: "draft" }));
    expect(screen.getAllByText(T.edit.direct).length).toBeGreaterThan(0);
  });

  it("requires a reason before an override edit commits", () => {
    const onEdit = vi.fn();
    renderCard(allowGate, { onEdit }, createObjectCardStub({ lifecycleState: "active" }));
    // open the override banner (the toggle button carries the override label)
    fireEvent.click(screen.getByRole("button", { name: T.edit.override }));
    fireEvent.click(screen.getByRole("button", { name: T.edit.apply }));
    expect(onEdit).not.toHaveBeenCalled();
    fireEvent.change(screen.getByLabelText(T.edit.reasonLabel), { target: { value: "감사 정정" } });
    fireEvent.click(screen.getByRole("button", { name: T.edit.apply }));
    expect(onEdit).toHaveBeenCalledWith({ mode: "override", reason: "감사 정정" });
  });
});

describe("ObjectCard relation drawing + actions", () => {
  it("draws an edge from a typed code on Enter", () => {
    const onRelationAdd = vi.fn();
    renderCard(allowGate, { onRelationAdd });
    const input = screen.getByLabelText(T.relations.codeLabel);
    fireEvent.change(input, { target: { value: "AT-CHO" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onRelationAdd).toHaveBeenCalledWith({ code: "AT-CHO", title: "AT-CHO", linkType: "relates_to" });
  });

  it("invokes an action through the audited execute stub", () => {
    const onAction = vi.fn();
    renderCard(allowGate, { onAction });
    fireEvent.click(screen.getByRole("button", { name: T.actionAria(T.samples.actions.reassign) }));
    expect(onAction).toHaveBeenCalledWith(expect.objectContaining({ key: "reassign" }), {});
  });

  it("gates the action button behind the execute policy (deny-by-omission)", () => {
    const denyExecute: PolicyGate = { can: (action) => action !== OBJECT_CARD_ACTIONS.actionExecute };
    renderCard(denyExecute);
    expect(
      screen.queryByRole("button", { name: T.actionAria(T.samples.actions.reassign) }),
    ).toBeNull();
  });

  it("removes an edge by link id", () => {
    const onRelationRemove = vi.fn();
    renderCard(allowGate, { onRelationRemove });
    const removeButtons = screen.getAllByText(T.relations.remove);
    fireEvent.click(removeButtons[0]);
    expect(onRelationRemove).toHaveBeenCalledWith("lnk-1");
  });
});

describe("ObjectCard code resolve (run-log/code chip targets)", () => {
  it("draws the edge with the server-resolved title, not the fabricated code text", async () => {
    const onRelationAdd = vi.fn();
    const onResolveCode = vi.fn().mockResolvedValue({ title: "5호기 지게차" });
    renderCard(allowGate, { onRelationAdd, onResolveCode });
    const input = screen.getByLabelText(T.relations.codeLabel);
    fireEvent.change(input, { target: { value: "EQ-118" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(onResolveCode).toHaveBeenCalledWith("EQ-118");
    await waitFor(() => {
      expect(onRelationAdd).toHaveBeenCalledWith({
        code: "EQ-118",
        title: "5호기 지게차",
        linkType: "relates_to",
      });
    });
  });

  it("refuses to draw an edge for a code that fails to resolve (no fabricated title, deny-by-omission)", async () => {
    const onRelationAdd = vi.fn();
    const onResolveCode = vi.fn().mockResolvedValue(null);
    renderCard(allowGate, { onRelationAdd, onResolveCode });
    const input = screen.getByLabelText(T.relations.codeLabel);
    fireEvent.change(input, { target: { value: "EQ-999" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(await screen.findByText(DYN.relations.codeNotFound)).toBeTruthy();
    expect(onRelationAdd).not.toHaveBeenCalled();
  });
});

describe("ObjectCard acting chips (dynamic layer)", () => {
  it("navigates on click and stays inert without a handler", () => {
    const onActingChipClick = vi.fn();
    renderCard(allowGate, { onActingChipClick });
    const chip = screen.getByRole("button", {
      name: DYN.acting.navigateAria("wf-wo-review", T.acting.automation),
    });
    fireEvent.click(chip);
    expect(onActingChipClick).toHaveBeenCalledWith(
      expect.objectContaining({ id: "wf-1", kind: "automation" }),
    );
  });

  it("is disabled (no-op) when no navigate handler is wired", () => {
    renderCard(allowGate);
    const chip = screen.getByRole("button", {
      name: DYN.acting.navigateAria("wf-wo-review", T.acting.automation),
    });
    expect((chip as HTMLButtonElement).disabled).toBe(true);
  });
});

describe("ObjectCard actionable controls", () => {
  it("does not render mutation controls when no real handler is wired", () => {
    renderCard(allowGate);
    expect(screen.queryByRole("button", { name: T.actionAria(T.samples.actions.reassign) })).toBeNull();
    expect(screen.queryByRole("button", { name: T.edit.override })).toBeNull();
    expect(screen.queryByRole("button", { name: T.relations.add })).toBeNull();
  });

  it("can add real relation and edit handlers after mount without changing hook order", () => {
    const descriptor = createObjectCardStub({ lifecycleState: "active" });
    const view = renderCard(allowGate, undefined, descriptor);

    view.rerender(
      <PolicyGateProvider gate={allowGate}>
        <ObjectCard
          descriptor={descriptor}
          handlers={{ onRelationAdd: vi.fn(), onEdit: vi.fn() }}
        />
      </PolicyGateProvider>,
    );

    expect(screen.getByRole("button", { name: T.relations.add })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: T.edit.override })).toBeInTheDocument();
  });

  it("can retire relation and edit handlers after mount without changing hook order", () => {
    const descriptor = createObjectCardStub({ lifecycleState: "active" });
    const handlers = { onRelationAdd: vi.fn(), onEdit: vi.fn() };
    const view = renderCard(
      allowGate,
      handlers,
      descriptor,
    );
    fireEvent.change(screen.getByLabelText(T.relations.codeLabel), {
      target: { value: "EQ-118" },
    });
    fireEvent.click(screen.getByRole("button", { name: T.edit.override }));

    view.rerender(
      <PolicyGateProvider gate={allowGate}>
        <ObjectCard descriptor={descriptor} />
      </PolicyGateProvider>,
    );

    expect(screen.queryByRole("button", { name: T.relations.add })).toBeNull();
    expect(screen.queryByRole("button", { name: T.edit.override })).toBeNull();

    view.rerender(
      <PolicyGateProvider gate={allowGate}>
        <ObjectCard descriptor={descriptor} handlers={handlers} />
      </PolicyGateProvider>,
    );
    expect(screen.getByLabelText(T.relations.codeLabel)).toHaveValue("");
    expect(screen.queryByLabelText(T.edit.reasonLabel)).toBeNull();
  });
});

// ── L-F2 · shared-card a11y ───────────────────────────────────────────────
// The drag hosts were bare `<span {...objDrag(...)}>`: no role, no tab stop, no
// keyboard path to the reference they carry. 13 module lanes were poised to
// copy that shape, so the fix lands once, here, in the shared card.

const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "textarea:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

function focusableWithin(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR));
}

function stubClipboard(writeText: (text: string) => Promise<void>): void {
  Object.defineProperty(navigator, "clipboard", {
    value: { writeText: vi.fn(writeText) },
    configurable: true,
    writable: true,
  });
}

describe("ObjectCard drag host is a keyboard-operable control", () => {
  const stub = createObjectCardStub();
  const headerName = A11Y.copyRefAria(stub.code, stub.title);

  it("renders the header object code as a focusable button, not an inert span", () => {
    renderCard(allowGate);
    const host = screen.getByRole("button", { name: headerName });
    expect(host.tagName).toBe("BUTTON");
    expect(host.getAttribute("draggable")).toBe("true");
    expect(host.getAttribute("data-obj-code")).toBe(stub.code);
  });

  it("renders every relation row's far-end reference as a focusable button", () => {
    renderCard(allowGate);
    const relation = stub.relations[0];
    const host = screen.getByRole("button", {
      name: A11Y.copyRefAria(relation.code, relation.title),
    });
    expect(host.tagName).toBe("BUTTON");
    expect(host.getAttribute("data-obj-code")).toBe(relation.code);
  });

  it("copies the exact drag payload token on keyboard activation", async () => {
    const user = userEvent.setup();
    const written: string[] = [];
    stubClipboard((text) => {
      written.push(text);
      return Promise.resolve();
    });
    renderCard(allowGate);
    const host = screen.getByRole("button", { name: headerName });
    host.focus();
    expect(document.activeElement).toBe(host);
    await user.keyboard("{Enter}");
    await waitFor(() => {
      expect(written).toEqual([objectRefToken(stub.code, stub.title)]);
    });
    expect(await screen.findByText(A11Y.copied)).toBeTruthy();
  });

  it("reports a failed copy instead of dying silently", async () => {
    const user = userEvent.setup();
    stubClipboard(() => Promise.reject(new Error("denied")));
    renderCard(allowGate);
    const host = screen.getByRole("button", { name: headerName });
    host.focus();
    await user.keyboard("{Enter}");
    expect(await screen.findByRole("alert")).toHaveTextContent(A11Y.copyFailed);
  });
});

function ModalHost({ descriptor = createObjectCardStub() }: { descriptor?: ObjectCardDescriptor }) {
  const [open, setOpen] = useState(false);
  return (
    <PolicyGateProvider gate={allowGate}>
      <button
        type="button"
        onClick={() => {
          setOpen(true);
        }}
      >
        open-card
      </button>
      {open ? (
        <ObjectCardModal
          descriptor={descriptor}
          onClose={() => {
            setOpen(false);
          }}
        />
      ) : null}
    </PolicyGateProvider>
  );
}

describe("ObjectCardModal focus management", () => {
  it("moves initial focus into the dialog on open", async () => {
    const user = userEvent.setup();
    render(<ModalHost />);
    await user.click(screen.getByRole("button", { name: "open-card" }));
    const dialog = screen.getByRole("dialog");
    expect(dialog.contains(document.activeElement)).toBe(true);
    expect(document.activeElement).toBe(
      screen.getByRole("button", { name: ko.console.window.close }),
    );
  });

  it("keeps Tab inside the dialog (forward wrap)", async () => {
    const user = userEvent.setup();
    render(<ModalHost />);
    await user.click(screen.getByRole("button", { name: "open-card" }));
    const dialog = screen.getByRole("dialog");
    const focusable = focusableWithin(dialog);
    expect(focusable.length).toBeGreaterThan(1);
    focusable[focusable.length - 1].focus();
    await user.tab();
    expect(document.activeElement).toBe(focusable[0]);
  });

  it("keeps Shift+Tab inside the dialog (backward wrap)", async () => {
    const user = userEvent.setup();
    render(<ModalHost />);
    await user.click(screen.getByRole("button", { name: "open-card" }));
    const dialog = screen.getByRole("dialog");
    const focusable = focusableWithin(dialog);
    focusable[0].focus();
    await user.tab({ shift: true });
    expect(document.activeElement).toBe(focusable[focusable.length - 1]);
  });

  it("returns focus to the invoking control when Escape closes the dialog", async () => {
    const user = userEvent.setup();
    render(<ModalHost />);
    const opener = screen.getByRole("button", { name: "open-card" });
    await user.click(opener);
    expect(document.activeElement).not.toBe(opener);
    await user.keyboard("{Escape}");
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).toBeNull();
    });
    expect(document.activeElement).toBe(opener);
  });

  it("returns focus to the invoking control when the close button closes the dialog", async () => {
    const user = userEvent.setup();
    render(<ModalHost />);
    const opener = screen.getByRole("button", { name: "open-card" });
    await user.click(opener);
    await user.click(screen.getByRole("button", { name: ko.console.window.close }));
    await waitFor(() => {
      expect(screen.queryByRole("dialog")).toBeNull();
    });
    expect(document.activeElement).toBe(opener);
  });
});

// ── L-F2 · objectCardWindowEntry contract freeze ──────────────────────────
// Frozen for the wave: L-X10..L-X13 call this exact signature and import it
// from `console/objectcard`, never from `console/window`. A change to the
// parameter list, the return shape, or the descriptor→entry field mapping
// breaks every module lane that adopts the open gesture.
type FrozenObjectCardWindowEntry = (
  descriptor: ObjectCardDescriptor,
  handlers?: ObjectCardHandlers,
) => WindowEntry;

describe("objectCardWindowEntry frozen contract", () => {
  it("matches the frozen (descriptor, handlers?) => WindowEntry signature", () => {
    const frozen: FrozenObjectCardWindowEntry = objectCardWindowEntry;
    expect(frozen).toBe(objectCardWindowEntry);
    // handlers is optional at the type level and at runtime.
    expect(objectCardWindowEntry(createObjectCardStub())).toBeTruthy();
  });

  it("maps descriptor id/title/code onto the window entry verbatim", () => {
    const descriptor = createObjectCardStub();
    const entry = objectCardWindowEntry(descriptor);
    expect(entry.id).toBe(descriptor.id);
    expect(entry.title).toBe(descriptor.title);
    expect(entry.code).toBe(descriptor.code);
    expect(Object.keys(entry).sort()).toEqual(["code", "id", "render", "title"]);
  });

  it("renders the card and threads handlers through entry.render()", () => {
    const onAction = vi.fn();
    const entry = objectCardWindowEntry(createObjectCardStub(), { onAction });
    render(<PolicyGateProvider gate={allowGate}>{entry.render()}</PolicyGateProvider>);
    expect(screen.getByText(T.layers.semantic)).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: T.actionAria(T.samples.actions.reassign) }));
    expect(onAction).toHaveBeenCalledWith(expect.objectContaining({ key: "reassign" }), {});
  });
});
