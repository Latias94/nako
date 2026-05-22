package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFacetFamily
import dev.taru.android.browse.CanonicalMetadataDto
import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.browse.GenreDto
import dev.taru.android.browse.GenreListResponse
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.LibrarySourcesResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.PersonDto
import dev.taru.android.browse.PersonResponse
import dev.taru.android.browse.SearchItemHit
import dev.taru.android.browse.SearchResponse
import dev.taru.android.browse.TagDto
import dev.taru.android.browse.TagListResponse
import dev.taru.android.media.MediaProbeDto
import dev.taru.android.media.SourceProbeResponse
import dev.taru.android.playback.ClientPlaybackDecision
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackCapabilities
import dev.taru.android.playback.PlaybackDecisionResponse
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.PlaybackMediaSourceDto
import dev.taru.android.playback.PlaybackRequestDescriptor
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.PlaybackStartRequest
import dev.taru.android.playback.PlaybackStartResult
import dev.taru.android.playback.SafePlaybackDiagnostics
import dev.taru.android.player.PlaybackResumeSource
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.player.playbackLaunchRequest
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.util.concurrent.ConcurrentHashMap

class BrowseSessionLoadingTest {
    @Test
    fun `home load enters loading then publishes content`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            homeState = BrowseUiState.Content(
                libraries = testLibraries("library-movies"),
                items = testItems("night-harbor"),
            ),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        val job = session.dispatch(BrowseAction.LoadHome)

