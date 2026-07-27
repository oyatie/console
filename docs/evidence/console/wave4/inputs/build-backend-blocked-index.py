#!/usr/bin/env python3
"""Generate docs/evidence/console/wave4/backend-blocked-index.json.

Row bodies are read from the committed fidelity register so they cannot drift
from the audit. Only the classification columns (owning backend surface,
CRM relevance, wave-4 disposition) are hand-authored here.
"""
import json
import sys

REG = "docs/evidence/console/wave4/inputs/fidelity-registers.json"
OUT = "docs/evidence/console/wave4/backend-blocked-index.json"

# id -> (label, what the missing surface actually is)
SURFACES = {
    "ONT-OBJECT-CARD-READ": "Canonical 3-layer object-card read (properties/relations/lifecycle/audit per kind). Today GET /api/objects/{kind}/{id} returns ObjectHead = code/title/status only.",
    "ONT-DYNAMICS-ACTING": "Acting automations/policies for a type or instance. NOT missing: ontology/rest/src/lib.rs:1562-1588 already serves instance_acting / object_type_acting — blocked on the module type being REGISTERED, not on a new endpoint.",
    "ONT-TYPE-REGISTRY": "OT- type card / type-explore surface bound to a module header chip. GET /api/v1/object-types exists and primes codeGrammar; the type-card screen does not.",
    "ONT-SERIES": "SR- parent series objects (이력·추세·다음 회차) and their read surface.",
    "ONT-OBJECT-SEARCH": "PBAC-filtered cross-object code/typeahead search (§4-27-4). Contract is owned by codex/console-search-object-fabric-20260724.",
    "ONT-LINK-RESOLVER": "Cross-module object-link resolution/drill registry so a code token in any module resolves to its object (§4.7-10 언급=링크).",
    "APPR-COMPOSE": "Console 기안/전자결재 composer surface with prefill, and the draft→approve lifecycle in front of direct writes (§3.9.0).",
    "WORKFORCE-POOL": "Workforce/talent-pool registry (유형·단가·자격·평점·가용성, per-assignment contract issuance) and its registration/proposal mutations.",
    "CUSTOMER-REGISTRY-READ": "Read/resolve surface over registry_customers so customer identity renders and drills outside the owning crate.",
    "PLATFORM-CODE-ISSUANCE": "Human display-code issuance on a domain contract (PS-, EQ-, RC-) with a code counter, so the object carries a reference token (§4.7-3).",
    "TYPED-REASON-ENUM": "Curated reason enum on a mutation DTO instead of free text (§4-19, §4.8).",
    "EXTENSIBLE-ENUM-SCHEMA": "N+1 「직접 입력」 enum extension flowing back to the type schema (§4-27-3).",
    "LIST-SERVER-SORT": "Server-side sort/keyset parameters on a list endpoint so header sort stays truthful across pages.",
    "ADR-0025-EXPOSURE-MANIFEST": "Not a backend surface: EXPOSED_SCREEN_KEYS. Dark screens are legitimately unreachable targets; the count of dead-ends is the artifact.",
    "PERSON-CARD-SURFACE": "Exposed person/인사 카드 surface a @mention or directory row can open.",
    "PAYROLL-LINE-AMOUNTS": "기본급/수당/공제/실지급 (+ prior-run delta) on the payroll line DTO.",
    "PAYROLL-RUN-CALC-BREAKDOWN": "Per-법인 totals, prior-run delta and 사업자 부담 on the run calc summary contract.",
    "PAYROLL-RUN-DEADLINE": "Decision-due timestamp on the payroll run contract.",
    "ATT-SCHEDULE-READ": "Schedule/timetable read surface — the 계획 half of 계획 vs 실적. No schedule registry exists.",
    "ATT-COVER-FORWARD-QUEUE": "Approved+pending leave for D+7 by cover-required role, feeding the preventive cover planner.",
    "ATT-SITE-ATTRIBUTION": "Site/team attribution on attendance records (or a roster join) for grouping, filters and group notes.",
    "ATT-CLOSE-DEADLINE": "Deadline field on the attendance close board.",
    "WORKITEM-CREATE": "Follow-up work-item create/link surface (mywork).",
    "ORG-UNIT-BRANCH-LINK": "branch ↔ org_unit linkage and per-branch headcount on the hr org-chart / identity contract.",
    "ORG-CHANGE-OP-VOCAB": "Org-change proposal op vocabulary extensions (site-scoped reassign, dissolve org-unit, rename entity) — digest + backend.",
    "ORG-ENTITY-CREATE": "org-entity create (or a CREATE_ENTITY proposal op); /api/v1/org-entities is read-only.",
    "ORG-ENTITY-PROFILE-FINANCE": "Entity profile (등록/관할) + PBAC-gated finance summary + view-audit event on expand.",
    "XMOD-SUBJECT-CONTEXT": "Per-subject cross-module context read (attendance summary + recent tasks + KPI) for scorecard auto-attach.",
    "INV-INGEST": "실사 업로드 data-ingest surface, and backend modelling of in-transit stock.",
    "OPS-MAP-GEO": "Ops-map console surface: site/unit geo read API + map module.",
    "EVIDENCE-READ-PRESIGN": "Evidence read / presigned-GET so an evidence entry opens and can be verify-inspected.",
    "EVIDENCE-DERIVED-PREVIEW": "GET copies/{id}/derived-preview + the ZIP entry-index REST behind the watermarked viewer.",
    "EVIDENCE-ORIGINAL-FORBID-AUDIT": "Server-side original-copy stream endpoint so a blocked attempt is recorded as an audited forbid instead of a client-side denial.",
    "DOCS-RECORDS-REGISTRY": "Records-registry list REST covering the non-EV finalized types (AP-/JL-/NT-/IN-/C-).",
    "DOCS-EXPORT-EGRESS": "Evidence/records export REST behind the §3.10-⑤ egress gate (only the audit action is defined; no endpoint).",
    "DOCS-REGISTER-PENDING-APPROVAL": "Pending/approve stage on the records register contract (registration is immediate today).",
    "DOCS-FINALIZATION-STATE": "종결/finalization state on the records wire.",
    "BOARD-NOTICE-LINKED-OBJECTS": "Linked-object references on the notices contract (mail send, inbox delivery batch, records registration).",
    "DIR-CONTACT-PROFILE": "ext/phone/email on MessengerMemberSummary and Employee (or a directory-profile endpoint).",
    "DIR-ROSTER-ENRICHMENT": "PBAC-safe title/company/ext on the non-privileged roster endpoint (or one merged list with per-field deny-by-omission).",
    "CONSOLE-COMMAND-PALETTE": "Console command palette, and the collaborative-sheet export surface.",
    "LOGISTICS-READ-ENDPOINTS": "GET list endpoints for ASNs/fulfillments/shipments (+ stock projection) scoped by branch; the pilot router is write-only.",
    "RECRUIT-TALENTPOOL-REFS": "applicant_id / posting_id on the talent-pool payload so the row can drill.",
    "DOC-PROVENANCE-FILE-READ": "Original-document (provenance) read endpoint that logs an audited view.",
    "FIELD-CONTRACT-DIM": "site → 계약 (C- 코드·서비스 유형) and 상주 headcount on FieldSiteRow/FieldSiteDetail.",
    "FIELD-SERVICE-TYPE-ENUM": "Service-type enum on the contract, for the row/det en chip and its filter.",
    "WORK-JOURNAL-JL": "업무일지 (JL-) object and API.",
    "EQ-DISPOSITION-LINK-GL": "Disposition REPAIR → WO- object creation/link, and finance GL/voucher posting readback (financeGlPosting is hard-typed null).",
}

