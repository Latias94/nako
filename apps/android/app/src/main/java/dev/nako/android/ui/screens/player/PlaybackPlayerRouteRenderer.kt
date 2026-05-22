package dev.nako.android.ui.screens.player

import androidx.compose.runtime.Composable
import dev.nako.android.player.PlaybackLaunchRequest

internal interface PlayerRouteRenderer {
    @Composable
    fun Render(
        launch: PlaybackLaunchRequest,
        onBack: () -> Unit,
    )
}

@Composable
internal fun rememberPlaybackPlayerRouteRenderer(runtimeFactory: PlaybackSessionRuntimeFactory): PlayerRouteRenderer =
    androidx.compose.runtime.remember(runtimeFactory) {
        DefaultPlayerRouteRenderer(runtimeFactory)
    }

private class DefaultPlayerRouteRenderer(
    private val runtimeFactory: PlaybackSessionRuntimeFactory,
) : PlayerRouteRenderer {
    @Composable
    override fun Render(
        launch: PlaybackLaunchRequest,
        onBack: () -> Unit,
    ) {
        PlaybackPlayerRoute(
            launch = launch,
            runtimeFactory = runtimeFactory,
            onBack = onBack,
        )
    }
}
