package dev.nako.android.ui.connection

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
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.nako.android.connection.ConnectionCheckResult
import dev.nako.android.connection.ConnectionFailureCategory
import dev.nako.android.connection.InMemoryServerProfileStore
import dev.nako.android.connection.InMemoryTokenVault
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
import dev.nako.android.connection.NakoConnectionClient
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.theme.NakoAndroidTheme
import dev.nako.android.ui.theme.NakoShape
import dev.nako.android.ui.theme.NakoSpacing
import dev.nako.android.ui.theme.NakoTextMuted
import dev.nako.android.ui.theme.NakoTextSecondary
import dev.nako.sdk.NAKO_API_VERSION_HEADER

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun NakoConnectionShellContent(
    runtime: ConnectionRuntime,
    initialSnapshot: ServerProfileSnapshot,
    modifier: Modifier = Modifier,
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit = {},
) {
    val scope = rememberCoroutineScope()
    val session = remember(initialSnapshot, runtime, scope) {
        ConnectionSession(
            initialSnapshot = initialSnapshot,
            runtime = runtime,
            onSnapshotChanged = onSnapshotChanged,
            scope = scope,
        )
    }
    val state by session.state.collectAsStateWithLifecycle()

    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
        contentColor = MaterialTheme.colorScheme.onBackground,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState())
                .padding(NakoSpacing.xlarge),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.large),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(NakoSpacing.small)) {
                Text(
                    text = "Nako",
                    style = MaterialTheme.typography.headlineLarge,
                )
                Text(
                    text = "Connect to a server",
                    color = NakoTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }

            Surface(
                modifier = Modifier.fillMaxWidth(),
                shape = NakoShape.medium,
                color = MaterialTheme.colorScheme.surface,
                tonalElevation = NakoSpacing.xsmall,
            ) {
                Column(
                    modifier = Modifier.padding(NakoSpacing.large),
                    verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                ) {
                    OutlinedTextField(
                        modifier = Modifier.fillMaxWidth(),
                        value = state.displayName,
                        onValueChange = { session.dispatch(ConnectionAction.DisplayNameChanged(it)) },
                        label = { Text("Display name") },
                        singleLine = true,
                    )
                    OutlinedTextField(
                        modifier = Modifier.fillMaxWidth(),
                        value = state.serverUrl,
                        onValueChange = { session.dispatch(ConnectionAction.ServerUrlChanged(it)) },
                        label = { Text("Server URL") },
                        singleLine = true,
                    )
                    OutlinedTextField(
                        modifier = Modifier.fillMaxWidth(),
                        value = state.accessToken,
                        onValueChange = { session.dispatch(ConnectionAction.AccessTokenChanged(it)) },
                        label = { Text(stringResource(NakoStrings.accessKeyLabel)) },
                        placeholder = { Text(stringResource(NakoStrings.accessKeyLabel)) },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                    )

                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                        verticalArrangement = Arrangement.spacedBy(NakoSpacing.small),
                    ) {
                        Button(
                            enabled = !state.isChecking,
                            onClick = {
                                session.dispatch(ConnectionAction.TestConnection)
                            },
                        ) {
                            Text(if (state.isChecking) "Testing" else "Test Connection")
                        }

                        Button(
                            enabled = state.canSave,
                            onClick = {
                                session.dispatch(ConnectionAction.SaveProfile)
                            },
                        ) {
                            Text("Save")
                        }
                    }

                    state.checkResult?.let { result ->
                        ConnectionResultSummary(result)
                    }
                }
            }

            SavedServerProfiles(
                snapshot = state.snapshot,
                onSwitch = { profile ->
                    session.dispatch(ConnectionAction.SwitchProfile(profile))
                },
            )
        }
    }
}

