package dev.nako.android.playback

import dev.nako.android.connection.InMemoryTokenVault
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.connection.NakoHttpResponse
import dev.nako.android.connection.NakoHttpTransport
import dev.nako.android.player.DevicePlaybackPosition
import dev.nako.android.player.DevicePlaybackPositionKey
import dev.nako.android.player.InMemoryDevicePlaybackPositionStore
import dev.nako.android.player.PlaybackResumeSource
import dev.nako.android.userplayback.UserPlaybackStateDto
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import dev.nako.sdk.NAKO_API_VERSION_HEADER
import dev.nako.sdk.NAKO_PLAYBACK_SESSION_ID_HEADER

class PlaybackStartCoordinatorTest {
    @Test
    fun `remux start preflights session and builds launch with authoritative resume`() = runBlocking {
        val transport = StartCoordinatorFakeTransport(
            StartCoordinatorResponseStep(
                ok(
                    body = "",
                    headers = mapOf(
                        NAKO_API_VERSION_HEADER to listOf("v1"),
                        NAKO_PLAYBACK_SESSION_ID_HEADER to listOf("session-remux-1"),
                    ),
                ),
            ),
        )
        val playbackClient = NakoPlaybackClient(transport)
        val profile = profile("http://home.example.test/api")
        val tokenVault = InMemoryTokenVault().apply {
            saveToken(profile.tokenReference, "secret-token")
        }
        val capabilities = PlaybackCapabilities(
            directPlay = true,
            containers = listOf("mp4"),
            videoCodecs = listOf("h264"),
            audioCodecs = listOf("aac"),
        )
        val decision = playbackDecision(ClientPlaybackMode.Remux, ClientOutputContainer.Mkv)
        val target = requireNotNull(
            playbackClient.recommendedPlaybackTarget(
                profile = profile,
                decision = decision,
                capabilities = capabilities,
            ),
        )
        val coordinator = PlaybackStartCoordinator(
            playbackClient = playbackClient,
            positionStore = InMemoryDevicePlaybackPositionStore(),
        )

        val result = coordinator.start(
            profile = profile,
            tokenVault = tokenVault,
            request = PlaybackStartRequest(
                title = "Night Harbor",
                mediaItemId = "item-1",
                sourceId = "source 1",
                decision = decision,
                capabilities = capabilities,
                target = target,
                userPlaybackState = userPlaybackState(
                    resumePositionMs = 92_000,
                    sourceId = "source 1",
                ),
            ),
        )

        assertTrue(result is PlaybackStartResult.Success)
        val success = result as PlaybackStartResult.Success
        assertEquals("session-remux-1", success.preparedTarget.sessionId)
        assertEquals("Night Harbor", success.launch.title)
        assertEquals(ClientPlaybackMode.Remux, success.launch.playbackMode)
        assertEquals("session-remux-1", success.launch.sessionId)
        assertEquals(92_000L, success.launch.resumePositionMs)
        assertEquals(PlaybackResumeSource.UserPlaybackState, success.launch.resumeSource)
        assertEquals("HEAD", transport.requests.single().method)
        assertEquals(
            "http://home.example.test/api/sources/source%201/stream/remux?direct_play=true&container=mp4&video_codec=h264&audio_codec=aac&output_container=mkv",
            transport.requests.single().url,
        )
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertFalse(success.toString().contains("secret-token"))
        assertFalse(success.launch.toString().contains("secret-token"))
    }

