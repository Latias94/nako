package dev.taru.android.ui.browse

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.runtime.Composable
import dev.taru.android.browse.MediaItemDto

@Composable
internal fun LibrariesScreen(
    state: BrowseUiState,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    TaruScrollColumn {
        PageTitle(
            title = "Libraries",
            subtitle = "Browse structural Media Libraries and visible Media Items.",
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
                FacetChipRow(
                    targets = libraryFacetTargets(),
                    selected = libraryFacetTargets().firstOrNull(),
                    onSelected = onOpenFacet,
                )
                if (state.libraries.libraries.isEmpty()) {
                    EmptyCard(
                        title = "No Media Libraries",
                        body = "This server has no visible Media Libraries for the current access token.",
                    )
                } else {
                    state.libraries.libraries.forEach { library ->
                        LibraryListCard(library = library)
                    }
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
                    state.items.items.forEach { item ->
                        MediaItemRow(
                            item = item,
                            onOpenItem = onOpenItem,
                        )
                    }
                }
            }
        }
    }
}

private fun libraryFacetTargets(): List<BrowseFacetTarget> =
    listOf(
        BrowseFacetTarget(BrowseFacetUiFamily.Genre, "Genres"),
        BrowseFacetTarget(BrowseFacetUiFamily.Tag, "Tags"),
        BrowseFacetTarget(BrowseFacetUiFamily.Person, "People"),
        BrowseFacetTarget(BrowseFacetUiFamily.Year, "Years"),
        BrowseFacetTarget(BrowseFacetUiFamily.Collection, "Collections"),
        BrowseFacetTarget(BrowseFacetUiFamily.Studio, "Studios"),
    )
