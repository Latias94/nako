package dev.nako.android.ui.screens.player

import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.PlaybackLaunchRequest
import dev.nako.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Test

class PlaybackPictureInPictureTest {
    @Test
    fun pictureInPictureRequestUsesStableVideoAspectRatio() {
        val request = playbackPictureInPictureRequest(launch())

        assertEquals(16, request.aspectRatioWidth)
        assertEquals(9, request.aspectRatioHeight)
    }
}

private fun launch(): PlaybackLaunchRequest =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = PlaybackRequestTarget(
            request = PlaybackRequestDescriptor(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream/hls/playlist.m3u8",
            ),
        ),
        serverProfileId = "server-1",
        mediaItemId = "item-1",
        sourceId = "source-1",
        playbackMode = ClientPlaybackMode.Transcode,
        sessionId = "session-1",
    )
