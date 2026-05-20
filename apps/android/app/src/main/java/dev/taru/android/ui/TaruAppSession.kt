package dev.taru.android.ui

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileRepository
import dev.taru.android.connection.ServerProfileSnapshot
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

internal data class TaruAppState(
    val snapshot: ServerProfileSnapshot = ServerProfileSnapshot(),
    val connectionRequested: Boolean = false,
) {
    val activeProfile: ServerProfile? = ServerProfileRepository(snapshot).activeProfile()
    val shouldShowConnection: Boolean = activeProfile == null || connectionRequested
}

internal sealed interface TaruAppAction {
    data class SnapshotChanged(val snapshot: ServerProfileSnapshot) : TaruAppAction
    data object RequestConnection : TaruAppAction
}

internal interface TaruAppRuntime {
    fun saveSnapshot(snapshot: ServerProfileSnapshot)
}

internal class TaruAppSession(
    initialSnapshot: ServerProfileSnapshot,
    private val runtime: TaruAppRuntime,
) {
    private val _state = MutableStateFlow(
        TaruAppState(
            snapshot = initialSnapshot,
            connectionRequested = ServerProfileRepository(initialSnapshot).activeProfile() == null,
        ),
    )
    val state: StateFlow<TaruAppState> = _state.asStateFlow()

    fun dispatch(action: TaruAppAction) {
        when (action) {
            is TaruAppAction.SnapshotChanged -> {
                runtime.saveSnapshot(action.snapshot)
                _state.update {
                    TaruAppState(
                        snapshot = action.snapshot,
                        connectionRequested = ServerProfileRepository(action.snapshot).activeProfile() == null,
                    )
                }
            }
            TaruAppAction.RequestConnection -> {
                _state.update { it.copy(connectionRequested = true) }
            }
        }
    }
}
