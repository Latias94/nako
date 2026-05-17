package dev.taru.android.ui.browse

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.PrimaryTabRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Tab
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.BrowseResult
import dev.taru.android.browse.CanonicalMetadataDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.ItemsResponse
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TokenVault
import dev.taru.android.ui.theme.TaruAndroidTheme
import dev.taru.android.ui.theme.TaruAspectRatio
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
fun TaruBrowseShell(
    profile: ServerProfile,
    tokenVault: TokenVault,
    browseClient: TaruBrowseClient,
    onChangeServer: () -> Unit,
    modifier: Modifier = Modifier,
) {
    var selectedDestination by remember { mutableIntStateOf(0) }
    var refreshKey by remember { mutableIntStateOf(0) }
    var selectedItemId by remember(profile.id) { mutableStateOf<String?>(null) }
    var state by remember(profile.id, refreshKey) {
        mutableStateOf<BrowseUiState>(BrowseUiState.Loading)
    }
    var detailRefreshKey by remember { mutableIntStateOf(0) }
    var detailState by remember(profile.id, selectedItemId, detailRefreshKey) {
        mutableStateOf<ItemDetailUiState>(ItemDetailUiState.Idle)
    }

    LaunchedEffect(profile.id, refreshKey) {
        selectedItemId = null
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            state = BrowseUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Re-authenticate this server before browsing.",
                ),
            )
            return@LaunchedEffect
        }

        val libraries = browseClient.listLibraries(
            profile = profile,
            accessToken = accessToken,
            page = PageRequest(limit = 50, offset = 0),
        )
        if (libraries is BrowseResult.Failure) {
            state = BrowseUiState.Failure(libraries.diagnostics)
            return@LaunchedEffect
        }

        val items = browseClient.listItems(
            profile = profile,
            accessToken = accessToken,
            page = PageRequest(limit = 24, offset = 0),
        )
        if (items is BrowseResult.Failure) {
            state = BrowseUiState.Failure(items.diagnostics)
            return@LaunchedEffect
        }

        state = BrowseUiState.Content(
            libraries = (libraries as BrowseResult.Success).value,
            items = (items as BrowseResult.Success).value,
        )
    }

    LaunchedEffect(profile.id, selectedItemId, detailRefreshKey) {
        val itemId = selectedItemId
        if (itemId == null) {
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
                itemId = itemId,
            )
        ) {
            is BrowseResult.Success -> ItemDetailUiState.Content(result.value)
            is BrowseResult.Failure -> ItemDetailUiState.Failure(result.diagnostics)
        }
    }

    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
        contentColor = MaterialTheme.colorScheme.onBackground,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(TaruSpacing.xlarge),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.large),
        ) {
            AppHeader(
                profile = profile,
                onChangeServer = onChangeServer,
            )

            PrimaryTabRow(selectedTabIndex = selectedDestination) {
                BrowseDestination.entries.forEachIndexed { index, destination ->
                    Tab(
                        selected = selectedDestination == index,
                        onClick = { selectedDestination = index },
                        text = { Text(destination.label) },
                    )
                }
            }

            when (val current = state) {
                BrowseUiState.Loading -> BrowseLoadingState()
                is BrowseUiState.Failure -> BrowseFailureState(
                    diagnostics = current.diagnostics,
                    onRetry = { refreshKey += 1 },
                    onChangeServer = onChangeServer,
                )
                is BrowseUiState.Content -> if (selectedItemId != null) {
                    ItemDetailContent(
                        state = detailState,
                        onBack = { selectedItemId = null },
                        onRetry = { detailRefreshKey += 1 },
                        onChangeServer = onChangeServer,
                    )
                } else {
                    when (BrowseDestination.entries[selectedDestination]) {
                        BrowseDestination.Home -> HomeContent(
                            state = current,
                            onOpenLibraries = { selectedDestination = 1 },
                            onOpenItem = { selectedItemId = it.id },
                        )
                        BrowseDestination.Libraries -> LibrariesContent(
                            state = current,
                            onOpenItem = { selectedItemId = it.id },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun AppHeader(
    profile: ServerProfile,
    onChangeServer: () -> Unit,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = profile.displayName,
                style = MaterialTheme.typography.headlineSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                text = profile.baseUrl,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        OutlinedButton(onClick = onChangeServer) {
            Text("Server")
        }
    }
}

@Composable
private fun BrowseLoadingState() {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = "Loading library",
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = "Fetching visible Media Libraries and Media Items.",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
private fun BrowseFailureState(
    diagnostics: SafeBrowseDiagnostics,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .border(
                width = TaruSpacing.xsmall / 4,
                color = MaterialTheme.colorScheme.error,
                shape = TaruShape.medium,
            ),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            Text(
                text = browseFailureTitle(diagnostics.category),
                style = MaterialTheme.typography.titleLarge,
            )
            Text(
                text = diagnostics.userMessage,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
            diagnostics.publicError?.let { publicError ->
                Text(
                    text = "${publicError.code}: ${publicError.message}",
                    color = TaruTextMuted,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                Button(onClick = onRetry) {
                    Text("Retry")
                }
                OutlinedButton(onClick = onChangeServer) {
                    Text("Change server")
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun HomeContent(
    state: BrowseUiState.Content,
    onOpenLibraries: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.large)) {
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            MetricTile(label = "Libraries", value = state.libraries.libraries.size.toString())
            MetricTile(label = "Visible items", value = state.items.items.size.toString())
        }

        if (state.libraries.libraries.isEmpty()) {
            EmptyState(
                title = "No Media Libraries",
                body = "This server has no visible Media Libraries for the current access token.",
            )
        } else {
            SectionHeader(
                title = "Media Libraries",
                count = state.libraries.page.returned,
            )
            state.libraries.libraries.take(3).forEach { library ->
                LibraryRow(library = library)
            }
            OutlinedButton(onClick = onOpenLibraries) {
                Text("All libraries")
            }
        }

        if (state.items.items.isNotEmpty()) {
            SectionHeader(
                title = "Latest visible items",
                count = state.items.page.returned,
            )
            MediaItemStrip(
                items = state.items.items.take(6),
                onOpenItem = onOpenItem,
            )
        } else if (state.libraries.libraries.isNotEmpty()) {
            EmptyState(
                title = "No visible items",
                body = "The current access token can see libraries, but no Media Items were returned.",
            )
        }
    }
}

@Composable
private fun LibrariesContent(
    state: BrowseUiState.Content,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.large)) {
        if (state.libraries.libraries.isEmpty()) {
            EmptyState(
                title = "No Media Libraries",
                body = "This server has no visible Media Libraries for the current access token.",
            )
        } else {
            SectionHeader(
                title = "Media Libraries",
                count = state.libraries.page.returned,
            )
            state.libraries.libraries.forEach { library ->
                LibraryRow(library = library)
            }
        }

        SectionHeader(
            title = "Media Items",
            count = state.items.page.returned,
        )
        if (state.items.items.isEmpty()) {
            EmptyState(
                title = "No visible items",
                body = "The selected server returned an empty Media Item page.",
            )
        } else {
            state.items.items.forEach { item ->
                MediaItemRow(
                    item = item,
                    onOpenItem = onOpenItem,
                )
            }
        }
    }
}

@Composable
private fun MetricTile(
    label: String,
    value: String,
) {
    Surface(
        modifier = Modifier.width(TaruSpacing.xxlarge * 4),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = value,
                style = MaterialTheme.typography.headlineSmall,
            )
            Text(
                text = label,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.labelMedium,
            )
        }
    }
}

@Composable
private fun SectionHeader(
    title: String,
    count: Int,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        Text(
            text = title,
            modifier = Modifier.weight(1f),
            style = MaterialTheme.typography.titleLarge,
        )
        Text(
            text = count.toString(),
            color = TaruTextMuted,
            style = MaterialTheme.typography.labelMedium,
        )
    }
}

@Composable
private fun LibraryRow(library: LibraryDto) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Row(
            modifier = Modifier.padding(TaruSpacing.large),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            LibrarySwatch(library.name)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = library.name,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = listOfNotNull(
                        library.options?.preset,
                        library.options?.domain,
                    ).joinToString(" / ").ifBlank { "Media Library" },
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun LibrarySwatch(seed: String) {
    val color = if (seed.hashCode() % 2 == 0) {
        MaterialTheme.colorScheme.primary.copy(alpha = 0.82f)
    } else {
        Color(0xFF66D2C8)
    }
    Surface(
        modifier = Modifier
            .width(TaruSpacing.xxlarge)
            .height(TaruSpacing.xxlarge),
        shape = TaruShape.small,
        color = color,
    ) {}
}

@Composable
private fun MediaItemStrip(
    items: List<MediaItemDto>,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        items.forEach { item ->
            MediaItemPoster(
                item = item,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun MediaItemPoster(
    item: MediaItemDto,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    Column(
        modifier = Modifier
            .width(TaruSpacing.xxlarge * 3)
            .clickable { onOpenItem(item) },
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
    ) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .aspectRatio(TaruAspectRatio.poster),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surfaceVariant,
        ) {
            Column(
                modifier = Modifier.padding(TaruSpacing.small),
                verticalArrangement = Arrangement.Bottom,
            ) {
                Text(
                    text = item.kind,
                    color = TaruTextMuted,
                    style = MaterialTheme.typography.labelMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        Text(
            text = item.metadata.title,
            style = MaterialTheme.typography.bodyMedium,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun MediaItemRow(
    item: MediaItemDto,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onOpenItem(item) },
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Row(
            modifier = Modifier.padding(TaruSpacing.large),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            Surface(
                modifier = Modifier
                    .width(TaruSpacing.xxlarge)
                    .aspectRatio(TaruAspectRatio.poster),
                shape = TaruShape.small,
                color = MaterialTheme.colorScheme.surfaceVariant,
            ) {}
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = item.metadata.title,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                Text(
                    text = itemSecondaryText(item),
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun ItemDetailContent(
    state: ItemDetailUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.large)) {
        OutlinedButton(onClick = onBack) {
            Text("Back")
        }

        when (state) {
            ItemDetailUiState.Idle,
            ItemDetailUiState.Loading,
            -> BrowseLoadingState()
            is ItemDetailUiState.Failure -> BrowseFailureState(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is ItemDetailUiState.Content -> ReadOnlyItemDetail(response = state.response)
        }
    }
}

@Composable
private fun ReadOnlyItemDetail(response: ItemDetailResponse) {
    val item = response.item
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.large)) {
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surface,
        ) {
            Column(
                modifier = Modifier.padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            ) {
                Text(
                    text = item.metadata.title,
                    style = MaterialTheme.typography.headlineSmall,
                )
                Text(
                    text = itemSecondaryText(item),
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                item.metadata.overview?.takeIf { it.isNotBlank() }?.let { overview ->
                    Text(
                        text = overview,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                }
            }
        }

        MetadataChips(metadata = item.metadata)

        SectionHeader(title = "Detail facts", count = response.sources.size)
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surface,
        ) {
            Column(
                modifier = Modifier.padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
            ) {
                DetailFact("Media Sources", response.sources.size.toString())
                DetailFact("Credits", response.credits.size.toString())
                DetailFact("Genres", response.genres.size.toString())
                DetailFact("Tags", response.tags.size.toString())
                DetailFact("Collections", response.collections.size.toString())
                DetailFact("Studios", response.studios.size.toString())
                DetailFact("Images", response.images.size.toString())
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun MetadataChips(metadata: CanonicalMetadataDto) {
    val chips = buildList {
        metadata.releaseDate?.take(4)?.let { add(it) }
        metadata.runtimeMinutes?.let { add("$it min") }
        metadata.ratings.firstOrNull()?.let { add("${it.source}: ${it.value}") }
        addAll(metadata.genres.take(4))
        addAll(metadata.tags.take(4))
    }

    if (chips.isEmpty()) {
        return
    }

    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
    ) {
        chips.forEach { chip ->
            Surface(
                shape = TaruShape.small,
                color = MaterialTheme.colorScheme.surfaceVariant,
            ) {
                Text(
                    modifier = Modifier.padding(
                        horizontal = TaruSpacing.medium,
                        vertical = TaruSpacing.small,
                    ),
                    text = chip,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
        }
    }
}

@Composable
private fun DetailFact(
    label: String,
    value: String,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
    ) {
        Text(
            text = label,
            modifier = Modifier.weight(1f),
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodyMedium,
        )
        Text(
            text = value,
            style = MaterialTheme.typography.bodyMedium,
        )
    }
}

@Composable
private fun EmptyState(
    title: String,
    body: String,
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = body,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

private fun itemSecondaryText(item: MediaItemDto): String =
    listOfNotNull(
        item.kind,
        item.metadata.releaseDate?.take(4),
        item.metadata.runtimeMinutes?.let { "$it min" },
    ).joinToString(" / ")

private fun browseFailureTitle(category: BrowseFailureCategory): String =
    when (category) {
        BrowseFailureCategory.MissingItem -> "Media Item unavailable"
        BrowseFailureCategory.MissingAccessToken -> "Authentication required"
        BrowseFailureCategory.UnreachableServer -> "Server unreachable"
        BrowseFailureCategory.Unauthorized -> "Authentication failed"
        BrowseFailureCategory.Forbidden -> "Permission denied"
        BrowseFailureCategory.UnsupportedApiVersion -> "Unsupported server"
        BrowseFailureCategory.TlsOrCertificate -> "Certificate problem"
        BrowseFailureCategory.PublicApiError -> "Browse failed"
        BrowseFailureCategory.InvalidResponse -> "Invalid response"
    }

private enum class BrowseDestination(val label: String) {
    Home("Home"),
    Libraries("Libraries"),
}

private sealed interface BrowseUiState {
    data object Loading : BrowseUiState

    data class Content(
        val libraries: LibraryListResponse,
        val items: ItemsResponse,
    ) : BrowseUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : BrowseUiState
}

private sealed interface ItemDetailUiState {
    data object Idle : ItemDetailUiState

    data object Loading : ItemDetailUiState

    data class Content(
        val response: ItemDetailResponse,
    ) : ItemDetailUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : ItemDetailUiState
}

@Preview
@Composable
private fun TaruBrowseShellPreview() {
    val tokenVault = InMemoryTokenVault().apply {
        saveToken("server-token:server-1", "preview-token")
    }
    TaruAndroidTheme(darkTheme = true) {
        TaruBrowseShell(
            profile = ServerProfile(
                id = "server-1",
                displayName = "Home",
                baseUrl = "http://localhost:3000",
                tokenReference = "server-token:server-1",
                lastObservedApiVersion = "v1",
            ),
            tokenVault = tokenVault,
            browseClient = TaruBrowseClient(
                transport = object : TaruHttpTransport {
                    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse =
                        if (request.url.contains("/libraries")) {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"libraries":[{"id":"library-1","name":"Movies","options":{"domain":"video","preset":"movies"}}],"page":{"limit":50,"offset":0,"returned":1}}""",
                            )
                        } else {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"items":[{"id":"item-1","kind":"movie","metadata":{"title":"Arrival","release_date":"2016-11-11","runtime_minutes":116,"genres":[],"tags":[],"ratings":[],"images":[]}}],"page":{"limit":24,"offset":0,"returned":1}}""",
                            )
                        }
                },
            ),
            onChangeServer = {},
        )
    }
}
