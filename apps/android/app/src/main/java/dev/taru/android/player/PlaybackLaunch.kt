package dev.taru.android.player

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget

data class PlaybackLaunchRequest(
    val title: String,
    val request: TaruHttpRequest,
    val safeRequest: SafeRequestPreview,
    val serverProfileId: String,
    val mediaItemId: String,
    val sourceId: String,
    val playbackMode: ClientPlaybackMode,
    val sessionId: String? = null,
    val resumePositionMs: Long? = null,
) {
    val positionKey: DevicePlaybackPositionKey =
        DevicePlaybackPositionKey(
            serverProfileId = serverProfileId,
            mediaItemId = mediaItemId,
            sourceId = sourceId,
        )

    override fun toString(): String =
        "PlaybackLaunchRequest(title=$title, safeRequest=$safeRequest, serverProfileId=$serverProfileId, mediaItemId=$mediaItemId, sourceId=$sourceId, playbackMode=$playbackMode, sessionId=$sessionId, resumePositionMs=$resumePositionMs)"
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
): PlaybackLaunchRequest =
    PlaybackLaunchRequest(
        title = title,
        request = target.request,
        safeRequest = target.safeRequest,
        serverProfileId = serverProfileId,
        mediaItemId = mediaItemId,
        sourceId = sourceId,
        playbackMode = playbackMode,
        sessionId = sessionId,
        resumePositionMs = resumePositionMs,
    )
