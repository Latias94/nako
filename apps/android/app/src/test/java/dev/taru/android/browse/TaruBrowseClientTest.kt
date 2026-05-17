package dev.taru.android.browse

import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileRepository
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

class TaruBrowseClientTest {
    @Test
    fun `list libraries decodes page and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "libraries": [
                        {
                          "id": "library-1",
                          "name": "Movies",
                          "roots": ["file:///srv/media/movies"],
                          "options": {
                            "domain": "video",
                            "preset": "movies",
                            "scan": {"realtime_monitor": true}
                          }
                        }
                      ],
                      "page": {"limit": 20, "offset": 40, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            page = PageRequest(limit = 20, offset = 40),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/libraries?limit=20&offset=40", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Movies", success.value.libraries.single().name)
        assertEquals("movies", success.value.libraries.single().options?.preset)
        assertEquals(40L, success.value.page.offset)
        assertFalse(success.request.toString().contains("secret-token"))
    }

    @Test
    fun `list items decodes minimal media item tracer`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "items": [
                        {
                          "id": "item-1",
                          "kind": "movie",
                          "parent_id": null,
                          "metadata": {
                            "title": "Arrival",
                            "release_date": "2016-11-11",
                            "runtime_minutes": 116,
                            "genres": ["Science Fiction"],
                            "tags": [],
                            "ratings": [],
                            "images": []
                          }
                        }
                      ],
                      "page": {"limit": 24, "offset": 0, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listItems(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            page = PageRequest(limit = 24, offset = 0),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/items?limit=24&offset=0", transport.requests.single().url)
        assertEquals("Arrival", success.value.items.single().metadata.title)
        assertEquals("movie", success.value.items.single().kind)
        assertEquals(1, success.value.page.returned)
    }

    @Test
    fun `empty libraries response remains a successful empty state input`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok("""{"libraries":[],"page":{"limit":50,"offset":0,"returned":0}}"""),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertTrue(success.value.libraries.isEmpty())
        assertEquals(0, success.value.page.returned)
    }

    @Test
    fun `unauthorized browse response is actionable and sanitized`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 401,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"unauthorized","message":"bad token secret-token in file:///tmp/source.mkv"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.Unauthorized, diagnostics.category)
        assertEquals(401, diagnostics.statusCode)
        assertEquals("unauthorized", diagnostics.publicError?.code)
        assertEquals("bad token <redacted> in <local-path>", diagnostics.publicError?.message)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("file:///tmp"))
    }

    @Test
    fun `unreachable browse request returns sanitized diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ThrowStep(IOException("failed with secret-token at C:\\media\\demo.mkv")),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listItems(
            profile = profile("https://taru.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.UnreachableServer, diagnostics.category)
        assertEquals("transport_error", diagnostics.publicError?.code)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
    }

    @Test
    fun `public api browse errors keep diagnostics client safe`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 500,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"storage_error","message":"ffmpeg.exe -i C:\\media\\demo.mkv secret-token"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.PublicApiError, diagnostics.category)
        assertEquals("storage_error", diagnostics.publicError?.code)
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
        assertFalse(diagnostics.toString().contains("ffmpeg.exe"))
    }

    @Test
    fun `active profile switching changes browse base url and token reference`() = runBlocking {
        val repository = ServerProfileRepository()
        val vault = InMemoryTokenVault()
        val home = repository.upsertConnectedProfile(
            displayName = "Home",
            tokenReference = null,
            result = successFor("http://home.example.test"),
        )
        vault.saveToken(home.tokenReference, "home-token")
        val lab = repository.upsertConnectedProfile(
            displayName = "Lab",
            tokenReference = null,
            result = successFor("http://lab.example.test"),
        )
        vault.saveToken(lab.tokenReference, "lab-token")
        val transport = FakeTransport(
            ResponseStep(ok("""{"libraries":[],"page":{"limit":50,"offset":0,"returned":0}}""")),
            ResponseStep(ok("""{"libraries":[],"page":{"limit":50,"offset":0,"returned":0}}""")),
        )
        val client = TaruBrowseClient(transport)

        val activeLab = repository.activeProfile() ?: error("active profile required")
        client.listLibraries(activeLab, vault.readToken(activeLab.tokenReference).orEmpty())
        repository.switchActive(home.id)
        val activeHome = repository.activeProfile() ?: error("active profile required")
        client.listLibraries(activeHome, vault.readToken(activeHome.tokenReference).orEmpty())

        assertEquals(
            listOf(
                "http://lab.example.test/libraries?limit=50&offset=0",
                "http://home.example.test/libraries?limit=50&offset=0",
            ),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer lab-token", "Bearer home-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
    }

    private fun profile(baseUrl: String): ServerProfile =
        ServerProfile(
            id = "server-1",
            displayName = "Home",
            baseUrl = baseUrl,
            tokenReference = "server-token:server-1",
            lastObservedApiVersion = "v1",
        )

    private fun successFor(baseUrl: String): ConnectionCheckResult.Success =
        ConnectionCheckResult.Success(
            normalizedBaseUrl = baseUrl,
            apiVersion = "v1",
            checkedAtMillis = 42L,
            healthRequest = SafeRequestPreview("GET", "$baseUrl/health"),
            authProbeRequest = SafeRequestPreview(
                method = "GET",
                url = "$baseUrl/libraries?limit=1&offset=0",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        )

    private fun ok(body: String): TaruHttpResponse =
        TaruHttpResponse(
            statusCode = 200,
            headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
            body = body,
        )
}

private sealed interface FakeStep

private data class ResponseStep(val response: TaruHttpResponse) : FakeStep

private data class ThrowStep(val error: IOException) : FakeStep

private class FakeTransport(
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