    @Test
    fun `missing token fails start without preflight`() = runBlocking {
        val transport = StartCoordinatorFakeTransport()
        val playbackClient = NakoPlaybackClient(transport)
        val profile = profile("http://home.example.test/api")
        val tokenVault = InMemoryTokenVault()
        val decision = playbackDecision(ClientPlaybackMode.Remux, ClientOutputContainer.Mp4)
        val coordinator = PlaybackStartCoordinator(
            playbackClient = playbackClient,
            positionStore = InMemoryDevicePlaybackPositionStore(),
        )

        val result = coordinator.start(
            profile = profile,
            tokenVault = tokenVault,
            request = PlaybackStartRequest(
                title = "Night Harbor",
                mediaItemId = "item-1",
                sourceId = "source 1",
                decision = decision,
                capabilities = PlaybackCapabilities(),
                target = requireNotNull(
                    playbackClient.recommendedPlaybackTarget(
                        profile = profile,
                        decision = decision,
                    ),
                ),
            ),
        )

        assertTrue(result is PlaybackStartResult.Failure)
        assertEquals(
            PlaybackFailureCategory.MissingAccessToken,
            (result as PlaybackStartResult.Failure).diagnostics.category,
        )
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `direct target uses current token without preflight and propagates device resume`() = runBlocking {
        val transport = StartCoordinatorFakeTransport()
        val playbackClient = NakoPlaybackClient(transport)
        val profile = profile("http://home.example.test/api")
        val tokenVault = InMemoryTokenVault().apply {
            saveToken(profile.tokenReference, "current-secret-token")
        }
        val positionStore = InMemoryDevicePlaybackPositionStore().apply {
            save(
                DevicePlaybackPosition(
                    key = DevicePlaybackPositionKey(
                        serverProfileId = profile.id,
                        mediaItemId = "item-1",
                        sourceId = "source 1",
                    ),
                    positionMs = 42_000,
                    updatedAtMillis = 1,
                ),
            )
        }
        val decision = playbackDecision(ClientPlaybackMode.DirectPlay)
        val target = requireNotNull(
            playbackClient.recommendedPlaybackTarget(
                profile = profile,
                decision = decision,
            ),
        )
        val coordinator = PlaybackStartCoordinator(
            playbackClient = playbackClient,
            positionStore = positionStore,
        )

        val result = coordinator.start(
            profile = profile,
            tokenVault = tokenVault,
            request = PlaybackStartRequest(
                title = "",
                mediaItemId = "item-1",
                sourceId = "source 1",
                decision = decision,
                capabilities = PlaybackCapabilities(),
                target = target,
                userPlaybackState = userPlaybackState(
                    resumePositionMs = 99_000,
                    sourceId = "source-other",
                ),
            ),
        )

        assertTrue(result is PlaybackStartResult.Success)
        val success = result as PlaybackStartResult.Success
        assertEquals("Nako Playback", success.launch.title)
        assertEquals(ClientPlaybackMode.DirectPlay, success.launch.playbackMode)
        assertEquals(null, success.launch.sessionId)
        assertEquals(42_000L, success.launch.resumePositionMs)
        assertEquals(PlaybackResumeSource.DeviceLocal, success.launch.resumeSource)
        assertEquals(null, success.launch.request.headers["Authorization"])
        assertEquals(
            "Bearer current-secret-token",
            success.launch.authenticatedRequest("current-secret-token").headers["Authorization"],
        )
        assertFalse(success.launch.toString().contains("current-secret-token"))
        assertTrue(transport.requests.isEmpty())
        assertFalse(success.toString().contains("current-secret-token"))
    }

    private fun profile(baseUrl: String): ServerProfile =
        ServerProfile(
            id = "server-1",
            displayName = "Home",
            baseUrl = baseUrl,
            tokenReference = "server-token:server-1",
            lastObservedApiVersion = "v1",
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
                ClientPlaybackMode.Unknown -> ClientPlaybackDecision(
                    mode = ClientPlaybackMode.Unknown,
                    reason = "unknown",
                )
            },
        )

    private fun userPlaybackState(
        resumePositionMs: Long?,
        sourceId: String?,
        watched: Boolean = false,
    ): UserPlaybackStateDto =
        UserPlaybackStateDto(
            itemId = "item-1",
            sourceId = sourceId,
            resumePositionMs = resumePositionMs,
            durationMs = 6_360_000,
            progressPercent = 1.44f,
            watched = watched,
            version = 1,
        )

    private fun ok(
        body: String,
        headers: Map<String, List<String>>,
    ): NakoHttpResponse =
        NakoHttpResponse(
            statusCode = 200,
            headers = headers,
            body = body,
        )
}

private sealed interface StartCoordinatorFakeStep

private data class StartCoordinatorResponseStep(
    val response: NakoHttpResponse,
) : StartCoordinatorFakeStep

private class StartCoordinatorFakeTransport(
    vararg steps: StartCoordinatorFakeStep,
) : NakoHttpTransport {
    private val steps = ArrayDeque(steps.toList())
    val requests = mutableListOf<NakoHttpRequest>()

    override suspend fun execute(request: NakoHttpRequest): NakoHttpResponse {
        requests += request
        return when (val step = steps.removeFirst()) {
            is StartCoordinatorResponseStep -> step.response
        }
    }
}
