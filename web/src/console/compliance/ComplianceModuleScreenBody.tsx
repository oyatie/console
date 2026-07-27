// 준법·의무 screen body — the SCREEN_REGISTRY mount for the "compliance" nav
// slot. The real surface (CP-/RG-/FW- catalog list, status/risk chips, FSM
// next-states, control→evidence coverage ledger) lives in GenericModuleScreen +
// complianceModuleScreen; this body binds the authenticated api client AND the
// policy gate, same idiom as ModuleFinanceScreenBody.
//
// Gate mirrors the nav gate for this slot (g(INTEGRITY_ROLES,
// [INTEGRITY_FINDINGS_READ])): EXECUTIVE/SUPER_ADMIN read the catalog, and a
// holder of the integrity-findings feature grant reads it too. Every other role
// is denied by omission (blank — the intended state for the unauthorized). The
// backend re-authorizes and RLS-scopes every compliance read.
import { useMemo } from "react";

import { useAuth } from "../../context/auth";
import { GenericModuleScreen } from "../modules/GenericModuleScreen";
import { PolicyGateProvider, type PolicyGate } from "../policy";
import { COMPLIANCE_ACTIONS } from "./complianceModel";
import { complianceModuleScreen } from "./complianceModuleScreen";
import { EvidenceBindingWorkbench } from "./EvidenceBindingWorkbench";

const COMPLIANCE_READ_ROLES = new Set(["EXECUTIVE", "SUPER_ADMIN"]);
const AUDIT_LOG_READ_ROLES = new Set(["ADMIN", "SUPER_ADMIN"]);

export function ComplianceModuleScreenBody() {
  const { api, session } = useAuth();
  const roles = session?.roles;
  const featureGrants = session?.feature_grants;
  // The provider-owned incarnation changes whenever the effective session or
  // tenant context changes. Never retain scope-bound rows across a missing or
  // changed incarnation; the backend remains the authorization authority.
  const authorityKey =
    session?.org_id && session.user_id && session.client_session_incarnation
      ? `${session.org_id}:${session.user_id}:${session.client_session_incarnation}`
      : undefined;

  const canRead =
    (roles?.some((role) => COMPLIANCE_READ_ROLES.has(role)) ?? false) ||
    (featureGrants?.includes(COMPLIANCE_ACTIONS.read) ?? false);
  const canReadAuditLog =
    (roles?.some((role) => AUDIT_LOG_READ_ROLES.has(role)) ?? false) ||
    (featureGrants?.includes(COMPLIANCE_ACTIONS.audit) ?? false);
  const gate = useMemo<PolicyGate>(
    () => ({
      can: (action) => {
        if (
          action === COMPLIANCE_ACTIONS.read ||
          action === COMPLIANCE_ACTIONS.regulationRead ||
          action === COMPLIANCE_ACTIONS.frameworkRead
        )
          return canRead;
        if (action === COMPLIANCE_ACTIONS.audit) return canReadAuditLog;
        return featureGrants?.includes(action) ?? false;
      },
    }),
    [canRead, canReadAuditLog, featureGrants],
  );
  // The backend permits this org-wide action to SUPER_ADMIN or an org-wide
  // custom grant. This is a conservative UI hint; the REST boundary remains
  // authoritative for every submission.
  const isOrgWideAdmin = roles?.includes("SUPER_ADMIN") ?? false;
  const canLinkEvidence =
    canRead &&
    (isOrgWideAdmin ||
      (featureGrants?.includes(COMPLIANCE_ACTIONS.evidenceLink) ?? false));
  const canAcceptEvidence =
    canRead &&
    (isOrgWideAdmin ||
      (featureGrants?.includes(COMPLIANCE_ACTIONS.domainManage) ?? false));

  return (
    <PolicyGateProvider gate={gate}>
      <GenericModuleScreen
        config={complianceModuleScreen}
        api={api}
        authorityKey={authorityKey}
      />
      <EvidenceBindingWorkbench
        key={authorityKey ?? "no-authority"}
        api={api}
        authorityKey={authorityKey}
        canRead={canRead}
        canLink={canLinkEvidence}
        canAccept={canAcceptEvidence}
      />
    </PolicyGateProvider>
  );
}
