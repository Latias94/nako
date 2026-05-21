package dev.taru.android.ui.browse

import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestDescriptor
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TaruRouteStackTest {
    @Test
    fun `root stack starts at top level and ignores root pop`() {
        val stack = TaruRouteStack.root()

        assertEquals(TaruRoute.TopLevel, stack.current)
        assertTrue(stack.isAtRoot)
        assertFalse(stack.canPop)
        assertEquals(stack, stack.pop())
    }

    @Test
    fun `facet opened from detail returns to the original detail`() {
        val detail = TaruRoute.ItemDetail("night-harbor")
        val facet = TaruRoute.BrowseFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Genre,
                label = "Mystery",
                id = "genre-mystery",
            ),
        )

        val stack = TaruRouteStack
            .root()
            .push(detail)
            .push(facet)

        assertEquals(facet, stack.current)
        assertEquals(detail, stack.pop().current)
    }

    @Test
    fun `player opened from detail returns to the original detail`() {
        val detail = TaruRoute.ItemDetail("night-harbor")
        val player = TaruRoute.Player(testPlaybackLaunch())

        val stack = TaruRouteStack
            .root()
            .push(detail)
            .push(player)

        assertEquals(player, stack.current)
        assertEquals(detail, stack.pop().current)
    }

    @Test
    fun `server profile returns to the top level owner`() {
        val stack = TaruRouteStack
            .root()
            .push(TaruRoute.ServerProfile)
            .pop()

        assertEquals(TaruRoute.TopLevel, stack.current)
        assertTrue(stack.isAtRoot)
    }

    @Test
    fun `library detail opened from libraries returns to top level libraries`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .selectDestination(TaruDestination.Libraries)
            .open(TaruRoute.LibraryDetail("library-movies"))
            .navigateBack()

        assertEquals(TaruDestination.Libraries, navigation.selectedDestination)
        assertEquals(TaruRoute.TopLevel, navigation.currentRoute)
        assertTrue(navigation.navigationVisible)
    }

    @Test
    fun `item opened from facet returns to the facet list`() {
        val originalDetail = TaruRoute.ItemDetail("night-harbor")
        val facet = TaruRoute.BrowseFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Tag,
                label = "Lighthouse",
                id = "tag-lighthouse",
            ),
        )
        val relatedDetail = TaruRoute.ItemDetail("related-night")

        val stack = TaruRouteStack
            .root()
            .push(originalDetail)
            .push(facet)
            .push(relatedDetail)

        assertEquals(facet, stack.pop().current)
        assertEquals(originalDetail, stack.pop().pop().current)
    }

    @Test
    fun `clear to root removes nested routes`() {
        val stack = TaruRouteStack
            .root()
            .push(TaruRoute.ItemDetail("night-harbor"))
            .push(TaruRoute.ServerProfile)
            .clearToRoot()

        assertEquals(TaruRoute.TopLevel, stack.current)
        assertTrue(stack.isAtRoot)
    }

    @Test
    fun `selecting a top level destination clears nested route stack`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .open(TaruRoute.ItemDetail("night-harbor"))
            .selectDestination(TaruDestination.Search)

        assertEquals(TaruDestination.Search, navigation.selectedDestination)
        assertEquals(TaruRoute.TopLevel, navigation.currentRoute)
        assertTrue(navigation.navigationVisible)
    }

    @Test
    fun `server profile opened from settings returns to settings root`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .selectDestination(TaruDestination.Settings)
            .open(TaruRoute.ServerProfile)
            .navigateBack()

        assertEquals(TaruDestination.Settings, navigation.selectedDestination)
        assertEquals(TaruRoute.TopLevel, navigation.currentRoute)
        assertTrue(navigation.navigationVisible)
    }

    @Test(expected = IllegalArgumentException::class)
    fun `top level cannot be pushed as a nested route`() {
        TaruRouteStack.root().push(TaruRoute.TopLevel)
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
