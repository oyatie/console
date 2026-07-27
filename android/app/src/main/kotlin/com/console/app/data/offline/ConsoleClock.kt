package com.console.app.data.offline

import java.time.OffsetDateTime
import java.time.ZoneOffset

fun interface ConsoleClock {
    fun now(): OffsetDateTime
}

object SystemFieldClock : ConsoleClock {
    override fun now(): OffsetDateTime = OffsetDateTime.now(ZoneOffset.UTC)
}