        assertEquals(BrowseUiState.Loading, session.state.value.browseState)
        job?.join()
        assertTrue(session.state.value.browseState is BrowseUiState.Content)
        assertEquals(1, dataSource.homeLoads)
    }

    @Test
    fun `search submit trims query and retry uses submitted query`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            searchState = SearchUiState.Content(testSearch("night-harbor")),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.SearchQueryChanged("  harbor  "))
        session.dispatch(BrowseAction.SubmitSearch)?.join()
        session.dispatch(BrowseAction.SearchQueryChanged("ignored local edit"))
        session.dispatch(BrowseAction.RetrySearch)?.join()

        assertEquals(listOf("harbor", "harbor"), dataSource.searchQueries)
        assertEquals("harbor", session.state.value.submittedSearchQuery)
        assertTrue(session.state.value.searchState is SearchUiState.Content)
    }

    @Test
    fun `blank search clears submitted query and returns idle state`() = runBlocking {
        val dataSource = RecordingBrowseDataSource()
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.SearchQueryChanged("  "))
        val job = session.dispatch(BrowseAction.SubmitSearch)

        assertEquals(null, job)
        assertEquals("", session.state.value.submittedSearchQuery)
        assertEquals(SearchUiState.Idle, session.state.value.searchState)
        assertEquals(emptyList<String>(), dataSource.searchQueries)
    }

    @Test
    fun `older search response cannot overwrite newer submitted query`() = runBlocking {
        val dataSource = DeferredSearchBrowseDataSource()
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(Dispatchers.Default + Job()),
        )

        session.dispatch(BrowseAction.SearchQueryChanged("first"))
        val firstJob = session.dispatch(BrowseAction.SubmitSearch)
        session.dispatch(BrowseAction.SearchQueryChanged("second"))
        val secondJob = session.dispatch(BrowseAction.SubmitSearch)

        dataSource.awaitSearch("first")
        dataSource.awaitSearch("second")
        dataSource.completeSearch("second")
        secondJob?.join()
        dataSource.completeSearch("first")
        firstJob?.join()

        val content = session.state.value.searchState as SearchUiState.Content
        assertEquals("second", session.state.value.submittedSearchQuery)
        assertEquals("second", content.response.hits.single().item.id)
    }

    @Test
    fun `search load more appends the next public page without inventing totals`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            searchStates = listOf(
                SearchUiState.Content(testSearch("night-harbor", page = testPage(limit = 2, offset = 0, returned = 2))),
                SearchUiState.Content(testSearch("arrival", page = testPage(limit = 2, offset = 2, returned = 1))),
            ),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.SearchQueryChanged("harbor"))
        session.dispatch(BrowseAction.SubmitSearch)?.join()
        session.dispatch(BrowseAction.LoadMoreSearch)?.join()
        val noMoreJob = session.dispatch(BrowseAction.LoadMoreSearch)

        val content = session.state.value.searchState as SearchUiState.Content
        assertEquals(listOf("night-harbor", "arrival"), content.response.hits.map { it.item.id })
        assertEquals(testPage(limit = 2, offset = 2, returned = 1), content.response.page)
        assertEquals(false, content.canLoadMore)
        assertEquals(null, noMoreJob)
        assertEquals(
            listOf(
                "harbor" to PageRequest(limit = 24, offset = 0),
                "harbor" to PageRequest(limit = 2, offset = 2),
            ),
            dataSource.searchRequests,
        )
    }

    @Test
    fun `relationship index load more appends rows with server page semantics`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            relationshipIndexStates = listOf(
                testGenreIndexContent("genre-mystery", "Mystery", page = testPage(limit = 1, offset = 0, returned = 1)),
                testGenreIndexContent("genre-drama", "Drama", page = testPage(limit = 1, offset = 1, returned = 1)),
            ),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Genres))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Genres)))?.join()
        session.dispatch(BrowseAction.LoadMoreRelationshipIndex)?.join()

        val content = session.state.value.relationshipIndexState as RelationshipIndexUiState.Content
        assertEquals(listOf("Mystery", "Drama"), content.rows.map { it.title })
        assertEquals(testPage(limit = 1, offset = 1, returned = 1), content.page)
        assertEquals(
            listOf(
                RelationshipIndexFamily.Genres to PageRequest(limit = 50, offset = 0),
                RelationshipIndexFamily.Genres to PageRequest(limit = 1, offset = 1),
            ),
            dataSource.relationshipIndexRequests,
        )
    }

    @Test
    fun `facet load more appends related items and preserves current route`() = runBlocking {
        val target = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Genre,
            label = "Mystery",
            id = "genre-mystery",
        )
        val dataSource = RecordingBrowseDataSource(
            facetStates = listOf(
                FacetUiState.Content(testFacet(target, itemId = "night-harbor", page = testPage(limit = 1, offset = 0, returned = 1))),
                FacetUiState.Content(testFacet(target, itemId = "arrival", page = testPage(limit = 1, offset = 1, returned = 0))),
            ),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenFacet(target))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.BrowseFacet(target)))?.join()
        session.dispatch(BrowseAction.LoadMoreFacet)?.join()

        val content = session.state.value.facetState as FacetUiState.Content
        assertEquals(TaruRoute.BrowseFacet(target), session.state.value.currentRoute)
        assertEquals(listOf("night-harbor", "arrival"), content.response.items.map { it.id })
        assertEquals(false, content.canLoadMore)
        assertEquals(
            listOf(
                target to PageRequest(limit = 24, offset = 0),
                target to PageRequest(limit = 1, offset = 1),
            ),
            dataSource.facetRequests,
        )
    }

    @Test
    fun `item detail load selects first source and starts source probe`() = runBlocking {
        val resumePosition = ResumePlaybackPosition(
            positionMs = 92_000,
            source = PlaybackResumeSource.DeviceLocal,
        )
        val dataSource = RecordingBrowseDataSource(
            detailState = ItemDetailUiState.Content(
                testDetail(
                    itemId = "night-harbor",
                    sourceIds = listOf("source-a", "source-b"),
                ),
            ),
            sourceProbeState = SourceProbeUiState.Content(testSourceProbe("source-a")),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            resumeResolver = StaticBrowseResumeResolver(resumePosition),
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenItem("night-harbor"))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.ItemDetail("night-harbor")))?.join()

        assertTrue(session.state.value.detailState is ItemDetailUiState.Content)
        assertEquals("source-a", session.state.value.selectedSourceId)
        assertEquals(resumePosition, session.state.value.resumePosition)
        assertTrue(session.state.value.sourceProbeState is SourceProbeUiState.Content)
        assertEquals(listOf("source-a"), dataSource.sourceProbeRequests)
        assertEquals(PlaybackSelectionUiState.Idle, session.state.value.playbackState)
    }

    @Test
    fun `selecting a source resets playback decision and probes the selected source`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            detailState = ItemDetailUiState.Content(
                testDetail(
                    itemId = "night-harbor",
                    sourceIds = listOf("source-a", "source-b"),
                ),
            ),
            sourceProbeState = SourceProbeUiState.Content(testSourceProbe("source-b")),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenItem("night-harbor"))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.ItemDetail("night-harbor")))?.join()
        session.dispatch(BrowseAction.RequestPlayback("source-a"))?.join()
        session.dispatch(BrowseAction.SelectSource("source-b"))?.join()

        assertEquals("source-b", session.state.value.selectedSourceId)
        assertEquals(null, session.state.value.playbackRequestSourceId)
        assertEquals(PlaybackSelectionUiState.Idle, session.state.value.playbackState)
        assertEquals(listOf("source-a", "source-a", "source-b"), dataSource.sourceProbeRequests)
    }

    @Test
    fun `request playback loads playback decision for selected source`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            playbackState = PlaybackSelectionUiState.Content(
                response = testPlaybackDecision("source-a"),
                target = testPlaybackTarget("source-a"),
                capabilities = PlaybackCapabilities(),
            ),
        )
        val session = BrowseSession(
            initialState = BrowseShellState(
                navigation = TaruBrowseNavigationState.root().open(TaruRoute.ItemDetail("night-harbor")),
                selectedSourceId = "source-a",
                detailState = ItemDetailUiState.Content(
                    testDetail(
                        itemId = "night-harbor",
                        sourceIds = listOf("source-a"),
                    ),
                ),
            ),
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.RequestPlayback("source-a"))?.join()

        assertEquals("source-a", session.state.value.playbackRequestSourceId)
        assertTrue(session.state.value.playbackState is PlaybackSelectionUiState.Content)
        assertEquals(listOf("source-a"), dataSource.playbackRequests)
    }

    @Test
    fun `start playback opens player route with prepared target`() = runBlocking {
        val target = testPlaybackTarget("source-a")
        val preparedTarget = testPlaybackTarget("source-a", sessionId = "session-1")
        val starter = RecordingPlaybackStarter(
            result = PlaybackStartResult.Success(
                launch = playbackLaunchRequest(
                    title = "Night Harbor",
                    target = preparedTarget,
                    serverProfileId = "server-1",
                    mediaItemId = "night-harbor",
                    sourceId = "source-a",
                    playbackMode = ClientPlaybackMode.Remux,
                    sessionId = "session-1",
                    resumePositionMs = 42_000,
                    resumeSource = PlaybackResumeSource.UserPlaybackState,
                ),
                preparedTarget = preparedTarget,
            ),
        )
        val playbackDecision = testPlaybackDecision("source-a", mode = ClientPlaybackMode.Remux)
        val session = BrowseSession(
            initialState = BrowseShellState(
                navigation = TaruBrowseNavigationState.root().open(TaruRoute.ItemDetail("night-harbor")),
                detailState = ItemDetailUiState.Content(
                    testDetail(
                        itemId = "night-harbor",
                        sourceIds = listOf("source-a"),
                    ),
                ),
                selectedSourceId = "source-a",
                playbackRequestSourceId = "source-a",
                playbackState = PlaybackSelectionUiState.Content(
                    response = playbackDecision,
                    target = target,
                    capabilities = PlaybackCapabilities(containers = listOf("mp4")),
                ),
            ),
            dataSource = RecordingBrowseDataSource(),
            playbackStarter = starter,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.StartPlayback(target))?.join()

        val playerRoute = session.state.value.currentRoute as TaruRoute.Player
        val playbackState = session.state.value.playbackState as PlaybackSelectionUiState.Content
        assertEquals("Night Harbor", starter.requests.single().title)
        assertEquals("night-harbor", starter.requests.single().mediaItemId)
        assertEquals("source-a", starter.requests.single().sourceId)
        assertEquals(playbackDecision, starter.requests.single().decision)
        assertEquals(preparedTarget, playbackState.target)
        assertEquals("session-1", playerRoute.launch.sessionId)
        assertTrue(session.state.value.detailState is ItemDetailUiState.Content)
        assertEquals("source-a", session.state.value.selectedSourceId)
    }

    @Test
    fun `start playback failure keeps diagnostics in playback state`() = runBlocking {
        val diagnostics = SafePlaybackDiagnostics(
            category = PlaybackFailureCategory.MissingAccessToken,
            userMessage = "Sign in again before requesting playback.",
        )
        val target = testPlaybackTarget("source-a")
        val session = BrowseSession(
            initialState = BrowseShellState(
                navigation = TaruBrowseNavigationState.root().open(TaruRoute.ItemDetail("night-harbor")),
                detailState = ItemDetailUiState.Content(
                    testDetail(
                        itemId = "night-harbor",
                        sourceIds = listOf("source-a"),
                    ),
                ),
                selectedSourceId = "source-a",
                playbackRequestSourceId = "source-a",
                playbackState = PlaybackSelectionUiState.Content(
                    response = testPlaybackDecision("source-a"),
                    target = target,
                    capabilities = PlaybackCapabilities(),
                ),
            ),
            dataSource = RecordingBrowseDataSource(),
            playbackStarter = RecordingPlaybackStarter(
                result = PlaybackStartResult.Failure(diagnostics),
            ),
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.StartPlayback(target))?.join()

        assertEquals(TaruRoute.ItemDetail("night-harbor"), session.state.value.currentRoute)
        assertEquals(
            diagnostics,
            (session.state.value.playbackState as PlaybackSelectionUiState.Failure).diagnostics,
        )
    }

    @Test
    fun `library detail route load ignores stale response after back`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            libraryDetailState = LibraryDetailUiState.Content(testLibrarySources("library-movies")),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(Dispatchers.Default + Job()),
        )

        session.dispatch(BrowseAction.SelectDestination(TaruDestination.Libraries))
        session.dispatch(BrowseAction.OpenLibraryDetail("library-movies"))
        val job = session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.LibraryDetail("library-movies")))
        session.dispatch(BrowseAction.Back)

        job?.join()
        assertEquals(TaruRoute.TopLevel, session.state.value.currentRoute)
        assertEquals(LibraryDetailUiState.Idle, session.state.value.libraryDetailState)
    }

    @Test
    fun `person detail route loads person and related media items`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            personDetailState = PersonDetailUiState.Content(
                response = testPersonDetail("person-1"),
                relatedItems = testPersonItems("person-1"),
            ),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenPersonDetail("person-1"))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.PersonDetail("person-1")))?.join()

        val content = session.state.value.personDetailState as PersonDetailUiState.Content
        assertEquals("Demo Actor", content.response.person.name)
        assertEquals("Night Harbor", content.relatedItems.items.single().metadata.title)
        assertEquals(listOf("person-1"), dataSource.personDetailRequests)
    }

    @Test
    fun `person detail route load ignores stale response after back`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            personDetailState = PersonDetailUiState.Content(
                response = testPersonDetail("person-1"),
                relatedItems = testPersonItems("person-1"),
            ),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(Dispatchers.Default + Job()),
        )

        session.dispatch(BrowseAction.OpenPersonDetail("person-1"))
        val job = session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.PersonDetail("person-1")))
        session.dispatch(BrowseAction.Back)

        job?.join()
        assertEquals(TaruRoute.TopLevel, session.state.value.currentRoute)
        assertEquals(PersonDetailUiState.Idle, session.state.value.personDetailState)
    }

    @Test
    fun `relationship index route loads genre rows and retry reloads current index`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            relationshipIndexState = testGenreIndexContent(),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Genres))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Genres)))?.join()
        session.dispatch(BrowseAction.RetryCurrentRoute)?.join()

        val content = session.state.value.relationshipIndexState as RelationshipIndexUiState.Content
        val row = content.rows.single()
        assertEquals(RelationshipIndexFamily.Genres, content.family)
        assertEquals("Mystery", row.title)
        assertEquals(BrowseFacetUiFamily.Genre, row.target.family)
        assertEquals("genre-mystery", row.target.id)
        assertEquals(
            listOf(
                RelationshipIndexFamily.Genres to PageRequest(limit = 50, offset = 0),
                RelationshipIndexFamily.Genres to PageRequest(limit = 50, offset = 0),
            ),
            dataSource.relationshipIndexRequests,
        )

        session.dispatch(BrowseAction.OpenFacet(row.target))
        assertEquals(TaruRoute.BrowseFacet(row.target), session.state.value.currentRoute)
    }

    @Test
    fun `relationship index route loads tag rows into tag facet route`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            relationshipIndexState = testTagIndexContent(),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Tags))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Tags)))?.join()

        val content = session.state.value.relationshipIndexState as RelationshipIndexUiState.Content
        val row = content.rows.single()
        assertEquals(RelationshipIndexFamily.Tags, content.family)
        assertEquals("Lighthouse", row.title)
        assertEquals(BrowseFacetUiFamily.Tag, row.target.family)
        assertEquals("tag-lighthouse", row.target.id)
        assertEquals(
            listOf(RelationshipIndexFamily.Tags to PageRequest(limit = 50, offset = 0)),
            dataSource.relationshipIndexRequests,
        )

        session.dispatch(BrowseAction.OpenFacet(row.target))
        assertEquals(TaruRoute.BrowseFacet(row.target), session.state.value.currentRoute)
    }

    @Test
    fun `relationship index route load ignores stale response after back`() = runBlocking {
        val dataSource = RecordingBrowseDataSource(
            relationshipIndexState = testGenreIndexContent(),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(Dispatchers.Default + Job()),
        )

        session.dispatch(BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Genres))
        val job = session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Genres)))
        session.dispatch(BrowseAction.Back)

        job?.join()
        assertEquals(TaruRoute.TopLevel, session.state.value.currentRoute)
        assertEquals(RelationshipIndexUiState.Idle, session.state.value.relationshipIndexState)
    }

    @Test
    fun `public backed facet loads content and unsupported facet stays local api gap`() = runBlocking {
        val backedFacet = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Genre,
            label = "Mystery",
            id = "genre-mystery",
        )
        val unsupportedFacet = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Studio,
            label = "Studio",
            id = "studio-1",
        )
        val dataSource = RecordingBrowseDataSource(
            facetState = FacetUiState.Content(testFacet(backedFacet)),
        )
        val session = BrowseSession(
            dataSource = dataSource,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(BrowseAction.OpenFacet(backedFacet))
        session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.BrowseFacet(backedFacet)))?.join()
        assertTrue(session.state.value.facetState is FacetUiState.Content)
        assertEquals(listOf(backedFacet), dataSource.facetTargets)

        session.dispatch(BrowseAction.OpenFacet(unsupportedFacet))
        val gapJob = session.dispatch(BrowseAction.RouteDisplayed(TaruRoute.BrowseFacet(unsupportedFacet)))
        assertEquals(null, gapJob)
        assertTrue(session.state.value.facetState is FacetUiState.ApiGap)
        assertEquals(listOf(backedFacet), dataSource.facetTargets)
    }
}

