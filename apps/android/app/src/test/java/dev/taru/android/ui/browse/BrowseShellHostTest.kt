package dev.taru.android.ui.browse

import dev.taru.android.browse.CanonicalMetadataDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.ItemsResponse
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.LibrarySourcesResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.SearchResponse
import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.media.MediaProbeDto
import dev.taru.android.media.SourceProbeResponse
import dev.taru.android.playback.ClientPlaybackDecision
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackCapabilities
import dev.taru.android.playback.PlaybackDecisionResponse
import dev.taru.android.playback.PlaybackMediaSourceDto
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.PlaybackStartRequest
import dev.taru.android.playback.PlaybackStartResult
import dev.taru.android.player.PlaybackResumeSource
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.player.playbackLaunchRequest
import dev.taru.android.ui.screens.settings.SettingsRuntime
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.yield
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class BrowseShellHostTest {
    @Test
    fun `host starts home load and publishes async state for saveable sync`() = runBlocking {
        val dataSource = RecordingHostBrowseDataSource()
        val savedStates = mutableListOf<BrowseShellState>()
        val host = BrowseShellHost(
            profile = testProfile(),
            snapshot = testSnapshot(),
            runtime = RecordingBrowseShellRuntime(dataSource = dataSource),
            parentScope = CoroutineScope(coroutineContext + Job()),
            saveState = { savedStates += it },
        )

        waitUntil { host.state.value.browseState is BrowseUiState.Content }
        waitUntil { savedStates.any { it.browseState is BrowseUiState.Content } }

        assertEquals(1, dataSource.homeLoads)
        assertTrue(savedStates.any { it.browseState is BrowseUiState.Content })
    }

    @Test
    fun `host turns route changes into displayed route loads without compose effects`() = runBlocking {
        val dataSource = RecordingHostBrowseDataSource()
        val host = BrowseShellHost(
            profile = testProfile(),
            snapshot = testSnapshot(),
            runtime = RecordingBrowseShellRuntime(dataSource = dataSource),
            parentScope = CoroutineScope(coroutineContext + Job()),
        )

        host.dispatch(BrowseAction.OpenItem("night-harbor"))
        waitUntil { host.state.value.detailState is ItemDetailUiState.Content }
        waitUntil { dataSource.sourceProbeRequests.isNotEmpty() }

        assertEquals(TaruRoute.ItemDetail("night-harbor"), host.state.value.currentRoute)
        assertEquals(listOf("night-harbor"), dataSource.detailRequests)
        assertEquals("source-a", host.state.value.selectedSourceId)
        assertEquals(listOf("source-a"), dataSource.sourceProbeRequests)
    }

    @Test
    fun `host saves async playback route changes`() = runBlocking {
        val target = testPlaybackTarget("source-a")
        val savedStates = mutableListOf<BrowseShellState>()
        val host = BrowseShellHost(
            profile = testProfile(),
            snapshot = testSnapshot(),
            runtime = RecordingBrowseShellRuntime(
                dataSource = RecordingHostBrowseDataSource(
                    playbackState = PlaybackSelectionUiState.Content(
                        response = testPlaybackDecision("source-a"),
                        target = target,
                        capabilities = PlaybackCapabilities(),
                    ),
                ),
                playbackStarter = RecordingHostPlaybackStarter(
                    result = PlaybackStartResult.Success(
                        launch = testPlaybackLaunch(target),
                        preparedTarget = target,
                    ),
                ),
            ),
            parentScope = CoroutineScope(coroutineContext + Job()),
            saveState = { savedStates += it },
        )

        host.dispatch(BrowseAction.OpenItem("night-harbor"))
        waitUntil { host.state.value.detailState is ItemDetailUiState.Content }
        host.dispatch(BrowseAction.RequestPlayback("source-a"))
        waitUntil { host.state.value.playbackState is PlaybackSelectionUiState.Content }
        host.dispatch(BrowseAction.StartPlayback(target))
        waitUntil { host.state.value.currentRoute is TaruRoute.Player }
        waitUntil { savedStates.any { it.currentRoute is TaruRoute.Player } }

        assertTrue(savedStates.any { it.currentRoute is TaruRoute.Player })
    }

    @Test
    fun `host forwards settings actions through runtime`() {
        val runtime = RecordingBrowseShellRuntime()
        val host = BrowseShellHost(
            profile = testProfile(),
            snapshot = testSnapshot(),
            runtime = runtime,
            parentScope = CoroutineScope(Job()),
        )

        host.dispatchSettings(dev.taru.android.ui.screens.settings.SettingsAction.SignOutActiveProfile)

        assertEquals(listOf("server-token:server-1"), runtime.deletedTokenReferences)
        assertEquals(1, runtime.connectionRequests)
        host.close()
    }
}

private class RecordingBrowseShellRuntime(
    private val dataSource: RecordingHostBrowseDataSource = RecordingHostBrowseDataSource(),
    private val playbackStarter: BrowsePlaybackStarter = RecordingHostPlaybackStarter(),
    private val resumeResolver: BrowseResumeResolver = NoopHostResumeResolver,
) : BrowseShellRuntime {
    val deletedTokenReferences: MutableList<String> = mutableListOf()
    var connectionRequests: Int = 0
        private set

    override fun dataSource(profile: ServerProfile): BrowseDataSource = dataSource

    override fun playbackStarter(profile: ServerProfile): BrowsePlaybackStarter = playbackStarter

    override fun resumeResolver(profile: ServerProfile): BrowseResumeResolver = resumeResolver

    override fun settingsRuntime(): SettingsRuntime =
        object : SettingsRuntime {
            override fun saveSnapshot(snapshot: ServerProfileSnapshot) = Unit

            override fun deleteToken(reference: String) {
                deletedTokenReferences += reference
            }

            override fun requestConnection() {
                connectionRequests += 1
            }
        }
}

