package com.console.app.data.offline

import com.console.api.client.model.SyncBatchRequest
import com.console.api.client.model.SyncBatchResponse

fun interface SyncGateway {
    suspend fun replay(deviceId: String, request: SyncBatchRequest): SyncBatchResponse
}