private class RecordingPlaybackStarter(
    private val result: PlaybackStartResult,
) : BrowsePlaybackStarter {
    val requests: MutableList<PlaybackStartRequest> = mutableListOf()

    override suspend fun start(request: PlaybackStartRequest): PlaybackStartResult {
        requests += request
        return result
    }
}

private class StaticBrowseResumeResolver(
    private val resumePosition: ResumePlaybackPosition?,
) : BrowseResumeResolver {
    override fun resolve(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition? = resumePosition
}

private class DeferredSearchBrowseDataSource : BrowseDataSource {
    private val searches: ConcurrentHashMap<String, CompletableDeferred<SearchUiState>> = ConcurrentHashMap()
    private val registered: ConcurrentHashMap<String, CompletableDeferred<Unit>> = ConcurrentHashMap()

    override suspend fun loadHome(): BrowseUiState =
        BrowseUiState.Content(
            libraries = testLibraries("library-default"),
            items = testItems("item-default"),
        )

    override suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState =
        LibraryDetailUiState.Content(testLibrarySources(libraryId))

    override suspend fun search(
        query: String,
        page: PageRequest,
    ): SearchUiState {
        val deferred = searches.computeIfAbsent(query) { CompletableDeferred() }
        registered
            .computeIfAbsent(query) { CompletableDeferred() }
            .complete(Unit)
        return deferred.await()
    }

    override suspend fun loadFacet(
        target: BrowseFacetTarget,
        page: PageRequest,
    ): FacetUiState =
        FacetUiState.Content(testFacet(target))

    override suspend fun loadRelationshipIndex(
        family: RelationshipIndexFamily,
        page: PageRequest,
    ): RelationshipIndexUiState =
        testGenreIndexContent()

    override suspend fun loadPersonDetail(personId: String): PersonDetailUiState =
        PersonDetailUiState.Content(
            response = testPersonDetail(personId),
            relatedItems = testPersonItems(personId),
        )

    override suspend fun loadItemDetail(itemId: String): ItemDetailUiState =
        ItemDetailUiState.Content(
            testDetail(
                itemId = itemId,
                sourceIds = listOf("source-default"),
            ),
        )

    override suspend fun loadSourceProbe(sourceId: String): SourceProbeUiState =
        SourceProbeUiState.Content(testSourceProbe(sourceId))

    override suspend fun loadPlaybackSelection(sourceId: String): PlaybackSelectionUiState =
        PlaybackSelectionUiState.Content(
            response = testPlaybackDecision(sourceId),
            target = testPlaybackTarget(sourceId),
            capabilities = PlaybackCapabilities(),
        )

    fun completeSearch(query: String) {
        searches
            .computeIfAbsent(query) { CompletableDeferred() }
            .complete(SearchUiState.Content(testSearch(query)))
    }

    suspend fun awaitSearch(query: String) {
        withTimeout(5_000) {
            registered
                .computeIfAbsent(query) { CompletableDeferred() }
                .await()
        }
    }
}

private class RecordingBrowseDataSource(
    private val homeState: BrowseUiState = BrowseUiState.Content(
        libraries = testLibraries("library-default"),
        items = testItems("item-default"),
    ),
    private val libraryDetailState: LibraryDetailUiState = LibraryDetailUiState.Content(
        testLibrarySources("library-default"),
    ),
    private val searchState: SearchUiState = SearchUiState.Content(testSearch("item-default")),
    private val searchStates: List<SearchUiState> = emptyList(),
    private val facetState: FacetUiState = FacetUiState.Content(
        testFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Genre,
                label = "Mystery",
                id = "genre-mystery",
            ),
        ),
    ),
    private val facetStates: List<FacetUiState> = emptyList(),
    private val personDetailState: PersonDetailUiState = PersonDetailUiState.Content(
        response = testPersonDetail("person-default"),
        relatedItems = testPersonItems("person-default"),
    ),
    private val relationshipIndexState: RelationshipIndexUiState = testGenreIndexContent(),
    private val relationshipIndexStates: List<RelationshipIndexUiState> = emptyList(),
    private val detailState: ItemDetailUiState = ItemDetailUiState.Content(
        testDetail(
            itemId = "item-default",
            sourceIds = listOf("source-default"),
        ),
    ),
    private val sourceProbeState: SourceProbeUiState = SourceProbeUiState.Content(
        testSourceProbe("source-default"),
    ),
    private val playbackState: PlaybackSelectionUiState = PlaybackSelectionUiState.Content(
        response = testPlaybackDecision("source-default"),
        target = testPlaybackTarget("source-default"),
        capabilities = PlaybackCapabilities(),
    ),
) : BrowseDataSource {
    var homeLoads: Int = 0
        private set
    val searchQueries: MutableList<String> = mutableListOf()
    val searchRequests: MutableList<Pair<String, PageRequest>> = mutableListOf()
    val facetTargets: MutableList<BrowseFacetTarget> = mutableListOf()
    val facetRequests: MutableList<Pair<BrowseFacetTarget, PageRequest>> = mutableListOf()
    val personDetailRequests: MutableList<String> = mutableListOf()
    val relationshipIndexRequests: MutableList<Pair<RelationshipIndexFamily, PageRequest>> = mutableListOf()
    val detailRequests: MutableList<String> = mutableListOf()
    val sourceProbeRequests: MutableList<String> = mutableListOf()
    val playbackRequests: MutableList<String> = mutableListOf()
    private val queuedSearchStates = ArrayDeque(searchStates)
    private val queuedFacetStates = ArrayDeque(facetStates)
    private val queuedRelationshipIndexStates = ArrayDeque(relationshipIndexStates)

    override suspend fun loadHome(): BrowseUiState {
        homeLoads += 1
        return homeState
    }

    override suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState =
        libraryDetailState

    override suspend fun search(
        query: String,
        page: PageRequest,
    ): SearchUiState {
        searchQueries += query
        searchRequests += query to page
        return queuedSearchStates.removeFirstOrNull() ?: searchState
    }

    override suspend fun loadFacet(
        target: BrowseFacetTarget,
        page: PageRequest,
    ): FacetUiState {
        facetTargets += target
        facetRequests += target to page
        return queuedFacetStates.removeFirstOrNull() ?: facetState
    }

    override suspend fun loadRelationshipIndex(
        family: RelationshipIndexFamily,
        page: PageRequest,
    ): RelationshipIndexUiState {
        relationshipIndexRequests += family to page
        return queuedRelationshipIndexStates.removeFirstOrNull() ?: relationshipIndexState
    }

    override suspend fun loadPersonDetail(personId: String): PersonDetailUiState {
        personDetailRequests += personId
        return personDetailState
    }

    override suspend fun loadItemDetail(itemId: String): ItemDetailUiState {
        detailRequests += itemId
        return detailState
    }

    override suspend fun loadSourceProbe(sourceId: String): SourceProbeUiState {
        sourceProbeRequests += sourceId
        return sourceProbeState
    }

    override suspend fun loadPlaybackSelection(sourceId: String): PlaybackSelectionUiState {
        playbackRequests += sourceId
        return playbackState
    }
}

