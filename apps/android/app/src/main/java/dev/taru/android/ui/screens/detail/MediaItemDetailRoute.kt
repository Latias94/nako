package dev.taru.android.ui.screens.detail

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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.artwork.PublicArtworkSlot
import dev.taru.android.artwork.preferredPublicArtwork
import dev.taru.android.browse.ItemCreditDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.ui.TaruStrings
import dev.taru.android.ui.artwork.ArtworkRequestResolver
import dev.taru.android.ui.artwork.TaruBackdropArtwork
import dev.taru.android.ui.artwork.TaruPosterArtwork
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
import dev.taru.android.ui.browse.SourceProbeUiState
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
    TaruScrollColumn {
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
            contentDescription = stringResource(TaruStrings.back),
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
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
        tonalElevation = 1.dp,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 360.dp),
        ) {
            TaruBackdropArtwork(
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
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.large),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.large),
                    verticalAlignment = Alignment.Bottom,
                ) {
                    PosterAnchor(
                        title = item.metadata.title,
                        kind = item.kind,
                        artworkRequest = posterRequest,
                    )
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
    artworkRequest: dev.taru.android.artwork.PublicArtworkRequest?,
) {
    TaruPosterArtwork(
        request = artworkRequest,
        title = title,
        kind = kind,
        modifier = Modifier
            .widthIn(min = 88.dp, max = 118.dp)
            .aspectRatio(TaruAspectRatio.poster),
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
    onOpenPersonDetail: (String) -> Unit,
) {
    SectionHeader(
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
    SectionHeader(title = "Related Media")
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

@Composable
private fun DetailRelationshipCard(
    rows: List<DetailRelationshipRow>,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onOpenPersonDetail: (String) -> Unit,
) {
    SurfaceCard {
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
                    .padding(vertical = TaruSpacing.small),
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconBadge(icon = row.icon, compact = true)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = row.title,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Text(
                        text = row.subtitle,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}

internal data class DetailRelationshipRow(
    val title: String,
    val subtitle: String,
    val icon: ImageVector,
    val target: DetailRelationshipTarget,
)

internal sealed interface DetailRelationshipTarget {
    data class Facet(val target: BrowseFacetTarget) : DetailRelationshipTarget
    data class PersonDetail(val personId: String) : DetailRelationshipTarget
}

internal fun creditRelationshipRows(response: ItemDetailResponse): List<DetailRelationshipRow> {
    val rows = response.credits.take(4).mapIndexed { index, credit ->
        val title = creditTitle(index, credit)
        val personId = credit.personId.takeIf { it.isNotBlank() }
        DetailRelationshipRow(
            title = title,
            subtitle = if (personId == null) {
                "Person link unavailable for this credit."
            } else {
                "Open this person and related titles."
            },
            icon = Icons.Rounded.Person,
            target = personId
                ?.let(DetailRelationshipTarget::PersonDetail)
                ?: DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(
                        family = BrowseFacetUiFamily.Person,
                        label = title,
                    ),
                ),
        )
    }
    return rows.ifEmpty {
        listOf(
            DetailRelationshipRow(
                title = "Cast",
                subtitle = "Credit names are not available for this item yet.",
                icon = Icons.Rounded.Person,
                target = DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(BrowseFacetUiFamily.Person, "Cast"),
                ),
            ),
            DetailRelationshipRow(
                title = "Director",
                subtitle = "Role-specific browsing is not available yet.",
                icon = Icons.Rounded.TheaterComedy,
                target = DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(BrowseFacetUiFamily.Person, "Director"),
                ),
            ),
            DetailRelationshipRow(
                title = "Writer",
                subtitle = "Role-specific browsing is not available yet.",
                icon = Icons.Rounded.Info,
                target = DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(BrowseFacetUiFamily.Person, "Writer"),
                ),
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

internal fun relatedCollectionsSubtitle(collectionCount: Int): String =
    if (collectionCount <= 0) {
        "More from this collection needs server support."
    } else {
        "$collectionCount collection link(s)"
    }

internal fun hierarchySubtitle(item: MediaItemDto): String =
    if (item.parentId.isNullOrBlank()) {
        "Series and extras browsing needs server support."
    } else {
        "This item belongs to a hierarchy, but browsing it needs server support."
    }

private fun creditTitle(index: Int, credit: ItemCreditDto): String {
    val role = credit.role
        ?.replace('_', ' ')
        ?.takeIf { it.isNotBlank() }
    val character = credit.character?.takeIf { it.isNotBlank() }
    return listOfNotNull(
        role?.replaceFirstChar { it.uppercase() },
        character?.let { "as $it" },
    ).joinToString(" / ").ifBlank { "Credit ${index + 1}" }
}