private data object NoopHostResumeResolver : BrowseResumeResolver {
    override fun resolve(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition? = null
}

private class RecordingHostPlaybackStarter(
    private val result: PlaybackStartResult = PlaybackStartResult.Success(
        launch = testPlaybackLaunch(testPlaybackTarget("source-a")),
        preparedTarget = testPlaybackTarget("source-a"),
    ),
) : BrowsePlaybackStarter {
    override suspend fun start(request: PlaybackStartRequest): PlaybackStartResult = result
}

private class RecordingHostBrowseDataSource(
    private val playbackState: PlaybackSelectionUiState = PlaybackSelectionUiState.Content(
        response = testPlaybackDecision("source-a"),
        target = testPlaybackTarget("source-a"),
        capabilities = PlaybackCapabilities(),
    ),
) : BrowseDataSource {
    var homeLoads: Int = 0
        private set
    val detailRequests: MutableList<String> = mutableListOf()
    val sourceProbeRequests: MutableList<String> = mutableListOf()

    override suspend fun loadHome(): BrowseUiState {
        homeLoads += 1
        return BrowseUiState.Content(
            libraries = LibraryListResponse(
                libraries = listOf(LibraryDto(id = "library-movies", name = "Movies")),
                page = testPage(1),
            ),
            items = ItemsResponse(
                items = listOf(testItem("night-harbor")),
                page = testPage(1),
            ),
        )
    }

    override suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState =
        LibraryDetailUiState.Content(
            LibrarySourcesResponse(
                library = LibraryDto(id = libraryId, name = "Movies"),
                page = testPage(0),
            ),
        )

    override suspend fun search(query: String): SearchUiState =
        SearchUiState.Content(SearchResponse(hits = emptyList(), page = testPage(0)))

    override suspend fun loadFacet(target: BrowseFacetTarget): FacetUiState =
        FacetUiState.Content(
            FacetItemsResponse(
                family = dev.taru.android.browse.BrowseFacetFamily.Genre,
                facetId = target.id.orEmpty(),
                facetLabel = target.label,
                items = emptyList(),
                page = testPage(0),
            ),
        )

    override suspend fun loadItemDetail(itemId: String): ItemDetailUiState {
        detailRequests += itemId
        return ItemDetailUiState.Content(
            ItemDetailResponse(
                item = testItem(itemId),
                sources = listOf(
                    MediaSourceDto(
                        id = "source-a",
                        libraryId = "library-movies",
                        itemId = itemId,
                    ),
                ),
            ),
        )
    }

    override suspend fun loadSourceProbe(sourceId: String): SourceProbeUiState {
        sourceProbeRequests += sourceId
        return SourceProbeUiState.Content(
            SourceProbeResponse(
                sourceId = sourceId,
                probe = MediaProbeDto(durationMs = 120_000),
            ),
        )
    }

    override suspend fun loadPlaybackSelection(sourceId: String): PlaybackSelectionUiState =
        playbackState
}

private suspend fun waitUntil(predicate: () -> Boolean) {
    withTimeout(5_000) {
        while (!predicate()) {
            yield()
        }
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

private fun testSnapshot(): ServerProfileSnapshot =
    ServerProfileSnapshot(
        profiles = listOf(testProfile()),
        activeProfileId = "server-1",
    )

private fun testItem(id: String): MediaItemDto =
    MediaItemDto(
        id = id,
        kind = "movie",
        metadata = CanonicalMetadataDto(title = "Night Harbor"),
    )

private fun testPlaybackDecision(sourceId: String): PlaybackDecisionResponse =
    PlaybackDecisionResponse(
        source = PlaybackMediaSourceDto(
            id = sourceId,
            libraryId = "library-movies",
            itemId = "night-harbor",
        ),
        decision = ClientPlaybackDecision(
            mode = ClientPlaybackMode.DirectPlay,
            reason = "direct",
        ),
    )

private fun testPlaybackTarget(sourceId: String): PlaybackRequestTarget =
    PlaybackRequestTarget(
        request = TaruHttpRequest(
            method = "GET",
            url = "http://127.0.0.1:3018/sources/$sourceId/stream",
            headers = emptyMap(),
        ),
        safeRequest = SafeRequestPreview(
            method = "GET",
            url = "http://127.0.0.1:3018/sources/$sourceId/stream",
            headers = emptyMap(),
        ),
    )

private fun testPlaybackLaunch(target: PlaybackRequestTarget) =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = target,
        serverProfileId = "server-1",
        mediaItemId = "night-harbor",
        sourceId = "source-a",
        playbackMode = ClientPlaybackMode.DirectPlay,
        resumePositionMs = 42_000,
        resumeSource = PlaybackResumeSource.DeviceLocal,
    )

private fun testPage(returned: Int): PageInfo =
    PageInfo(
        limit = 24,
        offset = 0,
        returned = returned,
    )
