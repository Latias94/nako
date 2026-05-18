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
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun DetailRouteContent(
    state: ItemDetailUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenFacet: (String) -> Unit,
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
    onOpenFacet: (String) -> Unit,
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

    val chipLabels = buildList {
        addAll(item.metadata.genres.take(4))
        addAll(item.metadata.tags.take(4))
        item.metadata.releaseDate?.take(4)?.let(::add)
        item.metadata.ratings.firstOrNull()?.value?.let(::add)
    }
    if (chipLabels.isNotEmpty()) {
        SectionHeader(title = "Metadata")
        FacetChipRow(
            labels = chipLabels,
            selected = chipLabels.firstOrNull(),
            onSelected = onOpenFacet,
        )
    }

    SectionHeader(
        title = "Cast & Crew",
        action = "${response.credits.size}",
    )
    RelationshipCard(
        rows = buildList {
            add(RelationshipRow("Actor", "Cast preview", Icons.Rounded.Person))
            add(RelationshipRow("Director", "Browse related Media Items", Icons.Rounded.TheaterComedy))
            add(RelationshipRow("Writer", "Public facet pending", Icons.Rounded.Info))
        },
        onOpenFacet = onOpenFacet,
    )

    SectionHeader(title = "Relationships")
    RelationshipCard(
        rows = listOf(
            RelationshipRow("Franchise Collection", "${response.collections.size} collection links", Icons.AutoMirrored.Rounded.LibraryBooks),
            RelationshipRow("Extras", "Behind the scenes, trailers, interviews", Icons.Rounded.Movie),
            RelationshipRow("Studios", "${response.studios.size} studio links", Icons.Rounded.Storage),
        ),
        onOpenFacet = onOpenFacet,
    )
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
