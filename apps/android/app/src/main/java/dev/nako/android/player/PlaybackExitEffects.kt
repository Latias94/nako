package dev.nako.android.player

import dev.nako.android.connection.ServerProfile
import dev.nako.android.playback.PlaybackResult
import dev.nako.android.playback.TranscodeSessionResponse
import dev.nako.android.userplayback.UserPlaybackResult
import dev.nako.android.userplayback.UserPlaybackStateResponse

data class PlaybackExitSnapshot(
    val isEnded: Boolean,
    val positionMs: Long,
    val durationMs: Long?,
)

data class PlaybackExitEffectResult(
    val savedDevicePosition: Boolean,
    val clearedDevicePosition: Boolean,
    val reportedUserPlaybackState: Boolean,
    val requestedSessionCancellation: Boolean,
)

internal suspend fun applyPlaybackExitEffects(
    launch: PlaybackLaunchRequest,
    snapshot: PlaybackExitSnapshot,
    profile: ServerProfile,
    readAccessToken: (String) -> String?,
    positionStore: DevicePlaybackPositionStore,
    updateProgress: suspend (
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        report: UserPlaybackStateReport.Progress,
    ) -> UserPlaybackResult<UserPlaybackStateResponse>,
    setWatchedState: suspend (
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        report: UserPlaybackStateReport.Watched,
    ) -> UserPlaybackResult<UserPlaybackStateResponse>,
    cancelPlaybackSession: suspend (
        profile: ServerProfile,
        accessToken: String,
        sessionId: String,
    ) -> PlaybackResult<TranscodeSessionResponse>,
): PlaybackExitEffectResult {
    val knownDurationMs = snapshot.durationMs?.takeIf { it > 0L }
    val clearedDevicePosition = snapshot.isEnded || snapshot.positionMs <= 0L
    if (clearedDevicePosition) {
        positionStore.clear(launch.positionKey)
    } else {
        positionStore.save(
            DevicePlaybackPosition(
                key = launch.positionKey,
                positionMs = snapshot.positionMs,
                durationMs = knownDurationMs,
                updatedAtMillis = System.currentTimeMillis(),
            ),
        )
    }

    val sessionId = launch.sessionId?.takeIf { it.isNotBlank() }
    val report = userPlaybackStateReport(
        launch = launch,
        isEnded = snapshot.isEnded,
        positionMs = snapshot.positionMs,
        durationMs = knownDurationMs,
    )
    if (report == null && (snapshot.isEnded || sessionId == null)) {
        return PlaybackExitEffectResult(
            savedDevicePosition = !clearedDevicePosition,
            clearedDevicePosition = clearedDevicePosition,
            reportedUserPlaybackState = false,
            requestedSessionCancellation = false,
        )
    }

    val accessToken = readAccessToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return PlaybackExitEffectResult(
            savedDevicePosition = !clearedDevicePosition,
            clearedDevicePosition = clearedDevicePosition,
            reportedUserPlaybackState = false,
            requestedSessionCancellation = false,
        )
    }

    val requestedSessionCancellation = !snapshot.isEnded && sessionId != null
    if (requestedSessionCancellation) {
        cancelPlaybackSession(profile, accessToken, sessionId)
    }

    val reportedUserPlaybackState = when (report) {
        is UserPlaybackStateReport.Progress -> {
            updateProgress(profile, accessToken, report.itemId, report)
            true
        }
        is UserPlaybackStateReport.Watched -> {
            setWatchedState(profile, accessToken, report.itemId, report)
            true
        }
        null -> false
    }

    return PlaybackExitEffectResult(
        savedDevicePosition = !clearedDevicePosition,
        clearedDevicePosition = clearedDevicePosition,
        reportedUserPlaybackState = reportedUserPlaybackState,
        requestedSessionCancellation = requestedSessionCancellation,
    )
}
