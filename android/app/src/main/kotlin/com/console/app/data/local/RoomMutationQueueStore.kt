package com.console.app.data.local

import com.console.app.data.offline.MutationQueueStore
import com.console.app.data.offline.QueuedMutation

class RoomMutationQueueStore(private val dao: MutationDao) : MutationQueueStore {
    override suspend fun upsert(mutation: QueuedMutation) {
        dao.upsert(mutation.toEntity())
    }

    override suspend fun pending(): List<QueuedMutation> =
        dao.pending().map { it.toDomain() }

    override suspend fun get(requestId: String): QueuedMutation? =
        dao.get(requestId)?.toDomain()

    override suspend fun markSynced(requestId: String, serverReplayed: Boolean) {
        dao.markSynced(requestId, serverReplayed)
    }

    override suspend fun markFailed(requestId: String, message: String) {
        dao.markFailed(requestId, message)
    }
}
