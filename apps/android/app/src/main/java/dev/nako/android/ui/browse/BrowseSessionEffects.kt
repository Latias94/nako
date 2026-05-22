package dev.nako.android.ui.browse

import kotlinx.coroutines.Job

internal interface BrowseSessionEffectSink {
    val currentState: BrowseShellState
    fun publishSaveableState(state: BrowseShellState)
    fun runLoadIntent(intent: BrowseSessionLoadIntent): Job?
}

internal sealed interface BrowseSessionLoadIntent {
    data class RouteDisplayed(val route: NakoRoute) : BrowseSessionLoadIntent
}

internal class BrowseSessionEffectCoordinator(
    initialState: BrowseShellState,
    private val sink: BrowseSessionEffectSink,
) {
    private var displayedRoute: NakoRoute? = null
    private var pendingRoute: NakoRoute = initialState.currentRoute

    fun start(): Job? {
        publishCurrentState()
        return loadPendingRouteIfNeeded()
    }

    fun accept(state: BrowseShellState): Job? {
        pendingRoute = state.currentRoute
        sink.publishSaveableState(state)
        return loadPendingRouteIfNeeded()
    }

    fun afterUserAction(): Job? {
        pendingRoute = sink.currentState.currentRoute
        publishCurrentState()
        return loadPendingRouteIfNeeded()
    }

    private fun loadPendingRouteIfNeeded(): Job? {
        val route = pendingRoute
        if (route == displayedRoute) {
            return null
        }

        displayedRoute = route
        val job = sink.runLoadIntent(BrowseSessionLoadIntent.RouteDisplayed(route))
        publishCurrentState()
        return job
    }

    private fun publishCurrentState() {
        sink.publishSaveableState(sink.currentState)
    }

}
