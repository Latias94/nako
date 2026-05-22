package dev.taru.android.ui.screens.player

import androidx.media3.common.Player
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestDescriptor
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.PlaybackExitSnapshot
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertSame
import org.junit.Assert.assertTrue
import org.junit.Test

class PlaybackSessionRuntimeTest {
    @Test
    fun `runtime owns prepare retry state events and idempotent exit effects`() {
        val launch = launch()
        val engine = RecordingRuntimeEngine()
        val exitRunner = RecordingRuntimeExitEffectRunner()
        val runtime = PlayerRouteHost(
            launch = launch,
            engine = engine,
            exitEffectRunner = exitRunner,
        )

        runtime.attach()
        runtime.prepare()
        engine.emitPlaybackStateChanged(Player.STATE_READY, isPlaying = true)
        runtime.back()
        runtime.dispose()
        runtime.retry()

        assertEquals(listOf(launch), engine.preparedLaunches)
        assertEquals("Playing", runtime.state.value.playerStateLabel)
        assertTrue(runtime.state.value.exitRequested)
        assertEquals(1, exitRunner.requests.size)
        assertSame(launch, exitRunner.requests.single().first)
        assertEquals(1, engine.releaseCount)
        assertFalse(engine.hasListeners)
    }

    @Test
    fun `runtime injected engine maps player errors through token safe presentation`() {
        val engine = RecordingRuntimeEngine()
        val runtime = PlayerRouteHost(
            launch = launch(),
            engine = engine,
            exitEffectRunner = RecordingRuntimeExitEffectRunner(),
        )

        runtime.attach()
        engine.emitPlayerError("ERROR_CODE_IO_BAD_HTTP_STATUS")

        val error = requireNotNull(runtime.state.value.playbackError)
        assertEquals("Error", runtime.state.value.playerStateLabel)
        assertTrue(error.diagnostics.contains("Bearer <redacted>"))
        assertFalse(error.diagnostics.contains("secret-token"))
    }

    @Test
    fun `runtime factory reads access token only while creating a playback runtime`() {
        val profile = profile()
        val tokenVault = RecordingTokenVault().apply {
            saveToken(profile.tokenReference, "secret-token")
        }
        val engineFactory = RecordingRuntimeEngineFactory()
        val runtimeFactory = AndroidPlaybackSessionRuntimeFactory.fromDependencies(
            profile = profile,
            tokenVault = tokenVault,
            engineFactory = engineFactory,
            exitEffectRunner = RecordingRuntimeExitEffectRunner(),
        )

        val runtime = runtimeFactory.create(launch())

        assertEquals(listOf(profile.tokenReference), tokenVault.readReferences)
        assertEquals(listOf("secret-token"), engineFactory.accessTokens)
        assertTrue(runtime.state.value.playerStateLabel.isNotBlank())
        assertFalse(runtimeFactory.toString().contains("secret-token"))
    }
}

private class RecordingRuntimeEngineFactory(
    private val engine: PlayerRouteEngine = RecordingRuntimeEngine(),
) : PlayerRouteEngineFactory {
    val accessTokens: MutableList<String> = mutableListOf()

    override fun create(accessToken: String): PlayerRouteEngine {
        accessTokens += accessToken
        return engine
    }
}

private class RecordingRuntimeEngine(
    override var playbackState: Int = Player.STATE_IDLE,
    override var isPlaying: Boolean = false,
) : PlayerRouteEngine {
    override val player: Player
        get() = error("Player instance is not needed by runtime unit tests.")

    val preparedLaunches: MutableList<PlaybackLaunchRequest> = mutableListOf()
    private val listeners: MutableList<PlayerRouteEngineListener> = mutableListOf()
    var releaseCount: Int = 0
        private set
    val hasListeners: Boolean
        get() = listeners.isNotEmpty()

    override fun prepare(launch: PlaybackLaunchRequest) {
        preparedLaunches += launch
    }

    override fun addListener(listener: PlayerRouteEngineListener) {
        listeners += listener
    }

    override fun removeListener(listener: PlayerRouteEngineListener) {
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

    fun emitPlaybackStateChanged(
        playbackState: Int,
        isPlaying: Boolean = this.isPlaying,
    ) {
        this.playbackState = playbackState
        this.isPlaying = isPlaying
        listeners.forEach { it.onPlaybackStateChanged(playbackState) }
    }

    fun emitPlayerError(errorCodeName: String?) {
        listeners.forEach { it.onPlayerError(errorCodeName) }
    }
}

private class RecordingRuntimeExitEffectRunner : PlaybackExitEffectRunner {
    val requests: MutableList<Pair<PlaybackLaunchRequest, PlaybackExitSnapshot>> = mutableListOf()

    override fun run(
        launch: PlaybackLaunchRequest,
        snapshot: PlaybackExitSnapshot,
    ) {
        requests += launch to snapshot
    }
}

private class RecordingTokenVault : TokenVault {
    private val tokens = linkedMapOf<String, String>()
    val readReferences: MutableList<String> = mutableListOf()

    override fun saveToken(
        reference: String,
        token: String,
    ) {
        tokens[reference] = token
    }

    override fun readToken(reference: String): String? {
        readReferences += reference
        return tokens[reference]
    }

    override fun deleteToken(reference: String) {
        tokens.remove(reference)
    }
}

private fun profile(): ServerProfile =
    ServerProfile(
        id = "server-1",
        displayName = "Home",
        baseUrl = "http://home.example.test",
        tokenReference = "server-token:server-1",
        lastObservedApiVersion = "v1",
    )

private fun launch(): PlaybackLaunchRequest =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = PlaybackRequestTarget(
            request = PlaybackRequestDescriptor(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream/hls/playlist.m3u8",
            ),
        ),
        serverProfileId = "server-1",
        mediaItemId = "item-1",
        sourceId = "source-1",
        playbackMode = ClientPlaybackMode.Transcode,
        sessionId = "session-1",
    )
