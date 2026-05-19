package dev.taru.android.ui.screens.player

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.CoroutineScope

internal interface PlayerRouteRenderer {
    @Composable
    fun Render(
        launch: PlaybackLaunchRequest,
        onBack: () -> Unit,
    )
}

@Composable
internal fun rememberPlaybackPlayerRouteRenderer(
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    userPlaybackClient: TaruUserPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    exitEffectScope: CoroutineScope,
): PlayerRouteRenderer =
    remember(profile, tokenVault, playbackClient, userPlaybackClient, positionStore, exitEffectScope) {
        DefaultPlayerRouteRenderer(
            profile = profile,
            tokenVault = tokenVault,
            playbackClient = playbackClient,
            userPlaybackClient = userPlaybackClient,
            positionStore = positionStore,
            exitEffectScope = exitEffectScope,
        )
    }

private class DefaultPlayerRouteRenderer(
    private val profile: ServerProfile,
    private val tokenVault: TokenVault,
    private val playbackClient: TaruPlaybackClient,
    private val userPlaybackClient: TaruUserPlaybackClient,
    private val positionStore: DevicePlaybackPositionStore,
    private val exitEffectScope: CoroutineScope,
) : PlayerRouteRenderer {
    @Composable
    override fun Render(
        launch: PlaybackLaunchRequest,
        onBack: () -> Unit,
    ) {
        PlaybackPlayerRoute(
            launch = launch,
            profile = profile,
            tokenVault = tokenVault,
            playbackClient = playbackClient,
            userPlaybackClient = userPlaybackClient,
            positionStore = positionStore,
            exitEffectScope = exitEffectScope,
            onBack = onBack,
        )
    }
}
