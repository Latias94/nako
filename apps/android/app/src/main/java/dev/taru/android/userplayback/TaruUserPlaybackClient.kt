package dev.taru.android.userplayback

import dev.taru.android.browse.PageRequest
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import java.io.IOException
import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import javax.net.ssl.SSLException
import kotlinx.serialization.SerializationException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

class TaruUserPlaybackClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = false
    },
) {
    suspend fun getState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): UserPlaybackResult<UserPlaybackStateResponse> {
        if (itemId.isBlank()) {
            return failure(
                category = UserPlaybackFailureCategory.MissingItem,
                userMessage = "Choose a Media Item before requesting resume state.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/users/me/playback-state/items/${encodePathSegment(itemId)}",
        )
    }

    suspend fun continueWatching(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(limit = 12, offset = 0),
    ): UserPlaybackResult<ContinueWatchingResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/users/me/playback-state/continue-watching${pageQuery(page)}",
        )

    suspend fun updateProgress(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        request: UpdatePlaybackProgressRequest,
    ): UserPlaybackResult<UserPlaybackStateResponse> {
        if (itemId.isBlank()) {
            return failure(
                category = UserPlaybackFailureCategory.MissingItem,
                userMessage = "Choose a Media Item before reporting playback progress.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            method = "PUT",
            pathAndQuery = "/users/me/playback-state/items/${encodePathSegment(itemId)}/progress",
            body = json.encodeToString(request),
        )
    }

    suspend fun setWatchedState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        request: SetWatchedStateRequest,
    ): UserPlaybackResult<UserPlaybackStateResponse> {
        if (itemId.isBlank()) {
            return failure(
                category = UserPlaybackFailureCategory.MissingItem,
                userMessage = "Choose a Media Item before changing watched state.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            method = "PUT",
            pathAndQuery = "/users/me/playback-state/items/${encodePathSegment(itemId)}/watched",
            body = json.encodeToString(request),
        )
    }

    private suspend inline fun <reified T> executeJson(
        profile: ServerProfile,
        accessToken: String,
        method: String = "GET",
        pathAndQuery: String,
        body: String? = null,
    ): UserPlaybackResult<T> {
        if (accessToken.isBlank()) {
            return failure(
                category = UserPlaybackFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before requesting User Playback State.",
            )
        }

        val request = authenticatedRequest(
            profile = profile,
            accessToken = accessToken,
            method = method,
            pathAndQuery = pathAndQuery,
            body = body,
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
                category = UserPlaybackFailureCategory.UnsupportedApiVersion,
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

        return UserPlaybackResult.Success(
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
        } catch (_: SSLException) {
            TransportResult.Failure(
                failure(
                    category = UserPlaybackFailureCategory.TlsOrCertificate,
                    userMessage = "The server TLS certificate could not be trusted.",
                    request = safeRequest(request),
                ),
            )
        } catch (error: IOException) {
            TransportResult.Failure(
                failure(
                    category = UserPlaybackFailureCategory.UnreachableServer,
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
    ): UserPlaybackResult.Failure {
        val category = when (response.statusCode) {
            401 -> UserPlaybackFailureCategory.Unauthorized
            403 -> UserPlaybackFailureCategory.Forbidden
            404 -> UserPlaybackFailureCategory.MissingItem
            409 -> UserPlaybackFailureCategory.Conflict
            else -> UserPlaybackFailureCategory.PublicApiError
        }
        val userMessage = when (category) {
            UserPlaybackFailureCategory.MissingItem ->
                "The requested Media Item is no longer available."
            UserPlaybackFailureCategory.Unauthorized ->
                "The access token is invalid or expired."
            UserPlaybackFailureCategory.Forbidden ->
                "This access token cannot use User Playback State for the requested item."
            UserPlaybackFailureCategory.Conflict ->
                "The server could not apply this User Playback State change yet."
            else ->
                "The server returned a User Playback State API error."
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

    private fun authenticatedRequest(
        profile: ServerProfile,
        accessToken: String,
        method: String,
        pathAndQuery: String,
        body: String?,
    ): TaruHttpRequest =
        TaruHttpRequest(
            method = method,
            url = joinUrl(profile.baseUrl, pathAndQuery),
            headers = buildMap {
                put("Authorization", "Bearer $accessToken")
                if (body != null) {
                    put("Content-Type", "application/json")
                }
            },
            body = body,
        )

    private fun invalidResponseFailure(request: TaruHttpRequest): UserPlaybackResult.Failure =
        failure(
            category = UserPlaybackFailureCategory.InvalidResponse,
            userMessage = "The User Playback State response could not be understood.",
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
            url = SensitiveText.sanitize(request.url),
            headers = request.headers.mapValues { (name, value) ->
                if (name.equals("Authorization", ignoreCase = true)) {
                    "Bearer ${TaruPublicApiContract.redacted}"
                } else {
                    SensitiveText.sanitize(value)
                }
            },
        )

    private fun failure(
        category: UserPlaybackFailureCategory,
        userMessage: String,
        statusCode: Int? = null,
        observedApiVersion: String? = null,
        publicError: PublicErrorEnvelope? = null,
        request: SafeRequestPreview? = null,
    ): UserPlaybackResult.Failure =
        UserPlaybackResult.Failure(
            diagnostics = SafeUserPlaybackDiagnostics(
                category = category,
                userMessage = userMessage,
                statusCode = statusCode,
                observedApiVersion = observedApiVersion,
                publicError = publicError,
                request = request,
            ),
        )

    private fun pageQuery(page: PageRequest): String =
        queryString(
            listOf(
                "limit" to page.limit.toString(),
                "offset" to page.offset.toString(),
            ),
        )

    private fun queryString(pairs: List<Pair<String, String>>): String =
        pairs.joinToString(
            separator = "&",
            prefix = "?",
        ) { (name, value) ->
            "${encodeQueryValue(name)}=${encodeQueryValue(value)}"
        }

    private fun joinUrl(baseUrl: String, pathAndQuery: String): String =
        "${baseUrl.trimEnd('/')}$pathAndQuery"

    private fun encodeQueryValue(value: String): String =
        URLEncoder.encode(value, StandardCharsets.UTF_8)

    private fun encodePathSegment(value: String): String =
        URLEncoder
            .encode(value, StandardCharsets.UTF_8)
            .replace("+", "%20")

    private sealed interface TransportResult {
        data class Response(val response: TaruHttpResponse) : TransportResult
        data class Failure(val failure: UserPlaybackResult.Failure) : TransportResult
    }
}
