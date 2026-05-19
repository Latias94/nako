package dev.taru.android.ui.screens.player

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.player.PlaybackExitCoordinator
import dev.taru.android.player.PlaybackExitSnapshot
import dev.taru.android.player.PlaybackLaunchRequest
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
