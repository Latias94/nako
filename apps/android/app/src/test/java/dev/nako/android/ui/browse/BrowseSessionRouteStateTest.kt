package dev.nako.android.ui.browse

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class BrowseSessionRouteStateTest {
    @Test
    fun `route state policy prepares only active route family and invalidates stale requests`() {
        val policy = BrowseRouteStatePolicy()
        val detailState = BrowseShellState(
            navigation = NakoBrowseNavigationState.root().open(NakoRoute.ItemDetail("night-harbor")),
            libraryDetailState = LibraryDetailUiState.Content(testLibrarySourcesState("library-old")),
            personDetailState = PersonDetailUiState.Content(
                response = testPersonResponse("person-old"),
                relatedItems = testFacetItems("person-old"),
            ),
            relationshipIndexState = testRelationshipContent(RelationshipIndexFamily.Tags),
            facetState = FacetUiState.Content(testFacetItems("tag-old")),
        )

        val prepared = policy.prepare(
            previous = BrowseShellState(),
            next = detailState,
        )

        assertEquals(ItemDetailUiState.Loading, prepared.detailState)
        assertEquals(LibraryDetailUiState.Idle, prepared.libraryDetailState)
        assertEquals(PersonDetailUiState.Idle, prepared.personDetailState)
        assertEquals(RelationshipIndexUiState.Idle, prepared.relationshipIndexState)
        assertEquals(FacetUiState.Idle, prepared.facetState)
        assertEquals(null, prepared.selectedSourceId)
        assertEquals(SourceProbeUiState.Idle, prepared.sourceProbeState)
        assertEquals(PlaybackSelectionUiState.Idle, prepared.playbackState)
        assertFalse(policy.acceptsItemDetail(0))
        assertTrue(policy.acceptsItemDetail(1))
        assertFalse(policy.acceptsSourceProbe(0))
        assertTrue(policy.acceptsSourceProbe(1))
    }

    @Test
    fun `unsupported facet route prepares api gap without starting public route loading`() {
        val target = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Studio,
            label = "Studio",
            id = "studio-1",
        )
        val policy = BrowseRouteStatePolicy()

        val prepared = policy.prepare(
            previous = BrowseShellState(),
            next = BrowseShellState(
                navigation = NakoBrowseNavigationState.root().open(NakoRoute.BrowseFacet(target)),
            ),
        )

        assertTrue(prepared.facetState is FacetUiState.ApiGap)
        assertEquals(ItemDetailUiState.Idle, prepared.detailState)
        assertTrue(policy.acceptsFacet(1))
    }

    @Test
    fun `unsupported facet api gap uses viewer language`() {
        val state = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Person,
            label = "Actor",
            id = null,
        ).apiGapState()

        assertEquals("This list is not available yet", state.title)
        assertFalse(state.body.contains("current response"))
        assertFalse(state.body.contains("stable id"))
        assertFalse(state.body.contains("API"))
        assertTrue(state.body.contains("server"))
    }

    @Test
    fun `player route remains transient and does not clear previous detail state`() {
        val policy = BrowseRouteStatePolicy()
        val launch = testPlaybackLaunchFixture()
        val current = BrowseShellState(
            navigation = NakoBrowseNavigationState.root().open(NakoRoute.ItemDetail("night-harbor")),
            detailState = ItemDetailUiState.Content(testDetailResponse("night-harbor", listOf("source-1"))),
            selectedSourceId = "source-1",
        )
        val next = current.copy(
            navigation = current.navigation.open(NakoRoute.Player(launch)),
        )

        val prepared = policy.prepare(
            previous = current,
            next = next,
        )

        assertEquals(NakoRoute.Player(launch), prepared.currentRoute)
        assertTrue(prepared.detailState is ItemDetailUiState.Content)
        assertEquals("source-1", prepared.selectedSourceId)
        assertTrue(policy.acceptsItemDetail(0))
    }
}
