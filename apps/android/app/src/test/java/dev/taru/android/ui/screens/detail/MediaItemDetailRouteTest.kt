package dev.taru.android.ui.screens.detail

import dev.taru.android.browse.CanonicalMetadataDto
import dev.taru.android.browse.ItemCreditDto
import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.ui.browse.BrowseFacetUiFamily
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
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
                        role = "actor",
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
    fun detailCopyUsesViewerLanguageForUnavailableRelationships() {
        assertEquals(
            "More from this collection needs server support.",
            relatedCollectionsSubtitle(collectionCount = 0),
        )
        assertEquals(
            "2 collection link(s)",
            relatedCollectionsSubtitle(collectionCount = 2),
        )
        assertEquals(
            "Series and extras browsing needs server support.",
            hierarchySubtitle(
                MediaItemDto(
                    id = "night-harbor",
                    kind = "movie",
                    metadata = CanonicalMetadataDto(title = "Night Harbor"),
                ),
            ),
        )
    }

    @Test
    fun creditCopyUsesViewerLanguageInsteadOfApiTerms() {
        val rows = creditRelationshipRows(
            response = testDetailResponse(
                credits = listOf(
                    ItemCreditDto(
                        personId = "person-keeper",
                        role = "actor",
                        character = "Keeper",
                    ),
                    ItemCreditDto(
                        personId = "",
                        role = "director",
                    ),
                ),
            ),
        )

        val subtitles = rows.joinToString(" ") { it.subtitle }
        assertTrue(subtitles.contains("related titles"))
        assertFalse(subtitles.contains("Media Items"))
        assertFalse(subtitles.contains("response"))
        assertFalse(subtitles.contains("Public Client API"))
    }

    @Test
    fun creditsWithoutStablePersonIdsStayExplicitPersonFacetGaps() {
        val rows = creditRelationshipRows(
            response = testDetailResponse(
                credits = listOf(
                    ItemCreditDto(
                        personId = "",
                        role = "director",
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
