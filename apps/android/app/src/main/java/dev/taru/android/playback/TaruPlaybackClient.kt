package dev.taru.android.playback

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.media.SourceProbeResponse
import java.io.IOException
import java.net.URLEncoder
import java.nio.charset.StandardCharsets
import javax.net.ssl.SSLException
import kotlinx.serialization.SerializationException
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json

class TaruPlaybackClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    suspend fun getSourceProbe(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
    ): PlaybackResult<SourceProbeResponse> {
        if (sourceId.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingSource,
                userMessage = "Choose a Media Source before requesting source facts.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/sources/${encodePathSegment(sourceId)}/probe",
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
                userMessage = "Choose a Media Source before requesting playback.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/sources/${encodePathSegment(sourceId)}/playback/decision${capabilitiesQuery(capabilities)}",
        )
    }

    fun directPlaybackTarget(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            accessToken = accessToken,
            method = "GET",
            pathAndQuery = "/sources/${encodePathSegment(sourceId)}/stream",
            range = range,
        )

    fun headDirectPlaybackTarget(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
        range: String? = null,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            accessToken = accessToken,
            method = "HEAD",
            pathAndQuery = "/sources/${encodePathSegment(sourceId)}/stream",
            range = range,
        )

    fun remuxPlaybackTarget(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
        outputContainer: ClientOutputContainer = ClientOutputContainer.Mp4,
        range: String? = null,
    ): PlaybackRequestTarget {
        val pathAndQuery = "/sources/${encodePathSegment(sourceId)}/stream/remux${
            remuxQuery(capabilities, outputContainer)
        }"
        return playbackTarget(
            profile = profile,
            accessToken = accessToken,
            method = "GET",
            pathAndQuery = pathAndQuery,
            range = range,
            sessionProbeRequest = authenticatedRequest(
                profile = profile,
                accessToken = accessToken,
                method = "HEAD",
                pathAndQuery = pathAndQuery,
            ),
        )
    }

    fun hlsPlaylistTarget(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
        capabilities: PlaybackCapabilities = PlaybackCapabilities(),
    ): PlaybackRequestTarget {
        val pathAndQuery = "/sources/${encodePathSegment(sourceId)}/stream/hls/playlist.m3u8${
            capabilitiesQuery(capabilities)
        }"
        return playbackTarget(
            profile = profile,
            accessToken = accessToken,
            method = "GET",
            pathAndQuery = pathAndQuery,
            sessionProbeRequest = authenticatedRequest(
                profile = profile,
                accessToken = accessToken,
                method = "GET",
                pathAndQuery = pathAndQuery,
            ),
        )
    }

    fun hlsSegmentTarget(
        profile: ServerProfile,
        accessToken: String,
        sessionId: String,
        segmentName: String,
    ): PlaybackRequestTarget =
        playbackTarget(
            profile = profile,
            accessToken = accessToken,
            method = "GET",
            pathAndQuery = "/playback/sessions/${encodePathSegment(sessionId)}/hls/segments/${
                encodePathSegment(segmentName)
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
            pathAndQuery = "/playback/sessions/${encodePathSegment(sessionId)}",
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
            pathAndQuery = "/playback/sessions/${encodePathSegment(sessionId)}/cancel",
        )
    }

    fun recommendedPlaybackTarget(
        profile: ServerProfile,
        accessToken: String,
        decision: PlaybackDecisionResponse,
    ): PlaybackRequestTarget? {
        val target = when (decision.decision.mode) {
            ClientPlaybackMode.DirectPlay -> directPlaybackTarget(
                profile = profile,
                accessToken = accessToken,
                sourceId = decision.source.id,
            )
            ClientPlaybackMode.Remux -> remuxPlaybackTarget(
                profile = profile,
                accessToken = accessToken,
                sourceId = decision.source.id,
                outputContainer = remuxOutputContainer(decision),
            )
            ClientPlaybackMode.Transcode -> hlsPlaylistTarget(
                profile = profile,
                accessToken = accessToken,
                sourceId = decision.source.id,
            )
        }
        return target
    }

    suspend fun prepareRecommendedPlaybackTarget(
        profile: ServerProfile,
        accessToken: String,
        decision: PlaybackDecisionResponse,
    ): PlaybackResult<PlaybackRequestTarget> {
        val target = recommendedPlaybackTarget(
            profile = profile,
            accessToken = accessToken,
            decision = decision,
        ) ?: return failure(
            category = PlaybackFailureCategory.UnsupportedSource,
            userMessage = "The server did not return a playable route for this source.",
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
        if (accessToken.isBlank()) {
            return failure(
                category = PlaybackFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before requesting playback.",
            )
        }

        val request = authenticatedRequest(
            profile = profile,
            accessToken = accessToken,
            method = method,
            pathAndQuery = pathAndQuery,
        )
        val response = when (val result = executeOrFailure(request, accessToken)) {
            is TransportResult.Failure -> return result.failure
            is TransportResult.Response -> result.response
        }

        if (!response.isSuccessful()) {
            return httpFailure(request, response, accessToken)
        }

        val observedApiVersion = response.header(TaruPublicApiContract.apiVersionHeader)
        if (observedApiVersion != null && observedApiVersion != TaruPublicApiContract.expectedApiVersion) {
            return failure(
                category = PlaybackFailureCategory.UnsupportedApiVersion,
                userMessage = "This server uses an unsupported Public Client API version.",
                observedApiVersion = observedApiVersion,
                request = safeRequest(request),
            )
        }

        val decoded = try {
            json.decodeFromString<T>(response.body)
        } catch (_: SerializationException) {
            return invalidResponseFailure(request)
        } catch (_: IllegalArgumentException) {
            return invalidResponseFailure(request)
        }

        return PlaybackResult.Success(
            value = decoded,
            request = safeRequest(request),
        )
    }

    private suspend fun executeOrFailure(
        request: TaruHttpRequest,
        accessToken: String,
    ): TransportResult =
        try {
            TransportResult.Response(transport.execute(request))
        } catch (_: SSLException) {
            TransportResult.Failure(
                failure(
                    category = PlaybackFailureCategory.TlsOrCertificate,
                    userMessage = "The server TLS certificate could not be trusted.",
                    request = safeRequest(request),
                ),
            )
        } catch (error: IOException) {
            TransportResult.Failure(
                failure(
                    category = PlaybackFailureCategory.UnreachableServer,
                    userMessage = "The server could not be reached. Check the address and network.",
                    publicError = PublicErrorEnvelope(
                        code = "transport_error",
                        message = SensitiveText.sanitize(error.message.orEmpty(), listOf(accessToken)),
                    ),
                    request = safeRequest(request),
                ),
            )
        }

    private fun httpFailure(
        request: TaruHttpRequest,
        response: TaruHttpResponse,
        accessToken: String,
    ): PlaybackResult.Failure {
        val publicError = parsePublicError(response.body, accessToken)
        val category = when (response.statusCode) {
            400 -> PlaybackFailureCategory.UnsupportedSource
            401 -> PlaybackFailureCategory.Unauthorized
            403 -> PlaybackFailureCategory.Forbidden
            404 -> PlaybackFailureCategory.SourceUnavailable
            409 -> PlaybackFailureCategory.SessionConflict
            else -> PlaybackFailureCategory.PublicApiError
        }
        val userMessage = when (category) {
            PlaybackFailureCategory.UnsupportedSource ->
                "This Media Source cannot be played with the current client capabilities."
            PlaybackFailureCategory.Unauthorized ->
                "The access token is invalid or expired."
            PlaybackFailureCategory.Forbidden ->
                "This access token cannot play the requested source."
            PlaybackFailureCategory.SourceUnavailable ->
                "The selected Media Source is no longer available."
            PlaybackFailureCategory.SessionConflict ->
                "A matching playback session is already running or cannot be changed yet."
            else ->
                "The server returned a playback API error."
        }

        return failure(
            category = category,
            userMessage = userMessage,
            statusCode = response.statusCode,
            observedApiVersion = response.header(TaruPublicApiContract.apiVersionHeader),
            publicError = publicError,
            request = safeRequest(request),
        )
    }

    private fun playbackTarget(
        profile: ServerProfile,
        accessToken: String,
        method: String,
        pathAndQuery: String,
        range: String? = null,
        sessionProbeRequest: TaruHttpRequest? = null,
    ): PlaybackRequestTarget {
        val headers = buildMap {
            put("Authorization", "Bearer $accessToken")
            range?.takeIf { it.isNotBlank() }?.let { put("Range", it) }
        }
        val request = TaruHttpRequest(
            method = method,
            url = joinUrl(profile.baseUrl, pathAndQuery),
            headers = headers,
        )
        return PlaybackRequestTarget(
            request = request,
            safeRequest = safeRequest(request),
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
            userMessage = "The playback route does not expose a public session preflight request.",
            request = target.safeRequest,
        )
        val response = when (val result = executeOrFailure(preflightRequest, accessToken)) {
            is TransportResult.Failure -> return result.failure
            is TransportResult.Response -> result.response
        }
        if (!response.isSuccessful()) {
            return httpFailure(preflightRequest, response, accessToken)
        }
        val observedApiVersion = response.header(TaruPublicApiContract.apiVersionHeader)
        if (observedApiVersion != null && observedApiVersion != TaruPublicApiContract.expectedApiVersion) {
            return failure(
                category = PlaybackFailureCategory.UnsupportedApiVersion,
                userMessage = "This server uses an unsupported Public Client API version.",
                observedApiVersion = observedApiVersion,
                request = safeRequest(preflightRequest),
            )
        }

        val sessionId = response
            .header(TaruPublicApiContract.playbackSessionIdHeader)
            ?.takeIf { it.isNotBlank() }
            ?: return failure(
                category = PlaybackFailureCategory.MissingSession,
                userMessage = "The server did not expose a playback session for this route.",
                request = safeRequest(preflightRequest),
            )

        return PlaybackResult.Success(
            value = target.copy(sessionId = sessionId),
            request = target.safeRequest,
        )
    }

    private fun authenticatedRequest(
        profile: ServerProfile,
        accessToken: String,
        method: String,
        pathAndQuery: String,
    ): TaruHttpRequest =
        TaruHttpRequest(
            method = method,
            url = joinUrl(profile.baseUrl, pathAndQuery),
            headers = mapOf("Authorization" to "Bearer $accessToken"),
        )

    private fun invalidResponseFailure(request: TaruHttpRequest): PlaybackResult.Failure =
        failure(
            category = PlaybackFailureCategory.InvalidResponse,
            userMessage = "The playback response could not be understood.",
            request = safeRequest(request),
        )

    private fun parsePublicError(
        body: String,
        accessToken: String,
    ): PublicErrorEnvelope? =
        try {
            SensitiveText.sanitizeEnvelope(
                json.decodeFromString<PublicErrorEnvelope>(body),
                listOf(accessToken),
            )
        } catch (_: SerializationException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }

    private fun safeRequest(request: TaruHttpRequest): SafeRequestPreview =
        SafeRequestPreview(
            method = request.method,
            url = SensitiveText.sanitize(request.url),
            headers = request.headers.mapValues { (name, value) ->
                if (name.equals("Authorization", ignoreCase = true)) {
                    "Bearer ${TaruPublicApiContract.redacted}"
                } else {
                    SensitiveText.sanitize(value)
                }
            },
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
        if (pairs.isEmpty()) {
            ""
        } else {
            pairs.joinToString(
                separator = "&",
                prefix = "?",
            ) { (name, value) ->
                "${encodeQueryValue(name)}=${encodeQueryValue(value)}"
            }
        }

    private fun joinUrl(baseUrl: String, pathAndQuery: String): String =
        "${baseUrl.trimEnd('/')}$pathAndQuery"

    private fun encodeQueryValue(value: String): String =
        URLEncoder.encode(value, StandardCharsets.UTF_8)

    private fun encodePathSegment(value: String): String =
        URLEncoder
            .encode(value, StandardCharsets.UTF_8)
            .replace("+", "%20")

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

    private sealed interface TransportResult {
        data class Response(val response: TaruHttpResponse) : TransportResult
        data class Failure(val failure: PlaybackResult.Failure) : TransportResult
    }
}
