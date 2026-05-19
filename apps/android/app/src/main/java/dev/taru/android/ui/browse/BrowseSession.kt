package dev.taru.android.ui.browse

import dev.taru.android.player.PlaybackLaunchRequest

internal data class BrowseShellState(
    val navigation: TaruBrowseNavigationState = TaruBrowseNavigationState.root(),
) {
    val selectedDestination: TaruDestination = navigation.selectedDestination
    val currentRoute: TaruRoute = navigation.currentRoute
    val navigationVisible: Boolean = navigation.navigationVisible
    val canNavigateBack: Boolean = navigation.canNavigateBack
}

internal sealed interface BrowseAction {
    data class SelectDestination(val destination: TaruDestination) : BrowseAction
    data class OpenItem(val itemId: String) : BrowseAction
    data class OpenLibraryDetail(val libraryId: String) : BrowseAction
    data class OpenFacet(val target: BrowseFacetTarget) : BrowseAction
    data class OpenPlayer(val launch: PlaybackLaunchRequest) : BrowseAction
    data object OpenServerProfile : BrowseAction
    data object Back : BrowseAction
}

internal class BrowseSession {
    fun reduce(
        current: BrowseShellState,
        action: BrowseAction,
    ): BrowseShellState =
        when (action) {
            BrowseAction.Back -> current.copy(
                navigation = current.navigation.navigateBack(),
            )
            BrowseAction.OpenServerProfile -> current.copy(
                navigation = current.navigation.open(TaruRoute.ServerProfile),
            )
            is BrowseAction.OpenFacet -> current.copy(
                navigation = current.navigation.open(TaruRoute.BrowseFacet(action.target)),
            )
            is BrowseAction.OpenItem -> current.copy(
                navigation = current.navigation.open(TaruRoute.ItemDetail(action.itemId)),
            )
            is BrowseAction.OpenLibraryDetail -> current.copy(
                navigation = current.navigation.open(TaruRoute.LibraryDetail(action.libraryId)),
            )
            is BrowseAction.OpenPlayer -> current.copy(
                navigation = current.navigation.open(TaruRoute.Player(action.launch)),
            )
            is BrowseAction.SelectDestination -> current.copy(
                navigation = current.navigation.selectDestination(action.destination),
            )
        }
}
