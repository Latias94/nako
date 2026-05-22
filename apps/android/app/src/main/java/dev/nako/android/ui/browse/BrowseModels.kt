package dev.nako.android.ui.browse

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.ui.graphics.vector.ImageVector
import dev.nako.android.browse.FacetItemsResponse
import dev.nako.android.browse.GenreListResponse
import dev.nako.android.browse.ItemDetailResponse
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.browse.ItemsResponse
import dev.nako.android.browse.LibrarySourcesResponse
import dev.nako.android.browse.LibraryListResponse
import dev.nako.android.browse.PageInfo
import dev.nako.android.browse.PersonResponse
import dev.nako.android.browse.PublicImageRefDto
import dev.nako.android.browse.SafeBrowseDiagnostics
import dev.nako.android.browse.SearchResponse
import dev.nako.android.browse.TagListResponse
import dev.nako.android.media.SourceProbeResponse
import dev.nako.android.playback.PlaybackCapabilities
import dev.nako.android.playback.PlaybackDecisionResponse
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.playback.SafePlaybackDiagnostics
import dev.nako.android.player.PlaybackLaunchRequest
import dev.nako.android.userplayback.ContinueWatchingResponse
import dev.nako.android.userplayback.UserPlaybackStateDto

internal enum class NakoDestination(
    val label: String,
    val icon: ImageVector,
) {
    Home("Home", Icons.Rounded.Home),
    Libraries("Libraries", Icons.AutoMirrored.Rounded.LibraryBooks),
    Search("Search", Icons.Rounded.Search),
    Settings("Settings", Icons.Rounded.Settings),
}

internal sealed interface NakoRoute {
    data object TopLevel : NakoRoute
    data class ItemDetail(val itemId: String) : NakoRoute
    data class LibraryDetail(val libraryId: String) : NakoRoute
    data class PersonDetail(val personId: String) : NakoRoute
    data class RelationshipIndex(val family: RelationshipIndexFamily) : NakoRoute
    data class Player(val launch: PlaybackLaunchRequest) : NakoRoute
    data class BrowseFacet(val target: BrowseFacetTarget) : NakoRoute
    data object ServerProfile : NakoRoute
}

internal class NakoRouteStack private constructor(
    private val entries: List<NakoRoute>,
) {
    val current: NakoRoute = entries.last()
    val isAtRoot: Boolean = entries.size == 1
    val canPop: Boolean = !isAtRoot
    val routes: List<NakoRoute> = entries

    init {
        require(entries.isNotEmpty()) { "Nako route stack cannot be empty." }
        require(entries.first() == NakoRoute.TopLevel) {
            "Nako route stack must be rooted at top level."
        }
        require(entries.drop(1).none { it == NakoRoute.TopLevel }) {
            "Top level route cannot appear inside nested routes."
        }
    }

    fun push(route: NakoRoute): NakoRouteStack {
        require(route != NakoRoute.TopLevel) {
            "Use clearToRoot() instead of pushing top level."
        }
        return NakoRouteStack(entries + route)
    }

    fun pop(): NakoRouteStack =
        if (canPop) {
            NakoRouteStack(entries.dropLast(1))
        } else {
            this
        }

    fun clearToRoot(): NakoRouteStack = root()

    override fun equals(other: Any?): Boolean =
        other is NakoRouteStack && entries == other.entries

    override fun hashCode(): Int = entries.hashCode()

    override fun toString(): String = "NakoRouteStack(entries=$entries)"

    companion object {
        fun root(): NakoRouteStack = NakoRouteStack(listOf(NakoRoute.TopLevel))
    }
}

internal data class NakoBrowseNavigationState(
    val selectedDestination: NakoDestination = NakoDestination.Home,
    val routeStack: NakoRouteStack = NakoRouteStack.root(),
) {
    val currentRoute: NakoRoute = routeStack.current
    val navigationVisible: Boolean = routeStack.isAtRoot
    val canNavigateBack: Boolean = routeStack.canPop

    fun selectDestination(destination: NakoDestination): NakoBrowseNavigationState =
        copy(
            selectedDestination = destination,
            routeStack = routeStack.clearToRoot(),
        )

    fun open(route: NakoRoute): NakoBrowseNavigationState =
        copy(routeStack = routeStack.push(route))

    fun navigateBack(): NakoBrowseNavigationState =
        copy(routeStack = routeStack.pop())

    companion object {
        fun root(): NakoBrowseNavigationState = NakoBrowseNavigationState()
    }
}

