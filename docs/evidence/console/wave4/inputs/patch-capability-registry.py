#!/usr/bin/env python3
"""Wave-4 L-P0-EPOCH registry correction.

Additive only:
  * refresh the stale source_revision pin,
  * seed CAP-SALES-CRM and CAP-ONTOLOGY-ENGINE,
  * drop `sales` from source_inventory.unmodeled_keys (CAP-SALES-CRM now owns it,
    and the route-key inventory must stay an exact bijection).

Every pre-existing capability row is left byte-identical.
"""
import json
from collections import OrderedDict

P = "docs/program/console-capability-registry.json"
CANDIDATE = "88c57a1d519b43bc4c0e7b721c62bc248b938b38"
HEAD = "4cabe239673e132765a003de04fd9dce5a86bfe2"
CONTROLS = [
    "CTRL-KR-PRIVACY-001",
    "CTRL-KR-WORKFORCE-001",
    "CTRL-KR-SAFETY-001",
    "CTRL-KR-FINANCE-001",
    "CTRL-KR-LOCATION-001",
    "CTRL-KR-RECORDS-001",
]
HOLD_REASON = (
    "Qualified Korean legal/control source and candidate-bound test/evidence receipt "
    "have not been admitted."
)


def score(weights, inputs):
    return round(sum(weights[k] * inputs[k] for k in inputs), 4)


def bindings(cap_id):
    return [
        {
            "jurisdiction_id": "JUR-KR-001",
            "control_id": control,
            "candidate_sha": CANDIDATE,
            "status": "HOLD",
            "reason": HOLD_REASON,
        }
        for control in CONTROLS
    ]


