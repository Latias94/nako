package dev.taru.android.ui

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.AndroidSecureTokenVault
import dev.taru.android.connection.JdkTaruHttpTransport
import dev.taru.android.connection.ServerProfileRepository
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.SharedPreferencesServerProfileStore
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.ui.browse.TaruBrowseShell
import dev.taru.android.ui.connection.TaruConnectionShellContent

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

    TaruAndroidAppContent(
        modifier = modifier,
        store = store,
        tokenVault = tokenVault,
        connectionClient = connectionClient,
        browseClient = browseClient,
        playbackClient = playbackClient,
    )
}

@Composable
fun TaruAndroidAppContent(
    store: ServerProfileStore,
    tokenVault: TokenVault,
    connectionClient: TaruConnectionClient,
    browseClient: TaruBrowseClient,
    playbackClient: TaruPlaybackClient,
    modifier: Modifier = Modifier,
) {
    var snapshot by remember { mutableStateOf(store.load()) }
    var showConnection by remember {
        mutableStateOf(activeProfile(snapshot) == null)
    }
    val activeProfile = activeProfile(snapshot)

    if (activeProfile == null || showConnection) {
        TaruConnectionShellContent(
            modifier = modifier,
            store = store,
            tokenVault = tokenVault,
            client = connectionClient,
            initialSnapshot = snapshot,
            onSnapshotChanged = { next ->
                snapshot = next
                if (activeProfile(next) != null) {
                    showConnection = false
                }
            },
        )
    } else {
        TaruBrowseShell(
            modifier = modifier,
            profile = activeProfile,
            snapshot = snapshot,
            tokenVault = tokenVault,
            browseClient = browseClient,
            playbackClient = playbackClient,
            onSnapshotChanged = { next ->
                store.save(next)
                snapshot = next
            },
            onChangeServer = { showConnection = true },
        )
    }
}

private fun activeProfile(snapshot: ServerProfileSnapshot) =
    ServerProfileRepository(snapshot).activeProfile()
