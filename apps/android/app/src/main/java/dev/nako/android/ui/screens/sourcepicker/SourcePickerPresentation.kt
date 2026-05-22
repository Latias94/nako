package dev.nako.android.ui.screens.sourcepicker

import dev.nako.android.browse.MediaSourceDto
import dev.nako.android.media.ClientMediaStreamKind
import dev.nako.android.media.MediaProbeDto
import dev.nako.android.playback.ClientOutputContainer
import dev.nako.android.playback.ClientPlaybackDecision
import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackDecisionResponse
import dev.nako.android.player.PlaybackResumeSource
import dev.nako.android.player.ResumePlaybackPosition
import dev.nako.android.ui.browse.byteSizeLabel

internal data class SourcePickerDisplayModel(
    val sourceId: String,
    val primaryLabel: String,
    val secondaryText: String,
    val factLabels: List<String>,
    val selected: Boolean,
    val playbackMode: PlaybackModePresentation?,
) {
    val stateDescription: String = if (selected) "Selected" else "Not selected"
    val accessibilityLabel: String =
        "${if (selected) "Selected version" else "Choose version"}: $primaryLabel. $secondaryText."
}

internal data class PlaybackModePresentation(
    val label: String,
    val consequence: String,
    val warning: String?,
)

internal fun selectedSource(
    sources: List<MediaSourceDto>,
    selectedSourceId: String?,
): MediaSourceDto? =
    sources.firstOrNull { it.id == selectedSourceId } ?: sources.firstOrNull()

internal fun sourcePickerDisplayModel(
    source: MediaSourceDto,
    index: Int,
    selected: Boolean,
    activeDecision: PlaybackDecisionResponse?,
): SourcePickerDisplayModel {
    val playbackMode = activeDecision
        ?.takeIf { it.source.id == source.id }
        ?.decision
        ?.let(::playbackModePresentation)
    return SourcePickerDisplayModel(
        sourceId = source.id,
        primaryLabel = source.fileName.ifBlank { "Version ${index + 1}" },
        secondaryText = sourcePickerSecondaryText(source, index),
        factLabels = sourcePickerFacts(source),
        selected = selected,
        playbackMode = playbackMode,
    )
}

internal fun playbackModePresentation(
    decision: ClientPlaybackDecision,
): PlaybackModePresentation =
    when (decision.mode) {
        ClientPlaybackMode.DirectPlay -> PlaybackModePresentation(
            label = "Direct",
            consequence = "Original stream when this device can play it.",
            warning = null,
        )
        ClientPlaybackMode.Remux -> PlaybackModePresentation(
            label = "Remux",
            consequence = "Nako changes the container while keeping the original video and audio.",
            warning = "Container change",
        )
        ClientPlaybackMode.Transcode -> {
            val isHls = decision.transcodePlan?.outputContainer == ClientOutputContainer.Hls
            PlaybackModePresentation(
                label = if (isHls) "HLS" else "Transcode",
                consequence = if (isHls) {
                    "Nako prepares an adaptive stream before playback."
                } else {
                    "Nako converts the media for this device."
                },
                warning = "Prepared on server",
            )
        }
        ClientPlaybackMode.Unknown -> PlaybackModePresentation(
            label = "Unknown",
            consequence = "This server returned a playback mode this app does not understand.",
            warning = "Unsupported",
        )
    }

private fun sourcePickerSecondaryText(
    source: MediaSourceDto,
    index: Int,
): String =
    listOfNotNull(
        source.libraryId.takeIf { it.isNotBlank() }?.let { "Library $it" },
        "Version ${index + 1}",
    ).joinToString(" / ")

private fun sourcePickerFacts(source: MediaSourceDto): List<String> =
    buildList {
        add(byteSizeLabel(source.sizeBytes))
        if (!source.fingerprint.isNullOrBlank()) {
            add("Fingerprint available")
        }
    }

internal data class ResumePositionPresentation(
    val title: String,
    val body: String,
)

internal fun resumePositionPresentation(position: ResumePlaybackPosition): ResumePositionPresentation =
    ResumePositionPresentation(
        title = resumePositionTitle(position),
        body = resumePositionBody(position),
    )

internal fun resumePositionTitle(position: ResumePlaybackPosition): String =
    when (position.source) {
        PlaybackResumeSource.UserPlaybackState -> "Resume from your last server position"
        PlaybackResumeSource.DeviceLocal -> "Resume where this device stopped"
    }

internal fun resumePositionBody(position: ResumePlaybackPosition): String =
    when (position.source) {
        PlaybackResumeSource.UserPlaybackState ->
            "Nako will continue from the last position saved by your server after checking this version."
        PlaybackResumeSource.DeviceLocal ->
            "This device has a saved position for the selected version. Nako checks it before playback."
    }

internal fun probeFactLabels(probe: MediaProbeDto?): List<String> =
    buildList {
        if (probe == null) return@buildList
        probe.container?.takeIf { it.isNotBlank() }?.let { add(it.uppercase()) }
        probe.durationMs?.let { add(durationLabel(it)) }
        probe.bitRate?.let { add(bitRateLabel(it)) }
        probe.streams.firstOrNull { it.width != null && it.height != null }?.let { stream ->
            add(
                listOfNotNull(
                    stream.width?.let { width -> stream.height?.let { height -> "${width}x$height" } },
                    stream.codec,
                ).joinToString(" / "),
            )
        }
        val audioCount = probe.streams.count { it.kind == ClientMediaStreamKind.Audio }
        val subtitleCount = probe.streams.count { it.kind == ClientMediaStreamKind.Subtitle }
        if (audioCount > 0) add("$audioCount audio")
        if (subtitleCount > 0) add("$subtitleCount subtitle")
    }.filter { it.isNotBlank() }

private fun durationLabel(durationMs: Long): String {
    val totalMinutes = durationMs.coerceAtLeast(0L) / 60_000L
    val hours = totalMinutes / 60L
    val minutes = totalMinutes % 60L
    return if (hours > 0) {
        "${hours}h ${minutes}m"
    } else {
        "${minutes}m"
    }
}

private fun bitRateLabel(bitRate: Long): String =
    if (bitRate >= 1_000_000L) {
        "${bitRate / 1_000_000L} Mbps"
    } else {
        "${bitRate / 1_000L} Kbps"
    }
