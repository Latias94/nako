package dev.nako.android.ui

import dev.nako.android.browse.NakoBrowseClient
import dev.nako.android.connection.InMemoryTokenVault
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.ServerProfileStore
import dev.nako.android.connection.NakoConnectionClient
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.connection.NakoHttpResponse
import dev.nako.android.connection.NakoHttpTransport
import dev.nako.android.playback.InMemoryPlaybackPreferencesStore
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.player.InMemoryDevicePlaybackPositionStore
import dev.nako.android.userplayback.NakoUserPlaybackClient
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NakoAppCompositionTest {
    @Test
    fun environmentCreatesSessionFromStoredSnapshot() {
        val environment = testEnvironment(
            store = RecordingServerProfileStore(
                initial = snapshot(activeProfileId = "server-1"),
            ),
        )

        val session = environment.createSession()

        assertFalse(session.state.value.shouldShowConnection)
        assertEquals("server-1", session.state.value.activeProfile?.id)
    }

    @Test
    fun environmentRuntimePersistsSnapshotThroughStore() {
        val store = RecordingServerProfileStore()
        val environment = testEnvironment(store = store)
        val next = snapshot(activeProfileId = "server-2")

        environment.createRuntime().saveSnapshot(next)

        assertEquals(next, store.savedSnapshots.single())
        assertEquals(next, store.load())
    }

    @Test
    fun sessionCreatedFromEnvironmentPersistsSnapshotChanges() {
        val store = RecordingServerProfileStore()
        val session = testEnvironment(store = store).createSession()
        val next = snapshot(activeProfileId = "server-1")

        session.dispatch(NakoAppAction.SnapshotChanged(next))

        assertEquals(next, store.savedSnapshots.single())
        assertFalse(session.state.value.shouldShowConnection)
    }

    @Test
    fun environmentConnectionRuntimePersistsSnapshotsThroughStore() {
        val store = RecordingServerProfileStore()
        val runtime = testEnvironment(store = store).createConnectionRuntime()
        val next = snapshot(activeProfileId = "server-1")

        runtime.saveSnapshot(next)

        assertEquals(next, store.savedSnapshots.single())
        assertEquals(next, store.load())
    }
}

private fun testEnvironment(
    store: ServerProfileStore = RecordingServerProfileStore(),
): NakoAppEnvironment {
    val transport = RecordingTransport()
    return NakoAppEnvironment(
        store = store,
        tokenVault = InMemoryTokenVault(),
        connectionClient = NakoConnectionClient(transport),
        browseClient = NakoBrowseClient(transport),
        playbackClient = NakoPlaybackClient(transport),
        playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
        userPlaybackClient = NakoUserPlaybackClient(transport),
        positionStore = InMemoryDevicePlaybackPositionStore(),
    )
}

private class RecordingServerProfileStore(
    initial: ServerProfileSnapshot = ServerProfileSnapshot(),
) : ServerProfileStore {
    private var snapshot: ServerProfileSnapshot = initial
    val savedSnapshots: MutableList<ServerProfileSnapshot> = mutableListOf()

    override fun load(): ServerProfileSnapshot = snapshot

    override fun save(snapshot: ServerProfileSnapshot) {
        this.snapshot = snapshot
        savedSnapshots += snapshot
    }
}

private class RecordingTransport : NakoHttpTransport {
    override suspend fun execute(request: NakoHttpRequest): NakoHttpResponse =
        NakoHttpResponse(statusCode = 500)
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
