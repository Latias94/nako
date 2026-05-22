package dev.nako.android.ui.screens.relationship

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
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.browse.BrowseFacetTarget
import dev.nako.android.ui.browse.EmptyCard
import dev.nako.android.ui.browse.FailureCard
import dev.nako.android.ui.browse.LoadMoreFooter
import dev.nako.android.ui.browse.LoadingCard
import dev.nako.android.ui.browse.RelationshipIndexFamily
import dev.nako.android.ui.browse.RelationshipIndexRow
import dev.nako.android.ui.browse.RelationshipIndexUiState
import dev.nako.android.ui.components.NakoArtworkBackdrop
import dev.nako.android.ui.components.NakoIconBadge
import dev.nako.android.ui.components.NakoScreenColumn
import dev.nako.android.ui.components.NakoSectionHeader
import dev.nako.android.ui.components.NakoStatusChip
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextMuted
import dev.nako.android.ui.theme.NakoTextSecondary

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
    NakoScreenColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = stringResource(NakoStrings.back),
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

    NakoSectionHeader(
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
        shape = NakoShape.medium,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 220.dp),
        ) {
            NakoArtworkBackdrop(
                title = presentation.title,
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(NakoSpacing.large),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    NakoIconBadge(icon = presentation.icon)
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
                    ) {
                        Text(
                            text = presentation.title,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = presentation.subtitle,
                            color = NakoTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                ) {
                    NakoStatusChip(text = "From server")
                    NakoStatusChip(text = presentation.resultLabel)
                    NakoStatusChip(text = presentation.returnedLabel)
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
        verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
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
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = if (index == 0) 0.72f else 0.38f),
        border = BorderStroke(
            1.dp,
            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.6f),
        ),
    ) {
        Row(
            modifier = Modifier.padding(NakoSpacing.medium),
            horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            NakoIconBadge(icon = icon, compact = true)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
            ) {
                Text(
                    text = row.title,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = row.subtitle,
                    color = NakoTextMuted,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            Text(
                text = row.target.id?.let { "Open" } ?: "Not available",
                color = NakoTextSecondary,
                style = MaterialTheme.typography.labelMedium,
            )
            Icon(
                imageVector = Icons.Rounded.ChevronRight,
                contentDescription = null,
                tint = NakoTextMuted,
            )
        }
    }
}
