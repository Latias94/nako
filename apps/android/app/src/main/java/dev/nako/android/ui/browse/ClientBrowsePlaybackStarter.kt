package dev.nako.android.ui.browse

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.PlaybackStartCoordinator
import dev.nako.android.playback.PlaybackStartRequest
import dev.nako.android.playback.PlaybackStartResult

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
