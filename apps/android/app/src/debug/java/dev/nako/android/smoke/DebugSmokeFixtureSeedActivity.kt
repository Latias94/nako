package dev.nako.android.smoke

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.os.Bundle
import dev.nako.android.connection.AndroidSecureTokenVault
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.SharedPreferencesServerProfileStore
import dev.nako.android.playback.PlaybackCapabilities
import dev.nako.android.playback.SharedPreferencesPlaybackPreferencesStore
import dev.nako.android.player.DevicePlaybackPosition
import dev.nako.android.player.DevicePlaybackPositionKey
import dev.nako.android.player.SharedPreferencesDevicePlaybackPositionStore
import dev.nako.sdk.NAKO_API_VERSION
import java.net.URI

class DebugSmokeFixtureSeedActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        runCatching {
            val request = debugSmokeFixtureSeedRequest(
                baseUrl = intent.getStringExtra(EXTRA_BASE_URL),
                accessToken = intent.getStringExtra(EXTRA_ACCESS_TOKEN),
                displayName = intent.getStringExtra(EXTRA_DISPLAY_NAME),
                checkedAtMillis = intent.getLongExtra(
                    EXTRA_CHECKED_AT_MILLIS,
                    System.currentTimeMillis(),
                ),
                resumeMediaItemId = intent.getStringExtra(EXTRA_RESUME_MEDIA_ITEM_ID),
                resumeSourceId = intent.getStringExtra(EXTRA_RESUME_SOURCE_ID),
                resumePositionMs = intent.longExtraOrNull(EXTRA_RESUME_POSITION_MS),
                resumeDurationMs = intent.longExtraOrNull(EXTRA_RESUME_DURATION_MS),
                forceRemux = intent.booleanExtraOrNull(EXTRA_FORCE_REMUX),
            )
            seedDebugSmokeFixture(this, request)
            setResult(RESULT_OK)
        }.onFailure { error ->
            setResult(
                RESULT_CANCELED,
                Intent().putExtra(EXTRA_ERROR, error.message.orEmpty()),
            )
        }
        finish()
    }

    companion object {
        const val EXTRA_BASE_URL = "base_url"
        const val EXTRA_ACCESS_TOKEN = "access_token"
        const val EXTRA_DISPLAY_NAME = "display_name"
        const val EXTRA_CHECKED_AT_MILLIS = "checked_at_millis"
        const val EXTRA_RESUME_MEDIA_ITEM_ID = "resume_media_item_id"
        const val EXTRA_RESUME_SOURCE_ID = "resume_source_id"
        const val EXTRA_RESUME_POSITION_MS = "resume_position_ms"
        const val EXTRA_RESUME_DURATION_MS = "resume_duration_ms"
        const val EXTRA_FORCE_REMUX = "force_remux"
        const val EXTRA_ERROR = "error"
    }
}

internal fun seedDebugSmokeFixture(
    context: Context,
    request: DebugSmokeFixtureSeedRequest,
) {
    val snapshot = debugSmokeFixtureProfileSnapshot(request)
    SharedPreferencesServerProfileStore(context).save(snapshot)
    AndroidSecureTokenVault(context).saveToken(request.tokenReference, request.accessToken)
    val playbackPreferences = SharedPreferencesPlaybackPreferencesStore(context)
    if (request.forceRemux) {
        playbackPreferences.saveCapabilities(
            request.profileId,
            PlaybackCapabilities(
                directPlay = true,
                containers = listOf("mp4"),
                videoCodecs = listOf("h264"),
                audioCodecs = listOf("aac"),
            ),
        )
    } else {
        playbackPreferences.clearCapabilities(request.profileId)
    }
    request.resumePosition?.let { resume ->
        SharedPreferencesDevicePlaybackPositionStore(context).save(
            DevicePlaybackPosition(
                key = DevicePlaybackPositionKey(
                    serverProfileId = request.profileId,
                    mediaItemId = resume.mediaItemId,
                    sourceId = resume.sourceId,
                ),
                positionMs = resume.positionMs,
                durationMs = resume.durationMs,
                updatedAtMillis = request.checkedAtMillis,
            ),
        )
    }
}

internal data class DebugSmokeFixtureSeedRequest(
    val baseUrl: String,
    val accessToken: String,
    val displayName: String,
    val checkedAtMillis: Long,
    val resumePosition: DebugSmokeFixtureResumePosition? = null,
    val forceRemux: Boolean = false,
) {
    val profileId: String = "server-1"
    val tokenReference: String = "server-token:$profileId"
}

