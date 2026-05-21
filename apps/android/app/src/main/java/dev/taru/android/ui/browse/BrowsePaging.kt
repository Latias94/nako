package dev.taru.android.ui.browse

import dev.taru.android.browse.FacetItemsResponse
import dev.taru.android.browse.BrowseFacetFamily
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.SearchResponse

internal fun PageInfo.nextPageRequestOrNull(): PageRequest? =
    if (limit > 0 && returned >= limit) {
        PageRequest(
            limit = limit,
            offset = offset + returned,
        )
    } else {
        null
    }

internal fun SearchResponse.appendPage(next: SearchResponse): SearchResponse =
    next.copy(hits = hits + next.hits)

internal fun FacetItemsResponse.appendPage(next: FacetItemsResponse): FacetItemsResponse =
    next.copy(items = items + next.items)

internal fun FacetItemsResponse.matchesTarget(target: BrowseFacetTarget): Boolean =
    facetId == target.id &&
        when (family) {
            BrowseFacetFamily.Genre -> target.family == BrowseFacetUiFamily.Genre
            BrowseFacetFamily.Tag -> target.family == BrowseFacetUiFamily.Tag
            BrowseFacetFamily.Person -> target.family == BrowseFacetUiFamily.Person
        }

internal fun RelationshipIndexUiState.Content.appendPage(
    next: RelationshipIndexUiState.Content,
): RelationshipIndexUiState.Content =
    next.copy(rows = rows + next.rows)
