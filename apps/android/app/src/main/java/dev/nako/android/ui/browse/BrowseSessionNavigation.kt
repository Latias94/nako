package dev.nako.android.ui.browse

internal class BrowseSessionNavigation(
    private val routeStatePolicy: BrowseRouteStatePolicy,
) {
    fun reduce(
        current: BrowseShellState,
        action: BrowseAction,
    ): BrowseShellState =
        when (action) {
            BrowseAction.Back -> current.copy(
                navigation = current.navigation.navigateBack(),
            )
            BrowseAction.OpenServerProfile -> current.copy(
                navigation = current.navigation.open(NakoRoute.ServerProfile),
            )
            is BrowseAction.OpenFacet -> current.copy(
                navigation = current.navigation.open(NakoRoute.BrowseFacet(action.target)),
            )
            is BrowseAction.OpenItem -> current.copy(
                navigation = current.navigation.open(NakoRoute.ItemDetail(action.itemId)),
            )
            is BrowseAction.OpenLibraryDetail -> current.copy(
                navigation = current.navigation.open(NakoRoute.LibraryDetail(action.libraryId)),
            )
            is BrowseAction.OpenPersonDetail -> current.copy(
                navigation = current.navigation.open(NakoRoute.PersonDetail(action.personId)),
            )
            is BrowseAction.OpenRelationshipIndex -> current.copy(
                navigation = current.navigation.open(NakoRoute.RelationshipIndex(action.family)),
            )
            is BrowseAction.OpenPlayer -> current.copy(
                navigation = current.navigation.open(NakoRoute.Player(action.launch)),
            )
            is BrowseAction.SelectDestination -> current.copy(
                navigation = current.navigation.selectDestination(action.destination),
            )
            BrowseAction.LoadHome,
            BrowseAction.RetryHome,
            BrowseAction.SubmitSearch,
            BrowseAction.RetrySearch,
            BrowseAction.LoadMoreSearch,
            BrowseAction.LoadMoreRelationshipIndex,
            BrowseAction.LoadMoreFacet,
            BrowseAction.RetryCurrentRoute,
            BrowseAction.RetrySourceProbe,
            BrowseAction.RetryPlaybackDecision,
            is BrowseAction.RouteDisplayed,
            is BrowseAction.SearchQueryChanged,
            is BrowseAction.SelectSource,
            is BrowseAction.RequestPlayback,
            is BrowseAction.StartPlayback,
            -> current
        }

    fun reduceAndPrepare(
        current: BrowseShellState,
        action: BrowseAction,
    ): BrowseShellState =
        routeStatePolicy.prepare(
            previous = current,
            next = reduce(current, action),
        )
}
