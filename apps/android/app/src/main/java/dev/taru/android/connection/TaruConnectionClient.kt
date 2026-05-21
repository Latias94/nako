package dev.taru.android.connection

import java.io.IOException
import java.net.URI
import javax.net.ssl.SSLException
import uniffi.taru_client_uniffi.CoreConnectionProbeOutcome
import uniffi.taru_client_uniffi.CoreConnectionProbeOutcomeKind
import uniffi.taru_client_uniffi.CoreHttpRequest

class TaruConnectionClient(
    private val transport: TaruHttpTransport,
    private val clockMillis: () -> Long = System::currentTimeMillis,
    private val securityPolicy: ConnectionSecurityPolicy = ConnectionSecurityPolicy.production(),
    private val connectionCore: ConnectionCore = RustConnectionCore,
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

        val startOutcome = connectionCore.startConnectionProbe(
            baseUrl = normalizedBaseUrl,
            accessToken = accessToken,
        )
        val healthRequest = when (startOutcome.kind) {
            CoreConnectionProbeOutcomeKind.FAILURE -> return failureFor(
                normalizedBaseUrl,
                failureFrom(startOutcome),
            )
            CoreConnectionProbeOutcomeKind.NEXT_REQUEST -> startOutcome.nextRequest
                ?: return invalidCoreOutcomeFailure(normalizedBaseUrl)
            CoreConnectionProbeOutcomeKind.SUCCESS -> return invalidCoreOutcomeFailure(normalizedBaseUrl)
        }
        val healthResponse = when (
            val result = executeCoreRequest(
                request = healthRequest,
                accessToken = accessToken,
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(normalizedBaseUrl, result.failure)
            is PublicApiResult.Success -> result.response.toCoreResponse(healthRequest.requestId)
        }

        val healthOutcome = connectionCore.advanceConnectionProbe(
            baseUrl = normalizedBaseUrl,
            accessToken = accessToken,
            response = healthResponse,
        )
        val authProbeRequest = when (healthOutcome.kind) {
            CoreConnectionProbeOutcomeKind.FAILURE -> return failureFor(
                normalizedBaseUrl,
                failureFrom(healthOutcome),
            )
            CoreConnectionProbeOutcomeKind.NEXT_REQUEST -> healthOutcome.nextRequest
                ?: return invalidCoreOutcomeFailure(normalizedBaseUrl)
            CoreConnectionProbeOutcomeKind.SUCCESS -> return invalidCoreOutcomeFailure(normalizedBaseUrl)
        }
        val authProbeResponse = when (
            val result = executeCoreRequest(
                request = authProbeRequest,
                accessToken = accessToken,
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(normalizedBaseUrl, result.failure)
            is PublicApiResult.Success -> result.response.toCoreResponse(authProbeRequest.requestId)
        }
        val authOutcome = connectionCore.advanceConnectionProbe(
            baseUrl = normalizedBaseUrl,
            accessToken = accessToken,
            response = authProbeResponse,
        )
        val success = when (authOutcome.kind) {
            CoreConnectionProbeOutcomeKind.FAILURE -> return failureFor(
                normalizedBaseUrl,
                failureFrom(authOutcome),
            )
            CoreConnectionProbeOutcomeKind.NEXT_REQUEST -> return invalidCoreOutcomeFailure(normalizedBaseUrl)
            CoreConnectionProbeOutcomeKind.SUCCESS -> authOutcome.success
                ?: return invalidCoreOutcomeFailure(normalizedBaseUrl)
        }

        return ConnectionCheckResult.Success(
            normalizedBaseUrl = normalizedBaseUrl,
            apiVersion = success.apiVersion,
            checkedAtMillis = clockMillis(),
            healthRequest = success.healthRequest.toAndroidPreview(),
            authProbeRequest = success.authProbeRequest.toAndroidPreview(),
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

    private suspend fun executeCoreRequest(
        request: CoreHttpRequest,
        accessToken: String,
    ): PublicApiResult<TaruHttpResponse> {
        val androidRequest = request.toAndroidRequest()
        val safeRequest = request.safePreview.toAndroidPreview()
        val response = try {
            transport.execute(androidRequest)
        } catch (error: CleartextHttpNotPermittedException) {
            return PublicApiResult.Failure(
                PublicApiFailure(
                    kind = PublicApiFailureKind.UnreachableServer,
                    publicError = PublicErrorEnvelope(
                        code = "cleartext_http_not_allowed",
                        message = SensitiveText.sanitize(
                            error.message.orEmpty(),
                            listOf(accessToken),
                        ),
                    ),
                    request = safeRequest,
                ),
            )
        } catch (_: SSLException) {
            return PublicApiResult.Failure(
                PublicApiFailure(
                    kind = PublicApiFailureKind.TlsOrCertificate,
                    request = safeRequest,
                ),
            )
        } catch (error: IOException) {
            return PublicApiResult.Failure(
                PublicApiFailure(
                    kind = PublicApiFailureKind.UnreachableServer,
                    publicError = PublicErrorEnvelope(
                        code = "transport_error",
                        message = SensitiveText.sanitize(
                            error.message.orEmpty(),
                            listOf(accessToken),
                        ),
                    ),
                    request = safeRequest,
                ),
            )
        }

        return PublicApiResult.Success(
            value = response,
            request = safeRequest,
            response = response,
        )
    }

    private fun invalidCoreOutcomeFailure(normalizedBaseUrl: String): ConnectionCheckResult.Failure =
        failure(
            normalizedBaseUrl = normalizedBaseUrl,
            category = ConnectionFailureCategory.InvalidResponse,
            userMessage = userMessageFor(ConnectionFailureCategory.InvalidResponse),
        )

    private fun failureFrom(outcome: CoreConnectionProbeOutcome): PublicApiFailure =
        outcome.failure?.toPublicApiFailure()
            ?: PublicApiFailure(PublicApiFailureKind.InvalidResponse)

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
}
