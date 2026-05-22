package dev.nako.android.ui

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileRepository
import dev.nako.android.connection.ServerProfileSnapshot
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

internal data class NakoAppState(
    val snapshot: ServerProfileSnapshot = ServerProfileSnapshot(),
    val connectionRequested: Boolean = false,
) {
    val activeProfile: ServerProfile? = ServerProfileRepository(snapshot).activeProfile()
    val shouldShowConnection: Boolean = activeProfile == null || connectionRequested
}

internal sealed interface NakoAppAction {
    data class SnapshotChanged(val snapshot: ServerProfileSnapshot) : NakoAppAction
    data object RequestConnection : NakoAppAction
}

internal interface NakoAppRuntime {
    fun saveSnapshot(snapshot: ServerProfileSnapshot)
}

internal class NakoAppSession(
    initialSnapshot: ServerProfileSnapshot,
    private val runtime: NakoAppRuntime,
) {
    private val _state = MutableStateFlow(
        NakoAppState(
            snapshot = initialSnapshot,
            connectionRequested = ServerProfileRepository(initialSnapshot).activeProfile() == null,
        ),
    )
    val state: StateFlow<NakoAppState> = _state.asStateFlow()

    fun dispatch(action: NakoAppAction) {
        when (action) {
            is NakoAppAction.SnapshotChanged -> {
                runtime.saveSnapshot(action.snapshot)
                _state.update {
                    NakoAppState(
                        snapshot = action.snapshot,
                        connectionRequested = ServerProfileRepository(action.snapshot).activeProfile() == null,
                    )
                }
            }
            NakoAppAction.RequestConnection -> {
                _state.update { it.copy(connectionRequested = true) }
            }
        }
    }
}
