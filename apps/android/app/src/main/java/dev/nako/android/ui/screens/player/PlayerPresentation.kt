package dev.nako.android.ui.screens.player

import dev.nako.android.connection.SafeRequestPreview
import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.player.PlaybackLaunchRequest
import dev.nako.android.player.PlaybackResumeSource

internal data class PlayerChromePresentation(
    val title: String,
    val modeLabel: String,
    val sourceLabel: String,
    val backdropTitle: String,
    val resumeLabel: String?,
    val sessionLabel: String?,
    val sessionAccessibilityLabel: String?,
)

internal const val PlayerMedia3ControllerClearanceDp = 132

internal data class PlaybackErrorPresentation(
    val title: String,
    val body: String,
    val primaryAction: String,
    val secondaryAction: String,
    val diagnostics: String,
)

internal fun playerChromePresentation(launch: PlaybackLaunchRequest): PlayerChromePresentation =
    PlayerChromePresentation(
        title = launch.title.ifBlank { "Nako Playback" },
        modeLabel = playerModeLabel(launch.playbackMode),
        sourceLabel = "Selected version",
        backdropTitle = launch.title.ifBlank { "Nako Playback" },
        resumeLabel = launch.resumePositionMs
            ?.takeIf { it > 0L }
            ?.let { "${resumeSourceLabel(launch.resumeSource)} ${durationLabel(it)}" },
        sessionLabel = launch.sessionId
            ?.takeIf { it.isNotBlank() }
            ?.let { "Playback session active" },
        sessionAccessibilityLabel = launch.sessionId
            ?.takeIf { it.isNotBlank() }
            ?.let { "Playback session is active" },
    )

internal fun playbackErrorPresentation(
    errorCodeName: String?,
    launch: PlaybackLaunchRequest,
): PlaybackErrorPresentation {
    val mode = playerModeLabel(launch.playbackMode)
    val code = errorCodeName?.takeIf { it.isNotBlank() } ?: "unknown"
    return PlaybackErrorPresentation(
        title = "Playback interrupted",
        body = "The player could not continue this $mode playback. Retry, or return to details and choose another version.",
        primaryAction = "Retry playback",
        secondaryAction = "Back to detail",
        diagnostics = playbackDiagnostics(
            errorCodeName = code,
            safeRequest = launch.safeRequest,
            playbackMode = launch.playbackMode,
            sessionActive = !launch.sessionId.isNullOrBlank(),
        ),
    )
}

internal fun playerModeLabel(mode: ClientPlaybackMode): String =
    when (mode) {
        ClientPlaybackMode.DirectPlay -> "Direct"
        ClientPlaybackMode.Remux -> "Remux"
        ClientPlaybackMode.Transcode -> "HLS"
        ClientPlaybackMode.Unknown -> "Unknown"
    }

private fun resumeSourceLabel(source: PlaybackResumeSource?): String =
    when (source) {
        PlaybackResumeSource.UserPlaybackState -> "Resume from server"
        PlaybackResumeSource.DeviceLocal,
        null,
        -> "Resume on this device"
    }

internal fun durationLabel(positionMs: Long): String {
    val totalSeconds = positionMs.coerceAtLeast(0L) / 1_000L
    val hours = totalSeconds / 3_600L
    val minutes = (totalSeconds % 3_600L) / 60L
    val seconds = totalSeconds % 60L
    return if (hours > 0) {
        "%d:%02d:%02d".format(hours, minutes, seconds)
    } else {
        "%d:%02d".format(minutes, seconds)
    }
}

private fun playbackDiagnostics(
    errorCodeName: String,
    safeRequest: SafeRequestPreview,
    playbackMode: ClientPlaybackMode,
    sessionActive: Boolean,
): String =
    buildString {
        appendLine("error=$errorCodeName")
        appendLine("mode=${playerModeLabel(playbackMode)}")
        appendLine("session_active=$sessionActive")
        appendLine("request=${safeRequest.method} ${safeRequest.url}")
        safeRequest.headers
            .toSortedMap(String.CASE_INSENSITIVE_ORDER)
            .forEach { (name, value) -> appendLine("header.$name=$value") }
    }.trim()
