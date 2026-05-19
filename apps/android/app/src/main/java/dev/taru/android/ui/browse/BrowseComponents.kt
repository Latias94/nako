package dev.taru.android.ui.browse

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.artwork.PublicArtworkSlot
import dev.taru.android.artwork.preferredPublicArtwork
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.ui.artwork.ArtworkRequestResolver
import dev.taru.android.ui.artwork.EmptyArtworkRequestResolver
import dev.taru.android.ui.artwork.TaruPosterArtwork
import dev.taru.android.ui.components.TaruArtworkBackdrop as DesignArtworkBackdrop
import dev.taru.android.ui.components.TaruIconBadge as DesignIconBadge
import dev.taru.android.ui.components.TaruPressableScale as DesignPressableScale
import dev.taru.android.ui.components.TaruScreenColumn as DesignScreenColumn
import dev.taru.android.ui.components.TaruSectionHeader as DesignSectionHeader
import dev.taru.android.ui.components.TaruStateCard as DesignStateCard
import dev.taru.android.ui.components.TaruStateTone
import dev.taru.android.ui.components.TaruStatusChip as DesignStatusChip
import dev.taru.android.ui.components.TaruStatusPill as DesignStatusPill
import dev.taru.android.ui.components.TaruSurfaceCard as DesignSurfaceCard
import dev.taru.android.ui.theme.TaruAspectRatio
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun TaruScrollColumn(content: @Composable ColumnScope.() -> Unit) {
    DesignScreenColumn(content = content)
}

@Composable
internal fun PageTitle(
    title: String,
    subtitle: String,
    icon: ImageVector,
    trailing: (@Composable () -> Unit)? = null,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconBadge(icon = icon)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.headlineLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = subtitle,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
        }
        trailing?.invoke()
    }
}

@Composable
internal fun SectionHeader(
    title: String,
    action: String? = null,
    onAction: (() -> Unit)? = null,
) {
    DesignSectionHeader(
        title = title,
        action = action,
        onAction = onAction,
    )
}

@Composable
internal fun SectionLabel(text: String) {
    Text(
        text = text,
        color = TaruTextSecondary,
        style = MaterialTheme.typography.titleMedium,
    )
}

@Composable
internal fun ResumePlaceholder() {
    SurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.Rounded.PlayArrow)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = "Resume data pending",
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    text = "Continue Watching appears only when authoritative User Playback State is available.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
    }
}

@Composable
internal fun SurfaceCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    DesignSurfaceCard(
        modifier = modifier,
        content = content,
    )
}

@Composable
internal fun InfoCard(
    title: String,
    body: String,
) {
    SurfaceCard {
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
        )
        Text(
            text = body,
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
internal fun LoadingCard(
    title: String,
    body: String,
) {
    DesignStateCard(
        title = title,
        body = body,
        tone = TaruStateTone.Loading,
    )
}

@Composable
internal fun EmptyCard(
    title: String,
    body: String,
) {
    DesignStateCard(
        title = title,
        body = body,
        tone = TaruStateTone.Neutral,
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
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.62f)),
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
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
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
            diagnostics.publicError?.let { publicError ->
                Text(
                    text = "${publicError.code}: ${publicError.message}",
                    color = TaruTextMuted,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                Button(onClick = onRetry) {
                    Text("Retry")
                }
                OutlinedButton(onClick = onChangeServer) {
                    Text("Change server")
                }
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
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
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
    PressableScale(
        modifier = Modifier.width(116.dp),
        onClick = { onOpenItem(item) },
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
            PosterArtworkSurface(
                item = item,
                artworkRequest = artworkRequest,
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(TaruAspectRatio.poster),
            )
            Text(
                text = item.metadata.title,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = itemSecondaryText(item),
                color = TaruTextMuted,
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
    PressableScale(onClick = { onOpenItem(item) }) {
        SurfaceCard {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                PosterArtworkSurface(
                    item = item,
                    artworkRequest = artworkRequest,
                    modifier = Modifier
                        .width(56.dp)
                        .aspectRatio(TaruAspectRatio.poster),
                    compact = true,
                )
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
                ) {
                    Text(
                        text = item.metadata.title,
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = itemSecondaryText(item),
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                StatusChip(text = trailingLabel)
            }
        }
    }
}

@Composable
private fun PosterArtworkSurface(
    item: MediaItemDto,
    artworkRequest: dev.taru.android.artwork.PublicArtworkRequest?,
    modifier: Modifier,
    compact: Boolean = false,
) {
    TaruPosterArtwork(
        request = artworkRequest,
        title = item.metadata.title,
        kind = item.kind,
        modifier = modifier,
        compact = compact,
    )
}

@Composable
internal fun LibraryCardRow(
    libraries: List<LibraryDto>,
    onOpenLibrary: ((LibraryDto) -> Unit)? = null,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
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

    PressableScale(
        modifier = Modifier.width(156.dp),
        onClick = { onOpenLibrary(library) },
    ) {
        LibraryTileSurface(library = library)
    }
}

@Composable
private fun LibraryTileSurface(library: LibraryDto) {
    Surface(
        modifier = Modifier.width(156.dp),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        ) {
            IconBadge(icon = Icons.AutoMirrored.Rounded.LibraryBooks)
            Text(
                text = library.name,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = librarySubtitle(library),
                color = TaruTextSecondary,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun LibraryListCard(library: LibraryDto) {
    SurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.AutoMirrored.Rounded.LibraryBooks)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = library.name,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = librarySubtitle(library),
                    color = TaruTextSecondary,
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
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
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
    SurfaceCard {
        rows.forEach { row ->
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onOpenFacet(row.target) }
                    .padding(vertical = TaruSpacing.small),
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconBadge(icon = row.icon, compact = true)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = row.title,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Text(
                        text = row.subtitle,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }
    }
}

@Composable
internal fun StatusPill(
    text: String,
    icon: ImageVector,
    onClick: (() -> Unit)? = null,
) {
    DesignStatusPill(
        text = text,
        icon = icon,
        onClick = onClick,
    )
}

@Composable
internal fun StatusChip(text: String) {
    DesignStatusChip(text = text)
}

@Composable
internal fun IconBadge(
    icon: ImageVector,
    compact: Boolean = false,
) {
    DesignIconBadge(
        icon = icon,
        compact = compact,
    )
}

@Composable
internal fun ArtworkBackdrop(
    title: String,
    modifier: Modifier = Modifier,
) {
    DesignArtworkBackdrop(
        title = title,
        modifier = modifier,
    )
}

@Composable
internal fun PressableScale(
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    content: @Composable () -> Unit,
) {
    DesignPressableScale(
        modifier = modifier,
        onClick = onClick,
        content = content,
    )
}