# "module.findings[n]" -> (primary surface, [additional surfaces], crm_relevant,
#                          disposition, note)
#
# crm_relevant := the owning backend surface is exercised, extended, or explicitly
# gap-manifested by a wave-4 CRM (sales) lane, so wave-4 CRM work moves this row.
#
# disposition ∈
#   unblocked_by_wave4_registration — L-X7 registering CRM types proves the surface
#                                     already exists; the module needs registration only
#   unblocked_by_wave4_exposure     — D-1/L-X14 exposing `sales` gives the row a target
#   pattern_proven_by_wave4_crm     — wave 4 builds the same primitive for sales; this
#                                     row still needs its own contract
#   gap_manifested_only             — a wave-4 CRM lane names the same gap with anchors
#                                     but does not close it
#   deferred_no_wave4_lane          — no wave-4 lane touches this at all
MAP = {
    "payroll.findings[1]": ("PAYROLL-LINE-AMOUNTS", [], False, "deferred_no_wave4_lane",
        "Payroll depth is R-1's wave; payroll must not be exposed until L-D0/L-D4 land."),
    "payroll.findings[4]": ("ONT-SERIES", [], False, "deferred_no_wave4_lane",
        "L-X7 registers projected types but creates no SR- series; §8 keeps the series chip out of wave 4."),
    "payroll.findings[5]": ("PAYROLL-RUN-CALC-BREAKDOWN", [], False, "deferred_no_wave4_lane", ""),
    "payroll.findings[9]": ("PAYROLL-RUN-DEADLINE", [], False, "deferred_no_wave4_lane", ""),
    "payroll.findings[13]": ("PLATFORM-CODE-ISSUANCE", [], True, "pattern_proven_by_wave4_crm",
        "L-X7 issues the DL- prefix + code counter through the legacy object_types registry, and L-F3 makes the frontend cost of a new prefix zero. PS- issuance is the same move on the payroll contract."),

    "recruiting.findings[0]": ("WORKFORCE-POOL", [], False, "deferred_no_wave4_lane",
        "Blocker: an OFFER-stage POOL_DAILY applicant has no primary CTA anywhere — the pipeline dead-ends."),
    "recruiting.findings[1]": ("WORKFORCE-POOL", [], False, "deferred_no_wave4_lane", ""),
    "recruiting.findings[6]": ("RECRUIT-TALENTPOOL-REFS", ["ONT-LINK-RESOLVER"], False, "deferred_no_wave4_lane", ""),
    "recruiting.findings[11]": ("DOC-PROVENANCE-FILE-READ", [], False, "deferred_no_wave4_lane", ""),

    "attendance.findings[1]": ("ATT-SCHEDULE-READ", [], False, "deferred_no_wave4_lane",
        "Blocker: 계획 vs 실적 is the module's core concept and the plan half does not exist."),
    "attendance.findings[4]": ("ATT-COVER-FORWARD-QUEUE", [], False, "deferred_no_wave4_lane", ""),
    "attendance.findings[5]": ("ATT-SITE-ATTRIBUTION", [], False, "deferred_no_wave4_lane", ""),
    "attendance.findings[8]": ("WORKITEM-CREATE", [], False, "deferred_no_wave4_lane", ""),
    "attendance.findings[9]": ("WORKFORCE-POOL", [], False, "deferred_no_wave4_lane", ""),
    "attendance.findings[11]": ("ATT-CLOSE-DEADLINE", [], False, "deferred_no_wave4_lane", ""),

    "org.findings[2]": ("ORG-UNIT-BRANCH-LINK", [], False, "deferred_no_wave4_lane", ""),
    "org.findings[3]": ("ORG-CHANGE-OP-VOCAB", [], False, "deferred_no_wave4_lane", ""),
    "org.findings[4]": ("ORG-ENTITY-CREATE", [], False, "deferred_no_wave4_lane", ""),
    "org.findings[5]": ("ORG-ENTITY-PROFILE-FINANCE", [], True, "pattern_proven_by_wave4_crm",
        "L-X8 builds the audited-sensitive-view + masking + deny-by-omission primitive for lead PII; the gated 재무 요약 needs the same primitive over its own contract."),
    "org.findings[7]": ("ORG-CHANGE-OP-VOCAB", [], False, "deferred_no_wave4_lane", ""),
    "org.findings[8]": ("ORG-CHANGE-OP-VOCAB", [], False, "deferred_no_wave4_lane", ""),
    "org.findings[11]": ("TYPED-REASON-ENUM", [], True, "pattern_proven_by_wave4_crm",
        "L-X3 ships the Closed-Lost reason enum for sales — the same §4-19 typed-reason move; org-change needs its own reason_kind field."),

    "evaluation.findings[2]": ("XMOD-SUBJECT-CONTEXT", [], False, "deferred_no_wave4_lane", ""),
    "evaluation.findings[4]": ("ONT-OBJECT-SEARCH", [], True, "gap_manifested_only",
        "Charter §7: the object typeahead/search contract is owned by codex/console-search-object-fabric-20260724 and L-X10 gap-manifests rather than builds it. Sales carries the identical free-text-ref hazard."),
    "evaluation.findings[11]": ("EXTENSIBLE-ENUM-SCHEMA", [], False, "deferred_no_wave4_lane",
        "Honest note: L-X3's Closed-Lost enum is also closed, so wave-4 CRM inherits the same §4-27-3 gap rather than resolving it."),

    "inventory.findings[1]": ("ONT-OBJECT-SEARCH", [], True, "gap_manifested_only",
        "Blocker: a UUID paste field for a WO/dispatch reference. Same missing search fabric as evaluation.findings[4]."),
    "inventory.findings[3]": ("ONT-DYNAMICS-ACTING", ["ONT-OBJECT-CARD-READ", "ONT-SERIES"], True, "unblocked_by_wave4_registration",
        "The automation-chip half needs no endpoint — instance_acting/object_type_acting are live; registering the inventory types is the whole fix. L-X7 proves the path on deal/listing/inquiry. Object-card and series halves stay blocked."),
    "inventory.findings[4]": ("ONT-LINK-RESOLVER", [], True, "pattern_proven_by_wave4_crm",
        "L-X7 must add a `deal` row to RESOLVABLE_KIND_AUTH (integrator manifest + security review) — the same chokepoint that would resolve WO-/PO-/dispatch tokens."),
    "inventory.findings[5]": ("APPR-COMPOSE", [], True, "pattern_proven_by_wave4_crm",
        "L-X5 builds a guarded composer for Won → contract C-; 부족 → 구매 기안 is the same closed loop on a different contract."),
    "inventory.findings[7]": ("ONT-TYPE-REGISTRY", [], True, "gap_manifested_only",
        "L-X7 seeds the legacy object_types row so /api/v1/object-types primes codeGrammar, but no type-card screen lands this wave. Interim fix is frontend-only: delete the inert 'IV' chip."),
    "inventory.findings[13]": ("INV-INGEST", [], False, "deferred_no_wave4_lane", ""),

    "dispatch.findings[2]": ("OPS-MAP-GEO", [], False, "deferred_no_wave4_lane", ""),
    "dispatch.findings[3]": ("ONT-OBJECT-CARD-READ", [], True, "gap_manifested_only",
        "L-X10 consumes the legacy object-card stack for deal and gap-manifests the useObjectCard/ObjectExplorerModel split; the richer canonical read surface is not built."),
    "dispatch.findings[4]": ("CUSTOMER-REGISTRY-READ", [], True, "gap_manifested_only",
        "L-X7 links deal → the seeded `customer` type, which makes a customer read/resolve surface a shared need; no wave-4 lane builds registry_customers reads."),
    "dispatch.findings[5]": ("APPR-COMPOSE", [], True, "pattern_proven_by_wave4_crm", ""),
    "dispatch.findings[7]": ("ONT-DYNAMICS-ACTING", [], True, "unblocked_by_wave4_registration",
        "Registration alone; the acting routes are live."),

    "maintenance.findings[1]": ("ONT-OBJECT-SEARCH", [], True, "gap_manifested_only",
        "A raw mechanic-UUID text input cannot ship as the assignment UX."),
    "maintenance.findings[3]": ("ONT-OBJECT-CARD-READ", ["ONT-SERIES", "ONT-DYNAMICS-ACTING"], True, "gap_manifested_only",
        "Registering a work_order kind in console/objectcard is frontend-only and available now; series and automation chips stay blocked."),
    "maintenance.findings[6]": ("EVIDENCE-READ-PRESIGN", [], False, "deferred_no_wave4_lane", ""),

    "field.findings[3]": ("FIELD-CONTRACT-DIM", [], True, "pattern_proven_by_wave4_crm",
        "L-X5 mints the C- contract object from a Won deal — the upstream half of field's contract dimension. The site↔contract read join is not built."),
    "field.findings[4]": ("ONT-LINK-RESOLVER", ["CUSTOMER-REGISTRY-READ", "OPS-MAP-GEO"], True, "gap_manifested_only", ""),
    "field.findings[5]": ("APPR-COMPOSE", [], True, "pattern_proven_by_wave4_crm", ""),
    "field.findings[6]": ("ONT-DYNAMICS-ACTING", [], True, "unblocked_by_wave4_registration",
        "Registration alone; the acting routes are live. 역학 0 today."),
    "field.findings[8]": ("ONT-OBJECT-CARD-READ", ["ONT-SERIES"], True, "gap_manifested_only", ""),
    "field.findings[13]": ("FIELD-SERVICE-TYPE-ENUM", [], False, "deferred_no_wave4_lane", ""),
    "field.findings[14]": ("WORK-JOURNAL-JL", [], False, "deferred_no_wave4_lane", ""),

    "docs.findings[0]": ("DOCS-RECORDS-REGISTRY", [], False, "deferred_no_wave4_lane",
        "Blocker: the module ships as the EV- archive only; the 7-type record archive the design specifies needs a records-registry list REST."),
    "docs.findings[3]": ("DOCS-EXPORT-EGRESS", [], False, "deferred_no_wave4_lane", ""),
    "docs.findings[4]": ("DOCS-REGISTER-PENDING-APPROVAL", [], False, "deferred_no_wave4_lane", ""),
    "docs.findings[5]": ("EVIDENCE-DERIVED-PREVIEW", [], False, "deferred_no_wave4_lane", ""),
    "docs.findings[6]": ("ONT-LINK-RESOLVER", [], True, "gap_manifested_only", ""),
    "docs.findings[7]": ("EVIDENCE-ORIGINAL-FORBID-AUDIT", [], False, "deferred_no_wave4_lane",
        "The UI is currently the fail-closed boundary, so a blocked attempt leaves no audit trace."),
    "docs.findings[9]": ("DOCS-FINALIZATION-STATE", [], False, "deferred_no_wave4_lane", ""),

    "notif.findings[0]": ("ONT-OBJECT-CARD-READ", ["ADR-0025-EXPOSURE-MANIFEST"], True, "unblocked_by_wave4_exposure",
        "The finding's own fix says 'no exposed object surface exists anywhere'. D-1/L-X14 exposing `sales` with an object card creates the first target notifModel.rowTarget can route to."),
    "notif.findings[1]": ("ADR-0025-EXPOSURE-MANIFEST", [], True, "unblocked_by_wave4_exposure",
        "Not a defect: authority-sanctioned ADR-0025 fail-closed. Adding `sales` to EXPOSED_SCREEN_KEYS self-heals exactly one screen-link target; the remaining dark targets are the count exposure reviews must see."),
    "notif.findings[4]": ("PERSON-CARD-SURFACE", [], False, "deferred_no_wave4_lane", ""),

    "board.findings[6]": ("BOARD-NOTICE-LINKED-OBJECTS", ["ONT-DYNAMICS-ACTING"], False, "deferred_no_wave4_lane",
        "The automation-chip half is unblocked by registration like the other dynamics rows, but the notices link contract is module-specific and no wave-4 lane touches it."),
    "board.findings[7]": ("ONT-TYPE-REGISTRY", [], True, "gap_manifested_only", ""),

    "directory.findings[0]": ("DIR-CONTACT-PROFILE", [], False, "deferred_no_wave4_lane",
        "Blocker: a directory with no contact channel at all — neither segment renders 연락처, email, or a 메일 action."),
    "directory.findings[1]": ("DIR-ROSTER-ENRICHMENT", [], False, "deferred_no_wave4_lane", ""),
    "directory.findings[3]": ("ONT-OBJECT-CARD-READ", ["ONT-DYNAMICS-ACTING"], True, "gap_manifested_only", ""),
    "directory.findings[4]": ("LIST-SERVER-SORT", [], True, "pattern_proven_by_wave4_crm",
        "L-X9 (keyset pagination) and L-X11 (ARIA grid semantics) build the truthful server-side list grammar for sales. §8 explicitly defers L-F4, so the primitive is NOT extracted this wave — directory would still need its own sort param."),
    "directory.findings[5]": ("CONSOLE-COMMAND-PALETTE", [], False, "deferred_no_wave4_lane", ""),

    "logistics.findings[6]": ("LOGISTICS-READ-ENDPOINTS", [], False, "deferred_no_wave4_lane",
        "The backend is a write-only pilot router: every queue starts empty each session and cross-session/org objects are invisible."),

    "equipment.findings[1]": ("ONT-DYNAMICS-ACTING", ["ONT-OBJECT-CARD-READ", "ONT-SERIES"], True, "unblocked_by_wave4_registration",
        "Blocker. The dynamics third of the §4-14 layer test needs registration only; object card and series stay blocked."),
    "equipment.findings[3]": ("PLATFORM-CODE-ISSUANCE", [], True, "pattern_proven_by_wave4_crm",
        "Raw UUIDs are user-facing text today. Same issuance move as L-X7's DL- prefix + counter."),
    "equipment.findings[4]": ("APPR-COMPOSE", [], True, "pattern_proven_by_wave4_crm", ""),
    "equipment.findings[5]": ("ONT-OBJECT-SEARCH", ["CUSTOMER-REGISTRY-READ"], True, "gap_manifested_only", ""),
    "equipment.findings[7]": ("EQ-DISPOSITION-LINK-GL", [], False, "deferred_no_wave4_lane", ""),
}

