package com.console.app.data.offline

import com.console.api.client.model.PriorityLevel
import com.console.api.client.model.SyncBatchResponse
import com.console.api.client.model.SyncOperationKind
import com.console.api.client.model.SyncOperationResult
import com.console.api.client.model.SyncOperationStatus
import com.console.api.client.model.WorkOrderStatus
import com.console.api.client.model.WorkOrderSummary
import com.console.api.client.model.WorkResultType
import java.io.IOException
import java.time.OffsetDateTime
import java.util.UUID
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.test.runTest

class OfflineQueueRepositoryTest {
    private val workOrderId = UUID.fromString("00000000-0000-0000-0000-000000000101")

    @Test
    fun enqueueOfflineStartRetriesSameRequestIdAndAcceptsCachedSyncResult() = runTest {
        val store = InMemoryMutationStore()
        val sync = RecordingSyncGateway()
        val repository = OfflineQueueRepository(
            store = store,
            syncGateway = sync,
            deviceIdProvider = { "device-a" },
            requestIdFactory = FixedRequestIdFactory("01HVSTART000000000000000000"),
            syncIdFactory = FixedRequestIdFactory("sync-a"),
            clock = FixedClock(OffsetDateTime.parse("2026-06-12T09:00:00Z")),
        )

        val requestId = repository.enqueueStart(workOrderId)

        assertEquals("01HVSTART000000000000000000", requestId)
        assertEquals(SyncState.PENDING, store.get(requestId)?.syncState)

        sync.failNext = IOException("offline")
        val offlineSummary = repository.replayPending()

        assertEquals(ReplaySummary(attempted = 1, applied = 0, failed = 1, cached = 0), offlineSummary)
        assertEquals(requestId, store.pending().single().requestId)

        sync.nextResponse = SyncBatchResponse(
            syncId = "sync-a",
            results = listOf(
                SyncOperationResult(
                    requestId = requestId,
                    operation = SyncOperationKind.WORK_ORDER_START,
                    status = SyncOperationStatus.APPLIED,
                    httpStatus = 200,
                    replayed = true,
                    result = summary(status = WorkOrderStatus.IN_PROGRESS),
                ),
            ),
        )

        val cachedSummary = repository.replayPending()

        assertEquals(ReplaySummary(attempted = 1, applied = 1, failed = 0, cached = 1), cachedSummary)
        assertTrue(store.get(requestId)?.isSynced == true)
        assertEquals(requestId, sync.requests.map { it.operations.single().requestId }.distinct().single())
        assertEquals("device-a", sync.deviceIds.distinct().single())
    }

    @Test
    fun failedOperationSurfacesQueueResultWithoutDroppingMutation() = runTest {
        val store = InMemoryMutationStore()
        val sync = RecordingSyncGateway()
        val repository = OfflineQueueRepository(
            store = store,
            syncGateway = sync,
            deviceIdProvider = { "device-a" },
            requestIdFactory = FixedRequestIdFactory("01HVREPORT0000000000000000"),
            syncIdFactory = FixedRequestIdFactory("sync-b"),
            clock = FixedClock(OffsetDateTime.parse("2026-06-12T10:00:00Z")),
        )
        val requestId = repository.enqueueReport(
            workOrderId = workOrderId,
            resultType = WorkResultType.TEMPORARY_ACTION,
            diagnosis = "유압 누유",
            actionTaken = "호스 교체 전 임시 조치",
        )
        sync.nextResponse = SyncBatchResponse(
            syncId = "sync-b",
            results = listOf(
                SyncOperationResult(
                    requestId = requestId,
                    operation = SyncOperationKind.WORK_ORDER_REPORT,
                    status = SyncOperationStatus.FAILED,
                    httpStatus = 409,
                    replayed = false,
                    error = com.console.api.client.model.SyncError(
                        code = "conflict",
                        message = "server wins",
                    ),
                ),
            ),
        )

        val summary = repository.replayPending()

        assertEquals(ReplaySummary(attempted = 1, applied = 0, failed = 1, cached = 0), summary)
        assertEquals(SyncState.FAILED, store.get(requestId)?.syncState)
        assertFalse(store.get(requestId)?.lastError.isNullOrBlank())
    }

    private fun summary(status: WorkOrderStatus) = WorkOrderSummary(
        id = workOrderId,
        requestNo = "20260612-001",
        branchId = UUID.fromString("00000000-0000-0000-0000-000000000201"),
        equipmentId = UUID.fromString("00000000-0000-0000-0000-000000000301"),
        customerId = UUID.fromString("00000000-0000-0000-0000-000000000401"),
        siteId = UUID.fromString("00000000-0000-0000-0000-000000000501"),
        status = status,
        priority = PriorityLevel.P1,
        resultType = WorkResultType.UNKNOWN,
        // Required-but-nullable on the wire; the field app has no domain field
        // for either, so the queue round-trips them as sent.
        maintenanceType = null,
        maintenanceCause = null,
        evidenceVerified = false,
    )
}
