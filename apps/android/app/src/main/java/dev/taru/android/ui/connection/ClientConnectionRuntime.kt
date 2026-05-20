package dev.taru.android.ui.connection

import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TokenVault

internal class ClientConnectionRuntime(
    private val store: ServerProfileStore,
    private val tokenVault: TokenVault,
    private val client: TaruConnectionClient,
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
