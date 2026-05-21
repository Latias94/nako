package dev.taru.android.ui.browse

import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestDescriptor
import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.player.playbackLaunchRequest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TaruBrowseNavigationStateSaverTest {
    @Test
    fun `safe nested browse route restores with selected top level destination`() {
        val detail = TaruRoute.ItemDetail("night-harbor")
        val facet = TaruRoute.BrowseFacet(
            BrowseFacetTarget(
                family = BrowseFacetUiFamily.Genre,
                label = "Mystery",
                id = "genre-mystery",
            ),
        )
        val navigation = TaruBrowseNavigationState
            .root()
            .selectDestination(TaruDestination.Libraries)
            .open(detail)
            .open(facet)

        val restored = restoreTaruBrowseNavigationState(
            navigation.toSaveablePayload(),
        )

        assertEquals(TaruDestination.Libraries, restored.selectedDestination)
        assertEquals(facet, restored.currentRoute)
        assertFalse(restored.navigationVisible)
        assertEquals(detail, restored.navigateBack().currentRoute)
    }

    @Test
    fun `server profile restores to settings owned nested route`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .selectDestination(TaruDestination.Settings)
            .open(TaruRoute.ServerProfile)

        val restored = restoreTaruBrowseNavigationState(
            navigation.toSaveablePayload(),
        )

        assertEquals(TaruDestination.Settings, restored.selectedDestination)
        assertEquals(TaruRoute.ServerProfile, restored.currentRoute)
        assertEquals(TaruDestination.Settings, restored.navigateBack().selectedDestination)
        assertEquals(TaruRoute.TopLevel, restored.navigateBack().currentRoute)
    }

    @Test
    fun `library detail route restores under libraries destination`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .selectDestination(TaruDestination.Libraries)
            .open(TaruRoute.LibraryDetail("library-movies"))

        val restored = restoreTaruBrowseNavigationState(
            navigation.toSaveablePayload(),
        )

        assertEquals(TaruDestination.Libraries, restored.selectedDestination)
        assertEquals(TaruRoute.LibraryDetail("library-movies"), restored.currentRoute)
        assertFalse(restored.navigationVisible)
    }

    @Test
    fun `person detail route restores without leaking unsafe data`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .open(TaruRoute.ItemDetail("night-harbor"))
            .open(TaruRoute.PersonDetail("person 1"))

        val payload = navigation.toSaveablePayload()
        val restored = restoreTaruBrowseNavigationState(payload)

        assertEquals(TaruRoute.PersonDetail("person 1"), restored.currentRoute)
        assertEquals(TaruRoute.ItemDetail("night-harbor"), restored.navigateBack().currentRoute)
        assertFalse(restored.navigationVisible)
        assertFalse(payload.contains("Bearer"))
    }

    @Test
    fun `relationship index route restores as safe nested route`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .open(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Genres))

        val payload = navigation.toSaveablePayload()
        val restored = restoreTaruBrowseNavigationState(payload)

        assertEquals(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Genres), restored.currentRoute)
        assertFalse(restored.navigationVisible)
        assertFalse(payload.contains("Bearer"))
    }

    @Test
    fun `tag relationship index route restores as safe nested route`() {
        val navigation = TaruBrowseNavigationState
            .root()
            .open(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Tags))

        val payload = navigation.toSaveablePayload()
        val restored = restoreTaruBrowseNavigationState(payload)

        assertEquals(TaruRoute.RelationshipIndex(RelationshipIndexFamily.Tags), restored.currentRoute)
        assertFalse(restored.navigationVisible)
        assertFalse(payload.contains("Bearer"))
    }

    @Test
    fun `player route is transient and restores to previous safe detail`() {
        val detail = TaruRoute.ItemDetail("night-harbor")
        val navigation = TaruBrowseNavigationState
            .root()
            .open(detail)
            .open(TaruRoute.Player(testPlaybackLaunch()))

        val payload = navigation.toSaveablePayload()
        val restored = restoreTaruBrowseNavigationState(payload)

        assertEquals(detail, restored.currentRoute)
        assertFalse(payload.contains("secret-token"))
        assertFalse(payload.contains("Bearer"))
        assertFalse(payload.contains("/sources/source-1/stream"))
    }

    @Test
    fun `invalid payload restores to safe root state`() {
        val restored = restoreTaruBrowseNavigationState("{not-json")

        assertEquals(TaruBrowseNavigationState.root(), restored)
        assertTrue(restored.navigationVisible)
    }

    @Test
    fun `unknown route and destination values restore safely`() {
        val restored = restoreTaruBrowseNavigationState(
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

        assertEquals(TaruDestination.Home, restored.selectedDestination)
        assertEquals(TaruRoute.ItemDetail("night-harbor"), restored.currentRoute)
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
