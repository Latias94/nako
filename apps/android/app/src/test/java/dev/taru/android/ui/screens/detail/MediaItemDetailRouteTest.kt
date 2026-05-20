package dev.taru.android.ui.screens.detail

import dev.taru.android.browse.CanonicalMetadataDto
import dev.taru.android.browse.ItemCreditDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.ui.browse.BrowseFacetUiFamily
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class MediaItemDetailRouteTest {
    @Test
    fun stablePersonCreditsOpenPersonDetailTargets() {
        val rows = creditRelationshipRows(
            response = testDetailResponse(
                credits = listOf(
                    ItemCreditDto(
                        personId = "person-keeper",
                        role = JsonPrimitive("actor"),
                        character = "Keeper",
                    ),
                ),
            ),
        )

        assertEquals(1, rows.size)
        assertEquals(
            DetailRelationshipTarget.PersonDetail("person-keeper"),
            rows.single().target,
        )
    }

    @Test
    fun creditsWithoutStablePersonIdsStayExplicitPersonFacetGaps() {
        val rows = creditRelationshipRows(
            response = testDetailResponse(
                credits = listOf(
                    ItemCreditDto(
                        personId = "",
                        role = JsonPrimitive("director"),
                    ),
                ),
            ),
        )

        val target = rows.single().target
        assertTrue(target is DetailRelationshipTarget.Facet)
        target as DetailRelationshipTarget.Facet
        assertEquals(BrowseFacetUiFamily.Person, target.target.family)
        assertEquals(null, target.target.id)
    }
}

private fun testDetailResponse(
    credits: List<ItemCreditDto>,
): ItemDetailResponse =
    ItemDetailResponse(
        item = MediaItemDto(
            id = "night-harbor",
            kind = "movie",
            metadata = CanonicalMetadataDto(title = "Night Harbor"),
        ),
        credits = credits,
    )