internal data class DebugSmokeFixtureResumePosition(
    val mediaItemId: String,
    val sourceId: String,
    val positionMs: Long,
    val durationMs: Long?,
)

internal fun debugSmokeFixtureSeedRequest(
    baseUrl: String?,
    accessToken: String?,
    displayName: String?,
    checkedAtMillis: Long,
    resumeMediaItemId: String? = null,
    resumeSourceId: String? = null,
    resumePositionMs: Long? = null,
    resumeDurationMs: Long? = null,
    forceRemux: Boolean? = null,
): DebugSmokeFixtureSeedRequest {
    val normalizedBaseUrl = normalizeSmokeFixtureBaseUrl(baseUrl)
    val token = accessToken?.trim().orEmpty()
    require(token.isNotBlank()) { "Smoke fixture access token is required." }

    return DebugSmokeFixtureSeedRequest(
        baseUrl = normalizedBaseUrl,
        accessToken = token,
        displayName = displayName?.trim().takeUnless { it.isNullOrBlank() } ?: "Smoke Server",
        checkedAtMillis = checkedAtMillis,
        forceRemux = forceRemux ?: false,
        resumePosition = debugSmokeFixtureResumePosition(
            mediaItemId = resumeMediaItemId,
            sourceId = resumeSourceId,
            positionMs = resumePositionMs,
            durationMs = resumeDurationMs,
        ),
    )
}

internal fun debugSmokeFixtureProfileSnapshot(
    request: DebugSmokeFixtureSeedRequest,
): ServerProfileSnapshot =
    ServerProfileSnapshot(
        profiles = listOf(
            ServerProfile(
                id = request.profileId,
                displayName = request.displayName,
                baseUrl = request.baseUrl,
                tokenReference = request.tokenReference,
                lastObservedApiVersion = NAKO_API_VERSION,
                lastSuccessfulConnectionAtMillis = request.checkedAtMillis,
                lastPublicError = null,
            ),
        ),
        activeProfileId = request.profileId,
    )

private fun debugSmokeFixtureResumePosition(
    mediaItemId: String?,
    sourceId: String?,
    positionMs: Long?,
    durationMs: Long?,
): DebugSmokeFixtureResumePosition? {
    val hasResumeInput =
        !mediaItemId.isNullOrBlank() ||
            !sourceId.isNullOrBlank() ||
            positionMs != null ||
            durationMs != null
    if (!hasResumeInput) {
        return null
    }

    val normalizedMediaItemId = mediaItemId?.trim().orEmpty()
    val normalizedSourceId = sourceId?.trim().orEmpty()
    require(normalizedMediaItemId.isNotBlank()) {
        "Smoke fixture resume Media Item id is required when resume state is provided."
    }
    require(normalizedSourceId.isNotBlank()) {
        "Smoke fixture resume Media Source id is required when resume state is provided."
    }

    val normalizedPositionMs = requireNotNull(positionMs) {
        "Smoke fixture resume position is required when resume state is provided."
    }
    require(normalizedPositionMs > 0L) {
        "Smoke fixture resume position must be positive."
    }

    val normalizedDurationMs = durationMs?.also {
        require(it > 0L) { "Smoke fixture resume duration must be positive when provided." }
    }

    return DebugSmokeFixtureResumePosition(
        mediaItemId = normalizedMediaItemId,
        sourceId = normalizedSourceId,
        positionMs = normalizedPositionMs,
        durationMs = normalizedDurationMs,
    )
}

private fun normalizeSmokeFixtureBaseUrl(input: String?): String {
    val trimmed = input?.trim()?.trimEnd('/').orEmpty()
    require(trimmed.isNotBlank()) { "Smoke fixture base URL is required." }

    val uri = runCatching { URI(trimmed) }.getOrNull()
        ?: throw IllegalArgumentException("Smoke fixture base URL must be valid.")
    val scheme = uri.scheme?.lowercase()
    require(scheme == "http" || scheme == "https") {
        "Smoke fixture base URL must use HTTP or HTTPS."
    }
    require(!uri.host.isNullOrBlank()) { "Smoke fixture base URL must include a host." }

    return uri.toString().trimEnd('/')
}

private fun Intent.longExtraOrNull(name: String): Long? =
    if (hasExtra(name)) getLongExtra(name, 0L) else null

private fun Intent.booleanExtraOrNull(name: String): Boolean? =
    if (hasExtra(name)) getBooleanExtra(name, false) else null
