package dev.taru.android.browse

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import java.io.IOException
import javax.net.ssl.SSLException
import kotlinx.serialization.SerializationException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

class TaruBrowseClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    suspend fun listLibraries(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<LibraryListResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/libraries?limit=${page.limit}&offset=${page.offset}",
        )

    suspend fun listItems(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<ItemsResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/items?limit=${page.limit}&offset=${page.offset}",
        )

    private suspend inline fun <reified T> executeJson(
        profile: ServerProfile,
        accessToken: String,
        pathAndQuery: String,
    ): BrowseResult<T> {
        if (accessToken.isBlank()) {
            return failure(
                category = BrowseFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before browsing.",
            )
        }

        val request = TaruHttpRequest(
            method = "GET",
            url = joinUrl(profile.baseUrl, pathAndQuery),
            headers = mapOf("Authorization" to "Bearer $accessToken"),
        )
        val response = when (val result = executeOrFailure(request, accessToken)) {
            is TransportResult.Failure -> return result.failure
            is TransportResult.Response -> result.response
        }

        if (!response.isSuccessful()) {
            return httpFailure(request, response, accessToken)
        }

        val observedApiVersion = response.header(TaruPublicApiContract.apiVersionHeader)
        if (observedApiVersion != null && observedApiVersion != TaruPublicApiContract.expectedApiVersion) {
            return failure(
                category = BrowseFailureCategory.UnsupportedApiVersion,
                userMessage = "This server uses an unsupported Public Client API version.",
                observedApiVersion = observedApiVersion,
                request = safeRequest(request),
            )
        }

        val decoded = try {
            json.decodeFromString<T>(response.body)
        } catch (_: SerializationException) {
            return invalidResponseFailure(request)
        } catch (_: IllegalArgumentException) {
            return invalidResponseFailure(request)
        }

        return BrowseResult.Success(
            value = decoded,
            request = safeRequest(request),
        )
    }

    private suspend fun executeOrFailure(
        request: TaruHttpRequest,
        accessToken: String,
    ): TransportResult =
        try {
            TransportResult.Response(transport.execute(request))
        } catch (error: SSLException) {
            TransportResult.Failure(
                failure(
                    category = BrowseFailureCategory.TlsOrCertificate,
                    userMessage = "The server TLS certificate could not be trusted.",
                    request = safeRequest(request),
                ),
            )
        } catch (error: IOException) {
            TransportResult.Failure(
                failure(
                    category = BrowseFailureCategory.UnreachableServer,
                    userMessage = "The server could not be reached. Check the address and network.",
                    publicError = PublicErrorEnvelope(
                        code = "transport_error",
                        message = SensitiveText.sanitize(error.message.orEmpty(), listOf(accessToken)),
                    ),
                    request = safeRequest(request),
                ),
            )
        }

    private fun httpFailure(
        request: TaruHttpRequest,
        response: TaruHttpResponse,
        accessToken: String,
    ): BrowseResult.Failure {
        val category = when (response.statusCode) {
            401 -> BrowseFailureCategory.Unauthorized
            403 -> BrowseFailureCategory.Forbidden
            else -> BrowseFailureCategory.PublicApiError
        }
        val userMessage = when (category) {
            BrowseFailureCategory.Unauthorized ->
                "The access token is invalid or expired."
            BrowseFailureCategory.Forbidden ->
                "This access token cannot browse the requested content."
            else ->
                "The server returned a public API error."
        }

        return failure(
            category = category,
            userMessage = userMessage,
            statusCode = response.statusCode,
            observedApiVersion = response.header(TaruPublicApiContract.apiVersionHeader),
            publicError = parsePublicError(response.body, accessToken),
            request = safeRequest(request),
        )
    }

    private fun invalidResponseFailure(request: TaruHttpRequest): BrowseResult.Failure =
        failure(
            category = BrowseFailureCategory.InvalidResponse,
            userMessage = "The server response could not be understood.",
            request = safeRequest(request),
        )

    private fun parsePublicError(
        body: String,
        accessToken: String,
    ): PublicErrorEnvelope? =
        try {
            SensitiveText.sanitizeEnvelope(
                json.decodeFromString<PublicErrorEnvelope>(body),
                listOf(accessToken),
            )
        } catch (_: SerializationException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }

    private fun safeRequest(request: TaruHttpRequest): SafeRequestPreview =
        SafeRequestPreview(
            method = request.method,
            url = request.url,
            headers = request.headers.mapValues { (name, value) ->
                if (name.equals("Authorization", ignoreCase = true)) {
                    "Bearer ${TaruPublicApiContract.redacted}"
                } else {
                    SensitiveText.sanitize(value)
                }
            },
        )

    private fun failure(
        category: BrowseFailureCategory,
        userMessage: String,
        statusCode: Int? = null,
        observedApiVersion: String? = null,
        publicError: PublicErrorEnvelope? = null,
        request: SafeRequestPreview? = null,
    ): BrowseResult.Failure =
        BrowseResult.Failure(
            diagnostics = SafeBrowseDiagnostics(
                category = category,
                userMessage = userMessage,
                statusCode = statusCode,
                observedApiVersion = observedApiVersion,
                publicError = publicError,
                request = request,
            ),
        )

    private fun joinUrl(baseUrl: String, pathAndQuery: String): String =
        "${baseUrl.trimEnd('/')}$pathAndQuery"

    private sealed interface TransportResult {
        data class Response(val response: TaruHttpResponse) : TransportResult
        data class Failure(val failure: BrowseResult.Failure) : TransportResult
    }
}
