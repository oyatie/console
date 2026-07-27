package com.console.app.data.messenger

import com.console.api.client.model.MessengerMessageSummary
import com.console.api.client.model.MessengerThreadSummary

fun MessengerThreadSummary.toMessengerThread(): MessengerThread =
    MessengerThread(
        id = id,
        kind = kind,
        branchId = branchId,
        title = title,
        workOrderId = workOrderId,
        lastMessageId = lastMessageId,
        lastMessageAt = lastMessageAt,
        memberCount = memberCount,
        createdAt = createdAt,
        updatedAt = updatedAt,
    )

fun MessengerMessageSummary.toMessengerMessage(): MessengerMessage =
    MessengerMessage(
        id = id,
        threadId = threadId,
        branchId = branchId,
        senderId = senderId,
        body = body,
        readCount = readCount,
        readTargetCount = readTargetCount,
        attachmentEvidenceIds = attachmentEvidenceIds,
        sentAt = sentAt,
        createdAt = createdAt,
    )
