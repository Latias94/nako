package dev.taru.android.playback

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.media.MediaProbeDto
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

private const val AuthorizationHeaderName = "Authorization"
private val BearerHeaderPattern = Regex("(?i)\\bBearer\\s+")

data class PlaybackCapabilities(
    val directPlay: Boolean? = null,
    val containers: List<String> = emptyList(),
    val videoCodecs: List<String> = emptyList(),
    val audioCodecs: List<String> = emptyList(),
)

@Serializable
data class PlaybackDecisionResponse(
    val source: PlaybackMediaSourceDto,
    val probe: MediaProbeDto? = null,
    val decision: ClientPlaybackDecision,
)

@Serializable
data class PlaybackMediaSourceDto(
    val id: String,
    @SerialName("library_id")
    val libraryId: String = "",
    @SerialName("item_id")
    val itemId: String = "",
    val locator: String = "",
    @SerialName("file_name")
    val fileName: String = "",
    @SerialName("size_bytes")
    val sizeBytes: Long? = null,
    val fingerprint: String? = null,
)

@Serializable
data class ClientPlaybackDecision(
    val mode: ClientPlaybackMode,
    val reason: String,
    @SerialName("direct_play")
    val directPlay: ClientDirectPlayPlan? = null,
    @SerialName("transcode_plan")
    val transcodePlan: ClientTranscodePlan? = null,
)

@Serializable
enum class ClientPlaybackMode {
    @SerialName("direct_play")
    DirectPlay,

    @SerialName("remux")
    Remux,

    @SerialName("transcode")
    Transcode,
}

@Serializable
data class ClientDirectPlayPlan(
    @SerialName("source_id")
    val sourceId: String,
    @SerialName("content_type")
    val contentType: String,
    @SerialName("supports_range_requests")
    val supportsRangeRequests: Boolean,
)

@Serializable
data class ClientTranscodePlan(
    @SerialName("output_container")
    val outputContainer: ClientOutputContainer,
    @SerialName("video_codec")
    val videoCodec: String? = null,
    @SerialName("audio_codec")
    val audioCodec: String? = null,
    @SerialName("hardware_acceleration")
    val hardwareAcceleration: ClientHardwareAcceleration,
)

@Serializable
enum class ClientOutputContainer {
    @SerialName("hls")
    Hls,

    @SerialName("mp4")
    Mp4,

    @SerialName("mkv")
    Mkv,
}

@Serializable
enum class ClientHardwareAcceleration {
    @SerialName("none")
    None,

    @SerialName("vaapi")
    Vaapi,

    @SerialName("nvenc")
    Nvenc,

    @SerialName("quick_sync")
    QuickSync,
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
                put(AuthorizationHeaderName, "Bearer ${TaruPublicApiContract.redacted}")
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

@Serializable
data class TranscodeSessionResponse(
    val session: TranscodeSessionDto,
)

@Serializable
data class TranscodeSessionDto(
    val id: String,
    @SerialName("source_id")
    val sourceId: String,
    val kind: ClientTranscodeSessionKind,
    @SerialName("request_key")
    val requestKey: String,
    val state: ClientTranscodeSessionState,
    @SerialName("failure_category")
    val failureCategory: ClientTranscodeFailureCategory? = null,
    @SerialName("failure_message")
    val failureMessage: String? = null,
    @SerialName("created_at")
    val createdAt: String,
    @SerialName("updated_at")
    val updatedAt: String,
    @SerialName("started_at")
    val startedAt: String? = null,
    @SerialName("completed_at")
    val completedAt: String? = null,
)

@Serializable
enum class ClientTranscodeSessionKind {
    @SerialName("remux")
    Remux,

    @SerialName("hls_transcode")
    HlsTranscode,
}

@Serializable
enum class ClientTranscodeSessionState {
    @SerialName("planned")
    Planned,

    @SerialName("starting")
    Starting,

    @SerialName("running")
    Running,

    @SerialName("cancel_requested")
    CancelRequested,

    @SerialName("cancelled")
    Cancelled,

    @SerialName("failed")
    Failed,

    @SerialName("finished")
    Finished,
}

@Serializable
enum class ClientTranscodeFailureCategory {
    @SerialName("invalid_request")
    InvalidRequest,

    @SerialName("runner")
    Runner,

    @SerialName("timeout")
    Timeout,

    @SerialName("storage")
    Storage,

    @SerialName("stale")
    Stale,

    @SerialName("cancelled")
    Cancelled,

    @SerialName("unknown")
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
    val expectedApiVersion: String = dev.taru.android.connection.TaruPublicApiContract.expectedApiVersion,
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
