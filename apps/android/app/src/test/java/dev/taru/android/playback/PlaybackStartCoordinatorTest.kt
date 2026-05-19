package dev.taru.android.playback

import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.player.DevicePlaybackPosition
import dev.taru.android.player.DevicePlaybackPositionKey
import dev.taru.android.player.InMemoryDevicePlaybackPositionStore
import dev.taru.android.player.PlaybackResumeSource
import dev.taru.android.userplayback.UserPlaybackStateDto
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PlaybackStartCoordinatorTest {
    @Test
    fun `remux start preflights session and builds launch with authoritative resume`() = runBlocking {
        val transport = StartCoordinatorFakeTransport(
            StartCoordinatorResponseStep(
                ok(
                    body = "",
                    headers = mapOf(
                        TaruPublicApiContract.apiVersionHeader to listOf("v1"),
                        TaruPublicApiContract.playbackSessionIdHeader to listOf("session-remux-1"),
                    ),
                ),
            ),
        )
        val playbackClient = TaruPlaybackClient(transport)
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
                accessToken = "secret-token",
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
        val playbackClient = TaruPlaybackClient(transport)
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
                        accessToken = "secret-token",
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
        val playbackClient = TaruPlaybackClient(transport)
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
                accessToken = "secret-token",
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
        assertEquals("Taru Playback", success.launch.title)
        assertEquals(ClientPlaybackMode.DirectPlay, success.launch.playbackMode)
        assertEquals(null, success.launch.sessionId)
        assertEquals(42_000L, success.launch.resumePositionMs)
        assertEquals(PlaybackResumeSource.DeviceLocal, success.launch.resumeSource)
        assertEquals("Bearer current-secret-token", success.launch.request.headers["Authorization"])
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
    ): TaruHttpResponse =
        TaruHttpResponse(
            statusCode = 200,
            headers = headers,
            body = body,
        )
}

private sealed interface StartCoordinatorFakeStep

private data class StartCoordinatorResponseStep(
    val response: TaruHttpResponse,
) : StartCoordinatorFakeStep

private class StartCoordinatorFakeTransport(
    vararg steps: StartCoordinatorFakeStep,
) : TaruHttpTransport {
    private val steps = ArrayDeque(steps.toList())
    val requests = mutableListOf<TaruHttpRequest>()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return when (val step = steps.removeFirst()) {
            is StartCoordinatorResponseStep -> step.response
        }
    }
}
