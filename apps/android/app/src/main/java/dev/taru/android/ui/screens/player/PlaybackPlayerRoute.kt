package dev.taru.android.ui.screens.player

import android.view.ViewGroup
import androidx.activity.compose.BackHandler
import androidx.annotation.OptIn
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.ContentCopy
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.media3.common.util.UnstableApi
import androidx.media3.ui.PlayerView
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.ui.TaruStrings
import dev.taru.android.ui.artwork.TaruPlayerBackdrop
import dev.taru.android.ui.components.TaruIconBadge
import dev.taru.android.ui.components.TaruStatusChip
import dev.taru.android.ui.rememberTaruClipboard
import dev.taru.android.ui.theme.TaruAccent
import dev.taru.android.ui.theme.TaruScrim
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
@OptIn(UnstableApi::class)
internal fun PlaybackPlayerRoute(
    launch: PlaybackLaunchRequest,
    runtimeFactory: PlaybackSessionRuntimeFactory,
    onBack: () -> Unit,
) {
    val clipboard = rememberTaruClipboard()
    val chrome = remember(launch) { playerChromePresentation(launch) }
    val runtime = remember(launch, runtimeFactory) {
        runtimeFactory.create(launch)
    }
    val pictureInPictureGateway = rememberPlaybackPictureInPictureGateway()
    val sessionState by runtime.state.collectAsStateWithLifecycle()

    val handleBack = {
        runtime.back()
        onBack()
    }
    BackHandler(onBack = handleBack)

    DisposableEffect(runtime) {
        runtime.attach()
        onDispose {
            runtime.dispose()
        }
    }
    LaunchedEffect(runtime) {
        runtime.prepare()
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF05090B)),
    ) {
        TaruPlayerBackdrop(
            title = chrome.backdropTitle,
            modifier = Modifier.matchParentSize(),
        )

        AndroidView(
            modifier = Modifier.fillMaxSize(),
            factory = { viewContext ->
                PlayerView(viewContext).apply {
                    this.player = runtime.player
                    useController = true
                    setArtworkDisplayMode(PlayerView.ARTWORK_DISPLAY_MODE_OFF)
                    layoutParams = ViewGroup.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT,
                    )
                    setBackgroundColor(android.graphics.Color.TRANSPARENT)
                    setShutterBackgroundColor(android.graphics.Color.TRANSPARENT)
                }
            },
            update = { it.player = runtime.player },
        )

        PlayerTopOverlay(
            chrome = chrome,
            playerState = sessionState.playerStateLabel,
            canEnterPictureInPicture = pictureInPictureGateway.isAvailable,
            onEnterPictureInPicture = { pictureInPictureGateway.enter(launch) },
            onBack = handleBack,
        )

        AnimatedVisibility(
            visible = sessionState.isPreparingOrBuffering,
            modifier = Modifier.align(Alignment.Center),
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            PlayerCenterStatus(playerState = sessionState.playerStateLabel)
        }

        AnimatedVisibility(
            visible = sessionState.playbackError == null,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = PlayerMedia3ControllerClearanceDp.dp),
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            PlayerBottomOverlay(
                chrome = chrome,
                playerState = sessionState.playerStateLabel,
            )
        }

        sessionState.playbackError?.let { presentation ->
            PlaybackErrorSheet(
                presentation = presentation,
                onRetry = {
                    runtime.retry()
                },
                onBack = handleBack,
                onCopyDiagnostics = {
                    clipboard.copyPlainText("Taru playback diagnostics", presentation.diagnostics)
                },
                modifier = Modifier.align(Alignment.BottomCenter),
            )
        }
    }
}

