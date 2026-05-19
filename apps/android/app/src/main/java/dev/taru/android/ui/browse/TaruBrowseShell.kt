package dev.taru.android.ui.browse

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
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
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackResult
import dev.taru.android.playback.SafePlaybackDiagnostics
import dev.taru.android.playback.PlaybackFailureCategory
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
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

@Composable
fun TaruBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    playbackClient: TaruPlaybackClient,
    playbackPreferencesStore: PlaybackPreferencesStore,
    userPlaybackClient: TaruUserPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    playerExitEffectScope: CoroutineScope,
    onChangeServer: () -> Unit,
    modifier: Modifier = Modifier,
    snapshot: ServerProfileSnapshot = ServerProfileSnapshot(
        profiles = listOf(profile),
        activeProfileId = profile.id,
    ),
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    val routeScope = rememberCoroutineScope()
    val browseDataSource = remember(profile, tokenVault, browseClient, userPlaybackClient) {
        ClientBrowseDataSource(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            userPlaybackClient = userPlaybackClient,
        )
    }
    var savedShellState by rememberSaveable(
        profile.id,
        stateSaver = BrowseShellStateSaver,
    ) {
        mutableStateOf(BrowseShellState())
    }
    val browseSession = remember(profile.id, browseDataSource, routeScope) {
        BrowseSession(
            initialState = savedShellState,
            dataSource = browseDataSource,
            scope = routeScope,
        )
    }
    val shellState by browseSession.state.collectAsState()
    val selectedDestination = shellState.selectedDestination
    val route = shellState.currentRoute
    fun dispatchBrowseAction(action: BrowseAction) {
        browseSession.dispatch(action)
        savedShellState = browseSession.state.value
    }
    var detailRefreshKey by remember { mutableIntStateOf(0) }
    var detailState by remember(profile.id, route, detailRefreshKey) {
        mutableStateOf<ItemDetailUiState>(ItemDetailUiState.Idle)
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

    LaunchedEffect(profile.id) {
        browseSession.dispatch(BrowseAction.LoadHome)
    }

    LaunchedEffect(profile.id, route) {
        browseSession.dispatch(BrowseAction.RouteDisplayed(route))
    }

    BackHandler(enabled = shellState.canNavigateBack) {
        dispatchBrowseAction(BrowseAction.Back)
    }

    TaruAdaptiveAppShell(
        modifier = modifier,
        destinations = shellDestinations,
        selectedDestination = selectedDestination,
        navigationVisible = shellState.navigationVisible,
        onDestinationSelected = {
            dispatchBrowseAction(BrowseAction.SelectDestination(it))
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
                    browseState = shellState.browseState,
                    searchQuery = shellState.searchQuery,
                    searchState = shellState.searchState,
                    snapshot = snapshot,
                    artworkSource = artworkSource,
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryHome) },
                    onSearchQueryChange = { dispatchBrowseAction(BrowseAction.SearchQueryChanged(it)) },
                    onSubmitSearch = { dispatchBrowseAction(BrowseAction.SubmitSearch) },
                    onRetrySearch = { dispatchBrowseAction(BrowseAction.RetrySearch) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { dispatchBrowseAction(BrowseAction.OpenItem(it.id)) },
                    onOpenLibrary = {
                        dispatchBrowseAction(BrowseAction.SelectDestination(TaruDestination.Libraries))
                    },
                    onOpenSearch = {
                        dispatchBrowseAction(BrowseAction.SelectDestination(TaruDestination.Search))
                    },
                    onOpenServerProfile = { dispatchBrowseAction(BrowseAction.OpenServerProfile) },
                    onOpenFacet = { dispatchBrowseAction(BrowseAction.OpenFacet(it)) },
                    onOpenLibraryDetail = { libraryId ->
                        dispatchBrowseAction(BrowseAction.OpenLibraryDetail(libraryId))
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
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { detailRefreshKey += 1 },
                    onRetryPlayback = { playbackRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenFacet = { dispatchBrowseAction(BrowseAction.OpenFacet(it)) },
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
                                        dispatchBrowseAction(BrowseAction.OpenPlayer(start.launch))
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
                    state = shellState.libraryDetailState,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { itemId ->
                        dispatchBrowseAction(BrowseAction.OpenItem(itemId))
                    },
                )
                is TaruRoute.Player -> PlaybackPlayerRoute(
                    launch = currentRoute.launch,
                    profile = profile,
                    tokenVault = tokenVault,
                    playbackClient = playbackClient,
                    userPlaybackClient = userPlaybackClient,
                    positionStore = positionStore,
                    exitEffectScope = playerExitEffectScope,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                )
                is TaruRoute.BrowseFacet -> BrowseFacetRouteContent(
                    target = currentRoute.target,
                    state = shellState.facetState,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { dispatchBrowseAction(BrowseAction.OpenItem(it.id)) },
                )
                TaruRoute.ServerProfile -> ServerProfileScreen(
                    activeProfile = profile,
                    snapshot = snapshot,
                    tokenVault = tokenVault,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
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
