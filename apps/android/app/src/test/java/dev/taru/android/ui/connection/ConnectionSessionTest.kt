package dev.taru.android.ui.connection

import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.ConnectionFailureCategory
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeConnectionDiagnostics
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfileSnapshot
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ConnectionSessionTest {
    @Test
    fun formEditsClearPreviousCheckResult() = runBlocking {
        val runtime = RecordingConnectionRuntime(success = successFor("http://home.example.test"))
        val session = ConnectionSession(
            initialSnapshot = ServerProfileSnapshot(),
            runtime = runtime,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(ConnectionAction.ServerUrlChanged("http://home.example.test"))
        session.dispatch(ConnectionAction.AccessTokenChanged("secret-token"))
        session.dispatch(ConnectionAction.TestConnection)?.join()

        assertTrue(session.state.value.canSave)

        session.dispatch(ConnectionAction.AccessTokenChanged("new-token"))

        assertFalse(session.state.value.canSave)
        assertEquals(null, session.state.value.checkResult)
        assertFalse(session.state.value.isChecking)
    }

    @Test
    fun successfulCheckThenSavePersistsProfileTokenAndClearsSensitiveFormState() = runBlocking {
        val runtime = RecordingConnectionRuntime(success = successFor("http://home.example.test"))
        val snapshots = mutableListOf<ServerProfileSnapshot>()
        val session = ConnectionSession(
            initialSnapshot = ServerProfileSnapshot(),
            runtime = runtime,
            onSnapshotChanged = snapshots::add,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(ConnectionAction.DisplayNameChanged("Home"))
        session.dispatch(ConnectionAction.ServerUrlChanged(" http://home.example.test/ "))
        session.dispatch(ConnectionAction.AccessTokenChanged("secret-token"))
        session.dispatch(ConnectionAction.TestConnection)?.join()
        session.dispatch(ConnectionAction.SaveProfile)

        val state = session.state.value
        val profile = state.snapshot.profiles.single()
        assertEquals(profile.id, state.snapshot.activeProfileId)
        assertEquals("Home", profile.displayName)
        assertEquals("http://home.example.test", profile.baseUrl)
        assertEquals("", state.displayName)
        assertEquals("", state.accessToken)
        assertEquals(null, state.checkResult)
        assertEquals("secret-token", runtime.tokens[profile.tokenReference])
        assertEquals(state.snapshot, runtime.savedSnapshots.last())
        assertEquals(state.snapshot, snapshots.last())
    }

    @Test
    fun failedCheckRecordsDiagnosticsAgainstMatchingSavedProfile() = runBlocking {
        val initial = ServerProfileSnapshot()
        val setupRuntime = RecordingConnectionRuntime(success = successFor("http://home.example.test"))
        val setup = ConnectionSession(
            initialSnapshot = initial,
            runtime = setupRuntime,
            scope = CoroutineScope(coroutineContext + Job()),
        )
        setup.dispatch(ConnectionAction.DisplayNameChanged("Home"))
        setup.dispatch(ConnectionAction.ServerUrlChanged("http://home.example.test"))
        setup.dispatch(ConnectionAction.AccessTokenChanged("secret-token"))
        setup.dispatch(ConnectionAction.TestConnection)?.join()
        setup.dispatch(ConnectionAction.SaveProfile)

        val runtime = RecordingConnectionRuntime(failure = failureFor("http://home.example.test"))
        val session = ConnectionSession(
            initialSnapshot = setup.state.value.snapshot,
            runtime = runtime,
            scope = CoroutineScope(coroutineContext + Job()),
        )

        session.dispatch(ConnectionAction.AccessTokenChanged("bad-token"))
        session.dispatch(ConnectionAction.TestConnection)?.join()

        val profile = session.state.value.snapshot.profiles.single()
        assertEquals("unauthorized", profile.lastPublicError?.code)
        assertEquals(session.state.value.snapshot, runtime.savedSnapshots.last())
    }

    @Test
    fun switchingProfilePublishesSnapshotAndUpdatesFormServerUrl() = runBlocking {
        val runtime = RecordingConnectionRuntime(success = successFor("http://home.example.test"))
        val session = ConnectionSession(
            initialSnapshot = ServerProfileSnapshot(),
            runtime = runtime,
            scope = CoroutineScope(coroutineContext + Job()),
        )
        session.dispatch(ConnectionAction.DisplayNameChanged("Home"))
        session.dispatch(ConnectionAction.ServerUrlChanged("http://home.example.test"))
        session.dispatch(ConnectionAction.AccessTokenChanged("home-token"))
        session.dispatch(ConnectionAction.TestConnection)?.join()
        session.dispatch(ConnectionAction.SaveProfile)

        runtime.success = successFor("http://lab.example.test")
        session.dispatch(ConnectionAction.DisplayNameChanged("Lab"))
        session.dispatch(ConnectionAction.ServerUrlChanged("http://lab.example.test"))
        session.dispatch(ConnectionAction.AccessTokenChanged("lab-token"))
        session.dispatch(ConnectionAction.TestConnection)?.join()
        session.dispatch(ConnectionAction.SaveProfile)

        val home = session.state.value.snapshot.profiles.first { it.displayName == "Home" }
        session.dispatch(ConnectionAction.SwitchProfile(home))

        assertEquals(home.id, session.state.value.snapshot.activeProfileId)
        assertEquals(home.baseUrl, session.state.value.serverUrl)
        assertEquals(null, session.state.value.checkResult)
    }

    private fun successFor(baseUrl: String): ConnectionCheckResult.Success =
        ConnectionCheckResult.Success(
            normalizedBaseUrl = baseUrl,
            apiVersion = "v1",
            checkedAtMillis = 42L,
            healthRequest = SafeRequestPreview("GET", "$baseUrl/health"),
            authProbeRequest = SafeRequestPreview(
                method = "GET",
                url = "$baseUrl/libraries?limit=1&offset=0",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        )

    private fun failureFor(baseUrl: String): ConnectionCheckResult.Failure =
        ConnectionCheckResult.Failure(
            normalizedBaseUrl = baseUrl,
            diagnostics = SafeConnectionDiagnostics(
                category = ConnectionFailureCategory.Unauthorized,
                userMessage = "The access token is invalid or expired.",
                publicError = PublicErrorEnvelope("unauthorized", "authentication required"),
            ),
        )
}

private class RecordingConnectionRuntime(
    var success: ConnectionCheckResult.Success? = null,
    var failure: ConnectionCheckResult.Failure? = null,
) : ConnectionRuntime {
    val tokens = linkedMapOf<String, String>()
    val savedSnapshots = mutableListOf<ServerProfileSnapshot>()
    val tests = mutableListOf<Pair<String, String>>()

    override suspend fun testConnection(
        serverUrl: String,
        accessToken: String,
    ): ConnectionCheckResult {
        tests += serverUrl to accessToken
        return failure ?: requireNotNull(success)
    }

    override fun saveToken(
        reference: String,
        token: String,
    ) {
        tokens[reference] = token
    }

    override fun saveSnapshot(snapshot: ServerProfileSnapshot) {
        savedSnapshots += snapshot
    }
}
