import { useCallback, useEffect, useId, useRef, useState, type KeyboardEvent } from "react";

import { boardStrings as text } from "../../i18n/board";
import { timestampLabel } from "./boardModuleConfig";
import type { BoardApi, BoardNotice, NoticeProgress, NoticeReceipt } from "./boardApi";

const PAGE_SIZE = 50;

type ReceiptFilter = "all" | "acked" | "pending";

function acknowledgedParam(filter: ReceiptFilter): boolean | undefined {
  if (filter === "acked") return true;
  if (filter === "pending") return false;
  return undefined;
}

type Props = {
  boardApi: BoardApi;
  notice: BoardNotice;
  onClose: () => void;
};

/** Manager-only receipts drill — the 직원 1:N history layer + chase list. */
export function BoardReceiptsPanel({ boardApi, notice, onClose }: Props) {
  const [filter, setFilter] = useState<ReceiptFilter>("all");
  const [items, setItems] = useState<NoticeReceipt[]>([]);
  const [total, setTotal] = useState(0);
  const [progress, setProgress] = useState<NoticeProgress>();
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [error, setError] = useState<string>();
  const [attempt, setAttempt] = useState(0);
  const abortRef = useRef<AbortController | undefined>(undefined);
  const dialogLabelId = useId();
  const panelRef = useRef<HTMLDivElement>(null);

  // Move focus into the dialog on open (Escape is handled on the overlay, so it
  // must contain focus) and hand it back to the opener on close.
  useEffect(() => {
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    panelRef.current?.focus();
    return () => {
      opener?.focus();
    };
  }, []);

  useEffect(() => {
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;
    void Promise.all([
      boardApi.progress(notice.id, controller.signal),
      boardApi.receipts(
        notice.id,
        { acknowledged: acknowledgedParam(filter), limit: PAGE_SIZE, offset: 0 },
        controller.signal,
      ),
    ])
      .then(([nextProgress, page]) => {
        if (controller.signal.aborted) return;
        setProgress(nextProgress);
        setItems(page.items);
        setTotal(page.total);
        setLoading(false);
      })
      .catch((cause: unknown) => {
        if (controller.signal.aborted) return;
        setError(cause instanceof Error ? cause.message : text.receipts.error);
        setLoading(false);
      });
    return () => {
      controller.abort();
    };
  }, [boardApi, notice.id, filter, attempt]);

  const loadMore = useCallback(async () => {
    if (loadingMore) return;
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;
    setLoadingMore(true);
    try {
      const page = await boardApi.receipts(
        notice.id,
        { acknowledged: acknowledgedParam(filter), limit: PAGE_SIZE, offset: items.length },
        controller.signal,
      );
      if (controller.signal.aborted) return;
      setItems((current) => [...current, ...page.items]);
      setTotal(page.total);
    } catch (cause) {
      if (!controller.signal.aborted) {
        setError(cause instanceof Error ? cause.message : text.receipts.error);
      }
    } finally {
      if (!controller.signal.aborted) setLoadingMore(false);
    }
  }, [boardApi, filter, items.length, loadingMore, notice.id]);

  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Escape") {
      event.stopPropagation();
      onClose();
    }
  };

  const done = progress?.acknowledged ?? 0;
  const progressTotal = progress?.total ?? 0;
  const pct = Math.round((done / Math.max(1, progressTotal)) * 100);
  const filters: { key: ReceiptFilter; label: string }[] = [
    { key: "all", label: text.receipts.filterAll },
    { key: "acked", label: text.receipts.filterAcked },
    { key: "pending", label: text.receipts.filterPending },
  ];

  return (
    <div className="board-overlay" onKeyDown={onKeyDown}>
      <div
        ref={panelRef}
        tabIndex={-1}
        className="board-panel"
        role="dialog"
        aria-modal="true"
        aria-labelledby={dialogLabelId}
      >
        <header className="board-panel__header">
          <h3 id={dialogLabelId} className="board-panel__title">
            <span className="board-code">{notice.code ?? "—"}</span> {text.receipts.title}
          </h3>
          <button type="button" className="board-panel__close" onClick={onClose} aria-label={text.receipts.close}>
            ×
          </button>
        </header>
        {progress ? (
          <div className="board-prog" data-fidelity="board-receipts-prog">
            <span className="board-prog__label">{text.receipts.progressLabel}</span>
            <div className="board-prog__track">
              <div
                className={pct >= 100 ? "board-prog__fill board-prog__fill--ok" : "board-prog__fill board-prog__fill--warn"}
                style={{ width: `${String(Math.min(100, pct))}%` }}
              />
            </div>
            <span className={pct >= 100 ? "board-prog__value board-prog__value--ok" : "board-prog__value board-prog__value--warn"}>
              {`${done.toLocaleString("ko-KR")} / ${progressTotal.toLocaleString("ko-KR")} (${String(pct)}%)`}
            </span>
          </div>
        ) : null}
        <div className="board-filters">
          {filters.map((entry) => (
            <button
              key={entry.key}
              type="button"
              className={entry.key === filter ? "board-chip board-chip--active" : "board-chip"}
              aria-pressed={entry.key === filter}
              onClick={() => {
                if (entry.key === filter) return;
                setLoading(true);
                setError(undefined);
                setItems([]);
                setFilter(entry.key);
              }}
            >
              {entry.label}
            </button>
          ))}
        </div>
        <div className="board-panel__scroll">
          {loading ? (
            <p className="board-status" role="status">{text.receipts.loading}</p>
          ) : error ? (
            <div className="board-inline-alert" role="alert">
              <span>{error}</span>
              <button
                type="button"
                className="board-btn board-btn--ghost"
                onClick={() => {
                  setLoading(true);
                  setError(undefined);
                  setAttempt((current) => current + 1);
                }}
              >
                {text.receipts.retry}
              </button>
            </div>
          ) : items.length === 0 ? (
            <p className="board-status" role="status">
              {filter === "pending" ? text.receipts.emptyPending : text.receipts.empty}
            </p>
          ) : (
            <ul className="board-receipts" aria-label={text.receipts.title}>
              {items.map((receipt) => (
                <li key={receipt.recipient_user_id} className="board-receipts__row">
                  <span className="board-receipts__name">{receipt.display_name}</span>
                  {receipt.acknowledged_at ? (
                    <span className="board-chip board-chip--ok">
                      {`${text.receipts.ackedChip} · ${timestampLabel(receipt.acknowledged_at)}`}
                    </span>
                  ) : (
                    <span className="board-chip board-chip--warn">{text.receipts.pendingChip}</span>
                  )}
                </li>
              ))}
            </ul>
          )}
          {!loading && !error && items.length < total ? (
            <button type="button" className="board-btn board-btn--ghost" disabled={loadingMore} onClick={() => void loadMore()}>
              {text.receipts.more}
            </button>
          ) : null}
        </div>
      </div>
    </div>
  );
}
