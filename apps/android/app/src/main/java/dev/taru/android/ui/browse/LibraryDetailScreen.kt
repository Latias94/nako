package dev.taru.android.ui.browse

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.LibrarySourceResponse
import dev.taru.android.media.ClientMediaStreamKind
import dev.taru.android.media.MediaProbeDto
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun LibraryDetailRouteContent(
    state: LibraryDetailUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (String) -> Unit,
) {
    TaruScrollColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = "Back",
            )
        }
        when (state) {
            LibraryDetailUiState.Idle,
            LibraryDetailUiState.Loading,
            -> LoadingCard(
                title = "Loading Media Library",
                body = "Fetching library summary and safe Media Source inventory.",
            )
            is LibraryDetailUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is LibraryDetailUiState.Content -> LibraryDetailScreen(
                library = state.response.library,
                sources = state.response.sources,
                returned = state.response.page.returned,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun LibraryDetailScreen(
    library: LibraryDto,
    sources: List<LibrarySourceResponse>,
    returned: Int,
    onOpenItem: (String) -> Unit,
) {
    LibraryDetailHeader(library = library)

    SectionHeader(
        title = "Source inventory",
        action = returned.toString(),
    )
    if (sources.isEmpty()) {
        EmptyCard(
            title = "No Media Sources",
            body = "This library has no visible Media Sources in the current Public Client API page.",
        )
    } else {
        Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium)) {
            sources.forEach { row ->
                LibrarySourceRow(
                    row = row,
                    onOpenItem = onOpenItem,
                )
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun LibraryDetailHeader(library: LibraryDto) {
    SurfaceCard {
        Row(
            modifier = Modifier.fillMaxWidth(),
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
                    style = MaterialTheme.typography.headlineMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = "Public Client API backed Media Library.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                ) {
                    StatusChip(text = librarySubtitle(library))
                    StatusChip(text = "${library.roots.size} root(s)")
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun LibrarySourceRow(
    row: LibrarySourceResponse,
    onOpenItem: (String) -> Unit,
) {
    val item = row.item
    PressableScale(
        onClick = {
            item
                ?.id
                ?.takeIf(String::isNotBlank)
                ?.let(onOpenItem)
        },
    ) {
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surface,
            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline.copy(alpha = 0.18f)),
        ) {
            Row(
                modifier = Modifier.padding(TaruSpacing.medium),
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconBadge(
                    icon = if (item == null) Icons.Rounded.Storage else Icons.Rounded.Info,
                    compact = true,
                )
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
                ) {
                    Text(
                        text = item?.metadata?.title?.takeIf { it.isNotBlank() }
                            ?: row.source.fileName.ifBlank { "Media Source" },
                        style = MaterialTheme.typography.titleMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = row.source.fileName.ifBlank { "File name unavailable" },
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    ) {
                        sourceFactLabels(row).forEach { fact -> StatusChip(text = fact) }
                    }
                    if (item == null) {
                        Text(
                            text = "This source is not linked to a visible Media Item in this response.",
                            color = TaruTextMuted,
                            style = MaterialTheme.typography.labelMedium,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
            }
        }
    }
}

internal fun sourceFactLabels(row: LibrarySourceResponse): List<String> =
    buildList {
        row.source.sizeBytes?.let { add(byteSizeLabel(it)) }
        row.probe?.container?.takeIf { it.isNotBlank() }?.let { add(it.uppercase()) }
        row.probe?.durationMs?.let { add(durationLabel(it)) }
        row.probe?.bitRate?.let { add(bitRateLabel(it)) }
        val video = row.probe?.streams?.firstOrNull { it.width != null && it.height != null }
        video?.let { stream ->
            add(
                listOfNotNull(
                    stream.width?.let { width -> stream.height?.let { height -> "${width}x$height" } },
                    stream.codec,
                ).joinToString(" / "),
            )
        }
        row.probe?.streamCountLabel(ClientMediaStreamKind.Audio, "audio")?.let(::add)
        row.probe?.streamCountLabel(ClientMediaStreamKind.Subtitle, "subtitle")?.let(::add)
    }.filter { it.isNotBlank() }

private fun MediaProbeDto.streamCountLabel(
    kind: ClientMediaStreamKind,
    label: String,
): String? =
    streams
        .count { it.kind == kind }
        .takeIf { it > 0 }
        ?.let { "$it $label" }

private fun durationLabel(durationMs: Long): String {
    val totalMinutes = durationMs.coerceAtLeast(0L) / 60_000L
    val hours = totalMinutes / 60L
    val minutes = totalMinutes % 60L
    return if (hours > 0) {
        "${hours}h ${minutes}m"
    } else {
        "${minutes}m"
    }
}

private fun bitRateLabel(bitRate: Long): String =
    if (bitRate >= 1_000_000L) {
        "${bitRate / 1_000_000L} Mbps"
    } else {
        "${bitRate / 1_000L} Kbps"
    }
