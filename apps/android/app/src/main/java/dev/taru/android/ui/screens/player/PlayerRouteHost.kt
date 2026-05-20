package dev.taru.android.ui.screens.player

import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import dev.taru.android.player.PlaybackExitSnapshot
import dev.taru.android.player.PlaybackLaunchRequest
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

internal class PlayerRouteHost(
    private val launch: PlaybackLaunchRequest,
    private val engine: PlayerRouteEngine,
    private val exitEffectRunner: PlaybackExitEffectRunner,
    initialState: PlayerSessionState = PlayerSessionState(),
) {
    private val session = PlayerSession(
        launch = launch,
        initialState = initialState,
    )
    private val _state = MutableStateFlow(session.state)
    val state: StateFlow<PlayerSessionState> = _state.asStateFlow()

    private var isAttached: Boolean = false
    private var isReleased: Boolean = false
    private val listener = object : PlayerRouteEngineListener {
        override fun onPlaybackStateChanged(playbackState: Int) {
            dispatch(
                PlayerSessionEvent.PlaybackStateChanged(
                    state = playbackEngineState(playbackState),
                    isPlaying = engine.isPlaying,
                ),
            )
        }

        override fun onIsPlayingChanged(isPlaying: Boolean) {
            dispatch(
                PlayerSessionEvent.IsPlayingChanged(
                    isPlaying = isPlaying,
                    currentState = playbackEngineState(engine.playbackState),
                ),
            )
        }

        override fun onPlayerError(errorCodeName: String?) {
            dispatch(PlayerSessionEvent.Error(errorCodeName))
        }
    }

    val player: Player
        get() = engine.player

    fun attach() {
        if (isAttached || isReleased) {
            return
        }
        engine.addListener(listener)
        isAttached = true
    }

    fun prepare() {
        if (isReleased) {
            return
        }
        attach()
        dispatch(PlayerSessionEvent.Prepare)
        engine.prepare(launch)
    }

    fun retry() {
        if (isReleased) {
            return
        }
        attach()
        dispatch(PlayerSessionEvent.Retry)
        engine.prepare(launch)
    }

    fun back() {
        dispatch(PlayerSessionEvent.Back)
    }

    fun dispose() {
        dispatch(PlayerSessionEvent.Dispose)
        detach()
        if (!isReleased) {
            engine.release()
            isReleased = true
        }
    }

    private fun dispatch(event: PlayerSessionEvent) {
        val transition = session.dispatch(event)
        _state.value = transition.state
        if (transition.requestExitEffects) {
            exitEffectRunner.run(
                launch = launch,
                snapshot = engine.snapshot(),
            )
        }
    }

    private fun detach() {
        if (!isAttached) {
            return
        }
        engine.removeListener(listener)
        isAttached = false
    }
}

internal interface PlayerRouteEngine {
    val player: Player
    val playbackState: Int
    val isPlaying: Boolean
    fun prepare(launch: PlaybackLaunchRequest)
    fun addListener(listener: PlayerRouteEngineListener)
    fun removeListener(listener: PlayerRouteEngineListener)
    fun snapshot(): PlaybackExitSnapshot
    fun release()
}

internal interface PlayerRouteEngineListener {
    fun onPlaybackStateChanged(playbackState: Int)
    fun onIsPlayingChanged(isPlaying: Boolean)
    fun onPlayerError(errorCodeName: String?)
}

internal class PlaybackControllerRouteEngine(
    private val controller: PlaybackEngineController,
) : PlayerRouteEngine {
    private val listeners: MutableMap<PlayerRouteEngineListener, Player.Listener> = mutableMapOf()

    override val player: Player
        get() = controller.player

    override val playbackState: Int
        get() = controller.playbackState

    override val isPlaying: Boolean
        get() = controller.isPlaying

    override fun prepare(launch: PlaybackLaunchRequest) {
        controller.prepare(launch)
    }

    override fun addListener(listener: PlayerRouteEngineListener) {
        if (listeners.containsKey(listener)) {
            return
        }
        val media3Listener = object : Player.Listener {
            override fun onPlaybackStateChanged(playbackState: Int) {
                listener.onPlaybackStateChanged(playbackState)
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) {
                listener.onIsPlayingChanged(isPlaying)
            }

            override fun onPlayerError(error: PlaybackException) {
                listener.onPlayerError(error.errorCodeName)
            }
        }
        listeners[listener] = media3Listener
        controller.addListener(media3Listener)
    }

    override fun removeListener(listener: PlayerRouteEngineListener) {
        listeners.remove(listener)?.let(controller::removeListener)
    }

    override fun snapshot(): PlaybackExitSnapshot =
        controller.snapshot()

    override fun release() {
        controller.release()
    }
}

internal fun playbackEngineState(playbackState: Int): PlayerEngineState =
    when (playbackState) {
        Player.STATE_IDLE -> PlayerEngineState.Idle
        Player.STATE_BUFFERING -> PlayerEngineState.Buffering
        Player.STATE_READY -> PlayerEngineState.Ready
        Player.STATE_ENDED -> PlayerEngineState.Ended
        else -> PlayerEngineState.Unknown
    }
