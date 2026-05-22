package dev.nako.android.ui.connection

import dev.nako.android.connection.ConnectionCheckResult
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.ServerProfileStore
import dev.nako.android.connection.NakoConnectionClient
import dev.nako.android.connection.TokenVault

internal class ClientConnectionRuntime(
    private val store: ServerProfileStore,
    private val tokenVault: TokenVault,
    private val client: NakoConnectionClient,
) : ConnectionRuntime {
    override suspend fun testConnection(
        serverUrl: String,
        accessToken: String,
    ): ConnectionCheckResult =
        client.testConnection(serverUrl, accessToken)

    override fun saveToken(
        reference: String,
        token: String,
    ) {
        tokenVault.saveToken(reference, token)
    }

    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        store.save(snapshot)
    }
}