DEFERRAL_CLASS = "§8 'All 13 non-CRM module FE lanes' — the 13 undeepened modules receive only their share of the Phase-0 shared fixes; their fidelity registers stay open, gap-manifested as a block with anchors."


def main() -> int:
    reg = json.load(open(REG, encoding="utf-8"))
    rows = []
    seen = set()
    for register in reg["registers"]:
        module = register["module"]
        for index, finding in enumerate(register["findings"]):
            if not finding.get("backend_blocked"):
                continue
            key = f"{module}.findings[{index}]"
            if key not in MAP:
                print(f"UNMAPPED backend-blocked finding: {key}", file=sys.stderr)
                return 1
            seen.add(key)
            primary, extra, crm, disposition, note = MAP[key]
            for surface in [primary, *extra]:
                if surface not in SURFACES:
                    print(f"UNKNOWN surface {surface} on {key}", file=sys.stderr)
                    return 1
            rows.append({
                "id": key,
                "module": module,
                "capability_id": register["cap"],
                "severity": finding["severity"],
                "finding": {
                    "design_says": finding["design_says"],
                    "code_does": finding["code_does"],
                    "fix": finding["fix"],
                },
                "backend_surface": {
                    "id": primary,
                    "what_is_missing": SURFACES[primary],
                },
                "additional_surfaces": [
                    {"id": s, "what_is_missing": SURFACES[s]} for s in extra
                ],
                "crm_relevant": crm,
                "wave4_disposition": disposition,
                "wave4_note": note,
                "wave4_owning_lane": None,
                "deferral": DEFERRAL_CLASS,
            })
    unused = sorted(set(MAP) - seen)
    if unused:
        print(f"MAP rows with no matching finding: {unused}", file=sys.stderr)
        return 1

    by_surface = {}
    for row in rows:
        by_surface.setdefault(row["backend_surface"]["id"], []).append(row["id"])

    out = {
        "schema": "wave4-backend-blocked-index-v1",
        "produced": "2026-07-25",
        "purpose": (
            "One row per backend-blocked fidelity finding. Depth-first (D-0) drops 13 of 15 "
            "module lanes, so this ledger of what is NOT being done is the wave's main honesty "
            "artifact and the integrator's completion checklist."
        ),
        "generated_from": {
            "register": REG,
            "register_schema": reg["schema"],
            "total_findings": sum(len(r["findings"]) for r in reg["registers"]),
            "backend_blocked_findings": len(rows),
            "modules": len(reg["registers"]),
        },
        "column_semantics": {
            "backend_surface": "The backend surface that owns the block. Row bodies come from the committed register; this classification is hand-authored in the generator and reviewable against the register text.",
            "crm_relevant": "True when the owning surface is exercised, extended, or explicitly gap-manifested by a wave-4 CRM (sales) lane, so wave-4 CRM work moves this row. False means no wave-4 lane touches it.",
            "wave4_disposition": {
                "unblocked_by_wave4_registration": "instance_acting / object_type_acting are already live (ontology/rest/src/lib.rs:1562-1588); the module needs type REGISTRATION, not a new endpoint. L-X7 proves the path.",
                "unblocked_by_wave4_exposure": "D-1 / L-X14 exposing `sales` gives the row its first reachable target.",
                "pattern_proven_by_wave4_crm": "Wave 4 builds the same primitive for sales; this row still needs its own contract.",
                "gap_manifested_only": "A wave-4 CRM lane names the same gap with anchors but does not close it.",
                "deferred_no_wave4_lane": "No wave-4 lane touches this at all.",
            },
            "wave4_owning_lane": "Always null this wave: depth-first assigns no lane to the 13 undeepened modules. Recorded explicitly rather than left blank.",
        },
        "counts": {
            "rows": len(rows),
            "crm_relevant": sum(1 for r in rows if r["crm_relevant"]),
            "not_crm_relevant": sum(1 for r in rows if not r["crm_relevant"]),
            "by_severity": {
                sev: sum(1 for r in rows if r["severity"] == sev)
                for sev in ("blocker", "major", "minor")
            },
            "by_disposition": {
                d: sum(1 for r in rows if r["wave4_disposition"] == d)
                for d in sorted({r["wave4_disposition"] for r in rows})
            },
            "by_module": {
                m: sum(1 for r in rows if r["module"] == m)
                for m in sorted({r["module"] for r in rows})
            },
        },
        "surfaces": [
            {
                "id": sid,
                "what_is_missing": SURFACES[sid],
                "primary_for": by_surface.get(sid, []),
            }
            for sid in sorted(SURFACES)
        ],
        "rows": rows,
    }
    with open(OUT, "w", encoding="utf-8") as handle:
        json.dump(out, handle, ensure_ascii=False, indent=1)
        handle.write("\n")
    json.load(open(OUT, encoding="utf-8"))
    print(json.dumps(out["counts"], ensure_ascii=False, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
