package dev.taru.android.ui.screens.player

import android.content.Context
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.hls.HlsMediaSource
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import dev.taru.android.player.PlaybackExitSnapshot
import dev.taru.android.player.PlaybackLaunchRequest

internal interface PlaybackEngineController {
    val player: Player
    val playbackState: Int
    val isPlaying: Boolean
    fun prepare(launch: PlaybackLaunchRequest)
    fun addListener(listener: Player.Listener)
    fun removeListener(listener: Player.Listener)
    fun snapshot(): PlaybackExitSnapshot
    fun release()
}

@OptIn(UnstableApi::class)
internal fun media3PlaybackEngineController(
    context: Context,
    launch: PlaybackLaunchRequest,
): PlaybackEngineController {
    val dataSourceFactory = DefaultHttpDataSource.Factory()
        .setDefaultRequestProperties(launch.request.headers)
    val mediaSourceFactory = DefaultMediaSourceFactory(context)
        .setDataSourceFactory(dataSourceFactory)
    return Media3PlaybackEngineController(
        player = ExoPlayer.Builder(context)
            .setMediaSourceFactory(mediaSourceFactory)
            .build(),
    )
}

@OptIn(UnstableApi::class)
private class Media3PlaybackEngineController(
    override val player: ExoPlayer,
) : PlaybackEngineController {
    override val playbackState: Int
        get() = player.playbackState

    override val isPlaying: Boolean
        get() = player.isPlaying

    override fun prepare(launch: PlaybackLaunchRequest) {
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

    override fun addListener(listener: Player.Listener) {
        player.addListener(listener)
    }

    override fun removeListener(listener: Player.Listener) {
        player.removeListener(listener)
    }

    override fun snapshot(): PlaybackExitSnapshot =
        PlaybackExitSnapshot(
            isEnded = player.playbackState == Player.STATE_ENDED,
            positionMs = player.currentPosition,
            durationMs = player.duration,
        )

    override fun release() {
        player.release()
    }
}
