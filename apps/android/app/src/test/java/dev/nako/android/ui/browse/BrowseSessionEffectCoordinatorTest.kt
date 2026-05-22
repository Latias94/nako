package dev.nako.android.ui.browse

import kotlinx.coroutines.Job
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Test

class BrowseSessionEffectCoordinatorTest {
    @Test
    fun `start publishes initial state and emits exactly one initial route load intent`() {
        val sink = RecordingBrowseSessionEffectSink()
        val coordinator = BrowseSessionEffectCoordinator(
            initialState = sink.currentState,
            sink = sink,
        )

        val job = coordinator.start()
        coordinator.accept(sink.currentState)

        assertNull(job)
        assertEquals(listOf(NakoRoute.TopLevel), sink.loadedRoutes)
        assertEquals(3, sink.savedStates.size)
        assertEquals(NakoRoute.TopLevel, sink.savedStates.last().currentRoute)
    }

    @Test
    fun `state route changes are translated into load intents without duplicate loads`() {
        val sink = RecordingBrowseSessionEffectSink()
        val coordinator = BrowseSessionEffectCoordinator(
            initialState = sink.currentState,
            sink = sink,
        )

        coordinator.start()
        sink.currentState = BrowseShellState(
            navigation = NakoBrowseNavigationState.root().open(NakoRoute.ItemDetail("night-harbor")),
        )
        val routeLoad = coordinator.accept(sink.currentState)
        coordinator.accept(sink.currentState)

        assertNull(routeLoad)
        assertEquals(
            listOf(
                NakoRoute.TopLevel,
                NakoRoute.ItemDetail("night-harbor"),
            ),
            sink.loadedRoutes,
        )
        assertEquals(NakoRoute.ItemDetail("night-harbor"), sink.savedStates.last().currentRoute)
    }

    @Test
    fun `after user action publishes freshest state and returns route load job`() {
        val routeJob = Job()
        val itemRoute = NakoRoute.ItemDetail("night-harbor")
        val sink = RecordingBrowseSessionEffectSink(routeJobs = mapOf(itemRoute to routeJob))
        val coordinator = BrowseSessionEffectCoordinator(
            initialState = sink.currentState,
            sink = sink,
        )

        coordinator.start()
        sink.currentState = BrowseShellState(
            navigation = NakoBrowseNavigationState.root().open(itemRoute),
            detailState = ItemDetailUiState.Loading,
        )
        val returnedJob = coordinator.afterUserAction()

        assertSame(routeJob, returnedJob)
        assertEquals(listOf(NakoRoute.TopLevel, itemRoute), sink.loadedRoutes)
        assertEquals(ItemDetailUiState.Loading, sink.savedStates.last().detailState)
    }

    @Test
    fun `player route is still saveable published while load intent clears no route state`() {
        val launch = testPlaybackLaunchFixture()
        val playerRoute = NakoRoute.Player(launch)
        val detailContent = ItemDetailUiState.Content(testDetailResponse("night-harbor", listOf("source-a")))
        val sink = RecordingBrowseSessionEffectSink()
        val coordinator = BrowseSessionEffectCoordinator(
            initialState = sink.currentState,
            sink = sink,
        )

        coordinator.start()
        sink.currentState = BrowseShellState(
            navigation = NakoBrowseNavigationState
                .root()
                .open(NakoRoute.ItemDetail("night-harbor"))
                .open(playerRoute),
            detailState = detailContent,
            selectedSourceId = "source-a",
        )
        coordinator.accept(sink.currentState)

        assertEquals(listOf(NakoRoute.TopLevel, playerRoute), sink.loadedRoutes)
        assertEquals(playerRoute, sink.savedStates.last().currentRoute)
        assertEquals(detailContent, sink.savedStates.last().detailState)
        assertEquals("source-a", sink.savedStates.last().selectedSourceId)
        assertEquals(
            NakoRoute.ItemDetail("night-harbor"),
            restoreBrowseShellState(sink.savedStates.last().toSaveablePayload()).currentRoute,
        )
    }
}

private class RecordingBrowseSessionEffectSink(
    override var currentState: BrowseShellState = BrowseShellState(),
    private val routeJobs: Map<NakoRoute, Job?> = emptyMap(),
) : BrowseSessionEffectSink {
    val savedStates: MutableList<BrowseShellState> = mutableListOf()
    val loadedRoutes: MutableList<NakoRoute> = mutableListOf()

    override fun publishSaveableState(state: BrowseShellState) {
        savedStates += state
    }

    override fun runLoadIntent(intent: BrowseSessionLoadIntent): Job? =
        when (intent) {
            is BrowseSessionLoadIntent.RouteDisplayed -> {
                loadedRoutes += intent.route
                routeJobs[intent.route]
            }
    }
}
