package dev.taru.android.ui.browse

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
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.PlaybackPreferencesStore
import dev.taru.android.playback.PlaybackResult
import dev.taru.android.playback.SafePlaybackDiagnostics
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.userplayback.SafeUserPlaybackDiagnostics
import dev.taru.android.userplayback.TaruUserPlaybackClient
import dev.taru.android.userplayback.UserPlaybackResult
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.sync.withPermit

internal class ClientBrowseDataSource(
    private val profile: ServerProfile,
    private val tokenVault: TokenVault,
    private val browseClient: TaruBrowseClient,
    private val playbackClient: TaruPlaybackClient,
    private val playbackPreferencesStore: PlaybackPreferencesStore,
    private val userPlaybackClient: TaruUserPlaybackClient,
) : BrowseDataSource {
    override suspend fun loadHome(): BrowseUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return BrowseUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before browsing.",
                ),
            )
        }

        val libraries = browseClient.listLibraries(
            profile = profile,
            accessToken = accessToken,
            page = PageRequest(limit = 50, offset = 0),
        )
        val items = browseClient.listItems(
            profile = profile,
            accessToken = accessToken,
            page = PageRequest(limit = 24, offset = 0),
        )

        if (libraries is BrowseResult.Failure && items is BrowseResult.Failure) {
            return BrowseUiState.Failure(items.diagnostics)
        }

        val continueWatching = userPlaybackClient.continueWatching(
            profile = profile,
            accessToken = accessToken,
            page = PageRequest(limit = 12, offset = 0),
        )
        val itemPage = (items as? BrowseResult.Success)?.value
        val visibleArtwork = itemPage
            ?.let { page ->
                loadVisibleArtworkRefs(
                    accessToken = accessToken,
                    items = page.items,
                )
            }
            ?: HomeArtworkState()
        val continueArtwork = (continueWatching as? UserPlaybackResult.Success)
            ?.value
            ?.items
            ?.mapNotNull { row ->
                row.images
                    .takeIf { it.isNotEmpty() }
                    ?.let { row.item.id to it }
            }
            ?.toMap()
            .orEmpty()
        val artwork = visibleArtwork.copy(
            artworkByItemId = visibleArtwork.artworkByItemId + continueArtwork,
        )

        return BrowseUiState.Content(
            home = HomeReadModel(
                libraries = libraries.toHomeSectionState(),
                items = items.toHomeSectionState(),
                continueWatching = continueWatching.toBrowseHomeSectionState(),
                artwork = artwork,
            ),
        )
    }

    override suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return LibraryDetailUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before opening library details.",
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

    override suspend fun search(
        query: String,
        page: PageRequest,
    ): SearchUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return SearchUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before searching.",
                ),
            )
        }

        return when (
            val result = browseClient.searchItems(
                profile = profile,
                accessToken = accessToken,
                query = SearchRequest(
                    query = query,
                    page = page,
                ),
            )
        ) {
            is BrowseResult.Success -> SearchUiState.Content(result.value)
            is BrowseResult.Failure -> SearchUiState.Failure(result.diagnostics)
        }
    }

    override suspend fun loadPersonDetail(personId: String): PersonDetailUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return PersonDetailUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before opening person details.",
                ),
            )
        }

        val detail = browseClient.personDetail(
            profile = profile,
            accessToken = accessToken,
            personId = personId,
        )
        if (detail is BrowseResult.Failure) {
            return PersonDetailUiState.Failure(detail.diagnostics)
        }

        return when (
            val relatedItems = browseClient.listPersonItems(
                profile = profile,
                accessToken = accessToken,
                personId = personId,
                page = PageRequest(limit = 24, offset = 0),
            )
        ) {
            is BrowseResult.Success -> PersonDetailUiState.Content(
                response = (detail as BrowseResult.Success).value,
                relatedItems = relatedItems.value,
            )
            is BrowseResult.Failure -> PersonDetailUiState.Failure(relatedItems.diagnostics)
        }
    }

    override suspend fun loadRelationshipIndex(
        family: RelationshipIndexFamily,
        page: PageRequest,
    ): RelationshipIndexUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return RelationshipIndexUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before browsing related labels.",
                ),
            )
        }

        return when (family) {
            RelationshipIndexFamily.Genres -> when (
                val result = browseClient.listGenres(
                    profile = profile,
                    accessToken = accessToken,
                    page = page,
                )
            ) {
                is BrowseResult.Success -> result.value.toRelationshipIndexContent()
                is BrowseResult.Failure -> RelationshipIndexUiState.Failure(result.diagnostics)
            }
            RelationshipIndexFamily.Tags -> when (
                val result = browseClient.listTags(
                    profile = profile,
                    accessToken = accessToken,
                    page = page,
                )
            ) {
                is BrowseResult.Success -> result.value.toRelationshipIndexContent()
                is BrowseResult.Failure -> RelationshipIndexUiState.Failure(result.diagnostics)
            }
        }
    }

    override suspend fun loadFacet(
        target: BrowseFacetTarget,
        page: PageRequest,
    ): FacetUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return FacetUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before browsing this list.",
                ),
            )
        }

        val facetId = target.id.orEmpty()
        val result: BrowseResult<FacetItemsResponse> = when (target.family) {
            BrowseFacetUiFamily.Genre -> browseClient.listGenreItems(
                profile = profile,
                accessToken = accessToken,
                genreId = facetId,
                page = page,
            )
            BrowseFacetUiFamily.Tag -> browseClient.listTagItems(
                profile = profile,
                accessToken = accessToken,
                tagId = facetId,
                page = page,
            )
            BrowseFacetUiFamily.Person -> browseClient.listPersonItems(
                profile = profile,
                accessToken = accessToken,
                personId = facetId,
                page = page,
            )
            else -> return target.apiGapState()
        }

        return when (result) {
            is BrowseResult.Success -> FacetUiState.Content(result.value)
            is BrowseResult.Failure -> FacetUiState.Failure(result.diagnostics)
        }
    }

    override suspend fun loadItemDetail(itemId: String): ItemDetailUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return ItemDetailUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before opening details.",
                ),
            )
        }

        return when (
            val result = browseClient.itemDetail(
                profile = profile,
                accessToken = accessToken,
                itemId = itemId,
            )
        ) {
            is BrowseResult.Success -> {
                val userPlaybackState = when (
                    val stateResult = userPlaybackClient.getState(
                        profile = profile,
                        accessToken = accessToken,
                        itemId = itemId,
                    )
                ) {
                    is UserPlaybackResult.Success -> stateResult.value.state
                    is UserPlaybackResult.Failure -> null
                }
                ItemDetailUiState.Content(
                    response = result.value,
                    userPlaybackState = userPlaybackState,
                )
            }
            is BrowseResult.Failure -> ItemDetailUiState.Failure(result.diagnostics)
        }
    }

    override suspend fun loadSourceProbe(sourceId: String): SourceProbeUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return SourceProbeUiState.Failure(
                SafePlaybackDiagnostics(
                    category = PlaybackFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before loading version details.",
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

    override suspend fun loadPlaybackSelection(sourceId: String): PlaybackSelectionUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return PlaybackSelectionUiState.Failure(
                SafePlaybackDiagnostics(
                    category = PlaybackFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before requesting playback.",
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
                    decision = result.value,
                    capabilities = capabilities,
                ),
                capabilities = capabilities,
            )
            is PlaybackResult.Failure -> PlaybackSelectionUiState.Failure(result.diagnostics)
        }
    }

    private suspend fun loadVisibleArtworkRefs(
        accessToken: String,
        items: List<MediaItemDto>,
    ): HomeArtworkState = coroutineScope {
        val semaphore = Semaphore(permits = 4)
        val rows = items
            .map { item ->
                async {
                    semaphore.withPermit {
                        val result = browseClient.itemImages(
                            profile = profile,
                            accessToken = accessToken,
                            itemId = item.id,
                        )
                        when (result) {
                            is BrowseResult.Success -> HomeArtworkLoadResult(
                                itemId = item.id,
                                images = result.value.images,
                            )
                            is BrowseResult.Failure -> HomeArtworkLoadResult(
                                itemId = item.id,
                                failure = HomeArtworkFailure(
                                    itemId = item.id,
                                    diagnostics = result.diagnostics,
                                ),
                            )
                        }
                    }
                }
            }
            .awaitAll()
        HomeArtworkState(
            artworkByItemId = rows
                .mapNotNull { result ->
                    result.images
                        ?.takeIf { it.isNotEmpty() }
                        ?.let { result.itemId to it }
                }
                .toMap(),
            failures = rows.mapNotNull { it.failure },
        )
    }
}

