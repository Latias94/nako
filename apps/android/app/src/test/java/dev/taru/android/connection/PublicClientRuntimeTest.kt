package dev.taru.android.connection

import dev.taru.sdk.TARU_API_VERSION_HEADER
import java.io.IOException
import kotlinx.coroutines.runBlocking
import kotlinx.serialization.Serializable
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PublicClientRuntimeTest {
    @Test
    fun `authenticated json validates token before building request`() = runBlocking {
        val transport = FakeRuntimeTransport()
        val runtime = PublicClientRuntime(transport)
        var builtRequest = false

        val result = runtime.executeAuthenticatedJson<RuntimeDto, RuntimeDto>(
            accessToken = " ",
            buildRequest = {
                builtRequest = true
                TaruHttpRequest("GET", "http://home.example.test/items")
            },
            transform = { it },
        )

        assertTrue(result is PublicApiResult.Failure)
        val failure = (result as PublicApiResult.Failure).failure
        assertEquals(PublicApiFailureKind.MissingAccessToken, failure.kind)
        assertFalse(builtRequest)
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `authenticated json executes decodes transforms and redacts safe request`() = runBlocking {
        val transport = FakeRuntimeTransport(
            RuntimeResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TARU_API_VERSION_HEADER to listOf("v1")),
                    body = """{"name":"Night Harbor"}""",
                ),
            ),
        )
        val runtime = PublicClientRuntime(transport)

        val result = runtime.executeAuthenticatedJson<RuntimeDto, String>(
            accessToken = "secret-token",
            buildRequest = { token ->
                TaruHttpRequest(
                    method = "GET",
                    url = "http://home.example.test/items",
                    headers = mapOf("Authorization" to "Bearer $token"),
                )
            },
            transform = { it.name },
        )

        assertTrue(result is PublicApiResult.Success)
        val success = result as PublicApiResult.Success
        assertEquals("Night Harbor", success.value)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `core response preserves rust core safe preview for successful connection steps`() = runBlocking {
        val transport = FakeRuntimeTransport(
            RuntimeResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TARU_API_VERSION_HEADER to listOf("v2")),
                    body = """{"status":"ok"}""",
                ),
            ),
        )
        val runtime = PublicClientRuntime(transport)
        val safePreview = SafeRequestPreview(
            method = "GET",
            url = "http://home.example.test/health",
            headers = mapOf("Authorization" to "Bearer <redacted>"),
        )

        val result = runtime.executeCoreResponse(
            request = TaruHttpRequest(
                method = "GET",
                url = "http://home.example.test/health",
                headers = mapOf("Authorization" to "Bearer secret-token"),
            ),
            safeRequest = safePreview,
            secrets = listOf("secret-token"),
        )

        assertTrue(result is PublicApiResult.Success)
        val success = result as PublicApiResult.Success
        assertEquals(safePreview, success.request)
        assertEquals("v2", success.response.header(TARU_API_VERSION_HEADER))
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `core response uses rust core safe preview when transport fails before a sanitized request exists`() = runBlocking {
        val transport = FakeRuntimeTransport(
            RuntimeThrowStep(IOException("failed for secret-token at C:\\media\\night.mkv")),
        )
        val runtime = PublicClientRuntime(transport)
        val safePreview = SafeRequestPreview(
            method = "GET",
            url = "http://home.example.test/health",
        )

        val result = runtime.executeCoreResponse(
            request = TaruHttpRequest(
                method = "GET",
                url = "http://home.example.test/health",
            ),
            safeRequest = safePreview,
            secrets = listOf("secret-token"),
        )

        assertTrue(result is PublicApiResult.Failure)
        val failure = (result as PublicApiResult.Failure).failure
        assertEquals(PublicApiFailureKind.UnreachableServer, failure.kind)
        assertEquals(safePreview, failure.request)
        assertEquals("transport_error", failure.publicError?.code)
        assertFalse(failure.toString().contains("secret-token"))
        assertFalse(failure.toString().contains("C:\\media"))
    }

    @Serializable
    private data class RuntimeDto(val name: String)
}

private sealed interface FakeRuntimeStep

private data class RuntimeResponseStep(val response: TaruHttpResponse) : FakeRuntimeStep

private data class RuntimeThrowStep(val error: IOException) : FakeRuntimeStep

private class FakeRuntimeTransport(
    vararg steps: FakeRuntimeStep,
) : TaruHttpTransport {
    private val steps = ArrayDeque(steps.toList())
    val requests = mutableListOf<TaruHttpRequest>()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return when (val step = steps.removeFirst()) {
            is RuntimeResponseStep -> step.response
            is RuntimeThrowStep -> throw step.error
        }
    }
}
