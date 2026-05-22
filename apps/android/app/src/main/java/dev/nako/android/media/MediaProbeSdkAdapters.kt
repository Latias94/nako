package dev.nako.android.media

import dev.nako.sdk.MediaProbeDto as SdkMediaProbeDto
import dev.nako.sdk.MediaStreamDto as SdkMediaStreamDto
import dev.nako.sdk.SourceProbeResponse as SdkSourceProbeResponse

internal fun SdkSourceProbeResponse.toAndroid(): SourceProbeResponse =
    SourceProbeResponse(
        sourceId = sourceId,
        probe = probe.toAndroid(),
    )

internal fun SdkMediaProbeDto.toAndroid(): MediaProbeDto =
    MediaProbeDto(
        durationMs = durationMs,
        container = container,
        bitRate = bitRate,
        streams = streams.map(SdkMediaStreamDto::toAndroid),
    )

internal fun SdkMediaStreamDto.toAndroid(): MediaStreamDto =
    MediaStreamDto(
        index = index,
        kind = kind.toAndroidMediaStreamKind(),
        codec = codec,
        language = language,
        durationMs = durationMs,
        bitRate = bitRate,
        width = width,
        height = height,
        channels = channels,
        sampleRate = sampleRate,
    )

private fun String.toAndroidMediaStreamKind(): ClientMediaStreamKind =
    when (trim().lowercase()) {
        "video" -> ClientMediaStreamKind.Video
        "audio" -> ClientMediaStreamKind.Audio
        "subtitle" -> ClientMediaStreamKind.Subtitle
        "data" -> ClientMediaStreamKind.Data
        "attachment" -> ClientMediaStreamKind.Attachment
        else -> ClientMediaStreamKind.Other
    }
