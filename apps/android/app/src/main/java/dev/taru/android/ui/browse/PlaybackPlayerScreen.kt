package dev.taru.android.ui.browse

import android.view.ViewGroup
import androidx.annotation.OptIn
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.graphics.Color
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
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
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
    var playerState by remember { mutableStateOf("Preparing") }
    var playbackError by remember { mutableStateOf<String?>(null) }
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

    DisposableEffect(player) {
        val listener = object : Player.Listener {
            override fun onPlaybackStateChanged(playbackState: Int) {
                playerState = when (playbackState) {
                    Player.STATE_IDLE -> "Idle"
                    Player.STATE_BUFFERING -> "Buffering"
                    Player.STATE_READY -> if (player.playWhenReady) "Playing" else "Ready"
                    Player.STATE_ENDED -> "Ended"
                    else -> "Unknown"
                }
            }

            override fun onPlayerError(error: PlaybackException) {
                playerState = "Error"
                playbackError = error.errorCodeName
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) {
                if (player.playbackState == Player.STATE_READY) {
                    playerState = if (isPlaying) "Playing" else "Paused"
                }
            }
        }
        player.addListener(listener)
        onDispose {
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
            player.removeListener(listener)
            player.release()
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(TaruSpacing.medium),
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
        Row(
            modifier = Modifier
                .align(Alignment.TopStart)
                .fillMaxWidth()
                .padding(TaruSpacing.small),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        ) {
            IconButton(onClick = onBack) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                    contentDescription = "Back",
                    tint = Color.White,
                )
            }
            Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall)) {
                Text(
                    text = launch.title,
                    color = Color.White,
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    text = playbackError?.let { "Playback error: $it" } ?: "Media3: $playerState",
                    color = TaruTextMuted,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
        }
    }
}
