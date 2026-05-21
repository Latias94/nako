package dev.taru.android.playback

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.PublicApiAuth
import dev.taru.android.connection.PublicApiFailure
import dev.taru.android.connection.PublicApiFailureKind
import dev.taru.android.connection.PublicApiResult
import dev.taru.android.connection.PublicApiUrl
import dev.taru.android.connection.PublicClientApiExecutor
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.media.SourceProbeResponse
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

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/sources/${PublicApiUrl.encodePathSegment(sourceId)}/probe",
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

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/sources/${PublicApiUrl.encodePathSegment(sourceId)}/playback/decision${capabilitiesQuery(capabilities)}",
        )
    }

    fun directPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            method = "GET",
            pathAndQuery = "/sources/${PublicApiUrl.encodePathSegment(sourceId)}/stream",
            range = range,
        )

    fun headDirectPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            method = "HEAD",
            pathAndQuery = "/sources/${PublicApiUrl.encodePathSegment(sourceId)}/stream",
            range = range,
        )

    fun remuxPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
        outputContainer: ClientOutputContainer = ClientOutputContainer.Mp4,
        range: String? = null,
    ): PlaybackRequestTarget {
        val pathAndQuery = "/sources/${PublicApiUrl.encodePathSegment(sourceId)}/stream/remux${
            remuxQuery(capabilities, outputContainer)
        }"
        return playbackTarget(
            profile = profile,
            method = "GET",
            pathAndQuery = pathAndQuery,
            range = range,
            sessionProbeRequest = playbackRequestDescriptor(
                profile = profile,
                method = "HEAD",
                pathAndQuery = pathAndQuery,
            ),
        )
    }

    fun hlsPlaylistTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackRequestTarget {
        val pathAndQuery = "/sources/${PublicApiUrl.encodePathSegment(sourceId)}/stream/hls/playlist.m3u8${
            capabilitiesQuery(capabilities)
        }"
        return playbackTarget(
            profile = profile,
            method = "GET",
            pathAndQuery = pathAndQuery,
            sessionProbeRequest = playbackRequestDescriptor(
                profile = profile,
                method = "GET",
                pathAndQuery = pathAndQuery,
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
            method = "GET",
            pathAndQuery = "/playback/sessions/${PublicApiUrl.encodePathSegment(sessionId)}/hls/segments/${
                PublicApiUrl.encodePathSegment(segmentName)
            }",
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

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/playback/sessions/${PublicApiUrl.encodePathSegment(sessionId)}",
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

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            method = "POST",
            pathAndQuery = "/playback/sessions/${PublicApiUrl.encodePathSegment(sessionId)}/cancel",
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
        }
    }

    private suspend inline fun <reified T> executeJson(
        profile: ServerProfile,
        accessToken: String,
        method: String = "GET",
        pathAndQuery: String,
    ): PlaybackResult<T> {
        return when (
            val result = executor.executeJson<T>(
                baseUrl = profile.baseUrl,
                pathAndQuery = pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
                method = method,
            )
        ) {
            is PublicApiResult.Success -> PlaybackResult.Success(
                value = result.value,
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
        method: String,
        pathAndQuery: String,
        range: String? = null,
        sessionProbeRequest: PlaybackRequestDescriptor? = null,
    ): PlaybackRequestTarget {
        val headers = buildMap {
            range?.takeIf { it.isNotBlank() }?.let { put("Range", it) }
        }
        val request = PlaybackRequestDescriptor(
            method = method,
            url = PublicApiUrl.join(profile.baseUrl, pathAndQuery),
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
            .header(TaruPublicApiContract.playbackSessionIdHeader)
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
        method: String,
        pathAndQuery: String,
    ): PlaybackRequestDescriptor =
        PlaybackRequestDescriptor(
            method = method,
            url = PublicApiUrl.join(profile.baseUrl, pathAndQuery),
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

    private fun capabilitiesQuery(capabilities: PlaybackCapabilities): String =
        queryString(
            buildList {
                capabilities.directPlay?.let {
                    add("direct_play" to if (it) "true" else "false")
                }
                addCsv("container", capabilities.containers)
                addCsv("video_codec", capabilities.videoCodecs)
                addCsv("audio_codec", capabilities.audioCodecs)
            },
        )

    private fun remuxQuery(
        capabilities: PlaybackCapabilities,
        outputContainer: ClientOutputContainer,
    ): String =
        queryString(
            buildList {
                capabilities.directPlay?.let {
                    add("direct_play" to if (it) "true" else "false")
                }
                addCsv("container", capabilities.containers)
                addCsv("video_codec", capabilities.videoCodecs)
                addCsv("audio_codec", capabilities.audioCodecs)
                add("output_container" to outputContainer.wireValue)
            },
        )

    private fun MutableList<Pair<String, String>>.addCsv(
        name: String,
        values: List<String>,
    ) {
        val cleaned = values.map(String::trim).filter(String::isNotEmpty)
        if (cleaned.isNotEmpty()) {
            add(name to cleaned.joinToString(","))
        }
    }

    private fun queryString(pairs: List<Pair<String, String>>): String =
        PublicApiUrl.queryString(pairs)

    private val ClientOutputContainer.wireValue: String
        get() = when (this) {
            ClientOutputContainer.Hls -> "hls"
            ClientOutputContainer.Mp4 -> "mp4"
            ClientOutputContainer.Mkv -> "mkv"
        }

    private fun remuxOutputContainer(decision: PlaybackDecisionResponse): ClientOutputContainer =
        decision.decision.transcodePlan
            ?.outputContainer
            ?.takeIf { it != ClientOutputContainer.Hls }
            ?: ClientOutputContainer.Mp4
}
