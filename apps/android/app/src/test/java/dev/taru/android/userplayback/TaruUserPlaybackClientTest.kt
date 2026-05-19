package dev.taru.android.userplayback

import dev.taru.android.browse.PageRequest
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

class TaruUserPlaybackClientTest {
    @Test
    fun `continue watching uses authoritative user playback route and safe response surface`() = runBlocking {
        val transport = FakeUserPlaybackTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "items": [
                        {
                          "item": {
                            "id": "item 1",
                            "kind": "movie",
                            "metadata": {
                              "title": "Night Harbor",
                              "release_date": "2026-01-01",
                              "runtime_minutes": 106,
                              "genres": ["Mystery"],
                              "tags": [],
                              "ratings": []
                            }
                          },
                          "state": {
                            "item_id": "item 1",
                            "source_id": "source 1",
                            "resume_position_ms": 92000,
                            "duration_ms": 6360000,
                            "progress_percent": 1.44,
                            "watched": false,
                            "watched_at": null,
                            "last_played_at": "2026-05-19T00:00:00Z",
                            "updated_at": "2026-05-19T00:00:00Z",
                            "version": 7
                          },
                          "images": []
                        }
                      ],
                      "page": {"limit": 12, "offset": 0, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruUserPlaybackClient(transport)

        val result = client.continueWatching(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            page = PageRequest(limit = 12, offset = 0),
        )

        assertTrue(result is UserPlaybackResult.Success)
        val success = result as UserPlaybackResult.Success
        assertEquals(
            "http://home.example.test/api/users/me/playback-state/continue-watching?limit=12&offset=0",
            transport.requests.single().url,
        )
        assertEquals("GET", transport.requests.single().method)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Night Harbor", success.value.items.single().item.metadata.title)
        assertEquals(92_000L, success.value.items.single().state.resumePositionMs)
        assertEquals(false, success.value.items.single().state.watched)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `state lookup update progress and watched state use item scoped public routes`() = runBlocking {
        val transport = FakeUserPlaybackTransport(
            ResponseStep(ok(stateBody(positionMs = 92_000, watched = false))),
            ResponseStep(ok(stateBody(positionMs = 123_000, watched = false))),
            ResponseStep(ok(stateBody(positionMs = null, watched = true))),
        )
        val client = TaruUserPlaybackClient(transport)
        val profile = profile("http://home.example.test")

        val state = client.getState(profile, "secret-token", "item 1")
        val progress = client.updateProgress(
            profile = profile,
            accessToken = "secret-token",
            itemId = "item 1",
            request = UpdatePlaybackProgressRequest(
                sourceId = "source 1",
                positionMs = 123_000,
                durationMs = 6_360_000,
                reportedAt = "2026-05-19T00:00:00Z",
            ),
        )
        val watched = client.setWatchedState(
            profile = profile,
            accessToken = "secret-token",
            itemId = "item 1",
            request = SetWatchedStateRequest(
                watched = true,
                sourceId = "source 1",
                positionMs = 6_350_000,
                durationMs = 6_360_000,
                markedAt = "2026-05-19T00:01:00Z",
            ),
        )

        assertTrue(state is UserPlaybackResult.Success)
        assertTrue(progress is UserPlaybackResult.Success)
        assertTrue(watched is UserPlaybackResult.Success)
        assertEquals(
            listOf("GET", "PUT", "PUT"),
            transport.requests.map { it.method },
        )
        assertEquals(
            listOf(
                "http://home.example.test/users/me/playback-state/items/item%201",
                "http://home.example.test/users/me/playback-state/items/item%201/progress",
                "http://home.example.test/users/me/playback-state/items/item%201/watched",
            ),
            transport.requests.map { it.url },
        )
        assertEquals("application/json", transport.requests[1].headers["Content-Type"])
        assertEquals("application/json", transport.requests[2].headers["Content-Type"])
        assertTrue(transport.requests[1].body.orEmpty().contains(""""position_ms":123000"""))
        assertTrue(transport.requests[1].body.orEmpty().contains(""""source_id":"source 1""""))
        assertTrue(transport.requests[2].body.orEmpty().contains(""""watched":true"""))
        assertTrue(transport.requests[2].body.orEmpty().contains(""""marked_at":"2026-05-19T00:01:00Z""""))
        assertEquals(123_000L, (progress as UserPlaybackResult.Success).value.state.resumePositionMs)
        assertEquals(true, (watched as UserPlaybackResult.Success).value.state.watched)
    }

    @Test
    fun `blank item and missing token fail locally without transport`() = runBlocking {
        val transport = FakeUserPlaybackTransport()
        val client = TaruUserPlaybackClient(transport)

        val blankItem = client.updateProgress(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = " ",
            request = UpdatePlaybackProgressRequest(positionMs = 42),
        )
        val missingToken = client.continueWatching(
            profile = profile("http://home.example.test"),
            accessToken = " ",
        )

        assertTrue(blankItem is UserPlaybackResult.Failure)
        assertEquals(
            UserPlaybackFailureCategory.MissingItem,
            (blankItem as UserPlaybackResult.Failure).diagnostics.category,
        )
        assertTrue(missingToken is UserPlaybackResult.Failure)
        assertEquals(
            UserPlaybackFailureCategory.MissingAccessToken,
            (missingToken as UserPlaybackResult.Failure).diagnostics.category,
        )
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `user playback errors are actionable and sanitized`() = runBlocking {
        val transport = FakeUserPlaybackTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 401,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"unauthorized","message":"bad token secret-token in C:\\media\\night.mkv"}""",
                ),
            ),
        )
        val client = TaruUserPlaybackClient(transport)

        val result = client.getState(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item-1",
        )

        assertTrue(result is UserPlaybackResult.Failure)
        val diagnostics = (result as UserPlaybackResult.Failure).diagnostics
        assertEquals(UserPlaybackFailureCategory.Unauthorized, diagnostics.category)
        assertEquals(401, diagnostics.statusCode)
        assertEquals("unauthorized", diagnostics.publicError?.code)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
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

    private fun stateBody(
        positionMs: Long?,
        watched: Boolean,
    ): String {
        val resume = positionMs?.toString() ?: "null"
        return """
        {
          "state": {
            "item_id": "item 1",
            "source_id": "source 1",
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

private sealed interface FakeStep

private data class ResponseStep(val response: TaruHttpResponse) : FakeStep

private data class ThrowStep(val error: IOException) : FakeStep

private class FakeUserPlaybackTransport(
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
