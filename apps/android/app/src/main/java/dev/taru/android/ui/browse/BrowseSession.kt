package dev.taru.android.ui.browse

import dev.taru.android.player.PlaybackLaunchRequest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

internal data class BrowseShellState(
    val navigation: TaruBrowseNavigationState = TaruBrowseNavigationState.root(),
    val browseState: BrowseUiState = BrowseUiState.Loading,
    val libraryDetailState: LibraryDetailUiState = LibraryDetailUiState.Idle,
    val searchQuery: String = "",
    val submittedSearchQuery: String = "",
    val searchState: SearchUiState = SearchUiState.Idle,
    val facetState: FacetUiState = FacetUiState.Idle,
) {
    val selectedDestination: TaruDestination = navigation.selectedDestination
    val currentRoute: TaruRoute = navigation.currentRoute
    val navigationVisible: Boolean = navigation.navigationVisible
    val canNavigateBack: Boolean = navigation.canNavigateBack
}

internal sealed interface BrowseAction {
    data class SelectDestination(val destination: TaruDestination) : BrowseAction
    data class OpenItem(val itemId: String) : BrowseAction
    data class OpenLibraryDetail(val libraryId: String) : BrowseAction
    data class OpenFacet(val target: BrowseFacetTarget) : BrowseAction
    data class OpenPlayer(val launch: PlaybackLaunchRequest) : BrowseAction
    data class RouteDisplayed(val route: TaruRoute) : BrowseAction
    data class SearchQueryChanged(val query: String) : BrowseAction
    data object OpenServerProfile : BrowseAction
    data object Back : BrowseAction
    data object LoadHome : BrowseAction
    data object RetryHome : BrowseAction
    data object SubmitSearch : BrowseAction
    data object RetrySearch : BrowseAction
    data object RetryCurrentRoute : BrowseAction
}

internal interface BrowseDataSource {
    suspend fun loadHome(): BrowseUiState
    suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState
    suspend fun search(query: String): SearchUiState
    suspend fun loadFacet(target: BrowseFacetTarget): FacetUiState
}

