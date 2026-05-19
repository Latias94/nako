package dev.taru.android.playback

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import java.io.IOException
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TaruPlaybackClientTest {
    @Test
    fun `source probe uses public route and decodes safe media facts`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "source_id": "source 1",
                      "probe": {
                        "duration_ms": 7200000,
                        "container": "matroska",
                        "bit_rate": 12000000,
                        "streams": [
                          {
                            "index": 0,
                            "kind": "video",
                            "codec": "h265",
                            "language": null,
                            "duration_ms": 7200000,
                            "bit_rate": 11000000,
                            "width": 3840,
                            "height": 2160,
                            "channels": null,
                            "sample_rate": null
                          },
                          {
                            "index": 1,
                            "kind": "audio",
                            "codec": "aac",
                            "language": "eng",
                            "duration_ms": 7200000,
                            "bit_rate": 384000,
                            "width": null,
                            "height": null,
                            "channels": 6,
                            "sample_rate": 48000
                          }
                        ]
                      }
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)

        val result = client.getSourceProbe(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            sourceId = "source 1",
        )

        assertTrue(result is PlaybackResult.Success)
        val success = result as PlaybackResult.Success
        assertEquals(
            "http://home.example.test/api/sources/source%201/probe",
            transport.requests.single().url,
        )
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("source 1", success.value.sourceId)
        assertEquals("matroska", success.value.probe.container)
        assertEquals(3840, success.value.probe.streams.first().width)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `playback decision encodes capability query decodes response and redacts safe request`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "source": {
                        "id": "source 1",
                        "library_id": "library-1",
                        "item_id": "item-1",
                        "locator": "file:///srv/media/night-harbor.mkv",
                        "file_name": "night-harbor.mkv",
                        "size_bytes": 42,
                        "fingerprint": null
                      },
                      "probe": {
                        "duration_ms": 6360000,
                        "container": "matroska",
                        "bit_rate": 18000000,
                        "streams": [
                          {
                            "index": 0,
                            "kind": "video",
                            "codec": "h264",
                            "language": null,
                            "duration_ms": 6360000,
                            "bit_rate": 16000000,
                            "width": 3840,
                            "height": 2160,
                            "channels": null,
                            "sample_rate": null
                          },
                          {
                            "index": 1,
                            "kind": "audio",
                            "codec": "aac",
                            "language": "eng",
                            "duration_ms": 6360000,
                            "bit_rate": 384000,
                            "width": null,
                            "height": null,
                            "channels": 6,
                            "sample_rate": 48000
                          }
                        ]
                      },
                      "decision": {
                        "mode": "direct_play",
                        "reason": "client supports this source",
                        "direct_play": {
                          "source_id": "source 1",
                          "content_type": "video/x-matroska",
                          "supports_range_requests": true
                        },
                        "transcode_plan": null
                      }
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)

        val result = client.getPlaybackDecision(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sourceId = "source 1",
            capabilities = PlaybackCapabilities(
                directPlay = true,
                containers = listOf("mp4", "webm"),
                videoCodecs = listOf("h264"),
                audioCodecs = listOf("aac", "opus"),
            ),
        )

        assertTrue(result is PlaybackResult.Success)
        val success = result as PlaybackResult.Success
        assertEquals(
            "http://home.example.test/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm&video_codec=h264&audio_codec=aac%2Copus",
            transport.requests.single().url,
        )
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals(ClientPlaybackMode.DirectPlay, success.value.decision.mode)
        assertEquals("video/x-matroska", success.value.decision.directPlay?.contentType)
        assertEquals(3840, success.value.probe?.streams?.first()?.width)
        assertEquals("night-harbor.mkv", success.value.source.fileName)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `streaming targets use stable paths methods headers queries and safe previews`() {
        val client = TaruPlaybackClient(FakePlaybackTransport())
        val profile = profile("http://home.example.test/api")
        val capabilities = PlaybackCapabilities(
            directPlay = false,
            containers = listOf("mp4", "mkv"),
            videoCodecs = listOf("h264"),
            audioCodecs = listOf("aac"),
        )

        val direct = client.directPlaybackTarget(
            profile = profile,
            accessToken = "secret-token",
            sourceId = "source 1",
            range = "bytes=10-20",
        )
        val remux = client.remuxPlaybackTarget(
            profile = profile,
            accessToken = "secret-token",
            sourceId = "source 1",
            capabilities = capabilities,
            outputContainer = ClientOutputContainer.Mkv,
            range = "bytes=0-",
        )
        val hls = client.hlsPlaylistTarget(
            profile = profile,
            accessToken = "secret-token",
            sourceId = "source 1",
            capabilities = PlaybackCapabilities(
                containers = listOf("hls"),
                videoCodecs = listOf("h264"),
            ),
        )
        val segment = client.hlsSegmentTarget(
            profile = profile,
            accessToken = "secret-token",
            sessionId = "session 1",
            segmentName = "seg 001.ts",
        )

        assertEquals("GET", direct.request.method)
        assertEquals("GET", remux.request.method)
        assertEquals("GET", hls.request.method)
        assertEquals("GET", segment.request.method)
        assertEquals("http://home.example.test/api/sources/source%201/stream", direct.request.url)
        assertEquals(
            "http://home.example.test/api/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv",
            remux.request.url,
        )
        assertEquals(
            "http://home.example.test/api/sources/source%201/stream/hls/playlist.m3u8?container=hls&video_codec=h264",
            hls.request.url,
        )
        assertEquals(
            "http://home.example.test/api/playback/sessions/session%201/hls/segments/seg%20001.ts",
            segment.request.url,
        )
        assertEquals("bytes=10-20", direct.request.headers["Range"])
        assertEquals("bytes=0-", remux.request.headers["Range"])
        listOf(direct, remux, hls, segment).forEach { target ->
            assertEquals("Bearer secret-token", target.request.headers["Authorization"])
            assertEquals("Bearer <redacted>", target.safeRequest.headers["Authorization"])
            assertFalse(target.safeRequest.toString().contains("secret-token"))
            assertFalse(target.toString().contains("secret-token"))
        }
    }

    @Test
    fun `transcode playback decision decodes without internal input locator`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "source": {
                        "id": "source-1",
                        "library_id": "library-1",
                        "item_id": "item-1",
                        "file_name": "night-harbor.mkv",
                        "size_bytes": 42,
                        "fingerprint": null
                      },
                      "probe": null,
                      "decision": {
                        "mode": "transcode",
                        "reason": "client needs HLS",
                        "direct_play": null,
                        "transcode_plan": {
                          "output_container": "hls",
                          "video_codec": "h264",
                          "audio_codec": "aac",
                          "hardware_acceleration": "none"
                        }
                      }
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)

        val result = client.getPlaybackDecision(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sourceId = "source-1",
        )

        assertTrue(result is PlaybackResult.Success)
        val success = result as PlaybackResult.Success
        assertEquals(ClientPlaybackMode.Transcode, success.value.decision.mode)
        assertEquals(ClientOutputContainer.Hls, success.value.decision.transcodePlan?.outputContainer)
        assertFalse(success.toString().contains("input_locator"))
    }

    @Test
    fun `playback session inspection and cancellation use public routes and redacted previews`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "session": {
                        "id": "session 1",
                        "source_id": "source 1",
                        "kind": "remux",
                        "request_key": "remux:mp4",
                        "state": "running",
                        "failure_category": null,
                        "failure_message": null,
                        "created_at": "2026-05-18T06:00:00Z",
                        "updated_at": "2026-05-18T06:00:01Z",
                        "started_at": "2026-05-18T06:00:01Z",
                        "completed_at": null,
                        "output_path": "G:/server/staging/session-1/out.mp4"
                      }
                    }
                    """.trimIndent(),
                ),
            ),
            ResponseStep(
                ok(
                    """
                    {
                      "session": {
                        "id": "session 1",
                        "source_id": "source 1",
                        "kind": "remux",
                        "request_key": "remux:mp4",
                        "state": "cancel_requested",
                        "failure_category": "cancelled",
                        "failure_message": "playback session cancellation requested",
                        "created_at": "2026-05-18T06:00:00Z",
                        "updated_at": "2026-05-18T06:00:02Z",
                        "started_at": "2026-05-18T06:00:01Z",
                        "completed_at": null
                      }
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)
        val profile = profile("http://home.example.test/api")

        val inspected = client.getPlaybackSession(
            profile = profile,
            accessToken = "secret-token",
            sessionId = "session 1",
        )
        val cancelled = client.cancelPlaybackSession(
            profile = profile,
            accessToken = "secret-token",
            sessionId = "session 1",
        )

        assertTrue(inspected is PlaybackResult.Success)
        assertTrue(cancelled is PlaybackResult.Success)
        val inspectedSuccess = inspected as PlaybackResult.Success
        val cancelledSuccess = cancelled as PlaybackResult.Success
        assertEquals(ClientTranscodeSessionKind.Remux, inspectedSuccess.value.session.kind)
        assertEquals(ClientTranscodeSessionState.Running, inspectedSuccess.value.session.state)
        assertEquals(ClientTranscodeSessionState.CancelRequested, cancelledSuccess.value.session.state)
        assertEquals(ClientTranscodeFailureCategory.Cancelled, cancelledSuccess.value.session.failureCategory)
        assertEquals("GET", transport.requests[0].method)
        assertEquals("POST", transport.requests[1].method)
        assertEquals(
            "http://home.example.test/api/playback/sessions/session%201",
            transport.requests[0].url,
        )
        assertEquals(
            "http://home.example.test/api/playback/sessions/session%201/cancel",
            transport.requests[1].url,
        )
        assertEquals("Bearer secret-token", transport.requests[1].headers["Authorization"])
        assertEquals("Bearer <redacted>", inspectedSuccess.request.headers["Authorization"])
        assertEquals("Bearer <redacted>", cancelledSuccess.request.headers["Authorization"])
        assertFalse(inspectedSuccess.toString().contains("secret-token"))
        assertFalse(cancelledSuccess.toString().contains("secret-token"))
        assertFalse(inspectedSuccess.toString().contains("G:/server"))
    }

    @Test
    fun `blank playback session fails locally without transport`() = runBlocking {
        val transport = FakePlaybackTransport()
        val client = TaruPlaybackClient(transport)

        val result = client.cancelPlaybackSession(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sessionId = " ",
        )

        assertTrue(result is PlaybackResult.Failure)
        assertEquals(
            PlaybackFailureCategory.MissingSession,
            (result as PlaybackResult.Failure).diagnostics.category,
        )
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `recommended playback target follows decision mode without exposing local locators`() = runBlocking {
        val client = TaruPlaybackClient(FakePlaybackTransport())
        val profile = profile("http://home.example.test")
        val directDecision = PlaybackDecisionResponse(
            source = PlaybackMediaSourceDto(
                id = "source 1",
                libraryId = "library-1",
                itemId = "item-1",
                locator = "file:///srv/media/night-harbor.mkv",
                fileName = "night-harbor.mkv",
            ),
            decision = ClientPlaybackDecision(
                mode = ClientPlaybackMode.DirectPlay,
                reason = "direct",
                directPlay = ClientDirectPlayPlan(
                    sourceId = "source 1",
                    contentType = "video/x-matroska",
                    supportsRangeRequests = true,
                ),
            ),
        )
        val remuxDecision = directDecision.copy(
            decision = ClientPlaybackDecision(
                mode = ClientPlaybackMode.Remux,
                reason = "container",
                transcodePlan = ClientTranscodePlan(
                    outputContainer = ClientOutputContainer.Mkv,
                    videoCodec = "h264",
                    audioCodec = "aac",
                    hardwareAcceleration = ClientHardwareAcceleration.None,
                ),
            ),
        )
        val hlsDecision = directDecision.copy(
            decision = ClientPlaybackDecision(
                mode = ClientPlaybackMode.Transcode,
                reason = "needs hls",
                transcodePlan = ClientTranscodePlan(
                    outputContainer = ClientOutputContainer.Hls,
                    videoCodec = "h264",
                    audioCodec = "aac",
                    hardwareAcceleration = ClientHardwareAcceleration.None,
                ),
            ),
        )

        val direct = client.recommendedPlaybackTarget(profile, "secret-token", directDecision)
        val remux = client.recommendedPlaybackTarget(profile, "secret-token", remuxDecision)
        val hls = client.recommendedPlaybackTarget(profile, "secret-token", hlsDecision)

        assertEquals("http://home.example.test/sources/source%201/stream", direct?.request?.url)
        assertEquals(
            "http://home.example.test/sources/source%201/stream/remux?output_container=mkv",
            remux?.request?.url,
        )
        assertEquals(
            "http://home.example.test/sources/source%201/stream/hls/playlist.m3u8",
            hls?.request?.url,
        )
        listOf(direct, remux, hls).forEach { target ->
            requireNotNull(target)
            assertFalse(target.safeRequest.toString().contains("secret-token"))
            assertFalse(target.safeRequest.toString().contains("file:///srv"))
        }
    }

    @Test
    fun `preparing remux playback target reads session id from public head response`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                ok(
                    body = "",
                    headers = mapOf(
                        TaruPublicApiContract.apiVersionHeader to listOf("v1"),
                        TaruPublicApiContract.playbackSessionIdHeader to listOf("session-remux-1"),
                    ),
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)
        val result = client.prepareRecommendedPlaybackTarget(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            decision = playbackDecision(ClientPlaybackMode.Remux, ClientOutputContainer.Mkv),
        )

        assertTrue(result is PlaybackResult.Success)
        val success = result as PlaybackResult.Success
        assertEquals("session-remux-1", success.value.sessionId)
        assertEquals("HEAD", transport.requests.single().method)
        assertEquals(
            "http://home.example.test/api/sources/source%201/stream/remux?output_container=mkv",
            transport.requests.single().url,
        )
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertFalse(success.value.toString().contains("secret-token"))
        assertFalse(success.request.toString().contains("secret-token"))
    }

    @Test
    fun `preparing hls playback target reads session id from public playlist response`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                ok(
                    body = "#EXTM3U\n",
                    headers = mapOf(
                        TaruPublicApiContract.apiVersionHeader to listOf("v1"),
                        TaruPublicApiContract.playbackSessionIdHeader to listOf("session-hls-1"),
                    ),
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)
        val result = client.prepareRecommendedPlaybackTarget(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            decision = playbackDecision(ClientPlaybackMode.Transcode, ClientOutputContainer.Hls),
        )

        assertTrue(result is PlaybackResult.Success)
        val success = result as PlaybackResult.Success
        assertEquals("session-hls-1", success.value.sessionId)
        assertEquals("GET", transport.requests.single().method)
        assertEquals(
            "http://home.example.test/sources/source%201/stream/hls/playlist.m3u8",
            transport.requests.single().url,
        )
        assertFalse(success.request.toString().contains("secret-token"))
    }

    @Test
    fun `session backed playback target fails when public session header is missing`() = runBlocking {
        val transport = FakePlaybackTransport(ResponseStep(ok(body = "")))
        val client = TaruPlaybackClient(transport)
        val result = client.prepareRecommendedPlaybackTarget(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            decision = playbackDecision(ClientPlaybackMode.Remux, ClientOutputContainer.Mp4),
        )

        assertTrue(result is PlaybackResult.Failure)
        val diagnostics = (result as PlaybackResult.Failure).diagnostics
        assertEquals(PlaybackFailureCategory.MissingSession, diagnostics.category)
        assertEquals("HEAD", transport.requests.single().method)
        assertFalse(diagnostics.toString().contains("secret-token"))
    }

    @Test
    fun `direct play target preparation remains sessionless and does not hit transport`() = runBlocking {
        val transport = FakePlaybackTransport()
        val client = TaruPlaybackClient(transport)
        val result = client.prepareRecommendedPlaybackTarget(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            decision = playbackDecision(ClientPlaybackMode.DirectPlay),
        )

        assertTrue(result is PlaybackResult.Success)
        val success = result as PlaybackResult.Success
        assertEquals(null, success.value.sessionId)
        assertEquals("GET", success.value.request.method)
        assertEquals("http://home.example.test/sources/source%201/stream", success.value.request.url)
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `playback errors are actionable and sanitized`() = runBlocking {
        val transport = FakePlaybackTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 400,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"unsupported","message":"ffmpeg.exe -i C:\\media\\night.mkv secret-token file:///tmp/source.mkv"}""",
                ),
            ),
        )
        val client = TaruPlaybackClient(transport)

        val result = client.getPlaybackDecision(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sourceId = "source-1",
        )

        assertTrue(result is PlaybackResult.Failure)
        val diagnostics = (result as PlaybackResult.Failure).diagnostics
        assertEquals(PlaybackFailureCategory.UnsupportedSource, diagnostics.category)
        assertEquals(400, diagnostics.statusCode)
        assertEquals("unsupported", diagnostics.publicError?.code)
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
        assertFalse(diagnostics.toString().contains("file:///tmp"))
        assertFalse(diagnostics.toString().contains("ffmpeg.exe"))
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
    }

    @Test
    fun `blank source routes fail locally without transport`() = runBlocking {
        val transport = FakePlaybackTransport()
        val client = TaruPlaybackClient(transport)

        val probe = client.getSourceProbe(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sourceId = " ",
        )
        val decision = client.getPlaybackDecision(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sourceId = " ",
        )

        assertTrue(probe is PlaybackResult.Failure)
        assertEquals(
            PlaybackFailureCategory.MissingSource,
            (probe as PlaybackResult.Failure).diagnostics.category,
        )
        assertTrue(decision is PlaybackResult.Failure)
        assertEquals(
            PlaybackFailureCategory.MissingSource,
            (decision as PlaybackResult.Failure).diagnostics.category,
        )
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `transport playback failure redacts token and local paths`() = runBlocking {
        val transport = FakePlaybackTransport(
            ThrowStep(IOException("failed for secret-token at C:\\media\\night.mkv")),
        )
        val client = TaruPlaybackClient(transport)

        val result = client.getPlaybackDecision(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            sourceId = "source-1",
        )

        assertTrue(result is PlaybackResult.Failure)
        val diagnostics = (result as PlaybackResult.Failure).diagnostics
        assertEquals(PlaybackFailureCategory.UnreachableServer, diagnostics.category)
        assertEquals("transport_error", diagnostics.publicError?.code)
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
    }

    private fun profile(baseUrl: String): ServerProfile =
        ServerProfile(
            id = "server-1",
            displayName = "Home",
            baseUrl = baseUrl,
            tokenReference = "server-token:server-1",
            lastObservedApiVersion = "v1",
        )

    private fun ok(body: String): TaruHttpResponse =
        TaruHttpResponse(
            statusCode = 200,
            headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
            body = body,
        )

    private fun ok(
        body: String,
        headers: Map<String, List<String>>,
    ): TaruHttpResponse =
        TaruHttpResponse(
            statusCode = 200,
            headers = headers,
            body = body,
        )

    private fun playbackDecision(
        mode: ClientPlaybackMode,
        outputContainer: ClientOutputContainer = ClientOutputContainer.Mp4,
    ): PlaybackDecisionResponse =
        PlaybackDecisionResponse(
            source = PlaybackMediaSourceDto(
                id = "source 1",
                libraryId = "library-1",
                itemId = "item-1",
                locator = "file:///srv/media/night-harbor.mkv",
                fileName = "night-harbor.mkv",
            ),
            decision = when (mode) {
                ClientPlaybackMode.DirectPlay -> ClientPlaybackDecision(
                    mode = ClientPlaybackMode.DirectPlay,
                    reason = "direct",
                    directPlay = ClientDirectPlayPlan(
                        sourceId = "source 1",
                        contentType = "video/x-matroska",
                        supportsRangeRequests = true,
                    ),
                )
                ClientPlaybackMode.Remux -> ClientPlaybackDecision(
                    mode = ClientPlaybackMode.Remux,
                    reason = "container",
                    transcodePlan = ClientTranscodePlan(
                        outputContainer = outputContainer,
                        videoCodec = "h264",
                        audioCodec = "aac",
                        hardwareAcceleration = ClientHardwareAcceleration.None,
                    ),
                )
                ClientPlaybackMode.Transcode -> ClientPlaybackDecision(
                    mode = ClientPlaybackMode.Transcode,
                    reason = "needs hls",
                    transcodePlan = ClientTranscodePlan(
                        outputContainer = outputContainer,
                        videoCodec = "h264",
                        audioCodec = "aac",
                        hardwareAcceleration = ClientHardwareAcceleration.None,
                    ),
                )
            },
        )
}

private sealed interface FakeStep

private data class ResponseStep(val response: TaruHttpResponse) : FakeStep

private data class ThrowStep(val error: IOException) : FakeStep

private class FakePlaybackTransport(
    vararg steps: FakeStep,
) : TaruHttpTransport {
    private val steps = ArrayDeque(steps.toList())
    val requests = mutableListOf<TaruHttpRequest>()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return when (val step = steps.removeFirst()) {
            is ResponseStep -> step.response
            is ThrowStep -> throw step.error
        }
    }
}
