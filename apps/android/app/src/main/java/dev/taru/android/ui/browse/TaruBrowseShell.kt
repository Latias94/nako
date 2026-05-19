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
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.BrowseResult
import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.PublicImageRefDto
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
import dev.taru.android.playback.PlaybackPreferencesStore
import dev.taru.android.playback.PlaybackStartCoordinator
import dev.taru.android.playback.PlaybackStartRequest
import dev.taru.android.playback.PlaybackStartResult
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.resolvePlaybackResumePosition
import dev.taru.android.artwork.PublicArtworkSource
import dev.taru.android.ui.screens.detail.DetailRouteContent
import dev.taru.android.ui.screens.player.PlaybackPlayerRoute
import dev.taru.android.ui.screens.settings.ServerProfileScreen
import dev.taru.android.ui.screens.settings.SettingsHomeScreen
import dev.taru.android.ui.shell.TaruAdaptiveAppShell
import dev.taru.android.ui.shell.TaruRouteTransition
import dev.taru.android.ui.shell.TaruShellDestination
import dev.taru.android.userplayback.TaruUserPlaybackClient
import dev.taru.android.userplayback.UserPlaybackResult
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit

@Composable
fun TaruBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    playbackClient: TaruPlaybackClient,
    playbackPreferencesStore: PlaybackPreferencesStore,
    userPlaybackClient: TaruUserPlaybackClient,
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
    val routeScope = rememberCoroutineScope()
    var refreshKey by remember { mutableIntStateOf(0) }
    var browseState by remember(profile.id, refreshKey) {
        mutableStateOf<BrowseUiState>(BrowseUiState.Loading)
    }
    var detailRefreshKey by remember { mutableIntStateOf(0) }
    var detailState by remember(profile.id, route, detailRefreshKey) {
        mutableStateOf<ItemDetailUiState>(ItemDetailUiState.Idle)
    }
    var libraryDetailRefreshKey by remember { mutableIntStateOf(0) }
    var libraryDetailState by remember(profile.id, route, libraryDetailRefreshKey) {
        mutableStateOf<LibraryDetailUiState>(LibraryDetailUiState.Idle)
    }
    var sourceProbeRefreshKey by remember { mutableIntStateOf(0) }
    var selectedSourceId by remember(profile.id, route) { mutableStateOf<String?>(null) }
    var sourceProbeState by remember(profile.id, route, selectedSourceId, sourceProbeRefreshKey) {
        mutableStateOf<SourceProbeUiState>(SourceProbeUiState.Idle)
    }
    var playbackRefreshKey by remember { mutableIntStateOf(0) }
    var playbackRequestSourceId by remember(profile.id, route) { mutableStateOf<String?>(null) }
    var playbackState by remember(profile.id, route, playbackRequestSourceId, playbackRefreshKey) {
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
    val artworkSource = PublicArtworkSource(
        profile = profile,
        accessToken = tokenVault.readToken(profile.tokenReference).orEmpty(),
    )
    val playbackStartCoordinator = remember(playbackClient, positionStore) {
        PlaybackStartCoordinator(
            playbackClient = playbackClient,
            positionStore = positionStore,
        )
    }

    LaunchedEffect(profile.id, refreshKey) {
        browseState = loadBrowseState(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            userPlaybackClient = userPlaybackClient,
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
            is BrowseResult.Success -> {
                val userPlaybackState = when (
                    val stateResult = userPlaybackClient.getState(
                        profile = profile,
                        accessToken = accessToken,
                        itemId = itemRoute.itemId,
                    )
                ) {
                    is UserPlaybackResult.Success -> stateResult.value.state
                    is UserPlaybackResult.Failure -> null
                }
                selectedSourceId = result.value.sources.firstOrNull()?.id
                playbackRequestSourceId = null
                ItemDetailUiState.Content(
                    response = result.value,
                    userPlaybackState = userPlaybackState,
                )
            }
            is BrowseResult.Failure -> ItemDetailUiState.Failure(result.diagnostics)
        }
    }

    LaunchedEffect(profile.id, route, libraryDetailRefreshKey) {
        val libraryRoute = route as? TaruRoute.LibraryDetail
        if (libraryRoute == null) {
            libraryDetailState = LibraryDetailUiState.Idle
            return@LaunchedEffect
        }

        libraryDetailState = LibraryDetailUiState.Loading
        libraryDetailState = loadLibraryDetailState(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            libraryId = libraryRoute.libraryId,
        )
    }

    LaunchedEffect(profile.id, route, selectedSourceId, sourceProbeRefreshKey) {
        val sourceId = selectedSourceId
        if (route !is TaruRoute.ItemDetail || sourceId.isNullOrBlank()) {
            sourceProbeState = SourceProbeUiState.Idle
            return@LaunchedEffect
        }

        sourceProbeState = SourceProbeUiState.Loading
        sourceProbeState = loadSourceProbeState(
            profile = profile,
            tokenVault = tokenVault,
            playbackClient = playbackClient,
            sourceId = sourceId,
        )
    }

    LaunchedEffect(profile.id, route, playbackRequestSourceId, playbackRefreshKey) {
        val sourceId = playbackRequestSourceId
        if (route !is TaruRoute.ItemDetail || sourceId.isNullOrBlank()) {
            playbackState = PlaybackSelectionUiState.Idle
            return@LaunchedEffect
        }

        playbackState = PlaybackSelectionUiState.Loading
        playbackState = loadPlaybackSelectionState(
            profile = profile,
            tokenVault = tokenVault,
            playbackClient = playbackClient,
            playbackPreferencesStore = playbackPreferencesStore,
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
                    artworkSource = artworkSource,
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
                    onOpenLibraryDetail = { libraryId ->
                        navigationState = navigationState.open(TaruRoute.LibraryDetail(libraryId))
                    },
                )
                is TaruRoute.ItemDetail -> DetailRouteContent(
                    state = detailState,
                    sourceProbeState = sourceProbeState,
                    playbackState = playbackState,
                    selectedSourceId = selectedSourceId,
                    resumePosition = detailResumePosition(
                        profileId = profile.id,
                        state = detailState,
                        selectedSourceId = selectedSourceId,
                        positionStore = positionStore,
                    ),
                    profile = profile,
                    accessToken = tokenVault.readToken(profile.tokenReference).orEmpty(),
                    onBack = { navigationState = navigationState.navigateBack() },
                    onRetry = { detailRefreshKey += 1 },
                    onRetryPlayback = { playbackRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenFacet = { navigationState = navigationState.open(TaruRoute.BrowseFacet(it)) },
                    onSelectSource = { sourceId ->
                        selectedSourceId = sourceId
                        playbackRequestSourceId = null
                    },
                    onRetrySourceProbe = { sourceProbeRefreshKey += 1 },
                    onRequestPlayback = {
                        selectedSourceId = it
                        playbackRequestSourceId = it
                        playbackRefreshKey += 1
                    },
                    onStartPlayback = { target ->
                        val detailContent = detailState as? ItemDetailUiState.Content
                        val detail = detailContent?.response
                        val item = detail?.item
                        val sourceId = selectedSourceId
                            ?: detail?.sources?.firstOrNull()?.id
                            ?: playbackState.contentOrNull()?.response?.source?.id
                        val playbackContent = playbackState.contentOrNull()
                        if (item != null && !sourceId.isNullOrBlank() && playbackContent != null) {
                            routeScope.launch {
                                playbackState = PlaybackSelectionUiState.Loading
                                when (val start = playbackStartCoordinator.start(
                                        profile = profile,
                                        tokenVault = tokenVault,
                                        request = PlaybackStartRequest(
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
                                        navigationState = navigationState.open(
                                            TaruRoute.Player(start.launch),
                                        )
                                        playbackState = playbackContent.copy(target = start.preparedTarget)
                                    }
                                    is PlaybackStartResult.Failure -> {
                                        playbackState = PlaybackSelectionUiState.Failure(start.diagnostics)
                                    }
                                }
                            }
                        }
                    },
                )
                is TaruRoute.LibraryDetail -> LibraryDetailRouteContent(
                    state = libraryDetailState,
                    onBack = { navigationState = navigationState.navigateBack() },
                    onRetry = { libraryDetailRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenItem = { itemId ->
                        navigationState = navigationState.open(TaruRoute.ItemDetail(itemId))
                    },
                )
                is TaruRoute.Player -> PlaybackPlayerRoute(
                    launch = currentRoute.launch,
                    profile = profile,
                    tokenVault = tokenVault,
                    playbackClient = playbackClient,
                    userPlaybackClient = userPlaybackClient,
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

private fun detailResumePosition(
    profileId: String,
    state: ItemDetailUiState,
    selectedSourceId: String?,
    positionStore: DevicePlaybackPositionStore,
): dev.taru.android.player.ResumePlaybackPosition? {
    val content = state as? ItemDetailUiState.Content ?: return null
    val detail = content.response
    val source = detail.sources.firstOrNull { it.id == selectedSourceId } ?: detail.sources.firstOrNull()
    val sourceId = source?.id?.takeIf { it.isNotBlank() } ?: return null
    return resolvePlaybackResumePosition(
        profileId = profileId,
        mediaItemId = detail.item.id,
        sourceId = sourceId,
        userPlaybackState = content.userPlaybackState,
        positionStore = positionStore,
    )
}

private suspend fun loadSourceProbeState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    sourceId: String,
): SourceProbeUiState {
    val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return SourceProbeUiState.Failure(
            SafePlaybackDiagnostics(
                category = PlaybackFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before loading source facts.",
            ),
        )
    }

    return when (
        val result = playbackClient.getSourceProbe(
            profile = profile,
            accessToken = accessToken,
            sourceId = sourceId,
        )
    ) {
        is PlaybackResult.Success -> SourceProbeUiState.Content(result.value)
        is PlaybackResult.Failure -> SourceProbeUiState.Failure(result.diagnostics)
    }
}

private suspend fun loadPlaybackSelectionState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    playbackPreferencesStore: PlaybackPreferencesStore,
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

    val capabilities = playbackPreferencesStore.loadCapabilities(profile.id)

    return when (
        val result = playbackClient.getPlaybackDecision(
            profile = profile,
            accessToken = accessToken,
            sourceId = sourceId,
            capabilities = capabilities,
        )
    ) {
        is PlaybackResult.Success -> PlaybackSelectionUiState.Content(
            response = result.value,
            target = playbackClient.recommendedPlaybackTarget(
                profile = profile,
                accessToken = accessToken,
                decision = result.value,
                capabilities = capabilities,
            ),
            capabilities = capabilities,
        )
        is PlaybackResult.Failure -> PlaybackSelectionUiState.Failure(result.diagnostics)
    }
}

private suspend fun loadLibraryDetailState(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    libraryId: String,
): LibraryDetailUiState {
    val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
    if (accessToken.isBlank()) {
        return LibraryDetailUiState.Failure(
            SafeBrowseDiagnostics(
                category = BrowseFailureCategory.MissingAccessToken,
                userMessage = "Re-authenticate this server before opening library detail.",
            ),
        )
    }

    val detail = browseClient.libraryDetail(
        profile = profile,
        accessToken = accessToken,
        libraryId = libraryId,
    )
    if (detail is BrowseResult.Failure) {
        return LibraryDetailUiState.Failure(detail.diagnostics)
    }

    return when (
        val sources = browseClient.librarySources(
            profile = profile,
            accessToken = accessToken,
            libraryId = libraryId,
            page = PageRequest(limit = 24, offset = 0),
        )
    ) {
        is BrowseResult.Success -> LibraryDetailUiState.Content(sources.value)
        is BrowseResult.Failure -> LibraryDetailUiState.Failure(sources.diagnostics)
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
    userPlaybackClient: TaruUserPlaybackClient,
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
    val itemPage = (items as BrowseResult.Success).value
    val continueWatching = when (
        val result = userPlaybackClient.continueWatching(
            profile = profile,
            accessToken = accessToken,
            page = PageRequest(limit = 12, offset = 0),
        )
    ) {
        is UserPlaybackResult.Success -> result.value
        is UserPlaybackResult.Failure -> null
    }
    val visibleArtwork = loadVisibleArtworkRefs(
        profile = profile,
        accessToken = accessToken,
        browseClient = browseClient,
        items = itemPage.items,
    )
    val continueArtwork = continueWatching
        ?.items
        ?.mapNotNull { row ->
            row.images
                .takeIf { it.isNotEmpty() }
                ?.let { row.item.id to it }
        }
        ?.toMap()
        .orEmpty()

    return BrowseUiState.Content(
        libraries = (libraries as BrowseResult.Success).value,
        items = itemPage,
        artworkByItemId = visibleArtwork + continueArtwork,
        continueWatching = continueWatching,
    )
}

private suspend fun loadVisibleArtworkRefs(
    profile: ServerProfile,
    accessToken: String,
    browseClient: TaruBrowseClient,
    items: List<MediaItemDto>,
): Map<String, List<PublicImageRefDto>> = coroutineScope {
    val semaphore = Semaphore(permits = 4)
    items
        .map { item ->
            async {
                semaphore.withPermit {
                    val result = browseClient.itemImages(
                        profile = profile,
                        accessToken = accessToken,
                        itemId = item.id,
                    )
                    if (result is BrowseResult.Success && result.value.images.isNotEmpty()) {
                        item.id to result.value.images
                    } else {
                        null
                    }
                }
            }
        }
        .awaitAll()
        .filterNotNull()
        .toMap()
}

@Composable
private fun TopLevelContent(
    profile: ServerProfile,
    selectedDestination: TaruDestination,
    browseState: BrowseUiState,
    searchQuery: String,
    searchState: SearchUiState,
    snapshot: ServerProfileSnapshot,
    artworkSource: PublicArtworkSource,
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
    onOpenLibraryDetail: (String) -> Unit,
) {
    when (selectedDestination) {
        TaruDestination.Home -> BrowseScaffoldContent {
            HomeScreen(
                profile = profile,
                state = browseState,
                artworkSource = artworkSource,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
                onOpenItem = onOpenItem,
                onOpenLibrary = onOpenLibrary,
                onOpenLibraryDetail = onOpenLibraryDetail,
                onOpenSearch = onOpenSearch,
                onOpenFacet = onOpenFacet,
            )
        }
        TaruDestination.Libraries -> BrowseScaffoldContent {
            LibrariesScreen(
                state = browseState,
                artworkSource = artworkSource,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
                onOpenLibrary = { library -> onOpenLibraryDetail(library.id) },
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
