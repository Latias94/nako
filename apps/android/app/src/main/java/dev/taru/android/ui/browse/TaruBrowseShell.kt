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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
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
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.SafeBrowseDiagnostics
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
                    snapshot = snapshot,
                    onRetry = { refreshKey += 1 },
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
                is TaruRoute.BrowseFacet -> PlaceholderRoute(
                    title = currentRoute.title,
                    subtitle = "Browse Facet Result",
                    body = "This route is reserved for public API backed genre, tag, person, studio, collection, year, and item-kind facets.",
                    onBack = { route = TaruRoute.TopLevel },
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
    snapshot: ServerProfileSnapshot,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
    onOpenLibrary: () -> Unit,
    onOpenSearch: () -> Unit,
    onOpenServerProfile: () -> Unit,
    onOpenFacet: (String) -> Unit,
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
        TaruDestination.Search -> PlaceholderTopLevel(
            title = "Search",
            subtitle = "Find a known title",
            body = "Search shell is ready for the public search API. Results will navigate into Media Item Detail.",
            icon = Icons.Rounded.Search,
        )
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
