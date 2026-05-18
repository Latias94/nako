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

class PlayerPresentationTest {
    @Test
    fun playerChromeLabelsLocalResumeWithoutClaimingUserPlaybackState() {
        val chrome = playerChromePresentation(
            launch = launch(resumePositionMs = 92_000),
        )

        assertEquals("Night Harbor", chrome.title)
        assertEquals("HLS", chrome.modeLabel)
        assertEquals("Local resume 1:32", chrome.resumeLabel)
    }

    @Test
    fun playbackErrorDiagnosticsUseSafeRequestOnly() {
        val presentation = playbackErrorPresentation(
            errorCodeName = "ERROR_CODE_IO_NETWORK_CONNECTION_FAILED",
            launch = launch(resumePositionMs = null),
        )

        assertEquals("Playback interrupted", presentation.title)
        assertTrue(presentation.diagnostics.contains("Bearer <redacted>"))
        assertFalse(presentation.diagnostics.contains("secret-token"))
    }

    private fun launch(resumePositionMs: Long?) =
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
            resumePositionMs = resumePositionMs,
        )
}
