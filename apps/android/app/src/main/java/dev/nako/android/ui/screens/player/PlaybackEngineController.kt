package dev.nako.android.ui.screens.player

import android.content.Context
import androidx.annotation.OptIn
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.datasource.DefaultHttpDataSource
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.hls.HlsMediaSource
import androidx.media3.exoplayer.source.DefaultMediaSourceFactory
import dev.nako.android.player.PlaybackExitSnapshot
import dev.nako.android.player.PlaybackLaunchRequest

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
    accessToken: String,
): PlaybackEngineController {
    val dataSourceFactory = DefaultHttpDataSource.Factory()
        .setDefaultRequestProperties(playbackAuthorizationHeaders(accessToken))
    val mediaSourceFactory = DefaultMediaSourceFactory(context)
        .setDataSourceFactory(dataSourceFactory)
    return Media3PlaybackEngineController(
        player = ExoPlayer.Builder(context)
            .setMediaSourceFactory(mediaSourceFactory)
            .build(),
        accessToken = accessToken,
    )
}

@OptIn(UnstableApi::class)
private class Media3PlaybackEngineController(
    override val player: ExoPlayer,
    private val accessToken: String,
) : PlaybackEngineController {
    override val playbackState: Int
        get() = player.playbackState

    override val isPlaying: Boolean
        get() = player.isPlaying

    override fun prepare(launch: PlaybackLaunchRequest) {
        player.stop()
        player.clearMediaItems()
        val finalRequest = launch.authenticatedRequest(accessToken)
        val mediaItem = MediaItem.fromUri(finalRequest.url)
        if (finalRequest.url.contains("/stream/hls/playlist.m3u8")) {
            val dataSourceFactory = DefaultHttpDataSource.Factory()
                .setDefaultRequestProperties(finalRequest.headers)
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

private fun playbackAuthorizationHeaders(accessToken: String): Map<String, String> =
    if (accessToken.isBlank()) {
        emptyMap()
    } else {
        mapOf("Authorization" to "Bearer $accessToken")
    }
