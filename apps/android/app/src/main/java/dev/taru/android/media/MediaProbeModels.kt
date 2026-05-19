package dev.taru.android.media

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class SourceProbeResponse(
    @SerialName("source_id")
    val sourceId: String,
    val probe: MediaProbeDto,
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
