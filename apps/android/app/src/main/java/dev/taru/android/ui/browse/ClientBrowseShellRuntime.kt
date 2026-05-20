package dev.taru.android.ui.browse

import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackPreferencesStore
import dev.taru.android.playback.PlaybackStartCoordinator
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.ui.screens.settings.SettingsRuntime
import dev.taru.android.userplayback.TaruUserPlaybackClient

internal class ClientBrowseShellRuntime(
    private val tokenVault: TokenVault,
    private val browseClient: TaruBrowseClient,
    private val playbackClient: TaruPlaybackClient,
    private val playbackPreferencesStore: PlaybackPreferencesStore,
    private val userPlaybackClient: TaruUserPlaybackClient,
    private val positionStore: DevicePlaybackPositionStore,
    private val onChangeServer: () -> Unit,
    private val onSnapshotChanged: (ServerProfileSnapshot) -> Unit,
) : BrowseShellRuntime {
    private val playbackStartCoordinator = PlaybackStartCoordinator(
        playbackClient = playbackClient,
        positionStore = positionStore,
    )

    override fun dataSource(profile: ServerProfile): BrowseDataSource =
        ClientBrowseDataSource(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            playbackClient = playbackClient,
            playbackPreferencesStore = playbackPreferencesStore,
            userPlaybackClient = userPlaybackClient,
        )

    override fun playbackStarter(profile: ServerProfile): BrowsePlaybackStarter =
        ClientBrowsePlaybackStarter(
            profile = profile,
            tokenVault = tokenVault,
            coordinator = playbackStartCoordinator,
        )

    override fun resumeResolver(profile: ServerProfile): BrowseResumeResolver =
        ClientBrowseResumeResolver(
            serverProfileId = profile.id,
            positionStore = positionStore,
        )

    override fun settingsRuntime(): SettingsRuntime =
        object : SettingsRuntime {
            override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
                onSnapshotChanged(snapshot)
            }

            override fun deleteToken(reference: String) {
                tokenVault.deleteToken(reference)
            }

            override fun requestConnection() {
                onChangeServer()
            }
        }
}
