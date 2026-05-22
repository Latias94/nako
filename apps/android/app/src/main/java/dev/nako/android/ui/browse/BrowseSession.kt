package dev.nako.android.ui.browse

import dev.nako.android.browse.PageRequest
import dev.nako.android.player.PlaybackLaunchRequest
import dev.nako.android.player.ResumePlaybackPosition
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.playback.PlaybackStartRequest
import dev.nako.android.playback.PlaybackStartResult
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

internal data class BrowseShellState(
    val navigation: NakoBrowseNavigationState = NakoBrowseNavigationState.root(),
    val browseState: BrowseUiState = BrowseUiState.Loading,
    val libraryDetailState: LibraryDetailUiState = LibraryDetailUiState.Idle,
    val searchQuery: String = "",
    val submittedSearchQuery: String = "",
    val searchState: SearchUiState = SearchUiState.Idle,
    val personDetailState: PersonDetailUiState = PersonDetailUiState.Idle,
    val relationshipIndexState: RelationshipIndexUiState = RelationshipIndexUiState.Idle,
    val facetState: FacetUiState = FacetUiState.Idle,
    val detailState: ItemDetailUiState = ItemDetailUiState.Idle,
    val selectedSourceId: String? = null,
    val resumePosition: ResumePlaybackPosition? = null,
    val sourceProbeState: SourceProbeUiState = SourceProbeUiState.Idle,
    val playbackRequestSourceId: String? = null,
    val playbackState: PlaybackSelectionUiState = PlaybackSelectionUiState.Idle,
) {
    val selectedDestination: NakoDestination = navigation.selectedDestination
    val currentRoute: NakoRoute = navigation.currentRoute
    val navigationVisible: Boolean = navigation.navigationVisible
    val canNavigateBack: Boolean = navigation.canNavigateBack
}

internal sealed interface BrowseAction {
    data class SelectDestination(val destination: NakoDestination) : BrowseAction
    data class OpenItem(val itemId: String) : BrowseAction
    data class OpenLibraryDetail(val libraryId: String) : BrowseAction
    data class OpenPersonDetail(val personId: String) : BrowseAction
    data class OpenRelationshipIndex(val family: RelationshipIndexFamily) : BrowseAction
    data class OpenFacet(val target: BrowseFacetTarget) : BrowseAction
    data class OpenPlayer(val launch: PlaybackLaunchRequest) : BrowseAction
    data class RouteDisplayed(val route: NakoRoute) : BrowseAction
    data class SearchQueryChanged(val query: String) : BrowseAction
    data class SelectSource(val sourceId: String) : BrowseAction
    data class RequestPlayback(val sourceId: String) : BrowseAction
    data class StartPlayback(val target: PlaybackRequestTarget) : BrowseAction
    data object OpenServerProfile : BrowseAction
    data object Back : BrowseAction
    data object LoadHome : BrowseAction
    data object RetryHome : BrowseAction
    data object SubmitSearch : BrowseAction
    data object RetrySearch : BrowseAction
    data object LoadMoreSearch : BrowseAction
    data object LoadMoreRelationshipIndex : BrowseAction
    data object LoadMoreFacet : BrowseAction
    data object RetryCurrentRoute : BrowseAction
    data object RetrySourceProbe : BrowseAction
    data object RetryPlaybackDecision : BrowseAction
}

internal interface BrowseDataSource {
    suspend fun loadHome(): BrowseUiState
    suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState
    suspend fun search(
        query: String,
        page: PageRequest = PageRequest(limit = 24, offset = 0),
    ): SearchUiState
    suspend fun loadPersonDetail(personId: String): PersonDetailUiState
    suspend fun loadRelationshipIndex(
        family: RelationshipIndexFamily,
        page: PageRequest = PageRequest(limit = 50, offset = 0),
    ): RelationshipIndexUiState
    suspend fun loadFacet(
        target: BrowseFacetTarget,
        page: PageRequest = PageRequest(limit = 24, offset = 0),
    ): FacetUiState
    suspend fun loadItemDetail(itemId: String): ItemDetailUiState
    suspend fun loadSourceProbe(sourceId: String): SourceProbeUiState
    suspend fun loadPlaybackSelection(sourceId: String): PlaybackSelectionUiState
}

internal interface BrowsePlaybackStarter {
    suspend fun start(request: PlaybackStartRequest): PlaybackStartResult
}

