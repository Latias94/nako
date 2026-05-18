package dev.taru.android.player

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
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
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.Transcode,
            sessionId = "session-1",
            resumePositionMs = 12_000,
        )

        assertTrue(launch.toString().contains("Bearer <redacted>"))
        assertTrue(launch.toString().contains("session-1"))
        assertFalse(launch.toString().contains("secret-token"))
    }

    @Test
    fun `launch request position key is scoped to active server media item and source`() {
        val launch = playbackLaunchRequest(
            title = "Night Harbor",
            target = PlaybackRequestTarget(
                request = TaruHttpRequest(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream",
                    headers = mapOf("Authorization" to "Bearer secret-token"),
                ),
                safeRequest = SafeRequestPreview(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream",
                    headers = mapOf("Authorization" to "Bearer <redacted>"),
                ),
            ),
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.DirectPlay,
        )

        assertEquals("server-1", launch.positionKey.serverProfileId)
        assertEquals("item-1", launch.positionKey.mediaItemId)
        assertEquals("source-1", launch.positionKey.sourceId)
    }

    @Test
    fun `device local playback position does not mix across server profiles`() {
        val store = InMemoryDevicePlaybackPositionStore()
        val homeKey = DevicePlaybackPositionKey(
            serverProfileId = "home-server",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )
        val travelKey = homeKey.copy(serverProfileId = "travel-server")

        store.save(
            DevicePlaybackPosition(
                key = homeKey,
                positionMs = 90_000,
                durationMs = 600_000,
                updatedAtMillis = 1_779_000_000_000,
            ),
        )

        assertEquals(90_000L, store.load(homeKey)?.positionMs)
        assertNull(store.load(travelKey))
    }

    @Test
    fun `device local playback position clears non positive positions`() {
        val store = InMemoryDevicePlaybackPositionStore()
        val key = DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 42_000,
                updatedAtMillis = 1,
            ),
        )
        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 0,
                updatedAtMillis = 2,
            ),
        )

        assertNull(store.load(key))
    }
}
