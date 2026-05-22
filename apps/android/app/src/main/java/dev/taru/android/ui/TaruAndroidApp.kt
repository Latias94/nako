package dev.taru.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.taru.android.ui.browse.TaruBrowseShell
import dev.taru.android.ui.connection.TaruConnectionShellContent
import dev.taru.android.ui.screens.player.rememberAndroidPlaybackSessionRuntimeFactory
import dev.taru.android.ui.screens.player.rememberPlaybackPlayerRouteRenderer
import kotlinx.coroutines.CoroutineScope

@Composable
fun TaruAndroidApp(
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val environmentFactory = remember { AndroidTaruAppEnvironmentFactory() }
    val environment = remember(context, environmentFactory) {
        environmentFactory.create(context)
    }
    val appSession = remember(environment) {
        environment.createSession()
    }
    val playerExitEffectScope = rememberCoroutineScope()

    TaruAndroidAppContent(
        modifier = modifier,
        environment = environment,
        appSession = appSession,
        playerExitEffectScope = playerExitEffectScope,
    )
}

@Composable
internal fun TaruAndroidAppContent(
    environment: TaruAppEnvironment,
    appSession: TaruAppSession,
    playerExitEffectScope: CoroutineScope,
    modifier: Modifier = Modifier,
) {
    val appState by appSession.state.collectAsStateWithLifecycle()
    val activeProfile = appState.activeProfile

    if (appState.shouldShowConnection) {
        TaruConnectionShellContent(
            modifier = modifier,
            runtime = environment.createConnectionRuntime(),
            initialSnapshot = appState.snapshot,
            onSnapshotChanged = { next ->
                appSession.dispatch(TaruAppAction.SnapshotChanged(next))
            },
        )
    } else {
        requireNotNull(activeProfile)
        val playbackSessionRuntimeFactory = rememberAndroidPlaybackSessionRuntimeFactory(
            profile = activeProfile,
            tokenVault = environment.tokenVault,
            playbackClient = environment.playbackClient,
            userPlaybackClient = environment.userPlaybackClient,
            positionStore = environment.positionStore,
            exitEffectScope = playerExitEffectScope,
        )
        val playerRouteRenderer = rememberPlaybackPlayerRouteRenderer(playbackSessionRuntimeFactory)
        TaruBrowseShell(
            modifier = modifier,
            profile = activeProfile,
            snapshot = appState.snapshot,
            tokenVault = environment.tokenVault,
            browseClient = environment.browseClient,
            playbackClient = environment.playbackClient,
            playbackPreferencesStore = environment.playbackPreferencesStore,
            userPlaybackClient = environment.userPlaybackClient,
            positionStore = environment.positionStore,
            playerRouteRenderer = playerRouteRenderer,
            onSnapshotChanged = { next ->
                appSession.dispatch(TaruAppAction.SnapshotChanged(next))
            },
            onChangeServer = {
                appSession.dispatch(TaruAppAction.RequestConnection)
            },
        )
    }
}
