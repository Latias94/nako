package dev.taru.android.ui.screens.relationship

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.ChevronRight
import androidx.compose.material.icons.rounded.TheaterComedy
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.ui.browse.ArtworkBackdrop
import dev.taru.android.ui.browse.BrowseFacetTarget
import dev.taru.android.ui.browse.EmptyCard
import dev.taru.android.ui.browse.FailureCard
import dev.taru.android.ui.browse.IconBadge
import dev.taru.android.ui.browse.LoadingCard
import dev.taru.android.ui.browse.RelationshipIndexFamily
import dev.taru.android.ui.browse.RelationshipIndexRow
import dev.taru.android.ui.browse.RelationshipIndexUiState
import dev.taru.android.ui.browse.SectionHeader
import dev.taru.android.ui.browse.StatusChip
import dev.taru.android.ui.browse.TaruScrollColumn
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

internal data class RelationshipIndexPresentation(
    val title: String,
    val subtitle: String,
    val sectionTitle: String,
    val emptyTitle: String,
    val emptyBody: String,
    val resultLabel: String,
    val returnedLabel: String,
    val rows: List<RelationshipIndexRow>,
)

internal fun relationshipIndexPresentation(
    content: RelationshipIndexUiState.Content,
): RelationshipIndexPresentation =
    RelationshipIndexPresentation(
        title = content.family.label,
        subtitle = when (content.family) {
            RelationshipIndexFamily.Genres -> "Server Genres Index"
            RelationshipIndexFamily.Tags -> "Server Tags Index"
        },
        sectionTitle = when (content.family) {
            RelationshipIndexFamily.Genres -> "Browse By Genre"
            RelationshipIndexFamily.Tags -> "Browse By Tag"
        },
        emptyTitle = when (content.family) {
            RelationshipIndexFamily.Genres -> "No Genres"
            RelationshipIndexFamily.Tags -> "No Tags"
        },
        emptyBody = when (content.family) {
            RelationshipIndexFamily.Genres -> "The active server returned no visible Genre labels for this access token."
            RelationshipIndexFamily.Tags -> "The active server returned no visible Tag labels for this access token."
        },
        resultLabel = "${content.rows.size} visible",
        returnedLabel = "${content.page.returned} returned",
        rows = content.rows,
    )

@Composable
internal fun RelationshipIndexRouteContent(
    family: RelationshipIndexFamily,
    state: RelationshipIndexUiState,
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
            RelationshipIndexUiState.Idle,
            RelationshipIndexUiState.Loading,
            -> LoadingCard(
                title = "Loading ${family.label}",
                body = "Fetching server-backed relationship labels.",
            )
            is RelationshipIndexUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is RelationshipIndexUiState.Content -> RelationshipIndexScreen(
                presentation = relationshipIndexPresentation(state),
                onOpenFacet = onOpenFacet,
            )
        }
    }
}

@Composable
private fun RelationshipIndexScreen(
    presentation: RelationshipIndexPresentation,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    RelationshipIndexHeader(presentation = presentation)

    SectionHeader(
        title = presentation.sectionTitle,
        action = presentation.resultLabel,
    )
    if (presentation.rows.isEmpty()) {
        EmptyCard(
            title = presentation.emptyTitle,
            body = presentation.emptyBody,
        )
    } else {
        RelationshipIndexRows(
            rows = presentation.rows,
            onOpenFacet = onOpenFacet,
        )
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun RelationshipIndexHeader(
    presentation: RelationshipIndexPresentation,
) {
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 220.dp),
        ) {
            ArtworkBackdrop(
                title = presentation.title,
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconBadge(icon = Icons.Rounded.TheaterComedy)
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
                    ) {
                        Text(
                            text = presentation.title,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = presentation.subtitle,
                            color = TaruTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                ) {
                    StatusChip(text = "Public API")
                    StatusChip(text = presentation.resultLabel)
                    StatusChip(text = presentation.returnedLabel)
                }
            }
        }
    }
}

@Composable
private fun RelationshipIndexRows(
    rows: List<RelationshipIndexRow>,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    Column(
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
    ) {
        rows.forEachIndexed { index, row ->
            GenreIndexRow(
                row = row,
                index = index,
                onOpenFacet = onOpenFacet,
            )
        }
    }
}

@Composable
private fun GenreIndexRow(
    row: RelationshipIndexRow,
    index: Int,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onOpenFacet(row.target) },
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = if (index == 0) 0.72f else 0.38f),
        border = BorderStroke(
            1.dp,
            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.6f),
        ),
    ) {
        Row(
            modifier = Modifier.padding(TaruSpacing.medium),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.Rounded.TheaterComedy, compact = true)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = row.title,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = row.subtitle,
                    color = TaruTextMuted,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Text(
                text = row.target.id?.let { "API backed" } ?: "API gap",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.labelMedium,
            )
            Icon(
                imageVector = Icons.Rounded.ChevronRight,
                contentDescription = null,
                tint = TaruTextMuted,
            )
        }
    }
}
