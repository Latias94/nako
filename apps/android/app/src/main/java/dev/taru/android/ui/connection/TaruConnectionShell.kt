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
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.ConnectionFailureCategory
import dev.taru.android.connection.InMemoryServerProfileStore
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TaruConnectionClient
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.ui.TaruStrings
import dev.taru.android.ui.theme.TaruAndroidTheme
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun TaruConnectionShellContent(
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
    val state by session.state.collectAsState()

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
                        label = { Text(stringResource(TaruStrings.accessKeyLabel)) },
                        placeholder = { Text(stringResource(TaruStrings.accessKeyLabel)) },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                    )

                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small),
                        verticalArrangement = Arrangement.spacedBy(TaruSpacing.small),
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
                                text = "Compatibility $version",
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
            runtime = ClientConnectionRuntime(
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
            ),
            initialSnapshot = ServerProfileSnapshot(),
        )
    }
}
