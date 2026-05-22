package dev.nako.android.ui.screens.player

import android.content.Context
import android.media.session.MediaSession
import android.media.session.PlaybackState
import androidx.media3.common.Player

internal class AndroidMediaSessionPlayerPlatformSessionFactory(
    private val context: Context,
) : PlayerPlatformSessionFactory {
    override fun create(playerProvider: () -> Player): PlayerPlatformSession =
        AndroidMediaSessionPlayerPlatformSession(
            context = context,
            playerProvider = playerProvider,
        )
}

private class AndroidMediaSessionPlayerPlatformSession(
    context: Context,
    private val playerProvider: () -> Player,
) : PlayerPlatformSession {
    private val session = MediaSession(context, SESSION_TAG).apply {
        setCallback(object : MediaSession.Callback() {
            override fun onPlay() {
                runCatching { playerProvider().play() }
            }

            override fun onPause() {
                runCatching { playerProvider().pause() }
            }

            override fun onStop() {
                runCatching { playerProvider().stop() }
            }

            override fun onSeekTo(pos: Long) {
                runCatching { playerProvider().seekTo(pos) }
            }
        })
        setActive(true)
    }

    override fun onPlaybackStateChanged(playbackState: Int, isPlaying: Boolean) {
        session.setPlaybackState(
            PlaybackState.Builder()
                .setActions(PLAYBACK_ACTIONS)
                .setState(
                    playbackState.toFrameworkPlaybackState(isPlaying),
                    PlaybackState.PLAYBACK_POSITION_UNKNOWN,
                    if (isPlaying) 1f else 0f,
                )
                .build(),
        )
    }

    override fun release() {
        session.setActive(false)
        session.release()
    }

    private companion object {
        const val SESSION_TAG = "NakoPlayback"
        const val PLAYBACK_ACTIONS =
            PlaybackState.ACTION_PLAY or
                PlaybackState.ACTION_PAUSE or
                PlaybackState.ACTION_PLAY_PAUSE or
                PlaybackState.ACTION_STOP or
                PlaybackState.ACTION_SEEK_TO
    }
}

private fun Int.toFrameworkPlaybackState(isPlaying: Boolean): Int =
    when (this) {
        Player.STATE_BUFFERING -> PlaybackState.STATE_BUFFERING
        Player.STATE_READY -> if (isPlaying) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED
        Player.STATE_ENDED -> PlaybackState.STATE_STOPPED
        Player.STATE_IDLE -> PlaybackState.STATE_NONE
        else -> PlaybackState.STATE_NONE
    }
