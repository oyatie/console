package com.console.app.data.location

import com.console.api.client.model.LocationConsentStatus
import com.console.api.client.model.LocationPingRequest
import com.console.app.data.api.ConsoleApiGateway
import java.time.OffsetDateTime

class LocationConsentRepository(
    private val api: ConsoleApiGateway,
) {
    suspend fun status(): LocationConsentStatus =
        api.getLocationConsentStatus()

    suspend fun grant(): LocationConsentStatus =
        api.grantLocationConsent()

    suspend fun suspend(): LocationConsentStatus =
        api.suspendLocationConsent()

    suspend fun resume(): LocationConsentStatus =
        api.resumeLocationConsent()

    suspend fun withdraw(): LocationConsentStatus =
        api.withdrawLocationConsent()

    suspend fun recordPingIfAllowed(
        state: GpsCollectionState,
        latitude: Double,
        longitude: Double,
        accuracyM: Double?,
        recordedAt: OffsetDateTime,
    ): Boolean {
        if (!state.mayCollect) {
            return false
        }

        api.recordLocationPing(
            LocationPingRequest(
                latitude = latitude,
                longitude = longitude,
                recordedAt = recordedAt,
                onDuty = state.onDuty,
                branchId = null,
                accuracyM = accuracyM,
            ),
        )
        return true
    }
}
