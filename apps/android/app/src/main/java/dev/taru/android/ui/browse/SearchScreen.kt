package dev.taru.android.ui.browse

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
import androidx.compose.ui.text.style.TextOverflow
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary
import kotlin.math.roundToInt

@Composable
internal fun SearchScreen(
    query: String,
    state: SearchUiState,
    onQueryChange: (String) -> Unit,
    onSubmit: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    TaruScrollColumn {
        PageTitle(
            title = "Search",
            subtitle = "Find a known title across the active server profile.",
            icon = Icons.Rounded.Search,
        )

        SurfaceCard {
            Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium)) {
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
                Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                    Button(
                        onClick = onSubmit,
                        enabled = query.isNotBlank(),
                    ) {
                        Text("Search")
                    }
                }
            }
        }

        when (state) {
            SearchUiState.Idle -> EmptyCard(
                title = "Ready to search",
                body = "Enter a title or keyword for the active server.",
            )
            SearchUiState.Loading -> LoadingCard(
                title = "Searching",
                body = "Fetching matching Media Items from the active server.",
            )
            is SearchUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is SearchUiState.Content -> SearchResults(
                state = state,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun SearchResults(
    state: SearchUiState.Content,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    val hits = state.response.hits
    SectionHeader(
        title = "Results",
        action = "${state.response.page.returned}",
    )
    if (hits.isEmpty()) {
        EmptyCard(
            title = "No matches",
            body = "The server returned no Media Items for this search.",
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
            text = "Showing ${state.response.page.returned} result(s).",
            color = TaruTextSecondary,
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
