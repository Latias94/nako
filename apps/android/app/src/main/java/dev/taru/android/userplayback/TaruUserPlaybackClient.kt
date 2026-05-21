package dev.taru.android.userplayback

import dev.taru.android.browse.PageRequest
import dev.taru.android.connection.PublicApiAuth
import dev.taru.android.connection.PublicApiFailure
import dev.taru.android.connection.PublicApiFailureKind
import dev.taru.android.connection.PublicApiResult
import dev.taru.android.connection.PublicApiUrl
import dev.taru.android.connection.PublicClientApiExecutor
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpTransport
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

class TaruUserPlaybackClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = false
    },
) {
    private val executor = PublicClientApiExecutor(transport, json)

    suspend fun getState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): UserPlaybackResult<UserPlaybackStateResponse> {
        if (itemId.isBlank()) {
            return failure(
                category = UserPlaybackFailureCategory.MissingItem,
                userMessage = "Choose a title before loading watch progress.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/users/me/playback-state/items/${PublicApiUrl.encodePathSegment(itemId)}",
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
                userMessage = "Choose a title before reporting playback progress.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            method = "PUT",
            pathAndQuery = "/users/me/playback-state/items/${PublicApiUrl.encodePathSegment(itemId)}/progress",
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
                userMessage = "Choose a title before changing watched state.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            method = "PUT",
            pathAndQuery = "/users/me/playback-state/items/${PublicApiUrl.encodePathSegment(itemId)}/watched",
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
        return when (
            val result = executor.executeJson<T>(
                baseUrl = profile.baseUrl,
                pathAndQuery = pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
                method = method,
                body = body,
            )
        ) {
            is PublicApiResult.Success -> UserPlaybackResult.Success(
                value = result.value,
                request = result.request,
            )
            is PublicApiResult.Failure -> failureFor(result.failure)
        }
    }

    private fun failureFor(failure: PublicApiFailure): UserPlaybackResult.Failure {
        val category = when (failure.kind) {
            PublicApiFailureKind.MissingAccessToken -> UserPlaybackFailureCategory.MissingAccessToken
            PublicApiFailureKind.UnreachableServer -> UserPlaybackFailureCategory.UnreachableServer
            PublicApiFailureKind.TlsOrCertificate -> UserPlaybackFailureCategory.TlsOrCertificate
            PublicApiFailureKind.UnsupportedApiVersion -> UserPlaybackFailureCategory.UnsupportedApiVersion
            PublicApiFailureKind.InvalidResponse -> UserPlaybackFailureCategory.InvalidResponse
            PublicApiFailureKind.HttpError -> when (failure.statusCode) {
                401 -> UserPlaybackFailureCategory.Unauthorized
                403 -> UserPlaybackFailureCategory.Forbidden
                404 -> UserPlaybackFailureCategory.MissingItem
                409 -> UserPlaybackFailureCategory.Conflict
                else -> UserPlaybackFailureCategory.PublicApiError
            }
        }
        return failure(
            category = category,
            userMessage = userMessageFor(category),
            statusCode = failure.statusCode,
            observedApiVersion = failure.observedApiVersion,
            publicError = failure.publicError,
            request = failure.request,
        )
    }

    private fun userMessageFor(category: UserPlaybackFailureCategory): String =
        when (category) {
            UserPlaybackFailureCategory.MissingAccessToken ->
                "Sign in again before loading watch progress."
            UserPlaybackFailureCategory.UnreachableServer ->
                "The server could not be reached. Check the address and network."
            UserPlaybackFailureCategory.TlsOrCertificate ->
                "The server TLS certificate could not be trusted."
            UserPlaybackFailureCategory.UnsupportedApiVersion ->
                "This server is not compatible with this Taru app version."
            UserPlaybackFailureCategory.InvalidResponse ->
                "The watch-progress reply could not be understood."
            UserPlaybackFailureCategory.MissingItem ->
                "The requested title is no longer available."
            UserPlaybackFailureCategory.Unauthorized ->
                "The server access key is invalid or expired."
            UserPlaybackFailureCategory.Forbidden ->
                "This profile cannot update watch progress for the requested title."
            UserPlaybackFailureCategory.Conflict ->
                "The server could not apply this watch-progress change yet."
            UserPlaybackFailureCategory.PublicApiError ->
                "The server reported a watch-progress issue."
        }

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
        PublicApiUrl.pageQuery(limit = page.limit, offset = page.offset)
}
