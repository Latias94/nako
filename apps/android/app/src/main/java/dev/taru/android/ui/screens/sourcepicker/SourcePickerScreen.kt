package dev.taru.android.ui.screens.sourcepicker

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.RadioButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.media.ClientMediaStreamKind
import dev.taru.android.media.MediaProbeDto
import dev.taru.android.playback.ClientPlaybackDecision
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.ClientOutputContainer
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.PlaybackDecisionResponse
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.PlaybackResumeSource
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.ui.TaruStrings
import dev.taru.android.ui.browse.IconBadge
import dev.taru.android.ui.browse.PlaybackSelectionUiState
import dev.taru.android.ui.browse.SourceProbeUiState
import dev.taru.android.ui.browse.StatusChip
import dev.taru.android.ui.browse.SurfaceCard
import dev.taru.android.ui.browse.byteSizeLabel
import dev.taru.android.ui.browse.playbackFailureTitle
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

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

@Composable
internal fun SourcePickerSurface(
    sources: List<MediaSourceDto>,
    sourceProbeState: SourceProbeUiState,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    resumePosition: ResumePlaybackPosition?,
    onSelectSource: (String) -> Unit,
    onRetrySourceProbe: () -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val selectedSource = selectedSource(sources, selectedSourceId)
    val activeDecision = (playbackState as? PlaybackSelectionUiState.Content)?.response
    val selectedDecision = activeDecision?.takeIf { it.source.id == selectedSource?.id }
    val models = sources.mapIndexed { index, source ->
        sourcePickerDisplayModel(
            source = source,
            index = index,
            selected = source.id == selectedSource?.id,
            activeDecision = activeDecision,
        )
    }

    SurfaceCard {
        SourcePickerHeader(
            sourceCount = sources.size,
            selectedSource = selectedSource,
        )

        if (sources.isEmpty()) {
            Text(
                text = "No playable version is available for this title.",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
            return@SurfaceCard
        }

        SourceProbePanel(
            selectedSource = selectedSource,
            state = sourceProbeState,
            onRetry = onRetrySourceProbe,
            onChangeServer = onChangeServer,
        )

        SelectedSourceDecisionPanel(
            selectedSource = selectedSource,
            playbackState = playbackState,
            selectedDecision = selectedDecision,
            resumePosition = resumePosition,
            onRequestDecision = {
                selectedSource?.id?.let(onRequestPlayback)
            },
            onRetryPlayback = onRetryPlayback,
            onChangeServer = onChangeServer,
            onStartPlayback = onStartPlayback,
        )

        Text(
            text = "Versions",
            color = TaruTextSecondary,
            style = MaterialTheme.typography.titleSmall,
        )

        Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
            models.forEach { model ->
                SourcePickerRow(
                    model = model,
                    enabled = playbackState !is PlaybackSelectionUiState.Loading,
                    onSelect = { onSelectSource(model.sourceId) },
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SourceProbePanel(
    selectedSource: MediaSourceDto?,
    state: SourceProbeUiState,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
) {
    if (selectedSource == null || state == SourceProbeUiState.Idle) return

    when (state) {
        SourceProbeUiState.Idle -> Unit
        SourceProbeUiState.Loading -> FlowRow(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        ) {
            StatusChip(text = "Checking version details")
        }
        is SourceProbeUiState.Content -> {
            val facts = if (state.response.sourceId == selectedSource.id) {
                probeFactLabels(state.response.probe)
            } else {
                emptyList()
            }
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                StatusChip(text = "Version details")
                facts.ifEmpty { listOf("No details available") }.forEach { fact ->
                    StatusChip(text = fact)
                }
            }
        }
        is SourceProbeUiState.Failure -> Row(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Version details unavailable",
                color = TaruTextMuted,
                style = MaterialTheme.typography.labelMedium,
            )
            OutlinedButton(onClick = onRetry) {
                Text(stringResource(TaruStrings.retry))
            }
            if (state.diagnostics.category in serverChangeCategories) {
                OutlinedButton(onClick = onChangeServer) {
                    Text(stringResource(TaruStrings.changeServer))
                }
            }
        }
    }
}

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
            consequence = "Taru changes the container while keeping the original video and audio.",
            warning = "Container change",
        )
        ClientPlaybackMode.Transcode -> {
            val isHls = decision.transcodePlan?.outputContainer == ClientOutputContainer.Hls
            PlaybackModePresentation(
                label = if (isHls) "HLS" else "Transcode",
                consequence = if (isHls) {
                    "Taru prepares an adaptive stream before playback."
                } else {
                    "Taru converts the media for this device."
                },
                warning = "Prepared on server",
            )
        }
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

@Composable
private fun SourcePickerHeader(
    sourceCount: Int,
    selectedSource: MediaSourceDto?,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconBadge(icon = Icons.Rounded.Storage)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = "Version",
                style = MaterialTheme.typography.titleLarge,
            )
            Text(
                text = selectedSource?.fileName?.takeIf { it.isNotBlank() }
                    ?: "Choose which version Taru should prepare.",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        StatusChip(text = "$sourceCount")
    }
}

@Composable
private fun SelectedSourceDecisionPanel(
    selectedSource: MediaSourceDto?,
    playbackState: PlaybackSelectionUiState,
    selectedDecision: PlaybackDecisionResponse?,
    resumePosition: ResumePlaybackPosition?,
    onRequestDecision: () -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.10f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.28f)),
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            when (playbackState) {
                PlaybackSelectionUiState.Idle -> DecisionIdleContent(
                    selectedSource = selectedSource,
                    resumePosition = resumePosition,
                    onRequestDecision = onRequestDecision,
                )
                PlaybackSelectionUiState.Loading -> Text(
                    text = "Checking the best way to play this version.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                is PlaybackSelectionUiState.Content -> {
                    if (selectedDecision == null) {
                        DecisionIdleContent(
                            selectedSource = selectedSource,
                            resumePosition = resumePosition,
                            onRequestDecision = onRequestDecision,
                        )
                    } else {
                        DecisionReadyContent(
                            state = playbackState,
                            selectedDecision = selectedDecision,
                            resumePosition = resumePosition,
                            onStartPlayback = onStartPlayback,
                        )
                    }
                }
                is PlaybackSelectionUiState.Failure -> DecisionFailureContent(
                    state = playbackState,
                    onRetry = onRetryPlayback,
                    onChangeServer = onChangeServer,
                )
            }
        }
    }
}

