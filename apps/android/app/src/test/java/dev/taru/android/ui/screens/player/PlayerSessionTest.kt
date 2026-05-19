package dev.taru.android.ui.screens.player

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlayerSessionTest {
    @Test
    fun playbackStateEventsProduceStableDisplayLabels() {
        val session = PlayerSession(launch())

        assertEquals("Preparing", session.state.playerStateLabel)

        session.dispatch(
            PlayerSessionEvent.PlaybackStateChanged(
                state = PlayerEngineState.Buffering,
                isPlaying = false,
            ),
        )
        assertEquals("Buffering", session.state.playerStateLabel)

        session.dispatch(
            PlayerSessionEvent.PlaybackStateChanged(
                state = PlayerEngineState.Ready,
                isPlaying = true,
            ),
        )
        assertEquals("Playing", session.state.playerStateLabel)

        session.dispatch(
            PlayerSessionEvent.IsPlayingChanged(
                currentState = PlayerEngineState.Ready,
                isPlaying = false,
            ),
        )
        assertEquals("Paused", session.state.playerStateLabel)
    }

    @Test
    fun errorEventStoresSanitizedDiagnosticsAndRetryClearsError() {
        val session = PlayerSession(launch())

        session.dispatch(PlayerSessionEvent.Error("ERROR_CODE_IO_BAD_HTTP_STATUS"))

        val error = requireNotNull(session.state.playbackError)
        assertEquals("Error", session.state.playerStateLabel)
        assertTrue(error.diagnostics.contains("Bearer <redacted>"))
        assertFalse(error.diagnostics.contains("secret-token"))

        session.dispatch(PlayerSessionEvent.Retry)

        assertEquals("Preparing", session.state.playerStateLabel)
        assertEquals(null, session.state.playbackError)
    }

    @Test
    fun backAndDisposeShareOneIdempotentExitRequest() {
        val session = PlayerSession(launch())

        val first = session.dispatch(PlayerSessionEvent.Back)
        val second = session.dispatch(PlayerSessionEvent.Dispose)
        val third = session.dispatch(PlayerSessionEvent.Back)

        assertTrue(first.requestExitEffects)
        assertFalse(second.requestExitEffects)
        assertFalse(third.requestExitEffects)
        assertTrue(session.state.exitRequested)
    }

    @Test
    fun prepareDoesNotClearExitRequestAfterBack() {
        val session = PlayerSession(launch())

        session.dispatch(PlayerSessionEvent.Back)
        val transition = session.dispatch(PlayerSessionEvent.Prepare)

        assertFalse(transition.requestExitEffects)
        assertTrue(session.state.exitRequested)
        assertEquals("Preparing", session.state.playerStateLabel)
    }

    private fun launch() =
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
}
