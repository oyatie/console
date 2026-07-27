import Foundation
import ConsoleAPIClient

private typealias LegacyLeaveRequestView = Components.Schemas.LeaveRequestView
private typealias LeaveRequestV2View = Components.Schemas.LeaveRequestV2View
private typealias LeaveRequestV2Page = Components.Schemas.LeaveRequestV2Page
private typealias ActionInboxResponse = Components.Schemas.ActionInboxResponse
private typealias EvidenceObjectPage = Components.Schemas.EvidenceObjectPage
private typealias FacilitiesCase = Components.Schemas.FacilitiesCase
private typealias ProductionCredential = Components.Schemas.ProductionSourceSystemCredential
private typealias ProductionReceipt = Components.Schemas.ProductionSourceSystemReceipt
private typealias ProductionIngressReceipt = Components.Schemas.ProductionSourceIngressReceipt
private typealias EvaluationUnitProgress = Components.Schemas.EvaluationUnitProgress
private typealias EvaluationSubjectSummary = Components.Schemas.EvaluationSubjectSummary
private typealias EvaluationCycleDetail = Components.Schemas.EvaluationCycleDetail
private typealias EvaluationReview = Components.Schemas.EvaluationReview
private typealias EvaluationSubjectDetail = Components.Schemas.EvaluationSubjectDetail
private typealias EvaluationPreflightItem = Components.Schemas.EvaluationPreflightItem
private typealias EvaluationPreflightReport = Components.Schemas.EvaluationPreflightReport
private typealias EvaluationTaskItem = Components.Schemas.EvaluationTaskItem

@main
private enum GeneratedClientContractTests {
    static func main() throws {
        let tests: [(String, () -> Bool)] = [
            ("decodes the frozen v1 leave response", decodesFrozenV1LeaveResponse),
            ("decodes required null charge_units", decodesRequiredNullChargeUnits),
            ("re-encodes null charge_units as an explicit field", reencodesNullChargeUnitsAsExplicitField),
            ("rejects a payload without charge_units", rejectsPayloadWithoutChargeUnits),
            ("decodes required non-null days", decodesRequiredNonNullDays),
            ("rejects a payload without days", rejectsPayloadWithoutDays),
            ("decodes required null leave next_cursor", decodesRequiredNullLeaveNextCursor),
            ("re-encodes null leave next_cursor as an explicit field", reencodesNullLeaveNextCursorAsExplicitField),
            ("rejects a leave page without next_cursor", rejectsLeavePageWithoutNextCursor),
            ("decodes required null action next_cursor", decodesRequiredNullActionNextCursor),
            ("re-encodes null action next_cursor as an explicit field", reencodesNullActionNextCursorAsExplicitField),
            ("rejects an action page without next_cursor", rejectsActionPageWithoutNextCursor),
            ("decodes required null evidence next_cursor", decodesRequiredNullEvidenceNextCursor),
            ("re-encodes null evidence next_cursor as an explicit field", reencodesNullEvidenceNextCursorAsExplicitField),
            ("rejects an evidence page without next_cursor", rejectsEvidencePageWithoutNextCursor),
            ("preserves required-nullable evaluation unit progress fields", preservesEvaluationUnitProgressNulls),
            ("rejects missing required-nullable evaluation unit progress fields", rejectsMissingEvaluationUnitProgressNulls),
            ("preserves required-nullable evaluation subject summary fields", preservesEvaluationSubjectSummaryNulls),
            ("rejects missing required-nullable evaluation subject summary fields", rejectsMissingEvaluationSubjectSummaryNulls),
            ("preserves required-nullable evaluation cycle detail fields", preservesEvaluationCycleDetailNulls),
            ("rejects missing required-nullable evaluation cycle detail fields", rejectsMissingEvaluationCycleDetailNulls),
            ("preserves required-nullable evaluation review fields", preservesEvaluationReviewNulls),
            ("rejects missing required-nullable evaluation review fields", rejectsMissingEvaluationReviewNulls),
            ("preserves required-nullable evaluation subject detail fields", preservesEvaluationSubjectDetailNulls),
            ("rejects missing required-nullable evaluation subject detail fields", rejectsMissingEvaluationSubjectDetailNulls),
            ("preserves required-nullable evaluation preflight item fields", preservesEvaluationPreflightItemNulls),
            ("rejects missing required-nullable evaluation preflight item fields", rejectsMissingEvaluationPreflightItemNulls),
            ("preserves required-nullable evaluation preflight report fields", preservesEvaluationPreflightReportNulls),
            ("rejects missing required-nullable evaluation preflight report fields", rejectsMissingEvaluationPreflightReportNulls),
            ("preserves required-nullable evaluation task fields", preservesEvaluationTaskItemNulls),
            ("rejects missing required-nullable evaluation task fields", rejectsMissingEvaluationTaskItemNulls),
            ("keeps facilities transition outputs and request bodies typed", keepsFacilitiesTransitionOutputsTyped),
            ("keeps production source receipts typed and serializes every ingress kind", keepsProductionSourceContractsTyped),
        ]
        let failures: [String] = tests.compactMap { name, test -> String? in
            if test() {
                print("PASS: \(name)")
                return nil
            }
            return name
        }

        guard failures.isEmpty else {
            throw ContractFailure(failures: failures)
        }
    }

