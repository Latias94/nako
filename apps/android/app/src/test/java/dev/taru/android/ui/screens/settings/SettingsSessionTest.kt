package dev.taru.android.ui.screens.settings

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import org.junit.Assert.assertEquals
import org.junit.Test

class SettingsSessionTest {
    @Test
    fun switchingProfilePublishesSnapshotThroughRuntimeAndCallback() {
        val runtime = RecordingSettingsRuntime()
        val callbacks = mutableListOf<ServerProfileSnapshot>()
        val snapshot = snapshot()
        val session = SettingsSession(
            initialSnapshot = snapshot,
            runtime = runtime,
            onSnapshotChanged = callbacks::add,
        )

        session.dispatch(SettingsAction.SwitchProfile("server-2"))

        assertEquals("server-2", session.snapshot.activeProfileId)
        assertEquals("server-2", runtime.savedSnapshots.single().activeProfileId)
        assertEquals(session.snapshot, callbacks.single())
    }

    @Test
    fun signOutDeletesActiveTokenAndRequestsConnectionWithoutChangingSnapshot() {
        val runtime = RecordingSettingsRuntime()
        val snapshot = snapshot()
        val session = SettingsSession(
            initialSnapshot = snapshot,
            runtime = runtime,
        )

        session.dispatch(SettingsAction.SignOutActiveProfile)

        assertEquals(listOf("server-token:server-1"), runtime.deletedTokens)
        assertEquals(1, runtime.connectionRequests)
        assertEquals(snapshot, session.snapshot)
        assertEquals(emptyList<ServerProfileSnapshot>(), runtime.savedSnapshots)
    }

    private fun snapshot(): ServerProfileSnapshot =
        ServerProfileSnapshot(
            profiles = listOf(
                ServerProfile(
                    id = "server-1",
                    displayName = "Home",
                    baseUrl = "http://home.example.test",
                    tokenReference = "server-token:server-1",
                ),
                ServerProfile(
                    id = "server-2",
                    displayName = "Lab",
                    baseUrl = "http://lab.example.test",
                    tokenReference = "server-token:server-2",
                ),
            ),
            activeProfileId = "server-1",
        )
}

private class RecordingSettingsRuntime : SettingsRuntime {
    val savedSnapshots = mutableListOf<ServerProfileSnapshot>()
    val deletedTokens = mutableListOf<String>()
    var connectionRequests = 0

    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        savedSnapshots += snapshot
    }

    override fun deleteToken(reference: String) {
        deletedTokens += reference
    }

    override fun requestConnection() {
        connectionRequests += 1
    }
}
