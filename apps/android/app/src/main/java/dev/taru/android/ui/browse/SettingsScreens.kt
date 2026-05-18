package dev.taru.android.ui.browse

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.ExitToApp
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.CheckCircle
import androidx.compose.material.icons.rounded.ClosedCaption
import androidx.compose.material.icons.rounded.ContentCopy
import androidx.compose.material.icons.rounded.ErrorOutline
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.Language
import androidx.compose.material.icons.rounded.Lock
import androidx.compose.material.icons.rounded.Movie
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material.icons.rounded.SaveAlt
import androidx.compose.material.icons.rounded.Security
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material.icons.rounded.SignalCellularAlt
import androidx.compose.material.icons.rounded.Storage
import androidx.compose.material.icons.rounded.SyncAlt
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileRepository
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TokenVault
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextSecondary
import kotlinx.coroutines.launch

@Composable
internal fun SettingsHomeScreen(
    profile: ServerProfile,
    snapshot: ServerProfileSnapshot,
    onChangeServer: () -> Unit,
    onOpenServerProfile: () -> Unit,
) {
    TaruScrollColumn {
        PageTitle(
            title = "Settings",
            subtitle = "Client identity, connection, and playback preferences.",
            icon = Icons.Rounded.Settings,
            trailing = {
                StatusPill(
                    text = profile.displayName,
                    icon = Icons.Rounded.Storage,
                    onClick = onOpenServerProfile,
                )
            },
        )

        SectionLabel("Active server")
        ServerSummaryCard(
            profile = profile,
            onClick = onOpenServerProfile,
        )

        SettingsGroup(
            title = "Account access",
            rows = listOf(
                SettingsRow("Switch server", null, Icons.Rounded.SyncAlt, onChangeServer),
                SettingsRow("Re-authenticate", null, Icons.Rounded.Security, onChangeServer),
                SettingsRow("Sign out", null, Icons.AutoMirrored.Rounded.ExitToApp, onOpenServerProfile),
            ),
        )

        SettingsGroup(
            title = "Playback preferences",
            rows = listOf(
                SettingsRow("Playback mode", "Auto", Icons.Rounded.PlayArrow),
                SettingsRow("Streaming preference", "Prefer Direct", Icons.Rounded.SignalCellularAlt),
                SettingsRow("Data saving", "Off", Icons.Rounded.SaveAlt),
                SettingsRow("Compatibility", "Compatibility first", Icons.Rounded.Movie),
            ),
        )

        SettingsGroup(
            title = "Subtitles",
            rows = listOf(
                SettingsRow("Subtitle mode", "Default", Icons.Rounded.ClosedCaption),
                SettingsRow("Preferred language", "English", Icons.Rounded.Language),
            ),
        )

        SettingsGroup(
            title = "Diagnostics",
            rows = listOf(
                SettingsRow("Last public error", profile.lastPublicError?.code ?: "None", Icons.Rounded.ErrorOutline),
                SettingsRow("Copy diagnostics", "Sanitized report", Icons.Rounded.ContentCopy),
            ),
        )

        SettingsGroup(
            title = "About",
            rows = listOf(
                SettingsRow("Profiles", snapshot.profiles.size.toString(), Icons.Rounded.Storage),
                SettingsRow("Version", "Taru 0.1.0", Icons.Rounded.Info),
            ),
        )
    }
}

@Composable
internal fun ServerProfileScreen(
    activeProfile: ServerProfile,
    snapshot: ServerProfileSnapshot,
    tokenVault: TokenVault,
    onBack: () -> Unit,
    onChangeServer: () -> Unit,
    onSnapshotChanged: (ServerProfileSnapshot) -> Unit,
) {
    val scope = rememberCoroutineScope()
    TaruScrollColumn {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            IconButton(onClick = onBack) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                    contentDescription = "Back",
                )
            }
            Text(
                text = "Server profile",
                style = MaterialTheme.typography.headlineLarge,
            )
        }

        ServerSummaryCard(profile = activeProfile)

        SurfaceCard {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconBadge(icon = Icons.Rounded.Language)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = "Base URL",
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Text(
                        text = activeProfile.baseUrl,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }

        SurfaceCard {
            Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium)) {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconBadge(icon = Icons.Rounded.Lock)
                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = "Server access token",
                            style = MaterialTheme.typography.titleMedium,
                        )
                        Text(
                            text = "Stored securely on this device",
                            color = TaruTextSecondary,
                            style = MaterialTheme.typography.bodyMedium,
                        )
                    }
                }
                Button(
                    onClick = onChangeServer,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Icon(
                        imageVector = Icons.Rounded.Security,
                        contentDescription = null,
                    )
                    Spacer(modifier = Modifier.width(TaruSpacing.small))
                    Text("Re-authenticate")
                }
                OutlinedButton(
                    onClick = onChangeServer,
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Icon(
                        imageVector = Icons.Rounded.Refresh,
                        contentDescription = null,
                    )
                    Spacer(modifier = Modifier.width(TaruSpacing.small))
                    Text("Replace access token")
                }
            }
        }

        SettingsGroup(
            title = "Connection",
            rows = listOf(
                SettingsRow("Public Client API", activeProfile.lastObservedApiVersion ?: "Unknown", Icons.Rounded.CheckCircle),
                SettingsRow("Copy diagnostics", "Sanitized report", Icons.Rounded.ContentCopy),
            ),
        )

        SettingsGroup(
            title = "Server profiles",
            rows = snapshot.profiles.map { profile ->
                SettingsRow(
                    label = profile.displayName,
                    value = if (profile.id == snapshot.activeProfileId) "Connected" else "Saved",
                    icon = Icons.Rounded.Storage,
                    onClick = {
                        val repository = ServerProfileRepository(snapshot)
                        repository.switchActive(profile.id)
                        onSnapshotChanged(repository.snapshot())
                    },
                )
            } + SettingsRow("Add server", null, Icons.Rounded.Add, onChangeServer),
        )

        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.error.copy(alpha = 0.14f),
            border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.52f)),
        ) {
            Row(
                modifier = Modifier
                    .clickable {
                        scope.launch {
                            tokenVault.deleteToken(activeProfile.tokenReference)
                            onChangeServer()
                        }
                    }
                    .padding(TaruSpacing.large),
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ExitToApp,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.error,
                )
                Text(
                    text = "Sign out from this server",
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.titleMedium,
                )
            }
        }
    }
}
