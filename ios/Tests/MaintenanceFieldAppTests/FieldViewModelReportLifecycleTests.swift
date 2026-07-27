@testable import MaintenanceFieldApp
import Combine
import HTTPTypes
import MaintenanceAPIClient
import MaintenanceFieldCore
import OpenAPIRuntime
import XCTest

@MainActor
final class FieldViewModelReportLifecycleTests: XCTestCase {
    func testConfirmedReportRemainsSuccessfulAfterSuccessfulTodayRefresh() async throws {
        let refreshedWorkOrder = workOrder.applyingSubmittedReport(
            ReportDraft(resultType: .completed, diagnosis: "Resolved", actionTaken: "Replaced"),
            syncState: .synced
        )
        let gateway = ControllableWorkOrderGateway(todayWorkOrders: [refreshedWorkOrder])
        let viewModel = makeViewModel(gateway: gateway)
        viewModel.selectedWorkOrder = workOrder
        viewModel.diagnosis = "Resolved"
        viewModel.actionTaken = "Replaced"

        await viewModel.submitReport()

        let didRefresh = await gateway.refreshStarted()
        XCTAssertTrue(didRefresh)
        XCTAssertEqual(viewModel.today.count, 1)
        XCTAssertEqual(viewModel.today.first?.status, .reportSubmitted)
        XCTAssertEqual(viewModel.selectedWorkOrder?.status, .reportSubmitted)
        XCTAssertEqual(viewModel.selectedWorkOrder?.syncState, .synced)
        XCTAssertEqual(viewModel.messageKey, "report_submitted")
        XCTAssertFalse(viewModel.isLoading)
    }

    func testLaterPresentationActionWinsWhileSuccessfulRefreshCompletes() async {
        let refreshedWorkOrder = workOrder.applyingSubmittedReport(
            ReportDraft(resultType: .completed, diagnosis: "Resolved", actionTaken: "Replaced"),
            syncState: .synced
        )
        let gateway = ControllableWorkOrderGateway(
            todayWorkOrders: [refreshedWorkOrder],
            holdsRefresh: true
        )
        let viewModel = makeViewModel(gateway: gateway)
        viewModel.selectedWorkOrder = workOrder
        viewModel.diagnosis = "Resolved"
        viewModel.actionTaken = "Replaced"

        let submission = Task { await viewModel.submitReport() }
        await gateway.waitUntilRefreshStarted()

        viewModel.cameraCaptureFailed()
        await gateway.releaseRefresh()
        await submission.value

        XCTAssertEqual(viewModel.messageKey, "operation_failed")
        XCTAssertFalse(viewModel.isLoading)
    }

    func testConfirmedReportRemainsSuccessfulAfterFailedTodayRefresh() async throws {
        let gateway = ControllableWorkOrderGateway(
            listError: URLError(.networkConnectionLost),
            listDelayNanoseconds: 100_000_000
        )
        let viewModel = makeViewModel(gateway: gateway)
        viewModel.selectedWorkOrder = workOrder
        viewModel.diagnosis = "Resolved"
        viewModel.actionTaken = "Replaced"
        var loading: [Bool] = []
        let loadingToken = viewModel.$isLoading.sink { loading.append($0) }
        defer { _ = loadingToken }

        let submission = Task { await viewModel.submitReport() }
        await gateway.waitUntilRefreshStarted()
        let didStartRefresh = await gateway.refreshStarted()
        XCTAssertTrue(didStartRefresh)

        XCTAssertEqual(viewModel.selectedWorkOrder?.status, .reportSubmitted)
        XCTAssertEqual(viewModel.selectedWorkOrder?.syncState, .synced)
        XCTAssertEqual(viewModel.messageKey, "report_submitted")
        XCTAssertTrue(viewModel.isLoading)
        XCTAssertTrue(loading.suffix(3).elementsEqual([true, false, true]))

        await submission.value

        XCTAssertEqual(viewModel.messageKey, "report_submitted")
        XCTAssertTrue(loading.suffix(4).elementsEqual([true, false, true, false]))
    }

    func testQueuedReportNeverPublishesConfirmedSuccess() async throws {
        let gateway = ControllableWorkOrderGateway(reportError: URLError(.timedOut))
        let viewModel = makeViewModel(gateway: gateway)
        viewModel.selectedWorkOrder = workOrder
        viewModel.diagnosis = "Resolved"
        viewModel.actionTaken = "Replaced"

        await viewModel.submitReport()

        XCTAssertEqual(viewModel.selectedWorkOrder?.status, .inProgress)
        XCTAssertEqual(viewModel.selectedWorkOrder?.syncState, .pending)
        XCTAssertEqual(viewModel.messageKey, "offline_queued")
        XCTAssertFalse(viewModel.isLoading)
    }

    func testRejectedReportPreservesFailureState() async throws {
        let gateway = ControllableWorkOrderGateway(reportError: MaintenanceGatewayError.apiResponse(operation: "submit", statusCode: 422))
        let viewModel = makeViewModel(gateway: gateway)
        viewModel.selectedWorkOrder = workOrder
        viewModel.diagnosis = "Resolved"
        viewModel.actionTaken = "Replaced"

        await viewModel.submitReport()

        XCTAssertEqual(viewModel.selectedWorkOrder?.status, .inProgress)
        XCTAssertEqual(viewModel.messageKey, "operation_failed")
        XCTAssertFalse(viewModel.isLoading)
    }

