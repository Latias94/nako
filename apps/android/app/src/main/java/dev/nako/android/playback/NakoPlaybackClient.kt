package dev.nako.android.playback

import dev.nako.android.connection.PublicApiFailure
import dev.nako.android.connection.PublicApiFailureKind
import dev.nako.android.connection.PublicApiResult
import dev.nako.android.connection.PublicClientRuntime
import dev.nako.android.connection.PublicErrorEnvelope
import dev.nako.android.connection.SafeRequestPreview
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.connection.NakoHttpTransport
import dev.nako.android.media.SourceProbeResponse
import dev.nako.android.media.toAndroid
import dev.nako.sdk.PlaybackDecisionResponse as SdkPlaybackDecisionResponse
import dev.nako.sdk.SourceProbeResponse as SdkSourceProbeResponse
import dev.nako.sdk.NAKO_PLAYBACK_SESSION_ID_HEADER
import dev.nako.sdk.TranscodeSessionResponse as SdkTranscodeSessionResponse
import kotlinx.serialization.json.Json

class NakoPlaybackClient(
    transport: NakoHttpTransport,
    json: Json = Json { ignoreUnknownKeys = true },
    private val playbackCore: PlaybackCore = RustPlaybackCore,
) {
    private val runtime = PublicClientRuntime(transport, json)

    suspend fun getSourceProbe(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
    ): PlaybackResult<SourceProbeResponse> {
        if (sourceId.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingSource,
                userMessage = "Choose a version before loading details.",
            )
        }

        return executeSdkJson<SdkSourceProbeResponse, SourceProbeResponse>(
            accessToken = accessToken,
            buildRequest = { token ->
                playbackCore
                    .sourceProbeRequest(profile, token, sourceId)
                    .authenticatedRequest(token)
            },
            transform = SdkSourceProbeResponse::toAndroid,
        )
    }

    suspend fun getPlaybackDecision(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackResult<PlaybackDecisionResponse> {
        if (sourceId.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingSource,
                userMessage = "Choose a version before requesting playback.",
            )
        }
        return executeSdkJson<SdkPlaybackDecisionResponse, PlaybackDecisionResponse>(
            accessToken = accessToken,
            buildRequest = { token ->
                playbackCore
                    .playbackDecisionRequest(
                        profile = profile,
                        accessToken = token,
                        sourceId = sourceId,
                        capabilities = capabilities,
                    )
                    .authenticatedRequest(token)
            },
            transform = SdkPlaybackDecisionResponse::toAndroid,
        )
    }

    fun directPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackCore
            .directPlaybackTarget(
                profile = profile,
                sourceId = sourceId,
            )
            .withRange(range)

    fun headDirectPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackCore
            .headDirectPlaybackTarget(
                profile = profile,
                sourceId = sourceId,
            )
            .withRange(range)

    fun remuxPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
        outputContainer: ClientOutputContainer = ClientOutputContainer.Mp4,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackCore
            .remuxPlaybackTarget(
                profile = profile,
                sourceId = sourceId,
                capabilities = capabilities,
                outputContainer = outputContainer,
            )
            .withRange(range)

    fun hlsPlaylistTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackRequestTarget =
        playbackCore.hlsPlaylistTarget(
            profile = profile,
            sourceId = sourceId,
            capabilities = capabilities,
        )

    fun hlsSegmentTarget(
        profile: ServerProfile,
        sessionId: String,
        segmentName: String,
    ): PlaybackRequestTarget =
        playbackCore.hlsSegmentTarget(
            profile = profile,
            sessionId = sessionId,
            segmentName = segmentName,
        )

    suspend fun getPlaybackSession(
        profile: ServerProfile,
        accessToken: String,
        sessionId: String,
    ): PlaybackResult<TranscodeSessionResponse> {
        if (sessionId.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingSession,
                userMessage = "Choose an active playback session before requesting status.",
            )
        }

        return executeSdkJson<SdkTranscodeSessionResponse, TranscodeSessionResponse>(
            accessToken = accessToken,
            buildRequest = { token ->
                playbackCore
                    .playbackSessionRequest(profile, token, sessionId)
                    .authenticatedRequest(token)
            },
            transform = SdkTranscodeSessionResponse::toAndroid,
        )
    }

    suspend fun cancelPlaybackSession(
        profile: ServerProfile,
        accessToken: String,
        sessionId: String,
    ): PlaybackResult<TranscodeSessionResponse> {
        if (sessionId.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingSession,
                userMessage = "Choose an active playback session before requesting cancellation.",
            )
        }

        return executeSdkJson<SdkTranscodeSessionResponse, TranscodeSessionResponse>(
            accessToken = accessToken,
            buildRequest = { token ->
                playbackCore
                    .cancelPlaybackSessionRequest(profile, token, sessionId)
                    .authenticatedRequest(token)
            },
            transform = SdkTranscodeSessionResponse::toAndroid,
        )
    }

    fun recommendedPlaybackTarget(
        profile: ServerProfile,
        decision: PlaybackDecisionResponse,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackRequestTarget? {
        return playbackCore.recommendedPlaybackTarget(
            profile = profile,
            decision = decision,
            capabilities = capabilities,
        )
    }

    suspend fun prepareRecommendedPlaybackTarget(
        profile: ServerProfile,
        accessToken: String,
        decision: PlaybackDecisionResponse,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackResult<PlaybackRequestTarget> {
        if (accessToken.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingAccessToken,
                userMessage = "Sign in again before requesting playback.",
            )
        }

        if (decision.decision.mode == ClientPlaybackMode.Denied) {
            return failure(
                category = PlaybackFailureCategory.Forbidden,
                userMessage = playbackDeniedMessage(decision.decision.denial),
            )
        }

        val target = recommendedPlaybackTarget(
            profile = profile,
            decision = decision,
            capabilities = capabilities,
        ) ?: return failure(
            category = PlaybackFailureCategory.UnsupportedSource,
            userMessage = "The server did not return a playable path for this version.",
        )

        return when (decision.decision.mode) {
            ClientPlaybackMode.DirectPlay -> PlaybackResult.Success(target, target.safeRequest)
            ClientPlaybackMode.Remux,
            ClientPlaybackMode.Transcode,
            -> prepareSessionBackedTarget(
                target = target,
                accessToken = accessToken,
            )
            ClientPlaybackMode.Denied -> failure(
                category = PlaybackFailureCategory.Forbidden,
                userMessage = playbackDeniedMessage(decision.decision.denial),
                request = target.safeRequest,
            )
            ClientPlaybackMode.Unknown -> failure(
                category = PlaybackFailureCategory.UnsupportedSource,
                userMessage = "The server returned a playback mode this app does not understand.",
                request = target.safeRequest,
            )
        }
    }

    private fun playbackDeniedMessage(denial: ClientPlaybackDenial?): String =
        when (denial?.reason) {
            "library_access_does_not_allow_play" -> "Your library access does not allow playback."
            "media_playback_disabled" -> "Playback is disabled for this profile."
            "direct_play_disabled" -> "Direct playback is disabled for this profile."
            "remux_disabled" -> "Remux playback is disabled for this profile."
            "audio_transcode_disabled" -> "Audio transcoding is disabled for this profile."
            "video_transcode_disabled" -> "Video transcoding is disabled for this profile."
            else -> "This profile is not allowed to start playback."
        }

    private suspend inline fun <reified WireT, AppT> executeSdkJson(
        accessToken: String,
        crossinline buildRequest: (String) -> NakoHttpRequest,
        noinline transform: (WireT) -> AppT,
    ): PlaybackResult<AppT> =
        when (
            val result = runtime.executeAuthenticatedJson(
                accessToken = accessToken,
                buildRequest = buildRequest,
                transform = transform,
            )
        ) {
            is PublicApiResult.Success -> PlaybackResult.Success(
                value = result.value,
                request = result.request,
            )
            is PublicApiResult.Failure -> failureFor(result.failure)
        }

    private fun failureFor(failure: PublicApiFailure): PlaybackResult.Failure {
        val category = when (failure.kind) {
            PublicApiFailureKind.MissingAccessToken -> PlaybackFailureCategory.MissingAccessToken
            PublicApiFailureKind.UnreachableServer -> PlaybackFailureCategory.UnreachableServer
            PublicApiFailureKind.TlsOrCertificate -> PlaybackFailureCategory.TlsOrCertificate
            PublicApiFailureKind.UnsupportedApiVersion -> PlaybackFailureCategory.UnsupportedApiVersion
            PublicApiFailureKind.InvalidResponse -> PlaybackFailureCategory.InvalidResponse
            PublicApiFailureKind.HttpError -> when (failure.statusCode) {
                400 -> PlaybackFailureCategory.UnsupportedSource
                401 -> PlaybackFailureCategory.Unauthorized
                403 -> PlaybackFailureCategory.Forbidden
                404 -> PlaybackFailureCategory.SourceUnavailable
                409 -> PlaybackFailureCategory.SessionConflict
                else -> PlaybackFailureCategory.PublicApiError
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

    private fun userMessageFor(category: PlaybackFailureCategory): String =
        when (category) {
            PlaybackFailureCategory.MissingSource ->
                "Choose a version before requesting playback."
            PlaybackFailureCategory.MissingSession ->
                "Choose an active playback session before requesting status."
            PlaybackFailureCategory.MissingAccessToken ->
                "Sign in again before requesting playback."
            PlaybackFailureCategory.UnreachableServer ->
                "The server could not be reached. Check the address and network."
            PlaybackFailureCategory.Unauthorized ->
                "The server access key is invalid or expired."
            PlaybackFailureCategory.Forbidden ->
                "This profile cannot play the requested version."
            PlaybackFailureCategory.UnsupportedApiVersion ->
                "This server is not compatible with this Nako app version."
            PlaybackFailureCategory.TlsOrCertificate ->
                "The server TLS certificate could not be trusted."
            PlaybackFailureCategory.UnsupportedSource ->
                "This version cannot be played with the current device capabilities."
            PlaybackFailureCategory.SourceUnavailable ->
                "The selected version is no longer available."
            PlaybackFailureCategory.SessionConflict ->
                "A matching playback session is already running or cannot be changed yet."
            PlaybackFailureCategory.InvalidResponse ->
                "The playback reply could not be understood."
            PlaybackFailureCategory.PublicApiError ->
                "The server reported a playback issue."
        }

    private fun PlaybackRequestTarget.withRange(range: String?): PlaybackRequestTarget =
        if (range.isNullOrBlank()) {
            this
        } else {
            copy(
                request = request.copy(headers = request.headers + ("Range" to range)),
            )
        }

    private suspend fun prepareSessionBackedTarget(
        target: PlaybackRequestTarget,
        accessToken: String,
    ): PlaybackResult<PlaybackRequestTarget> {
        if (!target.sessionId.isNullOrBlank()) {
            return PlaybackResult.Success(target, target.safeRequest)
        }

        val preflightRequest = target.sessionProbeRequest ?: return failure(
            category = PlaybackFailureCategory.MissingSession,
            userMessage = "The prepared playback path cannot start a tracked session.",
            request = target.safeRequest,
        )
        val response = when (
            val result = runtime.executeAuthenticatedResponse(
                accessToken = accessToken,
                request = preflightRequest.authenticatedRequest(accessToken),
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(result.failure)
            is PublicApiResult.Success -> result.response
        }

        val sessionId = response
            .header(NAKO_PLAYBACK_SESSION_ID_HEADER)
            ?.takeIf { it.isNotBlank() }
            ?: return failure(
                category = PlaybackFailureCategory.MissingSession,
                userMessage = "The server did not start a playback session for this path.",
                request = preflightRequest.safeRequest,
            )

        return PlaybackResult.Success(
            value = target.copy(sessionId = sessionId),
            request = target.safeRequest,
        )
    }

    private fun failure(
        category: PlaybackFailureCategory,
        userMessage: String,
        statusCode: Int? = null,
        observedApiVersion: String? = null,
        publicError: PublicErrorEnvelope? = null,
        request: SafeRequestPreview? = null,
    ): PlaybackResult.Failure =
        PlaybackResult.Failure(
            diagnostics = SafePlaybackDiagnostics(
                category = category,
                userMessage = userMessage,
                statusCode = statusCode,
                observedApiVersion = observedApiVersion,
                publicError = publicError,
                request = request,
            ),
        )

}
