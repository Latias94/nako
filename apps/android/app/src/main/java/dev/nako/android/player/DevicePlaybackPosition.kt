package dev.nako.android.player

import android.content.Context
import android.content.SharedPreferences
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

data class DevicePlaybackPositionKey(
    val serverProfileId: String,
    val mediaItemId: String,
    val sourceId: String,
) {
    init {
        require(serverProfileId.isNotBlank()) { "serverProfileId must not be blank" }
        require(mediaItemId.isNotBlank()) { "mediaItemId must not be blank" }
        require(sourceId.isNotBlank()) { "sourceId must not be blank" }
    }
}

data class DevicePlaybackPosition(
    val key: DevicePlaybackPositionKey,
    val positionMs: Long,
    val durationMs: Long? = null,
    val updatedAtMillis: Long,
)

interface DevicePlaybackPositionStore {
    fun load(key: DevicePlaybackPositionKey): DevicePlaybackPosition?
    fun save(position: DevicePlaybackPosition)
    fun clear(key: DevicePlaybackPositionKey)
}

class InMemoryDevicePlaybackPositionStore : DevicePlaybackPositionStore {
    private val positions = linkedMapOf<DevicePlaybackPositionKey, DevicePlaybackPosition>()

    override fun load(key: DevicePlaybackPositionKey): DevicePlaybackPosition? = positions[key]

    override fun save(position: DevicePlaybackPosition) {
        if (position.positionMs <= 0L) {
            positions.remove(position.key)
            return
        }
        positions[position.key] = position
    }

    override fun clear(key: DevicePlaybackPositionKey) {
        positions.remove(key)
    }
}

class SharedPreferencesDevicePlaybackPositionStore(
    private val preferences: SharedPreferences,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    },
) : DevicePlaybackPositionStore {
    constructor(
        context: Context,
        json: Json = Json {
            ignoreUnknownKeys = true
            encodeDefaults = true
        },
    ) : this(
        preferences = context.applicationContext.getSharedPreferences(
            preferencesName,
            Context.MODE_PRIVATE,
        ),
        json = json,
    )

    override fun load(key: DevicePlaybackPositionKey): DevicePlaybackPosition? {
        val encoded = preferences.getString(preferenceKey(key), null) ?: return null
        return runCatching {
            json.decodeFromString<StoredDevicePlaybackPosition>(encoded).toDomain(key)
                ?: run {
                    clear(key)
                    null
                }
        }.getOrElse {
            clear(key)
            null
        }
    }

    override fun save(position: DevicePlaybackPosition) {
        if (position.positionMs <= 0L) {
            clear(position.key)
            return
        }

        preferences.edit()
            .putString(preferenceKey(position.key), json.encodeToString(position.toStored()))
            .commit()
    }

    override fun clear(key: DevicePlaybackPositionKey) {
        preferences.edit().remove(preferenceKey(key)).commit()
    }

    private fun preferenceKey(key: DevicePlaybackPositionKey): String =
        listOf(key.serverProfileId, key.mediaItemId, key.sourceId)
            .joinToString(separator = ":") { encodeKeyPart(it) }

    private fun encodeKeyPart(value: String): String =
        value.encodeToByteArray().joinToString(separator = "") { byte ->
            "%02x".format(byte.toInt() and 0xff)
        }

    private companion object {
        const val preferencesName = "nako_device_playback_positions"
    }
}

@Serializable
private data class StoredDevicePlaybackPosition(
    val positionMs: Long,
    val durationMs: Long? = null,
    val updatedAtMillis: Long,
) {
    fun toDomain(key: DevicePlaybackPositionKey): DevicePlaybackPosition? {
        if (positionMs <= 0L) {
            return null
        }
        return DevicePlaybackPosition(
            key = key,
            positionMs = positionMs,
            durationMs = durationMs,
            updatedAtMillis = updatedAtMillis,
        )
    }
}

private fun DevicePlaybackPosition.toStored(): StoredDevicePlaybackPosition =
    StoredDevicePlaybackPosition(
        positionMs = positionMs,
        durationMs = durationMs,
        updatedAtMillis = updatedAtMillis,
    )
