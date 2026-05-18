package dev.taru.android.ui.screens.player

import android.view.ViewGroup
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.media3.common.MediaItem
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.hls.HlsMediaSource
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import androidx.media3.ui.PlayerView
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPosition
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.ui.browse.IconBadge
import dev.taru.android.ui.browse.StatusChip
import dev.taru.android.ui.theme.TaruAccent
import dev.taru.android.ui.theme.TaruScrim
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch

@Composable
@OptIn(UnstableApi::class)
internal fun PlaybackPlayerRoute(
    launch: PlaybackLaunchRequest,
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    onBack: () -> Unit,
) {
    val context = androidx.compose.ui.platform.LocalContext.current
    val clipboard = LocalClipboardManager.current
    var playerState by remember { mutableStateOf("Preparing") }
    var playbackError by remember { mutableStateOf<PlaybackErrorPresentation?>(null) }
    val chrome = remember(launch) { playerChromePresentation(launch) }
    val player = remember(launch.request.url) {
        val dataSourceFactory = DefaultHttpDataSource.Factory()
            .setDefaultRequestProperties(launch.request.headers)
        val mediaSourceFactory = DefaultMediaSourceFactory(context)
            .setDataSourceFactory(dataSourceFactory)
        ExoPlayer.Builder(context)
            .setMediaSourceFactory(mediaSourceFactory)
            .build()
    }

    LaunchedEffect(player, launch.request.url) {
        playbackError = null
        playerState = "Preparing"
        preparePlayer(player, launch)
    }

    DisposableEffect(player) {
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(playbackState: Int) {
                playerState = playerStateLabel(
                    playbackState = playbackState,
                    isPlaying = player.isPlaying,
                )
            }

            override fun onPlayerError(error: PlaybackException) {
                playerState = "Error"
                playbackError = playbackErrorPresentation(error.errorCodeName, launch)
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) {
                if (player.playbackState == Player.STATE_READY) {
                    playerState = if (isPlaying) "Playing" else "Paused"
                }
            }
        }
        player.addListener(listener)
        onDispose {
            persistPositionAndCancelSession(
                player = player,
                launch = launch,
                profile = profile,
                tokenVault = tokenVault,
                playbackClient = playbackClient,
                positionStore = positionStore,
            )
            player.removeListener(listener)
            player.release()
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color(0xFF05090B)),
    ) {
        AndroidView(
            modifier = Modifier.fillMaxSize(),
            factory = { viewContext ->
                PlayerView(viewContext).apply {
                    this.player = player
                    useController = true
                    layoutParams = ViewGroup.LayoutParams(
                        ViewGroup.LayoutParams.MATCH_PARENT,
                        ViewGroup.LayoutParams.MATCH_PARENT,
                    )
                    setShutterBackgroundColor(android.graphics.Color.BLACK)
                }
            },
            update = { it.player = player },
        )

        PlayerTopOverlay(
            chrome = chrome,
            playerState = playerState,
            onBack = onBack,
        )

        AnimatedVisibility(
            visible = playerState == "Buffering" || playerState == "Preparing",
            modifier = Modifier.align(Alignment.Center),
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            PlayerCenterStatus(playerState = playerState)
        }

        AnimatedVisibility(
            visible = playbackError == null,
            modifier = Modifier.align(Alignment.BottomCenter),
            enter = fadeIn(),
            exit = fadeOut(),
        ) {
            PlayerBottomOverlay(
                chrome = chrome,
                playerState = playerState,
            )
        }

        playbackError?.let { presentation ->
            PlaybackErrorSheet(
                presentation = presentation,
                onRetry = {
                    playbackError = null
                    playerState = "Preparing"
                    preparePlayer(player, launch)
                },
                onBack = onBack,
                onCopyDiagnostics = {
                    clipboard.setText(AnnotatedString(presentation.diagnostics))
                },
                modifier = Modifier.align(Alignment.BottomCenter),
            )
        }
    }
}

@OptIn(UnstableApi::class)
private fun preparePlayer(
    player: ExoPlayer,
    launch: PlaybackLaunchRequest,
) {
    player.stop()
    player.clearMediaItems()
    val mediaItem = MediaItem.fromUri(launch.request.url)
    if (launch.request.url.contains("/stream/hls/playlist.m3u8")) {
        val dataSourceFactory = DefaultHttpDataSource.Factory()
            .setDefaultRequestProperties(launch.request.headers)
        player.setMediaSource(
            HlsMediaSource.Factory(dataSourceFactory).createMediaSource(mediaItem),
        )
    } else {
        player.setMediaItem(mediaItem)
    }
    launch.resumePositionMs
        ?.takeIf { it > 0L }
        ?.let(player::seekTo)
    player.prepare()
    player.playWhenReady = true
}

private fun persistPositionAndCancelSession(
    player: Player,
    launch: PlaybackLaunchRequest,
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
) {
    val isEnded = player.playbackState == Player.STATE_ENDED
    val positionMs = player.currentPosition
    if (isEnded || positionMs <= 0L) {
        positionStore.clear(launch.positionKey)
    } else {
        positionStore.save(
            DevicePlaybackPosition(
                key = launch.positionKey,
                positionMs = positionMs,
                durationMs = player.duration.takeIf { it > 0L },
                updatedAtMillis = System.currentTimeMillis(),
            ),
        )
    }
    val sessionId = launch.sessionId?.takeIf { it.isNotBlank() }
    if (!isEnded && sessionId != null) {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isNotBlank()) {
            CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate).launch {
                playbackClient.cancelPlaybackSession(
                    profile = profile,
                    accessToken = accessToken,
                    sessionId = sessionId,
                )
            }
        }
    }
}

private fun playerStateLabel(
    playbackState: Int,
    isPlaying: Boolean,
): String =
    when (playbackState) {
        Player.STATE_IDLE -> "Idle"
        Player.STATE_BUFFERING -> "Buffering"
        Player.STATE_READY -> if (isPlaying) "Playing" else "Ready"
        Player.STATE_ENDED -> "Ended"
        else -> "Unknown"
    }

@Composable
private fun PlayerTopOverlay(
    chrome: PlayerChromePresentation,
    playerState: String,
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
                contentDescription = "Back",
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
        StatusChip(text = chrome.modeLabel)
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
            IconBadge(icon = Icons.Rounded.PlayArrow, compact = true)
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
                StatusChip(text = playerState)
            }
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                StatusChip(text = chrome.sourceLabel)
                chrome.resumeLabel?.let { StatusChip(text = it) }
                chrome.sessionLabel?.let { StatusChip(text = it) }
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
                IconBadge(icon = Icons.Rounded.ErrorOutline)
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
                Text("Copy diagnostics")
            }
            Spacer(modifier = Modifier.height(TaruSpacing.xsmall))
        }
    }
}
