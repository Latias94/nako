package dev.nako.android.ui.screens.settings

import dev.nako.android.connection.ServerProfileRepository
import dev.nako.android.connection.ServerProfileSnapshot

internal sealed interface SettingsAction {
    data class SwitchProfile(val profileId: String) : SettingsAction
    data object SignOutActiveProfile : SettingsAction
}

internal interface SettingsRuntime {
    fun saveSnapshot(snapshot: ServerProfileSnapshot)
    fun deleteToken(reference: String)
    fun requestConnection()
}

internal class SettingsSession(
    initialSnapshot: ServerProfileSnapshot,
    private val runtime: SettingsRuntime,
    private val onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    var snapshot: ServerProfileSnapshot = initialSnapshot
        private set

    fun dispatch(action: SettingsAction) {
        when (action) {
            is SettingsAction.SwitchProfile -> switchProfile(action.profileId)
            SettingsAction.SignOutActiveProfile -> signOutActiveProfile()
        }
    }

    private fun switchProfile(profileId: String) {
        val repository = ServerProfileRepository(snapshot)
        repository.switchActive(profileId)
        publishSnapshot(repository.snapshot())
    }

    private fun signOutActiveProfile() {
        val profile = ServerProfileRepository(snapshot).activeProfile() ?: return
        runtime.deleteToken(profile.tokenReference)
        runtime.requestConnection()
    }

    private fun publishSnapshot(next: ServerProfileSnapshot) {
        snapshot = next
        runtime.saveSnapshot(next)
        onSnapshotChanged(next)
    }
}
