import {flushSync} from "react-dom";
import {createRoot, type Root} from "react-dom/client";
import {decodeProjection, type Page} from "./people-projection";
import {PeopleView} from "./people-view";
import "./people.css";

declare global {
  interface Window { readonly __consolePeopleGuard?: {canPromote: () => boolean}; }
}

const fallback = document.getElementById("console-people-fallback");
const bootstrap = document.getElementById("console-people-bootstrap");
const guard = window.__consolePeopleGuard;
function untouched(): boolean {
  return !!fallback && !!guard?.canPromote()
    // A pending native autofocus belongs to validation/conflict recovery.
    && !fallback.querySelector("[autofocus]")
    // UA autofill/history restoration can change a value without an event.
    && [...fallback.querySelectorAll<HTMLInputElement>("input")].every(input => input.value === input.defaultValue);
}
if (fallback && bootstrap && guard && untouched()) {
  let page: Page | undefined;
  try {
    // Decode everything before creating any new actionable DOM.
    page = decodeProjection(JSON.parse(bootstrap.textContent ?? ""));
  } catch {
    const status = document.createElement("p");
    status.setAttribute("role", "status");
    status.setAttribute("data-console-people-render-status", "invalid-projection");
    status.className = "people-render-status";
    status.textContent = "화면을 표시하지 못했습니다. 현재 내용을 유지합니다.";
    // Keep the complete authorized fallback byte-for-byte intact, including
    // its form controls. This sibling is an informational status only.
    fallback.before(status);
  }
  if (page && untouched()) {
    const approvedPage = page;
    const host = document.createElement("div");
    host.setAttribute("data-console-people-root", "");
    let root: Root | undefined, failed = false;
    try {
      root = createRoot(host, {
        onUncaughtError: () => { failed = true; },
        onCaughtError: () => { failed = true; },
        onRecoverableError: () => { failed = true; },
      });
      // Commit into a detached host. A fault or interaction during host creation
      // leaves the entire existing document, including inputs and selection.
      flushSync(() => root!.render(<PeopleView page={approvedPage}/>));
      if (!failed && host.querySelector('[data-console-react-people="mounted"]') && untouched() && fallback.isConnected) {
        fallback.replaceWith(host);
      } else {
        root.unmount();
      }
    } catch {
      try { root?.unmount(); } catch { /* The untouched SSR document is the recovery path. */ }
    }
  }
}
