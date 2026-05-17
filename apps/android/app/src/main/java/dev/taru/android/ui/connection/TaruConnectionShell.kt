package dev.taru.android.ui.connection

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import dev.taru.android.connection.AndroidSecureTokenVault
import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.ConnectionFailureCategory
import dev.taru.android.connection.InMemoryServerProfileStore
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.JdkTaruHttpTransport
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileRepository
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.ServerProfileStore
import dev.taru.android.connection.SharedPreferencesServerProfileStore
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.connection.TokenVault
import dev.taru.android.ui.theme.TaruAndroidTheme
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary
import kotlinx.coroutines.launch

@Composable
fun TaruConnectionShell(
    modifier: Modifier = Modifier,
) {
    val context = LocalContext.current
    val store = remember { SharedPreferencesServerProfileStore(context) }
    val tokenVault = remember { AndroidSecureTokenVault(context) }
    val client = remember { TaruConnectionClient(JdkTaruHttpTransport()) }

    TaruConnectionShellContent(
        modifier = modifier,
        store = store,
        tokenVault = tokenVault,
        client = client,
    )
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun TaruConnectionShellContent(
    store: ServerProfileStore,
    tokenVault: TokenVault,
    client: TaruConnectionClient,
    modifier: Modifier = Modifier,
    initialSnapshot: ServerProfileSnapshot? = null,
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    val scope = rememberCoroutineScope()
    var snapshot by remember(initialSnapshot) { mutableStateOf(initialSnapshot ?: store.load()) }
    var displayName by remember { mutableStateOf("") }
    var serverUrl by remember { mutableStateOf(snapshot.profiles.firstOrNull()?.baseUrl.orEmpty()) }
    var accessToken by remember { mutableStateOf("") }
    var isChecking by remember { mutableStateOf(false) }
    var checkResult by remember { mutableStateOf<ConnectionCheckResult?>(null) }

    fun updateSnapshot(next: ServerProfileSnapshot) {
        store.save(next)
        snapshot = next
        onSnapshotChanged(next)
    }

    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
        contentColor = MaterialTheme.colorScheme.onBackground,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(TaruSpacing.xlarge),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.large),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                Text(
                    text = "Taru",
                    style = MaterialTheme.typography.headlineLarge,
                )
                Text(
                    text = "Connect to a server",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }

            Surface(
                modifier = Modifier.fillMaxWidth(),
                shape = TaruShape.medium,
                color = MaterialTheme.colorScheme.surface,
                tonalElevation = TaruSpacing.xsmall,
            ) {
                Column(
                    modifier = Modifier.padding(TaruSpacing.large),
                    verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                ) {
                    OutlinedTextField(
                        modifier = Modifier.fillMaxWidth(),
                        value = displayName,
                        onValueChange = { displayName = it },
                        label = { Text("Display name") },
                        singleLine = true,
                    )
                    OutlinedTextField(
                        modifier = Modifier.fillMaxWidth(),
                        value = serverUrl,
                        onValueChange = {
                            serverUrl = it
                            checkResult = null
                        },
                        label = { Text("Server URL") },
                        singleLine = true,
                    )
                    OutlinedTextField(
                        modifier = Modifier.fillMaxWidth(),
                        value = accessToken,
                        onValueChange = {
                            accessToken = it
                            checkResult = null
                        },
                        label = { Text("Access Token") },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                    )

                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                    ) {
                        Button(
                            enabled = !isChecking,
                            onClick = {
                                scope.launch {
                                    isChecking = true
                                    val result = client.testConnection(serverUrl, accessToken)
                                    checkResult = result
                                    if (result is ConnectionCheckResult.Failure) {
                                        val repository = ServerProfileRepository(snapshot)
                                        val matchedProfile = repository
                                            .listProfiles()
                                            .firstOrNull { it.baseUrl == result.normalizedBaseUrl }
                                        if (matchedProfile != null) {
                                            repository.recordFailure(matchedProfile.id, result)
                                            updateSnapshot(repository.snapshot())
                                        }
                                    }
                                    isChecking = false
                                }
                            },
                        ) {
                            Text(if (isChecking) "Testing" else "Test Connection")
                        }

                        val success = checkResult as? ConnectionCheckResult.Success
                        Button(
                            enabled = success != null,
                            onClick = {
                                if (success != null) {
                                    val repository = ServerProfileRepository(snapshot)
                                    val profile = repository.upsertConnectedProfile(
                                        displayName = displayName,
                                        tokenReference = null,
                                        result = success,
                                    )
                                    tokenVault.saveToken(profile.tokenReference, accessToken)
                                    updateSnapshot(repository.snapshot())
                                    accessToken = ""
                                    displayName = ""
                                }
                            },
                        ) {
                            Text("Save")
                        }
                    }

                    checkResult?.let { result ->
                        ConnectionResultSummary(result)
                    }
                }
            }

            SavedServerProfiles(
                snapshot = snapshot,
                onSwitch = { profile ->
                    val repository = ServerProfileRepository(snapshot)
                    repository.switchActive(profile.id)
                    updateSnapshot(repository.snapshot())
                    serverUrl = profile.baseUrl
                    checkResult = null
                },
            )
        }
    }
}

