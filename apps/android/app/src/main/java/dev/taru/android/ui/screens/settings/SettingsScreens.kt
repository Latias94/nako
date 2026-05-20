package dev.taru.android.ui.screens.settings

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
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
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.ui.rememberTaruClipboard
import dev.taru.android.ui.browse.IconBadge
import dev.taru.android.ui.browse.PageTitle
import dev.taru.android.ui.browse.SectionLabel
import dev.taru.android.ui.browse.StatusChip
import dev.taru.android.ui.browse.StatusPill
import dev.taru.android.ui.browse.SurfaceCard
import dev.taru.android.ui.browse.TaruScrollColumn
import dev.taru.android.ui.theme.TaruShape
import dev.taru.android.ui.theme.TaruSpacing
import dev.taru.android.ui.theme.TaruTextMuted
import dev.taru.android.ui.theme.TaruTextSecondary
import dev.taru.android.ui.theme.TaruTouchTarget

@Composable
internal fun SettingsHomeScreen(
    profile: ServerProfile,
    snapshot: ServerProfileSnapshot,
    onChangeServer: () -> Unit,
    onOpenServerProfile: () -> Unit,
) {
    val clipboard = rememberTaruClipboard()
    val diagnostics = remember(profile, snapshot) {
        settingsDiagnosticsPresentation(profile, snapshot)
    }

    TaruScrollColumn {
        PageTitle(
            title = "Settings",
            subtitle = "Client identity, playback defaults, and safe diagnostics.",
            icon = Icons.Rounded.Settings,
            trailing = {
                StatusPill(
                    text = profile.displayName,
                    icon = Icons.Rounded.Storage,
                    onClick = onOpenServerProfile,
                )
            },
        )

        ActiveServerPanel(
            profile = profile,
            diagnostics = diagnostics,
            onOpenServerProfile = onOpenServerProfile,
            onChangeServer = onChangeServer,
        )

        SettingsGroup(
            title = "Account Access",
            rows = listOf(
                SettingsRow("Switch server", "Choose active profile", Icons.Rounded.SyncAlt, onChangeServer),
                SettingsRow("Re-authenticate", "Replace access token", Icons.Rounded.Security, onChangeServer),
                SettingsRow("Server profile", "Connection details", Icons.Rounded.Storage, onOpenServerProfile),
            ),
        )

        SettingsGroup(
            title = "Playback",
            rows = listOf(
                SettingsRow("Playback decision", "Automatic", Icons.Rounded.PlayArrow),
                SettingsRow("Streaming preference", "Prefer Direct when available", Icons.Rounded.SignalCellularAlt),
                SettingsRow("Local resume", "Device-only until User Playback State API", Icons.Rounded.Movie),
            ),
        )

        SettingsGroup(
            title = "Tracks And Subtitles",
            rows = listOf(
                SettingsRow("Track selection", "Media3 controls", Icons.Rounded.Language),
                SettingsRow("Subtitle mode", "Player default", Icons.Rounded.ClosedCaption),
            ),
        )

        SettingsGroup(
            title = "Diagnostics",
            rows = listOf(
                SettingsRow("Public Client API", diagnostics.apiLabel, Icons.Rounded.CheckCircle),
                SettingsRow("Last public error", diagnostics.lastErrorLabel, Icons.Rounded.ErrorOutline),
                SettingsRow(
                    "Copy diagnostics",
                    "Sanitized report",
                    Icons.Rounded.ContentCopy,
                    onClick = { clipboard.copyPlainText("Taru diagnostics", diagnostics.report) },
                ),
            ),
        )

        SettingsGroup(
            title = "About",
            rows = listOf(
                SettingsRow("Profiles", diagnostics.profileCountLabel, Icons.Rounded.Storage),
                SettingsRow("Version", "Taru 0.1.0", Icons.Rounded.Info),
            ),
        )
    }
}

@Composable
internal fun ServerProfileScreen(
    activeProfile: ServerProfile,
    snapshot: ServerProfileSnapshot,
    onBack: () -> Unit,
    onChangeServer: () -> Unit,
    onSwitchProfile: (String) -> Unit,
    onSignOut: () -> Unit,
) {
    val clipboard = rememberTaruClipboard()
    val diagnostics = remember(activeProfile, snapshot) {
        settingsDiagnosticsPresentation(activeProfile, snapshot)
    }

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
            Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall)) {
                Text(
                    text = "Server Profile",
                    style = MaterialTheme.typography.headlineLarge,
                )
                Text(
                    text = "Access, connection, and profile switching.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }

        ServerIdentityPanel(
            profile = activeProfile,
            diagnostics = diagnostics,
        )

        AccessTokenPanel(
            onChangeServer = onChangeServer,
        )

        SettingsGroup(
            title = "Connection",
            rows = listOf(
                SettingsRow("Base URL", activeProfile.baseUrl, Icons.Rounded.Language),
                SettingsRow("Public Client API", diagnostics.apiLabel, Icons.Rounded.CheckCircle),
                SettingsRow("Last public error", diagnostics.lastErrorLabel, Icons.Rounded.ErrorOutline),
                SettingsRow(
                    "Copy diagnostics",
                    "Sanitized report",
                    Icons.Rounded.ContentCopy,
                    onClick = { clipboard.copyPlainText("Taru diagnostics", diagnostics.report) },
                ),
            ),
        )

        SettingsGroup(
            title = "Server Profiles",
            rows = snapshot.profiles.map { profile ->
                SettingsRow(
                    label = profile.displayName,
                    value = if (profile.id == snapshot.activeProfileId) "Connected" else "Saved",
                    icon = Icons.Rounded.Storage,
                    onClick = {
                        onSwitchProfile(profile.id)
                    },
                )
            } + SettingsRow("Add server", "Connect another profile", Icons.Rounded.Add, onChangeServer),
        )

        DangerSignOutPanel(
            onSignOut = onSignOut,
        )
    }
}

