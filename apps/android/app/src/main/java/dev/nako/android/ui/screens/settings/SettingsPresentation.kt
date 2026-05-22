package dev.nako.android.ui.screens.settings

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot

private const val UnknownApiLabel = "Not checked yet"
private const val NoRecentIssueLabel = "No recent issue"
private const val NoSuccessfulCheckLabel = "No successful check yet"

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
    val apiLabel = profile.lastObservedApiVersion ?: UnknownApiLabel
    val lastError = profile.lastPublicError?.code?.toSettingsErrorLabel() ?: NoRecentIssueLabel
    val connectionLabel = if (profile.lastSuccessfulConnectionAtMillis != null) {
        "Connection verified"
    } else {
        NoSuccessfulCheckLabel
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

private fun String.toSettingsErrorLabel(): String =
    when (this) {
        "transport_error" -> "Connection issue"
        "cleartext_http_not_allowed" -> "HTTPS required"
        "unsupported_api_version" -> "Unsupported server"
        "unauthorized" -> "Sign in again"
        "forbidden" -> "Access denied"
        "invalid_response" -> "Unexpected response"
        else -> replace('_', ' ')
            .replaceFirstChar { it.uppercase() }
            .ifBlank { NoRecentIssueLabel }
    }