internal class BrowseSession(
    initialState: BrowseShellState = BrowseShellState(),
    private val dataSource: BrowseDataSource? = null,
    private val scope: CoroutineScope? = null,
) {
    private val _state = MutableStateFlow(initialState)
    val state: StateFlow<BrowseShellState> = _state.asStateFlow()

    private var homeRequestId: Long = 0
    private var libraryDetailRequestId: Long = 0
    private var searchRequestId: Long = 0
    private var facetRequestId: Long = 0

    fun dispatch(action: BrowseAction): Job? =
        when (action) {
            BrowseAction.LoadHome,
            BrowseAction.RetryHome,
            -> loadHome()
            BrowseAction.SubmitSearch -> submitSearch()
            BrowseAction.RetrySearch -> retrySearch()
            BrowseAction.RetryCurrentRoute -> loadCurrentRoute()
            is BrowseAction.RouteDisplayed -> loadRoute(action.route)
            is BrowseAction.SearchQueryChanged -> {
                _state.update { it.copy(searchQuery = action.query) }
                null
            }
            else -> {
                _state.value = prepareRouteState(
                    previous = _state.value,
                    next = reduce(_state.value, action),
                )
                null
            }
        }

    fun reduce(
        current: BrowseShellState,
        action: BrowseAction,
    ): BrowseShellState =
        when (action) {
            BrowseAction.Back -> current.copy(
                navigation = current.navigation.navigateBack(),
            )
            BrowseAction.OpenServerProfile -> current.copy(
                navigation = current.navigation.open(TaruRoute.ServerProfile),
            )
            is BrowseAction.OpenFacet -> current.copy(
                navigation = current.navigation.open(TaruRoute.BrowseFacet(action.target)),
            )
            is BrowseAction.OpenItem -> current.copy(
                navigation = current.navigation.open(TaruRoute.ItemDetail(action.itemId)),
            )
            is BrowseAction.OpenLibraryDetail -> current.copy(
                navigation = current.navigation.open(TaruRoute.LibraryDetail(action.libraryId)),
            )
            is BrowseAction.OpenPlayer -> current.copy(
                navigation = current.navigation.open(TaruRoute.Player(action.launch)),
            )
            is BrowseAction.SelectDestination -> current.copy(
                navigation = current.navigation.selectDestination(action.destination),
            )
            BrowseAction.LoadHome,
            BrowseAction.RetryHome,
            BrowseAction.SubmitSearch,
            BrowseAction.RetrySearch,
            BrowseAction.RetryCurrentRoute,
            is BrowseAction.RouteDisplayed,
            is BrowseAction.SearchQueryChanged,
            -> current
        }

    private fun loadHome(): Job {
        val requestId = ++homeRequestId
        _state.update { it.copy(browseState = BrowseUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadHome()
            _state.update { current ->
                if (requestId == homeRequestId) {
                    current.copy(browseState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun submitSearch(): Job? {
        val query = _state.value.searchQuery.trim()
        searchRequestId += 1
        if (query.isBlank()) {
            _state.update {
                it.copy(
                    submittedSearchQuery = "",
                    searchState = SearchUiState.Idle,
                )
            }
            return null
        }

        return loadSearch(query)
    }

    private fun retrySearch(): Job? {
        val query = _state.value.submittedSearchQuery.trim()
        searchRequestId += 1
        if (query.isBlank()) {
            _state.update { it.copy(searchState = SearchUiState.Idle) }
            return null
        }

        return loadSearch(query)
    }

    private fun loadSearch(query: String): Job {
        val requestId = searchRequestId
        _state.update {
            it.copy(
                submittedSearchQuery = query,
                searchState = SearchUiState.Loading,
            )
        }
        return requiredScope().launch {
            val nextState = requiredDataSource().search(query)
            _state.update { current ->
                if (requestId == searchRequestId && current.submittedSearchQuery == query) {
                    current.copy(searchState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadCurrentRoute(): Job? =
        loadRoute(_state.value.currentRoute)

    private fun loadRoute(route: TaruRoute): Job? =
        when (route) {
            is TaruRoute.LibraryDetail -> loadLibraryDetail(route.libraryId)
            is TaruRoute.BrowseFacet -> loadFacet(route.target)
            else -> {
                libraryDetailRequestId += 1
                facetRequestId += 1
                _state.update {
                    it.copy(
                        libraryDetailState = LibraryDetailUiState.Idle,
                        facetState = FacetUiState.Idle,
                    )
                }
                null
            }
        }

    private fun prepareRouteState(
        previous: BrowseShellState,
        next: BrowseShellState,
    ): BrowseShellState {
        if (previous.currentRoute == next.currentRoute) {
            return next
        }

        var prepared = next
        prepared = when (val route = next.currentRoute) {
            is TaruRoute.LibraryDetail -> {
                libraryDetailRequestId += 1
                prepared.copy(libraryDetailState = LibraryDetailUiState.Loading)
            }
            else -> {
                libraryDetailRequestId += 1
                prepared.copy(libraryDetailState = LibraryDetailUiState.Idle)
            }
        }
        prepared = when (val route = next.currentRoute) {
            is TaruRoute.BrowseFacet -> {
                facetRequestId += 1
                prepared.copy(
                    facetState = if (route.target.isPublicRouteBacked) {
                        FacetUiState.Loading
                    } else {
                        route.target.apiGapState()
                    },
                )
            }
            else -> {
                facetRequestId += 1
                prepared.copy(facetState = FacetUiState.Idle)
            }
        }

        return prepared
    }

    private fun loadLibraryDetail(libraryId: String): Job {
        val requestId = ++libraryDetailRequestId
        _state.update { it.copy(libraryDetailState = LibraryDetailUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadLibraryDetail(libraryId)
            _state.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.LibraryDetail(libraryId)
                if (requestId == libraryDetailRequestId && routeStillCurrent) {
                    current.copy(libraryDetailState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadFacet(target: BrowseFacetTarget): Job? {
        val requestId = ++facetRequestId
        if (!target.isPublicRouteBacked) {
            _state.update { it.copy(facetState = target.apiGapState()) }
            return null
        }

        _state.update { it.copy(facetState = FacetUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadFacet(target)
            _state.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.BrowseFacet(target)
                if (requestId == facetRequestId && routeStillCurrent) {
                    current.copy(facetState = nextState)
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
}

internal fun BrowseFacetTarget.apiGapState(): FacetUiState.ApiGap =
    FacetUiState.ApiGap(
        title = "${family.label} not available",
        body = apiGapBody(),
    )

private fun BrowseFacetTarget.apiGapBody(): String =
    when (family) {
        BrowseFacetUiFamily.Genre,
        BrowseFacetUiFamily.Tag,
        BrowseFacetUiFamily.Person,
        -> if (id.isNullOrBlank()) {
            "This relationship is visible, but the current response does not include the stable id needed to open related Media Items."
        } else {
            "Related Media Items for this ${family.label} cannot be opened from the current state."
        }
        BrowseFacetUiFamily.Library -> "Library-scoped browsing is not available from this app version yet."
        BrowseFacetUiFamily.Studio -> "Studio-based browsing is not available from this app version yet."
        BrowseFacetUiFamily.Collection -> "Collection-based browsing is not available from this app version yet."
        BrowseFacetUiFamily.Year -> "Year-based browsing is not available from this app version yet."
        BrowseFacetUiFamily.ItemKind -> "Media Item kind browsing is not available from this app version yet."
        BrowseFacetUiFamily.SourceMode -> "Playback-mode browsing will become available after playback selection is implemented."
    }
