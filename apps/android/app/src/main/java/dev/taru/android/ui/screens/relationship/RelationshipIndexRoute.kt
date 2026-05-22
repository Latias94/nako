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
import androidx.compose.material.icons.rounded.LocalOffer
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.ui.TaruStrings
import dev.taru.android.ui.browse.BrowseFacetTarget
import dev.taru.android.ui.browse.EmptyCard
import dev.taru.android.ui.browse.FailureCard
import dev.taru.android.ui.browse.LoadMoreFooter
import dev.taru.android.ui.browse.LoadingCard
import dev.taru.android.ui.browse.RelationshipIndexFamily
import dev.taru.android.ui.browse.RelationshipIndexRow
import dev.taru.android.ui.browse.RelationshipIndexUiState
import dev.taru.android.ui.components.TaruArtworkBackdrop
import dev.taru.android.ui.components.TaruIconBadge
import dev.taru.android.ui.components.TaruScreenColumn
import dev.taru.android.ui.components.TaruSectionHeader
import dev.taru.android.ui.components.TaruStatusChip
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

internal data class RelationshipIndexPresentation(
    val title: String,
    val subtitle: String,
    val icon: ImageVector,
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
            RelationshipIndexFamily.Genres -> "Browse by genre"
            RelationshipIndexFamily.Tags -> "Browse by tag"
        },
        icon = relationshipIndexIcon(content.family),
        sectionTitle = when (content.family) {
            RelationshipIndexFamily.Genres -> "Browse By Genre"
            RelationshipIndexFamily.Tags -> "Browse By Tag"
        },
        emptyTitle = when (content.family) {
            RelationshipIndexFamily.Genres -> "No Genres"
            RelationshipIndexFamily.Tags -> "No Tags"
        },
        emptyBody = when (content.family) {
            RelationshipIndexFamily.Genres -> "No genres are visible for this server sign-in."
            RelationshipIndexFamily.Tags -> "No tags are visible for this server sign-in."
        },
        resultLabel = "${content.rows.size} visible",
        returnedLabel = "${content.page.returned} returned",
        rows = content.rows,
    )

private fun relationshipIndexIcon(family: RelationshipIndexFamily): ImageVector =
    when (family) {
        RelationshipIndexFamily.Genres -> Icons.Rounded.TheaterComedy
        RelationshipIndexFamily.Tags -> Icons.Rounded.LocalOffer
    }

@Composable
internal fun RelationshipIndexRouteContent(
    family: RelationshipIndexFamily,
    state: RelationshipIndexUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onLoadMore: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    TaruScreenColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = stringResource(TaruStrings.back),
            )
        }

        when (state) {
            RelationshipIndexUiState.Idle,
            RelationshipIndexUiState.Loading,
            -> LoadingCard(
                title = "Loading ${family.label}",
                body = "Loading labels from your server.",
            )
            is RelationshipIndexUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is RelationshipIndexUiState.Content -> RelationshipIndexScreen(
                presentation = relationshipIndexPresentation(state),
                content = state,
                onLoadMore = onLoadMore,
                onOpenFacet = onOpenFacet,
            )
        }
    }
}

@Composable
private fun RelationshipIndexScreen(
    presentation: RelationshipIndexPresentation,
    content: RelationshipIndexUiState.Content,
    onLoadMore: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    RelationshipIndexHeader(presentation = presentation)

    TaruSectionHeader(
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
            icon = presentation.icon,
            rows = presentation.rows,
            onOpenFacet = onOpenFacet,
        )
        LoadMoreFooter(
            canLoadMore = content.canLoadMore,
            isLoadingMore = content.isLoadingMore,
            failureMessage = content.loadMoreFailure?.userMessage,
            onLoadMore = onLoadMore,
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
            TaruArtworkBackdrop(
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
                    TaruIconBadge(icon = presentation.icon)
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
                    TaruStatusChip(text = "From server")
                    TaruStatusChip(text = presentation.resultLabel)
                    TaruStatusChip(text = presentation.returnedLabel)
                }
            }
        }
    }
}

@Composable
private fun RelationshipIndexRows(
    icon: ImageVector,
    rows: List<RelationshipIndexRow>,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    Column(
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
    ) {
        rows.forEachIndexed { index, row ->
            GenreIndexRow(
                icon = icon,
                row = row,
                index = index,
                onOpenFacet = onOpenFacet,
            )
        }
    }
}

@Composable
private fun GenreIndexRow(
    icon: ImageVector,
    row: RelationshipIndexRow,
    index: Int,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .semantics {
                contentDescription = "${row.title}. ${row.subtitle}"
                role = Role.Button
            }
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
            TaruIconBadge(icon = icon, compact = true)
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
                text = row.target.id?.let { "Open" } ?: "Not available",
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
