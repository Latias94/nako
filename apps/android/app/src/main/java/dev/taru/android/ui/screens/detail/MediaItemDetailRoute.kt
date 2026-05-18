package dev.taru.android.ui.screens.detail

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.TheaterComedy
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.ItemCreditDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.ui.browse.ArtworkBackdrop
import dev.taru.android.ui.browse.BrowseFacetTarget
import dev.taru.android.ui.browse.BrowseFacetUiFamily
import dev.taru.android.ui.browse.FacetChipRow
import dev.taru.android.ui.browse.FailureCard
import dev.taru.android.ui.browse.IconBadge
import dev.taru.android.ui.browse.InfoCard
import dev.taru.android.ui.browse.ItemDetailUiState
import dev.taru.android.ui.browse.LoadingCard
import dev.taru.android.ui.browse.PlaybackSelectionUiState
import dev.taru.android.ui.browse.RelationshipCard
import dev.taru.android.ui.browse.RelationshipRow
import dev.taru.android.ui.browse.SectionHeader
import dev.taru.android.ui.browse.StatusChip
import dev.taru.android.ui.browse.SurfaceCard
import dev.taru.android.ui.browse.TaruScrollColumn
import dev.taru.android.ui.browse.itemSecondaryText
import dev.taru.android.ui.browse.playbackModeLabel
import dev.taru.android.ui.screens.sourcepicker.SourcePickerSurface
import dev.taru.android.ui.screens.sourcepicker.selectedSource
import dev.taru.android.ui.theme.TaruAspectRatio
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun DetailRouteContent(
    state: ItemDetailUiState,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    deviceResumePositionMs: Long?,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    TaruScrollColumn {
        DetailBackButton(onBack = onBack)
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
                deviceResumePositionMs = deviceResumePositionMs,
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
private fun DetailBackButton(onBack: () -> Unit) {
    IconButton(onClick = onBack) {
        Icon(
            imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
            contentDescription = "Back",
        )
    }
}

@Composable
private fun MediaItemDetailScreen(
    response: ItemDetailResponse,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    deviceResumePositionMs: Long?,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onRequestPlayback: (String) -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val item = response.item
    val selectedSource = selectedSource(response.sources, selectedSourceId)

    DetailHero(
        item = item,
        selectedSource = selectedSource,
        playbackState = playbackState,
        deviceResumePositionMs = deviceResumePositionMs,
        onRequestPlayback = onRequestPlayback,
        onStartPlayback = onStartPlayback,
    )

    SourcePickerSurface(
        sources = response.sources,
        playbackState = playbackState,
        selectedSourceId = selectedSource?.id,
        deviceResumePositionMs = deviceResumePositionMs,
        onSelectSource = onRequestPlayback,
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

    DetailMetadataSection(
        response = response,
        onOpenFacet = onOpenFacet,
    )

    PeopleSection(
        response = response,
        onOpenFacet = onOpenFacet,
    )

    RelatedMediaSection(
        response = response,
        item = item,
        onOpenFacet = onOpenFacet,
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun DetailHero(
    item: MediaItemDto,
    selectedSource: MediaSourceDto?,
    playbackState: PlaybackSelectionUiState,
    deviceResumePositionMs: Long?,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
        tonalElevation = 1.dp,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 360.dp),
        ) {
            ArtworkBackdrop(
                title = item.metadata.title,
                modifier = Modifier.matchParentSize(),
            )
            Box(
                modifier = Modifier
                    .matchParentSize()
                    .background(
                        Brush.verticalGradient(
                            colors = listOf(
                                Color.Transparent,
                                MaterialTheme.colorScheme.background.copy(alpha = 0.58f),
                                MaterialTheme.colorScheme.background.copy(alpha = 0.96f),
                            ),
                        ),
                    ),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.large),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.large),
                    verticalAlignment = Alignment.Bottom,
                ) {
                    PosterAnchor(title = item.metadata.title)
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                    ) {
                        Text(
                            text = item.metadata.title,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 3,
                            overflow = TextOverflow.Ellipsis,
                        )
                        FlowRow(
                            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                            verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                        ) {
                            detailFactLabels(item).forEach { fact -> StatusChip(text = fact) }
                        }
                    }
                }
                DetailActionCluster(
                    selectedSource = selectedSource,
                    playbackState = playbackState,
                    deviceResumePositionMs = deviceResumePositionMs,
                    onRequestPlayback = onRequestPlayback,
                    onStartPlayback = onStartPlayback,
                )
            }
        }
    }
}

@Composable
private fun PosterAnchor(title: String) {
    Surface(
        modifier = Modifier
            .widthIn(min = 88.dp, max = 118.dp)
            .aspectRatio(TaruAspectRatio.poster),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface.copy(alpha = 0.78f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.28f)),
    ) {
        Box(
            modifier = Modifier
                .clip(TaruShape.medium)
                .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.13f)),
            contentAlignment = Alignment.Center,
        ) {
            Text(
                text = title.trim().take(1).ifBlank { "T" }.uppercase(),
                color = MaterialTheme.colorScheme.primary,
                style = MaterialTheme.typography.headlineMedium,
            )
        }
    }
}

