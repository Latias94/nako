package dev.nako.android.uniffi

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.nako_client_uniffi.CorePlaybackCapabilities
import uniffi.nako_client_uniffi.buildPlaybackDecisionRequest

@RunWith(AndroidJUnit4::class)
class NakoUniFfiNativeSmokeTest {
    @Test
    fun packaged_uniffi_library_loads_and_builds_core_request() {
        val request = buildPlaybackDecisionRequest(
            baseUrl = "https://nako.example/api",
            accessToken = "secret-token",
            sourceId = "source 1",
            capabilities = CorePlaybackCapabilities(
                directPlay = true,
                containers = listOf("mp4", "webm"),
                videoCodecs = listOf("h264"),
                audioCodecs = listOf("aac"),
            ),
        )

        assertEquals("playback.decision", request.requestId)
        assertEquals("GET", request.method)
        assertEquals(
            "https://nako.example/api/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm&video_codec=h264&audio_codec=aac",
            request.url,
        )
        assertEquals("Authorization", request.headers.single().name)
        assertEquals("Bearer secret-token", request.headers.single().value)
        assertEquals("Bearer <redacted>", request.safePreview.headers.single().value)
        assertFalse(request.safePreview.toString().contains("secret-token"))
    }
}
