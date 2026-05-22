package dev.nako.android.ui.components

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
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.nako.android.ui.theme.NakoArtworkAccents
import dev.nako.android.ui.theme.NakoElevation
import dev.nako.android.ui.theme.NakoMotion
import dev.nako.android.ui.theme.NakoScrim
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextMuted
import dev.nako.android.ui.theme.NakoTextSecondary
import dev.nako.android.ui.theme.NakoTouchTarget

internal enum class NakoStateTone {
    Neutral,
    Loading,
    Error,
}

@Composable
internal fun NakoScreenColumn(content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(NakoSpacing.large),
        verticalArrangement = Arrangement.spacedBy(NakoSpacing.large),
        content = content,
    )
}

@Composable
internal fun NakoSectionHeader(
    title: String,
    action: String? = null,
    onAction: (() -> Unit)? = null,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
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
internal fun NakoSurfaceCard(
    modifier: Modifier = Modifier,
    content: @Composable ColumnScope.() -> Unit,
) {
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surface,
        tonalElevation = NakoElevation.raised,
    ) {
        Column(
            modifier = Modifier.padding(NakoSpacing.large),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            content = content,
        )
    }
}

@Composable
internal fun NakoStateCard(
    title: String,
    body: String,
    tone: NakoStateTone = NakoStateTone.Neutral,
    modifier: Modifier = Modifier,
) {
    val transition = rememberInfiniteTransition(label = "nako-state-card")
    val loadingAlpha by transition.animateFloat(
        initialValue = 0.42f,
        targetValue = 0.78f,
        animationSpec = infiniteRepeatable(
            animation = tween(NakoMotion.loadingPulseMillis),
            repeatMode = RepeatMode.Reverse,
        ),
        label = "state-alpha",
    )
    val border = if (tone == NakoStateTone.Error) {
        BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.62f))
    } else {
        null
    }
    Surface(
        modifier = modifier.fillMaxWidth(),
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surface,
        border = border,
        tonalElevation = NakoElevation.raised,
    ) {
        Column(
            modifier = Modifier.padding(NakoSpacing.large),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = body,
                color = NakoTextSecondary.copy(
                    alpha = if (tone == NakoStateTone.Loading) loadingAlpha else 1f,
                ),
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
internal fun NakoIconBadge(
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
internal fun NakoStatusPill(
    text: String,
    icon: ImageVector,
    onClick: (() -> Unit)? = null,
) {
    Surface(
        modifier = Modifier
            .semantics {
                contentDescription = text
                if (onClick != null) {
                    role = Role.Button
                }
            }
            .clickable(enabled = onClick != null) { onClick?.invoke() },
        shape = CircleShape,
        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.16f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.42f)),
    ) {
        Row(
            modifier = Modifier.padding(
                horizontal = NakoSpacing.medium,
                vertical = NakoSpacing.small,
            ),
            horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
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
internal fun NakoStatusChip(text: String) {
    Surface(
        modifier = Modifier.semantics {
            contentDescription = "Status: $text"
        },
        shape = CircleShape,
        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.35f)),
    ) {
        Text(
            modifier = Modifier.padding(
                horizontal = NakoSpacing.medium,
                vertical = NakoSpacing.xsmall,
            ),
            text = text,
            color = MaterialTheme.colorScheme.primary,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}

@Composable
internal fun NakoSettingsRowSurface(
    onClick: (() -> Unit)?,
    content: @Composable () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = onClick != null) { onClick?.invoke() }
            .padding(NakoSpacing.medium),
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(modifier = Modifier.size(NakoTouchTarget.minimum)) {
            content()
        }
    }
}

@Composable
internal fun NakoArtworkBackdrop(
    title: String,
    modifier: Modifier = Modifier,
) {
    val accent = NakoArtworkAccents.fromSeed(title)
    Box(
        modifier = modifier.background(
            Brush.linearGradient(
                colors = listOf(
                    accent.container,
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
                            NakoScrim,
                            MaterialTheme.colorScheme.background.copy(alpha = 0.92f),
                        ),
                    ),
                ),
        )
    }
}

@Composable
internal fun NakoPressableScale(
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    content: @Composable () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val pressed by interactionSource.collectIsPressedAsState()
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.98f else 1f,
        animationSpec = tween(NakoMotion.pressMillis),
        label = "nako-press-scale",
    )
    Box(
        modifier = modifier
            .scale(scale)
            .clickable(
                interactionSource = interactionSource,
                indication = null,
                role = Role.Button,
                onClick = onClick,
            ),
    ) {
        content()
    }
}

@Composable
internal fun NakoMutedText(
    text: String,
    modifier: Modifier = Modifier,
) {
    Text(
        text = text,
        modifier = modifier,
        color = NakoTextMuted,
        style = MaterialTheme.typography.labelMedium,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}
