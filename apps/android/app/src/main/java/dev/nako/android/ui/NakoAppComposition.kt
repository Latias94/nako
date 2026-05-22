package dev.nako.android.ui

import android.content.Context
import dev.nako.android.BuildConfig
import dev.nako.android.browse.NakoBrowseClient
import dev.nako.android.connection.AndroidSecureTokenVault
import dev.nako.android.connection.ConnectionSecurityPolicy
import dev.nako.android.connection.JdkNakoHttpTransport
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.ServerProfileStore
import dev.nako.android.connection.SharedPreferencesServerProfileStore
import dev.nako.android.connection.NakoConnectionClient
import dev.nako.android.connection.NakoHttpTransport
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.PlaybackPreferencesStore
import dev.nako.android.playback.SharedPreferencesPlaybackPreferencesStore
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.player.DevicePlaybackPositionStore
import dev.nako.android.player.SharedPreferencesDevicePlaybackPositionStore
import dev.nako.android.ui.connection.ClientConnectionRuntime
import dev.nako.android.ui.connection.ConnectionRuntime
import dev.nako.android.userplayback.NakoUserPlaybackClient

internal class NakoAppEnvironment(
    val store: ServerProfileStore,
    val tokenVault: TokenVault,
    val connectionClient: NakoConnectionClient,
    val browseClient: NakoBrowseClient,
    val playbackClient: NakoPlaybackClient,
    val playbackPreferencesStore: PlaybackPreferencesStore,
    val userPlaybackClient: NakoUserPlaybackClient,
    val positionStore: DevicePlaybackPositionStore,
) {
    fun createSession(): NakoAppSession =
        NakoAppSession(
            initialSnapshot = store.load(),
            runtime = createRuntime(),
        )

    fun createRuntime(): NakoAppRuntime =
        StoreBackedNakoAppRuntime(store)

    fun createConnectionRuntime(): ConnectionRuntime =
        ClientConnectionRuntime(
            store = store,
            tokenVault = tokenVault,
            client = connectionClient,
        )
}

internal class AndroidNakoAppEnvironmentFactory(
    private val securityPolicy: ConnectionSecurityPolicy = defaultConnectionSecurityPolicy(),
    private val transportFactory: (ConnectionSecurityPolicy) -> NakoHttpTransport = { policy ->
        JdkNakoHttpTransport(securityPolicy = policy)
    },
) {
    fun create(context: Context): NakoAppEnvironment {
        val appContext = context.applicationContext
        val transport = transportFactory(securityPolicy)
        return NakoAppEnvironment(
            store = SharedPreferencesServerProfileStore(appContext),
            tokenVault = AndroidSecureTokenVault(appContext),
            connectionClient = NakoConnectionClient(
                transport = transport,
                securityPolicy = securityPolicy,
            ),
            browseClient = NakoBrowseClient(transport),
            playbackClient = NakoPlaybackClient(transport),
            playbackPreferencesStore = SharedPreferencesPlaybackPreferencesStore(appContext),
            userPlaybackClient = NakoUserPlaybackClient(transport),
            positionStore = SharedPreferencesDevicePlaybackPositionStore(appContext),
        )
    }
}

private fun defaultConnectionSecurityPolicy(): ConnectionSecurityPolicy =
    if (BuildConfig.NAKO_ALLOW_CLEARTEXT_HTTP) {
        ConnectionSecurityPolicy.allowCleartextForLocalDevelopment()
    } else {
        ConnectionSecurityPolicy.production()
    }

private class StoreBackedNakoAppRuntime(
    private val store: ServerProfileStore,
) : NakoAppRuntime {
    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        store.save(snapshot)
    }
}
