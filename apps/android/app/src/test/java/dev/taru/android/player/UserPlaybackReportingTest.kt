package dev.taru.android.player

import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestDescriptor
import dev.taru.android.playback.PlaybackRequestTarget
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class UserPlaybackReportingTest {
    @Test
    fun pausedPlaybackReportsProgressWithoutMarkingWatched() {
        val report = userPlaybackStateReport(
            launch = launch(),
            isEnded = false,
            positionMs = 92_000,
            durationMs = 6_360_000,
        )

        assertTrue(report is UserPlaybackStateReport.Progress)
        val progress = report as UserPlaybackStateReport.Progress
        assertEquals("item-1", progress.itemId)
        assertEquals("source-1", progress.request.sourceId)
        assertEquals(92_000L, progress.request.positionMs)
        assertEquals(6_360_000L, progress.request.durationMs)
    }

    @Test
    fun endedPlaybackReportsExplicitWatchedState() {
        val report = userPlaybackStateReport(
            launch = launch(),
            isEnded = true,
            positionMs = 6_350_000,
            durationMs = 6_360_000,
        )

        assertTrue(report is UserPlaybackStateReport.Watched)
        val watched = report as UserPlaybackStateReport.Watched
        assertEquals("item-1", watched.itemId)
        assertEquals(true, watched.request.watched)
        assertEquals("source-1", watched.request.sourceId)
        assertEquals(6_360_000L, watched.request.positionMs)
        assertEquals(6_360_000L, watched.request.durationMs)
    }

    @Test
    fun zeroUnfinishedPositionDoesNotReportServerProgress() {
        val report = userPlaybackStateReport(
            launch = launch(),
            isEnded = false,
            positionMs = 0,
            durationMs = null,
        )

        assertNull(report)
    }

    private fun launch(): PlaybackLaunchRequest =
        playbackLaunchRequest(
            title = "Night Harbor",
            target = PlaybackRequestTarget(
                request = PlaybackRequestDescriptor(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream",
                ),
            ),
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.DirectPlay,
        )
}
