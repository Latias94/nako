package dev.taru.android.ui

import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.playback.InMemoryPlaybackPreferencesStore
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.InMemoryDevicePlaybackPositionStore
import dev.taru.android.userplayback.TaruUserPlaybackClient
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TaruAppCompositionTest {
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

        session.dispatch(TaruAppAction.SnapshotChanged(next))

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
): TaruAppEnvironment {
    val transport = RecordingTransport()
    return TaruAppEnvironment(
        store = store,
        tokenVault = InMemoryTokenVault(),
        connectionClient = TaruConnectionClient(transport),
        browseClient = TaruBrowseClient(transport),
        playbackClient = TaruPlaybackClient(transport),
        playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
        userPlaybackClient = TaruUserPlaybackClient(transport),
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

private class RecordingTransport : TaruHttpTransport {
    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse =
        TaruHttpResponse(statusCode = 500)
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
