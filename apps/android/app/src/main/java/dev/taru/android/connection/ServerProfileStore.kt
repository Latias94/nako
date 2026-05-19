package dev.taru.android.connection

import android.content.Context
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

interface ServerProfileStore {
    fun load(): ServerProfileSnapshot
    fun save(snapshot: ServerProfileSnapshot)
}

class InMemoryServerProfileStore(
    initialSnapshot: ServerProfileSnapshot = ServerProfileSnapshot(),
) : ServerProfileStore {
    private var snapshot = initialSnapshot

    override fun load(): ServerProfileSnapshot = snapshot

    override fun save(snapshot: ServerProfileSnapshot) {
        this.snapshot = snapshot
    }
}

class SharedPreferencesServerProfileStore(
    context: Context,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    },
) : ServerProfileStore {
    private val preferences = context.applicationContext.getSharedPreferences(
        "taru_server_profiles",
        Context.MODE_PRIVATE,
    )

    override fun load(): ServerProfileSnapshot {
        val encoded = preferences.getString(snapshotKey, null) ?: return ServerProfileSnapshot()
        return runCatching {
            json.decodeFromString<ServerProfileSnapshot>(encoded)
        }.getOrDefault(ServerProfileSnapshot())
    }

    override fun save(snapshot: ServerProfileSnapshot) {
        preferences.edit().putString(snapshotKey, json.encodeToString(snapshot)).commit()
    }

    private companion object {
        const val snapshotKey = "snapshot"
    }
}
