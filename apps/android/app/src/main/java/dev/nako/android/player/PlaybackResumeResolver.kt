package dev.nako.android.player

import dev.nako.android.userplayback.UserPlaybackStateDto

internal fun resolvePlaybackResumePosition(
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
