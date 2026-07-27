package com.console.app

import android.content.Context
import com.console.app.auth.CredentialManagerPasskeyClient
import com.console.app.auth.MobilePasskeyStepUpRepository
import com.console.app.auth.PasskeyAuthRepository
import com.console.app.data.api.GeneratedConsoleApiGateway
import com.console.app.data.collaboration.MobileOperationsRepository
import com.console.app.data.evidence.EvidenceRepository
import com.console.app.data.location.LocationConsentRepository
import com.console.app.data.local.ConsoleDatabase
import com.console.app.data.local.RoomMessengerOutboxStore
import com.console.app.data.local.RoomMobileNotificationStore
import com.console.app.data.local.RoomMobileOperationsCacheStore
import com.console.app.data.local.RoomMobileSensitiveActionStore
import com.console.app.data.local.RoomMutationQueueStore
import com.console.app.data.local.RoomWorkOrderStore
import com.console.app.data.messenger.MessengerRepository
import com.console.app.data.offline.OfflineQueueRepository
import com.console.app.data.session.DeviceIdStore
import com.console.app.data.session.SessionTokenStore
import com.console.app.data.workorders.WorkOrderRepository
import okhttp3.OkHttpClient

class AppContainer(context: Context) {
    private val appContext = context.applicationContext
    private val database = ConsoleDatabase.create(appContext)
    private val httpClient = OkHttpClient.Builder().build()
    val sessionTokenStore = SessionTokenStore(appContext)
    val deviceIdStore = DeviceIdStore(appContext)
    private val passkeyCredentialClient = CredentialManagerPasskeyClient()
    val apiGateway = GeneratedConsoleApiGateway(
        basePath = BuildConfig.API_BASE_URL,
        httpClient = httpClient,
        accessTokenProvider = { sessionTokenStore.accessToken() },
    )
    val offlineQueue = OfflineQueueRepository(
        store = RoomMutationQueueStore(database.mutations()),
        syncGateway = apiGateway,
        deviceIdProvider = deviceIdStore::getOrCreate,
    )
    val workOrders = WorkOrderRepository(
        api = apiGateway,
        localStore = RoomWorkOrderStore(database.workOrders()),
        queue = offlineQueue,
    )
    val evidence = EvidenceRepository(
        api = apiGateway,
        uploads = database.evidenceUploads(),
        httpClient = httpClient,
    )
    val messenger = MessengerRepository(
        gateway = apiGateway,
        outbox = RoomMessengerOutboxStore(database.messengerOutbox()),
    )
    val locationConsent = LocationConsentRepository(apiGateway)
    val mobileOperations = MobileOperationsRepository(
        gateway = apiGateway,
        cache = RoomMobileOperationsCacheStore(database.mobileOperationsSnapshots()),
        notificationStore = RoomMobileNotificationStore(database.mobileNotifications()),
        sensitiveActionStore = RoomMobileSensitiveActionStore(database.mobileSensitiveActions()),
    )
    val passkeyStepUp = MobilePasskeyStepUpRepository(
        api = apiGateway,
        credentialClient = passkeyCredentialClient,
    )
    val auth = PasskeyAuthRepository(
        api = apiGateway,
        credentialClient = passkeyCredentialClient,
        tokenStore = sessionTokenStore,
        deviceIdStore = deviceIdStore,
        appVersion = BuildConfig.VERSION_NAME,
    )
}
