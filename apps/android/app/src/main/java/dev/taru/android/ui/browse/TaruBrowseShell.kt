package dev.taru.android.ui.browse

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.BrowseResult
import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.browse.SearchRequest
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackResult
import dev.taru.android.playback.SafePlaybackDiagnostics
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionKey
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.playbackLaunchRequest
import dev.taru.android.ui.screens.detail.DetailRouteContent
import dev.taru.android.ui.screens.player.PlaybackPlayerRoute
import dev.taru.android.ui.screens.settings.ServerProfileScreen
import dev.taru.android.ui.screens.settings.SettingsHomeScreen
import dev.taru.android.ui.shell.TaruAdaptiveAppShell
import dev.taru.android.ui.shell.TaruRouteTransition
import dev.taru.android.ui.shell.TaruShellDestination

@Composable
fun TaruBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    playbackClient: TaruPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    onChangeServer: () -> Unit,
    modifier: Modifier = Modifier,
    snapshot: ServerProfileSnapshot = ServerProfileSnapshot(
        profiles = listOf(profile),
        activeProfileId = profile.id,
    ),
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    var navigationState by rememberSaveable(
        profile.id,
        stateSaver = TaruBrowseNavigationStateSaver,
    ) {
        mutableStateOf(TaruBrowseNavigationState.root())
    }
    val selectedDestination = navigationState.selectedDestination
    val route = navigationState.currentRoute
    var refreshKey by remember { mutableIntStateOf(0) }
    var browseState by remember(profile.id, refreshKey) {
        mutableStateOf<BrowseUiState>(BrowseUiState.Loading)
    }
    var detailRefreshKey by remember { mutableIntStateOf(0) }
    var detailState by remember(profile.id, route, detailRefreshKey) {
        mutableStateOf<ItemDetailUiState>(ItemDetailUiState.Idle)
    }
    var playbackRefreshKey by remember { mutableIntStateOf(0) }
    var requestedSourceId by remember(profile.id, route) { mutableStateOf<String?>(null) }
    var playbackState by remember(profile.id, route, playbackRefreshKey) {
        mutableStateOf<PlaybackSelectionUiState>(PlaybackSelectionUiState.Idle)
    }
    var searchQuery by remember(profile.id) { mutableStateOf("") }
    var submittedSearchQuery by remember(profile.id) { mutableStateOf("") }
    var searchRefreshKey by remember { mutableIntStateOf(0) }
    var searchState by remember(profile.id) { mutableStateOf<SearchUiState>(SearchUiState.Idle) }
    var facetRefreshKey by remember { mutableIntStateOf(0) }
    var facetState by remember(profile.id, route, facetRefreshKey) {
        mutableStateOf<FacetUiState>(FacetUiState.Idle)
    }
    val shellDestinations = remember {
        TaruDestination.entries.map { destination ->
            TaruShellDestination(
                value = destination,
                label = destination.label,
                icon = destination.icon,
            )
        }
    }

    LaunchedEffect(profile.id, refreshKey) {
        browseState = loadBrowseState(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
        )
    }

    LaunchedEffect(profile.id, route, detailRefreshKey) {
        val itemRoute = route as? TaruRoute.ItemDetail
        if (itemRoute == null) {
            detailState = ItemDetailUiState.Idle
            return@LaunchedEffect
        }

        detailState = ItemDetailUiState.Loading
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            detailState = ItemDetailUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Re-authenticate this server before opening detail.",
                ),
            )
            return@LaunchedEffect
        }

        detailState = when (
            val result = browseClient.itemDetail(
                profile = profile,
                accessToken = accessToken,
                itemId = itemRoute.itemId,
            )
        ) {
            is BrowseResult.Success -> ItemDetailUiState.Content(result.value)
            is BrowseResult.Failure -> ItemDetailUiState.Failure(result.diagnostics)
        }
    }

    LaunchedEffect(profile.id, route, requestedSourceId, playbackRefreshKey) {
        val sourceId = requestedSourceId
        if (route !is TaruRoute.ItemDetail || sourceId.isNullOrBlank()) {
            playbackState = PlaybackSelectionUiState.Idle
            return@LaunchedEffect
        }

        playbackState = PlaybackSelectionUiState.Loading
        playbackState = loadPlaybackSelectionState(
            profile = profile,
            tokenVault = tokenVault,
            playbackClient = playbackClient,
            sourceId = sourceId,
        )
    }

    LaunchedEffect(profile.id, submittedSearchQuery, searchRefreshKey) {
        if (submittedSearchQuery.isBlank()) {
            searchState = SearchUiState.Idle
            return@LaunchedEffect
        }

        searchState = SearchUiState.Loading
        searchState = loadSearchState(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            query = submittedSearchQuery,
        )
    }

    LaunchedEffect(profile.id, route, facetRefreshKey) {
        val facetRoute = route as? TaruRoute.BrowseFacet
        if (facetRoute == null) {
            facetState = FacetUiState.Idle
            return@LaunchedEffect
        }

        val target = facetRoute.target
        if (!target.isPublicRouteBacked) {
            facetState = target.apiGapState()
            return@LaunchedEffect
        }

        facetState = FacetUiState.Loading
        facetState = loadFacetState(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            target = target,
        )
    }

    BackHandler(enabled = navigationState.canNavigateBack) {
        navigationState = navigationState.navigateBack()
    }

    TaruAdaptiveAppShell(
        modifier = modifier,
        destinations = shellDestinations,
        selectedDestination = selectedDestination,
        navigationVisible = navigationState.navigationVisible,
        onDestinationSelected = {
            navigationState = navigationState.selectDestination(it)
        },
    ) { innerPadding ->
        TaruRouteTransition(
            targetState = route,
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
        ) { currentRoute ->
            when (currentRoute) {
                TaruRoute.TopLevel -> TopLevelContent(
                    profile = profile,
                    selectedDestination = selectedDestination,
                    browseState = browseState,
                    searchQuery = searchQuery,
                    searchState = searchState,
                    snapshot = snapshot,
                    onRetry = { refreshKey += 1 },
                    onSearchQueryChange = { searchQuery = it },
                    onSubmitSearch = {
                        submittedSearchQuery = searchQuery.trim()
                        searchRefreshKey += 1
                    },
                    onRetrySearch = { searchRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenItem = { navigationState = navigationState.open(TaruRoute.ItemDetail(it.id)) },
                    onOpenLibrary = {
                        navigationState = navigationState.selectDestination(TaruDestination.Libraries)
                    },
                    onOpenSearch = {
                        navigationState = navigationState.selectDestination(TaruDestination.Search)
                    },
                    onOpenServerProfile = { navigationState = navigationState.open(TaruRoute.ServerProfile) },
                    onOpenFacet = { navigationState = navigationState.open(TaruRoute.BrowseFacet(it)) },
                )
                is TaruRoute.ItemDetail -> DetailRouteContent(
                    state = detailState,
                    playbackState = playbackState,
                    selectedSourceId = requestedSourceId,
                    deviceResumePositionMs = deviceResumePosition(
                        profileId = profile.id,
                        state = detailState,
                        selectedSourceId = requestedSourceId,
                        positionStore = positionStore,
                    ),
                    onBack = { navigationState = navigationState.navigateBack() },
                    onRetry = { detailRefreshKey += 1 },
                    onRetryPlayback = { playbackRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenFacet = { navigationState = navigationState.open(TaruRoute.BrowseFacet(it)) },
                    onRequestPlayback = {
                        requestedSourceId = it
                        playbackRefreshKey += 1
                    },
                    onStartPlayback = { target ->
                        val detail = (detailState as? ItemDetailUiState.Content)?.response
                        val item = detail?.item
                        val sourceId = requestedSourceId
                            ?: detail?.sources?.firstOrNull()?.id
                            ?: playbackState.contentOrNull()?.response?.source?.id
                        if (item != null && !sourceId.isNullOrBlank()) {
                            val title = item.metadata.title.ifBlank { "Taru Playback" }
                            val positionKey = DevicePlaybackPositionKey(
                                serverProfileId = profile.id,
                                mediaItemId = item.id,
                                sourceId = sourceId,
                            )
                            navigationState = navigationState.open(
                                TaruRoute.Player(
                                    playbackLaunchRequest(
                                        title = title,
                                        target = target,
                                        serverProfileId = profile.id,
                                        mediaItemId = item.id,
                                        sourceId = sourceId,
                                        playbackMode = playbackState.contentOrNull()
                                            ?.response
                                            ?.decision
                                            ?.mode
                                            ?: ClientPlaybackMode.DirectPlay,
                                        sessionId = null,
                                        resumePositionMs = positionKey
                                            .let(positionStore::load)
                                            ?.positionMs,
                                    ),
                                ),
                            )
                        }
                    },
                )
                is TaruRoute.Player -> PlaybackPlayerRoute(
                    launch = currentRoute.launch,
                    profile = profile,
                    tokenVault = tokenVault,
                    playbackClient = playbackClient,
                    positionStore = positionStore,
                    onBack = { navigationState = navigationState.navigateBack() },
                )
                is TaruRoute.BrowseFacet -> BrowseFacetRouteContent(
                    target = currentRoute.target,
                    state = facetState,
                    onBack = { navigationState = navigationState.navigateBack() },
                    onRetry = { facetRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenItem = { navigationState = navigationState.open(TaruRoute.ItemDetail(it.id)) },
                )
                TaruRoute.ServerProfile -> ServerProfileScreen(
                    activeProfile = profile,
                    snapshot = snapshot,
                    tokenVault = tokenVault,
                    onBack = { navigationState = navigationState.navigateBack() },
                    onChangeServer = onChangeServer,
                    onSnapshotChanged = onSnapshotChanged,
                )
            }
        }
    }
}

private fun PlaybackSelectionUiState.contentOrNull(): PlaybackSelectionUiState.Content? =
    this as? PlaybackSelectionUiState.Content

private fun deviceResumePosition(
    profileId: String,
    state: ItemDetailUiState,
    selectedSourceId: String?,
    positionStore: DevicePlaybackPositionStore,
): Long? {
    val detail = (state as? ItemDetailUiState.Content)?.response ?: return null
    val source = detail.sources.firstOrNull { it.id == selectedSourceId } ?: detail.sources.firstOrNull()
    val sourceId = source?.id?.takeIf { it.isNotBlank() } ?: return null
    return positionStore.load(
        DevicePlaybackPositionKey(
            serverProfileId = profileId,
            mediaItemId = detail.item.id,
            sourceId = sourceId,
        ),
    )?.positionMs
}

private suspend fun loadPlaybackSelectionState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    sourceId: String,
): PlaybackSelectionUiState {
    val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return PlaybackSelectionUiState.Failure(
            SafePlaybackDiagnostics(
                category = PlaybackFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before requesting playback.",
            ),
        )
    }

    return when (
        val result = playbackClient.getPlaybackDecision(
            profile = profile,
            accessToken = accessToken,
            sourceId = sourceId,
        )
    ) {
        is PlaybackResult.Success -> PlaybackSelectionUiState.Content(
            response = result.value,
            target = playbackClient.recommendedPlaybackTarget(
                profile = profile,
                accessToken = accessToken,
                decision = result.value,
            ),
        )
        is PlaybackResult.Failure -> PlaybackSelectionUiState.Failure(result.diagnostics)
    }
}

