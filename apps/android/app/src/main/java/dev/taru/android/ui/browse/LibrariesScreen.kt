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
import dev.taru.android.browse.ItemsResponse
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.ui.artwork.ArtworkRequestResolver
import dev.taru.android.ui.components.TaruIconBadge
import dev.taru.android.ui.components.TaruScreenColumn
import dev.taru.android.ui.components.TaruSectionHeader
import dev.taru.android.ui.components.TaruStatusChip
import dev.taru.android.ui.components.TaruSurfaceCard
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
    TaruScreenColumn {
        PageTitle(
            title = "Libraries",
            subtitle = "Browse your server by library.",
            icon = Icons.AutoMirrored.Rounded.LibraryBooks,
        )

        when (state) {
            BrowseUiState.Loading -> LoadingCard(
                title = "Loading libraries",
                body = "Loading the libraries visible to this profile.",
            )
            is BrowseUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is BrowseUiState.Content -> {
                val home = state.home
                LibraryOverviewCard(
                    libraryCount = home.libraries.valueOrNull()?.libraries?.size,
                    itemCount = home.items.valueOrNull()?.page?.returned,
                )

                LibrariesSection(
                    state = home.libraries,
                    onRetry = onRetry,
                    onOpenLibrary = onOpenLibrary,
                )

                LibraryVisibleTitlesSection(
                    state = home.items,
                    artworkResolver = artworkResolver,
                    artworkByItemId = home.artwork.artworkByItemId,
                    onRetry = onRetry,
                    onOpenItem = onOpenItem,
                )
            }
        }
    }
}

@Composable
private fun LibrariesSection(
    state: HomeSectionState<LibraryListResponse>,
    onRetry: () -> Unit,
    onOpenLibrary: (LibraryDto) -> Unit,
) {
    when (state) {
        is HomeSectionState.Available -> {
            TaruSectionHeader(
                title = "Media Libraries",
                action = "${state.value.page.returned}",
            )
            if (state.value.libraries.isEmpty()) {
                EmptyCard(
                    title = "No Media Libraries",
                    body = "This profile does not have any visible libraries yet.",
                )
            } else {
                LibraryCardRow(
                    libraries = state.value.libraries,
                    onOpenLibrary = onOpenLibrary,
                )
            }
        }
        is HomeSectionState.Unavailable -> {
            TaruSectionHeader(
                title = "Media Libraries",
                action = "Retry",
                onAction = onRetry,
            )
            LibrarySectionUnavailableCard(
                title = "Media Libraries unavailable",
                diagnostics = state.diagnostics,
            )
        }
        HomeSectionState.NotRequested -> Unit
    }
}

@Composable
private fun LibraryVisibleTitlesSection(
    state: HomeSectionState<ItemsResponse>,
    artworkResolver: ArtworkRequestResolver,
    artworkByItemId: Map<String, List<PublicImageRefDto>>,
    onRetry: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    when (state) {
        is HomeSectionState.Available -> {
            TaruSectionHeader(
                title = "Visible Titles",
                action = "${state.value.page.returned}",
            )
            if (state.value.items.isEmpty()) {
                EmptyCard(
                    title = "No visible items",
                    body = "This library view is empty right now.",
                )
            } else {
                MediaPosterRow(
                    items = state.value.items,
                    artworkResolver = artworkResolver,
                    artworkByItemId = artworkByItemId,
                    onOpenItem = onOpenItem,
                )
            }
        }
        is HomeSectionState.Unavailable -> {
            TaruSectionHeader(
                title = "Visible Titles",
                action = "Retry",
                onAction = onRetry,
            )
            LibrarySectionUnavailableCard(
                title = "Visible Titles unavailable",
                diagnostics = state.diagnostics,
            )
        }
        HomeSectionState.NotRequested -> Unit
    }
}

@Composable
private fun LibrarySectionUnavailableCard(
    title: String,
    diagnostics: SafeBrowseDiagnostics,
) {
    InfoCard(
        title = title,
        body = diagnostics.userMessage,
    )
}

@Composable
private fun LibraryOverviewCard(
    libraryCount: Int?,
    itemCount: Int?,
) {
    TaruSurfaceCard {
        Row(
            modifier = androidx.compose.ui.Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            TaruIconBadge(icon = Icons.AutoMirrored.Rounded.LibraryBooks)
            Column(
                modifier = androidx.compose.ui.Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = "Browse by library first",
                    style = MaterialTheme.typography.titleLarge,
                )
                Text(
                    text = "Related pages open only when your server shares linkable labels.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                    TaruStatusChip(
                        text = libraryCount?.let { "$it libraries" } ?: "Libraries unavailable",
                    )
                    TaruStatusChip(
                        text = itemCount?.let { "$it items" } ?: "Titles unavailable",
                    )
                }
            }
        }
    }
}