private fun testLibraries(id: String): LibraryListResponse =
    LibraryListResponse(
        libraries = listOf(
            LibraryDto(
                id = id,
                name = "Movies",
            ),
        ),
        page = testPage(returned = 1),
    )

private fun testLibrarySources(id: String): LibrarySourcesResponse =
    LibrarySourcesResponse(
        library = LibraryDto(
            id = id,
            name = "Movies",
        ),
        page = testPage(returned = 0),
    )

private fun testItems(id: String): dev.taru.android.browse.ItemsResponse =
    dev.taru.android.browse.ItemsResponse(
        items = listOf(testItem(id)),
        page = testPage(returned = 1),
    )

private fun testSearch(
    itemId: String,
    page: PageInfo = testPage(returned = 1),
): SearchResponse =
    SearchResponse(
        hits = listOf(
            SearchItemHit(
                item = testItem(itemId),
                score = 1.0f,
            ),
        ),
        page = page,
    )

private fun testFacet(
    target: BrowseFacetTarget,
    itemId: String = "night-harbor",
    page: PageInfo = testPage(returned = 1),
): FacetItemsResponse =
    FacetItemsResponse(
        family = BrowseFacetFamily.Genre,
        facetId = target.id.orEmpty(),
        facetLabel = target.label,
        items = listOf(testItem(itemId)),
        page = page,
    )

