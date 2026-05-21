package dev.taru.android.connection

import dev.taru.sdk.PageQuery
import dev.taru.sdk.TARU_API_VERSION
import dev.taru.sdk.TARU_API_VERSION_HEADER
import dev.taru.sdk.HealthResponse
import dev.taru.sdk.TaruPublicClientRequests
import java.net.URI
import kotlinx.serialization.SerializationException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

class TaruConnectionClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json { ignoreUnknownKeys = true },
    private val clockMillis: () -> Long = System::currentTimeMillis,
    private val securityPolicy: ConnectionSecurityPolicy = ConnectionSecurityPolicy.production(),
) {
    private val executor = PublicClientApiExecutor(transport, json)

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
        val uri = URI(normalizedBaseUrl)
        securityPolicy.cleartextFailure(uri)?.let { publicError ->
            return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.InsecureCleartextHttp,
                userMessage = "Use HTTPS for this server, or switch to a local-development build that explicitly allows HTTP.",
                publicError = publicError,
                request = SafeRequestPreview(
                    method = "GET",
                    url = SensitiveText.sanitize(normalizedBaseUrl),
                ),
            )
        }

        if (accessToken.isBlank()) {
            return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.MissingAccessToken,
                userMessage = "Enter a server access key.",
            )
        }

        val healthResult = when (
            val result = executor.executeResponse(
                baseUrl = normalizedBaseUrl,
                pathAndQuery = TaruPublicClientRequests.health().pathAndQuery,
                auth = PublicApiAuth.None,
                checkApiVersionHeader = true,
                extraSecrets = listOf(accessToken),
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(normalizedBaseUrl, result.failure)
            is PublicApiResult.Success -> result
        }
        val healthVersion = parseHealthVersion(healthResult.response.body)
            ?: return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.InvalidResponse,
                userMessage = userMessageFor(ConnectionFailureCategory.InvalidResponse),
                request = healthResult.request,
            )
        val observedHeaderVersion = healthResult.response.header(TARU_API_VERSION_HEADER)
        val observedVersion = when {
            healthVersion != TARU_API_VERSION -> healthVersion
            observedHeaderVersion != null -> observedHeaderVersion
            else -> healthVersion
        }
        if (healthVersion != TARU_API_VERSION ||
            observedHeaderVersion?.let { it != TARU_API_VERSION } == true
        ) {
            return failure(
                normalizedBaseUrl = normalizedBaseUrl,
                category = ConnectionFailureCategory.UnsupportedApiVersion,
                userMessage = "This server is not compatible with this Taru app version.",
                observedApiVersion = observedVersion,
                request = healthResult.request,
            )
        }

        val authProbeResult = when (
            val result = executor.executeResponse(
                baseUrl = normalizedBaseUrl,
                pathAndQuery = TaruPublicClientRequests
                    .listLibraries(PageQuery(limit = 1, offset = 0))
                    .pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(normalizedBaseUrl, result.failure)
            is PublicApiResult.Success -> result
        }

        return ConnectionCheckResult.Success(
            normalizedBaseUrl = normalizedBaseUrl,
            apiVersion = observedVersion,
            checkedAtMillis = clockMillis(),
            healthRequest = healthResult.request,
            authProbeRequest = authProbeResult.request,
        )
    }

    private fun failureFor(
        normalizedBaseUrl: String,
        failure: PublicApiFailure,
    ): ConnectionCheckResult.Failure {
        val category = when (failure.kind) {
            PublicApiFailureKind.MissingAccessToken -> ConnectionFailureCategory.MissingAccessToken
            PublicApiFailureKind.UnreachableServer -> ConnectionFailureCategory.UnreachableServer
            PublicApiFailureKind.TlsOrCertificate -> ConnectionFailureCategory.TlsOrCertificate
            PublicApiFailureKind.UnsupportedApiVersion -> ConnectionFailureCategory.UnsupportedApiVersion
            PublicApiFailureKind.InvalidResponse -> ConnectionFailureCategory.InvalidResponse
            PublicApiFailureKind.HttpError -> when (failure.statusCode) {
                401 -> ConnectionFailureCategory.Unauthorized
                else -> ConnectionFailureCategory.PublicApiError
            }
        }
        val resolvedCategory = if (failure.publicError?.code == "cleartext_http_not_allowed") {
            ConnectionFailureCategory.InsecureCleartextHttp
        } else {
            category
        }
        return failure(
            normalizedBaseUrl = normalizedBaseUrl,
            category = resolvedCategory,
            userMessage = userMessageFor(resolvedCategory),
            statusCode = failure.statusCode,
            observedApiVersion = failure.observedApiVersion,
            publicError = failure.publicError,
            request = failure.request,
        )
    }

    private fun userMessageFor(category: ConnectionFailureCategory): String =
        when (category) {
            ConnectionFailureCategory.InvalidUrl ->
                "Enter a valid HTTP or HTTPS Taru server URL."
            ConnectionFailureCategory.MissingAccessToken ->
                "Enter a server access key."
            ConnectionFailureCategory.UnreachableServer ->
                "The server could not be reached. Check the address and network."
            ConnectionFailureCategory.Unauthorized ->
                "The server access key is invalid or expired."
            ConnectionFailureCategory.UnsupportedApiVersion ->
                "This server is not compatible with this Taru app version."
            ConnectionFailureCategory.TlsOrCertificate ->
                "The server TLS certificate could not be trusted."
            ConnectionFailureCategory.InsecureCleartextHttp ->
                "Use HTTPS for this server, or switch to a local-development build that explicitly allows HTTP."
            ConnectionFailureCategory.PublicApiError ->
                "The server reported an issue."
            ConnectionFailureCategory.InvalidResponse ->
                "The server reply could not be understood."
        }

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

    private fun parseHealthVersion(body: String): String? =
        try {
            val health = json.decodeFromString<HealthResponse>(body)
            health.version.wireValue.takeIf {
                health.status.isNotBlank() && it.isNotBlank()
            }
        } catch (_: SerializationException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }
}