internal sealed interface BrowseUiState {
    data object Loading : BrowseUiState

    data class Content(
        val home: HomeReadModel,
    ) : BrowseUiState {
        constructor(
            libraries: LibraryListResponse,
            items: ItemsResponse,
            artworkByItemId: Map<String, List<PublicImageRefDto>> = emptyMap(),
            continueWatching: ContinueWatchingResponse? = null,
        ) : this(
            HomeReadModel(
                libraries = HomeSectionState.Available(libraries),
                items = HomeSectionState.Available(items),
                continueWatching = continueWatching
                    ?.let { HomeSectionState.Available(it) }
                    ?: HomeSectionState.NotRequested,
                artwork = HomeArtworkState(artworkByItemId = artworkByItemId),
            ),
        )

        val libraries: LibraryListResponse
            get() = home.libraries.valueOrNull() ?: emptyHomeLibraries
        val items: ItemsResponse
            get() = home.items.valueOrNull() ?: emptyHomeItems
        val artworkByItemId: Map<String, List<PublicImageRefDto>>
            get() = home.artwork.artworkByItemId
        val continueWatching: ContinueWatchingResponse?
            get() = home.continueWatching.valueOrNull()
    }

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : BrowseUiState
}

internal data class HomeReadModel(
    val libraries: HomeSectionState<LibraryListResponse>,
    val items: HomeSectionState<ItemsResponse>,
    val continueWatching: HomeSectionState<ContinueWatchingResponse> = HomeSectionState.NotRequested,
    val artwork: HomeArtworkState = HomeArtworkState(),
) {
    val featuredItem: MediaItemDto? =
        continueWatching.valueOrNull()?.items?.firstOrNull()?.item
            ?: items.valueOrNull()?.items?.firstOrNull()
}

internal sealed interface HomeSectionState<out T> {
    data object NotRequested : HomeSectionState<Nothing>

    data class Available<T>(
        val value: T,
    ) : HomeSectionState<T>

    data class Unavailable(
        val diagnostics: SafeBrowseDiagnostics,
    ) : HomeSectionState<Nothing>
}

internal fun <T> HomeSectionState<T>.valueOrNull(): T? =
    when (this) {
        is HomeSectionState.Available -> value
        HomeSectionState.NotRequested,
        is HomeSectionState.Unavailable,
        -> null
    }

internal data class HomeArtworkState(
    val artworkByItemId: Map<String, List<PublicImageRefDto>> = emptyMap(),
    val failures: List<HomeArtworkFailure> = emptyList(),
) {
    val hasFailures: Boolean = failures.isNotEmpty()
}

internal data class HomeArtworkFailure(
    val itemId: String,
    val diagnostics: SafeBrowseDiagnostics,
)

private val emptyHomePage = PageInfo(
    limit = 0,
    offset = 0,
    returned = 0,
)

private val emptyHomeLibraries = LibraryListResponse(
    libraries = emptyList(),
    page = emptyHomePage,
)

private val emptyHomeItems = ItemsResponse(
    items = emptyList(),
    page = emptyHomePage,
)

internal sealed interface ItemDetailUiState {
    data object Idle : ItemDetailUiState
    data object Loading : ItemDetailUiState

    data class Content(
        val response: ItemDetailResponse,
        val userPlaybackState: UserPlaybackStateDto? = null,
    ) : ItemDetailUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : ItemDetailUiState
}

internal sealed interface LibraryDetailUiState {
    data object Idle : LibraryDetailUiState
    data object Loading : LibraryDetailUiState

    data class Content(
        val response: LibrarySourcesResponse,
    ) : LibraryDetailUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : LibraryDetailUiState
}

internal sealed interface PlaybackSelectionUiState {
    data object Idle : PlaybackSelectionUiState
    data object Loading : PlaybackSelectionUiState

    data class Content(
        val response: PlaybackDecisionResponse,
        val target: PlaybackRequestTarget?,
        val capabilities: PlaybackCapabilities,
    ) : PlaybackSelectionUiState

    data class Failure(
        val diagnostics: SafePlaybackDiagnostics,
    ) : PlaybackSelectionUiState
}

internal sealed interface SourceProbeUiState {
    data object Idle : SourceProbeUiState
    data object Loading : SourceProbeUiState

