package dev.nako.android.ui.screens.person

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Badge
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.SortByAlpha
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.browse.PersonDto
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.browse.EmptyCard
import dev.nako.android.ui.browse.FailureCard
import dev.nako.android.ui.browse.InfoCard
import dev.nako.android.ui.browse.LoadingCard
import dev.nako.android.ui.browse.MediaPosterRow
import dev.nako.android.ui.browse.PersonDetailUiState
import dev.nako.android.ui.components.NakoArtworkBackdrop
import dev.nako.android.ui.components.NakoIconBadge
import dev.nako.android.ui.components.NakoScreenColumn
import dev.nako.android.ui.components.NakoSectionHeader
import dev.nako.android.ui.components.NakoStatusChip
import dev.nako.android.ui.components.NakoSurfaceCard
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextMuted
import dev.nako.android.ui.theme.NakoTextSecondary

@Composable
internal fun PersonDetailRouteContent(
    state: PersonDetailUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    NakoScreenColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = stringResource(NakoStrings.back),
            )
        }
        when (state) {
            PersonDetailUiState.Idle,
            PersonDetailUiState.Loading,
            -> LoadingCard(
                title = "Loading Person",
                body = "Loading credits, identity, and related titles.",
            )
            is PersonDetailUiState.Failure -> FailureCard(
                diagnostics = state.diagnostics,
                onRetry = onRetry,
                onChangeServer = onChangeServer,
            )
            is PersonDetailUiState.Content -> PersonDetailScreen(
                person = state.response.person,
                relatedItems = state.relatedItems.items,
                returned = state.relatedItems.page.returned,
                onOpenItem = onOpenItem,
            )
        }
    }
}

@Composable
private fun PersonDetailScreen(
    person: PersonDto,
    relatedItems: List<MediaItemDto>,
    returned: Int,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    PersonHeader(
        person = person,
        relatedCount = returned,
    )

    person.overview
        ?.takeIf { it.isNotBlank() }
        ?.let { overview ->
            InfoCard(
                title = "Overview",
                body = overview,
            )
        }

    PersonFacts(person = person)

    NakoSectionHeader(
        title = "Related Titles",
        action = returned.toString(),
    )
    if (relatedItems.isEmpty()) {
        EmptyCard(
            title = "No related titles",
            body = "No visible titles are linked to this person yet.",
        )
    } else {
        MediaPosterRow(
            items = relatedItems,
            onOpenItem = onOpenItem,
        )
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun PersonHeader(
    person: PersonDto,
    relatedCount: Int,
) {
    ElevatedCard(
        modifier = Modifier.fillMaxWidth(),
        shape = NakoShape.medium,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 210.dp),
        ) {
            NakoArtworkBackdrop(
                title = person.name,
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(NakoSpacing.large),
                verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    NakoIconBadge(icon = Icons.Rounded.Person)
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
                    ) {
                        Text(
                            text = person.name,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = person.sortName?.takeIf { it.isNotBlank() } ?: "Person",
                            color = NakoTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                ) {
                    NakoStatusChip(text = "Person")
                    NakoStatusChip(text = "$relatedCount related")
                    person.externalIds
                        .takeIf { it.isNotEmpty() }
                        ?.let { NakoStatusChip(text = "${it.size} external") }
                }
            }
        }
    }
}

@Composable
private fun PersonFacts(person: PersonDto) {
    val sortName = person.sortName?.takeIf { it.isNotBlank() }
    if (sortName == null && person.externalIds.isEmpty()) return

    NakoSectionHeader(title = "Identity")
    NakoSurfaceCard {
        sortName?.let {
            PersonFactRow(
                title = "Sort Name",
                value = it,
                icon = Icons.Rounded.SortByAlpha,
            )
        }
        if (person.externalIds.isNotEmpty()) {
            PersonFactRow(
                title = "External IDs",
                value = "${person.externalIds.size} linked",
                icon = Icons.Rounded.Badge,
            )
        }
    }
}

@Composable
private fun PersonFactRow(
    title: String,
    value: String,
    icon: ImageVector,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        NakoIconBadge(icon = icon, compact = true)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = value,
                color = NakoTextMuted,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
