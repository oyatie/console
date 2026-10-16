import {flushSync} from "react-dom";
import {createRoot, type Root} from "react-dom/client";
import {decodeProjection} from "./account-projection";
import {AccountView} from "./account-view";
import "./account.css";

type Controller = {readonly ready: Promise<boolean>; promote: (host: Element) => boolean; recover: (host: Element) => boolean};
declare global {
  interface Window {
    readonly __consoleAccountController?: Controller;
    readonly __consoleAccountGuard?: {canPromote: () => boolean};
  }
}
function controller(): Promise<Controller> {
  if (window.__consoleAccountController) return Promise.resolve(window.__consoleAccountController);
  return new Promise(resolve => {
    const available = () => {
      const handle = window.__consoleAccountController;
      if (handle) { window.removeEventListener("console-account-controller-ready", available); resolve(handle); }
    };
    window.addEventListener("console-account-controller-ready", available);
    available();
  });
}
function untouched(fallback: HTMLElement): boolean {
  return fallback.isConnected && !!window.__consoleAccountGuard?.canPromote()
    && !fallback.querySelector("[autofocus]")
    && [...fallback.querySelectorAll<HTMLInputElement>("input")].every(input => input.value === input.defaultValue && input.checked === input.defaultChecked);
}
async function mount(): Promise<void> {
  const fallback = document.getElementById("console-account-fallback"), bootstrap = document.getElementById("console-account-bootstrap");
  if (!fallback || !bootstrap || !untouched(fallback)) return;
  const source = bootstrap.textContent ?? "";
  if (source.length > 2097152) return;
  let projection;
  try { projection = decodeProjection(JSON.parse(source)); } catch { return; }
  const handle = await controller();
  if (!await handle.ready || !untouched(fallback)) return;
  const host = document.createElement("div");
  host.setAttribute("data-console-account-root", "");
  let root: Root | undefined, failed = false, promoted = false;
  const fault = () => {
    failed = true;
    if (promoted && handle.recover(host) && !host.querySelector("[data-account-render-error]")) {
      const message = document.createElement("p");
      message.className = "error account-render-error";
      message.setAttribute("role", "alert");
      message.setAttribute("data-account-render-error", "");
      message.textContent = "화면을 표시하는 중 문제가 생겼습니다. 입력과 진행 중인 요청을 유지합니다. 현재 안내에서 이어가세요.";
      host.append(message);
    }
  };
  try {
    root = createRoot(host, {onUncaughtError: fault, onCaughtError: fault, onRecoverableError: fault});
    flushSync(() => root!.render(<AccountView projection={projection}/>));
    if (!failed && untouched(fallback) && host.querySelector('[data-console-react-account="mounted"]')) {
      promoted = handle.promote(host);
    }
  } catch { /* The original scoped controller retains the complete fallback. */ }
  if (!promoted) { try { root?.unmount(); } catch { /* Detached failed React content has no authority. */ } }
  // A fault after user progress never restores blank fallback or dispatches a new attempt.
}
void mount().catch(() => { /* Preserve the original working document and controller. */ });
