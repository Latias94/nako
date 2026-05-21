package dev.taru.android.ui.browse

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class BrowseSearchSession(
    private val store: BrowseSessionStore,
    private val scope: CoroutineScope,
    private val dataSource: BrowseDataSource,
) {
    private var searchRequestId: Long = 0

    fun submitSearch(): Job? {
        val query = store.value.searchQuery.trim()
        searchRequestId += 1
        if (query.isBlank()) {
            store.update {
                it.copy(
                    submittedSearchQuery = "",
                    searchState = SearchUiState.Idle,
                )
            }
            return null
        }

        return loadSearch(query)
    }

    fun retrySearch(): Job? {
        val query = store.value.submittedSearchQuery.trim()
        searchRequestId += 1
        if (query.isBlank()) {
            store.update { it.copy(searchState = SearchUiState.Idle) }
            return null
        }

        return loadSearch(query)
    }

    fun loadMoreSearch(): Job? {
        val currentContent = store.value.searchState as? SearchUiState.Content ?: return null
        if (currentContent.isLoadingMore) {
            return null
        }

        val nextPage = currentContent.response.page.nextPageRequestOrNull() ?: return null
        val query = store.value.submittedSearchQuery.trim()
        if (query.isBlank()) {
            return null
        }

        val requestId = ++searchRequestId
        store.update { current ->
            val content = current.searchState as? SearchUiState.Content ?: return@update current
            current.copy(
                searchState = content.copy(
                    isLoadingMore = true,
                    loadMoreFailure = null,
                ),
            )
        }

        return scope.launch {
            val nextState = dataSource.search(query = query, page = nextPage)
            store.update { current ->
                val content = current.searchState as? SearchUiState.Content ?: return@update current
                if (requestId != searchRequestId || current.submittedSearchQuery != query) {
                    return@update current
                }

                current.copy(
                    searchState = when (nextState) {
                        is SearchUiState.Content -> content.copy(
                            response = content.response.appendPage(nextState.response),
                            isLoadingMore = false,
                            loadMoreFailure = null,
                        )
                        is SearchUiState.Failure -> content.copy(
                            isLoadingMore = false,
                            loadMoreFailure = nextState.diagnostics,
                        )
                        SearchUiState.Idle,
                        SearchUiState.Loading,
                        -> content.copy(
                            isLoadingMore = false,
                        )
                    },
                )
            }
        }
    }

    private fun loadSearch(query: String): Job {
        val requestId = searchRequestId
        store.update {
            it.copy(
                submittedSearchQuery = query,
                searchState = SearchUiState.Loading,
            )
        }
        return scope.launch {
            val nextState = dataSource.search(query)
            store.update { current ->
                if (requestId == searchRequestId && current.submittedSearchQuery == query) {
                    current.copy(searchState = nextState)
                } else {
                    current
                }
            }
        }
    }
}
