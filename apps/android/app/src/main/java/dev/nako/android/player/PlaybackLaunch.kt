package dev.nako.android.player

import dev.nako.android.connection.SafeRequestPreview
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget

enum class PlaybackResumeSource {
    UserPlaybackState,
    DeviceLocal,
}

data class ResumePlaybackPosition(
    val positionMs: Long,
    val source: PlaybackResumeSource,
) {
    init {
        require(positionMs > 0L) { "positionMs must be positive" }
    }
}

data class PlaybackLaunchRequest(
    val title: String,
    val request: PlaybackRequestDescriptor,
    val serverProfileId: String,
    val mediaItemId: String,
    val sourceId: String,
    val playbackMode: ClientPlaybackMode,
    val sessionId: String? = null,
    val resumePositionMs: Long? = null,
    val resumeSource: PlaybackResumeSource? = null,
) {
    val safeRequest: SafeRequestPreview
        get() = request.safeRequest

    val positionKey: DevicePlaybackPositionKey =
        DevicePlaybackPositionKey(
            serverProfileId = serverProfileId,
            mediaItemId = mediaItemId,
            sourceId = sourceId,
        )

    override fun toString(): String =
        "PlaybackLaunchRequest(title=$title, safeRequest=$safeRequest, serverProfileId=$serverProfileId, mediaItemId=$mediaItemId, sourceId=$sourceId, playbackMode=$playbackMode, sessionId=$sessionId, resumePositionMs=$resumePositionMs, resumeSource=$resumeSource)"

    fun authenticatedRequest(accessToken: String): NakoHttpRequest =
        request.authenticatedRequest(accessToken)
}

fun playbackLaunchRequest(
    title: String,
    target: PlaybackRequestTarget,
    serverProfileId: String,
    mediaItemId: String,
    sourceId: String,
    playbackMode: ClientPlaybackMode,
    sessionId: String? = null,
    resumePositionMs: Long? = null,
    resumeSource: PlaybackResumeSource? = null,
): PlaybackLaunchRequest =
    PlaybackLaunchRequest(
        title = title,
        request = target.request,
        serverProfileId = serverProfileId,
        mediaItemId = mediaItemId,
        sourceId = sourceId,
        playbackMode = playbackMode,
        sessionId = sessionId,
        resumePositionMs = resumePositionMs,
        resumeSource = resumePositionMs
            ?.takeIf { it > 0L }
            ?.let { resumeSource ?: PlaybackResumeSource.DeviceLocal },
    )