private data class HomeArtworkLoadResult(
    val itemId: String,
    val images: List<PublicImageRefDto>? = null,
    val failure: HomeArtworkFailure? = null,
)

private fun <T> BrowseResult<T>.toHomeSectionState(): HomeSectionState<T> =
    when (this) {
        is BrowseResult.Success -> HomeSectionState.Available(value)
        is BrowseResult.Failure -> HomeSectionState.Unavailable(diagnostics)
    }

private fun SafeUserPlaybackDiagnostics.toBrowseDiagnostics(): SafeBrowseDiagnostics =
    SafeBrowseDiagnostics(
        category = runCatching { BrowseFailureCategory.valueOf(category.name) }
            .getOrDefault(BrowseFailureCategory.PublicApiError),
        userMessage = userMessage,
        statusCode = statusCode,
        expectedApiVersion = expectedApiVersion,
        observedApiVersion = observedApiVersion,
        publicError = publicError,
        request = request,
    )

private fun <T> UserPlaybackResult<T>.toBrowseHomeSectionState(): HomeSectionState<T> =
    when (this) {
        is UserPlaybackResult.Success -> HomeSectionState.Available(value)
        is UserPlaybackResult.Failure -> HomeSectionState.Unavailable(diagnostics.toBrowseDiagnostics())
    }
