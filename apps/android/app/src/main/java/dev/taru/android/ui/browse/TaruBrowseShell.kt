package dev.taru.android.ui.browse

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackPreferencesStore
import dev.taru.android.playback.PlaybackStartCoordinator
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.ui.artwork.ArtworkRequestResolver
import dev.taru.android.ui.artwork.TokenVaultArtworkRequestResolver
import dev.taru.android.ui.screens.detail.DetailRouteContent
import dev.taru.android.ui.screens.player.PlayerRouteRenderer
import dev.taru.android.ui.screens.settings.ServerProfileScreen
import dev.taru.android.ui.screens.settings.SettingsAction
import dev.taru.android.ui.screens.settings.SettingsHomeScreen
import dev.taru.android.ui.screens.settings.SettingsRuntime
import dev.taru.android.ui.screens.settings.SettingsSession
import dev.taru.android.ui.shell.TaruAdaptiveAppShell
import dev.taru.android.ui.shell.TaruRouteTransition
import dev.taru.android.ui.shell.TaruShellDestination
import dev.taru.android.userplayback.TaruUserPlaybackClient

@Composable
internal fun TaruBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    playbackClient: TaruPlaybackClient,
    playbackPreferencesStore: PlaybackPreferencesStore,
    userPlaybackClient: TaruUserPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    playerRouteRenderer: PlayerRouteRenderer,
    onChangeServer: () -> Unit,
    modifier: Modifier = Modifier,
    snapshot: ServerProfileSnapshot = ServerProfileSnapshot(
        profiles = listOf(profile),
        activeProfileId = profile.id,
    ),
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    val routeScope = rememberCoroutineScope()
    val playbackStartCoordinator = remember(playbackClient, positionStore) {
        PlaybackStartCoordinator(
            playbackClient = playbackClient,
            positionStore = positionStore,
        )
    }
    val browseDataSource = remember(profile, tokenVault, browseClient, userPlaybackClient) {
        ClientBrowseDataSource(
            profile = profile,
            tokenVault = tokenVault,
            browseClient = browseClient,
            playbackClient = playbackClient,
            playbackPreferencesStore = playbackPreferencesStore,
            userPlaybackClient = userPlaybackClient,
        )
    }
    val playbackStarter = remember(profile, tokenVault, playbackStartCoordinator) {
        ClientBrowsePlaybackStarter(
            profile = profile,
            tokenVault = tokenVault,
            coordinator = playbackStartCoordinator,
        )
    }
    val resumeResolver = remember(profile.id, positionStore) {
        ClientBrowseResumeResolver(
            serverProfileId = profile.id,
            positionStore = positionStore,
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
            playbackStarter = playbackStarter,
            resumeResolver = resumeResolver,
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
    val shellDestinations = remember {
        TaruDestination.entries.map { destination ->
            TaruShellDestination(
                value = destination,
                label = destination.label,
                icon = destination.icon,
            )
        }
    }
    val artworkResolver = remember(profile, tokenVault) {
        TokenVaultArtworkRequestResolver(
            profile = profile,
            tokenVault = tokenVault,
        )
    }
    val settingsSession = remember(snapshot, tokenVault, onChangeServer, onSnapshotChanged) {
        SettingsSession(
            initialSnapshot = snapshot,
            runtime = object : SettingsRuntime {
                override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
                    onSnapshotChanged(snapshot)
                }

                override fun deleteToken(reference: String) {
                    tokenVault.deleteToken(reference)
                }

                override fun requestConnection() {
                    onChangeServer()
                }
            },
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
                    artworkResolver = artworkResolver,
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
                    state = shellState.detailState,
                    sourceProbeState = shellState.sourceProbeState,
                    playbackState = shellState.playbackState,
                    selectedSourceId = shellState.selectedSourceId,
                    resumePosition = shellState.resumePosition,
                    artworkResolver = artworkResolver,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onRetryPlayback = { dispatchBrowseAction(BrowseAction.RetryPlaybackDecision) },
                    onChangeServer = onChangeServer,
                    onOpenFacet = { dispatchBrowseAction(BrowseAction.OpenFacet(it)) },
                    onSelectSource = { sourceId ->
                        dispatchBrowseAction(BrowseAction.SelectSource(sourceId))
                    },
                    onRetrySourceProbe = { dispatchBrowseAction(BrowseAction.RetrySourceProbe) },
                    onRequestPlayback = {
                        dispatchBrowseAction(BrowseAction.RequestPlayback(it))
                    },
                    onStartPlayback = { dispatchBrowseAction(BrowseAction.StartPlayback(it)) },
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
                is TaruRoute.Player -> playerRouteRenderer.Render(
                    launch = currentRoute.launch,
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
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onChangeServer = onChangeServer,
                    onSwitchProfile = { profileId ->
                        settingsSession.dispatch(SettingsAction.SwitchProfile(profileId))
                    },
                    onSignOut = {
                        settingsSession.dispatch(SettingsAction.SignOutActiveProfile)
                    },
                )
            }
        }
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
    artworkResolver: ArtworkRequestResolver,
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
                artworkResolver = artworkResolver,
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
                artworkResolver = artworkResolver,
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
