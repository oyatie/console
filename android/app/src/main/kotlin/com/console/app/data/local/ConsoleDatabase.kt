package com.console.app.data.local

import android.content.Context
import androidx.room.AutoMigration
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase

@Database(
    entities = [
        WorkOrderEntity::class,
        MutationEntity::class,
        EvidenceUploadEntity::class,
        MessengerOutboxEntity::class,
        MobileOperationsSnapshotEntity::class,
        MobileNotificationEntity::class,
        MobileSensitiveActionEntity::class,
    ],
    version = 5,
    exportSchema = true,
    autoMigrations = [
        AutoMigration(from = 1, to = 2),
        AutoMigration(from = 2, to = 3),
        AutoMigration(from = 3, to = 4),
        AutoMigration(from = 4, to = 5),
    ],
)
abstract class ConsoleDatabase : RoomDatabase() {
    abstract fun workOrders(): WorkOrderDao

    abstract fun mutations(): MutationDao

    abstract fun evidenceUploads(): EvidenceUploadDao

    abstract fun messengerOutbox(): MessengerOutboxDao

    abstract fun mobileOperationsSnapshots(): MobileOperationsSnapshotDao

    abstract fun mobileNotifications(): MobileNotificationDao

    abstract fun mobileSensitiveActions(): MobileSensitiveActionDao

    companion object {
        fun create(context: Context): ConsoleDatabase =
            Room.databaseBuilder(context, ConsoleDatabase::class.java, "console-field.db")
                .fallbackToDestructiveMigration(false)
                .build()
    }
}
