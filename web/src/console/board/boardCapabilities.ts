/** `Feature::NoticeManage` snake_case wire key (org-wide: `BranchScope::All`). */
export const NOTICE_MANAGE_FEATURE = "notice_manage";

/**
 * Advisory PolicyGated action id for the recipient-bound acknowledge control.
 * Visibility is row-gated on `my_receipt`; the backend binds the receipt to the
 * JWT principal (404 for non-recipients), so the UI decider always allows it.
 */
export const BOARD_ACK_ACTION = "notice_ack";

/** Structural subset of the canonical `PolicyGate` (console/policy/authz). */
export interface BoardPolicyGate {
  allows: (query: { feature: string; minPermission: "allow" }) => boolean;
}

export interface BoardCapabilities {
  /** Every authenticated org member reads published notices. */
  canRead: boolean;
  /** Draft/publish/receipts affordances — deny-by-omission without it. */
  canManage: boolean;
}

/** Pure projection adapter matching the notices backend feature gate. */
export function deriveBoardCapabilities(gate: BoardPolicyGate): BoardCapabilities {
  return {
    canRead: true,
    canManage: gate.allows({ feature: NOTICE_MANAGE_FEATURE, minPermission: "allow" }),
  };
}
