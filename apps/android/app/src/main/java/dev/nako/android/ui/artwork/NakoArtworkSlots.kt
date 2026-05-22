package dev.nako.android.ui.artwork

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.nako.android.artwork.PublicArtworkRequest
import dev.nako.android.ui.theme.NakoArtworkAccents
import dev.nako.android.ui.theme.NakoScrim
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextSecondary

internal data class NakoArtworkFallbackPresentation(
    val initial: String,
    val kindLabel: String,
    val seed: String,
)

internal fun artworkFallbackPresentation(
    title: String,
    kind: String?,
): NakoArtworkFallbackPresentation {
    val normalizedTitle = title.trim()
    val normalizedKind = kindLabel(kind)
    return NakoArtworkFallbackPresentation(
        initial = normalizedTitle.firstOrNull()?.uppercaseChar()?.toString() ?: "T",
        kindLabel = normalizedKind,
        seed = normalizedTitle.ifBlank { normalizedKind },
    )
}

@Composable
internal fun NakoPosterArtwork(
    request: PublicArtworkRequest?,
    title: String,
    kind: String,
    modifier: Modifier = Modifier,
    compact: Boolean = false,
    border: BorderStroke? = null,
) {
    val presentation = remember(title, kind) { artworkFallbackPresentation(title, kind) }
    val accent = NakoArtworkAccents.fromSeed(presentation.seed)
    Surface(
        modifier = modifier,
        shape = if (compact) NakoShape.small else NakoShape.medium,
        color = accent.container,
        border = border,
    ) {
        NakoArtworkImage(
            request = request,
            contentDescription = title,
            fallback = {
                NakoArtworkPlaceholder(
                    presentation = presentation,
                    compact = compact,
                )
            },
            overlay = {
                NakoPosterArtworkOverlay(
                    presentation = presentation,
                    compact = compact,
                )
            },
        )
    }
}

@Composable
internal fun NakoBackdropArtwork(
    request: PublicArtworkRequest?,
    title: String,
    modifier: Modifier = Modifier,
    overlayColors: List<Color> = listOf(
        Color.Transparent,
        MaterialTheme.colorScheme.background.copy(alpha = 0.62f),
        MaterialTheme.colorScheme.background.copy(alpha = 0.94f),
    ),
) {
    NakoArtworkImage(
        request = request,
        contentDescription = null,
        modifier = modifier,
        fallback = {
            NakoBackdropPlaceholder(
                title = title,
                modifier = Modifier.fillMaxSize(),
            )
        },
        overlay = {
            NakoArtworkGradientOverlay(colors = overlayColors)
        },
    )
}

@Composable
internal fun NakoPlayerBackdrop(
    title: String,
    modifier: Modifier = Modifier,
) {
    Box(modifier = modifier) {
        NakoBackdropPlaceholder(
            title = title,
            modifier = Modifier.fillMaxSize(),
            initialAlpha = 0.08f,
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            NakoScrim.copy(alpha = 0.28f),
                            Color.Transparent,
                            MaterialTheme.colorScheme.background.copy(alpha = 0.86f),
                        ),
                    ),
                ),
        )
    }
}

@Composable
internal fun NakoBackdropPlaceholder(
    title: String,
    modifier: Modifier = Modifier,
    initialAlpha: Float = 0.12f,
) {
    val presentation = remember(title) { artworkFallbackPresentation(title, null) }
    val accent = NakoArtworkAccents.fromSeed(presentation.seed)
    Box(
        modifier = modifier
            .background(
                Brush.linearGradient(
                    colors = listOf(
                        accent.container,
                        MaterialTheme.colorScheme.surfaceVariant,
                        MaterialTheme.colorScheme.background,
                    ),
                ),
            ),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = presentation.initial,
            color = accent.onContainer.copy(alpha = initialAlpha),
            style = MaterialTheme.typography.displayLarge,
        )
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            Color.Transparent,
                            NakoScrim.copy(alpha = 0.56f),
                            MaterialTheme.colorScheme.background.copy(alpha = 0.92f),
                        ),
                    ),
                ),
        )
    }
}

@Composable
private fun NakoArtworkPlaceholder(
    presentation: NakoArtworkFallbackPresentation,
    compact: Boolean,
) {
    val accent = NakoArtworkAccents.fromSeed(presentation.seed)
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.linearGradient(
                    colors = listOf(
                        accent.container,
                        MaterialTheme.colorScheme.surfaceVariant,
                        MaterialTheme.colorScheme.surface,
                    ),
                ),
            ),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = presentation.initial,
            color = accent.onContainer.copy(alpha = if (compact) 0.28f else 0.34f),
            style = if (compact) {
                MaterialTheme.typography.titleLarge
            } else {
                MaterialTheme.typography.displayMedium
            },
        )
    }
}

@Composable
private fun NakoPosterArtworkOverlay(
    presentation: NakoArtworkFallbackPresentation,
    compact: Boolean,
) {
    val accent = NakoArtworkAccents.fromSeed(presentation.seed)
    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    colors = listOf(
                        Color.Transparent,
                        MaterialTheme.colorScheme.background.copy(alpha = 0.54f),
                    ),
                ),
            ),
        contentAlignment = Alignment.BottomStart,
    ) {
        if (!compact) {
            Surface(
                modifier = Modifier.padding(NakoSpacing.small),
                shape = CircleShape,
                color = MaterialTheme.colorScheme.surface.copy(alpha = 0.72f),
                border = BorderStroke(1.dp, accent.outline.copy(alpha = 0.32f)),
            ) {
                Text(
                    modifier = Modifier.padding(
                        horizontal = NakoSpacing.small,
                        vertical = NakoSpacing.xsmall,
                    ),
                    text = presentation.kindLabel,
                    color = NakoTextSecondary,
                    style = MaterialTheme.typography.labelSmall,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

private fun kindLabel(kind: String?): String {
    val normalized = kind
        ?.replace('_', ' ')
        ?.replace('-', ' ')
        ?.trim()
        .orEmpty()
    if (normalized.isBlank()) return "Media"
    if (normalized.equals("media item", ignoreCase = true)) return "Title"
    return normalized
        .split(Regex("\\s+"))
        .joinToString(" ") { word ->
            word.lowercase().replaceFirstChar { first -> first.uppercase() }
        }
}
