package dev.nako.android.ui.browse

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.browse.NakoBrowseClient
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.PlaybackPreferencesStore
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.player.DevicePlaybackPositionStore
import dev.nako.android.ui.artwork.ArtworkRequestResolver
import dev.nako.android.ui.artwork.TokenVaultArtworkRequestResolver
import dev.nako.android.ui.screens.detail.DetailRouteContent
import dev.nako.android.ui.screens.person.PersonDetailRouteContent
import dev.nako.android.ui.screens.player.PlayerRouteRenderer
import dev.nako.android.ui.screens.relationship.RelationshipIndexRouteContent
import dev.nako.android.ui.screens.settings.ServerProfileScreen
import dev.nako.android.ui.screens.settings.SettingsAction
import dev.nako.android.ui.screens.settings.SettingsHomeScreen
import dev.nako.android.ui.shell.NakoAdaptiveAppShell
import dev.nako.android.ui.shell.NakoRouteTransition
import dev.nako.android.ui.shell.NakoShellDestination
import dev.nako.android.userplayback.NakoUserPlaybackClient

@Composable
internal fun NakoBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: NakoBrowseClient,
    playbackClient: NakoPlaybackClient,
    playbackPreferencesStore: PlaybackPreferencesStore,
    userPlaybackClient: NakoUserPlaybackClient,
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
    val currentOnChangeServer by rememberUpdatedState(onChangeServer)
    val currentOnSnapshotChanged by rememberUpdatedState(onSnapshotChanged)
    var savedShellState by rememberSaveable(
        profile.id,
        stateSaver = BrowseShellStateSaver,
    ) {
        mutableStateOf(BrowseShellState())
    }
    val runtime = remember(
        tokenVault,
        browseClient,
        playbackClient,
        playbackPreferencesStore,
        userPlaybackClient,
        positionStore,
    ) {
        ClientBrowseShellRuntime(
            tokenVault = tokenVault,
            browseClient = browseClient,
            playbackClient = playbackClient,
            playbackPreferencesStore = playbackPreferencesStore,
            userPlaybackClient = userPlaybackClient,
            positionStore = positionStore,
            onChangeServer = { currentOnChangeServer() },
            onSnapshotChanged = { currentOnSnapshotChanged(it) },
        )
    }
    val browseHost = remember(profile.id, snapshot, runtime, routeScope) {
        BrowseShellHost(
            profile = profile,
            snapshot = snapshot,
            initialState = savedShellState,
            runtime = runtime,
            parentScope = routeScope,
            saveState = { savedShellState = it },
        )
    }
    DisposableEffect(browseHost) {
        onDispose {
            browseHost.close()
        }
    }
    val shellState by browseHost.state.collectAsStateWithLifecycle()
    val selectedDestination = shellState.selectedDestination
    val route = shellState.currentRoute
    fun dispatchBrowseAction(action: BrowseAction) {
        browseHost.dispatch(action)
    }
    val shellDestinations = remember {
        NakoDestination.entries.map { destination ->
            NakoShellDestination(
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

    BackHandler(enabled = shellState.canNavigateBack) {
        dispatchBrowseAction(BrowseAction.Back)
    }

    NakoAdaptiveAppShell(
        modifier = modifier,
        destinations = shellDestinations,
        selectedDestination = selectedDestination,
        navigationVisible = shellState.navigationVisible,
        onDestinationSelected = {
            dispatchBrowseAction(BrowseAction.SelectDestination(it))
        },
    ) { innerPadding ->
        NakoRouteTransition(
            targetState = route,
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
        ) { currentRoute ->
            when (currentRoute) {
                NakoRoute.TopLevel -> TopLevelContent(
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
                    onLoadMoreSearch = { dispatchBrowseAction(BrowseAction.LoadMoreSearch) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { dispatchBrowseAction(BrowseAction.OpenItem(it.id)) },
                    onOpenLibrary = {
                        dispatchBrowseAction(BrowseAction.SelectDestination(NakoDestination.Libraries))
                    },
                    onOpenSearch = {
                        dispatchBrowseAction(BrowseAction.SelectDestination(NakoDestination.Search))
                    },
                    onOpenGenres = {
                        dispatchBrowseAction(BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Genres))
                    },
                    onOpenTags = {
                        dispatchBrowseAction(BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Tags))
                    },
                    onOpenServerProfile = { dispatchBrowseAction(BrowseAction.OpenServerProfile) },
                    onOpenFacet = { dispatchBrowseAction(BrowseAction.OpenFacet(it)) },
                    onOpenLibraryDetail = { libraryId ->
                        dispatchBrowseAction(BrowseAction.OpenLibraryDetail(libraryId))
                    },
                )
                is NakoRoute.ItemDetail -> DetailRouteContent(
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
                    onOpenPersonDetail = { personId ->
                        dispatchBrowseAction(BrowseAction.OpenPersonDetail(personId))
                    },
                    onSelectSource = { sourceId ->
                        dispatchBrowseAction(BrowseAction.SelectSource(sourceId))
                    },
                    onRetrySourceProbe = { dispatchBrowseAction(BrowseAction.RetrySourceProbe) },
                    onRequestPlayback = {
                        dispatchBrowseAction(BrowseAction.RequestPlayback(it))
                    },
                    onStartPlayback = { dispatchBrowseAction(BrowseAction.StartPlayback(it)) },
                )
                is NakoRoute.LibraryDetail -> LibraryDetailRouteContent(
                    state = shellState.libraryDetailState,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { itemId ->
                        dispatchBrowseAction(BrowseAction.OpenItem(itemId))
                    },
                )
                is NakoRoute.PersonDetail -> PersonDetailRouteContent(
                    state = shellState.personDetailState,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { dispatchBrowseAction(BrowseAction.OpenItem(it.id)) },
                )
                is NakoRoute.RelationshipIndex -> RelationshipIndexRouteContent(
                    family = currentRoute.family,
                    state = shellState.relationshipIndexState,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onLoadMore = { dispatchBrowseAction(BrowseAction.LoadMoreRelationshipIndex) },
                    onChangeServer = onChangeServer,
                    onOpenFacet = { dispatchBrowseAction(BrowseAction.OpenFacet(it)) },
                )
                is NakoRoute.Player -> playerRouteRenderer.Render(
                    launch = currentRoute.launch,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                )
                is NakoRoute.BrowseFacet -> BrowseFacetRouteContent(
                    target = currentRoute.target,
                    state = shellState.facetState,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onRetry = { dispatchBrowseAction(BrowseAction.RetryCurrentRoute) },
                    onLoadMore = { dispatchBrowseAction(BrowseAction.LoadMoreFacet) },
                    onChangeServer = onChangeServer,
                    onOpenItem = { dispatchBrowseAction(BrowseAction.OpenItem(it.id)) },
                )
                NakoRoute.ServerProfile -> ServerProfileScreen(
                    activeProfile = profile,
                    snapshot = snapshot,
                    onBack = { dispatchBrowseAction(BrowseAction.Back) },
                    onChangeServer = onChangeServer,
                    onSwitchProfile = { profileId ->
                        browseHost.dispatchSettings(SettingsAction.SwitchProfile(profileId))
                    },
                    onSignOut = {
                        browseHost.dispatchSettings(SettingsAction.SignOutActiveProfile)
                    },
                )
            }
        }
    }
}

@Composable
private fun TopLevelContent(
    profile: ServerProfile,
    selectedDestination: NakoDestination,
    browseState: BrowseUiState,
    searchQuery: String,
    searchState: SearchUiState,
    snapshot: ServerProfileSnapshot,
    artworkResolver: ArtworkRequestResolver,
    onRetry: () -> Unit,
    onSearchQueryChange: (String) -> Unit,
    onSubmitSearch: () -> Unit,
    onRetrySearch: () -> Unit,
    onLoadMoreSearch: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenLibrary: () -> Unit,
    onOpenSearch: () -> Unit,
    onOpenGenres: () -> Unit,
    onOpenTags: () -> Unit,
    onOpenServerProfile: () -> Unit,
    onOpenFacet: (BrowseFacetTarget) -> Unit,
    onOpenLibraryDetail: (String) -> Unit,
) {
    when (selectedDestination) {
        NakoDestination.Home -> BrowseScaffoldContent {
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
                onOpenGenres = onOpenGenres,
                onOpenTags = onOpenTags,
                onOpenFacet = onOpenFacet,
            )
        }
        NakoDestination.Libraries -> BrowseScaffoldContent {
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
        NakoDestination.Search -> BrowseScaffoldContent {
            SearchScreen(
                query = searchQuery,
                state = searchState,
                onQueryChange = onSearchQueryChange,
                onSubmit = onSubmitSearch,
                onRetry = onRetrySearch,
                onLoadMore = onLoadMoreSearch,
                onChangeServer = onChangeServer,
                onOpenItem = onOpenItem,
            )
        }
        NakoDestination.Settings -> BrowseScaffoldContent {
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