private fun testPersonDetail(personId: String): PersonResponse =
    PersonResponse(
        person = PersonDto(
            id = personId,
            name = "Demo Actor",
            sortName = "Actor, Demo",
            overview = "Keeps the lighthouse.",
        ),
    )

private fun testPersonItems(personId: String): FacetItemsResponse =
    FacetItemsResponse(
        family = BrowseFacetFamily.Person,
        facetId = personId,
        facetLabel = "Demo Actor",
        items = listOf(testItem("night-harbor")),
        page = testPage(returned = 1),
    )

private fun testGenreIndexContent(
    id: String = "genre-mystery",
    name: String = "Mystery",
    page: PageInfo = testPage(returned = 1),
): RelationshipIndexUiState.Content =
    GenreListResponse(
        genres = listOf(
            GenreDto(
                id = id,
                name = name,
            ),
        ),
        page = page,
    ).toRelationshipIndexContent()

private fun testTagIndexContent(
    id: String = "tag-lighthouse",
    name: String = "Lighthouse",
    page: PageInfo = testPage(returned = 1),
): RelationshipIndexUiState.Content =
    TagListResponse(
        tags = listOf(
            TagDto(
                id = id,
                name = name,
            ),
        ),
        page = page,
    ).toRelationshipIndexContent()

