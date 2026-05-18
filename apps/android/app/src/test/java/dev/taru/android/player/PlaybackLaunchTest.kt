package dev.taru.android.player

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.PlaybackRequestTarget
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlaybackLaunchTest {
    @Test
    fun `launch request debug output uses safe request preview only`() {
        val launch = playbackLaunchRequest(
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
        )

        assertTrue(launch.toString().contains("Bearer <redacted>"))
        assertFalse(launch.toString().contains("secret-token"))
    }
}

