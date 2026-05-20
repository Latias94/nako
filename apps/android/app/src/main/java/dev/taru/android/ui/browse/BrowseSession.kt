package dev.taru.android.ui.browse

import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.PlaybackStartRequest
import dev.taru.android.playback.PlaybackStartResult
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
    val selectedDestination: TaruDestination = navigation.selectedDestination
    val currentRoute: TaruRoute = navigation.currentRoute
    val navigationVisible: Boolean = navigation.navigationVisible
    val canNavigateBack: Boolean = navigation.canNavigateBack
}

internal sealed interface BrowseAction {
    data class SelectDestination(val destination: TaruDestination) : BrowseAction
    data class OpenItem(val itemId: String) : BrowseAction
    data class OpenLibraryDetail(val libraryId: String) : BrowseAction
    data class OpenPersonDetail(val personId: String) : BrowseAction
    data class OpenRelationshipIndex(val family: RelationshipIndexFamily) : BrowseAction
    data class OpenFacet(val target: BrowseFacetTarget) : BrowseAction
    data class OpenPlayer(val launch: PlaybackLaunchRequest) : BrowseAction
    data class RouteDisplayed(val route: TaruRoute) : BrowseAction
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
    data object RetryCurrentRoute : BrowseAction
    data object RetrySourceProbe : BrowseAction
    data object RetryPlaybackDecision : BrowseAction
}

internal interface BrowseDataSource {
    suspend fun loadHome(): BrowseUiState
    suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState
    suspend fun search(query: String): SearchUiState
    suspend fun loadPersonDetail(personId: String): PersonDetailUiState
    suspend fun loadRelationshipIndex(family: RelationshipIndexFamily): RelationshipIndexUiState
    suspend fun loadFacet(target: BrowseFacetTarget): FacetUiState
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
    val state: StateFlow<BrowseShellState> = _state.asStateFlow()

    private var homeRequestId: Long = 0
    private var libraryDetailRequestId: Long = 0
    private var searchRequestId: Long = 0
    private var personDetailRequestId: Long = 0
    private var relationshipIndexRequestId: Long = 0
    private var facetRequestId: Long = 0
    private var detailRequestId: Long = 0
    private var sourceProbeRequestId: Long = 0
    private var playbackSelectionRequestId: Long = 0

