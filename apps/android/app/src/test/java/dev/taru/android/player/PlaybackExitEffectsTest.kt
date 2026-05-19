package dev.taru.android.player

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.PlaybackResult
import dev.taru.android.playback.TranscodeSessionDto
import dev.taru.android.playback.TranscodeSessionResponse
import dev.taru.android.playback.ClientTranscodeSessionKind
import dev.taru.android.playback.ClientTranscodeSessionState
import dev.taru.android.userplayback.UserPlaybackResult
import dev.taru.android.userplayback.UserPlaybackStateDto
import dev.taru.android.userplayback.UserPlaybackStateResponse
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PlaybackExitEffectsTest {
    @Test
    fun `unfinished session playback saves position reports progress and cancels session`() = runBlocking {
        val store = InMemoryDevicePlaybackPositionStore()
        val calls = PlaybackExitCalls()

        val result = applyPlaybackExitEffects(
            launch = launch(sessionId = "session-1"),
            snapshot = PlaybackExitSnapshot(
                isEnded = false,
                positionMs = 92_000,
                durationMs = 6_360_000,
            ),
            profile = profile(),
            readAccessToken = { "secret-token" },
            positionStore = store,
            updateProgress = { updateProfile, accessToken, itemId, report ->
                calls.progress += PlaybackProgressCall(updateProfile.id, accessToken, itemId, report.request.positionMs)
                userPlaybackSuccess()
            },
            setWatchedState = { _, _, _, _ ->
                calls.watched += Unit
                userPlaybackSuccess()
            },
            cancelPlaybackSession = { cancelProfile, accessToken, sessionId ->
                calls.cancel += PlaybackCancelCall(cancelProfile.id, accessToken, sessionId)
                playbackSessionSuccess(sessionId)
            },
        )

        assertEquals(true, result.savedDevicePosition)
        assertEquals(false, result.clearedDevicePosition)
        assertEquals(true, result.reportedUserPlaybackState)
        assertEquals(true, result.requestedSessionCancellation)
        assertEquals(92_000L, store.load(launch(sessionId = "session-1").positionKey)?.positionMs)
        assertEquals(listOf(PlaybackProgressCall("server-1", "secret-token", "item-1", 92_000)), calls.progress)
        assertEquals(emptyList<Unit>(), calls.watched)
        assertEquals(listOf(PlaybackCancelCall("server-1", "secret-token", "session-1")), calls.cancel)
    }

    @Test
    fun `ended session playback reports watched and does not cancel finished session`() = runBlocking {
        val store = InMemoryDevicePlaybackPositionStore()
        val launch = launch(sessionId = "session-1")
        store.save(
            DevicePlaybackPosition(
                key = launch.positionKey,
                positionMs = 42_000,
                updatedAtMillis = 1,
            ),
        )
        val calls = PlaybackExitCalls()

        val result = applyPlaybackExitEffects(
            launch = launch,
            snapshot = PlaybackExitSnapshot(
                isEnded = true,
                positionMs = 6_350_000,
                durationMs = 6_360_000,
            ),
            profile = profile(),
            readAccessToken = { "secret-token" },
            positionStore = store,
            updateProgress = { _, _, _, _ ->
                calls.progress += PlaybackProgressCall("", "", "", -1)
                userPlaybackSuccess()
            },
            setWatchedState = { watchedProfile, accessToken, itemId, report ->
                calls.watchedState += PlaybackWatchedCall(watchedProfile.id, accessToken, itemId, report.request.watched)
                userPlaybackSuccess()
            },
            cancelPlaybackSession = { _, _, sessionId ->
                calls.cancel += PlaybackCancelCall("", "", sessionId)
                playbackSessionSuccess(sessionId)
            },
        )

        assertEquals(false, result.savedDevicePosition)
        assertEquals(true, result.clearedDevicePosition)
        assertEquals(true, result.reportedUserPlaybackState)
        assertEquals(false, result.requestedSessionCancellation)
        assertNull(store.load(launch.positionKey))
        assertEquals(emptyList<PlaybackProgressCall>(), calls.progress)
        assertEquals(listOf(PlaybackWatchedCall("server-1", "secret-token", "item-1", true)), calls.watchedState)
        assertEquals(emptyList<PlaybackCancelCall>(), calls.cancel)
    }

    @Test
    fun `missing token keeps local position but skips network exit effects`() = runBlocking {
        val store = InMemoryDevicePlaybackPositionStore()
        val calls = PlaybackExitCalls()
        val launch = launch(sessionId = "session-1")

        val result = applyPlaybackExitEffects(
            launch = launch,
            snapshot = PlaybackExitSnapshot(
                isEnded = false,
                positionMs = 10_000,
                durationMs = 6_360_000,
            ),
            profile = profile(),
            readAccessToken = { " " },
            positionStore = store,
            updateProgress = { _, _, _, _ ->
                calls.progress += PlaybackProgressCall("", "", "", -1)
                userPlaybackSuccess()
            },
            setWatchedState = { _, _, _, _ ->
                calls.watched += Unit
                userPlaybackSuccess()
            },
            cancelPlaybackSession = { _, _, sessionId ->
                calls.cancel += PlaybackCancelCall("", "", sessionId)
                playbackSessionSuccess(sessionId)
            },
        )

        assertEquals(true, result.savedDevicePosition)
        assertEquals(false, result.clearedDevicePosition)
        assertEquals(false, result.reportedUserPlaybackState)
        assertEquals(false, result.requestedSessionCancellation)
        assertEquals(10_000L, store.load(launch.positionKey)?.positionMs)
        assertTrue(calls.progress.isEmpty())
        assertTrue(calls.watchedState.isEmpty())
        assertTrue(calls.cancel.isEmpty())
    }

    private fun launch(sessionId: String?): PlaybackLaunchRequest =
        playbackLaunchRequest(
            title = "Night Harbor",
            target = PlaybackRequestTarget(
                request = TaruHttpRequest(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream/remux",
                    headers = mapOf("Authorization" to "Bearer secret-token"),
                ),
                safeRequest = SafeRequestPreview(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream/remux",
                    headers = mapOf("Authorization" to "Bearer <redacted>"),
                ),
                sessionId = sessionId,
            ),
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.Remux,
            sessionId = sessionId,
        )

    private fun profile(): ServerProfile =
        ServerProfile(
            id = "server-1",
            displayName = "Home",
            baseUrl = "http://home.example.test",
            tokenReference = "server-token:server-1",
            lastObservedApiVersion = "v1",
        )

    private fun userPlaybackSuccess(): UserPlaybackResult<UserPlaybackStateResponse> =
        UserPlaybackResult.Success(
            value = UserPlaybackStateResponse(
                state = UserPlaybackStateDto(
                    itemId = "item-1",
                    sourceId = "source-1",
                    resumePositionMs = 92_000,
                    durationMs = 6_360_000,
                    progressPercent = 1.4f,
                    watched = false,
                    watchedAt = null,
                    lastPlayedAt = "2026-05-19T00:00:00Z",
                    updatedAt = "2026-05-19T00:00:00Z",
                    version = 1,
                ),
            ),
            request = SafeRequestPreview(
                method = "PUT",
                url = "http://home.example.test/users/me/playback-state/items/item-1",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        )

    private fun playbackSessionSuccess(sessionId: String): PlaybackResult<TranscodeSessionResponse> =
        PlaybackResult.Success(
            value = TranscodeSessionResponse(
                session = TranscodeSessionDto(
                    id = sessionId,
                    sourceId = "source-1",
                    kind = ClientTranscodeSessionKind.Remux,
                    requestKey = "remux:mkv",
                    state = ClientTranscodeSessionState.CancelRequested,
                    createdAt = "2026-05-19T00:00:00Z",
                    updatedAt = "2026-05-19T00:00:01Z",
                ),
            ),
            request = SafeRequestPreview(
                method = "POST",
                url = "http://home.example.test/playback/sessions/$sessionId/cancel",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        )
}

private data class PlaybackProgressCall(
    val profileId: String,
    val accessToken: String,
    val itemId: String,
    val positionMs: Long,
)

private data class PlaybackWatchedCall(
    val profileId: String,
    val accessToken: String,
    val itemId: String,
    val watched: Boolean,
)

private data class PlaybackCancelCall(
    val profileId: String,
    val accessToken: String,
    val sessionId: String,
)

private class PlaybackExitCalls {
    val progress = mutableListOf<PlaybackProgressCall>()
    val watchedState = mutableListOf<PlaybackWatchedCall>()
    val watched = mutableListOf<Unit>()
    val cancel = mutableListOf<PlaybackCancelCall>()
}
