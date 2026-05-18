package dev.taru.android.ui.browse

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.TheaterComedy
import androidx.compose.material3.Button
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.ItemCreditDto
import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.playback.ClientHardwareAcceleration
import dev.taru.android.playback.ClientOutputContainer
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun DetailRouteContent(
    state: ItemDetailUiState,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    TaruScrollColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = "Back",
            )
        }
        when (state) {
            ItemDetailUiState.Idle,
            ItemDetailUiState.Loading,
            -> LoadingCard(
                title = "Loading Media Item",
                body = "Fetching Canonical Metadata and client-safe source facts.",
            )
            is ItemDetailUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is ItemDetailUiState.Content -> MediaItemDetailScreen(
                response = state.response,
                playbackState = playbackState,
                selectedSourceId = selectedSourceId,
                onOpenFacet = onOpenFacet,
                onRequestPlayback = onRequestPlayback,
                onRetryPlayback = onRetryPlayback,
                onChangeServer = onChangeServer,
                onStartPlayback = onStartPlayback,
            )
        }
    }
}

@Composable
private fun MediaItemDetailScreen(
    response: ItemDetailResponse,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onRequestPlayback: (String) -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val item = response.item
    val primarySource = response.sources.firstOrNull()
    val selectedSource = response.sources.firstOrNull { it.id == selectedSourceId } ?: primarySource
    val playbackEnabled = selectedSource != null && playbackState !is PlaybackSelectionUiState.Loading
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        colors = CardDefaults.elevatedCardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 280.dp),
        ) {
            ArtworkBackdrop(
                title = item.metadata.title,
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            ) {
                Text(
                    text = item.metadata.title,
                    style = MaterialTheme.typography.headlineLarge,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = itemSecondaryText(item),
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                    Button(
                        onClick = { selectedSource?.id?.let(onRequestPlayback) },
                        enabled = playbackEnabled,
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.PlayArrow,
                            contentDescription = null,
                        )
                        Spacer(modifier = Modifier.width(TaruSpacing.small))
                        Text("Play")
                    }
                    OutlinedButton(
                        onClick = { selectedSource?.id?.let(onRequestPlayback) },
                        enabled = playbackEnabled,
                    ) {
                        Text("Source")
                    }
                }
            }
        }
    }

    SourceSummaryCard(
        sources = response.sources,
        playbackState = playbackState,
        selectedSourceId = selectedSource?.id,
        onRequestPlayback = onRequestPlayback,
        onRetryPlayback = onRetryPlayback,
        onChangeServer = onChangeServer,
        onStartPlayback = onStartPlayback,
    )

    item.metadata.overview?.takeIf { it.isNotBlank() }?.let { overview ->
        InfoCard(
            title = "Overview",
            body = overview,
        )
    }

    val metadataTargets = buildMetadataTargets(response)
    if (metadataTargets.isNotEmpty()) {
        SectionHeader(title = "Metadata")
        FacetChipRow(
            targets = metadataTargets,
            selected = metadataTargets.firstOrNull(),
            onSelected = onOpenFacet,
        )
    }

    SectionHeader(
        title = "Cast & Crew",
        action = "${response.credits.size}",
    )
    RelationshipCard(
        rows = creditRelationshipRows(response),
        onOpenFacet = onOpenFacet,
    )

    SectionHeader(title = "Relationships")
    RelationshipCard(
        rows = listOf(
            RelationshipRow(
                title = "Collections",
                subtitle = "${response.collections.size} collection link(s)",
                icon = Icons.AutoMirrored.Rounded.LibraryBooks,
                target = BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Collection,
                    label = "Collections",
                    id = response.collections.firstOrNull()?.collectionId,
                ),
            ),
            RelationshipRow(
                title = "Hierarchy",
                subtitle = "Series, season, extras, and parent navigation are not available yet.",
                icon = Icons.Rounded.Movie,
                target = BrowseFacetTarget(BrowseFacetUiFamily.Library, "Hierarchy"),
            ),
            RelationshipRow(
                title = "Studios",
                subtitle = "${response.studios.size} studio link(s)",
                icon = Icons.Rounded.Storage,
                target = BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Studio,
                    label = "Studios",
                    id = response.studios.firstOrNull()?.studioId,
                ),
            ),
        ),
        onOpenFacet = onOpenFacet,
    )
}

