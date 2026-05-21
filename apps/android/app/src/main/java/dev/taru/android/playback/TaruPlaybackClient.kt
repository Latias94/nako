package dev.taru.android.playback

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.PublicApiAuth
import dev.taru.android.connection.PublicApiFailure
import dev.taru.android.connection.PublicApiFailureKind
import dev.taru.android.connection.PublicApiResult
import dev.taru.android.connection.PublicClientApiExecutor
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.urlOn
import dev.taru.android.media.toAndroid
import dev.taru.android.media.SourceProbeResponse
import dev.taru.sdk.TARU_PLAYBACK_SESSION_ID_HEADER
import dev.taru.sdk.PlaybackCapabilitiesQuery
import dev.taru.sdk.RemuxOutputContainer
import dev.taru.sdk.RemuxPlaybackQuery
import dev.taru.sdk.TaruPublicClientRequests
import dev.taru.sdk.TaruRequestDescriptor
import dev.taru.sdk.PlaybackDecisionResponse as SdkPlaybackDecisionResponse
import dev.taru.sdk.SourceProbeResponse as SdkSourceProbeResponse
import dev.taru.sdk.TranscodeSessionResponse as SdkTranscodeSessionResponse
import kotlinx.serialization.json.Json

class TaruPlaybackClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    private val executor = PublicClientApiExecutor(transport, json)

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
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.getSourceProbe(sourceId),
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
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.getSourcePlaybackDecision(
                sourceId = sourceId,
                capabilities = capabilities.toSdkPlaybackCapabilitiesQuery(),
            ),
            transform = SdkPlaybackDecisionResponse::toAndroid,
        )
    }

    fun directPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            descriptor = TaruPublicClientRequests.streamSource(sourceId),
            range = range,
        )

    fun headDirectPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            descriptor = TaruPublicClientRequests.headStreamSource(sourceId),
            range = range,
        )

    fun remuxPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
        outputContainer: ClientOutputContainer = ClientOutputContainer.Mp4,
        range: String? = null,
    ): PlaybackRequestTarget {
        val query = capabilities.toSdkRemuxPlaybackQuery(outputContainer)
        return playbackTarget(
            profile = profile,
            descriptor = TaruPublicClientRequests.remuxStreamSource(sourceId, query),
            range = range,
            sessionProbeRequest = playbackRequestDescriptor(
                profile = profile,
                descriptor = TaruPublicClientRequests.headRemuxStreamSource(sourceId, query),
            ),
        )
    }

    fun hlsPlaylistTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackRequestTarget {
        val descriptor = TaruPublicClientRequests.hlsPlaylistSource(
            sourceId = sourceId,
            capabilities = capabilities.toSdkPlaybackCapabilitiesQuery(),
        )
        return playbackTarget(
            profile = profile,
            descriptor = descriptor,
            sessionProbeRequest = playbackRequestDescriptor(
                profile = profile,
                descriptor = descriptor,
            ),
        )
    }

    fun hlsSegmentTarget(
        profile: ServerProfile,
        sessionId: String,
        segmentName: String,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            descriptor = TaruPublicClientRequests.hlsSegment(
                sessionId = sessionId,
                segmentName = segmentName,
            ),
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
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.getPlaybackSession(sessionId),
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
            profile = profile,
            accessToken = accessToken,
            descriptor = TaruPublicClientRequests.cancelPlaybackSession(sessionId),
            transform = SdkTranscodeSessionResponse::toAndroid,
        )
    }

    fun recommendedPlaybackTarget(
        profile: ServerProfile,
        decision: PlaybackDecisionResponse,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackRequestTarget? {
        val target = when (decision.decision.mode) {
            ClientPlaybackMode.DirectPlay -> directPlaybackTarget(
                profile = profile,
                sourceId = decision.source.id,
            )
            ClientPlaybackMode.Remux -> remuxPlaybackTarget(
                profile = profile,
                sourceId = decision.source.id,
                capabilities = capabilities,
                outputContainer = remuxOutputContainer(decision),
            )
            ClientPlaybackMode.Transcode -> hlsPlaylistTarget(
                profile = profile,
                sourceId = decision.source.id,
                capabilities = capabilities,
            )
            ClientPlaybackMode.Unknown -> null
        }
        return target
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
            ClientPlaybackMode.Unknown -> failure(
                category = PlaybackFailureCategory.UnsupportedSource,
                userMessage = "The server returned a playback mode this app does not understand.",
                request = target.safeRequest,
            )
        }
    }

    private suspend inline fun <reified WireT, AppT> executeSdkJson(
        profile: ServerProfile,
        accessToken: String,
        descriptor: TaruRequestDescriptor,
        transform: (WireT) -> AppT,
    ): PlaybackResult<AppT> {
        return when (
            val result = executor.executeJson<WireT>(
                baseUrl = profile.baseUrl,
                pathAndQuery = descriptor.pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
                method = descriptor.method,
            )
        ) {
            is PublicApiResult.Success -> PlaybackResult.Success(
                value = transform(result.value),
                request = result.request,
            )
            is PublicApiResult.Failure -> failureFor(result.failure)
        }
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
                "This server is not compatible with this Taru app version."
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

    private fun playbackTarget(
        profile: ServerProfile,
        descriptor: TaruRequestDescriptor,
        range: String? = null,
        sessionProbeRequest: PlaybackRequestDescriptor? = null,
    ): PlaybackRequestTarget {
        val headers = buildMap {
            range?.takeIf { it.isNotBlank() }?.let { put("Range", it) }
        }
        val request = PlaybackRequestDescriptor(
            method = descriptor.method,
            url = descriptor.urlOn(profile),
            headers = headers,
        )
        return PlaybackRequestTarget(
            request = request,
            sessionProbeRequest = sessionProbeRequest,
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
        val authenticatedPreflightRequest = preflightRequest.authenticatedRequest(accessToken)
        val response = when (
            val result = executor.executeRequest(
                request = authenticatedPreflightRequest,
                secrets = listOf(accessToken),
            )
        ) {
            is PublicApiResult.Failure -> return failureFor(result.failure)
            is PublicApiResult.Success -> result.response
        }

        val sessionId = response
            .header(TARU_PLAYBACK_SESSION_ID_HEADER)
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

    private fun playbackRequestDescriptor(
        profile: ServerProfile,
        descriptor: TaruRequestDescriptor,
    ): PlaybackRequestDescriptor =
        PlaybackRequestDescriptor(
            method = descriptor.method,
            url = descriptor.urlOn(profile),
        )

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

    private fun remuxOutputContainer(decision: PlaybackDecisionResponse): ClientOutputContainer =
        decision.decision.transcodePlan
            ?.outputContainer
            ?.takeIf { it != ClientOutputContainer.Hls && it != ClientOutputContainer.Unknown }
            ?: ClientOutputContainer.Mp4

    private fun PlaybackCapabilities.toSdkPlaybackCapabilitiesQuery(): PlaybackCapabilitiesQuery =
        PlaybackCapabilitiesQuery(
            directPlay = directPlay,
            containers = containers,
            videoCodecs = videoCodecs,
            audioCodecs = audioCodecs,
        )

    private fun PlaybackCapabilities.toSdkRemuxPlaybackQuery(
        outputContainer: ClientOutputContainer,
    ): RemuxPlaybackQuery =
        RemuxPlaybackQuery(
            directPlay = directPlay,
            containers = containers,
            videoCodecs = videoCodecs,
            audioCodecs = audioCodecs,
            outputContainer = outputContainer.toSdkRemuxOutputContainerOrNull(),
        )

    private fun ClientOutputContainer.toSdkRemuxOutputContainerOrNull(): RemuxOutputContainer? =
        when (this) {
            ClientOutputContainer.Mp4 -> RemuxOutputContainer.Mp4
            ClientOutputContainer.Mkv -> RemuxOutputContainer.Mkv
            ClientOutputContainer.Hls,
            ClientOutputContainer.Unknown,
            -> null
        }
}
