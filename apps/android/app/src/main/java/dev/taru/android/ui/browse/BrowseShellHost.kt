package dev.taru.android.ui.browse

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.ui.screens.settings.SettingsAction
import dev.taru.android.ui.screens.settings.SettingsRuntime
import dev.taru.android.ui.screens.settings.SettingsSession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch

internal interface BrowseShellRuntime {
    fun dataSource(profile: ServerProfile): BrowseDataSource
    fun playbackStarter(profile: ServerProfile): BrowsePlaybackStarter
    fun resumeResolver(profile: ServerProfile): BrowseResumeResolver
    fun settingsRuntime(): SettingsRuntime
}

internal class BrowseShellHost(
    profile: ServerProfile,
    snapshot: ServerProfileSnapshot,
    initialState: BrowseShellState = BrowseShellState(),
    runtime: BrowseShellRuntime,
    parentScope: CoroutineScope,
    private val saveState: (BrowseShellState) -> Unit = {},
) {
    private val hostJob = SupervisorJob(parentScope.coroutineContext[Job])
    private val hostScope = CoroutineScope(parentScope.coroutineContext + hostJob)
    private val browseSession = BrowseSession(
        initialState = initialState,
        dataSource = runtime.dataSource(profile),
        playbackStarter = runtime.playbackStarter(profile),
        resumeResolver = runtime.resumeResolver(profile),
        scope = hostScope,
    )
    private val settingsSession = SettingsSession(
        initialSnapshot = snapshot,
        runtime = runtime.settingsRuntime(),
    )
    private val effectCoordinator = BrowseSessionEffectCoordinator(
        initialState = initialState,
        sink = BrowseSessionSink(
            browseSession = browseSession,
            saveState = saveState,
        ),
    )

    val state: StateFlow<BrowseShellState> = browseSession.state

    init {
        hostScope.launch {
            browseSession.state.collect { next ->
                effectCoordinator.accept(next)
            }
        }
        browseSession.dispatch(BrowseAction.LoadHome)
        effectCoordinator.start()
    }

    fun dispatch(action: BrowseAction): Job? {
        val job = browseSession.dispatch(action)
        val effectJob = effectCoordinator.afterUserAction()
        return job ?: effectJob
    }

    fun dispatchSettings(action: SettingsAction) {
        settingsSession.dispatch(action)
    }

    fun close() {
        hostScope.cancel()
    }
}

private class BrowseSessionSink(
    private val browseSession: BrowseSession,
    private val saveState: (BrowseShellState) -> Unit,
) : BrowseSessionEffectSink {
    override val currentState: BrowseShellState
        get() = browseSession.state.value

    override fun publishSaveableState(state: BrowseShellState) {
        saveState(state)
    }

    override fun runLoadIntent(intent: BrowseSessionLoadIntent): Job? =
        when (intent) {
            is BrowseSessionLoadIntent.RouteDisplayed ->
                browseSession.dispatch(BrowseAction.RouteDisplayed(intent.route))
        }
}