@Composable
private fun PlayerTopOverlay(
    chrome: PlayerChromePresentation,
    playerState: String,
    canEnterPictureInPicture: Boolean,
    onEnterPictureInPicture: () -> Unit,
    onBack: () -> Unit,
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .background(
                Brush.verticalGradient(
                    colors = listOf(TaruScrim, Color.Transparent),
                ),
            )
            .padding(TaruSpacing.medium),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = stringResource(TaruStrings.back),
                tint = Color.White,
            )
        }
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = chrome.title,
                color = Color.White,
                style = MaterialTheme.typography.titleLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = listOf(chrome.modeLabel, playerState).joinToString(" / "),
                color = TaruTextSecondary,
                style = MaterialTheme.typography.labelMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        if (canEnterPictureInPicture) {
            IconButton(
                modifier = Modifier.semantics {
                    contentDescription = "Enter picture-in-picture"
                },
                onClick = onEnterPictureInPicture,
            ) {
                Text(
                    text = "PiP",
                    color = Color.White,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
        }
        TaruStatusChip(text = chrome.modeLabel)
    }
}

@Composable
private fun PlayerCenterStatus(playerState: String) {
    Surface(
        shape = TaruShape.expressive,
        color = TaruScrim,
        border = BorderStroke(1.dp, TaruAccent.copy(alpha = 0.36f)),
    ) {
        Row(
            modifier = Modifier.padding(
                horizontal = TaruSpacing.large,
                vertical = TaruSpacing.medium,
            ),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            TaruIconBadge(icon = Icons.Rounded.PlayArrow, compact = true)
            Text(
                text = playerState,
                color = Color.White,
                style = MaterialTheme.typography.titleMedium,
            )
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun PlayerBottomOverlay(
    chrome: PlayerChromePresentation,
    playerState: String,
    modifier: Modifier = Modifier,
) {
    val sessionAccessibilityLabel =
        chrome.sessionAccessibilityLabel
            ?: stringResource(TaruStrings.playerSessionActiveAccessibility)
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .padding(TaruSpacing.medium),
        shape = TaruShape.medium,
        color = TaruScrim,
        border = BorderStroke(1.dp, Color.White.copy(alpha = 0.12f)),
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = Icons.Rounded.Subtitles,
                    contentDescription = null,
                    tint = TaruTextSecondary,
                )
                Text(
                    text = "Tracks and subtitles use Media3 controls in this version.",
                    modifier = Modifier.weight(1f),
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                TaruStatusChip(text = playerState)
            }
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                TaruStatusChip(text = chrome.sourceLabel)
                chrome.resumeLabel?.let { TaruStatusChip(text = it) }
                chrome.sessionLabel?.let { label ->
                    Box(
                        modifier = Modifier.semantics {
                            contentDescription = sessionAccessibilityLabel
                        },
                    ) {
                        TaruStatusChip(text = label)
                    }
                }
            }
        }
    }
}

@Composable
private fun PlaybackErrorSheet(
    presentation: PlaybackErrorPresentation,
    onRetry: () -> Unit,
    onBack: () -> Unit,
    onCopyDiagnostics: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .padding(TaruSpacing.medium),
        shape = TaruShape.large,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.58f)),
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                TaruIconBadge(icon = Icons.Rounded.ErrorOutline)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = presentation.title,
                        style = MaterialTheme.typography.titleLarge,
                    )
                    Text(
                        text = presentation.body,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
            Text(
                text = "Diagnostics are sanitized before copying.",
                color = TaruTextMuted,
                style = MaterialTheme.typography.labelMedium,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                Button(onClick = onRetry) {
                    Text(presentation.primaryAction)
                }
                OutlinedButton(onClick = onBack) {
                    Text(presentation.secondaryAction)
                }
            }
            OutlinedButton(onClick = onCopyDiagnostics) {
                Icon(
                    imageVector = Icons.Rounded.ContentCopy,
                    contentDescription = null,
                )
                Spacer(modifier = Modifier.width(TaruSpacing.small))
                Text(stringResource(TaruStrings.copyDiagnostics))
            }
            Spacer(modifier = Modifier.height(TaruSpacing.xsmall))
        }
    }
}
