package dev.nako.android.ui.screens.player

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.TokenVault
import dev.nako.android.player.PlaybackExitCoordinator
import dev.nako.android.player.PlaybackExitSnapshot
import dev.nako.android.player.PlaybackLaunchRequest
import kotlinx.coroutines.CoroutineScope

internal interface PlaybackExitEffectRunner {
    fun run(
        launch: PlaybackLaunchRequest,
        snapshot: PlaybackExitSnapshot,
    )
}

internal class CoroutinePlaybackExitEffectRunner(
    private val profile: ServerProfile,
    private val tokenVault: TokenVault,
    private val exitCoordinator: PlaybackExitCoordinator,
    private val exitEffectScope: CoroutineScope,
) : PlaybackExitEffectRunner {
    override fun run(
        launch: PlaybackLaunchRequest,
        snapshot: PlaybackExitSnapshot,
    ) {
        launchPlayerExitEffect(exitEffectScope) {
            exitCoordinator.applyExitEffects(
                launch = launch,
                snapshot = snapshot,
                profile = profile,
                tokenVault = tokenVault,
            )
        }
    }
}
