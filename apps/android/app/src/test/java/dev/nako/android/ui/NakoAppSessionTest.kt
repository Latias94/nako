package dev.nako.android.ui

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NakoAppSessionTest {
    @Test
    fun initialStateShowsConnectionWhenNoActiveProfileExists() {
        val session = NakoAppSession(
            initialSnapshot = ServerProfileSnapshot(),
            runtime = RecordingAppRuntime(),
        )

        assertTrue(session.state.value.shouldShowConnection)
        assertEquals(null, session.state.value.activeProfile)
    }

    @Test
    fun snapshotWithActiveProfileHidesConnectionAndPersistsSnapshot() {
        val runtime = RecordingAppRuntime()
        val session = NakoAppSession(
            initialSnapshot = ServerProfileSnapshot(),
            runtime = runtime,
        )

        session.dispatch(NakoAppAction.SnapshotChanged(snapshot(activeProfileId = "server-1")))

        assertFalse(session.state.value.shouldShowConnection)
        assertEquals("server-1", session.state.value.activeProfile?.id)
        assertEquals(session.state.value.snapshot, runtime.savedSnapshots.single())
    }

    @Test
    fun requestConnectionShowsConnectionEvenWhenActiveProfileExists() {
        val session = NakoAppSession(
            initialSnapshot = snapshot(activeProfileId = "server-1"),
            runtime = RecordingAppRuntime(),
        )

        session.dispatch(NakoAppAction.RequestConnection)

        assertTrue(session.state.value.shouldShowConnection)
        assertEquals("server-1", session.state.value.activeProfile?.id)
    }

    @Test
    fun savingActiveSnapshotAfterRequestConnectionReturnsToBrowse() {
        val session = NakoAppSession(
            initialSnapshot = snapshot(activeProfileId = "server-1"),
            runtime = RecordingAppRuntime(),
        )

        session.dispatch(NakoAppAction.RequestConnection)
        session.dispatch(NakoAppAction.SnapshotChanged(snapshot(activeProfileId = "server-2")))

        assertFalse(session.state.value.shouldShowConnection)
        assertEquals("server-2", session.state.value.activeProfile?.id)
    }

    private fun snapshot(activeProfileId: String): ServerProfileSnapshot =
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
            activeProfileId = activeProfileId,
        )
}

private class RecordingAppRuntime : NakoAppRuntime {
    val savedSnapshots = mutableListOf<ServerProfileSnapshot>()

    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        savedSnapshots += snapshot
    }
}