@Composable
private fun ConnectionResultSummary(result: ConnectionCheckResult) {
    val text = when (result) {
        is ConnectionCheckResult.Success ->
            "Connected. Public Client API ${result.apiVersion} is supported."
        is ConnectionCheckResult.Failure -> when (result.diagnostics.category) {
            ConnectionFailureCategory.InvalidUrl,
            ConnectionFailureCategory.MissingAccessToken,
            ConnectionFailureCategory.UnreachableServer,
            ConnectionFailureCategory.Unauthorized,
            ConnectionFailureCategory.UnsupportedApiVersion,
            ConnectionFailureCategory.TlsOrCertificate,
            ConnectionFailureCategory.PublicApiError,
            ConnectionFailureCategory.InvalidResponse,
            -> result.diagnostics.userMessage
        }
    }
    val secondary = when (result) {
        is ConnectionCheckResult.Success -> "Save this profile to make it active."
        is ConnectionCheckResult.Failure -> result.diagnostics.publicError?.let {
            "${it.code}: ${it.message}"
        } ?: "Check the server address, token, or network and retry."
    }

    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
        ) {
            Text(
                text = text,
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = secondary,
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}

@Composable
private fun SavedServerProfiles(
    snapshot: ServerProfileSnapshot,
    onSwitch: (ServerProfile) -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium)) {
        Row(modifier = Modifier.fillMaxWidth()) {
            Text(
                text = "Server profiles",
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.titleLarge,
            )
            Text(
                text = "${snapshot.profiles.size}",
                color = TaruTextMuted,
                style = MaterialTheme.typography.labelMedium,
            )
        }

        if (snapshot.profiles.isEmpty()) {
            Text(
                text = "No saved server profiles.",
                color = TaruTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }

        snapshot.profiles.forEach { profile ->
            val isActive = profile.id == snapshot.activeProfileId
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .border(
                        width = TaruSpacing.xsmall / 4,
                        color = if (isActive) MaterialTheme.colorScheme.primary else Color.White.copy(alpha = 0.08f),
                        shape = TaruShape.medium,
                    ),
                shape = TaruShape.medium,
                color = MaterialTheme.colorScheme.surface,
            ) {
                Row(
                    modifier = Modifier.padding(TaruSpacing.large),
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                ) {
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
                    ) {
                        Text(
                            text = profile.displayName,
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Text(
                            text = profile.baseUrl,
                            color = TaruTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                        )
                        profile.lastObservedApiVersion?.let { version ->
                            Text(
                                text = "API $version",
                                color = TaruTextMuted,
                                style = MaterialTheme.typography.labelMedium,
                            )
                        }
                    }
                    OutlinedButton(
                        enabled = !isActive,
                        onClick = { onSwitch(profile) },
                    ) {
                        Text(if (isActive) "Active" else "Switch")
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(TaruSpacing.medium))
    }
}

@Preview
@Composable
private fun TaruConnectionShellPreview() {
    TaruAndroidTheme(darkTheme = true) {
        TaruConnectionShellContent(
            store = InMemoryServerProfileStore(),
            tokenVault = InMemoryTokenVault(),
            client = TaruConnectionClient(
                transport = object : dev.taru.android.connection.TaruHttpTransport {
                    override suspend fun execute(
                        request: dev.taru.android.connection.TaruHttpRequest,
                    ): dev.taru.android.connection.TaruHttpResponse =
                        dev.taru.android.connection.TaruHttpResponse(
                            statusCode = 200,
                            headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                            body = """{"status":"ok","version":"v1"}""",
                        )
                },
            ),
        )
    }
}
