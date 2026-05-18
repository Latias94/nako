package dev.taru.android.ui.browse

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.Button
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.connection.ServerProfile
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun HomeScreen(
    profile: ServerProfile,
    state: BrowseUiState,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenLibrary: () -> Unit,
    onOpenSearch: () -> Unit,
    onOpenFacet: (String) -> Unit,
) {
    TaruScrollColumn {
        HomeHeader(
            profile = profile,
            featuredItem = (state as? BrowseUiState.Content)?.items?.items?.firstOrNull(),
            onOpenItem = onOpenItem,
            onChangeServer = onChangeServer,
            onOpenSearch = onOpenSearch,
        )

        when (state) {
            BrowseUiState.Loading -> LoadingCard(
                title = "Loading library",
                body = "Fetching visible Media Libraries and Media Items.",
            )
            is BrowseUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is BrowseUiState.Content -> {
                SectionHeader(
                    title = "Continue Watching",
                    action = "Public API pending",
                )
                ResumePlaceholder()

                SectionHeader(
                    title = "Media Libraries",
                    action = "View all",
                    onAction = onOpenLibrary,
                )
                if (state.libraries.libraries.isEmpty()) {
                    EmptyCard(
                        title = "No Media Libraries",
                        body = "This server has no visible Media Libraries for the current access token.",
                    )
                } else {
                    LibraryCardRow(libraries = state.libraries.libraries.take(4))
                }

                SectionHeader(
                    title = "Recently Added",
                    action = "${state.items.page.returned}",
                )
                if (state.items.items.isEmpty()) {
                    EmptyCard(
                        title = "No visible items",
                        body = "The current access token can see libraries, but no Media Items were returned.",
                    )
                } else {
                    MediaPosterRow(
                        items = state.items.items.take(8),
                        onOpenItem = onOpenItem,
                    )
                }

                SectionHeader(title = "Search shortcuts")
                FacetChipRow(
                    labels = listOf("All", "Movies", "Series", "Mystery", "Direct"),
                    selected = "All",
                    onSelected = onOpenFacet,
                )
            }
        }
    }
}

@Composable
private fun HomeHeader(
    profile: ServerProfile,
    featuredItem: MediaItemDto?,
    onOpenItem: (MediaItemDto) -> Unit,
    onChangeServer: () -> Unit,
    onOpenSearch: () -> Unit,
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
                .heightIn(min = 220.dp),
        ) {
            ArtworkBackdrop(
                title = featuredItem?.metadata?.title ?: "Taru",
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.large),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    StatusPill(
                        text = profile.displayName,
                        icon = Icons.Rounded.Storage,
                        onClick = onChangeServer,
                    )
                    Spacer(modifier = Modifier.weight(1f))
                    IconButton(onClick = onOpenSearch) {
                        Icon(
                            imageVector = Icons.Rounded.Search,
                            contentDescription = "Search",
                        )
                    }
                }

                Spacer(modifier = Modifier.height(TaruSpacing.xlarge))

                Text(
                    text = featuredItem?.metadata?.title ?: "Browse your library",
                    style = MaterialTheme.typography.headlineLarge,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = featuredItem?.let(::itemSecondaryText)
                        ?: "Choose a Media Library or search for a known title.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )

                featuredItem?.let { item ->
                    Button(onClick = { onOpenItem(item) }) {
                        Text("Open detail")
                        Spacer(modifier = Modifier.width(TaruSpacing.small))
                        Icon(
                            imageVector = Icons.Rounded.Search,
                            contentDescription = null,
                        )
                    }
                }
            }
        }
    }
}
