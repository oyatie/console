import { recruitingStrings as text } from "../../i18n/recruiting";
import type { RecruitPreflightCheck } from "./recruitingApi";
import { preflightCheckLabel } from "./recruitingFormat";

export interface PreflightState {
  postingId: string;
  roleTitle: string;
  state: "loading" | "ready" | "error";
  checks: RecruitPreflightCheck[];
  publishable: boolean;
  attest: boolean;
  /** Server-reported failure from a rejected publish (422 fail-closed). */
  error?: string;
}

type Props = {
  preflight: PreflightState;
  busy: boolean;
  onToggleAttest: () => void;
  onPublish: () => void;
  onRetry: () => void;
  onClose: () => void;
};

function CheckIcon({ ok }: { ok: boolean }) {
  return ok ? (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--ok-solid)" strokeWidth="2.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M20 6 9 17l-5-5" /></svg>
  ) : (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="var(--danger-tx)" strokeWidth="2.6" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M18 6 6 18 M6 6l12 12" /></svg>
  );
}

/** §4-29 publish gate: server checks + manual exposure attest — fail closed. */
export function PreflightModal({ preflight, busy, onToggleAttest, onPublish, onRetry, onClose }: Props) {
  const publishable = preflight.state === "ready" && preflight.publishable;
  return (
    <div
      className="recruiting__overlay recruiting__overlay--center"
      onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}
      onKeyDown={(event) => { if (event.key === "Escape") onClose(); }}
    >
      <div role="dialog" aria-modal="true" aria-labelledby="recruiting-preflight-title" className="recruiting__preflight">
        <div className="recruiting__preflight-head">
          <span id="recruiting-preflight-title" className="recruiting__preflight-title">{text.preflight.title}</span>
          <span className="recruiting__chip recruiting__chip--muted">{preflight.roleTitle}</span>
        </div>
        {preflight.state === "loading" && <p className="recruiting__state" role="status">{text.preflight.loading}</p>}
        {preflight.state === "error" && (
          <div className="recruiting__banner recruiting__banner--danger" role="alert">
            <span>{text.preflight.error}</span>
            <button type="button" className="recruiting__ghost" onClick={onRetry}>{text.retry}</button>
          </div>
        )}
        {preflight.state === "ready" && (
          <div className="recruiting__preflight-checks">
            {preflight.checks.map((check) => (
              <div key={check.key} className="recruiting__preflight-check">
                <CheckIcon ok={check.ok} />
                <span className="recruiting__preflight-check-key">{preflightCheckLabel(check.key)}</span>
                <span className={check.ok ? "recruiting__preflight-check-note" : "recruiting__preflight-check-note recruiting__preflight-check-note--bad"}>{check.note}</span>
              </div>
            ))}
          </div>
        )}
        {preflight.error !== undefined && (
          <div className="recruiting__banner recruiting__banner--danger" role="alert">{preflight.error}</div>
        )}
        <button type="button" className="recruiting__attest" onClick={onToggleAttest} aria-pressed={preflight.attest}>
          <span className={preflight.attest ? "recruiting__attest-box recruiting__attest-box--on" : "recruiting__attest-box"} aria-hidden="true">
            <svg width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="var(--surface)" strokeWidth="3.5" strokeLinecap="round" strokeLinejoin="round"><path d="M20 6 9 17l-5-5" /></svg>
          </span>
          <span className="recruiting__attest-text">{text.preflight.attest}</span>
        </button>
        {preflight.state === "ready" && !preflight.publishable && (
          <div className="recruiting__banner recruiting__banner--warn">{text.preflight.blocked}</div>
        )}
        <div className="recruiting__preflight-foot">
          <button type="button" className="recruiting__ghost" autoFocus onClick={onClose}>{text.preflight.cancel}</button>
          {publishable && preflight.attest && (
            <button type="button" className="recruiting__primary" disabled={busy} onClick={onPublish}>{text.preflight.publish}</button>
          )}
        </div>
      </div>
    </div>
  );
}
