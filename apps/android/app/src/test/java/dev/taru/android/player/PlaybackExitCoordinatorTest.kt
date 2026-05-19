package dev.taru.android.player

import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PlaybackExitCoordinatorTest {
    @Test
    fun `unfinished session playback saves progress reports state and cancels session through clients`() = runBlocking {
        val transport = ExitCoordinatorFakeTransport(
            ExitCoordinatorResponseStep(ok(sessionBody("session-1"))),
            ExitCoordinatorResponseStep(ok(stateBody(positionMs = 92_000, watched = false))),
        )
        val profile = profile()
        val tokenVault = InMemoryTokenVault().apply {
            saveToken(profile.tokenReference, "secret-token")
        }
        val store = InMemoryDevicePlaybackPositionStore()
        val coordinator = PlaybackExitCoordinator(
            playbackClient = TaruPlaybackClient(transport),
            userPlaybackClient = TaruUserPlaybackClient(transport),
            positionStore = store,
        )

        val result = coordinator.applyExitEffects(
            launch = launch(sessionId = "session-1"),
            snapshot = PlaybackExitSnapshot(
                isEnded = false,
                positionMs = 92_000,
                durationMs = 6_360_000,
            ),
            profile = profile,
            tokenVault = tokenVault,
        )

        assertEquals(true, result.savedDevicePosition)
        assertEquals(true, result.reportedUserPlaybackState)
        assertEquals(true, result.requestedSessionCancellation)
        assertEquals(92_000L, store.load(launch(sessionId = "session-1").positionKey)?.positionMs)
        assertEquals(
            listOf(
                "POST http://home.example.test/api/playback/sessions/session-1/cancel",
                "PUT http://home.example.test/api/users/me/playback-state/items/item-1/progress",
            ),
            transport.requests.map { "${it.method} ${it.url}" },
        )
        assertTrue(transport.requests[1].body.orEmpty().contains(""""position_ms":92000"""))
        assertEquals("Bearer secret-token", transport.requests[0].headers["Authorization"])
        assertEquals("Bearer secret-token", transport.requests[1].headers["Authorization"])
        assertFalse(result.toString().contains("secret-token"))
    }

    @Test
    fun `ended playback clears local position and reports watched without cancelling session`() = runBlocking {
        val transport = ExitCoordinatorFakeTransport(
            ExitCoordinatorResponseStep(ok(stateBody(positionMs = null, watched = true))),
        )
        val profile = profile()
        val tokenVault = InMemoryTokenVault().apply {
            saveToken(profile.tokenReference, "secret-token")
        }
        val launch = launch(sessionId = "session-1")
        val store = InMemoryDevicePlaybackPositionStore().apply {
            save(
                DevicePlaybackPosition(
                    key = launch.positionKey,
                    positionMs = 42_000,
                    updatedAtMillis = 1,
                ),
            )
        }
        val coordinator = PlaybackExitCoordinator(
            playbackClient = TaruPlaybackClient(transport),
            userPlaybackClient = TaruUserPlaybackClient(transport),
            positionStore = store,
        )

        val result = coordinator.applyExitEffects(
            launch = launch,
            snapshot = PlaybackExitSnapshot(
                isEnded = true,
                positionMs = 6_350_000,
                durationMs = 6_360_000,
            ),
            profile = profile,
            tokenVault = tokenVault,
        )

        assertEquals(false, result.savedDevicePosition)
        assertEquals(true, result.clearedDevicePosition)
        assertEquals(true, result.reportedUserPlaybackState)
        assertEquals(false, result.requestedSessionCancellation)
        assertNull(store.load(launch.positionKey))
        assertEquals(
            listOf("PUT http://home.example.test/api/users/me/playback-state/items/item-1/watched"),
            transport.requests.map { "${it.method} ${it.url}" },
        )
        assertTrue(transport.requests.single().body.orEmpty().contains(""""watched":true"""))
    }

    @Test
    fun `missing token keeps local position and skips client calls`() = runBlocking {
        val transport = ExitCoordinatorFakeTransport()
        val profile = profile()
        val store = InMemoryDevicePlaybackPositionStore()
        val coordinator = PlaybackExitCoordinator(
            playbackClient = TaruPlaybackClient(transport),
            userPlaybackClient = TaruUserPlaybackClient(transport),
            positionStore = store,
        )

        val result = coordinator.applyExitEffects(
            launch = launch(sessionId = "session-1"),
            snapshot = PlaybackExitSnapshot(
                isEnded = false,
                positionMs = 10_000,
                durationMs = 6_360_000,
            ),
            profile = profile,
            tokenVault = InMemoryTokenVault(),
        )

        assertEquals(true, result.savedDevicePosition)
        assertEquals(false, result.reportedUserPlaybackState)
        assertEquals(false, result.requestedSessionCancellation)
        assertEquals(10_000L, store.load(launch(sessionId = "session-1").positionKey)?.positionMs)
        assertTrue(transport.requests.isEmpty())
    }

    private fun launch(sessionId: String?): PlaybackLaunchRequest =
        playbackLaunchRequest(
            title = "Night Harbor",
            target = PlaybackRequestTarget(
                request = TaruHttpRequest(
                    method = "GET",
                    url = "http://home.example.test/api/sources/source-1/stream/remux",
                    headers = mapOf("Authorization" to "Bearer secret-token"),
                ),
                safeRequest = SafeRequestPreview(
                    method = "GET",
                    url = "http://home.example.test/api/sources/source-1/stream/remux",
                    headers = mapOf("Authorization" to "Bearer <redacted>"),
                ),
                sessionId = sessionId,
            ),
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.Remux,
            sessionId = sessionId,
        )

    private fun profile(): ServerProfile =
        ServerProfile(
            id = "server-1",
            displayName = "Home",
            baseUrl = "http://home.example.test/api",
            tokenReference = "server-token:server-1",
            lastObservedApiVersion = "v1",
        )

    private fun ok(body: String): TaruHttpResponse =
        TaruHttpResponse(
            statusCode = 200,
            headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
            body = body,
        )

    private fun sessionBody(sessionId: String): String =
        """
        {
          "session": {
            "id": "$sessionId",
            "source_id": "source-1",
            "kind": "remux",
            "request_key": "remux:mkv",
            "state": "cancel_requested",
            "failure_category": "cancelled",
            "failure_message": "playback session cancellation requested",
            "created_at": "2026-05-19T00:00:00Z",
            "updated_at": "2026-05-19T00:00:01Z"
          }
        }
        """.trimIndent()

    private fun stateBody(
        positionMs: Long?,
        watched: Boolean,
    ): String {
        val resume = positionMs?.toString() ?: "null"
        return """
        {
          "state": {
            "item_id": "item-1",
            "source_id": "source-1",
            "resume_position_ms": $resume,
            "duration_ms": 6360000,
            "progress_percent": 1.44,
            "watched": $watched,
            "watched_at": null,
            "last_played_at": "2026-05-19T00:00:00Z",
            "updated_at": "2026-05-19T00:00:00Z",
            "version": 7
          }
        }
        """.trimIndent()
    }
}

private sealed interface ExitCoordinatorFakeStep

private data class ExitCoordinatorResponseStep(
    val response: TaruHttpResponse,
) : ExitCoordinatorFakeStep

private class ExitCoordinatorFakeTransport(
    vararg steps: ExitCoordinatorFakeStep,
) : TaruHttpTransport {
    private val steps = ArrayDeque(steps.toList())
    val requests = mutableListOf<TaruHttpRequest>()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return when (val step = steps.removeFirst()) {
            is ExitCoordinatorResponseStep -> step.response
        }
    }
}