private fun buildMetadataTargets(response: ItemDetailResponse): List<BrowseFacetTarget> {
    val item = response.item
    return buildList {
        item.metadata.genres.take(4).forEachIndexed { index, label ->
            add(
                BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Genre,
                    label = label,
                    id = response.genres.getOrNull(index)?.genreId,
                ),
            )
        }
        item.metadata.tags.take(4).forEachIndexed { index, label ->
            add(
                BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Tag,
                    label = label,
                    id = response.tags.getOrNull(index)?.tagId,
                ),
            )
        }
        item.metadata.releaseDate?.take(4)?.let { year ->
            add(BrowseFacetTarget(BrowseFacetUiFamily.Year, year))
        }
        add(BrowseFacetTarget(BrowseFacetUiFamily.ItemKind, item.kind))
    }
}

private fun creditRelationshipRows(response: ItemDetailResponse): List<RelationshipRow> {
    val rows = response.credits.take(3).mapIndexed { index, credit ->
        val title = creditTitle(index, credit)
        RelationshipRow(
            title = title,
            subtitle = if (credit.personId.isBlank()) {
                "Person link unavailable for this credit."
            } else {
                "Browse related Media Items."
            },
            icon = Icons.Rounded.Person,
            target = BrowseFacetTarget(
                family = BrowseFacetUiFamily.Person,
                label = title,
                id = credit.personId,
            ),
        )
    }
    return rows.ifEmpty {
        listOf(
            RelationshipRow(
                title = "Cast",
                subtitle = "Credit names are not available for this item yet.",
                icon = Icons.Rounded.Person,
                target = BrowseFacetTarget(BrowseFacetUiFamily.Person, "Cast"),
            ),
            RelationshipRow(
                title = "Director",
                subtitle = "Role-specific browsing is not available yet.",
                icon = Icons.Rounded.TheaterComedy,
                target = BrowseFacetTarget(BrowseFacetUiFamily.Person, "Director"),
            ),
            RelationshipRow(
                title = "Writer",
                subtitle = "Role-specific browsing is not available yet.",
                icon = Icons.Rounded.Info,
                target = BrowseFacetTarget(BrowseFacetUiFamily.Person, "Writer"),
            ),
        )
    }
}

private fun creditTitle(index: Int, credit: ItemCreditDto): String {
    val role = credit.role
        ?.toString()
        ?.trim('"')
        ?.replace('_', ' ')
        ?.takeIf { it.isNotBlank() && it != "null" }
    return role?.replaceFirstChar { it.uppercase() } ?: "Credit ${index + 1}"
}

@Composable
private fun SourceSummaryCard(
    sources: List<MediaSourceDto>,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    onRequestPlayback: (String) -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val source = sources.firstOrNull { it.id == selectedSourceId } ?: sources.firstOrNull()
    SurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.Rounded.CheckCircle)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = "Playback Source Selection",
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    text = if (source != null) {
                        "${sources.size} Media Source candidate(s). ${source.fileName.ifBlank { "Selected source" }} / ${byteSizeLabel(source.sizeBytes)}"
                    } else {
                        "No playable Media Source is available yet."
                    },
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
            StatusChip(
                text = when (playbackState) {
                    PlaybackSelectionUiState.Idle -> if (source == null) "Unavailable" else "Ready"
                    PlaybackSelectionUiState.Loading -> "Checking"
                    is PlaybackSelectionUiState.Content -> playbackModeLabel(playbackState.response.decision.mode)
                    is PlaybackSelectionUiState.Failure -> "Failed"
                },
            )
        }

        if (source != null) {
            Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                Button(
                    onClick = { onRequestPlayback(source.id) },
                    enabled = playbackState !is PlaybackSelectionUiState.Loading,
                ) {
                    Text("Request decision")
                }
                OutlinedButton(
                    onClick = onRetryPlayback,
                    enabled = playbackState !is PlaybackSelectionUiState.Idle &&
                        playbackState !is PlaybackSelectionUiState.Loading,
                ) {
                    Text("Refresh")
                }
            }
        }

        sources.take(4).forEach { candidate ->
            SourceCandidateRow(
                source = candidate,
                selected = candidate.id == source?.id,
                enabled = playbackState !is PlaybackSelectionUiState.Loading,
                onSelect = { onRequestPlayback(candidate.id) },
            )
        }

        when (playbackState) {
            PlaybackSelectionUiState.Idle -> Unit
            PlaybackSelectionUiState.Loading -> Text(
                text = "Checking Public Client API playback decision.",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
            is PlaybackSelectionUiState.Content -> PlaybackDecisionSummary(
                state = playbackState,
                onStartPlayback = onStartPlayback,
            )
            is PlaybackSelectionUiState.Failure -> PlaybackFailureSummary(
                state = playbackState,
                onRetry = onRetryPlayback,
                onChangeServer = onChangeServer,
            )
        }
    }
}

