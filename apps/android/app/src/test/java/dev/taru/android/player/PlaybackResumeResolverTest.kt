package dev.taru.android.player

import dev.taru.android.userplayback.UserPlaybackStateDto
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PlaybackResumeResolverTest {
    @Test
    fun authoritativeUserPlaybackStateWinsOverDeviceLocalResume() {
        val store = InMemoryDevicePlaybackPositionStore()
        store.save(
            DevicePlaybackPosition(
                key = key(),
                positionMs = 12_000,
                updatedAtMillis = 1,
            ),
        )

        val resume = resolvePlaybackResumePosition(
            profileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            userPlaybackState = state(resumePositionMs = 92_000, sourceId = "source-1"),
            positionStore = store,
        )

        assertEquals(92_000L, resume?.positionMs)
        assertEquals(PlaybackResumeSource.UserPlaybackState, resume?.source)
    }

    @Test
    fun watchedOrDifferentSourceServerStateFallsBackToDeviceLocalResume() {
        val store = InMemoryDevicePlaybackPositionStore()
        store.save(
            DevicePlaybackPosition(
                key = key(),
                positionMs = 12_000,
                updatedAtMillis = 1,
            ),
        )

        val watched = resolvePlaybackResumePosition(
            profileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            userPlaybackState = state(resumePositionMs = null, sourceId = "source-1", watched = true),
            positionStore = store,
        )
        val differentSource = resolvePlaybackResumePosition(
            profileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            userPlaybackState = state(resumePositionMs = 92_000, sourceId = "source-2"),
            positionStore = store,
        )

        assertEquals(12_000L, watched?.positionMs)
        assertEquals(PlaybackResumeSource.DeviceLocal, watched?.source)
        assertEquals(12_000L, differentSource?.positionMs)
        assertEquals(PlaybackResumeSource.DeviceLocal, differentSource?.source)
    }

    @Test
    fun missingServerAndLocalStateHasNoResume() {
        val resume = resolvePlaybackResumePosition(
            profileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            userPlaybackState = null,
            positionStore = InMemoryDevicePlaybackPositionStore(),
        )

        assertNull(resume)
    }

    private fun key(): DevicePlaybackPositionKey =
        DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

    private fun state(
        resumePositionMs: Long?,
        sourceId: String?,
        watched: Boolean = false,
    ): UserPlaybackStateDto =
        UserPlaybackStateDto(
            itemId = "item-1",
            sourceId = sourceId,
            resumePositionMs = resumePositionMs,
            durationMs = 6_360_000,
            progressPercent = 1.44f,
            watched = watched,
            version = 1,
        )
}
