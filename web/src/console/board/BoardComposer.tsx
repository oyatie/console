import { useCallback, useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

import { boardStrings as text } from "../../i18n/board";
import type {
  BoardApi,
  BoardNotice,
  BranchSummary,
  NoticeAudienceScope,
  NoticeCategory,
} from "./boardApi";

const CATEGORIES: NoticeCategory[] = ["general", "legal", "hr_order", "training"];

interface ComposerForm {
  title: string;
  body: string;
  category: NoticeCategory;
  scope: NoticeAudienceScope;
  branchIds: string[];
}

function seedForm(draft: BoardNotice | undefined): ComposerForm {
  return {
    title: draft?.title ?? "",
    body: draft?.body ?? "",
    category: draft && CATEGORIES.includes(draft.category) ? draft.category : "general",
    scope: draft?.audience_scope === "branches" ? "branches" : "org",
    branchIds: draft?.audience_branches.map((branch) => branch.id) ?? [],
  };
}

function restoreForm(storageKey: string, draft: BoardNotice | undefined): ComposerForm {
  try {
    const stored = window.sessionStorage.getItem(storageKey);
    if (stored) {
      const parsed = JSON.parse(stored) as Partial<ComposerForm>;
      const seeded = seedForm(draft);
      return {
        title: typeof parsed.title === "string" ? parsed.title : seeded.title,
        body: typeof parsed.body === "string" ? parsed.body : seeded.body,
        category: typeof parsed.category === "string" && CATEGORIES.includes(parsed.category)
          ? parsed.category
          : seeded.category,
        scope: parsed.scope === "branches" || parsed.scope === "org" ? parsed.scope : seeded.scope,
        branchIds: Array.isArray(parsed.branchIds)
          ? parsed.branchIds.filter((id): id is string => typeof id === "string")
          : seeded.branchIds,
      };
    }
  } catch {
    // Corrupt/unavailable storage falls back to the server draft seed.
  }
  return seedForm(draft);
}

type Props = {
  boardApi: BoardApi;
  /** Present = edit an existing server draft; absent = new draft. */
  draft?: BoardNotice;
  /** Unsaved-field survival across refresh, scoped to org:user:draft. */
  storageKey: string;
  onClose: () => void;
  onSaved: (notice: BoardNotice) => void;
};

/** Draft composer — the publish flow's scope selection happens here. */
export function BoardComposer({ boardApi, draft, storageKey, onClose, onSaved }: Props) {
  const [form, setForm] = useState<ComposerForm>(() => restoreForm(storageKey, draft));
  const [branches, setBranches] = useState<BranchSummary[]>();
  const [branchesError, setBranchesError] = useState(false);
  const [branchesAttempt, setBranchesAttempt] = useState(0);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string>();
  const titleId = useId();
  const bodyId = useId();
  const categoryId = useId();
  const dialogLabelId = useId();
  const abortRef = useRef<AbortController | undefined>(undefined);
  const titleRef = useRef<HTMLInputElement>(null);

  // Move focus onto the title input on open (the overlay Escape handler needs
  // focus inside the dialog) and hand it back to the opener on close. Explicit
  // (not autoFocus) so the opener is captured before focus moves.
  useEffect(() => {
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    titleRef.current?.focus();
    return () => {
      opener?.focus();
    };
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    boardApi
      .listBranches(controller.signal)
      .then((all) => {
        if (!controller.signal.aborted) {
          setBranches(all.filter((branch) => branch.deactivated_at === null));
        }
      })
      .catch(() => {
        if (!controller.signal.aborted) setBranchesError(true);
      });
    return () => {
      controller.abort();
    };
  }, [boardApi, branchesAttempt]);

  useEffect(() => () => {
    abortRef.current?.abort();
  }, []);

  const update = useCallback((next: Partial<ComposerForm>) => {
    setForm((current) => {
      const merged = { ...current, ...next };
      try {
        window.sessionStorage.setItem(storageKey, JSON.stringify(merged));
      } catch {
        // Storage-quota/unavailable: the server draft remains the durable copy.
      }
      return merged;
    });
  }, [storageKey]);

  const toggleBranch = (id: string) => {
    update({
      branchIds: form.branchIds.includes(id)
        ? form.branchIds.filter((existing) => existing !== id)
        : [...form.branchIds, id],
    });
  };

  const validationError = !form.title.trim()
    ? text.composer.titleRequired
    : !form.body.trim()
      ? text.composer.bodyRequired
      : form.scope === "branches" && form.branchIds.length === 0
        ? text.composer.branchesRequired
        : undefined;

  const save = async () => {
    if (saving) return;
    if (validationError) {
      setError(validationError);
      return;
    }
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;
    setSaving(true);
    setError(undefined);
    const input = {
      title: form.title.trim(),
      body: form.body,
      category: form.category,
      audience: form.scope === "branches"
        ? { scope: "branches" as const, branch_ids: form.branchIds }
        : { scope: "org" as const },
    };
    try {
      const saved = draft
        ? await boardApi.updateDraft(draft.id, input, controller.signal)
        : await boardApi.createDraft(input, controller.signal);
      try {
        window.sessionStorage.removeItem(storageKey);
      } catch {
        // Best-effort cleanup only.
      }
      onSaved(saved);
    } catch (cause) {
      if (!controller.signal.aborted) {
        setError(cause instanceof Error ? cause.message : text.composer.saveError);
        setSaving(false);
      }
    }
  };

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Escape") {
      event.stopPropagation();
      onClose();
    }
  };

  return (
    <div className="board-overlay" onKeyDown={onKeyDown}>
      <div className="board-panel" role="dialog" aria-modal="true" aria-labelledby={dialogLabelId}>
        <header className="board-panel__header">
          <h3 id={dialogLabelId} className="board-panel__title">
            {draft ? text.composer.editTitle : text.composer.createTitle}
          </h3>
          <button type="button" className="board-panel__close" onClick={onClose} aria-label={text.composer.close}>
            ×
          </button>
        </header>
        <div className="board-panel__scroll">
          <label className="board-field" htmlFor={titleId}>
            <span className="board-field__label">{text.composer.titleLabel}</span>
            <input
              ref={titleRef}
              id={titleId}
              className="board-field__input"
              value={form.title}
              maxLength={300}
              onChange={(event) => {
                update({ title: event.target.value });
              }}
            />
          </label>
          <label className="board-field" htmlFor={bodyId}>
            <span className="board-field__label">{text.composer.bodyLabel}</span>
            <textarea
              id={bodyId}
              className="board-field__input board-field__input--area"
              value={form.body}
              maxLength={20000}
              onChange={(event) => {
                update({ body: event.target.value });
              }}
            />
          </label>
          <label className="board-field" htmlFor={categoryId}>
            <span className="board-field__label">{text.composer.categoryLabel}</span>
            <select
              id={categoryId}
              className="board-field__input"
              value={form.category}
              onChange={(event) => {
                update({ category: event.target.value as NoticeCategory });
              }}
            >
              {CATEGORIES.map((category) => (
                <option key={category} value={category}>
                  {text.category[category]}
                </option>
              ))}
            </select>
          </label>
          <fieldset className="board-field board-field--group">
            <legend className="board-field__label">{text.composer.audienceLabel}</legend>
            <label className="board-choice">
              <input
                type="radio"
                name="board-audience-scope"
                checked={form.scope === "org"}
                onChange={() => {
                  update({ scope: "org" });
                }}
              />
              <span>{text.composer.scopeOrg}</span>
            </label>
            <label className="board-choice">
              <input
                type="radio"
                name="board-audience-scope"
                checked={form.scope === "branches"}
                onChange={() => {
                  update({ scope: "branches" });
                }}
              />
              <span>{text.composer.scopeBranches}</span>
            </label>
            {form.scope === "branches" ? (
              branchesError ? (
                <div className="board-inline-alert" role="alert">
                  <span>{text.composer.branchesError}</span>
                  <button
                    type="button"
                    className="board-btn board-btn--ghost"
                    onClick={() => {
                      setBranchesError(false);
                      setBranches(undefined);
                      setBranchesAttempt((attempt) => attempt + 1);
                    }}
                  >
                    {text.composer.retry}
                  </button>
                </div>
              ) : branches === undefined ? (
                <p className="board-status" role="status">{text.composer.branchesLoading}</p>
              ) : (
                <div className="board-branches">
                  {branches.map((branch) => (
                    <label key={branch.id} className="board-choice">
                      <input
                        type="checkbox"
                        checked={form.branchIds.includes(branch.id)}
                        onChange={() => {
                          toggleBranch(branch.id);
                        }}
                      />
                      <span>{branch.name}</span>
                    </label>
                  ))}
                </div>
              )
            ) : null}
          </fieldset>
          {error ? <div className="board-inline-alert" role="alert"><span>{error}</span></div> : null}
        </div>
        <footer className="board-panel__footer">
          <button type="button" className="board-btn board-btn--ghost" onClick={onClose}>
            {text.composer.close}
          </button>
          <button type="button" className="board-btn" disabled={saving} onClick={() => void save()}>
            {text.composer.save}
          </button>
        </footer>
      </div>
    </div>
  );
}
