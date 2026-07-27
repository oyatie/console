import { useCallback, useEffect, useId, useMemo, useRef, useState, type KeyboardEvent } from "react";

import type { ConsoleApiClient } from "../../api/client";
import { recruitingStrings as text } from "../../i18n/recruiting";
import { CandidateCard } from "./CandidateCard";
import { PostingComposer } from "./PostingComposer";
import { PreflightModal, type PreflightState } from "./PreflightModal";
import {
  createRecruitingApi,
  isConflict,
  isDenied,
  RecruitingApiError,
  type CreateRecruitPostingRequest,
  type HireRecruitApplicantRequest,
  type RecruitApplicantDetailResponse,
  type RecruitApplicantRouting,
  type RecruitApplicantRow,
  type RecruitPostingRow,
  type RecruitPostingView,
  type RecruitTalentPoolItem,
} from "./recruitingApi";
import type { RecruitingCapabilities } from "./recruitingCapabilities";
import {
  STAGE_ORDER,
  applicantStatusLine,
  dateTimeLabel,
  deadlineLabel,
  headStatLine,
  rejectReasonLabel,
  scoreLabel,
  stageLabel,
} from "./recruitingFormat";
import "./recruiting.css";

type Props = {
  api: ConsoleApiClient;
  actorId: string | undefined;
  capabilities: RecruitingCapabilities;
  /** Changes whenever auth replaces the effective tenant/session. */
  sessionKey: string | undefined;
  /** Cross-screen object traversal (people, object explorer). */
  onNavigate?: (path: string) => void;
};

const apiFenceIds = new WeakMap<object, number>();
let nextApiFenceId = 1;

function apiFenceKey(api: ConsoleApiClient): number {
  const reference = api as object;
  const existing = apiFenceIds.get(reference);
  if (existing) return existing;
  const id = nextApiFenceId++;
  apiFenceIds.set(reference, id);
  return id;
}

function messageOf(cause: unknown): string {
  if (isConflict(cause)) return text.conflict;
  if (cause instanceof Error && cause.message) return cause.message;
  return text.actionError;
}

const STAGE_KEYS = ["applied", "screening", "interview", "offer"] as const;

function formText(data: FormData, name: string): string {
  const value = data.get(name);
  return typeof value === "string" ? value : "";
}

/** Pure — the subrow's one action-driving label per pipeline state. */
function nextActionLabel(
  posting: RecruitPostingRow,
  applicant: RecruitApplicantRow,
  capabilities: RecruitingCapabilities,
): string | undefined {
  if (!capabilities.canManage) return undefined;
  if ((applicant.rejected_at !== null)) return text.reconsider;
  if (applicant.stage === "APPLIED" || applicant.stage === "SCREENING") {
    return text.advanceTo(stageLabel(STAGE_ORDER[STAGE_ORDER.indexOf(applicant.stage) + 1]));
  }
  if (applicant.stage === "INTERVIEW") return text.advanceTo(stageLabel("OFFER"));
  if (applicant.stage === "OFFER" && posting.employment_type !== "POOL_DAILY" && capabilities.canHire) {
    return text.card.ctaHire;
  }
  return undefined;
}

interface Branch {
  id: string;
  name: string;
}

/**
 * Re-mount synchronously whenever effective authority changes: effects run too
 * late to fence an old tenant/session's selection, drafts, or busy state.
 */
export function RecruitingScreen(props: Props) {
  const capabilityKey = [props.capabilities.canRead, props.capabilities.canManage, props.capabilities.canHire].join(":");
  const sessionFence = [
    props.sessionKey ?? "no-session",
    props.actorId ?? "no-actor",
    String(apiFenceKey(props.api)),
    capabilityKey,
  ].join(":");
  return <RecruitingScreenImpl key={sessionFence} {...props} />;
}

