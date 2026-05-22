package dev.nako.android.ui.browse

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.components.NakoScreenColumn
import dev.nako.android.ui.components.NakoSectionHeader
import dev.nako.android.ui.components.NakoSurfaceCard
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextSecondary
import kotlin.math.roundToInt

@Composable
internal fun SearchScreen(
    query: String,
    state: SearchUiState,
    onQueryChange: (String) -> Unit,
    onSubmit: () -> Unit,
    onRetry: () -> Unit,
    onLoadMore: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    NakoScreenColumn {
        PageTitle(
            title = "Search",
            subtitle = "Find a known title on your server.",
            icon = Icons.Rounded.Search,
        )

        NakoSurfaceCard {
            Column(verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium)) {
                OutlinedTextField(
                    modifier = Modifier.fillMaxWidth(),
                    value = query,
                    onValueChange = onQueryChange,
                    singleLine = true,
                    label = { Text("Title") },
                    leadingIcon = {
                        Icon(
                            imageVector = Icons.Rounded.Search,
                            contentDescription = null,
                        )
                    },
                )
                Row(horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small)) {
                    Button(
                        onClick = onSubmit,
                    enabled = query.isNotBlank(),
                ) {
                        Text(stringResource(NakoStrings.search))
                    }
                }
            }
        }

        when (state) {
            SearchUiState.Idle -> EmptyCard(
                title = "Ready to search",
                body = "Enter a title or keyword for this server.",
            )
            SearchUiState.Loading -> LoadingCard(
                title = "Searching",
                body = "Loading matching titles from your server.",
            )
            is SearchUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is SearchUiState.Content -> SearchResults(
                state = state,
                onLoadMore = onLoadMore,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun SearchResults(
    state: SearchUiState.Content,
    onLoadMore: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    val hits = state.response.hits
    NakoSectionHeader(
        title = "Results",
        action = "${state.response.page.returned}",
    )
    if (hits.isEmpty()) {
        EmptyCard(
            title = "No matches",
            body = "No titles matched this search.",
        )
    } else {
        hits.forEach { hit ->
            MediaItemRow(
                item = hit.item,
                onOpenItem = onOpenItem,
                trailingLabel = "${(hit.score * 100).roundToInt()}%",
            )
        }
        Text(
            text = "Showing ${hits.size} result(s). Latest page added ${state.response.page.returned}.",
            color = NakoTextSecondary,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        LoadMoreFooter(
            canLoadMore = state.canLoadMore,
            isLoadingMore = state.isLoadingMore,
            failureMessage = state.loadMoreFailure?.userMessage,
            onLoadMore = onLoadMore,
        )
    }
}
