package dev.taru.android.player

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.PlaybackRequestTarget

data class PlaybackLaunchRequest(
    val title: String,
    val request: TaruHttpRequest,
    val safeRequest: SafeRequestPreview,
) {
    override fun toString(): String =
        "PlaybackLaunchRequest(title=$title, safeRequest=$safeRequest)"
}

fun playbackLaunchRequest(
    title: String,
    target: PlaybackRequestTarget,
): PlaybackLaunchRequest =
    PlaybackLaunchRequest(
        title = title,
        request = target.request,
        safeRequest = target.safeRequest,
    )

