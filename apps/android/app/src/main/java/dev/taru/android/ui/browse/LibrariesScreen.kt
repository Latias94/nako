package dev.taru.android.ui.browse

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.LibraryDto
import dev.taru.android.ui.artwork.ArtworkRequestResolver
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun LibrariesScreen(
    state: BrowseUiState,
    artworkResolver: ArtworkRequestResolver,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenLibrary: (LibraryDto) -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    TaruScrollColumn {
        PageTitle(
            title = "Libraries",
            subtitle = "Structural entry points for the active server.",
            icon = Icons.AutoMirrored.Rounded.LibraryBooks,
        )

        when (state) {
            BrowseUiState.Loading -> LoadingCard(
                title = "Loading libraries",
                body = "Fetching the active server library list.",
            )
            is BrowseUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is BrowseUiState.Content -> {
                LibraryOverviewCard(
                    libraryCount = state.libraries.libraries.size,
                    itemCount = state.items.page.returned,
                )

                SectionHeader(
                    title = "Media Libraries",
                    action = "${state.libraries.page.returned}",
                )
                if (state.libraries.libraries.isEmpty()) {
                    EmptyCard(
                        title = "No Media Libraries",
                        body = "This server has no visible Media Libraries for the current access token.",
                    )
                } else {
                    LibraryCardRow(
                        libraries = state.libraries.libraries,
                        onOpenLibrary = onOpenLibrary,
                    )
                }

                SectionHeader(
                    title = "Visible Media Items",
                    action = "${state.items.page.returned}",
                )
                if (state.items.items.isEmpty()) {
                    EmptyCard(
                        title = "No visible items",
                        body = "The selected server returned an empty Media Item page.",
                    )
                } else {
                    MediaPosterRow(
                        items = state.items.items,
                        artworkResolver = artworkResolver,
                        artworkByItemId = state.artworkByItemId,
                        onOpenItem = onOpenItem,
                    )
                }
            }
        }
    }
}

@Composable
private fun LibraryOverviewCard(
    libraryCount: Int,
    itemCount: Int,
) {
    SurfaceCard {
        Row(
            modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            IconBadge(icon = Icons.AutoMirrored.Rounded.LibraryBooks)
            Column(
                modifier = androidx.compose.ui.Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = "Browse by library first",
                    style = MaterialTheme.typography.titleLarge,
                )
                Text(
                    text = "Facet pages open only when the Public Client API returns stable relationship ids.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                    StatusChip(text = "$libraryCount libraries")
                    StatusChip(text = "$itemCount items")
                }
            }
        }
    }
}
