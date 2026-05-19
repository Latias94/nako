package dev.taru.android.playback

import android.content.Context
import android.content.SharedPreferences
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

interface PlaybackPreferencesStore {
    fun loadCapabilities(serverProfileId: String): PlaybackCapabilities
    fun saveCapabilities(serverProfileId: String, capabilities: PlaybackCapabilities)
    fun clearCapabilities(serverProfileId: String)
}

class InMemoryPlaybackPreferencesStore : PlaybackPreferencesStore {
    private val capabilities = linkedMapOf<String, PlaybackCapabilities>()

    override fun loadCapabilities(serverProfileId: String): PlaybackCapabilities =
        capabilities[serverProfileId] ?: PlaybackCapabilities()

    override fun saveCapabilities(
        serverProfileId: String,
        capabilities: PlaybackCapabilities,
    ) {
        require(serverProfileId.isNotBlank()) { "serverProfileId must not be blank" }
        this.capabilities[serverProfileId] = capabilities.normalized()
    }

    override fun clearCapabilities(serverProfileId: String) {
        capabilities.remove(serverProfileId)
    }
}

class SharedPreferencesPlaybackPreferencesStore(
    private val preferences: SharedPreferences,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    },
) : PlaybackPreferencesStore {
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

    override fun loadCapabilities(serverProfileId: String): PlaybackCapabilities {
        val encoded = preferences.getString(preferenceKey(serverProfileId), null)
            ?: return PlaybackCapabilities()
        return runCatching {
            json.decodeFromString<StoredPlaybackCapabilities>(encoded).toDomain()
        }.getOrElse {
            clearCapabilities(serverProfileId)
            PlaybackCapabilities()
        }
    }

    override fun saveCapabilities(
        serverProfileId: String,
        capabilities: PlaybackCapabilities,
    ) {
        require(serverProfileId.isNotBlank()) { "serverProfileId must not be blank" }
        preferences.edit()
            .putString(
                preferenceKey(serverProfileId),
                json.encodeToString(capabilities.normalized().toStored()),
            )
            .commit()
    }

    override fun clearCapabilities(serverProfileId: String) {
        preferences.edit().remove(preferenceKey(serverProfileId)).commit()
    }

    private fun preferenceKey(serverProfileId: String): String =
        "capabilities:${encodeKeyPart(serverProfileId)}"

    private fun encodeKeyPart(value: String): String =
        value.encodeToByteArray().joinToString(separator = "") { byte ->
            "%02x".format(byte.toInt() and 0xff)
        }

    private companion object {
        const val preferencesName = "taru_playback_preferences"
    }
}

private fun PlaybackCapabilities.normalized(): PlaybackCapabilities =
    PlaybackCapabilities(
        directPlay = directPlay,
        containers = containers.cleaned(),
        videoCodecs = videoCodecs.cleaned(),
        audioCodecs = audioCodecs.cleaned(),
    )

private fun List<String>.cleaned(): List<String> =
    map(String::trim)
        .filter(String::isNotEmpty)
        .distinctBy(String::lowercase)

@Serializable
private data class StoredPlaybackCapabilities(
    val directPlay: Boolean? = null,
    val containers: List<String> = emptyList(),
    val videoCodecs: List<String> = emptyList(),
    val audioCodecs: List<String> = emptyList(),
) {
    fun toDomain(): PlaybackCapabilities =
        PlaybackCapabilities(
            directPlay = directPlay,
            containers = containers,
            videoCodecs = videoCodecs,
            audioCodecs = audioCodecs,
        ).normalized()
}

private fun PlaybackCapabilities.toStored(): StoredPlaybackCapabilities =
    StoredPlaybackCapabilities(
        directPlay = directPlay,
        containers = containers,
        videoCodecs = videoCodecs,
        audioCodecs = audioCodecs,
    )
