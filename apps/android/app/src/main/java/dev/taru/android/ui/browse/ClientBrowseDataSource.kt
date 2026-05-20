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
            accessToken = accessToken,
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

    override suspend fun loadLibraryDetail(libraryId: String): LibraryDetailUiState {
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

    override suspend fun search(query: String): SearchUiState {
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

    override suspend fun loadPersonDetail(personId: String): PersonDetailUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return PersonDetailUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Re-authenticate this server before opening person detail.",
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

    override suspend fun loadFacet(target: BrowseFacetTarget): FacetUiState {
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

    override suspend fun loadItemDetail(itemId: String): ItemDetailUiState {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return ItemDetailUiState.Failure(
                SafeBrowseDiagnostics(
                    category = BrowseFailureCategory.MissingAccessToken,
                    userMessage = "Re-authenticate this server before opening detail.",
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

    override suspend fun loadPlaybackSelection(sourceId: String): PlaybackSelectionUiState {
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

    private suspend fun loadVisibleArtworkRefs(
        accessToken: String,
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
}
