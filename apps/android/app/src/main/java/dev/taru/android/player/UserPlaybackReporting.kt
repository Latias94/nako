package dev.taru.android.player

import dev.taru.android.userplayback.SetWatchedStateRequest
import dev.taru.android.userplayback.UpdatePlaybackProgressRequest

internal sealed interface UserPlaybackStateReport {
    val itemId: String

    data class Progress(
        override val itemId: String,
        val request: UpdatePlaybackProgressRequest,
    ) : UserPlaybackStateReport

    data class Watched(
        override val itemId: String,
        val request: SetWatchedStateRequest,
    ) : UserPlaybackStateReport
}

internal fun userPlaybackStateReport(
    launch: PlaybackLaunchRequest,
    isEnded: Boolean,
    positionMs: Long,
    durationMs: Long?,
): UserPlaybackStateReport? {
    val knownDurationMs = durationMs?.takeIf { it > 0L }
    return if (isEnded) {
        UserPlaybackStateReport.Watched(
            itemId = launch.mediaItemId,
            request = SetWatchedStateRequest(
                watched = true,
                sourceId = launch.sourceId,
                positionMs = knownDurationMs ?: positionMs.takeIf { it > 0L },
                durationMs = knownDurationMs,
            ),
        )
    } else {
        val positivePositionMs = positionMs.takeIf { it > 0L } ?: return null
        UserPlaybackStateReport.Progress(
            itemId = launch.mediaItemId,
            request = UpdatePlaybackProgressRequest(
                sourceId = launch.sourceId,
                positionMs = positivePositionMs,
                durationMs = knownDurationMs,
            ),
        )
    }
}
