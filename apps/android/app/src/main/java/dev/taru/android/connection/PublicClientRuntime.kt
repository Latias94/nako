package dev.taru.android.connection

import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

/**
 * Android-owned runtime seam for Public Client API request execution.
 *
 * Route-family clients keep product semantics and model mapping. This runtime
 * owns the generic flow around access-key validation, request execution,
 * response decoding, safe request previews, and transport failure propagation.
 */
internal class PublicClientRuntime(
    @PublishedApi internal val executor: PublicClientApiExecutor,
) {
    constructor(
        transport: TaruHttpTransport,
        json: Json = Json { ignoreUnknownKeys = true },
    ) : this(PublicClientApiExecutor(transport, json))

    suspend inline fun <reified WireT, AppT> executeAuthenticatedJson(
        accessToken: String,
        crossinline buildRequest: (String) -> TaruHttpRequest,
        noinline transform: (WireT) -> AppT,
    ): PublicApiResult<AppT> {
        missingAccessTokenFailure(accessToken)?.let { return PublicApiResult.Failure(it) }
        val request = buildRequest(accessToken)
        return executeJson(
            request = request,
            secrets = listOf(accessToken),
            transform = transform,
        )
    }

    suspend fun executeAuthenticatedResponse(
        accessToken: String,
        buildRequest: (String) -> TaruHttpRequest,
        checkApiVersionHeader: Boolean = true,
    ): PublicApiResult<TaruHttpResponse> {
        missingAccessTokenFailure(accessToken)?.let { return PublicApiResult.Failure(it) }
        return executeResponse(
            request = buildRequest(accessToken),
            secrets = listOf(accessToken),
            checkApiVersionHeader = checkApiVersionHeader,
        )
    }

    suspend fun executeAuthenticatedResponse(
        accessToken: String,
        request: TaruHttpRequest,
        checkApiVersionHeader: Boolean = true,
    ): PublicApiResult<TaruHttpResponse> =
        executeAuthenticatedResponse(
            accessToken = accessToken,
            buildRequest = { request },
            checkApiVersionHeader = checkApiVersionHeader,
        )

    suspend fun executeCoreResponse(
        request: TaruHttpRequest,
        safeRequest: SafeRequestPreview,
        secrets: Iterable<String>,
    ): PublicApiResult<TaruHttpResponse> {
        val safeSecrets = secrets.toList()
        return when (
            val result = executor.executeRequest(
                request = request,
                secrets = safeSecrets,
                checkApiVersionHeader = false,
            )
        ) {
            is PublicApiResult.Success -> PublicApiResult.Success(
                value = result.value,
                request = safeRequest,
                response = result.response,
            )
            is PublicApiResult.Failure -> PublicApiResult.Failure(
                result.failure.copy(request = result.failure.request ?: safeRequest),
            )
        }
    }

    suspend inline fun <reified WireT, AppT> executeJson(
        request: TaruHttpRequest,
        secrets: Iterable<String>,
        checkApiVersionHeader: Boolean = true,
        crossinline transform: (WireT) -> AppT,
    ): PublicApiResult<AppT> =
        when (
            val result = executeResponse(
                request = request,
                secrets = secrets,
                checkApiVersionHeader = checkApiVersionHeader,
            )
        ) {
            is PublicApiResult.Failure -> result
            is PublicApiResult.Success -> {
                val decoded = runCatching {
                    executor.json.decodeFromString<WireT>(result.response.body)
                }.getOrElse {
                    return invalidResponseFailure(result.request)
                }
                PublicApiResult.Success(
                    value = transform(decoded),
                    request = result.request,
                    response = result.response,
                )
            }
        }

    suspend fun executeResponse(
        request: TaruHttpRequest,
        secrets: Iterable<String>,
        checkApiVersionHeader: Boolean = true,
    ): PublicApiResult<TaruHttpResponse> =
        executor.executeRequest(
            request = request,
            secrets = secrets,
            checkApiVersionHeader = checkApiVersionHeader,
        )

    fun missingAccessTokenFailure(accessToken: String): PublicApiFailure? =
        if (accessToken.isBlank()) {
            PublicApiFailure(kind = PublicApiFailureKind.MissingAccessToken)
        } else {
            null
        }

    @PublishedApi
    internal fun invalidResponseFailure(request: SafeRequestPreview): PublicApiResult.Failure =
        PublicApiResult.Failure(
            PublicApiFailure(
                kind = PublicApiFailureKind.InvalidResponse,
                request = request,
            ),
        )
}