@Composable
private fun DetailActionCluster(
    selectedSource: MediaSourceDto?,
    playbackState: PlaybackSelectionUiState,
    deviceResumePositionMs: Long?,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val preparedTarget = (playbackState as? PlaybackSelectionUiState.Content)
        ?.takeIf { it.response.source.id == selectedSource?.id }
        ?.target
    FlowRow(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
    ) {
        Button(
            onClick = {
                if (preparedTarget != null) {
                    onStartPlayback(preparedTarget)
                } else {
                    selectedSource?.id?.let(onRequestPlayback)
                }
            },
            enabled = selectedSource != null && playbackState !is PlaybackSelectionUiState.Loading,
        ) {
            Icon(
                imageVector = Icons.Rounded.PlayArrow,
                contentDescription = null,
            )
            Spacer(modifier = Modifier.width(TaruSpacing.small))
            Text(if (deviceResumePositionMs != null) "Resume" else "Play")
        }
        OutlinedButton(
            onClick = { selectedSource?.id?.let(onRequestPlayback) },
            enabled = selectedSource != null && playbackState !is PlaybackSelectionUiState.Loading,
        ) {
            Text("Check source")
        }
        PlaybackStatusChip(
            playbackState = playbackState,
            selectedSource = selectedSource,
        )
    }
}

@Composable
private fun PlaybackStatusChip(
    playbackState: PlaybackSelectionUiState,
    selectedSource: MediaSourceDto?,
) {
    val label = when (playbackState) {
        PlaybackSelectionUiState.Idle -> if (selectedSource == null) "No source" else "Needs check"
        PlaybackSelectionUiState.Loading -> "Checking"
        is PlaybackSelectionUiState.Content -> if (playbackState.response.source.id == selectedSource?.id) {
            playbackModeLabel(playbackState.response.decision.mode)
        } else {
            "Check source"
        }
        is PlaybackSelectionUiState.Failure -> "Playback issue"
    }
    StatusChip(text = label)
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun DetailMetadataSection(
    response: ItemDetailResponse,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    val metadataTargets = buildMetadataTargets(response)
    val ratingLabels = response.item.metadata.ratings
        .take(3)
        .map { rating -> "${rating.source}: ${rating.value}" }
    if (metadataTargets.isEmpty() && ratingLabels.isEmpty()) return

    SectionHeader(title = "Metadata")
    SurfaceCard {
        if (metadataTargets.isNotEmpty()) {
            FacetChipRow(
                targets = metadataTargets,
                selected = null,
                onSelected = onOpenFacet,
            )
        }
        if (ratingLabels.isNotEmpty()) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                ratingLabels.forEach { label -> StatusChip(text = label) }
            }
        }
    }
}

@Composable
private fun PeopleSection(
    response: ItemDetailResponse,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    SectionHeader(
        title = "Cast & Crew",
        action = response.credits.size.takeIf { it > 0 }?.toString(),
    )
    RelationshipCard(
        rows = creditRelationshipRows(response),
        onOpenFacet = onOpenFacet,
    )
}

@Composable
private fun RelatedMediaSection(
    response: ItemDetailResponse,
    item: MediaItemDto,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    SectionHeader(title = "Related Media")
    RelationshipCard(
        rows = listOf(
            RelationshipRow(
                title = "Collections",
                subtitle = if (response.collections.isEmpty()) {
                    "Collection browsing needs a Public Client API relationship."
                } else {
                    "${response.collections.size} collection link(s)"
                },
                icon = Icons.AutoMirrored.Rounded.LibraryBooks,
                target = BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Collection,
                    label = "Collections",
                    id = response.collections.firstOrNull()?.collectionId,
                ),
            ),
            RelationshipRow(
                title = "Hierarchy",
                subtitle = hierarchySubtitle(item),
                icon = Icons.Rounded.Movie,
                target = BrowseFacetTarget(BrowseFacetUiFamily.Library, "Hierarchy"),
            ),
            RelationshipRow(
                title = "Studios",
                subtitle = if (response.studios.isEmpty()) {
                    "Studio browsing is not available from this response yet."
                } else {
                    "${response.studios.size} studio link(s)"
                },
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
    val rows = response.credits.take(4).mapIndexed { index, credit ->
        val title = creditTitle(index, credit)
        RelationshipRow(
            title = title,
            subtitle = if (credit.personId.isBlank()) {
                "Person link unavailable for this credit."
            } else {
                "Open related Media Items from this person."
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

private fun detailFactLabels(item: MediaItemDto): List<String> =
    buildList {
        itemSecondaryText(item).takeIf { it.isNotBlank() }?.let { add(it) }
        item.metadata.ratings.firstOrNull()?.let { add(it.value) }
        item.metadata.originalTitle?.takeIf { it.isNotBlank() && it != item.metadata.title }?.let {
            add("Original title available")
        }
        item.parentId?.takeIf { it.isNotBlank() }?.let { add("In hierarchy") }
    }.ifEmpty { listOf(item.kind) }

private fun hierarchySubtitle(item: MediaItemDto): String =
    if (item.parentId.isNullOrBlank()) {
        "Series, season, extras, and parent navigation need explicit API support."
    } else {
        "This item has a parent relationship, but hierarchy browsing is not available yet."
    }

private fun creditTitle(index: Int, credit: ItemCreditDto): String {
    val role = credit.role
        ?.toString()
        ?.trim('"')
        ?.replace('_', ' ')
        ?.takeIf { it.isNotBlank() && it != "null" }
    val character = credit.character?.takeIf { it.isNotBlank() }
    return listOfNotNull(
        role?.replaceFirstChar { it.uppercase() },
        character?.let { "as $it" },
    ).joinToString(" / ").ifBlank { "Credit ${index + 1}" }
}
