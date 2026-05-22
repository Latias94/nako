package dev.nako.android.ui.browse

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
private const val RoutePersonDetail = "person_detail"
private const val RouteRelationshipIndex = "relationship_index"
private const val RouteBrowseFacet = "browse_facet"
private const val RouteServerProfile = "server_profile"

private val NavigationStateJson = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

internal val NakoBrowseNavigationStateSaver: Saver<NakoBrowseNavigationState, String> =
    Saver(
        save = { it.toSaveablePayload() },
        restore = ::restoreNakoBrowseNavigationState,
    )

internal val BrowseShellStateSaver: Saver<BrowseShellState, String> =
    Saver(
        save = { it.toSaveablePayload() },
        restore = ::restoreBrowseShellState,
    )

internal fun BrowseShellState.toSaveablePayload(): String =
    navigation.toSaveablePayload()

internal fun restoreBrowseShellState(payload: String): BrowseShellState =
    BrowseShellState(
        navigation = restoreNakoBrowseNavigationState(payload),
    )

internal fun NakoBrowseNavigationState.toSaveablePayload(): String =
    NavigationStateJson.encodeToString(
        SavedBrowseNavigationState(
            selectedDestination = selectedDestination.name,
            routes = routeStack.routes
                .mapNotNull(NakoRoute::toSavedRouteOrNull)
                .ifEmpty { listOf(SavedNakoRoute(type = RouteTopLevel)) },
        ),
    )

internal fun restoreNakoBrowseNavigationState(payload: String): NakoBrowseNavigationState =
    runCatching {
        val saved = NavigationStateJson.decodeFromString<SavedBrowseNavigationState>(payload)
        if (saved.version != SaveVersion) {
            return@runCatching NakoBrowseNavigationState.root()
        }

        NakoBrowseNavigationState(
            selectedDestination = saved.selectedDestination.toDestinationOrHome(),
            routeStack = saved.routes.toRouteStack(),
        )
    }.getOrElse {
        NakoBrowseNavigationState.root()
    }

private fun NakoRoute.toSavedRouteOrNull(): SavedNakoRoute? =
    when (this) {
        NakoRoute.TopLevel -> SavedNakoRoute(type = RouteTopLevel)
        is NakoRoute.ItemDetail -> SavedNakoRoute(
            type = RouteItemDetail,
            itemId = itemId,
        )
        is NakoRoute.LibraryDetail -> SavedNakoRoute(
            type = RouteLibraryDetail,
            libraryId = libraryId,
        )
        is NakoRoute.PersonDetail -> SavedNakoRoute(
            type = RoutePersonDetail,
            personId = personId,
        )
        is NakoRoute.RelationshipIndex -> SavedNakoRoute(
            type = RouteRelationshipIndex,
            relationshipFamily = family.name,
        )
        is NakoRoute.BrowseFacet -> SavedNakoRoute(
            type = RouteBrowseFacet,
            facetFamily = target.family.name,
            facetLabel = target.label,
            facetId = target.id,
        )
        NakoRoute.ServerProfile -> SavedNakoRoute(type = RouteServerProfile)
        is NakoRoute.Player -> null
    }

private fun String.toDestinationOrHome(): NakoDestination =
    runCatching { NakoDestination.valueOf(this) }.getOrDefault(NakoDestination.Home)

private fun List<SavedNakoRoute>.toRouteStack(): NakoRouteStack =
    fold(NakoRouteStack.root()) { stack, savedRoute ->
        when (val route = savedRoute.toRouteOrNull()) {
            null -> stack
            NakoRoute.TopLevel -> stack.clearToRoot()
            else -> stack.push(route)
        }
    }

private fun SavedNakoRoute.toRouteOrNull(): NakoRoute? =
    when (type) {
        RouteTopLevel -> NakoRoute.TopLevel
        RouteItemDetail -> itemId
            ?.takeIf(String::isNotBlank)
            ?.let(NakoRoute::ItemDetail)
        RouteLibraryDetail -> libraryId
            ?.takeIf(String::isNotBlank)
            ?.let(NakoRoute::LibraryDetail)
        RoutePersonDetail -> personId
            ?.takeIf(String::isNotBlank)
            ?.let(NakoRoute::PersonDetail)
        RouteRelationshipIndex -> relationshipFamily
            ?.let { runCatching { RelationshipIndexFamily.valueOf(it) }.getOrNull() }
            ?.let(NakoRoute::RelationshipIndex)
        RouteBrowseFacet -> toBrowseFacetOrNull()
        RouteServerProfile -> NakoRoute.ServerProfile
        else -> null
    }

private fun SavedNakoRoute.toBrowseFacetOrNull(): NakoRoute.BrowseFacet? {
    val family = facetFamily
        ?.let { runCatching { BrowseFacetUiFamily.valueOf(it) }.getOrNull() }
        ?: return null
    val label = facetLabel?.takeIf(String::isNotBlank) ?: return null
    return NakoRoute.BrowseFacet(
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
    val routes: List<SavedNakoRoute>,
)

@Serializable
private data class SavedNakoRoute(
    val type: String,
    @SerialName("item_id")
    val itemId: String? = null,
    @SerialName("library_id")
    val libraryId: String? = null,
    @SerialName("person_id")
    val personId: String? = null,
    @SerialName("relationship_family")
    val relationshipFamily: String? = null,
    @SerialName("facet_family")
    val facetFamily: String? = null,
    @SerialName("facet_label")
    val facetLabel: String? = null,
    @SerialName("facet_id")
    val facetId: String? = null,
)
