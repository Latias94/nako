package dev.taru.android.ui.screens.detail

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Person
import androidx.compose.material.icons.rounded.TheaterComedy
import androidx.compose.ui.graphics.vector.ImageVector
import dev.taru.android.browse.ItemCreditDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.ui.browse.BrowseFacetTarget
import dev.taru.android.ui.browse.BrowseFacetUiFamily
import dev.taru.android.ui.browse.itemSecondaryText

internal data class DetailRelationshipRow(
    val title: String,
    val subtitle: String,
    val icon: ImageVector,
    val target: DetailRelationshipTarget,
)

internal sealed interface DetailRelationshipTarget {
    data class Facet(val target: BrowseFacetTarget) : DetailRelationshipTarget
    data class PersonDetail(val personId: String) : DetailRelationshipTarget
}

internal fun buildMetadataTargets(response: ItemDetailResponse): List<BrowseFacetTarget> {
    val item = response.item
    return buildList {
        item.metadata.genres.take(4).forEachIndexed { index, label ->
            add(
                BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Genre,
                    label = label,
                    id = response.genres.getOrNull(index)?.genreId,
                ),
            )
        }
        item.metadata.tags.take(4).forEachIndexed { index, label ->
            add(
                BrowseFacetTarget(
                    family = BrowseFacetUiFamily.Tag,
                    label = label,
                    id = response.tags.getOrNull(index)?.tagId,
                ),
            )
        }
        item.metadata.releaseDate?.take(4)?.let { year ->
            add(BrowseFacetTarget(BrowseFacetUiFamily.Year, year))
        }
        add(BrowseFacetTarget(BrowseFacetUiFamily.ItemKind, item.kind))
    }
}

internal fun creditRelationshipRows(response: ItemDetailResponse): List<DetailRelationshipRow> {
    val rows = response.credits.take(4).mapIndexed { index, credit ->
        val title = creditTitle(index, credit)
        val personId = credit.personId.takeIf { it.isNotBlank() }
        DetailRelationshipRow(
            title = title,
            subtitle = if (personId == null) {
                "Person link unavailable for this credit."
            } else {
                "Open this person and related titles."
            },
            icon = Icons.Rounded.Person,
            target = personId
                ?.let(DetailRelationshipTarget::PersonDetail)
                ?: DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(
                        family = BrowseFacetUiFamily.Person,
                        label = title,
                    ),
                ),
        )
    }
    return rows.ifEmpty {
        listOf(
            DetailRelationshipRow(
                title = "Cast",
                subtitle = "Credit names are not available for this item yet.",
                icon = Icons.Rounded.Person,
                target = DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(BrowseFacetUiFamily.Person, "Cast"),
                ),
            ),
            DetailRelationshipRow(
                title = "Director",
                subtitle = "Role-specific browsing is not available yet.",
                icon = Icons.Rounded.TheaterComedy,
                target = DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(BrowseFacetUiFamily.Person, "Director"),
                ),
            ),
            DetailRelationshipRow(
                title = "Writer",
                subtitle = "Role-specific browsing is not available yet.",
                icon = Icons.Rounded.Info,
                target = DetailRelationshipTarget.Facet(
                    BrowseFacetTarget(BrowseFacetUiFamily.Person, "Writer"),
                ),
            ),
        )
    }
}

internal fun detailFactLabels(item: MediaItemDto): List<String> =
    buildList {
        itemSecondaryText(item).takeIf { it.isNotBlank() }?.let { add(it) }
        item.metadata.ratings.firstOrNull()?.let { add(it.value) }
        item.metadata.originalTitle?.takeIf { it.isNotBlank() && it != item.metadata.title }?.let {
            add("Original title available")
        }
        item.parentId?.takeIf { it.isNotBlank() }?.let { add("In hierarchy") }
    }.ifEmpty { listOf(item.kind) }

internal fun relatedCollectionsSubtitle(collectionCount: Int): String =
    if (collectionCount <= 0) {
        "More from this collection needs server support."
    } else {
        "$collectionCount collection link(s)"
    }

internal fun hierarchySubtitle(item: MediaItemDto): String =
    if (item.parentId.isNullOrBlank()) {
        "Series and extras browsing needs server support."
    } else {
        "This item belongs to a hierarchy, but browsing it needs server support."
    }

private fun creditTitle(index: Int, credit: ItemCreditDto): String {
    val role = credit.role
        ?.replace('_', ' ')
        ?.takeIf { it.isNotBlank() }
    val character = credit.character?.takeIf { it.isNotBlank() }
    return listOfNotNull(
        role?.replaceFirstChar { it.uppercase() },
        character?.let { "as $it" },
    ).joinToString(" / ").ifBlank { "Credit ${index + 1}" }
}