    private static func decodesFrozenV1LeaveResponse() -> Bool {
        guard let decoded = try? decoder.decode(LegacyLeaveRequestView.self, from: validV1Payload) else {
            return false
        }
        return decoded.days == 1.0
    }

    private static func decodesRequiredNullChargeUnits() -> Bool {
        guard let decoded = try? decoder.decode(LeaveRequestV2View.self, from: validPayload) else {
            return false
        }
        return decoded.chargeUnits == nil
    }

    private static func reencodesNullChargeUnitsAsExplicitField() -> Bool {
        guard
            let decoded = try? decoder.decode(LeaveRequestV2View.self, from: validPayload),
            let data = try? encoder.encode(decoded),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            return false
        }
        return object.keys.contains("charge_units") && object["charge_units"] is NSNull
    }

    private static func decodesRequiredNonNullDays() -> Bool {
        guard let decoded = try? decoder.decode(LeaveRequestV2View.self, from: validPayload) else {
            return false
        }
        return decoded.days == 1.0
    }

    private static func rejectsPayloadWithoutChargeUnits() -> Bool {
        rejectsPayloadWithoutRequiredKey("charge_units")
    }

    private static func rejectsPayloadWithoutDays() -> Bool {
        rejectsPayloadWithoutRequiredKey("days")
    }

    private static func rejectsPayloadWithoutRequiredKey(_ key: String) -> Bool {
        rejectsPayloadWithoutRequiredKey(key, payload: validPayload, as: LeaveRequestV2View.self)
    }

    private static func decodesRequiredNullLeaveNextCursor() -> Bool {
        guard let decoded = try? decoder.decode(LeaveRequestV2Page.self, from: validLeavePage) else {
            return false
        }
        return decoded.nextCursor == nil
    }

    private static func reencodesNullLeaveNextCursorAsExplicitField() -> Bool {
        explicitNullRoundTrips(validLeavePage, as: LeaveRequestV2Page.self, key: "next_cursor")
    }

    private static func rejectsLeavePageWithoutNextCursor() -> Bool {
        rejectsPayloadWithoutRequiredKey("next_cursor", payload: validLeavePage, as: LeaveRequestV2Page.self)
    }

    private static func decodesRequiredNullActionNextCursor() -> Bool {
        guard let decoded = try? decoder.decode(ActionInboxResponse.self, from: validActionPage) else {
            return false
        }
        return decoded.nextCursor == nil
    }

    private static func reencodesNullActionNextCursorAsExplicitField() -> Bool {
        explicitNullRoundTrips(validActionPage, as: ActionInboxResponse.self, key: "next_cursor")
    }

    private static func rejectsActionPageWithoutNextCursor() -> Bool {
        rejectsPayloadWithoutRequiredKey("next_cursor", payload: validActionPage, as: ActionInboxResponse.self)
    }

    private static func decodesRequiredNullEvidenceNextCursor() -> Bool {
        guard let decoded = try? decoder.decode(EvidenceObjectPage.self, from: validEvidencePage) else {
            return false
        }
        return decoded.nextCursor == nil
    }

    private static func reencodesNullEvidenceNextCursorAsExplicitField() -> Bool {
        explicitNullRoundTrips(validEvidencePage, as: EvidenceObjectPage.self, key: "next_cursor")
    }

    private static func rejectsEvidencePageWithoutNextCursor() -> Bool {
        rejectsPayloadWithoutRequiredKey("next_cursor", payload: validEvidencePage, as: EvidenceObjectPage.self)
    }

    private static func preservesEvaluationUnitProgressNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationUnitProgressPayload,
            as: EvaluationUnitProgress.self,
            keys: ["org_unit"]
        )
    }

    private static func rejectsMissingEvaluationUnitProgressNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["org_unit"],
            payload: evaluationUnitProgressPayload,
            as: EvaluationUnitProgress.self
        )
    }

    private static func preservesEvaluationSubjectSummaryNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationSubjectSummaryPayload,
            as: EvaluationSubjectSummary.self,
            keys: ["org_unit", "final_grade", "rv_code"]
        )
    }

    private static func rejectsMissingEvaluationSubjectSummaryNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["org_unit", "final_grade", "rv_code"],
            payload: evaluationSubjectSummaryPayload,
            as: EvaluationSubjectSummary.self
        )
    }

    private static func preservesEvaluationCycleDetailNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationCycleDetailPayload,
            as: EvaluationCycleDetail.self,
            keys: ["opened_at", "calibration_started_at", "finalized_at", "archived_at"]
        )
    }

    private static func rejectsMissingEvaluationCycleDetailNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["opened_at", "calibration_started_at", "finalized_at", "archived_at"],
            payload: evaluationCycleDetailPayload,
            as: EvaluationCycleDetail.self
        )
    }

    private static func preservesEvaluationReviewNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationReviewPayload,
            as: EvaluationReview.self,
            keys: ["grade", "note", "submitted_at"]
        )
    }

    private static func rejectsMissingEvaluationReviewNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["grade", "note", "submitted_at"],
            payload: evaluationReviewPayload,
            as: EvaluationReview.self
        )
    }

    private static func preservesEvaluationSubjectDetailNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationSubjectDetailPayload,
            as: EvaluationSubjectDetail.self,
            keys: [
                "org_unit",
                "final_grade",
                "rv_code",
                "calibrated_grade",
                "calibration_reason",
                "calibrated_by",
                "calibrated_at",
                "finalized_at",
            ]
        )
    }

    private static func rejectsMissingEvaluationSubjectDetailNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            [
                "org_unit",
                "final_grade",
                "rv_code",
                "calibrated_grade",
                "calibration_reason",
                "calibrated_by",
                "calibrated_at",
                "finalized_at",
            ],
            payload: evaluationSubjectDetailPayload,
            as: EvaluationSubjectDetail.self
        )
    }

    private static func preservesEvaluationPreflightItemNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationPreflightItemPayload,
            as: EvaluationPreflightItem.self,
            keys: ["subject_id"]
        )
    }

    private static func rejectsMissingEvaluationPreflightItemNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["subject_id"],
            payload: evaluationPreflightItemPayload,
            as: EvaluationPreflightItem.self
        )
    }

    private static func preservesEvaluationPreflightReportNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationPreflightReportPayload,
            as: EvaluationPreflightReport.self,
            keys: ["next_transition"]
        )
    }

    private static func rejectsMissingEvaluationPreflightReportNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["next_transition"],
            payload: evaluationPreflightReportPayload,
            as: EvaluationPreflightReport.self
        )
    }

    private static func preservesEvaluationTaskItemNulls() -> Bool {
        explicitNullsRoundTrip(
            evaluationTaskItemPayload,
            as: EvaluationTaskItem.self,
            keys: ["review_status"]
        )
    }

    private static func rejectsMissingEvaluationTaskItemNulls() -> Bool {
        rejectsPayloadWithoutRequiredKeys(
            ["review_status"],
            payload: evaluationTaskItemPayload,
            as: EvaluationTaskItem.self
        )
    }

    private static func keepsFacilitiesTransitionOutputsTyped() -> Bool {
        func caseFromTriage(_ body: Operations.TriageFacilitiesCase.Output.Ok.Body) -> FacilitiesCase {
            switch body { case let .json(caseValue): return caseValue }
        }
        func caseFromAssign(_ body: Operations.AssignFacilitiesCase.Output.Ok.Body) -> FacilitiesCase {
            switch body { case let .json(caseValue): return caseValue }
        }
        func caseFromStart(_ body: Operations.StartFacilitiesCase.Output.Ok.Body) -> FacilitiesCase {
            switch body { case let .json(caseValue): return caseValue }
        }
        func caseFromSubmit(_ body: Operations.SubmitFacilitiesExecution.Output.Ok.Body) -> FacilitiesCase {
            switch body { case let .json(caseValue): return caseValue }
        }
        func caseFromAcceptance(_ body: Operations.DecideFacilitiesAcceptance.Output.Ok.Body) -> FacilitiesCase {
            switch body { case let .json(caseValue): return caseValue }
        }
        func caseFromObservation(_ body: Operations.RecordFacilitiesObservation.Output.Ok.Body) -> FacilitiesCase {
            switch body { case let .json(caseValue): return caseValue }
        }
        func typedTriageRequest(_ body: Operations.TriageFacilitiesCase.Input.Body) {}
        func typedAssignRequest(_ body: Operations.AssignFacilitiesCase.Input.Body) {}
        func typedSubmitRequest(_ body: Operations.SubmitFacilitiesExecution.Input.Body) {}
        func typedAcceptanceRequest(_ body: Operations.DecideFacilitiesAcceptance.Input.Body) {}
        func typedObservationRequest(_ body: Operations.RecordFacilitiesObservation.Input.Body) {}
        func caseBranch(_ caseValue: FacilitiesCase) -> Components.Schemas.Uuid { caseValue.branchId }
        _ = [caseFromTriage, caseFromAssign, caseFromStart, caseFromSubmit, caseFromAcceptance, caseFromObservation]
        _ = caseBranch
        _ = [typedTriageRequest, typedAssignRequest, typedSubmitRequest, typedAcceptanceRequest, typedObservationRequest]
        return true
    }

    private static func keepsProductionSourceContractsTyped() -> Bool {
        func credentialFromRegister(_ body: Operations.RegisterProductionSourceSystem.Output.Created.Body) -> ProductionCredential {
            switch body { case let .json(value): return value }
        }
        func credentialFromRotate(_ body: Operations.RotateProductionSourceSystem.Output.Ok.Body) -> ProductionCredential {
            switch body { case let .json(value): return value }
        }
        func receiptFromDisable(_ body: Operations.DisableProductionSourceSystem.Output.Ok.Body) -> ProductionReceipt {
            switch body { case let .json(value): return value }
        }
        func receiptFromIngress(_ body: Operations.IngestProductionSource.Output.Ok.Body) -> ProductionIngressReceipt {
            switch body { case let .json(value): return value }
        }
        func typedRegisterRequest(_ body: Operations.RegisterProductionSourceSystem.Input.Body) {}
        func typedRotateRequest(_ body: Operations.RotateProductionSourceSystem.Input.Body) {}
        func typedDisableRequest(_ body: Operations.DisableProductionSourceSystem.Input.Body) {}
        _ = [credentialFromRegister, credentialFromRotate, receiptFromDisable, receiptFromIngress]
        _ = [typedRegisterRequest, typedRotateRequest, typedDisableRequest]

        guard let credential = try? decoder.decode(
            ProductionCredential.self,
            from: Data(#"{"id":"00000000-0000-0000-0000-000000000001","source_system":"erp","enabled":true,"credential_generation":1,"secret":"one-time-secret"}"#.utf8)
        ), credential.secret == "one-time-secret" else { return false }

        let id = "00000000-0000-0000-0000-000000000001"
        let ingress: [(String, Components.Schemas.ProductionSourceIngress)] = [
            ("demand", .demand(.init(kind: .demand, id: id, inquiryId: id, productCode: "WIDGET", quantity: 1, dueAt: Date(timeIntervalSince1970: 0), sourceId: "erp", sourceVersion: "v1"))),
            ("capacity", .capacity(.init(kind: .capacity, id: id, siteId: id, capacityDate: "2026-07-23", availableQuantity: 1, sourceId: "mes", sourceVersion: "v1"))),
            ("material", .material(.init(kind: .material, materialItemId: id, quantityOnHandMilli: 1, safetyStockMilli: 0, sourceId: "wms", sourceVersion: "v1"))),
        ]
        return ingress.allSatisfy { expectedKind, value in
            guard
                let encoded = try? encoder.encode(value),
                let object = try? JSONSerialization.jsonObject(with: encoded) as? [String: Any]
            else { return false }
            return object["kind"] as? String == expectedKind
        }
    }

    private static func explicitNullRoundTrips<Model: Codable>(
        _ payload: Data,
        as type: Model.Type,
        key: String
    ) -> Bool {
        guard
            let decoded = try? decoder.decode(type, from: payload),
            let data = try? encoder.encode(decoded),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            return false
        }
        return object.keys.contains(key) && object[key] is NSNull
    }

    private static func explicitNullsRoundTrip<Model: Codable>(
        _ payload: Data,
        as type: Model.Type,
        keys: [String]
    ) -> Bool {
        guard
            let decoded = try? decoder.decode(type, from: payload),
            let data = try? encoder.encode(decoded),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            return false
        }
        return keys.allSatisfy { object.keys.contains($0) && object[$0] is NSNull }
    }

    private static func rejectsPayloadWithoutRequiredKeys<Model: Decodable>(
        _ keys: [String],
        payload: Data,
        as type: Model.Type
    ) -> Bool {
        guard (try? decoder.decode(type, from: payload)) != nil else {
            return false
        }
        return keys.allSatisfy { key in
            rejectsPayloadWithoutRequiredKey(key, payload: payload, as: type)
        }
    }

    private static func rejectsPayloadWithoutRequiredKey<Model: Decodable>(
        _ key: String,
        payload: Data,
        as type: Model.Type
    ) -> Bool {
        guard
            var object = try? JSONSerialization.jsonObject(with: payload) as? [String: Any],
            let payload = try? JSONSerialization.data(withJSONObject: object.removingValue(forKey: key))
        else {
            return false
        }
        do {
            _ = try decoder.decode(type, from: payload)
            return false
        } catch is DecodingError {
            return true
        } catch {
            return false
        }
    }

    private static var decoder: JSONDecoder {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }

    private static var encoder: JSONEncoder {
        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        return encoder
    }

    private static var validPayload: Data {
        Data(
            #"{"id":"00000000-0000-0000-0000-000000000001","branch_id":"00000000-0000-0000-0000-000000000002","requester_user_id":"00000000-0000-0000-0000-000000000003","subject_employee_id":"00000000-0000-0000-0000-000000000004","leave_type":"annual","days":1.0,"charge_units":null,"charge_state":"review_required","charge_review_reasons":["missing_calendar"],"request_version":1,"charge_version":0,"start_date":"2026-07-20","end_date":"2026-07-20","reason":"Annual leave","status":"pending","created_at":"2026-07-19T12:00:00Z"}"#.utf8
        )
    }

    private static var validV1Payload: Data {
        Data(
            #"{"id":"00000000-0000-0000-0000-000000000001","branch_id":"00000000-0000-0000-0000-000000000002","requester_user_id":"00000000-0000-0000-0000-000000000003","subject_employee_id":"00000000-0000-0000-0000-000000000004","leave_type":"annual","days":1.0,"start_date":"2026-07-20","end_date":"2026-07-20","reason":"Annual leave","status":"pending","decided_by":null,"decided_at":null,"created_at":"2026-07-19T12:00:00Z"}"#.utf8
        )
    }

    private static var validLeavePage: Data {
        Data(#"{"items":[],"next_cursor":null}"#.utf8)
    }

    private static var validActionPage: Data {
        Data(#"{"items":[],"total":0,"total_is_exact":true,"next_cursor":null}"#.utf8)
    }

    private static var validEvidencePage: Data {
        Data(#"{"items":[],"limit":50,"offset":0,"total":0,"as_of":42,"next_cursor":null}"#.utf8)
    }

    private static var evaluationUnitProgressPayload: Data {
        Data(#"{"org_unit":null,"total":1,"manager_submitted":0}"#.utf8)
    }

    private static var evaluationSubjectSummaryPayload: Data {
        Data(
            #"{"id":"00000000-0000-0000-0000-000000000001","cycle_id":"00000000-0000-0000-0000-000000000002","employee_id":"00000000-0000-0000-0000-000000000003","employee_name":"Employee","org_unit":null,"manager_user_id":"00000000-0000-0000-0000-000000000004","state":"ENROLLED","final_grade":null,"rv_code":null}"#.utf8
        )
    }

    private static var evaluationCycleDetailPayload: Data {
        Data(
            #"{"id":"00000000-0000-0000-0000-000000000001","name":"2026 Review","kind":"REGULAR","period_label":"2026","due_date":"2026-12-31","stage":"DRAFT","subjects_total":0,"manager_submitted":0,"self_submitted":0,"calibrated":0,"finalized":0,"created_at":"2026-07-25T12:00:00Z","opened_at":null,"calibration_started_at":null,"finalized_at":null,"archived_at":null,"created_by":"00000000-0000-0000-0000-000000000002","progress_by_unit":[],"subjects":[]}"#.utf8
        )
    }

    private static var evaluationReviewPayload: Data {
        Data(
            #"{"id":"00000000-0000-0000-0000-000000000001","subject_id":"00000000-0000-0000-0000-000000000002","kind":"SELF","status":"DRAFT","evaluator_user_id":"00000000-0000-0000-0000-000000000003","grade":null,"note":null,"evidence_links":[],"submitted_at":null,"updated_at":"2026-07-25T12:00:00Z"}"#.utf8
        )
    }

    private static var evaluationSubjectDetailPayload: Data {
        Data(
            #"{"id":"00000000-0000-0000-0000-000000000001","cycle_id":"00000000-0000-0000-0000-000000000002","employee_id":"00000000-0000-0000-0000-000000000003","employee_name":"Employee","org_unit":null,"manager_user_id":"00000000-0000-0000-0000-000000000004","state":"ENROLLED","final_grade":null,"rv_code":null,"goals":[],"reviews":[],"calibrated_grade":null,"calibration_reason":null,"calibrated_by":null,"calibrated_at":null,"finalized_at":null}"#.utf8
        )
    }

    private static var evaluationPreflightItemPayload: Data {
        Data(#"{"code":"READY","message":"Ready","subject_id":null}"#.utf8)
    }

    private static var evaluationPreflightReportPayload: Data {
        Data(#"{"next_transition":null,"blockers":[],"advisories":[]}"#.utf8)
    }

    private static var evaluationTaskItemPayload: Data {
        Data(
            #"{"subject_id":"00000000-0000-0000-0000-000000000001","cycle_id":"00000000-0000-0000-0000-000000000002","cycle_name":"2026 Review","due_date":"2026-12-31","employee_id":"00000000-0000-0000-0000-000000000003","employee_name":"Employee","kind":"SELF","review_status":null}"#.utf8
        )
    }
}

private struct ContractFailure: Error, CustomStringConvertible {
    let failures: [String]

    var description: String {
        "Generated client contract failures: \(failures.joined(separator: ", "))"
    }
}

private extension Dictionary where Key == String, Value == Any {
    mutating func removingValue(forKey key: String) -> Self {
        removeValue(forKey: key)
        return self
    }
}
