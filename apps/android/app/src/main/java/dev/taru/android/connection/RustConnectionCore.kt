package dev.taru.android.connection

import uniffi.taru_client_uniffi.CoreConnectionProbeOutcome
import uniffi.taru_client_uniffi.CoreHttpHeader
import uniffi.taru_client_uniffi.CoreHttpRequest
import uniffi.taru_client_uniffi.CoreHttpResponse
import uniffi.taru_client_uniffi.CoreRuntimeFailure
import uniffi.taru_client_uniffi.CoreRuntimeFailureKind
import uniffi.taru_client_uniffi.CoreSafeRequestPreview

interface ConnectionCore {
    fun startConnectionProbe(
        baseUrl: String,
        accessToken: String,
    ): CoreConnectionProbeOutcome

    fun advanceConnectionProbe(
        baseUrl: String,
        accessToken: String,
        response: CoreHttpResponse,
    ): CoreConnectionProbeOutcome
}

object RustConnectionCore : ConnectionCore {
    override fun startConnectionProbe(
        baseUrl: String,
        accessToken: String,
    ): CoreConnectionProbeOutcome =
        uniffi.taru_client_uniffi.startConnectionProbe(
            baseUrl = baseUrl,
            accessToken = accessToken,
        )

    override fun advanceConnectionProbe(
        baseUrl: String,
        accessToken: String,
        response: CoreHttpResponse,
    ): CoreConnectionProbeOutcome =
        uniffi.taru_client_uniffi.advanceConnectionProbe(
            baseUrl = baseUrl,
            accessToken = accessToken,
            response = response,
        )
}

internal fun CoreHttpRequest.toAndroidRequest(): TaruHttpRequest =
    TaruHttpRequest(
        method = method,
        url = url,
        headers = headers.toHeaderMap(),
        body = bodyUtf8,
    )

internal fun TaruHttpResponse.toCoreResponse(requestId: String): CoreHttpResponse =
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

private fun CoreRuntimeFailureKind.toPublicApiFailureKind(): PublicApiFailureKind =
    when (this) {
        CoreRuntimeFailureKind.MISSING_ACCESS_TOKEN -> PublicApiFailureKind.MissingAccessToken
        CoreRuntimeFailureKind.UNSUPPORTED_API_VERSION -> PublicApiFailureKind.UnsupportedApiVersion
        CoreRuntimeFailureKind.INVALID_RESPONSE -> PublicApiFailureKind.InvalidResponse
        CoreRuntimeFailureKind.HTTP_ERROR -> PublicApiFailureKind.HttpError
    }

private fun List<CoreHttpHeader>.toHeaderMap(): Map<String, String> =
    associate { header -> header.name to header.value }
