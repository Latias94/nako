package dev.nako.android.ui.browse

import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NakoBrowseNavigationStateSaverTest {
    @Test
    fun `safe nested browse route restores with selected top level destination`() {
        val detail = NakoRoute.ItemDetail("night-harbor")
        val facet = NakoRoute.BrowseFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Genre,
                label = "Mystery",
                id = "genre-mystery",
            ),
        )
        val navigation = NakoBrowseNavigationState
            .root()
            .selectDestination(NakoDestination.Libraries)
            .open(detail)
            .open(facet)

        val restored = restoreNakoBrowseNavigationState(
            navigation.toSaveablePayload(),
        )

        assertEquals(NakoDestination.Libraries, restored.selectedDestination)
        assertEquals(facet, restored.currentRoute)
        assertFalse(restored.navigationVisible)
        assertEquals(detail, restored.navigateBack().currentRoute)
    }

    @Test
    fun `server profile restores to settings owned nested route`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .selectDestination(NakoDestination.Settings)
            .open(NakoRoute.ServerProfile)

        val restored = restoreNakoBrowseNavigationState(
            navigation.toSaveablePayload(),
        )

        assertEquals(NakoDestination.Settings, restored.selectedDestination)
        assertEquals(NakoRoute.ServerProfile, restored.currentRoute)
        assertEquals(NakoDestination.Settings, restored.navigateBack().selectedDestination)
        assertEquals(NakoRoute.TopLevel, restored.navigateBack().currentRoute)
    }

    @Test
    fun `library detail route restores under libraries destination`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .selectDestination(NakoDestination.Libraries)
            .open(NakoRoute.LibraryDetail("library-movies"))

        val restored = restoreNakoBrowseNavigationState(
            navigation.toSaveablePayload(),
        )

        assertEquals(NakoDestination.Libraries, restored.selectedDestination)
        assertEquals(NakoRoute.LibraryDetail("library-movies"), restored.currentRoute)
        assertFalse(restored.navigationVisible)
    }

    @Test
    fun `person detail route restores without leaking unsafe data`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .open(NakoRoute.ItemDetail("night-harbor"))
            .open(NakoRoute.PersonDetail("person 1"))

        val payload = navigation.toSaveablePayload()
        val restored = restoreNakoBrowseNavigationState(payload)

        assertEquals(NakoRoute.PersonDetail("person 1"), restored.currentRoute)
        assertEquals(NakoRoute.ItemDetail("night-harbor"), restored.navigateBack().currentRoute)
        assertFalse(restored.navigationVisible)
        assertFalse(payload.contains("Bearer"))
    }

    @Test
    fun `relationship index route restores as safe nested route`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .open(NakoRoute.RelationshipIndex(RelationshipIndexFamily.Genres))

        val payload = navigation.toSaveablePayload()
        val restored = restoreNakoBrowseNavigationState(payload)

        assertEquals(NakoRoute.RelationshipIndex(RelationshipIndexFamily.Genres), restored.currentRoute)
        assertFalse(restored.navigationVisible)
        assertFalse(payload.contains("Bearer"))
    }

    @Test
    fun `tag relationship index route restores as safe nested route`() {
        val navigation = NakoBrowseNavigationState
            .root()
            .open(NakoRoute.RelationshipIndex(RelationshipIndexFamily.Tags))

        val payload = navigation.toSaveablePayload()
        val restored = restoreNakoBrowseNavigationState(payload)

        assertEquals(NakoRoute.RelationshipIndex(RelationshipIndexFamily.Tags), restored.currentRoute)
        assertFalse(restored.navigationVisible)
        assertFalse(payload.contains("Bearer"))
    }

    @Test
    fun `player route is transient and restores to previous safe detail`() {
        val detail = NakoRoute.ItemDetail("night-harbor")
        val navigation = NakoBrowseNavigationState
            .root()
            .open(detail)
            .open(NakoRoute.Player(testPlaybackLaunch()))

        val payload = navigation.toSaveablePayload()
        val restored = restoreNakoBrowseNavigationState(payload)

        assertEquals(detail, restored.currentRoute)
        assertFalse(payload.contains("secret-token"))
        assertFalse(payload.contains("Bearer"))
        assertFalse(payload.contains("/sources/source-1/stream"))
    }

    @Test
    fun `invalid payload restores to safe root state`() {
        val restored = restoreNakoBrowseNavigationState("{not-json")

        assertEquals(NakoBrowseNavigationState.root(), restored)
        assertTrue(restored.navigationVisible)
    }

    @Test
    fun `unknown route and destination values restore safely`() {
        val restored = restoreNakoBrowseNavigationState(
            """
            {
              "version": 1,
              "selected_destination": "Missing",
              "routes": [
                {"type": "top_level"},
                {"type": "future_route", "item_id": "unsafe"},
                {"type": "item_detail", "item_id": "night-harbor"}
              ]
            }
            """.trimIndent(),
        )

        assertEquals(NakoDestination.Home, restored.selectedDestination)
        assertEquals(NakoRoute.ItemDetail("night-harbor"), restored.currentRoute)
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
