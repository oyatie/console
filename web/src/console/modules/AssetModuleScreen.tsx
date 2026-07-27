import { useEffect, useState } from "react";

import { useActiveBranchId, useAuth } from "../../context/auth";
import {
  DENY_ALL_PROJECTION,
  makePolicyGate,
  parseAuthzResponse,
  type PolicyGate,
} from "../policy/authz";
import { AssetWorkspace } from "../asset/AssetWorkspace";
import { assetStrings as text } from "../../i18n/asset";

function useAssetGate(): PolicyGate {
  const { api, session } = useAuth();
  const key = `${session?.client_session_incarnation ?? ""}:${session?.access_token ?? ""}`;
  const [resolved, setResolved] = useState(() => ({ key: "", api, gate: makePolicyGate(DENY_ALL_PROJECTION, false) }));

  useEffect(() => {
    const controller = new AbortController();
    void api.GET("/api/v1/me/authz", { signal: controller.signal }).then((response) => {
      if (controller.signal.aborted || !response.data) return;
      setResolved({ key, api, gate: makePolicyGate(parseAuthzResponse(response.data), true) });
    }).catch(() => {
      // The asset surface intentionally stays deny-by-omission on authz failure.
    });
    return () => { controller.abort(); };
  }, [api, key]);

  return resolved.key === key && resolved.api === api
    ? resolved.gate
    : makePolicyGate(DENY_ALL_PROJECTION, false);
}

/** Authoritative authz-backed asset screen. JWT claims are never used as UI authority. */
export function AssetModuleScreen() {
  const { api, session } = useAuth();
  const activeBranchId = useActiveBranchId();
  const gate = useAssetGate();
  const sessionKey = `${session?.client_session_incarnation ?? ""}:${session?.access_token ?? ""}`;
  const allows = (feature: string, branch?: string) => gate.ready && gate.allows({ feature, branch });

  if (!gate.ready) return <main className="asset-module-status" role="status">{text.authorizationChecking}</main>;
  if (!allows("work_order_read_all", activeBranchId)) return null;
  return <AssetWorkspace
    key={sessionKey}
    api={api}
    sessionKey={sessionKey}
    capabilities={{
      canRead: true,
      canManage: (branch) => allows("equipment_manage", branch),
      canReadCost: (branch) => allows("equipment_cost_ledger_read", branch),
      canImport: (branch) => allows("master_list_import", branch),
    }}
  />;
}