@Composable
private fun DecisionIdleContent(
    selectedSource: MediaSourceDto?,
    resumePosition: ResumePlaybackPosition?,
    onRequestDecision: () -> Unit,
) {
    Text(
        text = if (resumePosition != null) {
            resumePositionTitle(resumePosition)
        } else {
            "Ready to check playback"
        },
        style = MaterialTheme.typography.titleMedium,
    )
    Text(
        text = if (resumePosition != null) {
            resumePositionBody(resumePosition)
        } else {
            "Taru will check the best playback path before the player opens."
        },
        color = TaruTextSecondary,
        style = MaterialTheme.typography.bodyMedium,
    )
    Button(
        onClick = onRequestDecision,
        enabled = selectedSource != null,
    ) {
        Icon(
            imageVector = Icons.Rounded.PlayArrow,
            contentDescription = null,
        )
        Spacer(modifier = Modifier.width(TaruSpacing.small))
        Text(if (resumePosition != null) "Resume" else "Play")
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun DecisionReadyContent(
    state: PlaybackSelectionUiState.Content,
    selectedDecision: PlaybackDecisionResponse,
    resumePosition: ResumePlaybackPosition?,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val presentation = playbackModePresentation(selectedDecision.decision)
    Row(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconBadge(icon = Icons.Rounded.CheckCircle, compact = true)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = "${presentation.label} ready",
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = presentation.consequence,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        presentation.warning?.let { StatusChip(text = it) }
    }

    if (selectedDecision.decision.reason.isNotBlank()) {
        Text(
            text = selectedDecision.decision.reason,
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
    }

    val probeFacts = probeFactLabels(selectedDecision.probe)
    if (probeFacts.isNotEmpty()) {
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        ) {
            probeFacts.forEach { fact -> StatusChip(text = fact) }
        }
    }

    state.target?.let { target ->
        Button(onClick = { onStartPlayback(target) }) {
            Icon(
                imageVector = Icons.Rounded.PlayArrow,
                contentDescription = null,
            )
            Spacer(modifier = Modifier.width(TaruSpacing.small))
            Text(if (resumePosition != null) "Start resume" else "Start playback")
        }
    } ?: Text(
        text = "No playable version was prepared.",
        color = TaruTextMuted,
        style = MaterialTheme.typography.labelMedium,
    )
}

private fun resumePositionTitle(position: ResumePlaybackPosition): String =
    when (position.source) {
        PlaybackResumeSource.UserPlaybackState -> "Resume from your last server position"
        PlaybackResumeSource.DeviceLocal -> "Resume where this device stopped"
    }

private fun resumePositionBody(position: ResumePlaybackPosition): String =
    when (position.source) {
        PlaybackResumeSource.UserPlaybackState ->
            "Taru will continue from the last position saved by your server after checking this version."
        PlaybackResumeSource.DeviceLocal ->
            "This device has a saved position for the selected version. Taru checks it before playback."
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

@Composable
private fun DecisionFailureContent(
    state: PlaybackSelectionUiState.Failure,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconBadge(icon = Icons.Rounded.Info, compact = true)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = playbackFailureTitle(state.diagnostics.category),
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = state.diagnostics.userMessage,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
    state.diagnostics.publicError?.let { publicError ->
        Text(
            text = "${publicError.code}: ${publicError.message}",
            color = TaruTextMuted,
            style = MaterialTheme.typography.labelMedium,
        )
    }
    Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
        Button(onClick = onRetry) {
            Text(stringResource(TaruStrings.retry))
        }
        OutlinedButton(onClick = onChangeServer) {
            Text(stringResource(TaruStrings.changeServer))
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SourcePickerRow(
    model: SourcePickerDisplayModel,
    enabled: Boolean,
    onSelect: () -> Unit,
) {
    val borderColor = if (model.selected) {
        MaterialTheme.colorScheme.primary.copy(alpha = 0.58f)
    } else {
        MaterialTheme.colorScheme.outline.copy(alpha = 0.24f)
    }
    val containerColor = if (model.selected) {
        MaterialTheme.colorScheme.primary.copy(alpha = 0.08f)
    } else {
        MaterialTheme.colorScheme.surface
    }
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .semantics {
                contentDescription = model.accessibilityLabel
                stateDescription = model.stateDescription
                role = Role.RadioButton
                selected = model.selected
            }
            .clickable(
                enabled = enabled,
                role = Role.RadioButton,
                onClickLabel = if (model.selected) "Keep selected version" else "Choose this version",
                onClick = onSelect,
            ),
        shape = TaruShape.medium,
        color = containerColor,
        border = BorderStroke(1.dp, borderColor),
    ) {
        Row(
            modifier = Modifier.padding(TaruSpacing.medium),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            RadioButton(
                selected = model.selected,
                onClick = null,
                enabled = enabled,
            )
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = model.primaryLabel,
                        modifier = Modifier.weight(1f),
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    model.playbackMode?.let { StatusChip(text = it.label) }
                }
                Text(
                    text = model.secondaryText,
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
                ) {
                    model.factLabels.forEach { fact -> StatusChip(text = fact) }
                    if (model.selected) {
                        StatusChip(text = "Selected")
                    }
                }
            }
        }
    }
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

private val serverChangeCategories = setOf(
    PlaybackFailureCategory.MissingAccessToken,
    PlaybackFailureCategory.Unauthorized,
    PlaybackFailureCategory.Forbidden,
    PlaybackFailureCategory.UnsupportedApiVersion,
)
