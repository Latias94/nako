package dev.taru.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.AndroidSecureTokenVault
import dev.taru.android.connection.JdkTaruHttpTransport
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.SharedPreferencesServerProfileStore
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.playback.PlaybackPreferencesStore
import dev.taru.android.playback.SharedPreferencesPlaybackPreferencesStore
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.SharedPreferencesDevicePlaybackPositionStore
import dev.taru.android.ui.browse.TaruBrowseShell
import dev.taru.android.ui.connection.TaruConnectionShellContent
import dev.taru.android.ui.screens.player.rememberPlaybackPlayerRouteRenderer
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.CoroutineScope

@Composable
fun TaruAndroidApp(
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val transport = remember { JdkTaruHttpTransport() }
    val store = remember { SharedPreferencesServerProfileStore(context) }
    val tokenVault = remember { AndroidSecureTokenVault(context) }
    val connectionClient = remember { TaruConnectionClient(transport) }
    val browseClient = remember { TaruBrowseClient(transport) }
    val playbackClient = remember { TaruPlaybackClient(transport) }
    val playbackPreferencesStore = remember { SharedPreferencesPlaybackPreferencesStore(context) }
    val userPlaybackClient = remember { TaruUserPlaybackClient(transport) }
    val positionStore = remember { SharedPreferencesDevicePlaybackPositionStore(context) }
    val playerExitEffectScope = rememberCoroutineScope()

    TaruAndroidAppContent(
        modifier = modifier,
        store = store,
        tokenVault = tokenVault,
        connectionClient = connectionClient,
        browseClient = browseClient,
        playbackClient = playbackClient,
        playbackPreferencesStore = playbackPreferencesStore,
        userPlaybackClient = userPlaybackClient,
        positionStore = positionStore,
        playerExitEffectScope = playerExitEffectScope,
    )
}

@Composable
fun TaruAndroidAppContent(
    store: ServerProfileStore,
    tokenVault: TokenVault,
    connectionClient: TaruConnectionClient,
    browseClient: TaruBrowseClient,
    playbackClient: TaruPlaybackClient,
    playbackPreferencesStore: PlaybackPreferencesStore,
    userPlaybackClient: TaruUserPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    playerExitEffectScope: CoroutineScope,
    modifier: Modifier = Modifier,
) {
    val appRuntime = remember(store) {
        object : TaruAppRuntime {
            override fun saveSnapshot(snapshot: dev.taru.android.connection.ServerProfileSnapshot) {
                store.save(snapshot)
            }
        }
    }
    val appSession = remember(store, appRuntime) {
        TaruAppSession(
            initialSnapshot = store.load(),
            runtime = appRuntime,
        )
    }
    val appState by appSession.state.collectAsState()
    val activeProfile = appState.activeProfile

    if (appState.shouldShowConnection) {
        TaruConnectionShellContent(
            modifier = modifier,
            store = store,
            tokenVault = tokenVault,
            client = connectionClient,
            initialSnapshot = appState.snapshot,
            onSnapshotChanged = { next ->
                appSession.dispatch(TaruAppAction.SnapshotChanged(next))
            },
        )
    } else {
        requireNotNull(activeProfile)
        val playerRouteRenderer = rememberPlaybackPlayerRouteRenderer(
            profile = activeProfile,
            tokenVault = tokenVault,
            playbackClient = playbackClient,
            userPlaybackClient = userPlaybackClient,
            positionStore = positionStore,
            exitEffectScope = playerExitEffectScope,
        )
        TaruBrowseShell(
            modifier = modifier,
            profile = activeProfile,
            snapshot = appState.snapshot,
            tokenVault = tokenVault,
            browseClient = browseClient,
            playbackClient = playbackClient,
            playbackPreferencesStore = playbackPreferencesStore,
            userPlaybackClient = userPlaybackClient,
            positionStore = positionStore,
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
