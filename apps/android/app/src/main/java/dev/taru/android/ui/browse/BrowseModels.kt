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
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.SafeBrowseDiagnostics
import dev.taru.android.browse.SearchResponse
import dev.taru.android.playback.PlaybackDecisionResponse
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.SafePlaybackDiagnostics
import dev.taru.android.player.PlaybackLaunchRequest

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
    data class Player(val launch: PlaybackLaunchRequest) : TaruRoute
    data class BrowseFacet(val target: BrowseFacetTarget) : TaruRoute
    data object ServerProfile : TaruRoute
}

internal sealed interface BrowseUiState {
    data object Loading : BrowseUiState

    data class Content(
        val libraries: LibraryListResponse,
        val items: ItemsResponse,
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
    ) : ItemDetailUiState

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : ItemDetailUiState
}

internal sealed interface PlaybackSelectionUiState {
    data object Idle : PlaybackSelectionUiState
    data object Loading : PlaybackSelectionUiState

    data class Content(
        val response: PlaybackDecisionResponse,
        val target: PlaybackRequestTarget?,
    ) : PlaybackSelectionUiState

    data class Failure(
        val diagnostics: SafePlaybackDiagnostics,
    ) : PlaybackSelectionUiState
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

internal data class SettingsRow(
    val label: String,
    val value: String?,
    val icon: ImageVector,
    val onClick: (() -> Unit)? = null,
)
