package dev.nako.android.ui.screens.player

import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.PlaybackResumeSource
import dev.nako.android.player.playbackLaunchRequest
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
        assertEquals("Night Harbor", chrome.backdropTitle)
        assertEquals("HLS", chrome.modeLabel)
        assertEquals("Resume on this device 1:32", chrome.resumeLabel)
    }

    @Test
    fun playerChromeLabelsAuthoritativeUserPlaybackStateResume() {
        val chrome = playerChromePresentation(
            launch = launch(
                resumePositionMs = 92_000,
                resumeSource = PlaybackResumeSource.UserPlaybackState,
            ),
        )

        assertEquals("Resume from server 1:32", chrome.resumeLabel)
    }

    @Test
    fun playerChromeKeepsSessionIdOutOfVisibleAndAccessibilityLabels() {
        val chrome = playerChromePresentation(
            launch = launch(
                resumePositionMs = null,
                sessionId = "session-1",
            ),
        )

        assertEquals("Playback session active", chrome.sessionLabel)
        assertEquals("Playback session is active", chrome.sessionAccessibilityLabel)
        assertFalse(chrome.toString().contains("session-1"))
    }

    @Test
    fun playerBackdropUsesStableFallbackTitleWhenLaunchTitleIsBlank() {
        val chrome = playerChromePresentation(
            launch = launch(title = " ", resumePositionMs = null),
        )

        assertEquals("Nako Playback", chrome.title)
        assertEquals("Nako Playback", chrome.backdropTitle)
    }

    @Test
    fun playerContextChromeKeepsClearanceForMedia3Controls() {
        assertTrue(PlayerMedia3ControllerClearanceDp >= 96)
    }

    @Test
    fun playbackErrorDiagnosticsUseSafeRequestOnly() {
        val presentation = playbackErrorPresentation(
            errorCodeName = "ERROR_CODE_IO_NETWORK_CONNECTION_FAILED",
            launch = launch(resumePositionMs = null),
        )

        assertEquals("Playback interrupted", presentation.title)
        assertFalse(presentation.body.contains("route"))
        assertTrue(presentation.diagnostics.contains("Bearer <redacted>"))
        assertFalse(presentation.diagnostics.contains("secret-token"))
    }

    private fun launch(
        title: String = "Night Harbor",
        resumePositionMs: Long?,
        resumeSource: PlaybackResumeSource? = null,
        sessionId: String = "session-1",
    ) =
        playbackLaunchRequest(
            title = title,
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
            sessionId = sessionId,
            resumePositionMs = resumePositionMs,
            resumeSource = resumeSource,
        )
}
