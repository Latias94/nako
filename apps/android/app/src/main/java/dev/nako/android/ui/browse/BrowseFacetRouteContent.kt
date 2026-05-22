package dev.nako.android.ui.browse

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Category
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.TheaterComedy
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.components.NakoArtworkBackdrop
import dev.nako.android.ui.components.NakoIconBadge
import dev.nako.android.ui.components.NakoScreenColumn
import dev.nako.android.ui.components.NakoSectionHeader
import dev.nako.android.ui.components.NakoStatusChip
import dev.nako.android.ui.components.NakoSurfaceCard
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextSecondary

@Composable
internal fun BrowseFacetRouteContent(
    target: BrowseFacetTarget,
    state: FacetUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onLoadMore: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    NakoScreenColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = stringResource(NakoStrings.back),
            )
        }
        FacetHeader(
            target = target,
            returned = (state as? FacetUiState.Content)?.response?.page?.returned,
        )

        when (state) {
            FacetUiState.Idle,
            FacetUiState.Loading,
            -> LoadingCard(
                title = "Loading facet",
                body = "Loading related titles from your server.",
            )
            is FacetUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is FacetUiState.ApiGap -> EmptyCard(
                title = state.title,
                body = state.body,
            )
            is FacetUiState.Content -> FacetResults(
                target = target,
                state = state,
                onLoadMore = onLoadMore,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun FacetResults(
    target: BrowseFacetTarget,
    state: FacetUiState.Content,
    onLoadMore: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    NakoSectionHeader(
        title = "Related Titles",
        action = "${state.response.page.returned}",
    )
    if (state.response.items.isEmpty()) {
        EmptyCard(
            title = "No related items",
            body = "Your server did not find related titles for this list.",
        )
    } else {
        FacetResultSummary(target = target, state = state)
        MediaPosterRow(
            items = state.response.items,
            onOpenItem = onOpenItem,
        )
        LoadMoreFooter(
            canLoadMore = state.canLoadMore,
            isLoadingMore = state.isLoadingMore,
            failureMessage = state.loadMoreFailure?.userMessage,
            onLoadMore = onLoadMore,
        )
    }
}

@Composable
private fun FacetHeader(
    target: BrowseFacetTarget,
    returned: Int?,
) {
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        shape = NakoShape.medium,
        colors = CardDefaults.elevatedCardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 190.dp),
        ) {
            NakoArtworkBackdrop(
                title = "${target.family.label}:${target.label}",
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(NakoSpacing.large),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    NakoIconBadge(icon = target.family.icon())
                    Column {
                        Text(
                            text = target.label,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = target.family.label,
                            color = NakoTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small)) {
                    NakoStatusChip(text = target.id?.let { "From server" } ?: "Not available")
                    NakoStatusChip(text = returned?.let { "$it results" } ?: "Results")
                }
            }
        }
    }
}

@Composable
private fun FacetResultSummary(
    target: BrowseFacetTarget,
    state: FacetUiState.Content,
) {
    NakoSurfaceCard {
        Text(
            text = "Browsing ${target.family.label.lowercase()} relationship",
            style = MaterialTheme.typography.titleMedium,
        )
        Text(
            text = "Results come from your server. Nako keeps unavailable lists explicit instead of guessing locally.",
            color = NakoTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small)) {
            NakoStatusChip(text = state.response.family.name)
            NakoStatusChip(text = state.response.facetLabel)
        }
    }
}

private fun BrowseFacetUiFamily.icon(): ImageVector =
    when (this) {
        BrowseFacetUiFamily.Genre -> Icons.Rounded.TheaterComedy
        BrowseFacetUiFamily.Tag -> Icons.Rounded.LocalOffer
        BrowseFacetUiFamily.Person -> Icons.Rounded.Person
        BrowseFacetUiFamily.Studio -> Icons.Rounded.Storage
        BrowseFacetUiFamily.Collection -> Icons.AutoMirrored.Rounded.LibraryBooks
        BrowseFacetUiFamily.Year -> Icons.Rounded.Search
        BrowseFacetUiFamily.ItemKind -> Icons.Rounded.Movie
        BrowseFacetUiFamily.Library -> Icons.Rounded.Category
        BrowseFacetUiFamily.SourceMode -> Icons.Rounded.Info
    }
