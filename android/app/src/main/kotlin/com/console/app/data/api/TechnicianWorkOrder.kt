package com.console.app.data.api

import com.console.api.client.model.PriorityLevel
import com.console.api.client.model.WorkOrderStatus
import com.console.app.data.offline.SyncState
import java.time.OffsetDateTime
import java.util.UUID

data class TechnicianWorkOrder(
    val id: UUID,
    val requestNo: String,
    val managementNo: String,
    val modelName: String,
    val customerName: String,
    val siteName: String,
    val priority: PriorityLevel,
    val prioritySort: Int,
    val status: WorkOrderStatus,
    val targetDueAt: OffsetDateTime?,
    val symptom: String?,
    val syncState: SyncState,
    val assigneeNames: List<String>,
)
