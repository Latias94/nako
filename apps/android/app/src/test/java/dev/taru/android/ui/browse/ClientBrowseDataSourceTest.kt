package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.playback.InMemoryPlaybackPreferencesStore
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class ClientBrowseDataSourceTest {
    @Test
    fun `detail source probe and playback decision fail safely without access token`() = runBlocking {
        val transport = RecordingTransport()
        val dataSource = ClientBrowseDataSource(
            profile = testProfile(),
            tokenVault = InMemoryTokenVault(),
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            userPlaybackClient = TaruUserPlaybackClient(transport),
        )

        val detailState = dataSource.loadItemDetail("night-harbor")
        val probeState = dataSource.loadSourceProbe("source-a")
        val playbackState = dataSource.loadPlaybackSelection("source-a")

        assertEquals(
            BrowseFailureCategory.MissingAccessToken,
            (detailState as ItemDetailUiState.Failure).diagnostics.category,
        )
        assertEquals(
            PlaybackFailureCategory.MissingAccessToken,
            (probeState as SourceProbeUiState.Failure).diagnostics.category,
        )
        assertEquals(
            PlaybackFailureCategory.MissingAccessToken,
            (playbackState as PlaybackSelectionUiState.Failure).diagnostics.category,
        )
        assertEquals(emptyList<TaruHttpRequest>(), transport.requests)
    }
}

private class RecordingTransport : TaruHttpTransport {
    val requests: MutableList<TaruHttpRequest> = mutableListOf()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return TaruHttpResponse(statusCode = 500)
    }
}

private fun testProfile(): ServerProfile =
    ServerProfile(
        id = "server-1",
        displayName = "Home",
        baseUrl = "http://127.0.0.1:3018",
        tokenReference = "server-token:server-1",
        lastObservedApiVersion = "v1",
    )
