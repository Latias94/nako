package dev.taru.android.ui.browse

internal data class BrowseRouteRequestIds(
    val libraryDetail: Long = 0,
    val personDetail: Long = 0,
    val relationshipIndex: Long = 0,
    val facet: Long = 0,
    val detail: Long = 0,
    val sourceProbe: Long = 0,
    val playbackSelection: Long = 0,
)

internal class BrowseRouteStatePolicy(
    requestIds: BrowseRouteRequestIds = BrowseRouteRequestIds(),
) {
    var requestIds: BrowseRouteRequestIds = requestIds
        private set

    fun prepare(
        previous: BrowseShellState,
        next: BrowseShellState,
    ): BrowseShellState {
        if (previous.currentRoute == next.currentRoute) {
            return next
        }
        if (next.currentRoute is TaruRoute.Player) {
            return next
        }

        var prepared = next
        prepared = prepareDetailFamily(prepared)
        prepared = prepareLibraryFamily(prepared)
        prepared = preparePersonFamily(prepared)
        prepared = prepareRelationshipFamily(prepared)
        prepared = prepareFacetFamily(prepared)
        return prepared
    }

    fun clearForNonLoadableRoute(state: BrowseShellState): BrowseShellState {
        requestIds = requestIds.copy(
            detail = requestIds.detail + 1,
            libraryDetail = requestIds.libraryDetail + 1,
            personDetail = requestIds.personDetail + 1,
            relationshipIndex = requestIds.relationshipIndex + 1,
            facet = requestIds.facet + 1,
            sourceProbe = requestIds.sourceProbe + 1,
            playbackSelection = requestIds.playbackSelection + 1,
        )
        return state.copy(
            detailState = ItemDetailUiState.Idle,
            selectedSourceId = null,
            resumePosition = null,
            sourceProbeState = SourceProbeUiState.Idle,
            playbackRequestSourceId = null,
            playbackState = PlaybackSelectionUiState.Idle,
            libraryDetailState = LibraryDetailUiState.Idle,
            personDetailState = PersonDetailUiState.Idle,
            relationshipIndexState = RelationshipIndexUiState.Idle,
            facetState = FacetUiState.Idle,
        )
    }

    fun beginLibraryDetail(): Long {
        val requestId = requestIds.libraryDetail + 1
        requestIds = requestIds.copy(libraryDetail = requestId)
        return requestId
    }

    fun beginPersonDetail(): Long {
        val requestId = requestIds.personDetail + 1
        requestIds = requestIds.copy(personDetail = requestId)
        return requestId
    }

    fun beginRelationshipIndex(): Long {
        val requestId = requestIds.relationshipIndex + 1
        requestIds = requestIds.copy(relationshipIndex = requestId)
        return requestId
    }

    fun beginFacet(): Long {
        val requestId = requestIds.facet + 1
        requestIds = requestIds.copy(facet = requestId)
        return requestId
    }

    fun beginItemDetail(): Long {
        val requestId = requestIds.detail + 1
        requestIds = requestIds.copy(detail = requestId)
        return requestId
    }

    fun invalidateSourceProbe() {
        requestIds = requestIds.copy(sourceProbe = requestIds.sourceProbe + 1)
    }

    fun beginSourceProbe(): Long {
        val requestId = requestIds.sourceProbe + 1
        requestIds = requestIds.copy(sourceProbe = requestId)
        return requestId
    }

    fun invalidatePlaybackSelection() {
        requestIds = requestIds.copy(playbackSelection = requestIds.playbackSelection + 1)
    }

    fun beginPlaybackSelection(): Long {
        val requestId = requestIds.playbackSelection + 1
        requestIds = requestIds.copy(playbackSelection = requestId)
        return requestId
    }

    fun beginSourceAndPlaybackSelection() {
        requestIds = requestIds.copy(
            sourceProbe = requestIds.sourceProbe + 1,
            playbackSelection = requestIds.playbackSelection + 1,
        )
    }

    fun acceptsLibraryDetail(requestId: Long): Boolean =
        requestId == requestIds.libraryDetail

    fun acceptsPersonDetail(requestId: Long): Boolean =
        requestId == requestIds.personDetail

    fun acceptsRelationshipIndex(requestId: Long): Boolean =
        requestId == requestIds.relationshipIndex

    fun acceptsFacet(requestId: Long): Boolean =
        requestId == requestIds.facet

    fun acceptsItemDetail(requestId: Long): Boolean =
        requestId == requestIds.detail

    fun acceptsSourceProbe(requestId: Long): Boolean =
        requestId == requestIds.sourceProbe

    fun acceptsPlaybackSelection(requestId: Long): Boolean =
        requestId == requestIds.playbackSelection

    private fun prepareDetailFamily(state: BrowseShellState): BrowseShellState {
        requestIds = requestIds.copy(
            detail = requestIds.detail + 1,
            sourceProbe = requestIds.sourceProbe + 1,
            playbackSelection = requestIds.playbackSelection + 1,
        )
        return when (state.currentRoute) {
            is TaruRoute.ItemDetail -> state.copy(
                detailState = ItemDetailUiState.Loading,
                selectedSourceId = null,
                resumePosition = null,
                sourceProbeState = SourceProbeUiState.Idle,
                playbackRequestSourceId = null,
                playbackState = PlaybackSelectionUiState.Idle,
            )
            else -> state.copy(
                detailState = ItemDetailUiState.Idle,
                selectedSourceId = null,
                resumePosition = null,
                sourceProbeState = SourceProbeUiState.Idle,
                playbackRequestSourceId = null,
                playbackState = PlaybackSelectionUiState.Idle,
            )
        }
    }

    private fun prepareLibraryFamily(state: BrowseShellState): BrowseShellState {
        requestIds = requestIds.copy(libraryDetail = requestIds.libraryDetail + 1)
        return when (state.currentRoute) {
            is TaruRoute.LibraryDetail -> state.copy(libraryDetailState = LibraryDetailUiState.Loading)
            else -> state.copy(libraryDetailState = LibraryDetailUiState.Idle)
        }
    }

    private fun preparePersonFamily(state: BrowseShellState): BrowseShellState {
        requestIds = requestIds.copy(personDetail = requestIds.personDetail + 1)
        return when (state.currentRoute) {
            is TaruRoute.PersonDetail -> state.copy(personDetailState = PersonDetailUiState.Loading)
            else -> state.copy(personDetailState = PersonDetailUiState.Idle)
        }
    }

    private fun prepareRelationshipFamily(state: BrowseShellState): BrowseShellState {
        requestIds = requestIds.copy(relationshipIndex = requestIds.relationshipIndex + 1)
        return when (state.currentRoute) {
            is TaruRoute.RelationshipIndex -> state.copy(relationshipIndexState = RelationshipIndexUiState.Loading)
            else -> state.copy(relationshipIndexState = RelationshipIndexUiState.Idle)
        }
    }

    private fun prepareFacetFamily(state: BrowseShellState): BrowseShellState {
        requestIds = requestIds.copy(facet = requestIds.facet + 1)
        return when (val route = state.currentRoute) {
            is TaruRoute.BrowseFacet -> state.copy(
                facetState = if (route.target.isPublicRouteBacked) {
                    FacetUiState.Loading
                } else {
                    route.target.apiGapState()
                },
            )
            else -> state.copy(facetState = FacetUiState.Idle)
        }
    }
}