def main():
    with open(P, encoding="utf-8") as handle:
        reg = json.load(handle, object_pairs_hook=OrderedDict)

    weights = reg["priority_formula"]
    existing = {c["id"] for c in reg["capabilities"]}
    assert "CAP-SALES-CRM" not in existing and "CAP-ONTOLOGY-ENGINE" not in existing

    before = json.dumps(reg["capabilities"], ensure_ascii=False, sort_keys=True)

    reg["source_revision"] = f"origin/codex/operational-object-runtime-progress@{HEAD}"
    reg["source_revision_note"] = (
        "Refreshed 2026-07-25 (L-P0-EPOCH). The previous pin origin/main@8e42b9a2 was 935 "
        "commits behind while last_refreshed claimed 2026-07-25. This SHA is the wave-4 spine "
        "tip and the base HEAD of every wave-4 lane worktree. provenance.authority_base_sha "
        "and candidate.sha are deliberately NOT moved here: rebinding the candidate is the "
        "integrator's act, and the validator requires authority_base_sha to stay distinct "
        "from the candidate."
    )

    keys = reg["source_inventory"]["unmodeled_keys"]
    assert any(k["key"] == "sales" for k in keys), "sales already modelled"
    reg["source_inventory"]["unmodeled_keys"] = [k for k in keys if k["key"] != "sales"]
    reg["source_inventory"]["reclassified"] = [
        {
            "key": "sales",
            "from": "HOLD_UNMAPPED",
            "to": "CAP-SALES-CRM",
            "as_of": "2026-07-25",
            "reason": (
                "Route-key inventory must remain an exact bijection over capability "
                "route_keys plus unmodeled_keys; CAP-SALES-CRM now owns `sales`."
            ),
        }
    ]

    sales_inputs = OrderedDict([
        ("user_workflow_value", 0.95),
        ("dependency_unlock", 0.6),
        ("correctness_and_risk_reduction", 0.8),
        ("visual_or_functional_parity_gap", 0.9),
        ("business_coverage_gain", 0.95),
        ("verification_readiness", 0.6),
        ("collision_probability", 0.35),
        ("unpriced_dependency_cost", 0.5),
    ])
    ontology_inputs = OrderedDict([
        ("user_workflow_value", 0.6),
        ("dependency_unlock", 0.95),
        ("correctness_and_risk_reduction", 0.85),
        ("visual_or_functional_parity_gap", 0.7),
        ("business_coverage_gain", 0.5),
        ("verification_readiness", 0.7),
        ("collision_probability", 0.45),
        ("unpriced_dependency_cost", 0.4),
    ])

    sales = OrderedDict([
        ("id", "CAP-SALES-CRM"),
        ("label", "Sales CRM — deal pipeline, stage governance, lead PII"),
        ("evidence_basis", ["repository_contract", "design_reference", "dated_audit"]),
        ("frontier", 1),
        ("priority", OrderedDict([
            ("as_of", "2026-07-25"),
            ("inputs", sales_inputs),
            ("score", score(weights, sales_inputs)),
        ])),
        ("dependencies", ["CAP-ONTOLOGY-ENGINE", "CAP-CONSOLE-SHELL"]),
        ("owner", "unassigned"),
        ("worktree", None),
        ("branch", None),
        ("commits", []),
        ("integration_commits", []),
        ("ownership", OrderedDict([
            ("frontend_roots", ["web/src/console/sales/**", "web/src/i18n/salesCrm.ts"]),
            ("backend_roots", ["backend/crates/sales/**"]),
            ("api_schema_roots", []),
            ("migration_owner", "console-consolidation"),
            ("integration_owner", "console-consolidation"),
            ("private_roots", [
                "web/src/console/sales/**",
                "web/src/i18n/salesCrm.ts",
                "backend/crates/sales/**",
            ]),
            ("serial_roots", [
                "web/src/i18n/ko.ts",
                "web/src/console/shell/nav.ts",
                "web/src/console/screens/registry.ts",
                "backend/openapi/openapi.yaml",
                "backend/crates/platform/db/migrations/**",
                "backend/app/src/lib.rs",
                "backend/app/src/objects.rs",
            ]),
        ])),
        ("routes", ["/console/sales", "/api/v1/sales"]),
        ("signature_story", OrderedDict([
            ("id", "STORY-SALES-CRM-001"),
            ("outcome", (
                "An authorized sales owner takes an inbound inquiry to a deal, advances it "
                "through governed stages with the evidence each stage requires, loses or wins "
                "it against a typed reason, and — on Won — issues a contract, with the lead's "
                "personal data consented, retention-clocked, masked by default and audited on "
                "every unmasked view."
            )),
        ])),
        ("evidence_path", "docs/evidence/console/CAP-SALES-CRM"),
        ("tests", OrderedDict([
            ("files", []),
            ("leaf_commands", ["git diff --check"]),
            ("buck2_targets", []),
        ])),
        ("state", OrderedDict([
            ("design_contract", "authority_names_crm1_to_crm6_and_wfl9_not_yet_contracted"),
            ("backend", "forklift_sales_catalog_only_no_deal_aggregate"),
            ("frontend", "listings_panel_and_inquiry_inbox_integrated_dark_on_pr488"),
            ("e2e", "component_tests_only_no_browser_replay"),
            ("runtime", "blocked"),
            ("independent_review", "missing"),
            ("production_exposure", "dark"),
        ])),
        ("notes", [
            (
                "Registry provenance gap this row closes: `sales` was once the sole entry in "
                "EXPOSED_SCREEN_KEYS with no capability row backing it, and was emptied by "
                "b9e7fd74 'fix(console): fail closed without exposure evidence'. Re-verified "
                f"2026-07-25 at {HEAD}: EXPOSED_SCREEN_KEYS is [] on this branch and on "
                "origin/codex/operational-object-runtime-progress; nav.test.ts asserts "
                "toEqual([]). The row must exist before any exposure request is admissible."
            ),
            (
                "Backend today is a forklift sales CATALOG, not a CRM: sales_listings (0043), "
                "sales_listing_media, customer_inquiries (NEW/CONTACTED/CLOSED), four admin + "
                "four storefront routes. RLS FORCE + org_isolation on all three tables. There "
                "is no customer master, deal/opportunity, quote, owner assignment, or branch "
                "scope. The frontend docstring states this honestly and the screen matches it."
            ),
            (
                "Known defects on the built screen, recorded so no future reader mistakes the "
                "dark mount for depth: client-side status FSM, offset pagination on both "
                "streams, listbox instead of an ARIA grid, no 409/412 handling, no idempotency "
                "key on the mutation, explanatory sales-kicker/sales-muted copy (a binding "
                "merge-gate breach already in the tree), inline Intl locale, and a public "
                "storefront that collects name + phone with no consent record and no retention "
                "clock (0043:98-111)."
            ),
        ]),
        ("resource_requirements", OrderedDict([
            ("writer", 1), ("postgres", 1), ("browser", 1),
            ("ios", 0), ("graph", 1), ("cas", 0),
        ])),
        ("historical_state", OrderedDict([
            ("design_contract", "declared"),
            ("backend", "catalog_only"),
            ("frontend", "panel_only"),
            ("e2e", "component_only"),
            ("runtime", "not_exposed"),
            ("independent_review", "not_started"),
            ("production_exposure", "dark"),
        ])),
        ("historical_commits", []),
        ("historical_integration_commits", []),
        ("truth", OrderedDict([
            ("declared", "DECLARED"),
            ("implementation", "HOLD"),
            ("verification", "HOLD"),
            ("exposure", "HOLD"),
        ])),
        ("delivery_unit", OrderedDict([
            ("id", "DU-SALES-CRM"),
            ("kind", "module_capability"),
            ("buck2_targets", []),
            ("reason", (
                "The sales quartet exists in-tree but no exact candidate-bound Buck2 target "
                "set has been independently established for it; this registry preserves HOLD "
                "rather than inventing a target."
            )),
            ("rust_status", "REQUIRED_UNRESOLVED"),
        ])),
        ("dependency_edges", [
            OrderedDict([("target", "CAP-ONTOLOGY-ENGINE"), ("type", "requires")]),
            OrderedDict([("target", "CAP-CONSOLE-SHELL"), ("type", "requires")]),
            OrderedDict([("target", "CAP-SHARED-ONTOLOGY-WORKFLOW"), ("type", "integrates_with")]),
        ]),
        ("route_presentation", OrderedDict([
            ("route_keys", ["sales"]),
            ("source_mounted", True),
            ("production_exposed", False),
            ("registry_body_present", True),
            ("nav_declared", True),
            ("evidence_receipt_status", "HOLD"),
            ("source", "exact candidate source inventory"),
        ])),
        ("candidate_evidence", OrderedDict([
            ("candidate_sha", CANDIDATE),
            ("status", "HOLD"),
            ("reason", (
                "Row seeded 2026-07-25 by L-P0-EPOCH so the capability has registry provenance "
                "before any exposure request. No candidate-bound completion receipt is "
                "admitted; the built surface is a catalog panel, not the CRM the authority "
                "specifies."
            )),
            ("paths", ["docs/evidence/console/CAP-SALES-CRM"]),
            ("contract", OrderedDict([
                ("source_sha", CANDIDATE),
                ("backend_binary_digest_or_build_sha", "HOLD: no candidate-bound backend build receipt admitted for the sales quartet."),
                ("database", "HOLD: no candidate-bound database receipt admitted."),
                ("api", "HOLD: no candidate-bound API receipt admitted."),
                ("browser", "HOLD: no candidate-bound browser replay admitted; component tests only."),
                ("trace_logs", "HOLD: no candidate-bound runtime trace admitted."),
            ])),
        ])),
        ("benchmark", OrderedDict([
            ("category", "B2B sales CRM — pipeline, stage governance, lead data protection"),
            ("comparator_sources", [
                OrderedDict([
                    ("source", "docs/program/benchmark-matrix/INDEX.md"),
                    ("observation_as_of", "2026-07-25"),
                    ("observation", (
                        "The 14-module matrix has no sales/CRM module dossier; the closest "
                        "adjacent reads are the cross-cutting steal-list items (multi-record "
                        "workspace, in-panel object-page sections) which are IA findings, not "
                        "CRM comparators."
                    )),
                    ("seed_only", True),
                ]),
                OrderedDict([
                    ("source", "docs/program/benchmark-matrix/support.md"),
                    ("observation_as_of", "2026-07-25"),
                    ("observation", (
                        "The public storefront inquiry intake was built on the support-intake "
                        "pattern, so this dossier is the nearest in-repo comparator for the "
                        "inbound half of the funnel only — it says nothing about pipeline, "
                        "stage governance, or forecast."
                    )),
                    ("seed_only", True),
                ]),
            ]),
            ("required_outcomes", (
                "An inbound inquiry becomes an owned deal, advances through governed stages "
                "with per-stage evidence, and terminates in a typed Won/Lost outcome with a "
                "durable receipt — enforced server-side, audited, and reproducible."
            )),
            ("differentiated_outcomes", (
                "The deal is an ontology object, not a CRM silo: it links to customer, "
                "contract and listing through the shared graph, dispatches its stage advance "
                "through a governed projected Action, and inherits the console's PBAC, audit "
                "and object-card grammar."
            )),
            ("non_goals", (
                "Prototype checklists, seeded pipeline rows, a listings panel renamed CRM, or "
                "the historical dark mount are not delivery evidence."
            )),
            ("verdict", "HOLD"),
            ("evidence_binding", f"candidate:{CANDIDATE}; no exact module comparison or user-story evidence admitted"),
            ("native_outcomes", [
                OrderedDict([
                    ("id", "CAP-SALES-CRM-N1"),
                    ("persona_scenario", "Sales owner converting an inbound inquiry"),
                    ("action_workflow", "inquiry converts to an owned deal with provenance preserved"),
                    ("measurable_assertion", "CAP-SALES-CRM: a converted inquiry yields a deal whose inflow provenance resolves back to the originating inquiry"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/native-1.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-SALES-CRM-N2"),
                    ("persona_scenario", "Sales owner advancing a deal"),
                    ("action_workflow", "stage advance is refused server-side without the stage's required evidence"),
                    ("measurable_assertion", "CAP-SALES-CRM: stage advance fails closed as console_rt when per-stage evidence is absent"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/native-2.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-SALES-CRM-N3"),
                    ("persona_scenario", "Sales manager reassigning ownership in bulk"),
                    ("action_workflow", "owner reassignment is deterministic and idempotent under retry"),
                    ("measurable_assertion", "CAP-SALES-CRM: a replayed reassignment with the same idempotency key produces one audited effect"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/native-3.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-SALES-CRM-N4"),
                    ("persona_scenario", "Sales owner closing a Won deal"),
                    ("action_workflow", "Won issues a contract through the guarded composer, not a direct write"),
                    ("measurable_assertion", "CAP-SALES-CRM: a Won transition yields exactly one contract object and a durable receipt"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/native-4.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-SALES-CRM-N5"),
                    ("persona_scenario", "Privacy-obligated reviewer reading lead contact data"),
                    ("action_workflow", "unmasking a lead's personal data is consented, retention-clocked and audited"),
                    ("measurable_assertion", "CAP-SALES-CRM: an unmasked lead view without consent is denied, and a permitted one emits a sensitive-view audit row"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/native-5.json"),
                    ("status", "HOLD"),
                ]),
            ]),
            ("omni_outcomes", [
                OrderedDict([
                    ("id", "CAP-SALES-CRM-O1"),
                    ("persona_scenario", "Cross-module operator drilling from a deal"),
                    ("action_workflow", "the deal opens as a 3-layer object card and drills to its linked customer and contract"),
                    ("measurable_assertion", "CAP-SALES-CRM omni: a DL- code resolves to a governed object card with scope-safe typed links"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/omni-1.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-SALES-CRM-O2"),
                    ("persona_scenario", "Auditor reconstructing a closed deal"),
                    ("action_workflow", "the full stage history and its actors are reconstructable across the module boundary"),
                    ("measurable_assertion", "CAP-SALES-CRM omni: deal lineage is auditable end-to-end from inquiry to contract"),
                    ("required_receipts", "docs/evidence/console/CAP-SALES-CRM/omni-2.json"),
                    ("status", "HOLD"),
                ]),
            ]),
            ("independent_outcome_review", OrderedDict([
                ("status", "HOLD"),
                ("reason", "No independent candidate-bound outcome review receipt is admitted."),
            ])),
            ("dossier_status", "HOLD_INSUFFICIENT_CATEGORY_DOSSIER"),
            ("missing_dossier_reason", (
                "No CRM-native comparator dossier exists in docs/program/benchmark-matrix; the "
                "adjacent support-intake and cross-cutting IA reads are seed references, not "
                "category evidence."
            )),
        ])),
        ("jurisdiction_bindings", bindings("CAP-SALES-CRM")),
    ])

    ontology = OrderedDict([
        ("id", "CAP-ONTOLOGY-ENGINE"),
        ("label", "Ontology engine — catalog upgrade path and projected-type dispatch"),
        ("evidence_basis", ["repository_contract", "design_reference", "dated_audit"]),
        ("frontier", 0),
        ("priority", OrderedDict([
            ("as_of", "2026-07-25"),
            ("inputs", ontology_inputs),
            ("score", score(weights, ontology_inputs)),
        ])),
        ("dependencies", ["CAP-SHARED-ONTOLOGY-WORKFLOW"]),
        ("owner", "unassigned"),
        ("worktree", None),
        ("branch", None),
        ("commits", []),
        ("integration_commits", []),
        ("ownership", OrderedDict([
            ("frontend_roots", []),
            ("backend_roots", [
                "backend/crates/ontology/adapter-postgres/src/seed.rs",
                "backend/crates/ontology/adapter-postgres/src/instances.rs",
                "backend/crates/ontology/adapter-postgres/src/lib.rs",
            ]),
            ("api_schema_roots", []),
            ("migration_owner", "console-consolidation"),
            ("integration_owner", "console-consolidation"),
            ("private_roots", []),
            ("serial_roots", [
                "backend/crates/ontology/adapter-postgres/src/seed.rs",
                "backend/crates/ontology/adapter-postgres/src/instances.rs",
                "backend/crates/ontology/adapter-postgres/src/lib.rs",
                "backend/crates/platform/db/migrations/**",
                "backend/app/src/lib.rs",
                "backend/app/src/objects.rs",
            ]),
        ])),
        ("routes", ["/api/v1/object-types", "/api/v1/ontology"]),
        ("signature_story", OrderedDict([
            ("id", "STORY-ONTOLOGY-ENGINE-001"),
            ("outcome", (
                "A domain crate registers a new object type into an already-seeded tenant "
                "without a reinstall, and a governed Action on that projected type dispatches "
                "into the owning domain use-case and returns a real result instead of "
                "NotWiredYet."
            )),
        ])),
        ("evidence_path", "docs/evidence/console/CAP-ONTOLOGY-ENGINE"),
        ("tests", OrderedDict([
            ("files", []),
            ("leaf_commands", ["git diff --check"]),
            ("buck2_targets", []),
        ])),
        ("state", OrderedDict([
            ("design_contract", "authority_confirmed_projection_model_no_lane_contract_yet"),
            ("backend", "27_types_seeded_and_frozen_no_additive_upgrade_function_exists"),
            ("frontend", "not_applicable"),
            ("e2e", "missing"),
            ("runtime", "blocked"),
            ("independent_review", "missing"),
            ("production_exposure", "dark"),
        ])),
        ("notes", [
            (
                "Why this is a distinct capability from CAP-SHARED-ONTOLOGY-WORKFLOW: that row "
                "covers the ontology manager/explorer surfaces and owns "
                "backend/crates/ontology/** as a private root. This row covers the two engine "
                "residuals the north star names — the catalog upgrade path and projected-type "
                "action dispatch — and therefore declares NO private roots. Its three files "
                "are serial roots taken under a strict two-link train (L-A1 then L-X7, same "
                "owner) with the parent capability's release confirmed first."
            ),
            (
                "Verified 2026-07-25: install_builtin_catalog requires an exact digest match "
                "against the migration-owned allowlist "
                "(0165_ontology_object_type_key_revisions.sql:121-131, enforced at 0165:1113-"
                "1124) and has two fail-closed guards with no upgrade path (0165:1128-1143): "
                "different_catalog_already_installed, and empty_org_required for any tenant "
                "with an existing ont_object_types row. No catalog-upgrade function exists in "
                "the repo, so every already-seeded tenant is frozen at 27 types."
            ),
            (
                "The 27 seeded types include customer (CU-), site (SI-), equipment (FL-), "
                "work_order (WO-) and contract, but no deal, sales_listing or "
                "customer_inquiry — which is why CAP-SALES-CRM depends on this row."
            ),
            (
                "Scout finding that changes the wave's arithmetic: instance_acting and "
                "object_type_acting are already live routes "
                "(backend/crates/ontology/rest/src/lib.rs:1562-1588), so the dynamics-layer "
                "findings in the fidelity registers are unblocked by REGISTRATION alone — no "
                "new endpoint and no openapi change. See "
                "docs/evidence/console/wave4/backend-blocked-index.json, disposition "
                "unblocked_by_wave4_registration."
            ),
            (
                "Named engine residuals carried, not silently held: seed.rs:121 hardcodes "
                "dispatch_target None and projected_draft ships actions: Vec::new(), so the "
                "one wired projected handler is unreachable today; submission criteria are "
                "hard-rejected for projected actions (a criterion would fail OPEN); projected "
                "dispatch returns receipt: None (rest/src/lib.rs:840); four-eyes for projected "
                "actions is consumed in a separate committed step, so a failed dispatch spends "
                "the approval; and resolve_by_code only queries ont_instances, so a code on a "
                "projected type cannot resolve through the ontology path."
            ),
        ]),
        ("resource_requirements", OrderedDict([
            ("writer", 1), ("postgres", 1), ("browser", 0),
            ("ios", 0), ("graph", 1), ("cas", 0),
        ])),
        ("historical_state", OrderedDict([
            ("design_contract", "declared"),
            ("backend", "seeded_catalog_frozen"),
            ("frontend", "not_applicable"),
            ("e2e", "missing"),
            ("runtime", "not_exposed"),
            ("independent_review", "not_started"),
            ("production_exposure", "dark"),
        ])),
        ("historical_commits", []),
        ("historical_integration_commits", []),
        ("truth", OrderedDict([
            ("declared", "DECLARED"),
            ("implementation", "HOLD"),
            ("verification", "HOLD"),
            ("exposure", "HOLD"),
        ])),
        ("delivery_unit", OrderedDict([
            ("id", "DU-ONTOLOGY-ENGINE"),
            ("kind", "platform_capability"),
            ("buck2_targets", []),
            ("reason", (
                "No exact candidate-bound Buck2 target set has been independently established "
                "for the ontology adapter; this registry preserves HOLD rather than inventing "
                "a target."
            )),
            ("rust_status", "REQUIRED_UNRESOLVED"),
        ])),
        ("dependency_edges", [
            OrderedDict([("target", "CAP-SHARED-ONTOLOGY-WORKFLOW"), ("type", "requires")]),
            OrderedDict([("target", "CAP-SALES-CRM"), ("type", "blocks")]),
        ]),
        ("route_presentation", OrderedDict([
            ("route_keys", []),
            ("source_mounted", False),
            ("production_exposed", False),
            ("registry_body_present", False),
            ("nav_declared", False),
            ("evidence_receipt_status", "HOLD"),
            ("source", "no console route key: this capability is an engine surface consumed through CAP-SHARED-ONTOLOGY-WORKFLOW's screens"),
        ])),
        ("candidate_evidence", OrderedDict([
            ("candidate_sha", CANDIDATE),
            ("status", "HOLD"),
            ("reason", (
                "Row seeded 2026-07-25 by L-P0-EPOCH. The engine exists and the 27-type catalog "
                "is installed, but the two capabilities this row names — additive catalog "
                "upgrade and a reachable projected-type Action — are absent at the candidate, "
                "and no candidate-bound receipt is admitted."
            )),
            ("paths", ["docs/evidence/console/CAP-ONTOLOGY-ENGINE"]),
            ("contract", OrderedDict([
                ("source_sha", CANDIDATE),
                ("backend_binary_digest_or_build_sha", "HOLD: no candidate-bound backend build receipt admitted for the ontology adapter."),
                ("database", "HOLD: no candidate-bound catalog-install or upgrade receipt admitted."),
                ("api", "HOLD: no candidate-bound API receipt admitted; the acting routes are present but unexercised for any new type."),
                ("browser", "HOLD: not applicable at this layer and no candidate-bound replay admitted."),
                ("trace_logs", "HOLD: no candidate-bound dispatch trace admitted."),
            ])),
        ])),
        ("benchmark", OrderedDict([
            ("category", "enterprise ontology / object platform — schema lifecycle and governed writeback"),
            ("comparator_sources", [
                OrderedDict([
                    ("source", "docs/program/benchmark-matrix/object-platform.md"),
                    ("observation_as_of", "2026-07-25"),
                    ("observation", (
                        "Category-native dossier: ours vs Palantir Foundry, SAP MDG/S-4HANA, "
                        "n8n, Slack, Teams, Asana, Rippling, observed 2026-07-19. It names the "
                        "two residuals this capability owns — steal-item 1 (broaden projected-"
                        "type action dispatch beyond the single real target) and steal-item 3 "
                        "(deepen the 27 seeded types), plus steal-item 13 (schema lifecycle is "
                        "linear draft-to-publish with no branch/merge-check)."
                    )),
                    ("seed_only", False),
                ]),
                OrderedDict([
                    ("source", "docs/program/ontology-coverage-matrix.md"),
                    ("observation_as_of", "2026-07-25"),
                    ("observation", (
                        "Read-only in-repo coverage audit cited by the dossier at file:line; "
                        "it is the evidence base for the OURS column rather than an external "
                        "comparator."
                    )),
                    ("seed_only", True),
                ]),
            ]),
            ("required_outcomes", (
                "A new object type reaches an already-seeded tenant without reinstalling the "
                "catalog, and a governed Action on a projected type executes in the owning "
                "domain use-case under the engine's authorization and audit."
            )),
            ("differentiated_outcomes", (
                "Domain crates stay the sole writers: projection registration adds semantic, "
                "kinetic and dynamic layers plus links and Actions without routing any write "
                "through the generic instance store."
            )),
            ("non_goals", (
                "Type counts, a registered type with no reachable Action, or a dispatch target "
                "that still returns NotWiredYet are not delivery evidence."
            )),
            ("verdict", "HOLD"),
            ("evidence_binding", f"candidate:{CANDIDATE}; dossier observed 2026-07-19, no candidate-bound outcome evidence admitted"),
            ("native_outcomes", [
                OrderedDict([
                    ("id", "CAP-ONTOLOGY-ENGINE-N1"),
                    ("persona_scenario", "Platform owner adding a type to a live tenant"),
                    ("action_workflow", "additive catalog upgrade installs new keys and leaves existing keys untouched"),
                    ("measurable_assertion", "CAP-ONTOLOGY-ENGINE: an additive upgrade on a tenant with existing ont_object_types rows succeeds and mutates no pre-existing key"),
                    ("required_receipts", "docs/evidence/console/CAP-ONTOLOGY-ENGINE/native-1.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-ONTOLOGY-ENGINE-N2"),
                    ("persona_scenario", "Platform owner re-running an upgrade"),
                    ("action_workflow", "the upgrade is idempotent and its digest chain stays allowlisted"),
                    ("measurable_assertion", "CAP-ONTOLOGY-ENGINE: a repeated additive upgrade is a no-op and the catalog digest remains allowlisted"),
                    ("required_receipts", "docs/evidence/console/CAP-ONTOLOGY-ENGINE/native-2.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-ONTOLOGY-ENGINE-N3"),
                    ("persona_scenario", "Operator invoking a governed Action on a projected type"),
                    ("action_workflow", "dispatch reaches the owning domain use-case instead of NotWiredYet"),
                    ("measurable_assertion", "CAP-ONTOLOGY-ENGINE: a projected-type Action dispatches into its domain use-case and the domain state changes"),
                    ("required_receipts", "docs/evidence/console/CAP-ONTOLOGY-ENGINE/native-3.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-ONTOLOGY-ENGINE-N4"),
                    ("persona_scenario", "Operator listing instances of a newly projected type"),
                    ("action_workflow", "the projected backing table is allowlisted so list does not fail after registration"),
                    ("measurable_assertion", "CAP-ONTOLOGY-ENGINE: listing a newly registered projected type returns rows rather than rejecting the backing table"),
                    ("required_receipts", "docs/evidence/console/CAP-ONTOLOGY-ENGINE/native-4.json"),
                    ("status", "HOLD"),
                ]),
            ]),
            ("omni_outcomes", [
                OrderedDict([
                    ("id", "CAP-ONTOLOGY-ENGINE-O1"),
                    ("persona_scenario", "Any module surfacing its dynamics layer"),
                    ("action_workflow", "acting automations for a registered type are readable through the existing acting routes"),
                    ("measurable_assertion", "CAP-ONTOLOGY-ENGINE omni: registering a module type makes object_type_acting return its rules with no new endpoint"),
                    ("required_receipts", "docs/evidence/console/CAP-ONTOLOGY-ENGINE/omni-1.json"),
                    ("status", "HOLD"),
                ]),
                OrderedDict([
                    ("id", "CAP-ONTOLOGY-ENGINE-O2"),
                    ("persona_scenario", "Console reader following a code token across modules"),
                    ("action_workflow", "a newly registered code prefix linkifies and resolves with no frontend map edit"),
                    ("measurable_assertion", "CAP-ONTOLOGY-ENGINE omni: a runtime-primed code prefix linkifies from bare text and resolves to a card slug without a frontend change"),
                    ("required_receipts", "docs/evidence/console/CAP-ONTOLOGY-ENGINE/omni-2.json"),
                    ("status", "HOLD"),
                ]),
            ]),
            ("independent_outcome_review", OrderedDict([
                ("status", "HOLD"),
                ("reason", "No independent candidate-bound outcome review receipt is admitted."),
            ])),
            ("dossier_status", "SOURCE_BOUNDED_STARTING_DOSSIER"),
        ])),
        ("jurisdiction_bindings", bindings("CAP-ONTOLOGY-ENGINE")),
    ])

    reg["capabilities"].append(sales)
    reg["capabilities"].append(ontology)

    after = json.dumps(
        [c for c in reg["capabilities"] if c["id"] not in ("CAP-SALES-CRM", "CAP-ONTOLOGY-ENGINE")],
        ensure_ascii=False, sort_keys=True,
    )
    assert before == after, "pre-existing capability rows were modified"

    with open(P, "w", encoding="utf-8") as handle:
        json.dump(reg, handle, ensure_ascii=False, indent=1)
        handle.write("\n")
    json.load(open(P, encoding="utf-8"))
    print("ok:", len(reg["capabilities"]), "capabilities")
    print("sales score", sales["priority"]["score"], "ontology score", ontology["priority"]["score"])


if __name__ == "__main__":
    main()
