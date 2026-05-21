package dev.taru.android.connection

import uniffi.taru_client_uniffi.CoreConnectionProbeOutcome as UniFfiConnectionProbeOutcome
import uniffi.taru_client_uniffi.CoreConnectionProbeOutcomeKind as UniFfiConnectionProbeOutcomeKind
import uniffi.taru_client_uniffi.CoreHttpHeader
import uniffi.taru_client_uniffi.CoreHttpRequest
import uniffi.taru_client_uniffi.CoreHttpResponse
import uniffi.taru_client_uniffi.CoreRuntimeFailure
import uniffi.taru_client_uniffi.CoreRuntimeFailureKind
import uniffi.taru_client_uniffi.CoreSafeRequestPreview

internal data class ConnectionCoreRequest(
    val requestId: String,
    val httpRequest: TaruHttpRequest,
    val safePreview: SafeRequestPreview,
)

internal data class ConnectionCoreSuccess(
    val apiVersion: String,
    val healthRequest: SafeRequestPreview,
    val authProbeRequest: SafeRequestPreview,
)

internal sealed interface ConnectionCoreOutcome {
    data class NextRequest(
        val request: ConnectionCoreRequest,
    ) : ConnectionCoreOutcome

    data class Success(
        val success: ConnectionCoreSuccess,
    ) : ConnectionCoreOutcome

    data class Failure(
        val failure: PublicApiFailure,
    ) : ConnectionCoreOutcome
}

internal interface ConnectionCore {
    fun startConnectionProbe(
        baseUrl: String,
        accessToken: String,
    ): ConnectionCoreOutcome

    fun advanceConnectionProbe(
        baseUrl: String,
        accessToken: String,
        request: ConnectionCoreRequest,
        response: TaruHttpResponse,
    ): ConnectionCoreOutcome
}

internal object RustConnectionCore : ConnectionCore {
    override fun startConnectionProbe(
        baseUrl: String,
        accessToken: String,
    ): ConnectionCoreOutcome =
        uniffi.taru_client_uniffi.startConnectionProbe(
            baseUrl = baseUrl,
            accessToken = accessToken,
        ).toConnectionCoreOutcome()

    override fun advanceConnectionProbe(
        baseUrl: String,
        accessToken: String,
        request: ConnectionCoreRequest,
        response: TaruHttpResponse,
    ): ConnectionCoreOutcome =
        uniffi.taru_client_uniffi.advanceConnectionProbe(
            baseUrl = baseUrl,
            accessToken = accessToken,
            response = response.toCoreResponse(request.requestId),
        ).toConnectionCoreOutcome()
}

internal fun CoreHttpRequest.toAndroidRequest(): TaruHttpRequest =
    TaruHttpRequest(
        method = method,
        url = url,
        headers = headers.toHeaderMap(),
        body = bodyUtf8,
    )

private fun CoreHttpRequest.toConnectionCoreRequest(): ConnectionCoreRequest =
    ConnectionCoreRequest(
        requestId = requestId,
        httpRequest = toAndroidRequest(),
        safePreview = safePreview.toAndroidPreview(),
    )

private fun TaruHttpResponse.toCoreResponse(requestId: String): CoreHttpResponse =
    CoreHttpResponse(
        requestId = requestId,
        statusCode = statusCode,
        headers = headers.flatMap { (name, values) ->
            if (values.isEmpty()) {
                listOf(CoreHttpHeader(name = name, value = ""))
            } else {
                values.map { value -> CoreHttpHeader(name = name, value = value) }
            }
        },
        bodyUtf8 = body,
    )

internal fun CoreSafeRequestPreview.toAndroidPreview(): SafeRequestPreview =
    SafeRequestPreview(
        method = method,
        url = SensitiveText.sanitize(url),
        headers = headers.toHeaderMap().mapValues { (_, value) ->
            SensitiveText.sanitize(value)
        },
    )

internal fun CoreRuntimeFailure.toPublicApiFailure(): PublicApiFailure =
    PublicApiFailure(
        kind = kind.toPublicApiFailureKind(),
        statusCode = statusCode,
        observedApiVersion = observedApiVersion,
        publicError = publicError?.let { error ->
            PublicErrorEnvelope(
                code = SensitiveText.sanitize(error.code),
                message = SensitiveText.sanitize(error.message),
            )
        },
        request = request?.toAndroidPreview(),
    )

private fun UniFfiConnectionProbeOutcome.toConnectionCoreOutcome(): ConnectionCoreOutcome =
    when (kind) {
        UniFfiConnectionProbeOutcomeKind.NEXT_REQUEST -> nextRequest
            ?.toConnectionCoreRequest()
            ?.let(ConnectionCoreOutcome::NextRequest)
            ?: invalidCoreOutcome()

        UniFfiConnectionProbeOutcomeKind.SUCCESS -> success
            ?.let { coreSuccess ->
                ConnectionCoreOutcome.Success(
                    ConnectionCoreSuccess(
                        apiVersion = coreSuccess.apiVersion,
                        healthRequest = coreSuccess.healthRequest.toAndroidPreview(),
                        authProbeRequest = coreSuccess.authProbeRequest.toAndroidPreview(),
                    ),
                )
            }
            ?: invalidCoreOutcome()

        UniFfiConnectionProbeOutcomeKind.FAILURE ->
            ConnectionCoreOutcome.Failure(
                failure?.toPublicApiFailure()
                    ?: PublicApiFailure(PublicApiFailureKind.InvalidResponse),
            )
    }

private fun invalidCoreOutcome(): ConnectionCoreOutcome =
    ConnectionCoreOutcome.Failure(PublicApiFailure(PublicApiFailureKind.InvalidResponse))

private fun CoreRuntimeFailureKind.toPublicApiFailureKind(): PublicApiFailureKind =
    when (this) {
        CoreRuntimeFailureKind.MISSING_ACCESS_TOKEN -> PublicApiFailureKind.MissingAccessToken
        CoreRuntimeFailureKind.UNSUPPORTED_API_VERSION -> PublicApiFailureKind.UnsupportedApiVersion
        CoreRuntimeFailureKind.INVALID_RESPONSE -> PublicApiFailureKind.InvalidResponse
        CoreRuntimeFailureKind.HTTP_ERROR -> PublicApiFailureKind.HttpError
    }

private fun List<CoreHttpHeader>.toHeaderMap(): Map<String, String> =
    associate { header -> header.name to header.value }
