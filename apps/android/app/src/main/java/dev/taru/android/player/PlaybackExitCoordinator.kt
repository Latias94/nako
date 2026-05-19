package dev.taru.android.player

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.userplayback.TaruUserPlaybackClient

internal class PlaybackExitCoordinator(
    private val playbackClient: TaruPlaybackClient,
    private val userPlaybackClient: TaruUserPlaybackClient,
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
