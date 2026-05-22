package dev.nako.android.ui.browse

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
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
import dev.nako.android.artwork.PublicArtworkSlot
import dev.nako.android.artwork.preferredPublicArtwork
import dev.nako.android.browse.LibraryDto
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.browse.PublicImageRefDto
import dev.nako.android.browse.SafeBrowseDiagnostics
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.artwork.ArtworkRequestResolver
import dev.nako.android.ui.artwork.EmptyArtworkRequestResolver
import dev.nako.android.ui.artwork.NakoPosterArtwork
import dev.nako.android.ui.components.NakoIconBadge
import dev.nako.android.ui.components.NakoPressableScale
import dev.nako.android.ui.components.NakoStateCard
import dev.nako.android.ui.components.NakoStateTone
import dev.nako.android.ui.components.NakoStatusChip
import dev.nako.android.ui.components.NakoSurfaceCard
import dev.nako.android.ui.theme.NakoAspectRatio
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextMuted
import dev.nako.android.ui.theme.NakoTextSecondary

@Composable
internal fun PageTitle(
    title: String,
    subtitle: String,
    icon: ImageVector,
    trailing: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        NakoIconBadge(icon = icon)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.headlineLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = subtitle,
                color = NakoTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        trailing?.invoke()
    }
}

@Composable
internal fun SectionLabel(text: String) {
    Text(
        text = text,
        color = NakoTextSecondary,
        style = MaterialTheme.typography.titleMedium,
    )
}

@Composable
internal fun ResumePlaceholder() {
    NakoSurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            NakoIconBadge(icon = Icons.Rounded.PlayArrow)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
            ) {
                Text(
                    text = "Resume data pending",
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    text = "Continue Watching appears after your server shares trusted watch progress.",
                    color = NakoTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
    }
}

@Composable
internal fun InfoCard(
    title: String,
    body: String,
) {
    NakoSurfaceCard {
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
        )
        Text(
            text = body,
            color = NakoTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
internal fun LoadingCard(
    title: String,
    body: String,
) {
    NakoStateCard(
        title = title,
        body = body,
        tone = NakoStateTone.Loading,
    )
}

@Composable
internal fun EmptyCard(
    title: String,
    body: String,
) {
    NakoStateCard(
        title = title,
        body = body,
        tone = NakoStateTone.Neutral,
    )
}

@Composable
internal fun FailureCard(
    diagnostics: SafeBrowseDiagnostics,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.62f)),
    ) {
        Column(
            modifier = Modifier.padding(NakoSpacing.large),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = Icons.Rounded.ErrorOutline,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                )
                Text(
                    text = browseFailureTitle(diagnostics.category),
                    style = MaterialTheme.typography.titleLarge,
                )
            }
            Text(
                text = diagnostics.userMessage,
                color = NakoTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
            diagnostics.publicError?.let { publicError ->
                Text(
                    text = "${publicError.code}: ${publicError.message}",
                    color = NakoTextMuted,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small)) {
                Button(onClick = onRetry) {
                    Text(stringResource(NakoStrings.retry))
                }
                OutlinedButton(onClick = onChangeServer) {
                    Text(stringResource(NakoStrings.changeServer))
                }
            }
        }
    }
}

