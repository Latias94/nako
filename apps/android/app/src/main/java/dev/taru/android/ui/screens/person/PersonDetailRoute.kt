package dev.taru.android.ui.screens.person

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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PersonDto
import dev.taru.android.ui.browse.EmptyCard
import dev.taru.android.ui.browse.FailureCard
import dev.taru.android.ui.browse.IconBadge
import dev.taru.android.ui.browse.InfoCard
import dev.taru.android.ui.browse.LoadingCard
import dev.taru.android.ui.browse.MediaPosterRow
import dev.taru.android.ui.browse.PersonDetailUiState
import dev.taru.android.ui.browse.SectionHeader
import dev.taru.android.ui.browse.StatusChip
import dev.taru.android.ui.browse.SurfaceCard
import dev.taru.android.ui.browse.TaruScrollColumn
import dev.taru.android.ui.browse.ArtworkBackdrop
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@Composable
internal fun PersonDetailRouteContent(
    state: PersonDetailUiState,
    onBack: () -> Unit,
    onRetry: () -> Unit,
    onChangeServer: () -> Unit,
    onOpenItem: (MediaItemDto) -> Unit,
) {
    TaruScrollColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = "Back",
            )
        }
        when (state) {
            PersonDetailUiState.Idle,
            PersonDetailUiState.Loading,
            -> LoadingCard(
                title = "Loading Person",
                body = "Gathering credits, identity, and related Media Items.",
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

    SectionHeader(
        title = "Related Media Items",
        action = returned.toString(),
    )
    if (relatedItems.isEmpty()) {
        EmptyCard(
            title = "No Related Media Items",
            body = "No visible Media Items are linked to this person yet.",
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
        shape = TaruShape.medium,
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(min = 210.dp),
        ) {
            ArtworkBackdrop(
                title = person.name,
                modifier = Modifier.matchParentSize(),
            )
            Column(
                modifier = Modifier
                    .align(Alignment.BottomStart)
                    .padding(TaruSpacing.large),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconBadge(icon = Icons.Rounded.Person)
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
                    ) {
                        Text(
                            text = person.name,
                            style = MaterialTheme.typography.headlineLarge,
                            maxLines = 2,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            text = person.sortName?.takeIf { it.isNotBlank() } ?: "Person",
                            color = TaruTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                }
                FlowRow(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                ) {
                    StatusChip(text = "Person")
                    StatusChip(text = "$relatedCount related")
                    person.externalIds
                        .takeIf { it.isNotEmpty() }
                        ?.let { StatusChip(text = "${it.size} external") }
                }
            }
        }
    }
}

@Composable
private fun PersonFacts(person: PersonDto) {
    val sortName = person.sortName?.takeIf { it.isNotBlank() }
    if (sortName == null && person.externalIds.isEmpty()) return

    SectionHeader(title = "Identity")
    SurfaceCard {
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
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconBadge(icon = icon, compact = true)
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = value,
                color = TaruTextMuted,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
