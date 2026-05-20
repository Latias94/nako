package dev.taru.android.ui.screens.player

import androidx.media3.common.Player
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.PlaybackExitSnapshot
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlayerRouteHostTest {
    @Test
    fun prepareAndRetryPrepareEngineAndPublishSessionState() {
        val engine = RecordingRouteEngine()
        val host = PlayerRouteHost(
            launch = launch(),
            engine = engine,
            exitEffectRunner = RecordingExitEffectRunner(),
        )

        host.attach()
        host.prepare()
        engine.emitPlaybackStateChanged(Player.STATE_BUFFERING)
        host.retry()

        assertEquals(listOf(launch(), launch()), engine.preparedLaunches)
        assertEquals("Preparing", host.state.value.playerStateLabel)
    }

    @Test
    fun engineCallbacksUpdateSessionState() {
        val engine = RecordingRouteEngine(
            playbackState = Player.STATE_READY,
            isPlaying = true,
        )
        val host = PlayerRouteHost(
            launch = launch(),
            engine = engine,
            exitEffectRunner = RecordingExitEffectRunner(),
        )

        host.attach()
        engine.emitPlaybackStateChanged(Player.STATE_READY)
        assertEquals("Playing", host.state.value.playerStateLabel)

        engine.emitIsPlayingChanged(false)
        assertEquals("Paused", host.state.value.playerStateLabel)
    }

    @Test
    fun backAndDisposeTriggerOneExitEffectAndReleaseEngine() {
        val engine = RecordingRouteEngine()
        val exitRunner = RecordingExitEffectRunner()
        val host = PlayerRouteHost(
            launch = launch(),
            engine = engine,
            exitEffectRunner = exitRunner,
        )

        host.attach()
        host.back()
        host.dispose()
        host.back()

        assertEquals(1, exitRunner.requests.size)
        assertEquals(1, engine.releaseCount)
        assertFalse(engine.hasListeners)
        assertTrue(host.state.value.exitRequested)
    }

    @Test
    fun attachDisposeAndPrepareAreIdempotentAcrossRelease() {
        val engine = RecordingRouteEngine()
        val host = PlayerRouteHost(
            launch = launch(),
            engine = engine,
            exitEffectRunner = RecordingExitEffectRunner(),
        )

        host.attach()
        host.attach()
        host.prepare()
        host.dispose()
        host.dispose()
        host.attach()
        host.prepare()
        host.retry()

        assertEquals(1, engine.listenerAddCount)
        assertEquals(1, engine.listenerRemoveCount)
        assertEquals(1, engine.releaseCount)
        assertEquals(listOf(launch()), engine.preparedLaunches)
    }

    @Test
    fun playerErrorStoresSanitizedDiagnostics() {
        val engine = RecordingRouteEngine()
        val host = PlayerRouteHost(
            launch = launch(),
            engine = engine,
            exitEffectRunner = RecordingExitEffectRunner(),
        )

        host.attach()
        engine.emitPlayerError("ERROR_CODE_IO_BAD_HTTP_STATUS")

        val error = requireNotNull(host.state.value.playbackError)
        assertEquals("Error", host.state.value.playerStateLabel)
        assertTrue(error.diagnostics.contains("Bearer <redacted>"))
        assertEquals(false, error.diagnostics.contains("secret-token"))
    }
}

private class RecordingRouteEngine(
    override var playbackState: Int = Player.STATE_IDLE,
    override var isPlaying: Boolean = false,
) : PlayerRouteEngine {
    override val player: Player
        get() = error("Player instance is not needed by host unit tests.")

    val preparedLaunches: MutableList<PlaybackLaunchRequest> = mutableListOf()
    private val listeners: MutableList<PlayerRouteEngineListener> = mutableListOf()
    var listenerAddCount: Int = 0
        private set
    var listenerRemoveCount: Int = 0
        private set
    var releaseCount: Int = 0
        private set
    val hasListeners: Boolean
        get() = listeners.isNotEmpty()

    override fun prepare(launch: PlaybackLaunchRequest) {
        preparedLaunches += launch
    }

    override fun addListener(listener: PlayerRouteEngineListener) {
        listenerAddCount += 1
        listeners += listener
    }

    override fun removeListener(listener: PlayerRouteEngineListener) {
        listenerRemoveCount += 1
        listeners -= listener
    }

    override fun snapshot(): PlaybackExitSnapshot =
        PlaybackExitSnapshot(
            isEnded = playbackState == Player.STATE_ENDED,
            positionMs = 12_000,
            durationMs = 120_000,
        )

    override fun release() {
        releaseCount += 1
    }

    fun emitPlaybackStateChanged(playbackState: Int) {
        this.playbackState = playbackState
        listeners.forEach { it.onPlaybackStateChanged(playbackState) }
    }

    fun emitIsPlayingChanged(isPlaying: Boolean) {
        this.isPlaying = isPlaying
        listeners.forEach { it.onIsPlayingChanged(isPlaying) }
    }

    fun emitPlayerError(errorCodeName: String?) {
        listeners.forEach { it.onPlayerError(errorCodeName) }
    }
}

private class RecordingExitEffectRunner : PlaybackExitEffectRunner {
    val requests: MutableList<Pair<PlaybackLaunchRequest, PlaybackExitSnapshot>> = mutableListOf()

    override fun run(
        launch: PlaybackLaunchRequest,
        snapshot: PlaybackExitSnapshot,
    ) {
        requests += launch to snapshot
    }
}

private fun launch(): PlaybackLaunchRequest =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = PlaybackRequestTarget(
            request = TaruHttpRequest(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream/hls/playlist.m3u8",
                headers = mapOf("Authorization" to "Bearer secret-token"),
            ),
            safeRequest = SafeRequestPreview(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream/hls/playlist.m3u8",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        ),
        serverProfileId = "server-1",
        mediaItemId = "item-1",
        sourceId = "source-1",
        playbackMode = ClientPlaybackMode.Transcode,
        sessionId = "session-1",
    )