    data class Content(
        val response: SourceProbeResponse,
    ) : SourceProbeUiState

    data class Failure(
        val diagnostics: SafePlaybackDiagnostics,
    ) : SourceProbeUiState
}

internal sealed interface SearchUiState {
    data object Idle : SearchUiState
    data object Loading : SearchUiState

    data class Content(
        val response: SearchResponse,
        val isLoadingMore: Boolean = false,
        val loadMoreFailure: SafeBrowseDiagnostics? = null,
    ) : SearchUiState {
        val canLoadMore: Boolean = response.page.nextPageRequestOrNull() != null
    }

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : SearchUiState
}

internal sealed interface FacetUiState {
    data object Idle : FacetUiState
    data object Loading : FacetUiState

    data class Content(
        val response: FacetItemsResponse,
        val isLoadingMore: Boolean = false,
        val loadMoreFailure: SafeBrowseDiagnostics? = null,
    ) : FacetUiState {
        val canLoadMore: Boolean = response.page.nextPageRequestOrNull() != null
    }

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : FacetUiState

    data class ApiGap(
        val title: String,
        val body: String,
    ) : FacetUiState
}

internal sealed interface PersonDetailUiState {
    data object Idle : PersonDetailUiState
    data object Loading : PersonDetailUiState

    data class Content(
        val response: PersonResponse,
        val relatedItems: FacetItemsResponse,
    ) : PersonDetailUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : PersonDetailUiState
}

internal sealed interface RelationshipIndexUiState {
    data object Idle : RelationshipIndexUiState
    data object Loading : RelationshipIndexUiState

    data class Content(
        val family: RelationshipIndexFamily,
        val rows: List<RelationshipIndexRow>,
        val page: PageInfo,
        val isLoadingMore: Boolean = false,
        val loadMoreFailure: SafeBrowseDiagnostics? = null,
    ) : RelationshipIndexUiState {
        val canLoadMore: Boolean = page.nextPageRequestOrNull() != null
    }

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : RelationshipIndexUiState
}

internal enum class RelationshipIndexFamily(
    val label: String,
) {
    Genres("Genres"),
    Tags("Tags"),
}

internal data class RelationshipIndexRow(
    val title: String,
    val subtitle: String,
    val target: BrowseFacetTarget,
)

internal data class BrowseFacetTarget(
    val family: BrowseFacetUiFamily,
    val label: String,
    val id: String? = null,
) {
    val isPublicRouteBacked: Boolean =
        !id.isNullOrBlank() &&
            family in setOf(
                BrowseFacetUiFamily.Genre,
                BrowseFacetUiFamily.Tag,
                BrowseFacetUiFamily.Person,
            )
}

internal enum class BrowseFacetUiFamily(
    val label: String,
) {
    Genre("Genre"),
    Tag("Tag"),
    Person("Person"),
    Studio("Studio"),
    Collection("Collection"),
    Year("Year"),
    ItemKind("Title type"),
    Library("Media Library"),
    SourceMode("Playback mode"),
}

internal data class RelationshipRow(
    val title: String,
    val subtitle: String,
    val icon: ImageVector,
    val target: BrowseFacetTarget,
)

internal fun GenreListResponse.toRelationshipIndexContent(): RelationshipIndexUiState.Content =
    RelationshipIndexUiState.Content(
        family = RelationshipIndexFamily.Genres,
        rows = genres
            .filter { genre -> genre.id.isNotBlank() && genre.name.isNotBlank() }
            .map { genre ->
                RelationshipIndexRow(
                    title = genre.name,
                    subtitle = "Genre",
                    target = BrowseFacetTarget(
                        family = BrowseFacetUiFamily.Genre,
                        label = genre.name,
                        id = genre.id,
                    ),
                )
            },
        page = page,
    )

internal fun TagListResponse.toRelationshipIndexContent(): RelationshipIndexUiState.Content =
    RelationshipIndexUiState.Content(
        family = RelationshipIndexFamily.Tags,
        rows = tags
            .filter { tag -> tag.id.isNotBlank() && tag.name.isNotBlank() }
            .map { tag ->
                RelationshipIndexRow(
                    title = tag.name,
                    subtitle = "Tag",
                    target = BrowseFacetTarget(
                        family = BrowseFacetUiFamily.Tag,
                        label = tag.name,
                        id = tag.id,
                    ),
                )
            },
        page = page,
    )
