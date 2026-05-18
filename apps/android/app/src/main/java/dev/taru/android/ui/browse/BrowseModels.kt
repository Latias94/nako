package dev.taru.android.ui.browse

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.LibraryBooks
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.ui.graphics.vector.ImageVector
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.ItemsResponse
import dev.taru.android.browse.LibraryListResponse
import dev.taru.android.browse.SafeBrowseDiagnostics

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
    data class BrowseFacet(val title: String) : TaruRoute
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

internal data class RelationshipRow(
    val title: String,
    val subtitle: String,
    val icon: ImageVector,
)

internal data class SettingsRow(
    val label: String,
    val value: String?,
    val icon: ImageVector,
    val onClick: (() -> Unit)? = null,
)