    fun dispatch(action: BrowseAction): Job? =
        when (action) {
            BrowseAction.LoadHome,
            BrowseAction.RetryHome,
            -> loadHome()
            BrowseAction.SubmitSearch -> submitSearch()
            BrowseAction.RetrySearch -> retrySearch()
            BrowseAction.RetryCurrentRoute -> loadCurrentRoute()
            BrowseAction.RetrySourceProbe -> loadSelectedSourceProbe()
            BrowseAction.RetryPlaybackDecision -> loadRequestedPlaybackSelection()
            is BrowseAction.StartPlayback -> startPlayback(action.target)
            is BrowseAction.RouteDisplayed -> loadRoute(action.route)
            is BrowseAction.SearchQueryChanged -> {
                _state.update { it.copy(searchQuery = action.query) }
                null
            }
            is BrowseAction.SelectSource -> {
                sourceProbeRequestId += 1
                playbackSelectionRequestId += 1
                _state.update {
                    it.copy(
                        selectedSourceId = action.sourceId,
                        resumePosition = resolveResumePosition(it.detailState, action.sourceId),
                        sourceProbeState = SourceProbeUiState.Loading,
                        playbackRequestSourceId = null,
                        playbackState = PlaybackSelectionUiState.Idle,
                    )
                }
                loadSourceProbe(action.sourceId)
            }
            is BrowseAction.RequestPlayback -> {
                sourceProbeRequestId += 1
                playbackSelectionRequestId += 1
                _state.update {
                    it.copy(
                        selectedSourceId = action.sourceId,
                        resumePosition = resolveResumePosition(it.detailState, action.sourceId),
                        sourceProbeState = SourceProbeUiState.Loading,
                        playbackRequestSourceId = action.sourceId,
                        playbackState = PlaybackSelectionUiState.Loading,
                    )
                }
                loadSourceProbe(action.sourceId)
                loadPlaybackSelection(action.sourceId)
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
            is BrowseAction.OpenPersonDetail -> current.copy(
                navigation = current.navigation.open(TaruRoute.PersonDetail(action.personId)),
            )
            is BrowseAction.OpenRelationshipIndex -> current.copy(
                navigation = current.navigation.open(TaruRoute.RelationshipIndex(action.family)),
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
            BrowseAction.RetrySourceProbe,
            BrowseAction.RetryPlaybackDecision,
            is BrowseAction.RouteDisplayed,
            is BrowseAction.SearchQueryChanged,
            is BrowseAction.SelectSource,
            is BrowseAction.RequestPlayback,
            is BrowseAction.StartPlayback,
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
            is TaruRoute.ItemDetail -> loadItemDetail(route.itemId)
            is TaruRoute.LibraryDetail -> loadLibraryDetail(route.libraryId)
            is TaruRoute.PersonDetail -> loadPersonDetail(route.personId)
            is TaruRoute.RelationshipIndex -> loadRelationshipIndex(route.family)
            is TaruRoute.BrowseFacet -> loadFacet(route.target)
            else -> {
                detailRequestId += 1
                libraryDetailRequestId += 1
                personDetailRequestId += 1
                relationshipIndexRequestId += 1
                facetRequestId += 1
                sourceProbeRequestId += 1
                playbackSelectionRequestId += 1
                _state.update {
                    it.copy(
                        detailState = ItemDetailUiState.Idle,
                        selectedSourceId = null,
                        resumePosition = null,
                        sourceProbeState = SourceProbeUiState.Idle,
                        playbackRequestSourceId = null,
                        playbackState = PlaybackSelectionUiState.Idle,
                        libraryDetailState = LibraryDetailUiState.Idle,
                        personDetailState = PersonDetailUiState.Idle,
                        relationshipIndexState = RelationshipIndexUiState.Idle,
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
        if (next.currentRoute is TaruRoute.Player) {
            return next
        }

        var prepared = next
        prepared = when (val route = next.currentRoute) {
            is TaruRoute.ItemDetail -> {
                detailRequestId += 1
                sourceProbeRequestId += 1
                playbackSelectionRequestId += 1
                prepared.copy(
                    detailState = ItemDetailUiState.Loading,
                    selectedSourceId = null,
                    resumePosition = null,
                    sourceProbeState = SourceProbeUiState.Idle,
                    playbackRequestSourceId = null,
                    playbackState = PlaybackSelectionUiState.Idle,
                )
            }
            else -> {
                detailRequestId += 1
                sourceProbeRequestId += 1
                playbackSelectionRequestId += 1
                prepared.copy(
                    detailState = ItemDetailUiState.Idle,
                    selectedSourceId = null,
                    resumePosition = null,
                    sourceProbeState = SourceProbeUiState.Idle,
                    playbackRequestSourceId = null,
                    playbackState = PlaybackSelectionUiState.Idle,
                )
            }
        }
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
            is TaruRoute.PersonDetail -> {
                personDetailRequestId += 1
                prepared.copy(personDetailState = PersonDetailUiState.Loading)
            }
            else -> {
                personDetailRequestId += 1
                prepared.copy(personDetailState = PersonDetailUiState.Idle)
            }
        }
        prepared = when (val route = next.currentRoute) {
            is TaruRoute.RelationshipIndex -> {
                relationshipIndexRequestId += 1
                prepared.copy(relationshipIndexState = RelationshipIndexUiState.Loading)
            }
            else -> {
                relationshipIndexRequestId += 1
                prepared.copy(relationshipIndexState = RelationshipIndexUiState.Idle)
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

    private fun loadItemDetail(itemId: String): Job {
        val requestId = ++detailRequestId
        _state.update {
            it.copy(
                detailState = ItemDetailUiState.Loading,
                selectedSourceId = null,
                resumePosition = null,
                sourceProbeState = SourceProbeUiState.Idle,
                playbackRequestSourceId = null,
                playbackState = PlaybackSelectionUiState.Idle,
            )
        }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadItemDetail(itemId)
            var acceptedSourceId: String? = null
            _state.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.ItemDetail(itemId)
                if (requestId == detailRequestId && routeStillCurrent) {
                    val selectedSourceId = nextState.firstSourceIdOrNull()
                    acceptedSourceId = selectedSourceId
                    current.copy(
                        detailState = nextState,
                        selectedSourceId = selectedSourceId,
                        resumePosition = resolveResumePosition(nextState, selectedSourceId),
                        sourceProbeState = if (selectedSourceId == null) {
                            SourceProbeUiState.Idle
                        } else {
                            SourceProbeUiState.Loading
                        },
                        playbackRequestSourceId = null,
                        playbackState = PlaybackSelectionUiState.Idle,
                    )
                } else {
                    current
                }
            }
            acceptedSourceId?.let { sourceId ->
                if (state.value.currentRoute == TaruRoute.ItemDetail(itemId)) {
                    loadSourceProbe(sourceId)
                }
            }
        }
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

    private fun loadPersonDetail(personId: String): Job {
        val requestId = ++personDetailRequestId
        _state.update { it.copy(personDetailState = PersonDetailUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadPersonDetail(personId)
            _state.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.PersonDetail(personId)
                if (requestId == personDetailRequestId && routeStillCurrent) {
                    current.copy(personDetailState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadRelationshipIndex(family: RelationshipIndexFamily): Job {
        val requestId = ++relationshipIndexRequestId
        _state.update { it.copy(relationshipIndexState = RelationshipIndexUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadRelationshipIndex(family)
            _state.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.RelationshipIndex(family)
                if (requestId == relationshipIndexRequestId && routeStillCurrent) {
                    current.copy(relationshipIndexState = nextState)
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

    private fun loadSelectedSourceProbe(): Job? {
        val sourceId = _state.value.selectedSourceId?.takeIf { it.isNotBlank() }
        return if (sourceId == null || _state.value.currentRoute !is TaruRoute.ItemDetail) {
            sourceProbeRequestId += 1
            _state.update { it.copy(sourceProbeState = SourceProbeUiState.Idle) }
            null
        } else {
            loadSourceProbe(sourceId)
        }
    }

    private fun loadSourceProbe(sourceId: String): Job {
        val requestId = ++sourceProbeRequestId
        _state.update { it.copy(sourceProbeState = SourceProbeUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadSourceProbe(sourceId)
            _state.update { current ->
                val routeStillCurrent = current.currentRoute is TaruRoute.ItemDetail
                if (
                    requestId == sourceProbeRequestId &&
                    routeStillCurrent &&
                    current.selectedSourceId == sourceId
                ) {
                    current.copy(sourceProbeState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadRequestedPlaybackSelection(): Job? {
        val sourceId = _state.value.playbackRequestSourceId?.takeIf { it.isNotBlank() }
        return if (sourceId == null || _state.value.currentRoute !is TaruRoute.ItemDetail) {
            playbackSelectionRequestId += 1
            _state.update { it.copy(playbackState = PlaybackSelectionUiState.Idle) }
            null
        } else {
            loadPlaybackSelection(sourceId)
        }
    }

    private fun loadPlaybackSelection(sourceId: String): Job {
        val requestId = ++playbackSelectionRequestId
        _state.update { it.copy(playbackState = PlaybackSelectionUiState.Loading) }
        return requiredScope().launch {
            val nextState = requiredDataSource().loadPlaybackSelection(sourceId)
            _state.update { current ->
                val routeStillCurrent = current.currentRoute is TaruRoute.ItemDetail
                if (
                    requestId == playbackSelectionRequestId &&
                    routeStillCurrent &&
                    current.playbackRequestSourceId == sourceId
                ) {
                    current.copy(playbackState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun startPlayback(target: PlaybackRequestTarget): Job? {
        val current = _state.value
        val detailContent = current.detailState as? ItemDetailUiState.Content ?: return null
        val playbackContent = current.playbackState as? PlaybackSelectionUiState.Content ?: return null
        val detail = detailContent.response
        val item = detail.item
        val sourceId = current.selectedSourceId
            ?: detail.sources.firstOrNull()?.id
            ?: playbackContent.response.source.id
        if (sourceId.isBlank()) {
            return null
        }

        _state.update { it.copy(playbackState = PlaybackSelectionUiState.Loading) }
        return requiredScope().launch {
            when (
                val start = requiredPlaybackStarter().start(
                    PlaybackStartRequest(
                        title = item.metadata.title,
                        mediaItemId = item.id,
                        sourceId = sourceId,
                        decision = playbackContent.response,
                        capabilities = playbackContent.capabilities,
                        target = target,
                        userPlaybackState = detailContent.userPlaybackState,
                    ),
                )
            ) {
                is PlaybackStartResult.Success -> {
                    _state.update { beforeOpen ->
                        beforeOpen.copy(
                            playbackState = playbackContent.copy(target = start.preparedTarget),
                        )
                    }
                    _state.value = prepareRouteState(
                        previous = _state.value,
                        next = reduce(_state.value, BrowseAction.OpenPlayer(start.launch)),
                    )
                }
                is PlaybackStartResult.Failure -> {
                    _state.update {
                        it.copy(
                            playbackState = PlaybackSelectionUiState.Failure(start.diagnostics),
                        )
                    }
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

    private fun requiredPlaybackStarter(): BrowsePlaybackStarter =
        requireNotNull(playbackStarter) {
            "BrowsePlaybackStarter is required for playback start action handling."
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

private fun ItemDetailUiState.firstSourceIdOrNull(): String? =
    (this as? ItemDetailUiState.Content)
        ?.response
        ?.sources
        ?.firstOrNull()
        ?.id

private data object EmptyBrowseResumeResolver : BrowseResumeResolver {
    override fun resolve(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition? = null
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