private fun testItem(id: String): MediaItemDto =
    MediaItemDto(
        id = id,
        kind = "movie",
        metadata = CanonicalMetadataDto(title = "Night Harbor"),
    )

private fun testDetail(
    itemId: String,
    sourceIds: List<String>,
): ItemDetailResponse =
    ItemDetailResponse(
        item = testItem(itemId),
        sources = sourceIds.map { sourceId ->
            MediaSourceDto(
                id = sourceId,
                libraryId = "library-movies",
                itemId = itemId,
            )
        },
    )

private fun testSourceProbe(sourceId: String): SourceProbeResponse =
    SourceProbeResponse(
        sourceId = sourceId,
        probe = MediaProbeDto(durationMs = 120_000),
    )

private fun testPlaybackDecision(
    sourceId: String,
    mode: ClientPlaybackMode = ClientPlaybackMode.DirectPlay,
): PlaybackDecisionResponse =
    PlaybackDecisionResponse(
        source = PlaybackMediaSourceDto(
            id = sourceId,
            libraryId = "library-movies",
            itemId = "night-harbor",
        ),
        decision = ClientPlaybackDecision(
            mode = mode,
            reason = "direct",
        ),
    )

private fun testPlaybackTarget(
    sourceId: String,
    sessionId: String? = null,
): PlaybackRequestTarget =
    PlaybackRequestTarget(
        request = PlaybackRequestDescriptor(
            method = "GET",
            url = "http://127.0.0.1:3018/sources/$sourceId/stream",
            headers = emptyMap(),
        ),
        sessionId = sessionId,
    )

private fun testPage(
    returned: Int,
    limit: Int = 24,
    offset: Long = 0,
): PageInfo =
    PageInfo(
        limit = limit,
        offset = offset,
        returned = returned,
    )
