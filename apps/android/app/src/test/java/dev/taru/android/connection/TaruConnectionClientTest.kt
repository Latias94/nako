package dev.taru.android.connection

import java.io.IOException
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class TaruConnectionClientTest {
    @Test
    fun `successful connection checks health then authenticated public route`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"status":"ok","version":"v1"}""",
                ),
            ),
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"items":[],"page":{"limit":1,"offset":0,"returned":0}}""",
                ),
            ),
        )
        val client = TaruConnectionClient(transport = transport, clockMillis = { 42L })

        val result = client.testConnection(
            baseUrlInput = " http://localhost:3000/ ",
            accessToken = "secret-token",
        )

        assertTrue(result is ConnectionCheckResult.Success)
        val success = result as ConnectionCheckResult.Success
        assertEquals("http://localhost:3000", success.normalizedBaseUrl)
        assertEquals("v1", success.apiVersion)
        assertEquals(42L, success.checkedAtMillis)
        assertEquals("http://localhost:3000/health", transport.requests[0].url)
        assertFalse(transport.requests[0].headers.containsKey("Authorization"))
        assertEquals("http://localhost:3000/libraries?limit=1&offset=0", transport.requests[1].url)
        assertEquals("Bearer secret-token", transport.requests[1].headers["Authorization"])
        assertEquals("Bearer <redacted>", success.authProbeRequest.headers["Authorization"])
    }

    @Test
    fun `unreachable server returns sanitized diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ThrowStep(IOException("failed with secret-token at C:\\media\\demo.mkv")),
        )
        val client = TaruConnectionClient(transport = transport)

        val result = client.testConnection(
            baseUrlInput = "https://taru.example.test",
            accessToken = "secret-token",
        )

        assertTrue(result is ConnectionCheckResult.Failure)
        val diagnostics = (result as ConnectionCheckResult.Failure).diagnostics
        assertEquals(ConnectionFailureCategory.UnreachableServer, diagnostics.category)
        assertEquals("transport_error", diagnostics.publicError?.code)
        assertNotNull(diagnostics.request)
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
    }

    @Test
    fun `unauthorized auth probe maps to actionable error and redacts token`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"status":"ok","version":"v1"}""",
                ),
            ),
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 401,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"unauthorized","message":"bad token secret-token in file:///tmp/source.mkv"}""",
                ),
            ),
        )
        val client = TaruConnectionClient(transport = transport)

        val result = client.testConnection(
            baseUrlInput = "http://localhost:3000",
            accessToken = "secret-token",
        )

        assertTrue(result is ConnectionCheckResult.Failure)
        val diagnostics = (result as ConnectionCheckResult.Failure).diagnostics
        assertEquals(ConnectionFailureCategory.Unauthorized, diagnostics.category)
        assertEquals(401, diagnostics.statusCode)
        assertEquals("unauthorized", diagnostics.publicError?.code)
        assertEquals("bad token <redacted> in <local-path>", diagnostics.publicError?.message)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("file:///tmp"))
    }

    @Test
    fun `unsupported api version is rejected before auth probe`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v2")),
                    body = """{"status":"ok","version":"v2"}""",
                ),
            ),
        )
        val client = TaruConnectionClient(transport = transport)

        val result = client.testConnection(
            baseUrlInput = "https://taru.example.test",
            accessToken = "secret-token",
        )

        assertTrue(result is ConnectionCheckResult.Failure)
        val diagnostics = (result as ConnectionCheckResult.Failure).diagnostics
        assertEquals(ConnectionFailureCategory.UnsupportedApiVersion, diagnostics.category)
        assertEquals("v2", diagnostics.observedApiVersion)
        assertEquals(1, transport.requests.size)
    }

    @Test
    fun `invalid url and missing token fail locally without transport`() = runBlocking {
        val transport = FakeTransport()
        val client = TaruConnectionClient(transport = transport)

        val invalidUrl = client.testConnection("ftp://taru.example.test", "secret-token")
        val missingToken = client.testConnection("http://localhost:3000", " ")

        assertEquals(
            ConnectionFailureCategory.InvalidUrl,
            (invalidUrl as ConnectionCheckResult.Failure).diagnostics.category,
        )
        assertEquals(
            ConnectionFailureCategory.MissingAccessToken,
            (missingToken as ConnectionCheckResult.Failure).diagnostics.category,
        )
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `profile repository keeps one active server and isolated token references`() {
        val repository = ServerProfileRepository()
        val first = repository.upsertConnectedProfile(
            displayName = "Home",
            tokenReference = null,
            result = successFor("http://home.example.test"),
        )
        val second = repository.upsertConnectedProfile(
            displayName = "Lab",
            tokenReference = null,
            result = successFor("http://lab.example.test"),
        )

        assertEquals(second.id, repository.snapshot().activeProfileId)
        assertEquals("http://lab.example.test", repository.activeBaseUrl())
        assertFalse(first.tokenReference == second.tokenReference)

        repository.switchActive(first.id)

        assertEquals(first.id, repository.snapshot().activeProfileId)
        assertEquals("http://home.example.test", repository.activeBaseUrl())
        assertEquals(
            listOf("http://home.example.test", "http://lab.example.test"),
            repository.listProfiles().map { it.baseUrl },
        )
    }

    @Test
    fun `profile failure diagnostics remain scoped to the matching server`() {
        val repository = ServerProfileRepository()
        val first = repository.upsertConnectedProfile(
            displayName = "Home",
            tokenReference = null,
            result = successFor("http://home.example.test"),
        )
        val second = repository.upsertConnectedProfile(
            displayName = "Lab",
            tokenReference = null,
            result = successFor("http://lab.example.test"),
        )

        repository.recordFailure(
            profileId = first.id,
            failure = ConnectionCheckResult.Failure(
                normalizedBaseUrl = first.baseUrl,
                diagnostics = SafeConnectionDiagnostics(
                    category = ConnectionFailureCategory.Unauthorized,
                    userMessage = "The access token is invalid or expired.",
                    publicError = PublicErrorEnvelope("unauthorized", "authentication required"),
                ),
            ),
        )

        assertEquals(second.id, repository.snapshot().activeProfileId)
        assertNull(repository.activeProfile()?.lastPublicError)

        repository.switchActive(first.id)

        assertEquals("unauthorized", repository.activeProfile()?.lastPublicError?.code)
        assertEquals(second.baseUrl, repository.listProfiles().first { it.id == second.id }.baseUrl)
    }

    @Test
    fun `token vault stores raw values behind references only`() {
        val vault = InMemoryTokenVault()

        vault.saveToken("server-token:server-1", "secret-token")

        assertEquals("secret-token", vault.readToken("server-token:server-1"))
        assertNull(vault.readToken("missing"))
        vault.deleteToken("server-token:server-1")
        assertNull(vault.readToken("server-token:server-1"))
    }

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
