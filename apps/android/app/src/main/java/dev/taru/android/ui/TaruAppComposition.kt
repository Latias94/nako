package dev.taru.android.ui

import android.content.Context
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.AndroidSecureTokenVault
import dev.taru.android.connection.JdkTaruHttpTransport
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.SharedPreferencesServerProfileStore
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackPreferencesStore
import dev.taru.android.playback.SharedPreferencesPlaybackPreferencesStore
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.SharedPreferencesDevicePlaybackPositionStore
import dev.taru.android.userplayback.TaruUserPlaybackClient

internal class TaruAppEnvironment(
    val store: ServerProfileStore,
    val tokenVault: TokenVault,
    val connectionClient: TaruConnectionClient,
    val browseClient: TaruBrowseClient,
    val playbackClient: TaruPlaybackClient,
    val playbackPreferencesStore: PlaybackPreferencesStore,
    val userPlaybackClient: TaruUserPlaybackClient,
    val positionStore: DevicePlaybackPositionStore,
) {
    fun createSession(): TaruAppSession =
        TaruAppSession(
            initialSnapshot = store.load(),
            runtime = createRuntime(),
        )

    fun createRuntime(): TaruAppRuntime =
        StoreBackedTaruAppRuntime(store)
}

internal class AndroidTaruAppEnvironmentFactory(
    private val transportFactory: () -> TaruHttpTransport = { JdkTaruHttpTransport() },
) {
    fun create(context: Context): TaruAppEnvironment {
        val appContext = context.applicationContext
        val transport = transportFactory()
        return TaruAppEnvironment(
            store = SharedPreferencesServerProfileStore(appContext),
            tokenVault = AndroidSecureTokenVault(appContext),
            connectionClient = TaruConnectionClient(transport),
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = SharedPreferencesPlaybackPreferencesStore(appContext),
            userPlaybackClient = TaruUserPlaybackClient(transport),
            positionStore = SharedPreferencesDevicePlaybackPositionStore(appContext),
        )
    }
}

private class StoreBackedTaruAppRuntime(
    private val store: ServerProfileStore,
) : TaruAppRuntime {
    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        store.save(snapshot)
    }
}