@Composable
internal fun LoadMoreFooter(
    canLoadMore: Boolean,
    isLoadingMore: Boolean,
    failureMessage: String?,
    onLoadMore: () -> Unit,
) {
    NakoSurfaceCard {
        Text(
            text = if (canLoadMore) {
                "More results may be available"
            } else {
                "You are all caught up"
            },
            style = MaterialTheme.typography.titleMedium,
        )
        Text(
            text = failureMessage
                ?: "Nako loads the next page from your server when one is available.",
            color = NakoTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
        if (canLoadMore || failureMessage != null) {
            Button(
                onClick = onLoadMore,
                enabled = canLoadMore && !isLoadingMore,
            ) {
                Text(
                    if (isLoadingMore) {
                        stringResource(NakoStrings.loadingMore)
                    } else {
                        stringResource(NakoStrings.loadMore)
                    },
                )
            }
        }
    }
}

@Composable
internal fun MediaPosterRow(
    items: List<MediaItemDto>,
    artworkResolver: ArtworkRequestResolver = EmptyArtworkRequestResolver,
    artworkByItemId: Map<String, List<PublicImageRefDto>> = emptyMap(),
    onOpenItem: (MediaItemDto) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
    ) {
        items.forEach { item ->
            MediaPosterCard(
                item = item,
                artworkResolver = artworkResolver,
                artworkRefs = artworkByItemId[item.id].orEmpty(),
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
internal fun MediaPosterCard(
    item: MediaItemDto,
    artworkResolver: ArtworkRequestResolver = EmptyArtworkRequestResolver,
    artworkRefs: List<PublicImageRefDto> = emptyList(),
    onOpenItem: (MediaItemDto) -> Unit,
) {
    val artworkRequest = artworkResolver.requestFor(
        preferredPublicArtwork(artworkRefs, PublicArtworkSlot.Poster),
    )
    NakoPressableScale(
        modifier = Modifier
            .width(116.dp)
            .semantics {
                contentDescription = mediaItemAccessibilityLabel(item)
                role = Role.Button
            },
        onClick = { onOpenItem(item) },
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(NakoSpacing.small)) {
            PosterArtworkSurface(
                item = item,
                artworkRequest = artworkRequest,
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(NakoAspectRatio.poster),
            )
            Text(
                text = item.metadata.title,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = itemSecondaryText(item),
                color = NakoTextMuted,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun MediaItemRow(
    item: MediaItemDto,
    onOpenItem: (MediaItemDto) -> Unit,
    trailingLabel: String = "Direct",
    artworkResolver: ArtworkRequestResolver = EmptyArtworkRequestResolver,
    artworkRefs: List<PublicImageRefDto> = emptyList(),
) {
    val artworkRequest = artworkResolver.requestFor(
        preferredPublicArtwork(artworkRefs, PublicArtworkSlot.Poster),
    )
    NakoPressableScale(
        modifier = Modifier.semantics {
            contentDescription = mediaItemAccessibilityLabel(item)
            role = Role.Button
        },
        onClick = { onOpenItem(item) },
    ) {
        NakoSurfaceCard {
            Row(
                horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                PosterArtworkSurface(
                    item = item,
                    artworkRequest = artworkRequest,
                    modifier = Modifier
                        .width(56.dp)
                        .aspectRatio(NakoAspectRatio.poster),
                    compact = true,
                )
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
                ) {
                    Text(
                        text = item.metadata.title,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = itemSecondaryText(item),
                        color = NakoTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                NakoStatusChip(text = trailingLabel)
            }
        }
    }
}

@Composable
private fun PosterArtworkSurface(
    item: MediaItemDto,
    artworkRequest: dev.nako.android.artwork.PublicArtworkRequest?,
    modifier: Modifier,
    compact: Boolean = false,
) {
    NakoPosterArtwork(
        request = artworkRequest,
        title = item.metadata.title,
        kind = item.kind,
        modifier = modifier,
        compact = compact,
    )
}

private fun mediaItemAccessibilityLabel(item: MediaItemDto): String =
    listOf(
        "Open ${item.metadata.title}",
        itemSecondaryText(item),
    )
        .filter { it.isNotBlank() }
        .joinToString(". ")

@Composable
internal fun LibraryCardRow(
    libraries: List<LibraryDto>,
    onOpenLibrary: ((LibraryDto) -> Unit)? = null,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
    ) {
        libraries.forEach { library ->
            LibraryTile(
                library = library,
                onOpenLibrary = onOpenLibrary,
            )
        }
    }
}

@Composable
internal fun LibraryTile(
    library: LibraryDto,
    onOpenLibrary: ((LibraryDto) -> Unit)? = null,
) {
    if (onOpenLibrary == null) {
        LibraryTileSurface(library = library)
        return
    }

    NakoPressableScale(
        modifier = Modifier
            .width(156.dp)
            .semantics {
                contentDescription = "${library.name}. ${librarySubtitle(library)}"
                role = Role.Button
            },
        onClick = { onOpenLibrary(library) },
    ) {
        LibraryTileSurface(library = library)
    }
}

@Composable
private fun LibraryTileSurface(library: LibraryDto) {
    Surface(
        modifier = Modifier.width(156.dp),
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Column(
            modifier = Modifier.padding(NakoSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
        ) {
            NakoIconBadge(icon = Icons.AutoMirrored.Rounded.LibraryBooks)
            Text(
                text = library.name,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = librarySubtitle(library),
                color = NakoTextSecondary,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun LibraryListCard(library: LibraryDto) {
    NakoSurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            NakoIconBadge(icon = Icons.AutoMirrored.Rounded.LibraryBooks)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
            ) {
                Text(
                    text = library.name,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = librarySubtitle(library),
                    color = NakoTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun FacetChipRow(
    targets: List<BrowseFacetTarget>,
    selected: BrowseFacetTarget? = null,
    onSelected: (BrowseFacetTarget) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
        verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
    ) {
        targets.forEach { target ->
            FilterChip(
                selected = target == selected,
                onClick = { onSelected(target) },
                label = {
                    Text(
                        text = target.label,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                },
            )
        }
    }
}

@Composable
internal fun RelationshipCard(
    rows: List<RelationshipRow>,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    NakoSurfaceCard {
        rows.forEach { row ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics {
                        contentDescription = "${row.title}. ${row.subtitle}"
                        role = Role.Button
                    }
                    .clickable { onOpenFacet(row.target) }
                    .padding(vertical = NakoSpacing.small),
                horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                NakoIconBadge(icon = row.icon, compact = true)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = row.title,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Text(
                        text = row.subtitle,
                        color = NakoTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}
