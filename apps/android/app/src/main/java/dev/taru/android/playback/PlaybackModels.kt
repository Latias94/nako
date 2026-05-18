package dev.taru.android.playback

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
data class MediaProbeDto(
    @SerialName("duration_ms")
    val durationMs: Long? = null,
    val container: String? = null,
    @SerialName("bit_rate")
    val bitRate: Long? = null,
    val streams: List<MediaStreamDto> = emptyList(),
)

@Serializable
data class MediaStreamDto(
    val index: Int,
    val kind: ClientMediaStreamKind,
    val codec: String? = null,
    val language: String? = null,
    @SerialName("duration_ms")
    val durationMs: Long? = null,
    @SerialName("bit_rate")
    val bitRate: Long? = null,
    val width: Int? = null,
    val height: Int? = null,
    val channels: Int? = null,
    @SerialName("sample_rate")
    val sampleRate: Int? = null,
)

@Serializable
enum class ClientMediaStreamKind {
    @SerialName("video")
    Video,

    @SerialName("audio")
    Audio,

    @SerialName("subtitle")
    Subtitle,

    @SerialName("data")
    Data,

    @SerialName("attachment")
    Attachment,

    @SerialName("other")
    Other,
}

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
    @SerialName("input_locator")
    val inputLocator: String,
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

data class PlaybackRequestTarget(
    val request: TaruHttpRequest,
    val safeRequest: SafeRequestPreview,
)

enum class PlaybackFailureCategory {
    MissingSource,
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
