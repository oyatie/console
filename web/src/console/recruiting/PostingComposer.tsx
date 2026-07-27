import { useId, useState } from "react";

import { recruitingStrings as text } from "../../i18n/recruiting";
import type {
  CreateRecruitPostingRequest,
  RecruitEmploymentType,
  RecruitPostingScope,
  RecruitPostingView,
} from "./recruitingApi";
import { employmentLabel } from "./recruitingFormat";

const EMPLOYMENT_TYPES: readonly RecruitEmploymentType[] = [
  "REGULAR",
  "RESIDENT_SHIFT",
  "PART_TIME",
  "POOL_DAILY",
];

type Props = {
  /** Present = editing an existing DRAFT (PUT); absent = new posting (POST). */
  posting?: RecruitPostingView;
  busy: boolean;
  /** Server-reported failure (validation/conflict) — rendered fail-closed. */
  error?: string;
  onSaveDraft: (input: CreateRecruitPostingRequest) => void;
  onPublish: (input: CreateRecruitPostingRequest) => void;
  onClose: () => void;
};

/** 공고 등록 컴포저 — typed fields, 초안 저장/게시, fail-closed validation. */
export function PostingComposer({ posting, busy, error, onSaveDraft, onPublish, onClose }: Props) {
  const [roleTitle, setRoleTitle] = useState(posting?.role_title ?? "");
  const [company, setCompany] = useState(posting?.company ?? text.composer.companies[0]);
  const [worksite, setWorksite] = useState(posting?.worksite ?? "");
  const [employment, setEmployment] = useState<RecruitEmploymentType>(posting?.employment_type ?? "REGULAR");
  const [scope, setScope] = useState<RecruitPostingScope>(posting?.scope ?? "EXTERNAL");
  const [headcount, setHeadcount] = useState(posting?.headcount ?? 1);
  const [deadline, setDeadline] = useState(posting?.deadline ?? "");
  const [requirements, setRequirements] = useState<string[]>(posting?.requirements ?? []);
  const [requirementDraft, setRequirementDraft] = useState("");
  const [invalid, setInvalid] = useState(false);
  const roleId = useId();
  const siteId = useId();
  const deadlineId = useId();
  const requirementId = useId();

  const addRequirement = () => {
    const value = requirementDraft.trim();
    if (!value) return;
    setRequirements((current) => (current.includes(value) ? current : [...current, value]));
    setRequirementDraft("");
  };

  const buildInput = (): CreateRecruitPostingRequest | undefined => {
    if (!roleTitle.trim() || !worksite.trim()) {
      setInvalid(true);
      return undefined;
    }
    return {
      role_title: roleTitle.trim(),
      company,
      worksite: worksite.trim(),
      employment_type: employment,
      scope,
      headcount: Math.max(1, headcount),
      ...(deadline ? { deadline } : {}),
      requirements,
    };
  };

  const submit = (publish: boolean) => {
    const input = buildInput();
    if (!input) return;
    if (publish) onPublish(input);
    else onSaveDraft(input);
  };

  return (
    <div
      className="recruiting__overlay recruiting__overlay--center"
      onClick={(event) => { if (event.target === event.currentTarget) onClose(); }}
      onKeyDown={(event) => { if (event.key === "Escape") onClose(); }}
    >
      <div role="dialog" aria-modal="true" aria-labelledby="recruiting-composer-title" className="recruiting__composer">
        <div className="recruiting__composer-head">
          <span id="recruiting-composer-title" className="recruiting__composer-title">
            {posting ? text.composer.editTitle : text.composer.title}
          </span>
          <span className="recruiting__chip recruiting__chip--mono">{posting?.posting_no ?? text.composer.draftChip}</span>
          <span className="recruiting__spacer" />
          <button type="button" className="recruiting__icon-button" aria-label={text.composer.close} autoFocus onClick={onClose}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.4" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M18 6 6 18 M6 6l12 12" /></svg>
          </button>
        </div>
        <div className="recruiting__composer-body">
          <div className="recruiting__field">
            <label className="recruiting__field-label" htmlFor={roleId}>{text.composer.role}</label>
            <input
              id={roleId}
              className="recruiting__input"
              value={roleTitle}
              placeholder={text.composer.rolePlaceholder}
              onChange={(event) => { setRoleTitle(event.target.value); setInvalid(false); }}
            />
          </div>
          <div className="recruiting__field-grid">
            <fieldset className="recruiting__field recruiting__fieldset">
              <legend className="recruiting__field-label">{text.composer.company}</legend>
              <div className="recruiting__chip-row">
                {text.composer.companies.map((option) => (
                  <button
                    key={option}
                    type="button"
                    className={option === company ? "recruiting__pick recruiting__pick--on" : "recruiting__pick"}
                    aria-pressed={option === company}
                    onClick={() => { setCompany(option); }}
                  >{option}</button>
                ))}
              </div>
            </fieldset>
            <div className="recruiting__field">
              <label className="recruiting__field-label" htmlFor={siteId}>{text.composer.site}</label>
              <input
                id={siteId}
                className="recruiting__input"
                value={worksite}
                placeholder={text.composer.sitePlaceholder}
                onChange={(event) => { setWorksite(event.target.value); setInvalid(false); }}
              />
            </div>
          </div>
          <fieldset className="recruiting__field recruiting__fieldset">
            <legend className="recruiting__field-label">{text.composer.employment}</legend>
            <div className="recruiting__chip-row">
              {EMPLOYMENT_TYPES.map((option) => (
                <button
                  key={option}
                  type="button"
                  className={option === employment ? "recruiting__pick recruiting__pick--on" : "recruiting__pick"}
                  aria-pressed={option === employment}
                  onClick={() => { setEmployment(option); }}
                >{employmentLabel(option)}</button>
              ))}
              {employment === "POOL_DAILY" && (
                <span className="recruiting__chip recruiting__chip--purple">{text.composer.poolChip}</span>
              )}
            </div>
          </fieldset>
          <div className="recruiting__field-grid recruiting__field-grid--tail">
            <fieldset className="recruiting__field recruiting__fieldset">
              <legend className="recruiting__field-label">{text.composer.scope}</legend>
              <div className="recruiting__chip-row">
                {(["EXTERNAL", "INTERNAL"] as const).map((option) => (
                  <button
                    key={option}
                    type="button"
                    className={option === scope ? "recruiting__pick recruiting__pick--on" : "recruiting__pick"}
                    aria-pressed={option === scope}
                    onClick={() => { setScope(option); }}
                  >{text.scopeLabel[option]}</button>
                ))}
                {scope === "INTERNAL" && (
                  <span className="recruiting__chip recruiting__chip--purple">{text.composer.internalNote}</span>
                )}
              </div>
            </fieldset>
            <fieldset className="recruiting__field recruiting__fieldset">
              <legend className="recruiting__field-label">{text.composer.headcount}</legend>
              <div className="recruiting__stepper">
                <button type="button" className="recruiting__stepper-button" aria-label="−" onClick={() => { setHeadcount((current) => Math.max(1, current - 1)); }}>−</button>
                <span className="recruiting__stepper-value">{String(headcount)}</span>
                <button type="button" className="recruiting__stepper-button" aria-label="+" onClick={() => { setHeadcount((current) => current + 1); }}>+</button>
              </div>
            </fieldset>
            <div className="recruiting__field">
              <label className="recruiting__field-label" htmlFor={deadlineId}>{text.composer.deadline}</label>
              <input
                id={deadlineId}
                className="recruiting__input recruiting__input--date"
                type="date"
                value={deadline}
                onChange={(event) => { setDeadline(event.target.value); }}
              />
            </div>
          </div>
          <div className="recruiting__field">
            <label className="recruiting__field-label" htmlFor={requirementId}>{text.composer.requirements}</label>
            <div className="recruiting__req-row">
              <input
                id={requirementId}
                className="recruiting__input"
                value={requirementDraft}
                placeholder={text.composer.requirementPlaceholder}
                onChange={(event) => { setRequirementDraft(event.target.value); }}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    event.preventDefault();
                    addRequirement();
                  }
                }}
              />
              <button type="button" className="recruiting__ghost" onClick={addRequirement}>{text.composer.addRequirement}</button>
            </div>
            {requirements.length > 0 && (
              <div className="recruiting__chip-row">
                {requirements.map((requirement) => (
                  <span key={requirement} className="recruiting__chip recruiting__chip--info">
                    {requirement}
                    <button
                      type="button"
                      className="recruiting__chip-x"
                      aria-label={`${text.composer.removeRequirement} — ${requirement}`}
                      onClick={() => { setRequirements((current) => current.filter((item) => item !== requirement)); }}
                    >×</button>
                  </span>
                ))}
              </div>
            )}
          </div>
          {invalid && (
            <div className="recruiting__banner recruiting__banner--danger" role="alert">{text.composer.validationError}</div>
          )}
          {error !== undefined && (
            <div className="recruiting__banner recruiting__banner--danger" role="alert">{error}</div>
          )}
        </div>
        <div className="recruiting__composer-foot">
          <button type="button" className="recruiting__ghost" disabled={busy} onClick={() => { submit(false); }}>{text.composer.saveDraft}</button>
          <span className="recruiting__spacer" />
          <button type="button" className="recruiting__primary" disabled={busy} onClick={() => { submit(true); }}>{text.composer.publish}</button>
        </div>
      </div>
    </div>
  );
}
