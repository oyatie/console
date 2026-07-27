import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useAuth } from "../../context/auth";
import { boardStrings as text } from "../../i18n/board";
import type { ConsoleApiClient } from "../../api/client";
import { PolicyGateProvider } from "../policy/PolicyGated";
import { ModuleScreen, type ModuleLoadState } from "../module/ModuleScreen";
import { createBoardApi, type BoardNotice } from "./boardApi";
import { BOARD_ACK_ACTION, NOTICE_MANAGE_FEATURE, deriveBoardCapabilities } from "./boardCapabilities";
import { buildBoardModuleConfig } from "./boardModuleConfig";
import { BoardComposer } from "./BoardComposer";
import { BoardReceiptsPanel } from "./BoardReceiptsPanel";
import { useBoardConsoleAuthz } from "./useBoardConsoleAuthz";
import "../tokens.css";
import "./board.css";

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

interface ComposerState {
  draft?: BoardNotice;
}

/**
 * Re-mount synchronously whenever effective authority changes. Effects run too
 * late to fence an old tenant/session's rows, selection, or overlay state.
 */
export function BoardScreenBody() {
  const { api, session } = useAuth();
  const gate = useBoardConsoleAuthz();
  const capabilities = useMemo(() => deriveBoardCapabilities(gate), [gate]);
  const sessionFence = [
    session?.org_id ?? "no-org",
    session?.user_id ?? "no-actor",
    session?.client_session_incarnation ?? "no-incarnation",
    apiFenceKey(api),
    capabilities.canManage ? "manage" : "read",
  ].join(":");
  return (
    <BoardBody
      key={sessionFence}
      api={api}
      canManage={capabilities.canManage}
      storageScope={`${session?.org_id ?? "no-org"}:${session?.user_id ?? "no-actor"}`}
    />
  );
}

function BoardBody({ api, canManage, storageScope }: {
  api: ConsoleApiClient;
  canManage: boolean;
  storageScope: string;
}) {
  const boardApi = useMemo(() => createBoardApi(api), [api]);
  const [rows, setRows] = useState<BoardNotice[]>([]);
  const [loadState, setLoadState] = useState<ModuleLoadState>("loading");
  const [branchFilter, setBranchFilter] = useState<{ id: string; name: string }>();
  const [composer, setComposer] = useState<ComposerState>();
  const [receiptsFor, setReceiptsFor] = useState<BoardNotice>();
  const [toast, setToast] = useState<string>();
  const [reloadNonce, setReloadNonce] = useState(0);
  const generation = useRef(0);
  const operation = useRef<AbortController | undefined>(undefined);
  const toastTimer = useRef<number | undefined>(undefined);

  useEffect(() => {
    const load = async () => {
      operation.current?.abort();
      const controller = new AbortController();
      operation.current = controller;
      const token = ++generation.current;
      setLoadState("loading");
      try {
        const notices = await boardApi.list(200, controller.signal);
        if (generation.current === token) {
          setRows(notices);
          setLoadState("idle");
        }
      } catch {
        if (generation.current === token && !controller.signal.aborted) {
          setLoadState("error");
        }
      }
    };
    const start = window.setTimeout(() => {
      void load();
    }, 0);
    return () => {
      window.clearTimeout(start);
      operation.current?.abort();
    };
  }, [boardApi, reloadNonce]);

  useEffect(() => () => {
    if (toastTimer.current !== undefined) window.clearTimeout(toastTimer.current);
  }, []);

  const onToast = useCallback((message: string) => {
    if (!message) return;
    if (toastTimer.current !== undefined) window.clearTimeout(toastTimer.current);
    setToast(message);
    toastTimer.current = window.setTimeout(() => {
      setToast(undefined);
      toastTimer.current = undefined;
    }, 3000);
  }, []);

  const onReload = useCallback(() => {
    setReloadNonce((nonce) => nonce + 1);
  }, []);

  const onEditDraft = useCallback((row: BoardNotice) => {
    setComposer({ draft: row });
  }, []);

  const onOpenReceipts = useCallback((row: BoardNotice) => {
    setReceiptsFor(row);
  }, []);

  const config = useMemo(
    () => buildBoardModuleConfig({ canManage, boardApi, onReload, onEditDraft, onOpenReceipts }),
    [canManage, boardApi, onReload, onEditDraft, onOpenReceipts],
  );

  // Audience-branch chip drill: narrow the list to that branch's notices.
  const onOpenObject = useCallback((code: string) => {
    for (const row of rows) {
      const branch = row.audience_branches.find((candidate) => candidate.id === code);
      if (branch) {
        setBranchFilter(branch);
        return;
      }
    }
  }, [rows]);

  const decide = useCallback(
    (action: string) => {
      if (action === NOTICE_MANAGE_FEATURE) return canManage;
      return action === BOARD_ACK_ACTION;
    },
    [canManage],
  );

  const filteredRows = useMemo(
    () => (branchFilter
      ? rows.filter((row) => row.audience_branches.some((branch) => branch.id === branchFilter.id))
      : rows),
    [rows, branchFilter],
  );

  return (
    <div className="board" data-board-root>
      {branchFilter ? (
        <div className="board-filter-bar">
          <button
            type="button"
            className="board-chip board-chip--active"
            aria-label={`${branchFilter.name} ${text.filterClear}`}
            onClick={() => {
              setBranchFilter(undefined);
            }}
          >
            {branchFilter.name} ×
          </button>
        </div>
      ) : null}
      <PolicyGateProvider decide={decide}>
        <ModuleScreen
          config={config}
          rows={filteredRows}
          loadState={loadState}
          api={api}
          onRetry={onReload}
          onOpenObject={onOpenObject}
          onToast={onToast}
          onPrimaryAction={() => {
            setComposer({});
          }}
        />
      </PolicyGateProvider>
      {composer ? (
        <BoardComposer
          boardApi={boardApi}
          draft={composer.draft}
          storageKey={`board-composer:${storageScope}:${composer.draft?.id ?? "new"}`}
          onClose={() => {
            setComposer(undefined);
          }}
          onSaved={() => {
            setComposer(undefined);
            onReload();
            onToast(text.toasts.draftSaved);
          }}
        />
      ) : null}
      {receiptsFor ? (
        <BoardReceiptsPanel
          boardApi={boardApi}
          notice={receiptsFor}
          onClose={() => {
            setReceiptsFor(undefined);
          }}
        />
      ) : null}
      {toast ? <div className="board-toast" role="status">{toast}</div> : null}
    </div>
  );
}
