package dev.taru.android.ui.browse

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
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import dev.taru.android.browse.MediaItemDto

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
        PageTitle(
            title = target.label,
            subtitle = target.family.label,
            icon = target.family.icon(),
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
                state = state,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun FacetResults(
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
        state.response.items.forEach { item ->
            MediaItemRow(
                item = item,
                onOpenItem = onOpenItem,
                trailingLabel = state.response.family.name,
            )
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
