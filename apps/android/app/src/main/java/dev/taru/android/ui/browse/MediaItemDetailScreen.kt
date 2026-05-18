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
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun DetailRouteContent(
    state: ItemDetailUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
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
                onOpenFacet = onOpenFacet,
            )
        }
    }
}

@Composable
private fun MediaItemDetailScreen(
    response: ItemDetailResponse,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    val item = response.item
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
                        onClick = {},
                        enabled = false,
                    ) {
                        Icon(
                            imageVector = Icons.Rounded.PlayArrow,
                            contentDescription = null,
                        )
                        Spacer(modifier = Modifier.width(TaruSpacing.small))
                        Text("Play")
                    }
                    OutlinedButton(
                        onClick = {},
                        enabled = false,
                    ) {
                        Text("Source")
                    }
                }
            }
        }
    }

    SourceSummaryCard(sourceCount = response.sources.size)

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
private fun SourceSummaryCard(sourceCount: Int) {
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
                    text = if (sourceCount > 0) {
                        "$sourceCount Media Source candidate(s). Playback decision arrives in ACF-040."
                    } else {
                        "No playable Media Source is available yet."
                    },
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
            StatusChip(text = if (sourceCount > 0) "ACF-040" else "Unavailable")
        }
    }
}
