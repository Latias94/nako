package dev.nako.android.ui.browse

import dev.nako.android.browse.BrowseFailureCategory
import dev.nako.android.browse.LibraryDto
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackFailureCategory

internal fun librarySubtitle(library: LibraryDto): String =
    listOfNotNull(
        library.options?.preset,
        library.options?.domain,
    ).joinToString(" / ").ifBlank { "Media Library" }

internal fun itemSecondaryText(item: MediaItemDto): String =
    listOfNotNull(
        item.kind,
        item.metadata.releaseDate?.take(4),
        item.metadata.runtimeMinutes?.let { "$it min" },
    ).joinToString(" / ")

internal fun browseFailureTitle(category: BrowseFailureCategory): String =
    when (category) {
        BrowseFailureCategory.MissingItem -> "Title unavailable"
        BrowseFailureCategory.MissingLibrary -> "Library unavailable"
        BrowseFailureCategory.MissingPerson -> "Person unavailable"
        BrowseFailureCategory.MissingAccessToken -> "Sign in required"
        BrowseFailureCategory.UnreachableServer -> "Server unreachable"
        BrowseFailureCategory.Unauthorized -> "Sign in again"
        BrowseFailureCategory.Forbidden -> "No access to this library"
        BrowseFailureCategory.UnsupportedApiVersion -> "Unsupported server"
        BrowseFailureCategory.TlsOrCertificate -> "Certificate problem"
        BrowseFailureCategory.PublicApiError -> "Browse failed"
        BrowseFailureCategory.InvalidResponse -> "Unexpected server reply"
    }

internal fun playbackFailureTitle(category: PlaybackFailureCategory): String =
    when (category) {
        PlaybackFailureCategory.MissingSource -> "Version unavailable"
        PlaybackFailureCategory.MissingSession -> "Playback session unavailable"
        PlaybackFailureCategory.MissingAccessToken -> "Sign in required"
        PlaybackFailureCategory.UnreachableServer -> "Server unreachable"
        PlaybackFailureCategory.Unauthorized -> "Sign in again"
        PlaybackFailureCategory.Forbidden -> "No access to this title"
        PlaybackFailureCategory.UnsupportedApiVersion -> "Unsupported server"
        PlaybackFailureCategory.TlsOrCertificate -> "Certificate problem"
        PlaybackFailureCategory.UnsupportedSource -> "Unsupported version"
        PlaybackFailureCategory.SourceUnavailable -> "Version unavailable"
        PlaybackFailureCategory.SessionConflict -> "Session conflict"
        PlaybackFailureCategory.PublicApiError -> "Playback request failed"
        PlaybackFailureCategory.InvalidResponse -> "Unexpected server reply"
    }

internal fun playbackModeLabel(mode: ClientPlaybackMode): String =
    when (mode) {
        ClientPlaybackMode.DirectPlay -> "Direct"
        ClientPlaybackMode.Remux -> "Remux"
        ClientPlaybackMode.Transcode -> "HLS"
        ClientPlaybackMode.Unknown -> "Unknown"
    }

internal fun byteSizeLabel(sizeBytes: Long?): String {
    val size = sizeBytes ?: return "Size unknown"
    val gib = 1024.0 * 1024.0 * 1024.0
    val mib = 1024.0 * 1024.0
    val kib = 1024.0
    return if (size >= gib) {
        "%.1f GiB".format(size / gib)
    } else if (size >= mib) {
        "%.1f MiB".format(size / mib)
    } else if (size >= kib) {
        "%.0f KiB".format(size / kib)
    } else {
        "$size B"
    }
}
