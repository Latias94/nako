package dev.nako.android.player

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.userplayback.NakoUserPlaybackClient

internal class PlaybackExitCoordinator(
    private val playbackClient: NakoPlaybackClient,
    private val userPlaybackClient: NakoUserPlaybackClient,
    private val positionStore: DevicePlaybackPositionStore,
) {
    suspend fun applyExitEffects(
        launch: PlaybackLaunchRequest,
        snapshot: PlaybackExitSnapshot,
        profile: ServerProfile,
        tokenVault: TokenVault,
    ): PlaybackExitEffectResult =
        applyPlaybackExitEffects(
            launch = launch,
            snapshot = snapshot,
            profile = profile,
            readAccessToken = tokenVault::readToken,
            positionStore = positionStore,
            updateProgress = { updateProfile, accessToken, itemId, report ->
                userPlaybackClient.updateProgress(
                    profile = updateProfile,
                    accessToken = accessToken,
                    itemId = itemId,
                    request = report.request,
                )
            },
            setWatchedState = { watchedProfile, accessToken, itemId, report ->
                userPlaybackClient.setWatchedState(
                    profile = watchedProfile,
                    accessToken = accessToken,
                    itemId = itemId,
                    request = report.request,
                )
            },
            cancelPlaybackSession = playbackClient::cancelPlaybackSession,
        )
}
