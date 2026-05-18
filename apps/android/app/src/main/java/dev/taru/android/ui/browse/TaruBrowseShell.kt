package dev.taru.android.ui.browse

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.dp
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
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
fun TaruBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    onChangeServer: () -> Unit,
    modifier: Modifier = Modifier,
    snapshot: ServerProfileSnapshot = ServerProfileSnapshot(
        profiles = listOf(profile),
        activeProfileId = profile.id,
    ),
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    var selectedDestination by remember { mutableStateOf(TaruDestination.Home) }
    var route by remember(profile.id) { mutableStateOf<TaruRoute>(TaruRoute.TopLevel) }
    var refreshKey by remember { mutableIntStateOf(0) }
    var browseState by remember(profile.id, refreshKey) {
        mutableStateOf<BrowseUiState>(BrowseUiState.Loading)
    }
    var detailRefreshKey by remember { mutableIntStateOf(0) }
    var detailState by remember(profile.id, route, detailRefreshKey) {
        mutableStateOf<ItemDetailUiState>(ItemDetailUiState.Idle)
    }
    var searchQuery by remember(profile.id) { mutableStateOf("") }
    var submittedSearchQuery by remember(profile.id) { mutableStateOf("") }
    var searchRefreshKey by remember { mutableIntStateOf(0) }
    var searchState by remember(profile.id) { mutableStateOf<SearchUiState>(SearchUiState.Idle) }
    var facetRefreshKey by remember { mutableIntStateOf(0) }
    var facetState by remember(profile.id, route, facetRefreshKey) {
        mutableStateOf<FacetUiState>(FacetUiState.Idle)
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

    Scaffold(
        modifier = modifier.fillMaxSize(),
        containerColor = MaterialTheme.colorScheme.background,
        bottomBar = {
            if (route is TaruRoute.TopLevel) {
                TaruBottomNavigation(
                    selectedDestination = selectedDestination,
                    onSelected = { selectedDestination = it },
                )
            }
        },
    ) { innerPadding ->
        AnimatedContent(
            targetState = route,
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
            transitionSpec = { fadeIn(tween(160)) togetherWith fadeOut(tween(120)) },
            label = "taru-route",
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
                    onOpenItem = { route = TaruRoute.ItemDetail(it.id) },
                    onOpenLibrary = { selectedDestination = TaruDestination.Libraries },
                    onOpenSearch = { selectedDestination = TaruDestination.Search },
                    onOpenServerProfile = { route = TaruRoute.ServerProfile },
                    onOpenFacet = { route = TaruRoute.BrowseFacet(it) },
                )
                is TaruRoute.ItemDetail -> DetailRouteContent(
                    state = detailState,
                    onBack = { route = TaruRoute.TopLevel },
                    onRetry = { detailRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenFacet = { route = TaruRoute.BrowseFacet(it) },
                )
                is TaruRoute.BrowseFacet -> BrowseFacetRouteContent(
                    target = currentRoute.target,
                    state = facetState,
                    onBack = { route = TaruRoute.TopLevel },
                    onRetry = { facetRefreshKey += 1 },
                    onChangeServer = onChangeServer,
                    onOpenItem = { route = TaruRoute.ItemDetail(it.id) },
                )
                TaruRoute.ServerProfile -> ServerProfileScreen(
                    activeProfile = profile,
                    snapshot = snapshot,
                    tokenVault = tokenVault,
                    onBack = { route = TaruRoute.TopLevel },
                    onChangeServer = onChangeServer,
                    onSnapshotChanged = onSnapshotChanged,
                )
            }
        }
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
        modifier = Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    colors = listOf(
                        MaterialTheme.colorScheme.background,
                        MaterialTheme.colorScheme.background,
                        MaterialTheme.colorScheme.surface.copy(alpha = 0.52f),
                    ),
                ),
            ),
    ) {
        content()
    }
}

@Composable
private fun TaruBottomNavigation(
    selectedDestination: TaruDestination,
    onSelected: (TaruDestination) -> Unit,
) {
    NavigationBar(
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = 0.dp,
    ) {
        TaruDestination.entries.forEach { destination ->
            val selected = destination == selectedDestination
            NavigationBarItem(
                selected = selected,
                onClick = { onSelected(destination) },
                icon = {
                    Icon(
                        imageVector = destination.icon,
                        contentDescription = destination.label,
                    )
                },
                label = { Text(destination.label) },
                colors = NavigationBarItemDefaults.colors(
                    selectedIconColor = MaterialTheme.colorScheme.onPrimary,
                    selectedTextColor = MaterialTheme.colorScheme.primary,
                    indicatorColor = MaterialTheme.colorScheme.primary,
                    unselectedIconColor = TaruTextSecondary,
                    unselectedTextColor = TaruTextSecondary,
                ),
            )
        }
    }
}
