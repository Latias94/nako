package dev.taru.android.ui.browse

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class BrowseRouteLoadingSession(
    private val store: BrowseSessionStore,
    private val scope: CoroutineScope,
    private val dataSource: BrowseDataSource,
    private val routeStatePolicy: BrowseRouteStatePolicy,
    private val detailSession: BrowseItemDetailSession,
) {
    fun loadCurrentRoute(): Job? =
        loadRoute(store.value.currentRoute)

    fun loadRoute(route: TaruRoute): Job? =
        when (route) {
            is TaruRoute.ItemDetail -> detailSession.loadItemDetail(route.itemId)
            is TaruRoute.LibraryDetail -> loadLibraryDetail(route.libraryId)
            is TaruRoute.PersonDetail -> loadPersonDetail(route.personId)
            is TaruRoute.RelationshipIndex -> loadRelationshipIndex(route.family)
            is TaruRoute.BrowseFacet -> loadFacet(route.target)
            is TaruRoute.Player -> null
            else -> {
                store.update(routeStatePolicy::clearForNonLoadableRoute)
                null
            }
        }

    fun loadMoreRelationshipIndex(): Job? {
        val route = store.value.currentRoute as? TaruRoute.RelationshipIndex ?: return null
        val currentContent =
            store.value.relationshipIndexState as? RelationshipIndexUiState.Content ?: return null
        if (currentContent.isLoadingMore || currentContent.family != route.family) {
            return null
        }

        val nextPage = currentContent.page.nextPageRequestOrNull() ?: return null
        val requestId = routeStatePolicy.beginRelationshipIndex()
        store.update { current ->
            val content =
                current.relationshipIndexState as? RelationshipIndexUiState.Content ?: return@update current
            current.copy(
                relationshipIndexState = content.copy(
                    isLoadingMore = true,
                    loadMoreFailure = null,
                ),
            )
        }

        return scope.launch {
            val nextState = dataSource.loadRelationshipIndex(
                family = route.family,
                page = nextPage,
            )
            store.update { current ->
                val content =
                    current.relationshipIndexState as? RelationshipIndexUiState.Content ?: return@update current
                val routeStillCurrent = current.currentRoute == route
                if (!routeStatePolicy.acceptsRelationshipIndex(requestId) || !routeStillCurrent) {
                    return@update current
                }

                current.copy(
                    relationshipIndexState = when (nextState) {
                        is RelationshipIndexUiState.Content -> content.appendPage(nextState).copy(
                            isLoadingMore = false,
                            loadMoreFailure = null,
                        )
                        is RelationshipIndexUiState.Failure -> content.copy(
                            isLoadingMore = false,
                            loadMoreFailure = nextState.diagnostics,
                        )
                        RelationshipIndexUiState.Idle,
                        RelationshipIndexUiState.Loading,
                        -> content.copy(isLoadingMore = false)
                    },
                )
            }
        }
    }

    fun loadMoreFacet(): Job? {
        val route = store.value.currentRoute as? TaruRoute.BrowseFacet ?: return null
        val currentContent = store.value.facetState as? FacetUiState.Content ?: return null
        if (
            currentContent.isLoadingMore ||
            !route.target.isPublicRouteBacked ||
            !currentContent.response.matchesTarget(route.target)
        ) {
            return null
        }

        val nextPage = currentContent.response.page.nextPageRequestOrNull() ?: return null
        val requestId = routeStatePolicy.beginFacet()
        store.update { current ->
            val content = current.facetState as? FacetUiState.Content ?: return@update current
            current.copy(
                facetState = content.copy(
                    isLoadingMore = true,
                    loadMoreFailure = null,
                ),
            )
        }

        return scope.launch {
            val nextState = dataSource.loadFacet(
                target = route.target,
                page = nextPage,
            )
            store.update { current ->
                val content = current.facetState as? FacetUiState.Content ?: return@update current
                val routeStillCurrent = current.currentRoute == route
                if (!routeStatePolicy.acceptsFacet(requestId) || !routeStillCurrent) {
                    return@update current
                }

                current.copy(
                    facetState = when (nextState) {
                        is FacetUiState.Content -> content.copy(
                            response = content.response.appendPage(nextState.response),
                            isLoadingMore = false,
                            loadMoreFailure = null,
                        )
                        is FacetUiState.Failure -> content.copy(
                            isLoadingMore = false,
                            loadMoreFailure = nextState.diagnostics,
                        )
                        FacetUiState.Idle,
                        FacetUiState.Loading,
                        is FacetUiState.ApiGap,
                        -> content.copy(isLoadingMore = false)
                    },
                )
            }
        }
    }

    private fun loadLibraryDetail(libraryId: String): Job {
        val requestId = routeStatePolicy.beginLibraryDetail()
        store.update { it.copy(libraryDetailState = LibraryDetailUiState.Loading) }
        return scope.launch {
            val nextState = dataSource.loadLibraryDetail(libraryId)
            store.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.LibraryDetail(libraryId)
                if (routeStatePolicy.acceptsLibraryDetail(requestId) && routeStillCurrent) {
                    current.copy(libraryDetailState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadPersonDetail(personId: String): Job {
        val requestId = routeStatePolicy.beginPersonDetail()
        store.update { it.copy(personDetailState = PersonDetailUiState.Loading) }
        return scope.launch {
            val nextState = dataSource.loadPersonDetail(personId)
            store.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.PersonDetail(personId)
                if (routeStatePolicy.acceptsPersonDetail(requestId) && routeStillCurrent) {
                    current.copy(personDetailState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadRelationshipIndex(family: RelationshipIndexFamily): Job {
        val requestId = routeStatePolicy.beginRelationshipIndex()
        store.update { it.copy(relationshipIndexState = RelationshipIndexUiState.Loading) }
        return scope.launch {
            val nextState = dataSource.loadRelationshipIndex(family)
            store.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.RelationshipIndex(family)
                if (routeStatePolicy.acceptsRelationshipIndex(requestId) && routeStillCurrent) {
                    current.copy(relationshipIndexState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadFacet(target: BrowseFacetTarget): Job? {
        val requestId = routeStatePolicy.beginFacet()
        if (!target.isPublicRouteBacked) {
            store.update { it.copy(facetState = target.apiGapState()) }
            return null
        }

        store.update { it.copy(facetState = FacetUiState.Loading) }
        return scope.launch {
            val nextState = dataSource.loadFacet(target)
            store.update { current ->
                val routeStillCurrent = current.currentRoute == TaruRoute.BrowseFacet(target)
                if (routeStatePolicy.acceptsFacet(requestId) && routeStillCurrent) {
                    current.copy(facetState = nextState)
                } else {
                    current
                }
            }
        }
    }
}
