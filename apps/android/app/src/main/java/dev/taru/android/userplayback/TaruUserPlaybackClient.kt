package dev.taru.android.userplayback

import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.toAndroid
import dev.taru.android.browse.toSdkPageQuery
import dev.taru.android.connection.PublicApiAuth
import dev.taru.android.connection.PublicApiFailure
import dev.taru.android.connection.PublicApiFailureKind
import dev.taru.android.connection.PublicApiResult
import dev.taru.android.connection.PublicClientApiExecutor
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.sdk.TaruPublicClientRequests
import dev.taru.sdk.TaruRequestDescriptor
import dev.taru.sdk.ContinueWatchingItemDto as SdkContinueWatchingItemDto
import dev.taru.sdk.ContinueWatchingResponse as SdkContinueWatchingResponse
import dev.taru.sdk.SetWatchedStateRequest as SdkSetWatchedStateRequest
import dev.taru.sdk.UpdatePlaybackProgressRequest as SdkUpdatePlaybackProgressRequest
import dev.taru.sdk.UserPlaybackStateDto as SdkUserPlaybackStateDto
import dev.taru.sdk.UserPlaybackStateResponse as SdkUserPlaybackStateResponse
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

        return executeSdkJson<SdkUserPlaybackStateResponse, UserPlaybackStateResponse>(
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.getUserPlaybackState(itemId),
            transform = SdkUserPlaybackStateResponse::toAndroid,
        )
    }

    suspend fun continueWatching(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(limit = 12, offset = 0),
    ): UserPlaybackResult<ContinueWatchingResponse> =
        executeSdkJson<SdkContinueWatchingResponse, ContinueWatchingResponse>(
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.listContinueWatching(page.toSdkPageQuery()),
            transform = SdkContinueWatchingResponse::toAndroid,
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

        return executeSdkJson<SdkUserPlaybackStateResponse, UserPlaybackStateResponse>(
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.updateUserPlaybackProgress(itemId),
            body = json.encodeToString(request.toSdk()),
            transform = SdkUserPlaybackStateResponse::toAndroid,
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

        return executeSdkJson<SdkUserPlaybackStateResponse, UserPlaybackStateResponse>(
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.setUserWatchedState(itemId),
            body = json.encodeToString(request.toSdk()),
            transform = SdkUserPlaybackStateResponse::toAndroid,
        )
    }

    private suspend inline fun <reified WireT, AppT> executeSdkJson(
        profile: ServerProfile,
        accessToken: String,
        descriptor: TaruRequestDescriptor,
        body: String? = null,
        transform: (WireT) -> AppT,
    ): UserPlaybackResult<AppT> {
        return when (
            val result = executor.executeJson<WireT>(
                baseUrl = profile.baseUrl,
                pathAndQuery = descriptor.pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
                method = descriptor.method,
                body = body,
            )
        ) {
            is PublicApiResult.Success -> UserPlaybackResult.Success(
                value = transform(result.value),
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

}

internal fun SdkUserPlaybackStateResponse.toAndroid(): UserPlaybackStateResponse =
    UserPlaybackStateResponse(state = state.toAndroid())

internal fun SdkContinueWatchingResponse.toAndroid(): ContinueWatchingResponse =
    ContinueWatchingResponse(
        items = items.map(SdkContinueWatchingItemDto::toAndroid),
        page = page.toAndroid(),
    )

private fun SdkContinueWatchingItemDto.toAndroid(): ContinueWatchingItemDto =
    ContinueWatchingItemDto(
        item = item.toAndroid(),
        state = state.toAndroid(),
        images = images.map { it.toAndroid() },
    )

private fun SdkUserPlaybackStateDto.toAndroid(): UserPlaybackStateDto =
    UserPlaybackStateDto(
        itemId = itemId,
        sourceId = sourceId,
        resumePositionMs = resumePositionMs,
        durationMs = durationMs,
        progressPercent = progressPercent,
        watched = watched,
        watchedAt = watchedAt,
        lastPlayedAt = lastPlayedAt,
        updatedAt = updatedAt,
        version = version,
    )

private fun UpdatePlaybackProgressRequest.toSdk(): SdkUpdatePlaybackProgressRequest =
    SdkUpdatePlaybackProgressRequest(
        sourceId = sourceId,
        positionMs = positionMs,
        durationMs = durationMs,
        reportedAt = reportedAt,
    )

private fun SetWatchedStateRequest.toSdk(): SdkSetWatchedStateRequest =
    SdkSetWatchedStateRequest(
        watched = watched,
        sourceId = sourceId,
        positionMs = positionMs,
        durationMs = durationMs,
        markedAt = markedAt,
    )
