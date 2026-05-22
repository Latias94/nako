package dev.nako.android.ui.screens.detail

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
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
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Storage
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.nako.android.artwork.PublicArtworkSlot
import dev.nako.android.artwork.preferredPublicArtwork
import dev.nako.android.browse.ItemDetailResponse
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.browse.MediaSourceDto
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.ResumePlaybackPosition
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.artwork.ArtworkRequestResolver
import dev.nako.android.ui.artwork.NakoBackdropArtwork
import dev.nako.android.ui.artwork.NakoPosterArtwork
import dev.nako.android.ui.browse.BrowseFacetTarget
import dev.nako.android.ui.browse.BrowseFacetUiFamily
import dev.nako.android.ui.browse.FacetChipRow
import dev.nako.android.ui.browse.FailureCard
import dev.nako.android.ui.browse.InfoCard
import dev.nako.android.ui.browse.ItemDetailUiState
import dev.nako.android.ui.browse.LoadingCard
import dev.nako.android.ui.browse.PlaybackSelectionUiState
import dev.nako.android.ui.browse.RelationshipCard
import dev.nako.android.ui.browse.RelationshipRow
import dev.nako.android.ui.browse.SourceProbeUiState
import dev.nako.android.ui.browse.playbackModeLabel
import dev.nako.android.ui.components.NakoIconBadge
import dev.nako.android.ui.components.NakoScreenColumn
import dev.nako.android.ui.components.NakoSectionHeader
import dev.nako.android.ui.components.NakoStatusChip
import dev.nako.android.ui.components.NakoSurfaceCard
import dev.nako.android.ui.screens.sourcepicker.SourcePickerSurface
import dev.nako.android.ui.screens.sourcepicker.selectedSource
import dev.nako.android.ui.theme.NakoAspectRatio
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextSecondary

@Composable
internal fun DetailRouteContent(
    state: ItemDetailUiState,
    sourceProbeState: SourceProbeUiState,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    resumePosition: ResumePlaybackPosition?,
    artworkResolver: ArtworkRequestResolver,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onOpenPersonDetail: (String) -> Unit,
    onSelectSource: (String) -> Unit,
    onRetrySourceProbe: () -> Unit,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    NakoScreenColumn {
        DetailBackButton(onBack = onBack)
        when (state) {
            ItemDetailUiState.Idle,
            ItemDetailUiState.Loading,
            -> LoadingCard(
                title = "Loading title",
                body = "Loading title details and playable versions.",
            )
            is ItemDetailUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is ItemDetailUiState.Content -> MediaItemDetailScreen(
                response = state.response,
                sourceProbeState = sourceProbeState,
                playbackState = playbackState,
                selectedSourceId = selectedSourceId,
                resumePosition = resumePosition,
                artworkResolver = artworkResolver,
                onOpenFacet = onOpenFacet,
                onOpenPersonDetail = onOpenPersonDetail,
                onSelectSource = onSelectSource,
                onRetrySourceProbe = onRetrySourceProbe,
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
            contentDescription = stringResource(NakoStrings.back),
        )
    }
}

@Composable
private fun MediaItemDetailScreen(
    response: ItemDetailResponse,
    sourceProbeState: SourceProbeUiState,
    playbackState: PlaybackSelectionUiState,
    selectedSourceId: String?,
    resumePosition: ResumePlaybackPosition?,
    artworkResolver: ArtworkRequestResolver,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onOpenPersonDetail: (String) -> Unit,
    onSelectSource: (String) -> Unit,
    onRetrySourceProbe: () -> Unit,
    onRequestPlayback: (String) -> Unit,
    onRetryPlayback: () -> Unit,
    onChangeServer: () -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val item = response.item
    val selectedSource = selectedSource(response.sources, selectedSourceId)

    DetailHero(
        response = response,
        item = item,
        selectedSource = selectedSource,
        playbackState = playbackState,
        resumePosition = resumePosition,
        artworkResolver = artworkResolver,
        onRequestPlayback = onRequestPlayback,
        onStartPlayback = onStartPlayback,
    )

    SourcePickerSurface(
        sources = response.sources,
        sourceProbeState = sourceProbeState,
        playbackState = playbackState,
        selectedSourceId = selectedSource?.id,
        resumePosition = resumePosition,
        onSelectSource = onSelectSource,
        onRetrySourceProbe = onRetrySourceProbe,
        onRetryPlayback = onRetryPlayback,
        onChangeServer = onChangeServer,
        onRequestPlayback = onRequestPlayback,
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
        onOpenPersonDetail = onOpenPersonDetail,
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
    response: ItemDetailResponse,
    item: MediaItemDto,
    selectedSource: MediaSourceDto?,
    playbackState: PlaybackSelectionUiState,
    resumePosition: ResumePlaybackPosition?,
    artworkResolver: ArtworkRequestResolver,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val backdropRequest = artworkResolver.requestFor(
        preferredPublicArtwork(response.images, PublicArtworkSlot.Backdrop),
    )
    val posterRequest = artworkResolver.requestFor(
        preferredPublicArtwork(response.images, PublicArtworkSlot.Poster),
    )
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
        tonalElevation = 1.dp,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 360.dp),
        ) {
            NakoBackdropArtwork(
                request = backdropRequest,
                title = item.metadata.title,
                modifier = Modifier.matchParentSize(),
                overlayColors = listOf(
                    Color.Transparent,
                    MaterialTheme.colorScheme.background.copy(alpha = 0.58f),
                    MaterialTheme.colorScheme.background.copy(alpha = 0.96f),
                ),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(NakoSpacing.large),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.large),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.large),
                    verticalAlignment = Alignment.Bottom,
                ) {
                    PosterAnchor(
                        title = item.metadata.title,
                        kind = item.kind,
                        artworkRequest = posterRequest,
                    )
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                    ) {
                        Text(
                            text = item.metadata.title,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 3,
                            overflow = TextOverflow.Ellipsis,
                        )
                        FlowRow(
                            horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                            verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                        ) {
                            detailFactLabels(item).forEach { fact -> NakoStatusChip(text = fact) }
                        }
                    }
                }
                DetailActionCluster(
                    selectedSource = selectedSource,
                    playbackState = playbackState,
                    resumePosition = resumePosition,
                    onRequestPlayback = onRequestPlayback,
                    onStartPlayback = onStartPlayback,
                )
            }
        }
    }
}

