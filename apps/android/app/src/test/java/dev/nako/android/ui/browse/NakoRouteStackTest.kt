package dev.nako.android.ui.browse

import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NakoRouteStackTest {
    @Test
    fun `root stack starts at top level and ignores root pop`() {
        val stack = NakoRouteStack.root()

        assertEquals(NakoRoute.TopLevel, stack.current)
        assertTrue(stack.isAtRoot)
        assertFalse(stack.canPop)
        assertEquals(stack, stack.pop())
    }

    @Test
    fun `facet opened from detail returns to the original detail`() {
        val detail = NakoRoute.ItemDetail("night-harbor")
        val facet = NakoRoute.BrowseFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Genre,
                label = "Mystery",
                id = "genre-mystery",
            ),
        )

        val stack = NakoRouteStack
            .root()
            .push(detail)
            .push(facet)

        assertEquals(facet, stack.current)
        assertEquals(detail, stack.pop().current)
    }

    @Test
    fun `player opened from detail returns to the original detail`() {
        val detail = NakoRoute.ItemDetail("night-harbor")
        val player = NakoRoute.Player(testPlaybackLaunch())

        val stack = NakoRouteStack
            .root()
            .push(detail)
            .push(player)

        assertEquals(player, stack.current)
        assertEquals(detail, stack.pop().current)
    }

    @Test
    fun `server profile returns to the top level owner`() {
        val stack = NakoRouteStack
            .root()
            .push(NakoRoute.ServerProfile)
            .pop()

        assertEquals(NakoRoute.TopLevel, stack.current)
        assertTrue(stack.isAtRoot)
    }

    @Test
    fun `library detail opened from libraries returns to top level libraries`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .selectDestination(NakoDestination.Libraries)
            .open(NakoRoute.LibraryDetail("library-movies"))
            .navigateBack()

        assertEquals(NakoDestination.Libraries, navigation.selectedDestination)
        assertEquals(NakoRoute.TopLevel, navigation.currentRoute)
        assertTrue(navigation.navigationVisible)
    }

    @Test
    fun `item opened from facet returns to the facet list`() {
        val originalDetail = NakoRoute.ItemDetail("night-harbor")
        val facet = NakoRoute.BrowseFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Tag,
                label = "Lighthouse",
                id = "tag-lighthouse",
            ),
        )
        val relatedDetail = NakoRoute.ItemDetail("related-night")

        val stack = NakoRouteStack
            .root()
            .push(originalDetail)
            .push(facet)
            .push(relatedDetail)

        assertEquals(facet, stack.pop().current)
        assertEquals(originalDetail, stack.pop().pop().current)
    }

    @Test
    fun `clear to root removes nested routes`() {
        val stack = NakoRouteStack
            .root()
            .push(NakoRoute.ItemDetail("night-harbor"))
            .push(NakoRoute.ServerProfile)
            .clearToRoot()

        assertEquals(NakoRoute.TopLevel, stack.current)
        assertTrue(stack.isAtRoot)
    }

    @Test
    fun `selecting a top level destination clears nested route stack`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .open(NakoRoute.ItemDetail("night-harbor"))
            .selectDestination(NakoDestination.Search)

        assertEquals(NakoDestination.Search, navigation.selectedDestination)
        assertEquals(NakoRoute.TopLevel, navigation.currentRoute)
        assertTrue(navigation.navigationVisible)
    }

    @Test
    fun `server profile opened from settings returns to settings root`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .selectDestination(NakoDestination.Settings)
            .open(NakoRoute.ServerProfile)
            .navigateBack()

        assertEquals(NakoDestination.Settings, navigation.selectedDestination)
        assertEquals(NakoRoute.TopLevel, navigation.currentRoute)
        assertTrue(navigation.navigationVisible)
    }

    @Test(expected = IllegalArgumentException::class)
    fun `top level cannot be pushed as a nested route`() {
        NakoRouteStack.root().push(NakoRoute.TopLevel)
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
