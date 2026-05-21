package dev.taru.android.playback

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.media.MediaProbeDto
import dev.taru.sdk.TARU_API_VERSION

private const val AuthorizationHeaderName = "Authorization"
private val BearerHeaderPattern = Regex("(?i)\\bBearer\\s+")

data class PlaybackCapabilities(
    val directPlay: Boolean? = null,
    val containers: List<String> = emptyList(),
    val videoCodecs: List<String> = emptyList(),
    val audioCodecs: List<String> = emptyList(),
)

data class PlaybackDecisionResponse(
    val source: PlaybackMediaSourceDto,
    val probe: MediaProbeDto? = null,
    val decision: ClientPlaybackDecision,
)

data class PlaybackMediaSourceDto(
    val id: String,
    val libraryId: String = "",
    val itemId: String = "",
    val fileName: String = "",
    val sizeBytes: Long? = null,
    val fingerprint: String? = null,
)

data class ClientPlaybackDecision(
    val mode: ClientPlaybackMode,
    val reason: String,
    val directPlay: ClientDirectPlayPlan? = null,
    val transcodePlan: ClientTranscodePlan? = null,
)

enum class ClientPlaybackMode {
    DirectPlay,
    Remux,
    Transcode,
    Unknown,
}

data class ClientDirectPlayPlan(
    val sourceId: String,
    val contentType: String,
    val supportsRangeRequests: Boolean,
)

data class ClientTranscodePlan(
    val outputContainer: ClientOutputContainer,
    val videoCodec: String? = null,
    val audioCodec: String? = null,
    val hardwareAcceleration: ClientHardwareAcceleration,
)

enum class ClientOutputContainer {
    Hls,
    Mp4,
    Mkv,
    Unknown,
}

enum class ClientHardwareAcceleration {
    None,
    Vaapi,
    Nvenc,
    QuickSync,
    Unknown,
}

data class PlaybackRequestDescriptor(
    val method: String,
    val url: String,
    val headers: Map<String, String> = emptyMap(),
) {
    init {
        require(headers.keys.none { it.equals(AuthorizationHeaderName, ignoreCase = true) }) {
            "Playback request descriptors must not carry Authorization."
        }
        require(headers.values.none { BearerHeaderPattern.containsMatchIn(it) }) {
            "Playback request descriptors must not carry bearer tokens."
        }
    }

    val safeRequest: SafeRequestPreview
        get() = SafeRequestPreview(
            method = method,
            url = SensitiveText.sanitize(url),
            headers = buildMap {
                headers.forEach { (name, value) ->
                    put(name, SensitiveText.sanitize(value))
                }
                put(AuthorizationHeaderName, "Bearer ${SensitiveText.redacted}")
            },
        )

    fun authenticatedRequest(accessToken: String): TaruHttpRequest {
        require(accessToken.isNotBlank()) {
            "A server access key is required to build the final playback request."
        }
        return TaruHttpRequest(
            method = method,
            url = url,
            headers = headers + (AuthorizationHeaderName to "Bearer $accessToken"),
        )
    }
}

data class PlaybackRequestTarget(
    val request: PlaybackRequestDescriptor,
    val sessionProbeRequest: PlaybackRequestDescriptor? = null,
    val sessionId: String? = null,
) {
    val safeRequest: SafeRequestPreview
        get() = request.safeRequest

    fun authenticatedRequest(accessToken: String): TaruHttpRequest =
        request.authenticatedRequest(accessToken)

    fun authenticatedSessionProbeRequest(accessToken: String): TaruHttpRequest? =
        sessionProbeRequest?.authenticatedRequest(accessToken)

    override fun toString(): String =
        "PlaybackRequestTarget(safeRequest=$safeRequest, hasSessionProbeRequest=${sessionProbeRequest != null}, sessionId=$sessionId)"
}

data class TranscodeSessionResponse(
    val session: TranscodeSessionDto,
)

data class TranscodeSessionDto(
    val id: String,
    val sourceId: String,
    val kind: ClientTranscodeSessionKind,
    val requestKey: String,
    val state: ClientTranscodeSessionState,
    val failureCategory: ClientTranscodeFailureCategory? = null,
    val failureMessage: String? = null,
    val createdAt: String,
    val updatedAt: String,
    val startedAt: String? = null,
    val completedAt: String? = null,
)

enum class ClientTranscodeSessionKind {
    Remux,
    HlsTranscode,
    Unknown,
}

enum class ClientTranscodeSessionState {
    Planned,
    Starting,
    Running,
    CancelRequested,
    Cancelled,
    Failed,
    Finished,
    Unknown,
}

enum class ClientTranscodeFailureCategory {
    InvalidRequest,
    Runner,
    Timeout,
    Storage,
    Stale,
    Cancelled,
    Unknown,
}

enum class PlaybackFailureCategory {
    MissingSource,
    MissingSession,
    MissingAccessToken,
    UnreachableServer,
    Unauthorized,
    Forbidden,
    UnsupportedApiVersion,
    TlsOrCertificate,
    UnsupportedSource,
    SourceUnavailable,
    SessionConflict,
    PublicApiError,
    InvalidResponse,
}

data class SafePlaybackDiagnostics(
    val category: PlaybackFailureCategory,
    val userMessage: String,
    val statusCode: Int? = null,
    val expectedApiVersion: String = TARU_API_VERSION,
    val observedApiVersion: String? = null,
    val publicError: PublicErrorEnvelope? = null,
    val request: SafeRequestPreview? = null,
)

sealed interface PlaybackResult<out T> {
    data class Success<T>(
        val value: T,
        val request: SafeRequestPreview,
    ) : PlaybackResult<T>

    data class Failure(
        val diagnostics: SafePlaybackDiagnostics,
    ) : PlaybackResult<Nothing>
}