function RecruitingScreenImpl({ api, capabilities, onNavigate }: Props) {
  const recruitingApi = useMemo(() => createRecruitingApi(api), [api]);
  const [postings, setPostings] = useState<RecruitPostingRow[]>([]);
  const [listState, setListState] = useState<"loading" | "ready" | "error">("loading");
  const [denied, setDenied] = useState(!capabilities.canRead);
  const [talent, setTalent] = useState<RecruitTalentPoolItem[]>();
  const [talentVisible, setTalentVisible] = useState(true);
  const [openId, setOpenId] = useState<string>();
  const [applicants, setApplicants] = useState<RecruitApplicantRow[]>();
  const [openState, setOpenState] = useState<"loading" | "ready" | "error">("ready");
  const [card, setCard] = useState<{ applicantId: string; postingId: string }>();
  const [cardDetail, setCardDetail] = useState<RecruitApplicantDetailResponse>();
  const [cardState, setCardState] = useState<"loading" | "ready" | "error">("ready");
  const [cardError, setCardError] = useState<string>();
  const [composer, setComposer] = useState<{ posting?: RecruitPostingRow }>();
  const [composerError, setComposerError] = useState<string>();
  const [preflight, setPreflight] = useState<PreflightState>();
  const [menuFor, setMenuFor] = useState<string>();
  const [applicantFormFor, setApplicantFormFor] = useState<string>();
  const [branches, setBranches] = useState<Branch[]>();
  const [branchesError, setBranchesError] = useState(false);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string>();
  const [toast, setToast] = useState<string>();
  const generation = useRef(0);
  const operation = useRef<AbortController>(undefined);
  const toastTimer = useRef<number>(undefined);
  const rowRefs = useRef(new Map<string, HTMLButtonElement>());
  const applicantNameId = useId();
  const applicantProfileId = useId();
  const applicantSourceId = useId();
  const headingId = useId();

  const isCurrent = useCallback((token: number) => generation.current === token, []);

  const showToast = useCallback((message: string) => {
    if (toastTimer.current !== undefined) window.clearTimeout(toastTimer.current);
    setToast(message);
    toastTimer.current = window.setTimeout(() => {
      setToast(undefined);
      toastTimer.current = undefined;
    }, 5000);
  }, []);

  useEffect(() => () => {
    if (toastTimer.current !== undefined) window.clearTimeout(toastTimer.current);
    operation.current?.abort();
  }, []);

  /** One server-reconciled read of everything currently on screen. */
  const reload = useCallback(async (context?: { openId?: string; card?: { applicantId: string } }) => {
    if (!capabilities.canRead) {
      setDenied(true);
      return;
    }
    operation.current?.abort();
    const controller = new AbortController();
    operation.current = controller;
    const token = ++generation.current;
    const targetOpen = context?.openId;
    const targetCard = context?.card;
    setListState((current) => (current === "ready" ? current : "loading"));
    try {
      const [listResult, talentResult, openResult, cardResult] = await Promise.allSettled([
        recruitingApi.listPostings(undefined, controller.signal),
        recruitingApi.listTalentPool(controller.signal),
        targetOpen !== undefined ? recruitingApi.getPosting(targetOpen, controller.signal) : Promise.resolve(undefined),
        targetCard !== undefined ? recruitingApi.getApplicant(targetCard.applicantId, controller.signal) : Promise.resolve(undefined),
      ]);
      if (!isCurrent(token)) return;
      if (listResult.status === "rejected") {
        if (isDenied(listResult.reason)) {
          setDenied(true);
          return;
        }
        if (!controller.signal.aborted) setListState("error");
        return;
      }
      setPostings(listResult.value.items);
      setListState("ready");
      if (talentResult.status === "fulfilled") {
        setTalent(talentResult.value.items);
        setTalentVisible(true);
      } else if (isDenied(talentResult.reason)) {
        // Deny-by-omission: the talent pool section renders nothing at all.
        setTalentVisible(false);
      } else {
        setTalent(undefined);
      }
      if (targetOpen !== undefined) {
        if (openResult.status === "fulfilled" && openResult.value !== undefined) {
          setApplicants(openResult.value.applicants);
          setOpenState("ready");
        } else {
          setApplicants(undefined);
          setOpenState("error");
        }
      }
      if (targetCard !== undefined) {
        if (cardResult.status === "fulfilled" && cardResult.value !== undefined) {
          setCardDetail(cardResult.value);
          setCardState("ready");
        } else {
          setCardState("error");
        }
      }
    } catch {
      if (isCurrent(token) && !controller.signal.aborted) setListState("error");
    }
  }, [capabilities.canRead, isCurrent, recruitingApi]);

  useEffect(() => {
    if (!capabilities.canRead) return;
    const start = window.setTimeout(() => {
      void reload();
    }, 0);
    return () => {
      window.clearTimeout(start);
    };
  }, [capabilities.canRead, reload]);

  const loadOpenPosting = useCallback((postingId: string) => {
    setOpenId(postingId);
    setApplicants(undefined);
    setOpenState("loading");
    const token = ++generation.current;
    void recruitingApi.getPosting(postingId).then(
      (detail) => {
        if (!isCurrent(token)) return;
        setApplicants(detail.applicants);
        setOpenState("ready");
        setPostings((current) => current.map((posting) => (posting.id === detail.posting.id ? { ...posting, ...detail.posting } : posting)));
      },
      (cause: unknown) => {
        if (!isCurrent(token)) return;
        if (isDenied(cause)) setDenied(true);
        else setOpenState("error");
      },
    );
  }, [isCurrent, recruitingApi]);

  const togglePosting = useCallback((postingId: string) => {
    setMenuFor(undefined);
    setApplicantFormFor(undefined);
    if (openId === postingId) {
      setOpenId(undefined);
      setApplicants(undefined);
      return;
    }
    loadOpenPosting(postingId);
  }, [loadOpenPosting, openId]);

  const openCard = useCallback((postingId: string, applicantId: string) => {
    setCard({ applicantId, postingId });
    setCardDetail(undefined);
    setCardError(undefined);
    setCardState("loading");
    setMenuFor(undefined);
    const token = ++generation.current;
    void recruitingApi.getApplicant(applicantId).then(
      (detail) => {
        if (!isCurrent(token)) return;
        setCardDetail(detail);
        setCardState("ready");
      },
      (cause: unknown) => {
        if (!isCurrent(token)) return;
        if (isDenied(cause)) setDenied(true);
        else setCardState("error");
      },
    );
  }, [isCurrent, recruitingApi]);

  const closeCard = useCallback(() => {
    setCard(undefined);
    setCardDetail(undefined);
    setCardError(undefined);
  }, []);

  /** Run a mutation, then re-read every surface from the server.
   * Resolves `{ value }` on success (even for void endpoints), `undefined` on failure. */
  const runMutation = useCallback(async <T,>(
    work: (signal: AbortSignal) => Promise<T>,
    options: { toast?: (value: T) => string; onError?: (message: string, cause: unknown) => void } = {},
  ): Promise<{ value: T } | undefined> => {
    if (busy) return undefined;
    const controller = new AbortController();
    const token = ++generation.current;
    setBusy(true);
    setActionError(undefined);
    setCardError(undefined);
    try {
      const value = await work(controller.signal);
      if (!isCurrent(token)) return undefined;
      if (options.toast) showToast(options.toast(value));
      await reload({ openId, card });
      return { value };
    } catch (cause) {
      if (!isCurrent(token) || controller.signal.aborted) return undefined;
      if (isDenied(cause)) {
        setDenied(true);
        return undefined;
      }
      const message = messageOf(cause);
      if (options.onError) options.onError(message, cause);
      else if (card) setCardError(message);
      else setActionError(message);
      if (isConflict(cause)) void reload({ openId, card });
      return undefined;
    } finally {
      // runMutation never runs concurrently (busy guard), so this is always ours.
      setBusy(false);
    }
  }, [busy, card, isCurrent, openId, reload, showToast]);

  // ---- posting actions -----------------------------------------------------

  const openPreflight = useCallback((posting: RecruitPostingView) => {
    setPreflight({ postingId: posting.id, roleTitle: posting.role_title, state: "loading", checks: [], publishable: false, attest: false });
    const token = ++generation.current;
    void recruitingApi.preflightPosting(posting.id).then(
      (result) => {
        if (!isCurrent(token)) return;
        setPreflight((current) => current && current.postingId === posting.id
          ? { ...current, state: "ready", checks: result.checks, publishable: result.publishable }
          : current);
      },
      (cause: unknown) => {
        if (!isCurrent(token)) return;
        if (isDenied(cause)) setDenied(true);
        else setPreflight((current) => current && current.postingId === posting.id ? { ...current, state: "error" } : current);
      },
    );
  }, [isCurrent, recruitingApi]);

  const publishFromPreflight = useCallback(() => {
    if (!preflight) return;
    const posting = postings.find((item) => item.id === preflight.postingId);
    if (!posting) return;
    void runMutation(
      (signal) => recruitingApi.publishPosting(posting.id, { attest_exposure_scope: true, expected_updated_at: posting.updated_at }, signal),
      {
        toast: () => text.toast.published(posting.role_title),
        onError: (message, cause) => {
          const checks = cause instanceof RecruitingApiError ? cause.checks : undefined;
          setPreflight((current) => current && current.postingId === posting.id
            ? { ...current, error: message, ...(checks ? { checks, publishable: false } : {}) }
            : current);
        },
      },
    ).then((outcome) => {
      if (outcome !== undefined) setPreflight(undefined);
    });
  }, [postings, preflight, recruitingApi, runMutation]);

  const saveComposer = useCallback((input: CreateRecruitPostingRequest, publish: boolean) => {
    const editing = composer?.posting;
    setComposerError(undefined);
    void runMutation(
      (signal) => editing
        ? recruitingApi.updatePosting(editing.id, { ...input, expected_updated_at: editing.updated_at }, signal)
        : recruitingApi.createPosting(input, signal),
      {
        toast: (posting) => (editing ? text.toast.draftUpdated(posting.role_title) : text.toast.draftSaved(posting.role_title)),
        onError: (message) => { setComposerError(message); },
      },
    ).then((outcome) => {
      if (outcome === undefined) return;
      setComposer(undefined);
      // Publish is only reachable through the §4-29 preflight gate.
      if (publish) openPreflight(outcome.value);
    });
  }, [composer, openPreflight, recruitingApi, runMutation]);

  const closePosting = useCallback((posting: RecruitPostingView) => {
    void runMutation(
      (signal) => recruitingApi.closePosting(posting.id, { expected_updated_at: posting.updated_at }, signal),
      { toast: () => text.toast.closed(posting.role_title) },
    );
  }, [recruitingApi, runMutation]);

  const registerApplicant = useCallback((postingId: string, name: string, profile: string, sourceDocument: string) => {
    const profileLines = profile.split("\n").map((line) => line.trim()).filter((line) => line !== "");
    void runMutation(
      (signal) => recruitingApi.createApplicant(postingId, {
        name,
        profile_lines: profileLines,
        ...(sourceDocument ? { source_document: sourceDocument } : {}),
      }, signal),
      { toast: (applicant) => text.toast.applicantCreated(applicant.name) },
    ).then((outcome) => {
      if (outcome !== undefined) setApplicantFormFor(undefined);
    });
  }, [recruitingApi, runMutation]);

  // ---- applicant actions ---------------------------------------------------

  const advanceApplicant = useCallback((applicant: RecruitApplicantRouting) => {
    const stageIndex = STAGE_ORDER.indexOf(applicant.stage);
    const next = STAGE_ORDER[stageIndex + 1] as (typeof STAGE_ORDER)[number] | undefined;
    void runMutation(
      (signal) => recruitingApi.advanceApplicant(applicant.id, { expected_updated_at: applicant.updated_at }, signal),
      { toast: () => text.toast.advanced(applicant.name, next !== undefined ? stageLabel(next) : stageLabel(applicant.stage)) },
    );
  }, [recruitingApi, runMutation]);

  const holdApplicant = useCallback((applicant: RecruitApplicantRouting) => {
    setMenuFor(undefined);
    void runMutation(
      (signal) => recruitingApi.holdApplicant(applicant.id, { hold: !applicant.hold }, signal),
      { toast: () => (applicant.hold ? text.toast.released(applicant.name) : text.toast.held(applicant.name)) },
    );
  }, [recruitingApi, runMutation]);

  const requestDocuments = useCallback((applicant: RecruitApplicantRouting) => {
    setMenuFor(undefined);
    void runMutation(
      (signal) => recruitingApi.requestDocuments(applicant.id, signal),
      { toast: () => text.toast.documentsRequested(applicant.name) },
    );
  }, [recruitingApi, runMutation]);

  const reinstateApplicant = useCallback((applicant: RecruitApplicantRouting) => {
    void runMutation(
      (signal) => recruitingApi.reinstateApplicant(applicant.id, signal),
      { toast: () => text.toast.reinstated(applicant.name) },
    );
  }, [recruitingApi, runMutation]);

  /** The subrow next-action counterpart of {@link nextActionLabel}. */
  const runNextAction = useCallback((posting: RecruitPostingView, applicant: RecruitApplicantRouting) => {
    if ((applicant.rejected_at !== null)) {
      reinstateApplicant(applicant);
      return;
    }
    if (applicant.stage === "APPLIED" || applicant.stage === "SCREENING") {
      advanceApplicant(applicant);
      return;
    }
    // INTERVIEW (offer-only advance) and OFFER (hire) resolve inside the card.
    openCard(posting.id, applicant.id);
  }, [advanceApplicant, openCard, reinstateApplicant]);

  const focusPostingRow = useCallback((postingId: string) => {
    closeCard();
    if (openId !== postingId) togglePosting(postingId);
    queueMicrotask(() => rowRefs.current.get(postingId)?.focus());
  }, [closeCard, openId, togglePosting]);

  const loadBranches = useCallback(() => {
    setBranchesError(false);
    const token = ++generation.current;
    void recruitingApi.listBranches().then(
      (items) => {
        if (isCurrent(token)) setBranches(items.map((branch) => ({ id: branch.id, name: branch.name })));
      },
      () => {
        if (isCurrent(token)) setBranchesError(true);
      },
    );
  }, [isCurrent, recruitingApi]);

  const rove = useCallback((event: KeyboardEvent<HTMLButtonElement>, index: number) => {
    const keys: Record<string, number> = {
      j: Math.min(index + 1, postings.length - 1),
      J: Math.min(index + 1, postings.length - 1),
      ArrowDown: Math.min(index + 1, postings.length - 1),
      k: Math.max(index - 1, 0),
      K: Math.max(index - 1, 0),
      ArrowUp: Math.max(index - 1, 0),
      Home: 0,
      End: postings.length - 1,
    };
    if (!(event.key in keys)) return;
    event.preventDefault();
    const target = postings[keys[event.key]] as RecruitPostingRow | undefined;
    if (target) rowRefs.current.get(target.id)?.focus();
  }, [postings]);

  // ---- render --------------------------------------------------------------

  if (denied) {
    return (
      <div className="recruiting recruiting--denied">
        <h1 className="recruiting__title">{text.title}</h1>
        <p className="recruiting__state" role="status">{text.denied}</p>
      </div>
    );
  }

  const cardPosting = card ? postings.find((posting) => posting.id === card.postingId) : undefined;

  return (
    <div className="recruiting" aria-busy={busy || listState === "loading"}>
      <header className="recruiting__head">
        <div>
          <h1 id={headingId} className="recruiting__title">{text.title}</h1>
          <p className="recruiting__stat">{headStatLine(postings)}</p>
        </div>
        <span className="recruiting__spacer" />
        {capabilities.canManage && (
          <button type="button" className="recruiting__primary recruiting__primary--signal" onClick={() => { setComposerError(undefined); setComposer({}); }}>
            <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M5 12h14 M12 5v14" /></svg>
            {text.newPosting}
          </button>
        )}
      </header>
      {actionError !== undefined && (
        <div className="recruiting__banner recruiting__banner--danger recruiting__banner--row" role="alert">
          <span className="recruiting__banner-grow">{actionError}</span>
          <button type="button" className="recruiting__ghost" onClick={() => { setActionError(undefined); void reload({ openId, card }); }}>{text.retry}</button>
        </div>
      )}
      <section className="recruiting__panel" aria-labelledby={headingId}>
        <div className="recruiting__panel-head">
          <span className="recruiting__panel-title">{text.listTitle}</span>
        </div>
        <div className="recruiting__scroll">
          <div className="recruiting__grid recruiting__grid--cols" aria-hidden="true">
            <span>{text.colPosting}</span>
            <span>{text.colFill}</span>
            <span>{text.colStages}</span>
            <span className="recruiting__right">{text.colDeadline}</span>
            <span />
          </div>
          {listState === "loading" && <p className="recruiting__state" role="status">{text.loading}</p>}
          {listState === "error" && (
            <div className="recruiting__banner recruiting__banner--danger recruiting__banner--row" role="alert">
              <span className="recruiting__banner-grow">{text.loadError}</span>
              <button type="button" className="recruiting__ghost" onClick={() => { void reload({ openId, card }); }}>{text.retry}</button>
            </div>
          )}
          {listState === "ready" && postings.length === 0 && <p className="recruiting__state" role="status">{text.empty}</p>}
          <ul className="recruiting__list">
            {postings.map((posting, index) => {
              const open = openId === posting.id;
              const fillPct = Math.min(100, posting.headcount > 0 ? Math.round((posting.hired_count / posting.headcount) * 100) : 0);
              return (
                <li key={posting.id}>
                  <div
                    className={open ? "recruiting__grid recruiting__row recruiting__row--open" : "recruiting__grid recruiting__row"}
                    onClick={() => { togglePosting(posting.id); }}
                  >
                    <span className="recruiting__row-main">
                      <span className="recruiting__row-line">
                        <button
                          type="button"
                          ref={(node) => { if (node) rowRefs.current.set(posting.id, node); else rowRefs.current.delete(posting.id); }}
                          className="recruiting__row-toggle"
                          aria-expanded={open}
                          title={text.rowHint}
                          onKeyDown={(event) => { rove(event, index); }}
                          onClick={(event) => { event.stopPropagation(); togglePosting(posting.id); }}
                        >{posting.role_title}</button>
                        <span className="recruiting__chip recruiting__chip--muted">{posting.company}</span>
                        {posting.scope === "INTERNAL" && <span className="recruiting__chip recruiting__chip--purple">{text.internalChip}</span>}
                        {posting.status === "DRAFT" && capabilities.canManage && (
                          <button
                            type="button"
                            className="recruiting__chip recruiting__chip--warn recruiting__chip--action"
                            title={text.draftPublishHint}
                            onClick={(event) => { event.stopPropagation(); openPreflight(posting); }}
                          >{text.draftPublish}</button>
                        )}
                        {posting.status === "DRAFT" && !capabilities.canManage && (
                          <span className="recruiting__chip recruiting__chip--warn">{text.postingStatus.DRAFT}</span>
                        )}
                        {posting.status === "CLOSED" && <span className="recruiting__chip recruiting__chip--muted">{text.postingStatus.CLOSED}</span>}
                      </span>
                      <span className="recruiting__row-site">{posting.worksite}</span>
                    </span>
                    <span className="recruiting__fill">
                      <span className="recruiting__fill-track">
                        <span
                          className={posting.hired_count >= posting.headcount ? "recruiting__fill-bar recruiting__fill-bar--done" : "recruiting__fill-bar"}
                          style={{ width: `${String(fillPct)}%` }}
                        />
                      </span>
                      <span className="recruiting__mono">{String(posting.hired_count)} / {String(posting.headcount)}</span>
                    </span>
                    <span className="recruiting__stages">
                      {STAGE_KEYS.map((key, stageIdx) => {
                        const count = posting.stage_counts[key];
                        return (
                          <span key={key} className={count > 0 ? "recruiting__stage-chip" : "recruiting__stage-chip recruiting__stage-chip--dim"}>
                            <span>{stageLabel(STAGE_ORDER[stageIdx])}</span>
                            <span className="recruiting__mono">{String(count)}</span>
                          </span>
                        );
                      })}
                    </span>
                    <span className={posting.deadline === null ? "recruiting__mono recruiting__due recruiting__due--open" : "recruiting__mono recruiting__due"}>
                      {deadlineLabel(posting.deadline)}
                    </span>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="var(--faint)" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="recruiting__chev" aria-hidden="true">
                      <path d={open ? "m18 15-6-6-6 6" : "m6 9 6 6 6-6"} />
                    </svg>
                  </div>
                  {open && (
                    <div className="recruiting__sub">
                      {openState === "loading" && <p className="recruiting__state" role="status">{text.detailLoading}</p>}
                      {openState === "error" && (
                        <div className="recruiting__banner recruiting__banner--danger recruiting__banner--row" role="alert">
                          <span className="recruiting__banner-grow">{text.loadError}</span>
                          <button type="button" className="recruiting__ghost" onClick={() => { loadOpenPosting(posting.id); }}>{text.retry}</button>
                        </div>
                      )}
                      {openState === "ready" && applicants !== undefined && (
                        <>
                          {applicants.length === 0 && <p className="recruiting__state">{text.applicantsEmpty}</p>}
                          {applicants.map((applicant) => {
                            const rejected = (applicant.rejected_at !== null);
                            const stageClass = rejected ? "recruiting__stage recruiting__stage--danger"
                              : applicant.stage === "OFFER" || applicant.stage === "HIRED" ? "recruiting__stage recruiting__stage--ok"
                                : applicant.stage === "INTERVIEW" ? "recruiting__stage recruiting__stage--purple"
                                  : applicant.stage === "SCREENING" ? "recruiting__stage recruiting__stage--info"
                                    : "recruiting__stage";
                            const nextLabel = nextActionLabel(posting, applicant, capabilities);
                            return (
                              <div key={applicant.id} className="recruiting__applicant">
                                <span className={stageClass}>{rejected ? text.rejectedStage : stageLabel(applicant.stage)}</span>
                                <button type="button" className="recruiting__applicant-name" title={text.applicantCardHint} onClick={() => { openCard(posting.id, applicant.id); }}>
                                  {applicant.name}
                                </button>
                                {applicant.hold && <span className="recruiting__chip recruiting__chip--muted">{text.holdChip}</span>}
                                {applicant.doc_requested && <span className="recruiting__chip recruiting__chip--warn">{text.docChip}</span>}
                                <span className="recruiting__applicant-line">{applicantStatusLine(applicant)}</span>
                                {nextLabel !== undefined && (
                                  <button type="button" className="recruiting__ghost" disabled={busy} onClick={() => { runNextAction(posting, applicant); }}>{nextLabel}</button>
                                )}
                                {capabilities.canManage && !rejected && applicant.stage !== "HIRED" && (
                                  <span className="recruiting__menu-anchor">
                                    <button
                                      type="button"
                                      className="recruiting__icon-button recruiting__icon-button--bordered"
                                      title={text.moreActionsHint}
                                      aria-expanded={menuFor === applicant.id}
                                      onClick={() => { setMenuFor((current) => (current === applicant.id ? undefined : applicant.id)); }}
                                    >
                                      <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><circle cx="5" cy="12" r="1.7" /><circle cx="12" cy="12" r="1.7" /><circle cx="19" cy="12" r="1.7" /></svg>
                                    </button>
                                    {menuFor === applicant.id && (
                                      <span className="recruiting__menu" role="menu">
                                        <button type="button" role="menuitem" className="recruiting__menu-item" onClick={() => { requestDocuments(applicant); }}>{text.menuRequestDocuments}</button>
                                        <button type="button" role="menuitem" className="recruiting__menu-item" onClick={() => { holdApplicant(applicant); }}>{applicant.hold ? text.menuRelease : text.menuHold}</button>
                                        <span className="recruiting__menu-rule" aria-hidden="true" />
                                        <button type="button" role="menuitem" className="recruiting__menu-item recruiting__menu-item--danger" onClick={() => { setMenuFor(undefined); openCard(posting.id, applicant.id); }}>{text.menuReject}</button>
                                      </span>
                                    )}
                                  </span>
                                )}
                              </div>
                            );
                          })}
                          {capabilities.canManage && (
                            <div className="recruiting__sub-actions">
                              {applicantFormFor === posting.id ? (
                                <form
                                  className="recruiting__applicant-form"
                                  onSubmit={(event) => {
                                    event.preventDefault();
                                    const data = new FormData(event.currentTarget);
                                    const name = formText(data, "name").trim();
                                    if (!name) return;
                                    registerApplicant(posting.id, name, formText(data, "profile"), formText(data, "source_document").trim());
                                  }}
                                >
                                  <div className="recruiting__field">
                                    <label className="recruiting__field-label" htmlFor={applicantNameId}>{text.applicantForm.name}</label>
                                    <input id={applicantNameId} name="name" className="recruiting__input" required />
                                  </div>
                                  <div className="recruiting__field">
                                    <label className="recruiting__field-label" htmlFor={applicantProfileId}>{text.applicantForm.profile}</label>
                                    <textarea id={applicantProfileId} name="profile" className="recruiting__input recruiting__input--area" rows={3} />
                                  </div>
                                  <div className="recruiting__field">
                                    <label className="recruiting__field-label" htmlFor={applicantSourceId}>{text.applicantForm.sourceDocument}</label>
                                    <input id={applicantSourceId} name="source_document" className="recruiting__input" />
                                  </div>
                                  <div className="recruiting__req-row">
                                    <button type="button" className="recruiting__ghost" onClick={() => { setApplicantFormFor(undefined); }}>{text.applicantForm.cancel}</button>
                                    <span className="recruiting__spacer" />
                                    <button type="submit" className="recruiting__primary" disabled={busy}>{text.applicantForm.submit}</button>
                                  </div>
                                </form>
                              ) : (
                                <button type="button" className="recruiting__ghost" onClick={() => { setApplicantFormFor(posting.id); }}>{text.applicantForm.open}</button>
                              )}
                              {posting.status === "DRAFT" && (
                                <button type="button" className="recruiting__ghost" onClick={() => { setComposerError(undefined); setComposer({ posting }); }}>{text.editDraft}</button>
                              )}
                              {posting.status === "PUBLISHED" && (
                                <button type="button" className="recruiting__ghost recruiting__ghost--danger" disabled={busy} onClick={() => { closePosting(posting); }}>{text.closePosting}</button>
                              )}
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  )}
                </li>
              );
            })}
          </ul>
        </div>
      </section>
      {talentVisible && talent !== undefined && (
        <section className="recruiting__panel" aria-label={text.talentPool.title}>
          <div className="recruiting__panel-head">
            <span className="recruiting__panel-title">{text.talentPool.title}</span>
          </div>
          {talent.length === 0 ? (
            <p className="recruiting__state">{text.talentPool.empty}</p>
          ) : (
            <ul className="recruiting__pool">
              {talent.map((item) => (
                <li key={`${item.applicant_no}-${item.rejected_at}`} className="recruiting__pool-row">
                  <span className="recruiting__chip recruiting__chip--mono">{item.applicant_no}</span>
                  <span className="recruiting__pool-name">{item.name}</span>
                  <span className="recruiting__pool-role">{item.role_title}</span>
                  <span className="recruiting__chip recruiting__chip--danger">{text.talentPool.reasonPrefix}{rejectReasonLabel(item.reason)}</span>
                  <span className="recruiting__mono recruiting__pool-at">{dateTimeLabel(item.rejected_at)}</span>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}
      {menuFor !== undefined && <div className="recruiting__scrim" onClick={() => { setMenuFor(undefined); }} />}
      {composer && (
        <PostingComposer
          posting={composer.posting}
          busy={busy}
          error={composerError}
          onSaveDraft={(input) => { saveComposer(input, false); }}
          onPublish={(input) => { saveComposer(input, true); }}
          onClose={() => { setComposer(undefined); }}
        />
      )}
      {preflight && (
        <PreflightModal
          preflight={preflight}
          busy={busy}
          onToggleAttest={() => { setPreflight((current) => (current ? { ...current, attest: !current.attest } : current)); }}
          onPublish={publishFromPreflight}
          onRetry={() => {
            const posting = postings.find((item) => item.id === preflight.postingId);
            if (posting) openPreflight(posting);
          }}
          onClose={() => { setPreflight(undefined); }}
        />
      )}
      {card && cardState === "loading" && (
        <div className="recruiting__overlay recruiting__overlay--end">
          <div role="dialog" aria-modal="true" aria-label={text.title} className="recruiting__card">
            <p className="recruiting__state" role="status">{text.card.cardLoading}</p>
          </div>
        </div>
      )}
      {card && cardState === "error" && (
        <div className="recruiting__overlay recruiting__overlay--end" onClick={(event) => { if (event.target === event.currentTarget) closeCard(); }} onKeyDown={(event) => { if (event.key === "Escape") closeCard(); }}>
          <div role="dialog" aria-modal="true" aria-label={text.title} className="recruiting__card">
            <div className="recruiting__banner recruiting__banner--danger recruiting__banner--row" role="alert">
              <span className="recruiting__banner-grow">{text.card.cardError}</span>
              <button type="button" className="recruiting__ghost" autoFocus onClick={() => { openCard(card.postingId, card.applicantId); }}>{text.retry}</button>
              <button type="button" className="recruiting__ghost" onClick={closeCard}>{text.card.close}</button>
            </div>
          </div>
        </div>
      )}
      {card && cardState === "ready" && cardDetail !== undefined && cardPosting !== undefined && (
        <CandidateCard
          posting={cardPosting}
          detail={cardDetail}
          capabilities={capabilities}
          busy={busy}
          error={cardError}
          branches={branches}
          branchesError={branchesError}
          onLoadBranches={loadBranches}
          onClose={closeCard}
          onOpenPosting={() => { focusPostingRow(cardPosting.id); }}
          onOpenEmployee={() => { if (onNavigate) onNavigate("/console/people"); }}
          onOpenPosition={() => { if (onNavigate) onNavigate("/console/objectExplorer"); }}
          onAdvance={() => { advanceApplicant(cardDetail.applicant); }}
          onAssess={(score) => {
            void runMutation(
              (signal) => recruitingApi.assessApplicant(cardDetail.applicant.id, { score }, signal),
              { toast: () => text.toast.assessed(cardDetail.applicant.name, scoreLabel(score)) },
            );
          }}
          onSendOffer={(input) => {
            void runMutation(
              (signal) => recruitingApi.extendOffer(cardDetail.applicant.id, input, signal),
              { toast: (offer) => text.toast.offerSent(cardDetail.applicant.name, offer.amount) },
            );
          }}
          onAdjustOffer={(offerId, input) => {
            void runMutation(
              (signal) => recruitingApi.adjustOffer(offerId, input, signal),
              { toast: (offer) => text.toast.offerAdjusted(cardDetail.applicant.name, offer.amount) },
            );
          }}
          onWithdrawOffer={(offerId, reason) => {
            void runMutation(
              (signal) => recruitingApi.withdrawOffer(offerId, { reason }, signal),
              { toast: () => text.toast.offerWithdrawn(cardDetail.applicant.name) },
            );
          }}
          onRecordReply={(offerId, decision) => {
            void runMutation(
              (signal) => recruitingApi.recordOfferReply(offerId, { decision }, signal),
              { toast: () => text.toast.offerReply(cardDetail.applicant.name, decision === "ACCEPTED" ? text.card.offerStatus.ACCEPTED : text.card.offerStatus.DECLINED) },
            );
          }}
          onRequestDocuments={() => { requestDocuments(cardDetail.applicant); }}
          onReject={(reason) => {
            void runMutation(
              (signal) => recruitingApi.rejectApplicant(cardDetail.applicant.id, { reason }, signal),
              { toast: () => text.toast.rejected(cardDetail.applicant.name, rejectReasonLabel(reason)) },
            );
          }}
          onReinstate={() => { reinstateApplicant(cardDetail.applicant); }}
          onHire={(input: HireRecruitApplicantRequest) => {
            void runMutation(
              (signal) => recruitingApi.hireApplicant(cardDetail.applicant.id, input, signal),
              {
                toast: (result) => text.toast.hired(
                  cardDetail.applicant.name,
                  `${String(result.posting.hired_count)} / ${String(result.posting.headcount)}`,
                ),
              },
            );
          }}
          onGuard={(message) => { setCardError(message); }}
        />
      )}
      {toast !== undefined && <div className="recruiting__toast" role="status">{toast}</div>}
    </div>
  );
}
