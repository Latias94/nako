package dev.nako.android.media

data class SourceProbeResponse(
    val sourceId: String,
    val probe: MediaProbeDto,
)

data class MediaProbeDto(
    val durationMs: Long? = null,
    val container: String? = null,
    val bitRate: Long? = null,
    val streams: List<MediaStreamDto> = emptyList(),
)

data class MediaStreamDto(
    val index: Int,
    val kind: ClientMediaStreamKind,
    val codec: String? = null,
    val language: String? = null,
    val durationMs: Long? = null,
    val bitRate: Long? = null,
    val width: Int? = null,
    val height: Int? = null,
    val channels: Int? = null,
    val sampleRate: Int? = null,
)

enum class ClientMediaStreamKind {
    Video,
    Audio,
    Subtitle,
    Data,
    Attachment,
    Other,
}
