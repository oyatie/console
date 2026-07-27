package com.console.app.data.api

import com.console.api.client.model.SubmitReportRequest
import com.console.api.client.model.WorkResultType

data class ReportDraft(
    val resultType: WorkResultType,
    val diagnosis: String,
    val actionTaken: String,
) {
    fun toSubmitReportRequest(): SubmitReportRequest = SubmitReportRequest(
        resultType = resultType,
        diagnosis = diagnosis.trim(),
        actionTaken = actionTaken.trim(),
    )
}
