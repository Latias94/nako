package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFacetFamily
import dev.taru.android.browse.CanonicalMetadataDto
import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.LibrarySourcesResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.SearchItemHit
import dev.taru.android.browse.SearchResponse
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

    override suspend fun search(query: String): SearchUiState {
        val deferred = searches.computeIfAbsent(query) { CompletableDeferred() }
        registered
            .computeIfAbsent(query) { CompletableDeferred() }
            .complete(Unit)
        return deferred.await()
    }

    override suspend fun loadFacet(target: BrowseFacetTarget): FacetUiState =
        FacetUiState.Content(testFacet(target))

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
    private val facetState: FacetUiState = FacetUiState.Content(
        testFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Genre,
                label = "Mystery",
                id = "genre-mystery",
            ),
        ),
    ),
) : BrowseDataSource {
    var homeLoads: Int = 0
        private set
    val searchQueries: MutableList<String> = mutableListOf()
    val facetTargets: MutableList<BrowseFacetTarget> = mutableListOf()

    override suspend fun loadHome(): BrowseUiState {
        homeLoads += 1
        return homeState
    }

    override suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState =
        libraryDetailState

    override suspend fun search(query: String): SearchUiState {
        searchQueries += query
        return searchState
    }

    override suspend fun loadFacet(target: BrowseFacetTarget): FacetUiState {
        facetTargets += target
        return facetState
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

private fun testSearch(itemId: String): SearchResponse =
    SearchResponse(
        hits = listOf(
            SearchItemHit(
                item = testItem(itemId),
                score = 1.0f,
            ),
        ),
        page = testPage(returned = 1),
    )

private fun testFacet(target: BrowseFacetTarget): FacetItemsResponse =
    FacetItemsResponse(
        family = BrowseFacetFamily.Genre,
        facetId = target.id.orEmpty(),
        facetLabel = target.label,
        items = listOf(testItem("night-harbor")),
        page = testPage(returned = 1),
    )

private fun testItem(id: String): MediaItemDto =
    MediaItemDto(
        id = id,
        kind = "movie",
        metadata = CanonicalMetadataDto(title = "Night Harbor"),
    )

private fun testPage(returned: Int): PageInfo =
    PageInfo(
        limit = 24,
        offset = 0,
        returned = returned,
    )
