package dev.taru.android.ui.browse

import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.playbackLaunchRequest
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
            BrowseAction.SelectDestination(TaruDestination.Search),
        )

        assertEquals(TaruDestination.Search, searchState.selectedDestination)
        assertEquals(TaruRoute.TopLevel, searchState.currentRoute)
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

        assertEquals(TaruRoute.BrowseFacet(facet), facetState.currentRoute)
        assertFalse(facetState.navigationVisible)

        val backState = session.reduce(facetState, BrowseAction.Back)
        assertEquals(TaruRoute.ItemDetail("night-harbor"), backState.currentRoute)

        val serverProfileState = session.reduce(backState, BrowseAction.OpenServerProfile)
        assertEquals(TaruRoute.ServerProfile, serverProfileState.currentRoute)
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

        assertEquals(TaruRoute.PersonDetail("person-1"), personState.currentRoute)
        assertFalse(personState.navigationVisible)
        assertEquals(TaruRoute.ItemDetail("night-harbor"), personState.navigation.navigateBack().currentRoute)
    }


    @Test
    fun `library detail route preserves libraries top level owner`() {
        val librariesState = session.reduce(
            BrowseShellState(),
            BrowseAction.SelectDestination(TaruDestination.Libraries),
        )
        val detailState = session.reduce(
            librariesState,
            BrowseAction.OpenLibraryDetail("library-movies"),
        )
        val backState = session.reduce(detailState, BrowseAction.Back)

        assertEquals(TaruDestination.Libraries, backState.selectedDestination)
        assertEquals(TaruRoute.TopLevel, backState.currentRoute)
        assertTrue(backState.navigationVisible)
    }

    @Test
    fun `initial navigation can be restored into session state`() {
        val initialNavigation = TaruBrowseNavigationState
            .root()
            .selectDestination(TaruDestination.Settings)
            .open(TaruRoute.ServerProfile)
        val initialState = BrowseShellState(
            navigation = initialNavigation,
        )

        assertEquals(TaruDestination.Settings, initialState.selectedDestination)
        assertEquals(TaruRoute.ServerProfile, initialState.currentRoute)

        val backState = session.reduce(initialState, BrowseAction.Back)
        assertEquals(TaruRoute.TopLevel, backState.currentRoute)
        assertEquals(TaruDestination.Settings, backState.selectedDestination)
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

        assertEquals(TaruRoute.Player(launch), playerState.currentRoute)
        assertEquals(TaruRoute.ItemDetail("night-harbor"), restored.currentRoute)
        assertFalse(playerState.toSaveablePayload().contains("secret-token"))
    }
}

private fun testPlaybackLaunch() =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = PlaybackRequestTarget(
            request = TaruHttpRequest(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream",
                headers = mapOf("Authorization" to "Bearer secret-token"),
            ),
            safeRequest = SafeRequestPreview(
                method = "GET",
                url = "http://127.0.0.1:3018/sources/source-1/stream",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        ),
        serverProfileId = "server-1",
        mediaItemId = "night-harbor",
        sourceId = "source-1",
        playbackMode = ClientPlaybackMode.DirectPlay,
    )
