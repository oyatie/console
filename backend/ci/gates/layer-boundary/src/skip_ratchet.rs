//! Shrink-only ratchet for Clean Architecture Rest/Worker skips (ADR-0045).
//!
//! `KNOWN_REST_OR_WORKER_SKIP_EDGES` names Rest/Worker → Domain or Adapter
//! edges that still exist on the tree that accepted ADR-0045. New skips fail
//! closed. Removing a listed edge without deleting the entry fails as stale.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownSkipEdge {
    pub from: &'static str,
    pub to: &'static str,
}

impl KnownSkipEdge {
    #[must_use]
    pub fn matches(self, from: &str, to: &str) -> bool {
        self.from == from && self.to == to
    }
}

pub const KNOWN_REST_OR_WORKER_SKIP_EDGES: &[KnownSkipEdge] = &[
    KnownSkipEdge {
        from: "console-analytics-quant-rest",
        to: "console-analytics-quant-service",
    },
    KnownSkipEdge {
        from: "console-attendance-rest",
        to: "console-attendance-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-attendance-rest",
        to: "console-attendance-domain",
    },
    KnownSkipEdge {
        from: "console-benefit-rest",
        to: "console-benefit-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-benefit-rest",
        to: "console-benefit-domain",
    },
    KnownSkipEdge {
        from: "console-comms-rest",
        to: "console-comms-adapter-imap",
    },
    KnownSkipEdge {
        from: "console-comms-rest",
        to: "console-comms-credential-cipher",
    },
    KnownSkipEdge {
        from: "console-comms-rest",
        to: "console-comms-adapter-mox",
    },
    KnownSkipEdge {
        from: "console-comms-rest",
        to: "console-comms-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-comms-rest",
        to: "console-comms-adapter-smtp",
    },
    KnownSkipEdge {
        from: "console-comms-rest",
        to: "console-comms-domain",
    },
    KnownSkipEdge {
        from: "console-compliance-rest",
        to: "console-compliance-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-compliance-rest",
        to: "console-compliance-domain",
    },
    KnownSkipEdge {
        from: "console-dispatch-rest",
        to: "console-dispatch-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-dispatch-rest",
        to: "console-dispatch-domain",
    },
    KnownSkipEdge {
        from: "console-dispatch-worker",
        to: "console-dispatch-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-docs-rest",
        to: "console-docs-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-docs-rest",
        to: "console-docs-domain",
    },
    KnownSkipEdge {
        from: "console-docs-rest",
        to: "console-governance-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-equipment-rest",
        to: "console-equipment-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-evaluation-rest",
        to: "console-evaluation-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-evaluation-rest",
        to: "console-evaluation-domain",
    },
    KnownSkipEdge {
        from: "console-finance-gl-rest",
        to: "console-finance-gl-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-finance-gl-rest",
        to: "console-finance-gl-domain",
    },
    KnownSkipEdge {
        from: "console-financial-rest",
        to: "console-financial-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-financial-rest",
        to: "console-financial-domain",
    },
    KnownSkipEdge {
        from: "console-governance-rest",
        to: "console-governance-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-governance-rest",
        to: "console-governance-domain",
    },
    KnownSkipEdge {
        from: "console-identity-rest",
        to: "console-identity-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-identity-rest",
        to: "console-identity-domain",
    },
    KnownSkipEdge {
        from: "console-inbox-rest",
        to: "console-inbox-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-inspection-rest",
        to: "console-inspection-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-inspection-rest",
        to: "console-inspection-domain",
    },
    KnownSkipEdge {
        from: "console-inventory-rest",
        to: "console-inventory-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-inventory-rest",
        to: "console-inventory-domain",
    },
    KnownSkipEdge {
        from: "console-leave-rest",
        to: "console-leave-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-leave-rest",
        to: "console-leave-domain",
    },
    KnownSkipEdge {
        from: "console-logistics-rest",
        to: "console-logistics-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-messenger-rest",
        to: "console-messenger-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-messenger-rest",
        to: "console-messenger-domain",
    },
    KnownSkipEdge {
        from: "console-notices-rest",
        to: "console-notices-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-notifications-rest",
        to: "console-notifications-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-notifications-rest",
        to: "console-notifications-domain",
    },
    KnownSkipEdge {
        from: "console-ontology-rest",
        to: "console-ontology-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-ontology-rest",
        to: "console-ontology-canonical-domain",
    },
    KnownSkipEdge {
        from: "console-ontology-rest",
        to: "console-ontology-domain",
    },
    KnownSkipEdge {
        from: "console-ontology-rest",
        to: "console-governance-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-ontology-rest",
        to: "console-governance-domain",
    },
    KnownSkipEdge {
        from: "console-orgchange-rest",
        to: "console-orgchange-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-orgchange-rest",
        to: "console-orgchange-domain",
    },
    KnownSkipEdge {
        from: "console-payroll-rest",
        to: "console-inbox-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-payroll-rest",
        to: "console-inbox-domain",
    },
    KnownSkipEdge {
        from: "console-payroll-rest",
        to: "console-payroll-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-payroll-rest",
        to: "console-payroll-domain",
    },
    KnownSkipEdge {
        from: "console-recruiting-rest",
        to: "console-recruiting-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-recruiting-rest",
        to: "console-recruiting-domain",
    },
    KnownSkipEdge {
        from: "console-registry-rest",
        to: "console-registry-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-registry-rest",
        to: "console-registry-domain",
    },
    KnownSkipEdge {
        from: "console-reporting-rest",
        to: "console-reporting-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-reporting-rest",
        to: "console-reporting-domain",
    },
    KnownSkipEdge {
        from: "console-sales-rest",
        to: "console-sales-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-sales-rest",
        to: "console-sales-domain",
    },
    KnownSkipEdge {
        from: "console-support-rest",
        to: "console-support-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-support-rest",
        to: "console-support-domain",
    },
    KnownSkipEdge {
        from: "console-todos-rest",
        to: "console-todos-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-todos-rest",
        to: "console-todos-domain",
    },
    KnownSkipEdge {
        from: "console-workorder-rest",
        to: "console-workflow-domain",
    },
    KnownSkipEdge {
        from: "console-workorder-rest",
        to: "console-workflow-runtime",
    },
    KnownSkipEdge {
        from: "console-workorder-rest",
        to: "console-workflow-runtime-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-workorder-rest",
        to: "console-workorder-adapter-postgres",
    },
    KnownSkipEdge {
        from: "console-workorder-rest",
        to: "console-workorder-domain",
    },
];

/// `Layer::Rest` crates with no sibling `console-<stem>-application`.
pub const KNOWN_REST_WITHOUT_APPLICATION: &[&str] = &[
    "console-analytics-quant-rest",
    "console-consulting-rest",
    "console-facilities-rest",
    "console-orgchange-rest",
    "console-production-rest",
];