private suspend fun loadSearchState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    query: String,
): SearchUiState {
    val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return SearchUiState.Failure(
            SafeBrowseDiagnostics(
                category = BrowseFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before searching.",
            ),
        )
    }

    return when (
        val result = browseClient.searchItems(
            profile = profile,
            accessToken = accessToken,
            query = SearchRequest(
                query = query,
                page = PageRequest(limit = 24, offset = 0),
            ),
        )
    ) {
        is BrowseResult.Success -> SearchUiState.Content(result.value)
        is BrowseResult.Failure -> SearchUiState.Failure(result.diagnostics)
    }
}

private suspend fun loadFacetState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    target: BrowseFacetTarget,
): FacetUiState {
    val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return FacetUiState.Failure(
            SafeBrowseDiagnostics(
                category = BrowseFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before browsing this facet.",
            ),
        )
    }

    val facetId = target.id.orEmpty()
    val result: BrowseResult<FacetItemsResponse> = when (target.family) {
        BrowseFacetUiFamily.Genre -> browseClient.listGenreItems(
            profile = profile,
            accessToken = accessToken,
            genreId = facetId,
            page = PageRequest(limit = 24, offset = 0),
        )
        BrowseFacetUiFamily.Tag -> browseClient.listTagItems(
            profile = profile,
            accessToken = accessToken,
            tagId = facetId,
            page = PageRequest(limit = 24, offset = 0),
        )
        BrowseFacetUiFamily.Person -> browseClient.listPersonItems(
            profile = profile,
            accessToken = accessToken,
            personId = facetId,
            page = PageRequest(limit = 24, offset = 0),
        )
        else -> return target.apiGapState()
    }

    return when (result) {
        is BrowseResult.Success -> FacetUiState.Content(result.value)
        is BrowseResult.Failure -> FacetUiState.Failure(result.diagnostics)
    }
}

