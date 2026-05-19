package dev.taru.android.ui.browse

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackStartCoordinator
import dev.taru.android.playback.PlaybackStartRequest
import dev.taru.android.playback.PlaybackStartResult

internal class ClientBrowsePlaybackStarter(
    private val profile: ServerProfile,
    private val tokenVault: TokenVault,
    private val coordinator: PlaybackStartCoordinator,
) : BrowsePlaybackStarter {
    override suspend fun start(request: PlaybackStartRequest): PlaybackStartResult =
        coordinator.start(
            profile = profile,
            tokenVault = tokenVault,
            request = request,
        )
}