@Composable
private fun ConnectionResultSummary(result: ConnectionCheckResult) {
    val text = when (result) {
        is ConnectionCheckResult.Success ->
            "Connected. This server is compatible (${result.apiVersion})."
        is ConnectionCheckResult.Failure -> when (result.diagnostics.category) {
            ConnectionFailureCategory.InvalidUrl,
            ConnectionFailureCategory.MissingAccessToken,
            ConnectionFailureCategory.UnreachableServer,
            ConnectionFailureCategory.Unauthorized,
            ConnectionFailureCategory.UnsupportedApiVersion,
            ConnectionFailureCategory.TlsOrCertificate,
            ConnectionFailureCategory.InsecureCleartextHttp,
            ConnectionFailureCategory.PublicApiError,
            ConnectionFailureCategory.InvalidResponse,
            -> result.diagnostics.userMessage
        }
    }
    val secondary = when (result) {
        is ConnectionCheckResult.Success -> "Save this profile to make it active."
        is ConnectionCheckResult.Failure -> result.diagnostics.publicError?.let {
            "${it.code}: ${it.message}"
        } ?: "Check the server address, sign-in key, or network and retry."
    }

    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = NakoShape.medium,
        color = MaterialTheme.colorScheme.surfaceVariant,
    ) {
        Column(
            modifier = Modifier.padding(NakoSpacing.medium),
            verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
        ) {
            Text(
                text = text,
                style = MaterialTheme.typography.titleMedium,
            )
            Text(
                text = secondary,
                color = NakoTextSecondary,
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
    Column(verticalArrangement = Arrangement.spacedBy(NakoSpacing.medium)) {
        Row(modifier = Modifier.fillMaxWidth()) {
            Text(
                text = "Server profiles",
                modifier = Modifier.weight(1f),
                style = MaterialTheme.typography.titleLarge,
            )
            Text(
                text = "${snapshot.profiles.size}",
                color = NakoTextMuted,
                style = MaterialTheme.typography.labelMedium,
            )
        }

        if (snapshot.profiles.isEmpty()) {
            Text(
                text = "No saved server profiles.",
                color = NakoTextSecondary,
                style = MaterialTheme.typography.bodyMedium,
            )
        }

        snapshot.profiles.forEach { profile ->
            val isActive = profile.id == snapshot.activeProfileId
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .border(
                        width = NakoSpacing.xsmall / 4,
                        color = if (isActive) MaterialTheme.colorScheme.primary else Color.White.copy(alpha = 0.08f),
                        shape = NakoShape.medium,
                    ),
                shape = NakoShape.medium,
                color = MaterialTheme.colorScheme.surface,
            ) {
                Row(
                    modifier = Modifier.padding(NakoSpacing.large),
                    horizontalArrangement = Arrangement.spacedBy(NakoSpacing.medium),
                ) {
                    Column(
                        modifier = Modifier.weight(1f),
                        verticalArrangement = Arrangement.spacedBy(NakoSpacing.xsmall),
                    ) {
                        Text(
                            text = profile.displayName,
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Text(
                            text = profile.baseUrl,
                            color = NakoTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                        )
                        profile.lastObservedApiVersion?.let { version ->
                            Text(
                                text = "Compatibility $version",
                                color = NakoTextMuted,
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

        Spacer(modifier = Modifier.height(NakoSpacing.medium))
    }
}

@Preview
@Composable
private fun NakoConnectionShellPreview() {
    NakoAndroidTheme(darkTheme = true) {
        NakoConnectionShellContent(
            runtime = ClientConnectionRuntime(
                store = InMemoryServerProfileStore(),
                tokenVault = InMemoryTokenVault(),
                client = NakoConnectionClient(
                    transport = object : dev.nako.android.connection.NakoHttpTransport {
                        override suspend fun execute(
                            request: dev.nako.android.connection.NakoHttpRequest,
                        ): dev.nako.android.connection.NakoHttpResponse =
                            dev.nako.android.connection.NakoHttpResponse(
                                statusCode = 200,
                                headers = mapOf(NAKO_API_VERSION_HEADER to listOf("v1")),
                                body = """{"status":"ok","version":"v1"}""",
                            )
                    },
                ),
            ),
            initialSnapshot = ServerProfileSnapshot(),
        )
    }
}
