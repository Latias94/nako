package dev.taru.android.ui.browse

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.connection.ServerProfile
import dev.taru.android.ui.theme.TaruAccentDim
import dev.taru.android.ui.theme.TaruAspectRatio
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary
import dev.taru.android.ui.theme.TaruTouchTarget

@Composable
internal fun TaruScrollColumn(content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(TaruSpacing.large),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.large),
        content = content,
    )
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
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title,
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.titleLarge,
        )
        if (action != null) {
            TextButton(
                onClick = { onAction?.invoke() },
                enabled = onAction != null,
            ) {
                Text(action)
            }
        }
    }
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
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
        tonalElevation = 1.dp,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            content = content,
        )
    }
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
    val transition = rememberInfiniteTransition(label = "loading-card")
    val alpha by transition.animateFloat(
        initialValue = 0.42f,
        targetValue = 0.78f,
        animationSpec = infiniteRepeatable(
            animation = tween(900),
            repeatMode = RepeatMode.Reverse,
        ),
        label = "loading-alpha",
    )
    SurfaceCard {
        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
        )
        Text(
            text = body,
            color = TaruTextSecondary.copy(alpha = alpha),
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
internal fun EmptyCard(
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
    onOpenItem: (MediaItemDto) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        items.forEach { item ->
            MediaPosterCard(
                item = item,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
internal fun MediaPosterCard(
    item: MediaItemDto,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    PressableScale(
        modifier = Modifier.width(116.dp),
        onClick = { onOpenItem(item) },
    ) {
        Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(TaruAspectRatio.poster),
                shape = TaruShape.medium,
                color = artworkColor(item.metadata.title),
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            Brush.verticalGradient(
                                colors = listOf(
                                    Color.Transparent,
                                    MaterialTheme.colorScheme.background.copy(alpha = 0.68f),
                                ),
                            ),
                        ),
                    contentAlignment = Alignment.BottomStart,
                ) {
                    Text(
                        modifier = Modifier.padding(TaruSpacing.small),
                        text = item.kind,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.labelMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
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
) {
    PressableScale(onClick = { onOpenItem(item) }) {
        SurfaceCard {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Surface(
                    modifier = Modifier
                        .width(56.dp)
                        .aspectRatio(TaruAspectRatio.poster),
                    shape = TaruShape.small,
                    color = artworkColor(item.metadata.title),
                ) {}
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
internal fun LibraryCardRow(libraries: List<LibraryDto>) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        libraries.forEach { library ->
            LibraryTile(library = library)
        }
    }
}

@Composable
internal fun LibraryTile(library: LibraryDto) {
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
internal fun SettingsGroup(
    title: String,
    rows: List<SettingsRow>,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
        SectionLabel(title)
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surface,
        ) {
            Column {
                rows.forEach { row ->
                    SettingsListRow(row)
                }
            }
        }
    }
}

@Composable
internal fun SettingsListRow(row: SettingsRow) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = TaruTouchTarget.minimum)
            .clickable(enabled = row.onClick != null) { row.onClick?.invoke() }
            .padding(TaruSpacing.medium),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = row.icon,
            contentDescription = null,
            tint = TaruTextSecondary,
        )
        Text(
            text = row.label,
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.titleMedium,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        row.value?.let {
            Text(
                text = it,
                color = if (it.equals("None", ignoreCase = true) || it.equals("Connected", ignoreCase = true)) {
                    MaterialTheme.colorScheme.primary
                } else {
                    TaruTextSecondary
                },
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun ServerSummaryCard(
    profile: ServerProfile,
    onClick: (() -> Unit)? = null,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = onClick != null) { onClick?.invoke() },
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.42f)),
    ) {
        Row(
            modifier = Modifier.padding(TaruSpacing.large),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.Rounded.Storage)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = profile.displayName,
                    style = MaterialTheme.typography.titleLarge,
                )
                Text(
                    text = "Connected${profile.lastObservedApiVersion?.let { " / API $it" }.orEmpty()}",
                    color = MaterialTheme.colorScheme.primary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Text(
                    text = profile.lastSuccessfulConnectionAtMillis?.let { "Last successful connection saved" }
                        ?: "No successful connection timestamp",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
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
    Surface(
        modifier = Modifier.clickable(enabled = onClick != null) { onClick?.invoke() },
        shape = CircleShape,
        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.16f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.42f)),
    ) {
        Row(
            modifier = Modifier.padding(
                horizontal = TaruSpacing.medium,
                vertical = TaruSpacing.small,
            ),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(16.dp),
                tint = MaterialTheme.colorScheme.primary,
            )
            Text(
                text = text,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

@Composable
internal fun StatusChip(text: String) {
    Surface(
        shape = CircleShape,
        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.35f)),
    ) {
        Text(
            modifier = Modifier.padding(
                horizontal = TaruSpacing.medium,
                vertical = TaruSpacing.xsmall,
            ),
            text = text,
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}

@Composable
internal fun IconBadge(
    icon: ImageVector,
    compact: Boolean = false,
) {
    val size = if (compact) 34.dp else 52.dp
    Surface(
        modifier = Modifier.size(size),
        shape = CircleShape,
        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.14f),
    ) {
        Box(contentAlignment = Alignment.Center) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.primary,
            )
        }
    }
}

@Composable
internal fun ArtworkBackdrop(
    title: String,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier = modifier.background(
            Brush.linearGradient(
                colors = listOf(
                    artworkColor(title),
                    MaterialTheme.colorScheme.surfaceVariant,
                    MaterialTheme.colorScheme.background,
                ),
            ),
        ),
    ) {
        Box(
            modifier = Modifier
                .matchParentSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            Color.Transparent,
                            MaterialTheme.colorScheme.background.copy(alpha = 0.86f),
                        ),
                    ),
                ),
        )
    }
}

@Composable
internal fun PressableScale(
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    content: @Composable () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val pressed by interactionSource.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.98f else 1f,
        animationSpec = tween(120),
        label = "press-scale",
    )
    Box(
        modifier = modifier
            .scale(scale)
            .clickable(
                interactionSource = interactionSource,
                indication = null,
                onClick = onClick,
            ),
    ) {
        content()
    }
}

@Composable
internal fun artworkColor(seed: String): Color {
    val palette = listOf(
        TaruAccentDim,
        Color(0xFF28465A),
        Color(0xFF3A3E5E),
        Color(0xFF3E4F40),
        Color(0xFF5A4338),
    )
    return palette[kotlin.math.abs(seed.hashCode()) % palette.size]
}