@Composable
private fun PosterAnchor(
    title: String,
    kind: String,
    artworkRequest: dev.nako.android.artwork.PublicArtworkRequest?,
) {
    NakoPosterArtwork(
        request = artworkRequest,
        title = title,
        kind = kind,
        modifier = Modifier
            .widthIn(min = 88.dp, max = 118.dp)
            .aspectRatio(NakoAspectRatio.poster),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.28f)),
    )
}

@Composable
private fun DetailActionCluster(
    selectedSource: MediaSourceDto?,
    playbackState: PlaybackSelectionUiState,
    resumePosition: ResumePlaybackPosition?,
    onRequestPlayback: (String) -> Unit,
    onStartPlayback: (PlaybackRequestTarget) -> Unit,
) {
    val preparedTarget = (playbackState as? PlaybackSelectionUiState.Content)
        ?.takeIf { it.response.source.id == selectedSource?.id }
        ?.target
    FlowRow(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
        verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
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
            Spacer(modifier = Modifier.width(NakoSpacing.small))
            Text(if (resumePosition != null) "Resume" else "Play")
        }
        OutlinedButton(
            onClick = { selectedSource?.id?.let(onRequestPlayback) },
            enabled = selectedSource != null && playbackState !is PlaybackSelectionUiState.Loading,
        ) {
            Text("Check version")
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
        PlaybackSelectionUiState.Idle -> if (selectedSource == null) "No version" else "Needs check"
        PlaybackSelectionUiState.Loading -> "Checking"
        is PlaybackSelectionUiState.Content -> if (playbackState.response.source.id == selectedSource?.id) {
            playbackModeLabel(playbackState.response.decision.mode)
        } else {
            "Check version"
        }
        is PlaybackSelectionUiState.Failure -> "Playback issue"
    }
    NakoStatusChip(text = label)
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

    NakoSectionHeader(title = "Metadata")
    NakoSurfaceCard {
        if (metadataTargets.isNotEmpty()) {
            FacetChipRow(
                targets = metadataTargets,
                selected = null,
                onSelected = onOpenFacet,
            )
        }
        if (ratingLabels.isNotEmpty()) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
            ) {
                ratingLabels.forEach { label -> NakoStatusChip(text = label) }
            }
        }
    }
}

@Composable
private fun PeopleSection(
    response: ItemDetailResponse,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onOpenPersonDetail: (String) -> Unit,
) {
    NakoSectionHeader(
        title = "Cast & Crew",
        action = response.credits.size.takeIf { it > 0 }?.toString(),
    )
    DetailRelationshipCard(
        rows = creditRelationshipRows(response),
        onOpenFacet = onOpenFacet,
        onOpenPersonDetail = onOpenPersonDetail,
    )
}

@Composable
private fun RelatedMediaSection(
    response: ItemDetailResponse,
    item: MediaItemDto,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    NakoSectionHeader(title = "Related Media")
    RelationshipCard(
        rows = listOf(
            RelationshipRow(
                title = "Collections",
                subtitle = relatedCollectionsSubtitle(response.collections.size),
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
                    "Studio browsing is not available for this title yet."
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

@Composable
private fun DetailRelationshipCard(
    rows: List<DetailRelationshipRow>,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onOpenPersonDetail: (String) -> Unit,
) {
    NakoSurfaceCard {
        rows.forEach { row ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics {
                        contentDescription = "${row.title}. ${row.subtitle}"
                        role = Role.Button
                    }
                    .clickable {
                        when (val target = row.target) {
                            is DetailRelationshipTarget.Facet -> onOpenFacet(target.target)
                            is DetailRelationshipTarget.PersonDetail -> onOpenPersonDetail(target.personId)
                        }
                    }
                    .padding(vertical = NakoSpacing.small),
                horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                NakoIconBadge(icon = row.icon, compact = true)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = row.title,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Text(
                        text = row.subtitle,
                        color = NakoTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}