private data class SettingsRow(
    val label: String,
    val value: String?,
    val icon: ImageVector,
    val onClick: (() -> Unit)? = null,
)

@Composable
private fun ActiveServerPanel(
    profile: ServerProfile,
    diagnostics: SettingsDiagnosticsPresentation,
    onOpenServerProfile: () -> Unit,
    onChangeServer: () -> Unit,
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onOpenServerProfile() },
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary.copy(alpha = 0.42f)),
    ) {
        Column(
            modifier = Modifier.padding(TaruSpacing.large),
            verticalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                IconBadge(icon = Icons.Rounded.Storage)
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = profile.displayName,
                        style = MaterialTheme.typography.titleLarge,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(
                        text = diagnostics.connectionLabel,
                        color = MaterialTheme.colorScheme.primary,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                    Text(
                        text = profile.baseUrl,
                        color = TaruTextSecondary,
                        style = MaterialTheme.typography.bodyMedium,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
                StatusChip(text = "API ${diagnostics.apiLabel}")
            }
            Row(horizontalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
                Button(onClick = onOpenServerProfile) {
                    Text("Profile")
                }
                OutlinedButton(onClick = onChangeServer) {
                    Text("Switch")
                }
            }
        }
    }
}

@Composable
private fun ServerIdentityPanel(
    profile: ServerProfile,
    diagnostics: SettingsDiagnosticsPresentation,
) {
    SurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.Rounded.Storage)
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(TaruSpacing.xsmall),
            ) {
                Text(
                    text = profile.displayName,
                    style = MaterialTheme.typography.titleLarge,
                )
                Text(
                    text = diagnostics.connectionLabel,
                    color = MaterialTheme.colorScheme.primary,
                    style = MaterialTheme.typography.bodyMedium,
                )
                Text(
                    text = "Token reference is stored locally; token value is never shown.",
                    color = TaruTextMuted,
                    style = MaterialTheme.typography.labelMedium,
                )
            }
        }
    }
}

@Composable
private fun AccessTokenPanel(onChangeServer: () -> Unit) {
    SurfaceCard {
        Row(
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            IconBadge(icon = Icons.Rounded.Lock)
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = "Server Access Token",
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    text = "Stored securely on this device. The token value is not displayed or copied.",
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

@Composable
private fun SettingsGroup(
    title: String,
    rows: List<SettingsRow>,
) {
    Column(verticalArrangement = Arrangement.spacedBy(TaruSpacing.small)) {
        SectionLabel(title)
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = TaruShape.medium,
            color = MaterialTheme.colorScheme.surface,
        ) {
            Column {
                rows.forEach { row -> SettingsListRow(row) }
            }
        }
    }
}

@Composable
private fun SettingsListRow(row: SettingsRow) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .heightIn(min = TaruTouchTarget.minimum)
            .clickable(enabled = row.onClick != null) { row.onClick?.invoke() }
            .padding(TaruSpacing.medium),
        horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            imageVector = row.icon,
            contentDescription = null,
            tint = TaruTextSecondary,
        )
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = row.label,
                style = MaterialTheme.typography.titleMedium,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            row.value?.let {
                Text(
                    text = it,
                    color = if (it.equals("None", ignoreCase = true) || it.equals("Connected", ignoreCase = true)) {
                        MaterialTheme.colorScheme.primary
                    } else {
                        TaruTextSecondary
                    },
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

@Composable
private fun DangerSignOutPanel(onSignOut: () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = TaruShape.medium,
        color = MaterialTheme.colorScheme.error.copy(alpha = 0.14f),
        border = BorderStroke(1.dp, MaterialTheme.colorScheme.error.copy(alpha = 0.52f)),
    ) {
        Row(
            modifier = Modifier
                .clickable(onClick = onSignOut)
                .padding(TaruSpacing.large),
            horizontalArrangement = Arrangement.spacedBy(TaruSpacing.medium),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ExitToApp,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.error,
            )
            Column {
                Text(
                    text = "Sign out from this server",
                    color = MaterialTheme.colorScheme.error,
                    style = MaterialTheme.typography.titleMedium,
                )
                Text(
                    text = "Deletes the local token reference and returns to connection setup.",
                    color = TaruTextSecondary,
                    style = MaterialTheme.typography.bodyMedium,
                )
            }
        }
    }
}
