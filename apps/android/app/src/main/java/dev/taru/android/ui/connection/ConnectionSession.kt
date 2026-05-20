package dev.taru.android.ui.connection

import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileRepository
import dev.taru.android.connection.ServerProfileSnapshot
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

internal data class ConnectionState(
    val snapshot: ServerProfileSnapshot = ServerProfileSnapshot(),
    val displayName: String = "",
    val serverUrl: String = "",
    val accessToken: String = "",
    val isChecking: Boolean = false,
    val checkResult: ConnectionCheckResult? = null,
) {
    val canSave: Boolean
        get() = checkResult is ConnectionCheckResult.Success
}

internal sealed interface ConnectionAction {
    data class DisplayNameChanged(val value: String) : ConnectionAction
    data class ServerUrlChanged(val value: String) : ConnectionAction
    data class AccessTokenChanged(val value: String) : ConnectionAction
    data class SwitchProfile(val profile: ServerProfile) : ConnectionAction
    data object TestConnection : ConnectionAction
    data object SaveProfile : ConnectionAction
}

internal interface ConnectionRuntime {
    suspend fun testConnection(
        serverUrl: String,
        accessToken: String,
    ): ConnectionCheckResult

    fun saveToken(
        reference: String,
        token: String,
    )

    fun saveSnapshot(snapshot: ServerProfileSnapshot)
}

internal class ConnectionSession(
    initialSnapshot: ServerProfileSnapshot,
    private val runtime: ConnectionRuntime,
    private val onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
    private val scope: CoroutineScope,
) {
    private val _state = MutableStateFlow(
        ConnectionState(
            snapshot = initialSnapshot,
            serverUrl = initialSnapshot.profiles.firstOrNull()?.baseUrl.orEmpty(),
        ),
    )
    val state: StateFlow<ConnectionState> = _state.asStateFlow()

    private var testRequestId: Long = 0

    fun dispatch(action: ConnectionAction): Job? =
        when (action) {
            is ConnectionAction.DisplayNameChanged -> {
                _state.update { it.copy(displayName = action.value) }
                null
            }
            is ConnectionAction.ServerUrlChanged -> {
                testRequestId += 1
                _state.update {
                    it.copy(
                        serverUrl = action.value,
                        checkResult = null,
                        isChecking = false,
                    )
                }
                null
            }
            is ConnectionAction.AccessTokenChanged -> {
                testRequestId += 1
                _state.update {
                    it.copy(
                        accessToken = action.value,
                        checkResult = null,
                        isChecking = false,
                    )
                }
                null
            }
            is ConnectionAction.SwitchProfile -> {
                val repository = ServerProfileRepository(_state.value.snapshot)
                repository.switchActive(action.profile.id)
                publishSnapshot(repository.snapshot())
                _state.update {
                    it.copy(
                        serverUrl = action.profile.baseUrl,
                        checkResult = null,
                    )
                }
                null
            }
            ConnectionAction.TestConnection -> testConnection()
            ConnectionAction.SaveProfile -> {
                saveProfile()
                null
            }
        }

    private fun testConnection(): Job {
        val requestId = ++testRequestId
        val requestState = _state.value
        _state.update {
            it.copy(
                isChecking = true,
                checkResult = null,
            )
        }
        return scope.launch {
            val result = runtime.testConnection(
                serverUrl = requestState.serverUrl,
                accessToken = requestState.accessToken,
            )
            _state.update { current ->
                if (requestId == testRequestId) {
                    current.copy(
                        isChecking = false,
                        checkResult = result,
                    )
                } else {
                    current
                }
            }
            if (requestId == testRequestId && result is ConnectionCheckResult.Failure) {
                recordFailure(result)
            }
        }
    }

    private fun saveProfile() {
        val current = _state.value
        val success = current.checkResult as? ConnectionCheckResult.Success ?: return
        val repository = ServerProfileRepository(current.snapshot)
        val profile = repository.upsertConnectedProfile(
            displayName = current.displayName,
            tokenReference = null,
            result = success,
        )
        runtime.saveToken(profile.tokenReference, current.accessToken)
        publishSnapshot(repository.snapshot())
        _state.update {
            it.copy(
                displayName = "",
                serverUrl = profile.baseUrl,
                accessToken = "",
                checkResult = null,
                isChecking = false,
            )
        }
    }

    private fun recordFailure(result: ConnectionCheckResult.Failure) {
        val normalizedBaseUrl = result.normalizedBaseUrl ?: return
        val repository = ServerProfileRepository(_state.value.snapshot)
        val matchedProfile = repository
            .listProfiles()
            .firstOrNull { it.baseUrl == normalizedBaseUrl }
            ?: return
        repository.recordFailure(matchedProfile.id, result)
        publishSnapshot(repository.snapshot())
    }

    private fun publishSnapshot(snapshot: ServerProfileSnapshot) {
        runtime.saveSnapshot(snapshot)
        _state.update { it.copy(snapshot = snapshot) }
        onSnapshotChanged(snapshot)
    }
}
