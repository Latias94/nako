package dev.taru.android.ui.browse

import dev.taru.android.player.DevicePlaybackPositionKey
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.PlaybackResumeSource
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.userplayback.UserPlaybackStateDto

internal fun resolvedResumePosition(
    profileId: String,
    mediaItemId: String,
    sourceId: String,
    userPlaybackState: UserPlaybackStateDto?,
    positionStore: DevicePlaybackPositionStore,
): ResumePlaybackPosition? {
    val authoritativeResume = userPlaybackState
        ?.takeIf { !it.watched }
        ?.takeIf { state ->
            state.sourceId.isNullOrBlank() || state.sourceId == sourceId
        }
        ?.resumePositionMs
        ?.takeIf { it > 0L }

    if (authoritativeResume != null) {
        return ResumePlaybackPosition(
            positionMs = authoritativeResume,
            source = PlaybackResumeSource.UserPlaybackState,
        )
    }

    val localResume = positionStore.load(
        DevicePlaybackPositionKey(
            serverProfileId = profileId,
            mediaItemId = mediaItemId,
            sourceId = sourceId,
        ),
    )?.positionMs?.takeIf { it > 0L }

    return localResume?.let { positionMs ->
        ResumePlaybackPosition(
            positionMs = positionMs,
            source = PlaybackResumeSource.DeviceLocal,
        )
    }
}
