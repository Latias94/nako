package dev.taru.android.ui.browse

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.LocalOffer
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.TheaterComedy
import androidx.compose.material3.Button
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.artwork.PublicArtworkSlot
import dev.taru.android.artwork.preferredPublicArtwork
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.connection.ServerProfile
import dev.taru.android.userplayback.ContinueWatchingItemDto
import dev.taru.android.ui.artwork.ArtworkRequestResolver
import dev.taru.android.ui.artwork.TaruBackdropArtwork
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun HomeScreen(
    profile: ServerProfile,
    state: BrowseUiState,
    artworkResolver: ArtworkRequestResolver,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenLibrary: () -> Unit,
    onOpenLibraryDetail: (String) -> Unit,
    onOpenSearch: () -> Unit,
    onOpenGenres: () -> Unit,
    onOpenTags: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    TaruScrollColumn {
        val content = state as? BrowseUiState.Content
        HomeHeader(
            profile = profile,
            featuredItem = content?.continueWatching?.items?.firstOrNull()?.item
                ?: content?.items?.items?.firstOrNull(),
            libraryCount = content?.libraries?.libraries?.size,
            itemCount = content?.items?.page?.returned,
            artworkResolver = artworkResolver,
            artworkByItemId = content?.artworkByItemId.orEmpty(),
            onOpenItem = onOpenItem,
            onChangeServer = onChangeServer,
            onOpenLibrary = onOpenLibrary,
            onOpenSearch = onOpenSearch,
        )

        when (state) {
            BrowseUiState.Loading -> LoadingCard(
                title = "Loading library",
                body = "Loading visible libraries and titles.",
            )
            is BrowseUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is BrowseUiState.Content -> {
                HomeAnchorRow(
                    onOpenLibrary = onOpenLibrary,
                    onOpenSearch = onOpenSearch,
                    onOpenGenres = onOpenGenres,
                    onOpenTags = onOpenTags,
                )

                val continueWatchingRows = state.continueWatching
                    ?.items
                    .orEmpty()
                    .filter { !it.state.watched && it.state.resumePositionMs != null }
                if (continueWatchingRows.isNotEmpty()) {
                    SectionHeader(
                        title = "Continue Watching",
                        action = "${continueWatchingRows.size}",
                    )
                    ContinueWatchingPosterRow(
                        rows = continueWatchingRows.take(8),
                        artworkResolver = artworkResolver,
                        artworkByItemId = state.artworkByItemId,
                        onOpenItem = onOpenItem,
                    )
                }

                SectionHeader(
                    title = "Media Libraries",
                    action = "View all",
                    onAction = onOpenLibrary,
                )
                if (state.libraries.libraries.isEmpty()) {
                    EmptyCard(
                        title = "No Media Libraries",
                        body = "This profile does not have any visible libraries yet.",
                    )
                } else {
                    LibraryCardRow(
                        libraries = state.libraries.libraries.take(4),
                        onOpenLibrary = { library -> onOpenLibraryDetail(library.id) },
                    )
                }

                SectionHeader(
                    title = "Visible Titles",
                    action = "${state.items.page.returned}",
                )
                if (state.items.items.isEmpty()) {
                    EmptyCard(
                        title = "No visible items",
                        body = "This profile can see libraries, but there are no visible titles yet.",
                    )
                } else {
                    MediaPosterRow(
                        items = state.items.items.take(8),
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
private fun ContinueWatchingPosterRow(
    rows: List<ContinueWatchingItemDto>,
    artworkResolver: ArtworkRequestResolver,
    artworkByItemId: Map<String, List<dev.taru.android.browse.PublicImageRefDto>>,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        rows.forEach { row ->
            Surface(
                modifier = Modifier.width(132.dp),
                shape = TaruShape.medium,
                color = MaterialTheme.colorScheme.primary.copy(alpha = 0.08f),
                border = androidx.compose.foundation.BorderStroke(
                    1.dp,
                    MaterialTheme.colorScheme.primary.copy(alpha = 0.24f),
                ),
            ) {
                Column(
                    modifier = Modifier.padding(TaruSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                ) {
                    MediaPosterCard(
                        item = row.item,
                        artworkResolver = artworkResolver,
                        artworkRefs = artworkByItemId[row.item.id].orEmpty(),
                        onOpenItem = onOpenItem,
                    )
                    StatusChip(text = continueWatchingProgressLabel(row))
                }
            }
        }
    }
}

@Composable
private fun HomeHeader(
    profile: ServerProfile,
    featuredItem: MediaItemDto?,
    libraryCount: Int?,
    itemCount: Int?,
    artworkResolver: ArtworkRequestResolver,
    artworkByItemId: Map<String, List<dev.taru.android.browse.PublicImageRefDto>>,
    onOpenItem: (MediaItemDto) -> Unit,
    onChangeServer: () -> Unit,
    onOpenLibrary: () -> Unit,
    onOpenSearch: () -> Unit,
) {
    val backdropRequest = artworkResolver.requestFor(
        featuredItem?.let { item ->
            preferredPublicArtwork(artworkByItemId[item.id].orEmpty(), PublicArtworkSlot.Backdrop)
        },
    )
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
                .heightIn(min = 260.dp),
        ) {
            TaruBackdropArtwork(
                request = backdropRequest,
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

                Spacer(modifier = Modifier.height(TaruSpacing.large))

                Text(
                    text = featuredItem?.metadata?.title ?: "Your Taru library",
                    style = MaterialTheme.typography.headlineLarge,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = featuredItem?.let(::itemSecondaryText)
                        ?: "Choose a library, search for a known title, or open the first visible title.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )

                Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                    featuredItem?.let { item ->
                        Button(onClick = { onOpenItem(item) }) {
                            Icon(
                                imageVector = Icons.Rounded.PlayArrow,
                                contentDescription = null,
                            )
                            Spacer(modifier = Modifier.width(TaruSpacing.small))
                            Text("Open detail")
                        }
                    }
                    OutlinedButton(onClick = onOpenLibrary) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Rounded.LibraryBooks,
                            contentDescription = null,
                        )
                        Spacer(modifier = Modifier.width(TaruSpacing.small))
                        Text("Libraries")
                    }
                }

                HomeHeroFacts(
                    libraryCount = libraryCount,
                    itemCount = itemCount,
                )
            }
        }
    }
}

private fun continueWatchingProgressLabel(row: ContinueWatchingItemDto): String =
    row.state.progressPercent
        ?.takeIf { it > 0f }
        ?.let { "%.0f%% watched".format(it.coerceIn(0f, 100f)) }
        ?: row.state.resumePositionMs?.let { "Resume ${durationLabel(it)}" }
        ?: "Resume"

private fun durationLabel(positionMs: Long): String {
    val totalMinutes = positionMs.coerceAtLeast(0L) / 60_000L
    val hours = totalMinutes / 60L
    val minutes = totalMinutes % 60L
    return if (hours > 0) {
        "${hours}h ${minutes}m"
    } else {
        "${minutes}m"
    }
}

@Composable
private fun HomeHeroFacts(
    libraryCount: Int?,
    itemCount: Int?,
) {
    Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
        StatusChip(text = libraryCount?.let { "$it libraries" } ?: "Libraries")
        StatusChip(text = itemCount?.let { "$it visible" } ?: "Visible items")
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun HomeAnchorRow(
    onOpenLibrary: () -> Unit,
    onOpenSearch: () -> Unit,
    onOpenGenres: () -> Unit,
    onOpenTags: () -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        HomeAnchorCard(
            title = "Media Libraries",
            body = "Browse your server by its library structure.",
            icon = Icons.AutoMirrored.Rounded.LibraryBooks,
            action = "Open",
            onClick = onOpenLibrary,
        )
        HomeAnchorCard(
            title = "Search",
            body = "Jump straight to a known title without browsing folders.",
            icon = Icons.Rounded.Search,
            action = "Search",
            onClick = onOpenSearch,
        )
        HomeAnchorCard(
            title = "Genres",
            body = "Browse server-backed genre labels and open related titles.",
            icon = Icons.Rounded.TheaterComedy,
            action = "Browse",
            onClick = onOpenGenres,
        )
        HomeAnchorCard(
            title = "Tags",
            body = "Browse server-backed tag labels and open related titles.",
            icon = Icons.Rounded.LocalOffer,
            action = "Browse",
            onClick = onOpenTags,
        )
    }
}

@Composable
private fun HomeAnchorCard(
    title: String,
    body: String,
    icon: ImageVector,
    action: String,
    onClick: () -> Unit,
) {
    PressableScale(
        modifier = Modifier.width(172.dp),
        onClick = onClick,
    ) {
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surface,
        ) {
            Column(
                modifier = Modifier.padding(TaruSpacing.medium),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                IconBadge(icon = icon, compact = true)
                Text(
                    text = title,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = body,
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 3,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = action,
                    color = MaterialTheme.colorScheme.primary,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
        }
    }
}
