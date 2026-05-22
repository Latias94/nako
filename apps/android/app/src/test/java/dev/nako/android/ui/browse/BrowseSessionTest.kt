package dev.nako.android.ui.browse

import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class BrowseSessionTest {
    private val session = BrowseSession()

    @Test
    fun `selecting top level destination clears nested routes`() {
        val itemState = session.reduce(
            BrowseShellState(),
            BrowseAction.OpenItem("night-harbor"),
        )

        val searchState = session.reduce(
            itemState,
            BrowseAction.SelectDestination(NakoDestination.Search),
        )

        assertEquals(NakoDestination.Search, searchState.selectedDestination)
        assertEquals(NakoRoute.TopLevel, searchState.currentRoute)
        assertTrue(searchState.navigationVisible)
    }

    @Test
    fun `item facet and server profile routes open through actions`() {
        val facet = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Genre,
            label = "Mystery",
            id = "genre-mystery",
        )
        val itemState = session.reduce(
            BrowseShellState(),
            BrowseAction.OpenItem("night-harbor"),
        )
        val facetState = session.reduce(itemState, BrowseAction.OpenFacet(facet))

        assertEquals(NakoRoute.BrowseFacet(facet), facetState.currentRoute)
        assertFalse(facetState.navigationVisible)

        val backState = session.reduce(facetState, BrowseAction.Back)
        assertEquals(NakoRoute.ItemDetail("night-harbor"), backState.currentRoute)

        val serverProfileState = session.reduce(backState, BrowseAction.OpenServerProfile)
        assertEquals(NakoRoute.ServerProfile, serverProfileState.currentRoute)
    }

    @Test
    fun `person detail route opens through stable person action`() {
        val itemState = session.reduce(
            BrowseShellState(),
            BrowseAction.OpenItem("night-harbor"),
        )

        val personState = session.reduce(
            itemState,
            BrowseAction.OpenPersonDetail("person-1"),
        )

        assertEquals(NakoRoute.PersonDetail("person-1"), personState.currentRoute)
        assertFalse(personState.navigationVisible)
        assertEquals(NakoRoute.ItemDetail("night-harbor"), personState.navigation.navigateBack().currentRoute)
    }

    @Test
    fun `relationship index route opens as nested home route`() {
        val indexState = session.reduce(
            BrowseShellState(),
            BrowseAction.OpenRelationshipIndex(RelationshipIndexFamily.Genres),
        )

        assertEquals(NakoRoute.RelationshipIndex(RelationshipIndexFamily.Genres), indexState.currentRoute)
        assertFalse(indexState.navigationVisible)
        assertEquals(NakoRoute.TopLevel, indexState.navigation.navigateBack().currentRoute)
    }

    @Test
    fun `library detail route preserves libraries top level owner`() {
        val librariesState = session.reduce(
            BrowseShellState(),
            BrowseAction.SelectDestination(NakoDestination.Libraries),
        )
        val detailState = session.reduce(
            librariesState,
            BrowseAction.OpenLibraryDetail("library-movies"),
        )
        val backState = session.reduce(detailState, BrowseAction.Back)

        assertEquals(NakoDestination.Libraries, backState.selectedDestination)
        assertEquals(NakoRoute.TopLevel, backState.currentRoute)
        assertTrue(backState.navigationVisible)
    }

    @Test
    fun `initial navigation can be restored into session state`() {
        val initialNavigation = NakoBrowseNavigationState
            .root()
            .selectDestination(NakoDestination.Settings)
            .open(NakoRoute.ServerProfile)
        val initialState = BrowseShellState(
            navigation = initialNavigation,
        )

        assertEquals(NakoDestination.Settings, initialState.selectedDestination)
        assertEquals(NakoRoute.ServerProfile, initialState.currentRoute)

        val backState = session.reduce(initialState, BrowseAction.Back)
        assertEquals(NakoRoute.TopLevel, backState.currentRoute)
        assertEquals(NakoDestination.Settings, backState.selectedDestination)
    }

    @Test
    fun `player route opens through session action and remains transient in save payload`() {
        val launch = testPlaybackLaunch()
        val detailState = session.reduce(
            BrowseShellState(),
            BrowseAction.OpenItem("night-harbor"),
        )

        val playerState = session.reduce(detailState, BrowseAction.OpenPlayer(launch))
        val restored = restoreBrowseShellState(playerState.toSaveablePayload())

        assertEquals(NakoRoute.Player(launch), playerState.currentRoute)
        assertEquals(NakoRoute.ItemDetail("night-harbor"), restored.currentRoute)
        assertFalse(playerState.toSaveablePayload().contains("secret-token"))
    }
}

private fun testPlaybackLaunch() =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = PlaybackRequestTarget(
            request = PlaybackRequestDescriptor(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream",
            ),
        ),
        serverProfileId = "server-1",
        mediaItemId = "night-harbor",
        sourceId = "source-1",
        playbackMode = ClientPlaybackMode.DirectPlay,
    )
