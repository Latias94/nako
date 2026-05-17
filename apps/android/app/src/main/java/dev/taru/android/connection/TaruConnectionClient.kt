package dev.taru.android.connection

import java.io.IOException
import java.net.URI
import javax.net.ssl.SSLException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json

class TaruConnectionClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json { ignoreUnknownKeys = true },
    private val clockMillis: () -> Long = System::currentTimeMillis,
) {
    suspend fun testConnection(
        baseUrlInput: String,
        accessToken: String,
    ): ConnectionCheckResult {
        val normalizedBaseUrl = normalizeBaseUrl(baseUrlInput)
            ?: return failure(
                normalizedBaseUrl = null,
                category = ConnectionFailureCategory.InvalidUrl,
                userMessage = "Enter a valid HTTP or HTTPS Taru server URL.",
            )

        if (accessToken.isBlank()) {
            return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.MissingAccessToken,
                userMessage = "Enter a server access token.",
            )
        }

        val healthRequest = TaruHttpRequest(
            method = "GET",
            url = joinUrl(normalizedBaseUrl, TaruPublicApiContract.healthPath),
        )
        val healthResponse = when (
            val result = executeOrFailure(normalizedBaseUrl, healthRequest, accessToken)
        ) {
            is TransportResult.Failure -> return result.failure
            is TransportResult.Response -> result.response
        }

        if (!healthResponse.isSuccessful()) {
            return httpFailure(normalizedBaseUrl, healthRequest, healthResponse, accessToken)
        }

        val health = decodeHealth(healthResponse)
            ?: return invalidResponseFailure(normalizedBaseUrl, healthRequest)
        val observedHeaderVersion = healthResponse.header(TaruPublicApiContract.apiVersionHeader)
        val observedVersion = when {
            health.version != TaruPublicApiContract.expectedApiVersion -> health.version
            observedHeaderVersion != null -> observedHeaderVersion
            else -> health.version
        }
        if (health.version != TaruPublicApiContract.expectedApiVersion ||
            observedHeaderVersion?.let { it != TaruPublicApiContract.expectedApiVersion } == true
        ) {
            return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.UnsupportedApiVersion,
                userMessage = "This server uses an unsupported Public Client API version.",
                observedApiVersion = observedVersion,
                request = safeRequest(healthRequest),
            )
        }

        val authProbeRequest = TaruHttpRequest(
            method = "GET",
            url = joinUrl(normalizedBaseUrl, TaruPublicApiContract.authProbePath),
            headers = mapOf("Authorization" to "Bearer $accessToken"),
        )
        val authProbeResponse = when (
            val result = executeOrFailure(normalizedBaseUrl, authProbeRequest, accessToken)
        ) {
            is TransportResult.Failure -> return result.failure
            is TransportResult.Response -> result.response
        }

        if (!authProbeResponse.isSuccessful()) {
            return httpFailure(normalizedBaseUrl, authProbeRequest, authProbeResponse, accessToken)
        }
        val authProbeVersion = authProbeResponse.header(TaruPublicApiContract.apiVersionHeader)
        if (authProbeVersion != null && authProbeVersion != TaruPublicApiContract.expectedApiVersion) {
            return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.UnsupportedApiVersion,
                userMessage = "This server uses an unsupported Public Client API version.",
                observedApiVersion = authProbeVersion,
                request = safeRequest(authProbeRequest),
            )
        }

        return ConnectionCheckResult.Success(
            normalizedBaseUrl = normalizedBaseUrl,
            apiVersion = observedVersion,
            checkedAtMillis = clockMillis(),
            healthRequest = safeRequest(healthRequest),
            authProbeRequest = safeRequest(authProbeRequest),
        )
    }

    private suspend fun executeOrFailure(
        normalizedBaseUrl: String,
        request: TaruHttpRequest,
        accessToken: String,
    ): TransportResult =
        try {
            TransportResult.Response(transport.execute(request))
        } catch (error: SSLException) {
            TransportResult.Failure(
                failure(
                    normalizedBaseUrl = normalizedBaseUrl,
                    category = ConnectionFailureCategory.TlsOrCertificate,
                    userMessage = "The server TLS certificate could not be trusted.",
                    request = safeRequest(request),
                ),
            )
        } catch (error: IOException) {
            TransportResult.Failure(
                failure(
                    normalizedBaseUrl = normalizedBaseUrl,
                    category = ConnectionFailureCategory.UnreachableServer,
                    userMessage = "The server could not be reached. Check the address and network.",
                    request = safeRequest(request),
                    publicError = PublicErrorEnvelope(
                        code = "transport_error",
                        message = SensitiveText.sanitize(error.message.orEmpty(), listOf(accessToken)),
                    ),
                ),
            )
        }

    private fun httpFailure(
        normalizedBaseUrl: String,
        request: TaruHttpRequest,
        response: TaruHttpResponse,
        accessToken: String,
    ): ConnectionCheckResult.Failure {
        val publicError = parsePublicError(response.body, accessToken)
        val category = when (response.statusCode) {
            401 -> ConnectionFailureCategory.Unauthorized
            else -> ConnectionFailureCategory.PublicApiError
        }
        val userMessage = when (category) {
            ConnectionFailureCategory.Unauthorized ->
                "The access token is invalid or expired."
            else ->
                "The server returned a public API error."
        }

        return failure(
            normalizedBaseUrl = normalizedBaseUrl,
            category = category,
            userMessage = userMessage,
            statusCode = response.statusCode,
            observedApiVersion = response.header(TaruPublicApiContract.apiVersionHeader),
            publicError = publicError,
            request = safeRequest(request),
        )
    }

    private fun decodeHealth(response: TaruHttpResponse): HealthEnvelope? =
        try {
            json.decodeFromString<HealthEnvelope>(response.body)
        } catch (_: SerializationException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }

    private fun invalidResponseFailure(
        normalizedBaseUrl: String,
        request: TaruHttpRequest,
    ): ConnectionCheckResult.Failure =
        failure(
            normalizedBaseUrl = normalizedBaseUrl,
            category = ConnectionFailureCategory.InvalidResponse,
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
        normalizedBaseUrl: String?,
        category: ConnectionFailureCategory,
        userMessage: String,
        statusCode: Int? = null,
        observedApiVersion: String? = null,
        publicError: PublicErrorEnvelope? = null,
        request: SafeRequestPreview? = null,
    ): ConnectionCheckResult.Failure =
        ConnectionCheckResult.Failure(
            normalizedBaseUrl = normalizedBaseUrl,
            diagnostics = SafeConnectionDiagnostics(
                category = category,
                userMessage = userMessage,
                statusCode = statusCode,
                observedApiVersion = observedApiVersion,
                publicError = publicError,
                request = request,
            ),
        )

    private fun normalizeBaseUrl(input: String): String? {
        val trimmed = input.trim().trimEnd('/')
        if (trimmed.isBlank()) {
            return null
        }

        val uri = runCatching { URI(trimmed) }.getOrNull() ?: return null
        val scheme = uri.scheme?.lowercase()
        if (scheme != "http" && scheme != "https") {
            return null
        }
        if (uri.host.isNullOrBlank()) {
            return null
        }
        return uri.toString().trimEnd('/')
    }

    private fun joinUrl(baseUrl: String, pathAndQuery: String): String = "$baseUrl$pathAndQuery"

    private sealed interface TransportResult {
        data class Response(val response: TaruHttpResponse) : TransportResult
        data class Failure(val failure: ConnectionCheckResult.Failure) : TransportResult
    }
}
