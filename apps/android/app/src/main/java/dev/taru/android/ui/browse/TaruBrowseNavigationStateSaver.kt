package dev.taru.android.ui.browse

import androidx.compose.runtime.saveable.Saver
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

private const val SaveVersion = 1
private const val RouteTopLevel = "top_level"
private const val RouteItemDetail = "item_detail"
private const val RouteLibraryDetail = "library_detail"
private const val RouteBrowseFacet = "browse_facet"
private const val RouteServerProfile = "server_profile"

private val NavigationStateJson = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

internal val TaruBrowseNavigationStateSaver: Saver<TaruBrowseNavigationState, String> =
    Saver(
        save = { it.toSaveablePayload() },
        restore = ::restoreTaruBrowseNavigationState,
    )

internal fun TaruBrowseNavigationState.toSaveablePayload(): String =
    NavigationStateJson.encodeToString(
        SavedBrowseNavigationState(
            selectedDestination = selectedDestination.name,
            routes = routeStack.routes
                .mapNotNull(TaruRoute::toSavedRouteOrNull)
                .ifEmpty { listOf(SavedTaruRoute(type = RouteTopLevel)) },
        ),
    )

internal fun restoreTaruBrowseNavigationState(payload: String): TaruBrowseNavigationState =
    runCatching {
        val saved = NavigationStateJson.decodeFromString<SavedBrowseNavigationState>(payload)
        if (saved.version != SaveVersion) {
            return@runCatching TaruBrowseNavigationState.root()
        }

        TaruBrowseNavigationState(
            selectedDestination = saved.selectedDestination.toDestinationOrHome(),
            routeStack = saved.routes.toRouteStack(),
        )
    }.getOrElse {
        TaruBrowseNavigationState.root()
    }

private fun TaruRoute.toSavedRouteOrNull(): SavedTaruRoute? =
    when (this) {
        TaruRoute.TopLevel -> SavedTaruRoute(type = RouteTopLevel)
        is TaruRoute.ItemDetail -> SavedTaruRoute(
            type = RouteItemDetail,
            itemId = itemId,
        )
        is TaruRoute.LibraryDetail -> SavedTaruRoute(
            type = RouteLibraryDetail,
            libraryId = libraryId,
        )
        is TaruRoute.BrowseFacet -> SavedTaruRoute(
            type = RouteBrowseFacet,
            facetFamily = target.family.name,
            facetLabel = target.label,
            facetId = target.id,
        )
        TaruRoute.ServerProfile -> SavedTaruRoute(type = RouteServerProfile)
        is TaruRoute.Player -> null
    }

private fun String.toDestinationOrHome(): TaruDestination =
    runCatching { TaruDestination.valueOf(this) }.getOrDefault(TaruDestination.Home)

private fun List<SavedTaruRoute>.toRouteStack(): TaruRouteStack =
    fold(TaruRouteStack.root()) { stack, savedRoute ->
        when (val route = savedRoute.toRouteOrNull()) {
            null -> stack
            TaruRoute.TopLevel -> stack.clearToRoot()
            else -> stack.push(route)
        }
    }

private fun SavedTaruRoute.toRouteOrNull(): TaruRoute? =
    when (type) {
        RouteTopLevel -> TaruRoute.TopLevel
        RouteItemDetail -> itemId
            ?.takeIf(String::isNotBlank)
            ?.let(TaruRoute::ItemDetail)
        RouteLibraryDetail -> libraryId
            ?.takeIf(String::isNotBlank)
            ?.let(TaruRoute::LibraryDetail)
        RouteBrowseFacet -> toBrowseFacetOrNull()
        RouteServerProfile -> TaruRoute.ServerProfile
        else -> null
    }

private fun SavedTaruRoute.toBrowseFacetOrNull(): TaruRoute.BrowseFacet? {
    val family = facetFamily
        ?.let { runCatching { BrowseFacetUiFamily.valueOf(it) }.getOrNull() }
        ?: return null
    val label = facetLabel?.takeIf(String::isNotBlank) ?: return null
    return TaruRoute.BrowseFacet(
        BrowseFacetTarget(
            family = family,
            label = label,
            id = facetId,
        ),
    )
}

@Serializable
private data class SavedBrowseNavigationState(
    val version: Int = SaveVersion,
    @SerialName("selected_destination")
    val selectedDestination: String,
    val routes: List<SavedTaruRoute>,
)

@Serializable
private data class SavedTaruRoute(
    val type: String,
    @SerialName("item_id")
    val itemId: String? = null,
    @SerialName("library_id")
    val libraryId: String? = null,
    @SerialName("facet_family")
    val facetFamily: String? = null,
    @SerialName("facet_label")
    val facetLabel: String? = null,
    @SerialName("facet_id")
    val facetId: String? = null,
)
