package dev.nako.android.connection

import java.net.URI

class NakoConnectionClient private constructor(
    transport: NakoHttpTransport,
    private val clockMillis: () -> Long = System::currentTimeMillis,
    private val securityPolicy: ConnectionSecurityPolicy = ConnectionSecurityPolicy.production(),
    private val connectionCore: ConnectionCore,
) {
    private val runtime = PublicClientRuntime(transport)

    constructor(
        transport: NakoHttpTransport,
        clockMillis: () -> Long = System::currentTimeMillis,
        securityPolicy: ConnectionSecurityPolicy = ConnectionSecurityPolicy.production(),
    ) : this(
        transport = transport,
        clockMillis = clockMillis,
        securityPolicy = securityPolicy,
        connectionCore = RustConnectionCore,
    )

    suspend fun testConnection(
        baseUrlInput: String,
        accessToken: String,
    ): ConnectionCheckResult {
        val normalizedBaseUrl = normalizeBaseUrl(baseUrlInput)
            ?: return failure(
                normalizedBaseUrl = null,
                category = ConnectionFailureCategory.InvalidUrl,
                userMessage = "Enter a valid HTTP or HTTPS Nako server URL.",
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
        val healthRequest = when (startOutcome) {
            is ConnectionCoreOutcome.Failure -> return failureFor(
                normalizedBaseUrl,
                startOutcome.failure,
            )
            is ConnectionCoreOutcome.NextRequest -> startOutcome.request
            is ConnectionCoreOutcome.Success -> return invalidCoreOutcomeFailure(normalizedBaseUrl)
        }
        val healthResponse = when (
            val result = executeCoreRequest(
                request = healthRequest,
                accessToken = accessToken,
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(normalizedBaseUrl, result.failure)
            is PublicApiResult.Success -> result.response
        }

        val healthOutcome = connectionCore.advanceConnectionProbe(
            baseUrl = normalizedBaseUrl,
            accessToken = accessToken,
            request = healthRequest,
            response = healthResponse,
        )
        val authProbeRequest = when (healthOutcome) {
            is ConnectionCoreOutcome.Failure -> return failureFor(
                normalizedBaseUrl,
                healthOutcome.failure,
            )
            is ConnectionCoreOutcome.NextRequest -> healthOutcome.request
            is ConnectionCoreOutcome.Success -> return invalidCoreOutcomeFailure(normalizedBaseUrl)
        }
        val authProbeResponse = when (
            val result = executeCoreRequest(
                request = authProbeRequest,
                accessToken = accessToken,
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(normalizedBaseUrl, result.failure)
            is PublicApiResult.Success -> result.response
        }
        val authOutcome = connectionCore.advanceConnectionProbe(
            baseUrl = normalizedBaseUrl,
            accessToken = accessToken,
            request = authProbeRequest,
            response = authProbeResponse,
        )
        val success = when (authOutcome) {
            is ConnectionCoreOutcome.Failure -> return failureFor(
                normalizedBaseUrl,
                authOutcome.failure,
            )
            is ConnectionCoreOutcome.NextRequest -> return invalidCoreOutcomeFailure(normalizedBaseUrl)
            is ConnectionCoreOutcome.Success -> authOutcome.success
        }

        return ConnectionCheckResult.Success(
            normalizedBaseUrl = normalizedBaseUrl,
            apiVersion = success.apiVersion,
            checkedAtMillis = clockMillis(),
            healthRequest = success.healthRequest,
            authProbeRequest = success.authProbeRequest,
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
                "Enter a valid HTTP or HTTPS Nako server URL."
            ConnectionFailureCategory.MissingAccessToken ->
                "Enter a server access key."
            ConnectionFailureCategory.UnreachableServer ->
                "The server could not be reached. Check the address and network."
            ConnectionFailureCategory.Unauthorized ->
                "The server access key is invalid or expired."
            ConnectionFailureCategory.UnsupportedApiVersion ->
                "This server is not compatible with this Nako app version."
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
        request: ConnectionCoreRequest,
        accessToken: String,
    ): PublicApiResult<NakoHttpResponse> =
        runtime.executeCoreResponse(
            request = request.httpRequest,
            safeRequest = request.safePreview,
            secrets = listOf(accessToken),
        )

    private fun invalidCoreOutcomeFailure(normalizedBaseUrl: String): ConnectionCheckResult.Failure =
        failure(
            normalizedBaseUrl = normalizedBaseUrl,
            category = ConnectionFailureCategory.InvalidResponse,
            userMessage = userMessageFor(ConnectionFailureCategory.InvalidResponse),
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
}
