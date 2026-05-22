package dev.nako.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.nako.android.ui.browse.NakoBrowseShell
import dev.nako.android.ui.connection.NakoConnectionShellContent
import dev.nako.android.ui.screens.player.rememberAndroidPlaybackSessionRuntimeFactory
import dev.nako.android.ui.screens.player.rememberPlaybackPlayerRouteRenderer
import kotlinx.coroutines.CoroutineScope

@Composable
fun NakoAndroidApp(
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val environmentFactory = remember { AndroidNakoAppEnvironmentFactory() }
    val environment = remember(context, environmentFactory) {
        environmentFactory.create(context)
    }
    val appSession = remember(environment) {
        environment.createSession()
    }
    val playerExitEffectScope = rememberCoroutineScope()

    NakoAndroidAppContent(
        modifier = modifier,
        environment = environment,
        appSession = appSession,
        playerExitEffectScope = playerExitEffectScope,
    )
}

@Composable
internal fun NakoAndroidAppContent(
    environment: NakoAppEnvironment,
    appSession: NakoAppSession,
    playerExitEffectScope: CoroutineScope,
    modifier: Modifier = Modifier,
) {
    val appState by appSession.state.collectAsStateWithLifecycle()
    val activeProfile = appState.activeProfile

    if (appState.shouldShowConnection) {
        NakoConnectionShellContent(
            modifier = modifier,
            runtime = environment.createConnectionRuntime(),
            initialSnapshot = appState.snapshot,
            onSnapshotChanged = { next ->
                appSession.dispatch(NakoAppAction.SnapshotChanged(next))
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
        NakoBrowseShell(
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
                appSession.dispatch(NakoAppAction.SnapshotChanged(next))
            },
            onChangeServer = {
                appSession.dispatch(NakoAppAction.RequestConnection)
            },
        )
    }
}
