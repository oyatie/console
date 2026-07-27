package com.console.app

import android.app.Application
import com.console.app.data.offline.ConnectivityReplayScheduler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob

class ConsoleApplication : Application() {
    private val applicationScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    lateinit var container: AppContainer
        private set
    private lateinit var replayScheduler: ConnectivityReplayScheduler

    override fun onCreate() {
        super.onCreate()
        container = AppContainer(this)
        replayScheduler = ConnectivityReplayScheduler(
            context = this,
            scope = applicationScope,
            queue = container.offlineQueue,
            onReplayFinished = {
                container.evidence.uploadPending()
                container.workOrders.refreshToday()
            },
        )
        replayScheduler.start()
    }

    override fun onTerminate() {
        replayScheduler.stop()
        super.onTerminate()
    }
}
