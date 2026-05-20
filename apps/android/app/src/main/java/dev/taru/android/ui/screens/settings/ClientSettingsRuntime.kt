package dev.taru.android.ui.screens.settings

import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.TokenVault

internal class ClientSettingsRuntime(
    private val store: ServerProfileStore,
    private val tokenVault: TokenVault,
    private val onChangeServer: () -> Unit,
) : SettingsRuntime {
    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        store.save(snapshot)
    }

    override fun deleteToken(reference: String) {
        tokenVault.deleteToken(reference)
    }

    override fun requestConnection() {
        onChangeServer()
    }
}
