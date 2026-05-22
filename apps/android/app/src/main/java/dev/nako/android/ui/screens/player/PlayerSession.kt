package dev.nako.android.ui.screens.player

import dev.nako.android.player.PlaybackLaunchRequest

internal enum class PlayerEngineState {
    Idle,
    Buffering,
    Ready,
    Ended,
    Unknown,
}

internal data class PlayerSessionState(
    val playerStateLabel: String = "Preparing",
    val playbackError: PlaybackErrorPresentation? = null,
    val exitRequested: Boolean = false,
) {
    val isPreparingOrBuffering: Boolean
        get() = playbackError == null &&
            (playerStateLabel == "Preparing" || playerStateLabel == "Buffering")
}

internal sealed interface PlayerSessionEvent {
    data object Prepare : PlayerSessionEvent
    data object Retry : PlayerSessionEvent
    data class PlaybackStateChanged(
        val state: PlayerEngineState,
        val isPlaying: Boolean,
    ) : PlayerSessionEvent
    data class IsPlayingChanged(
        val isPlaying: Boolean,
        val currentState: PlayerEngineState,
    ) : PlayerSessionEvent
    data class Error(
        val errorCodeName: String?,
    ) : PlayerSessionEvent
    data object Back : PlayerSessionEvent
    data object Dispose : PlayerSessionEvent
}

internal data class PlayerSessionTransition(
    val state: PlayerSessionState,
    val requestExitEffects: Boolean = false,
)

internal class PlayerSession(
    private val launch: PlaybackLaunchRequest,
    initialState: PlayerSessionState = PlayerSessionState(),
) {
    var state: PlayerSessionState = initialState
        private set

    fun dispatch(event: PlayerSessionEvent): PlayerSessionTransition {
        val transition = reducePlayerSession(
            state = state,
            event = event,
            launch = launch,
        )
        state = transition.state
        return transition
    }
}

internal fun reducePlayerSession(
    state: PlayerSessionState,
    event: PlayerSessionEvent,
    launch: PlaybackLaunchRequest,
): PlayerSessionTransition =
    when (event) {
        PlayerSessionEvent.Prepare,
        PlayerSessionEvent.Retry,
        -> PlayerSessionTransition(
            state = state.copy(
                playerStateLabel = "Preparing",
                playbackError = null,
            ),
        )
        is PlayerSessionEvent.PlaybackStateChanged -> PlayerSessionTransition(
            state = state.copy(
                playerStateLabel = playerStateLabel(
                    playbackState = event.state,
                    isPlaying = event.isPlaying,
                ),
            ),
        )
        is PlayerSessionEvent.IsPlayingChanged -> if (event.currentState == PlayerEngineState.Ready) {
            PlayerSessionTransition(
                state = state.copy(
                    playerStateLabel = if (event.isPlaying) "Playing" else "Paused",
                ),
            )
        } else {
            PlayerSessionTransition(state = state)
        }
        is PlayerSessionEvent.Error -> PlayerSessionTransition(
            state = state.copy(
                playerStateLabel = "Error",
                playbackError = playbackErrorPresentation(event.errorCodeName, launch),
            ),
        )
        PlayerSessionEvent.Back,
        PlayerSessionEvent.Dispose,
        -> exitTransition(state)
    }

private fun exitTransition(state: PlayerSessionState): PlayerSessionTransition =
    if (state.exitRequested) {
        PlayerSessionTransition(state = state)
    } else {
        PlayerSessionTransition(
            state = state.copy(exitRequested = true),
            requestExitEffects = true,
        )
    }

internal fun playerStateLabel(
    playbackState: PlayerEngineState,
    isPlaying: Boolean,
): String =
    when (playbackState) {
        PlayerEngineState.Idle -> "Idle"
        PlayerEngineState.Buffering -> "Buffering"
        PlayerEngineState.Ready -> if (isPlaying) "Playing" else "Ready"
        PlayerEngineState.Ended -> "Ended"
        PlayerEngineState.Unknown -> "Unknown"
    }