internal interface BrowseResumeResolver {
    fun resolve(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition?
}

internal class BrowseSession(
    initialState: BrowseShellState = BrowseShellState(),
    private val dataSource: BrowseDataSource? = null,
    private val playbackStarter: BrowsePlaybackStarter? = null,
    private val resumeResolver: BrowseResumeResolver = EmptyBrowseResumeResolver,
    private val scope: CoroutineScope? = null,
) {
    private val _state = MutableStateFlow(initialState)
    private val store = FlowBrowseSessionStore(_state)
    private val routeStatePolicy = BrowseRouteStatePolicy()
    private val navigationSession = BrowseSessionNavigation(routeStatePolicy)
    private val searchSession: BrowseSearchSession? = if (dataSource != null && scope != null) {
        BrowseSearchSession(
            store = store,
            scope = scope,
            dataSource = dataSource,
        )
    } else {
        null
    }
    private val detailSession: BrowseItemDetailSession? = if (dataSource != null && scope != null) {
        BrowseItemDetailSession(
            store = store,
            scope = scope,
            dataSource = dataSource,
            routeStatePolicy = routeStatePolicy,
            resolveResumePosition = ::resolveResumePosition,
        )
    } else {
        null
    }
    private val playbackSession: BrowsePlaybackSession? = if (playbackStarter != null && scope != null) {
        BrowsePlaybackSession(
            store = store,
            scope = scope,
            playbackStarter = playbackStarter,
            navigation = navigationSession,
        )
    } else {
        null
    }
    private val routeLoadingSession: BrowseRouteLoadingSession? =
        if (dataSource != null && scope != null && detailSession != null) {
            BrowseRouteLoadingSession(
                store = store,
                scope = scope,
                dataSource = dataSource,
                routeStatePolicy = routeStatePolicy,
                detailSession = detailSession,
            )
        } else {
            null
        }
    val state: StateFlow<BrowseShellState> = _state.asStateFlow()

    private var homeRequestId: Long = 0

    fun dispatch(action: BrowseAction): Job? =
        when (action) {
            BrowseAction.LoadHome,
            BrowseAction.RetryHome,
            -> loadHome()
            BrowseAction.SubmitSearch -> requiredSearchSession().submitSearch()
            BrowseAction.RetrySearch -> requiredSearchSession().retrySearch()
            BrowseAction.LoadMoreSearch -> requiredSearchSession().loadMoreSearch()
            BrowseAction.LoadMoreRelationshipIndex -> requiredRouteLoadingSession().loadMoreRelationshipIndex()
            BrowseAction.LoadMoreFacet -> requiredRouteLoadingSession().loadMoreFacet()
            BrowseAction.RetryCurrentRoute -> requiredRouteLoadingSession().loadCurrentRoute()
            BrowseAction.RetrySourceProbe -> requiredDetailSession().loadSelectedSourceProbe()
            BrowseAction.RetryPlaybackDecision -> requiredDetailSession().loadRequestedPlaybackSelection()
            is BrowseAction.StartPlayback -> requiredPlaybackSession().startPlayback(action.target)
            is BrowseAction.RouteDisplayed -> requiredRouteLoadingSession().loadRoute(action.route)
            is BrowseAction.SearchQueryChanged -> {
                store.update { it.copy(searchQuery = action.query) }
                null
            }
            is BrowseAction.SelectSource -> {
                requiredDetailSession().selectSource(action.sourceId)
            }
            is BrowseAction.RequestPlayback -> {
                requiredDetailSession().requestPlayback(action.sourceId).last()
            }
            else -> {
                store.set(navigationSession.reduceAndPrepare(store.value, action))
                null
            }
        }

    fun reduce(
        current: BrowseShellState,
        action: BrowseAction,
    ): BrowseShellState =
        navigationSession.reduce(current, action)

    private fun loadHome(): Job {
        val requestId = ++homeRequestId
        store.update { it.copy(browseState = BrowseUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadHome()
            store.update { current ->
                if (requestId == homeRequestId) {
                    current.copy(browseState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun requiredDataSource(): BrowseDataSource =
        requireNotNull(dataSource) {
            "BrowseDataSource is required for async BrowseAction handling."
        }

    private fun requiredScope(): CoroutineScope =
        requireNotNull(scope) {
            "CoroutineScope is required for async BrowseAction handling."
        }

    private fun requiredSearchSession(): BrowseSearchSession =
        requireNotNull(searchSession) {
            "BrowseSearchSession is required for async search action handling."
        }

    private fun requiredDetailSession(): BrowseItemDetailSession =
        requireNotNull(detailSession) {
            "BrowseItemDetailSession is required for detail/source/playback selection action handling."
        }

    private fun requiredPlaybackSession(): BrowsePlaybackSession =
        requireNotNull(playbackSession) {
            "BrowsePlaybackSession is required for playback start action handling."
        }

    private fun requiredRouteLoadingSession(): BrowseRouteLoadingSession =
        requireNotNull(routeLoadingSession) {
            "BrowseRouteLoadingSession is required for route loading action handling."
        }

    private fun resolveResumePosition(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition? =
        resumeResolver.resolve(
            detailState = detailState,
            selectedSourceId = selectedSourceId,
        )
}

private data object EmptyBrowseResumeResolver : BrowseResumeResolver {
    override fun resolve(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition? = null
}

internal fun BrowseFacetTarget.apiGapState(): FacetUiState.ApiGap =
    FacetUiState.ApiGap(
        title = "This list is not available yet",
        body = apiGapBody(),
    )

private fun BrowseFacetTarget.apiGapBody(): String =
    when (family) {
        BrowseFacetUiFamily.Genre,
        BrowseFacetUiFamily.Tag,
        BrowseFacetUiFamily.Person,
        -> if (id.isNullOrBlank()) {
            "Nako can show this ${family.label.lowercase()} label, but your server has not shared a linkable identity for related titles yet."
        } else {
            "Related titles for this ${family.label.lowercase()} cannot be opened from here yet."
        }
        BrowseFacetUiFamily.Library -> "Library browsing for this view needs server support."
        BrowseFacetUiFamily.Studio -> "Studio browsing needs server support."
        BrowseFacetUiFamily.Collection -> "Collection browsing needs server support."
        BrowseFacetUiFamily.Year -> "Year browsing needs server support."
        BrowseFacetUiFamily.ItemKind -> "Title-type browsing needs server support."
        BrowseFacetUiFamily.SourceMode -> "Playback-mode browsing is not available yet."
    }