private fun BrowseFacetTarget.apiGapState(): FacetUiState.ApiGap =
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

private suspend fun loadBrowseState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
): BrowseUiState {
    val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return BrowseUiState.Failure(
            SafeBrowseDiagnostics(
                category = BrowseFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before browsing.",
            ),
        )
    }

    val libraries = browseClient.listLibraries(
        profile = profile,
        accessToken = accessToken,
        page = PageRequest(limit = 50, offset = 0),
    )
    if (libraries is BrowseResult.Failure) {
        return BrowseUiState.Failure(libraries.diagnostics)
    }

    val items = browseClient.listItems(
        profile = profile,
        accessToken = accessToken,
        page = PageRequest(limit = 24, offset = 0),
    )
    if (items is BrowseResult.Failure) {
        return BrowseUiState.Failure(items.diagnostics)
    }

    return BrowseUiState.Content(
        libraries = (libraries as BrowseResult.Success).value,
        items = (items as BrowseResult.Success).value,
    )
}

@Composable
private fun TopLevelContent(
    profile: ServerProfile,
    selectedDestination: TaruDestination,
    browseState: BrowseUiState,
    searchQuery: String,
    searchState: SearchUiState,
    snapshot: ServerProfileSnapshot,
    onRetry: () -> Unit,
    onSearchQueryChange: (String) -> Unit,
    onSubmitSearch: () -> Unit,
    onRetrySearch: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenLibrary: () -> Unit,
    onOpenSearch: () -> Unit,
    onOpenServerProfile: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
) {
    when (selectedDestination) {
        TaruDestination.Home -> BrowseScaffoldContent {
            HomeScreen(
                profile = profile,
                state = browseState,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
                onOpenItem = onOpenItem,
                onOpenLibrary = onOpenLibrary,
                onOpenSearch = onOpenSearch,
                onOpenFacet = onOpenFacet,
            )
        }
        TaruDestination.Libraries -> BrowseScaffoldContent {
            LibrariesScreen(
                state = browseState,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
                onOpenItem = onOpenItem,
                onOpenFacet = onOpenFacet,
            )
        }
        TaruDestination.Search -> BrowseScaffoldContent {
            SearchScreen(
                query = searchQuery,
                state = searchState,
                onQueryChange = onSearchQueryChange,
                onSubmit = onSubmitSearch,
                onRetry = onRetrySearch,
                onChangeServer = onChangeServer,
                onOpenItem = onOpenItem,
            )
        }
        TaruDestination.Settings -> BrowseScaffoldContent {
            SettingsHomeScreen(
                profile = profile,
                snapshot = snapshot,
                onChangeServer = onChangeServer,
                onOpenServerProfile = onOpenServerProfile,
            )
        }
    }
}

@Composable
internal fun BrowseScaffoldContent(content: @Composable () -> Unit) {
    Box(
        modifier = Modifier.fillMaxSize(),
    ) {
        content()
    }
}
