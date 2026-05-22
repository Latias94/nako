package dev.nako.android.ui.browse

import dev.nako.android.browse.NakoBrowseClient
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.PlaybackPreferencesStore
import dev.nako.android.playback.PlaybackStartCoordinator
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.player.DevicePlaybackPositionStore
import dev.nako.android.ui.screens.settings.SettingsRuntime
import dev.nako.android.userplayback.NakoUserPlaybackClient

internal class ClientBrowseShellRuntime(
    private val tokenVault: TokenVault,
    private val browseClient: NakoBrowseClient,
    private val playbackClient: NakoPlaybackClient,
    private val playbackPreferencesStore: PlaybackPreferencesStore,
    private val userPlaybackClient: NakoUserPlaybackClient,
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