@Composable
private fun SourceCandidateRow(
    source: MediaSourceDto,
    selected: Boolean,
    enabled: Boolean,
    onSelect: () -> Unit,
) {
    OutlinedButton(
        modifier = Modifier.fillMaxWidth(),
        onClick = onSelect,
        enabled = enabled,
    ) {
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = source.fileName.ifBlank { "Media Source" },
                style = MaterialTheme.typography.labelLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = listOf(
                    source.libraryId.ifBlank { "library unknown" },
                    byteSizeLabel(source.sizeBytes),
                ).joinToString(" / "),
                color = TaruTextSecondary,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        StatusChip(text = if (selected) "Selected" else "Choose")
    }
}

@Composable
private fun PlaybackDecisionSummary(
    state: PlaybackSelectionUiState.Content,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val decision = state.response.decision
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
        Text(
            text = "${playbackModeLabel(decision.mode)} route prepared",
            style = MaterialTheme.typography.titleSmall,
        )
        Text(
            text = decision.reason,
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
        decision.directPlay?.let { direct ->
            Text(
                text = "${direct.contentType} / ranges ${if (direct.supportsRangeRequests) "supported" else "not supported"}",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        decision.transcodePlan?.let { plan ->
            Text(
                text = listOfNotNull(
                    outputContainerLabel(plan.outputContainer),
                    plan.videoCodec?.let { "video $it" },
                    plan.audioCodec?.let { "audio $it" },
                    hardwareLabel(plan.hardwareAcceleration),
                ).joinToString(" / "),
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
        state.target?.safeRequest?.let { request ->
            Text(
                text = "${request.method} ${request.url}",
                color = TaruTextMuted,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 3,
                overflow = TextOverflow.Ellipsis,
            )
        }
        state.target?.let { target ->
            Button(onClick = { onStartPlayback(target) }) {
                Icon(
                    imageVector = Icons.Rounded.PlayArrow,
                    contentDescription = null,
                )
                Spacer(modifier = Modifier.width(TaruSpacing.small))
                Text("Start playback")
            }
        } ?: Text(
            text = "No playable route was prepared.",
            color = TaruTextMuted,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}

@Composable
private fun PlaybackFailureSummary(
    state: PlaybackSelectionUiState.Failure,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
        Text(
            text = playbackFailureTitle(state.diagnostics.category),
            style = MaterialTheme.typography.titleSmall,
        )
        Text(
            text = state.diagnostics.userMessage,
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
        state.diagnostics.publicError?.let { publicError ->
            Text(
                text = "${publicError.code}: ${publicError.message}",
                color = TaruTextMuted,
                style = MaterialTheme.typography.labelMedium,
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
            Button(onClick = onRetry) {
                Text("Retry")
            }
            OutlinedButton(onClick = onChangeServer) {
                Text("Change server")
            }
        }
    }
}

private fun outputContainerLabel(container: ClientOutputContainer): String =
    when (container) {
        ClientOutputContainer.Hls -> "HLS"
        ClientOutputContainer.Mp4 -> "MP4"
        ClientOutputContainer.Mkv -> "MKV"
    }

private fun hardwareLabel(hardware: ClientHardwareAcceleration): String =
    when (hardware) {
        ClientHardwareAcceleration.None -> "CPU"
        ClientHardwareAcceleration.Vaapi -> "VAAPI"
        ClientHardwareAcceleration.Nvenc -> "NVENC"
        ClientHardwareAcceleration.QuickSync -> "Quick Sync"
    }
