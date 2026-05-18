package dev.taru.android.ui.screens.settings

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot

internal data class SettingsDiagnosticsPresentation(
    val apiLabel: String,
    val connectionLabel: String,
    val lastErrorLabel: String,
    val profileCountLabel: String,
    val report: String,
)

internal fun settingsDiagnosticsPresentation(
    profile: ServerProfile,
    snapshot: ServerProfileSnapshot,
): SettingsDiagnosticsPresentation {
    val apiLabel = profile.lastObservedApiVersion ?: "Unknown"
    val lastError = profile.lastPublicError?.code ?: "None"
    val connectionLabel = if (profile.lastSuccessfulConnectionAtMillis != null) {
        "Connection verified"
    } else {
        "No successful check recorded"
    }
    return SettingsDiagnosticsPresentation(
        apiLabel = apiLabel,
        connectionLabel = connectionLabel,
        lastErrorLabel = lastError,
        profileCountLabel = snapshot.profiles.size.toString(),
        report = buildString {
            appendLine("profile_id=${profile.id}")
            appendLine("display_name=${profile.displayName}")
            appendLine("base_url=${profile.baseUrl}")
            appendLine("api_version=$apiLabel")
            appendLine("connection=$connectionLabel")
            appendLine("last_public_error=$lastError")
            appendLine("profile_count=${snapshot.profiles.size}")
        }.trim(),
    )
}