    private func makeViewModel(gateway: ControllableWorkOrderGateway) -> FieldViewModel {
        let queue = OfflineQueueRepository(store: InMemoryMutationQueueStore(), syncGateway: gateway, deviceIDProvider: { "test-device" })
        let repository = WorkOrderRepository(gateway: gateway, cache: WorkOrderCacheStore(), offlineQueue: queue)
        let tokenProvider = CurrentTokenProvider(accessToken: "test-access-token")
        let auxiliaryGateway = GeneratedMaintenanceAPIGateway(
            serverURL: URL(string: "https://api.example.com")!,
            tokenProvider: tokenProvider,
            sessionStore: InMemorySessionTokenStore(),
            transport: SuccessfulRefreshTransport()
        )
        let live = FieldAppContainer.live()
        return FieldViewModel(container: FieldAppContainer(
            authRepository: live.authRepository,
            workOrderRepository: repository,
            evidenceRepository: EvidenceRepository(gateway: auxiliaryGateway, store: EmptyEvidenceUploadStore()),
            messengerRepository: MessengerRepository(gateway: auxiliaryGateway, outbox: InMemoryMessengerOutboxStore()),
            locationConsentRepository: LocationConsentRepository(gateway: auxiliaryGateway),
            mobileOperationsRepository: live.mobileOperationsRepository,
            passkeyStepUpRepository: live.passkeyStepUpRepository
        ))
    }

    private var workOrder: TechnicianWorkOrder {
        TechnicianWorkOrder(id: "00000000-0000-0000-0000-000000000111", requestNo: "WO-1", managementNo: "EQ-1", modelName: "Model", customerName: "Customer", siteName: "Site", priority: .p2, status: .inProgress, resultType: .unknown, targetDueAt: nil, createdAt: .distantPast, updatedAt: .distantPast, assigneeNames: [], syncState: .synced)
    }
}

private actor InMemorySessionTokenStore: SessionTokenStore {
    func load() -> AuthTokens? { nil }
    func consumeForRefresh() throws -> AuthTokens? { nil }
    func save(_ tokens: AuthTokens) throws {}
    func clear() throws {}
}

private actor EmptyEvidenceUploadStore: EvidenceUploadStore {
    func upsert(_ upload: PendingEvidenceUpload) throws {}
    func pending() -> [PendingEvidenceUpload] { [] }
    func markSynced(id: String) throws {}
    func markRetrying(id: String, message: String, retryAttemptCount: Int, nextRetryAt: Date) throws {}
    func markFailed(id: String, message: String) throws {}
}

private struct SuccessfulRefreshTransport: ClientTransport {
    func send(
        _ request: HTTPRequest,
        body: HTTPBody?,
        baseURL: URL,
        operationID: String
    ) async throws -> (HTTPResponse, HTTPBody?) {
        let bytes = Array(
            """
            {
              "consent_id": "00000000-0000-0000-0000-000000000201",
              "user_id": "00000000-0000-0000-0000-000000000202",
              "branch_id": "00000000-0000-0000-0000-000000000203",
              "state": "NO_RECORD",
              "may_collect": false
            }
            """.utf8
        )
        let responseBody = HTTPBody(AsyncStream { continuation in
            continuation.yield(bytes[...])
            continuation.finish()
        }, length: .known(Int64(bytes.count)))
        return (
            HTTPResponse(status: .ok, headerFields: [.contentType: "application/json"]),
            responseBody
        )
    }
}

private actor ControllableWorkOrderGateway: WorkOrderGateway {
    let reportError: Error?
    let listError: Error?
    let listDelayNanoseconds: UInt64
    let todayWorkOrders: [TechnicianWorkOrder]
    let holdsRefresh: Bool
    private var didStartRefresh = false
    private var refreshStartWaiters: [CheckedContinuation<Void, Never>] = []
    private var refreshReleaseContinuation: CheckedContinuation<Void, Never>?

    init(
        reportError: Error? = nil,
        listError: Error? = nil,
        listDelayNanoseconds: UInt64 = 0,
        todayWorkOrders: [TechnicianWorkOrder] = [],
        holdsRefresh: Bool = false
    ) {
        self.reportError = reportError
        self.listError = listError
        self.listDelayNanoseconds = listDelayNanoseconds
        self.todayWorkOrders = todayWorkOrders
        self.holdsRefresh = holdsRefresh
    }

    func listTodayWorkOrders() async throws -> [TechnicianWorkOrder] {
        didStartRefresh = true
        refreshStartWaiters.forEach { $0.resume() }
        refreshStartWaiters.removeAll()
        if holdsRefresh {
            await withCheckedContinuation { continuation in
                refreshReleaseContinuation = continuation
            }
        }
        if listDelayNanoseconds > 0 {
            try await Task.sleep(nanoseconds: listDelayNanoseconds)
        }
        if let listError { throw listError }
        return todayWorkOrders
    }

    func refreshStarted() -> Bool { didStartRefresh }

    func waitUntilRefreshStarted() async {
        guard didStartRefresh == false else { return }
        await withCheckedContinuation { continuation in
            refreshStartWaiters.append(continuation)
        }
    }

    func releaseRefresh() {
        refreshReleaseContinuation?.resume()
        refreshReleaseContinuation = nil
    }

    func getWorkOrderDetail(id: Components.Schemas.Uuid) async throws -> TechnicianWorkOrder { throw URLError(.unsupportedURL) }
    func startWorkOrder(id: Components.Schemas.Uuid) async throws { throw URLError(.unsupportedURL) }
    func submitReport(id: Components.Schemas.Uuid, draft: ReportDraft) async throws {
        if let reportError { throw reportError }
    }
    func replay(deviceID: String, request: Components.Schemas.SyncBatchRequest) async throws -> Components.Schemas.SyncBatchResponse {
        Components.Schemas.SyncBatchResponse(syncId: "sync", results: [])
    }
}
