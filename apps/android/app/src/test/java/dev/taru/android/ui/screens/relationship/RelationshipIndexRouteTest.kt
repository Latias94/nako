package dev.taru.android.ui.screens.relationship

import dev.taru.android.browse.PageInfo
import dev.taru.android.ui.browse.BrowseFacetTarget
import dev.taru.android.ui.browse.BrowseFacetUiFamily
import dev.taru.android.ui.browse.RelationshipIndexFamily
import dev.taru.android.ui.browse.RelationshipIndexRow
import dev.taru.android.ui.browse.RelationshipIndexUiState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test

class RelationshipIndexRouteTest {
    @Test
    fun presentationKeepsStableGenreTargetsAndCounts() {
        val target = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Genre,
            label = "Mystery",
            id = "genre-mystery",
        )
        val presentation = relationshipIndexPresentation(
            RelationshipIndexUiState.Content(
                family = RelationshipIndexFamily.Genres,
                rows = listOf(
                    RelationshipIndexRow(
                        title = "Mystery",
                        subtitle = "Genre",
                        target = target,
                    ),
                ),
                page = PageInfo(limit = 50, offset = 0, returned = 12),
            ),
        )

        assertEquals("Genres", presentation.title)
        assertEquals("Server Genres Index", presentation.subtitle)
        assertEquals("1 visible", presentation.resultLabel)
        assertEquals("12 returned", presentation.returnedLabel)
        assertEquals(BrowseFacetUiFamily.Genre, presentation.rows.single().target.family)
        assertEquals("genre-mystery", presentation.rows.single().target.id)
        assertFalse(presentation.toString().contains("Bearer"))
    }
}
