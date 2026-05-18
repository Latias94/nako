package dev.taru.android.ui.browse

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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun BrowseFacetRouteContent(
    target: BrowseFacetTarget,
    state: FacetUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    TaruScrollColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = "Back",
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
                body = "Fetching related Media Items from the active server.",
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
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun FacetResults(
    target: BrowseFacetTarget,
    state: FacetUiState.Content,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    SectionHeader(
        title = "Related Media Items",
        action = "${state.response.page.returned}",
    )
    if (state.response.items.isEmpty()) {
        EmptyCard(
            title = "No related items",
            body = "The active server returned an empty page for this relationship.",
        )
    } else {
        FacetResultSummary(target = target, state = state)
        MediaPosterRow(
            items = state.response.items,
            onOpenItem = onOpenItem,
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
        shape = TaruShape.medium,
        colors = CardDefaults.elevatedCardColors(
            containerColor = MaterialTheme.colorScheme.surfaceVariant,
        ),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 190.dp),
        ) {
            ArtworkBackdrop(
                title = "${target.family.label}:${target.label}",
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            ) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconBadge(icon = target.family.icon())
                    Column {
                        Text(
                            text = target.label,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = target.family.label,
                            color = TaruTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                    StatusChip(text = target.id?.let { "API backed" } ?: "API gap")
                    StatusChip(text = returned?.let { "$it results" } ?: "Results")
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
    SurfaceCard {
        Text(
            text = "Browsing ${target.family.label.lowercase()} relationship",
            style = MaterialTheme.typography.titleMedium,
        )
        Text(
            text = "Results come from the active server relationship route. Unsupported families stay explicit API-gap states instead of local filtering.",
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
        Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
            StatusChip(text = state.response.family.name)
            StatusChip(text = state.response.facetLabel)
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
