package dev.taru.android.ui.browse

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.ui.graphics.vector.ImageVector
import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.ItemsResponse
import dev.taru.android.browse.LibrarySourcesResponse
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.PersonResponse
import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.browse.SearchResponse
import dev.taru.android.playback.PlaybackDecisionResponse
import dev.taru.android.playback.PlaybackCapabilities
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.SafePlaybackDiagnostics
import dev.taru.android.media.SourceProbeResponse
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.userplayback.ContinueWatchingResponse
import dev.taru.android.userplayback.UserPlaybackStateDto

internal enum class TaruDestination(
    val label: String,
    val icon: ImageVector,
) {
    Home("Home", Icons.Rounded.Home),
    Libraries("Libraries", Icons.AutoMirrored.Rounded.LibraryBooks),
    Search("Search", Icons.Rounded.Search),
    Settings("Settings", Icons.Rounded.Settings),
}

internal sealed interface TaruRoute {
    data object TopLevel : TaruRoute
    data class ItemDetail(val itemId: String) : TaruRoute
    data class LibraryDetail(val libraryId: String) : TaruRoute
    data class PersonDetail(val personId: String) : TaruRoute
    data class Player(val launch: PlaybackLaunchRequest) : TaruRoute
    data class BrowseFacet(val target: BrowseFacetTarget) : TaruRoute
    data object ServerProfile : TaruRoute
}

internal class TaruRouteStack private constructor(
    private val entries: List<TaruRoute>,
) {
    val current: TaruRoute = entries.last()
    val isAtRoot: Boolean = entries.size == 1
    val canPop: Boolean = !isAtRoot
    val routes: List<TaruRoute> = entries

    init {
        require(entries.isNotEmpty()) { "Taru route stack cannot be empty." }
        require(entries.first() == TaruRoute.TopLevel) {
            "Taru route stack must be rooted at top level."
        }
        require(entries.drop(1).none { it == TaruRoute.TopLevel }) {
            "Top level route cannot appear inside nested routes."
        }
    }

    fun push(route: TaruRoute): TaruRouteStack {
        require(route != TaruRoute.TopLevel) {
            "Use clearToRoot() instead of pushing top level."
        }
        return TaruRouteStack(entries + route)
    }

    fun pop(): TaruRouteStack =
        if (canPop) {
            TaruRouteStack(entries.dropLast(1))
        } else {
            this
        }

    fun clearToRoot(): TaruRouteStack = root()

    override fun equals(other: Any?): Boolean =
        other is TaruRouteStack && entries == other.entries

    override fun hashCode(): Int = entries.hashCode()

    override fun toString(): String = "TaruRouteStack(entries=$entries)"

    companion object {
        fun root(): TaruRouteStack = TaruRouteStack(listOf(TaruRoute.TopLevel))
    }
}

internal data class TaruBrowseNavigationState(
    val selectedDestination: TaruDestination = TaruDestination.Home,
    val routeStack: TaruRouteStack = TaruRouteStack.root(),
) {
    val currentRoute: TaruRoute = routeStack.current
    val navigationVisible: Boolean = routeStack.isAtRoot
    val canNavigateBack: Boolean = routeStack.canPop

    fun selectDestination(destination: TaruDestination): TaruBrowseNavigationState =
        copy(
            selectedDestination = destination,
            routeStack = routeStack.clearToRoot(),
        )

    fun open(route: TaruRoute): TaruBrowseNavigationState =
        copy(routeStack = routeStack.push(route))

    fun navigateBack(): TaruBrowseNavigationState =
        copy(routeStack = routeStack.pop())

    companion object {
        fun root(): TaruBrowseNavigationState = TaruBrowseNavigationState()
    }
}

internal sealed interface BrowseUiState {
    data object Loading : BrowseUiState

    data class Content(
        val libraries: LibraryListResponse,
        val items: ItemsResponse,
        val artworkByItemId: Map<String, List<PublicImageRefDto>> = emptyMap(),
        val continueWatching: ContinueWatchingResponse? = null,
    ) : BrowseUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : BrowseUiState
}

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
    ) : SearchUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : SearchUiState
}

internal sealed interface FacetUiState {
    data object Idle : FacetUiState
    data object Loading : FacetUiState

    data class Content(
        val response: FacetItemsResponse,
    ) : FacetUiState

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
    ItemKind("Media Item kind"),
    Library("Media Library"),
    SourceMode("Playback mode"),
}

internal data class RelationshipRow(
    val title: String,
    val subtitle: String,
    val icon: ImageVector,
    val target: BrowseFacetTarget,
)
