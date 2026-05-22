package dev.nako.android.connection

import dev.nako.sdk.NAKO_API_VERSION
import dev.nako.sdk.NAKO_API_VERSION_HEADER
import java.io.IOException
import javax.net.ssl.SSLException
import kotlinx.serialization.SerializationException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

internal enum class PublicApiFailureKind {
    MissingAccessToken,
    UnreachableServer,
    TlsOrCertificate,
    UnsupportedApiVersion,
    InvalidResponse,
    HttpError,
}

internal data class PublicApiFailure(
    val kind: PublicApiFailureKind,
    val statusCode: Int? = null,
    val observedApiVersion: String? = null,
    val publicError: PublicErrorEnvelope? = null,
    val request: SafeRequestPreview? = null,
)

internal sealed interface PublicApiResult<out T> {
    data class Success<T>(
        val value: T,
        val request: SafeRequestPreview,
        val response: NakoHttpResponse,
    ) : PublicApiResult<T>

    data class Failure(
        val failure: PublicApiFailure,
    ) : PublicApiResult<Nothing>
}

internal class PublicClientApiExecutor(
    private val transport: NakoHttpTransport,
    @PublishedApi internal val json: Json = Json { ignoreUnknownKeys = true },
) {
    suspend fun executeRequest(
        request: NakoHttpRequest,
        secrets: Iterable<String> = emptyList(),
        checkApiVersionHeader: Boolean = true,
    ): PublicApiResult<NakoHttpResponse> {
        val response = when (val result = executeTransport(request, secrets)) {
            is TransportResult.Failure -> return result.failure
            is TransportResult.Response -> result.response
        }
        val safeRequest = safeRequest(request, secrets)

        if (!response.isSuccessful()) {
            return PublicApiResult.Failure(
                PublicApiFailure(
                    kind = PublicApiFailureKind.HttpError,
                    statusCode = response.statusCode,
                    observedApiVersion = response.header(NAKO_API_VERSION_HEADER),
                    publicError = parsePublicError(response.body, secrets),
                    request = safeRequest,
                ),
            )
        }

        val observedApiVersion = response.header(NAKO_API_VERSION_HEADER)
        if (
            checkApiVersionHeader &&
            observedApiVersion != null &&
            observedApiVersion != NAKO_API_VERSION
        ) {
            return PublicApiResult.Failure(
                PublicApiFailure(
                    kind = PublicApiFailureKind.UnsupportedApiVersion,
                    observedApiVersion = observedApiVersion,
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

    fun safeRequest(
        request: NakoHttpRequest,
        secrets: Iterable<String> = emptyList(),
    ): SafeRequestPreview =
        SafeRequestPreview(
            method = request.method,
            url = SensitiveText.sanitize(request.url, secrets),
            headers = request.headers.mapValues { (name, value) ->
                if (name.equals("Authorization", ignoreCase = true)) {
                    "Bearer ${SensitiveText.redacted}"
                } else {
                    SensitiveText.sanitize(value, secrets)
                }
            },
        )

    private suspend fun executeTransport(
        request: NakoHttpRequest,
        secrets: Iterable<String>,
    ): TransportResult =
        try {
            TransportResult.Response(transport.execute(request))
        } catch (error: CleartextHttpNotPermittedException) {
            TransportResult.Failure(
                PublicApiResult.Failure(
                    PublicApiFailure(
                        kind = PublicApiFailureKind.UnreachableServer,
                        publicError = PublicErrorEnvelope(
                            code = "cleartext_http_not_allowed",
                            message = SensitiveText.sanitize(error.message.orEmpty(), secrets),
                        ),
                        request = safeRequest(request, secrets),
                    ),
                ),
            )
        } catch (_: SSLException) {
            TransportResult.Failure(
                PublicApiResult.Failure(
                    PublicApiFailure(
                        kind = PublicApiFailureKind.TlsOrCertificate,
                        request = safeRequest(request, secrets),
                    ),
                ),
            )
        } catch (error: IOException) {
            TransportResult.Failure(
                PublicApiResult.Failure(
                    PublicApiFailure(
                        kind = PublicApiFailureKind.UnreachableServer,
                        publicError = PublicErrorEnvelope(
                            code = "transport_error",
                            message = SensitiveText.sanitize(error.message.orEmpty(), secrets),
                        ),
                        request = safeRequest(request, secrets),
                    ),
                ),
            )
        }

    fun parsePublicError(
        body: String,
        secrets: Iterable<String>,
    ): PublicErrorEnvelope? =
        try {
            SensitiveText.sanitizeEnvelope(
                json.decodeFromString<PublicErrorEnvelope>(body),
                secrets,
            )
        } catch (_: SerializationException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }

    private sealed interface TransportResult {
        data class Response(val response: NakoHttpResponse) : TransportResult
        data class Failure(val failure: PublicApiResult.Failure) : TransportResult
    }
}
