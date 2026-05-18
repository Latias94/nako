package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.MediaItemDto
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackFailureCategory

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
        BrowseFailureCategory.MissingItem -> "Media Item unavailable"
        BrowseFailureCategory.MissingAccessToken -> "Authentication required"
        BrowseFailureCategory.UnreachableServer -> "Server unreachable"
        BrowseFailureCategory.Unauthorized -> "Authentication failed"
        BrowseFailureCategory.Forbidden -> "Permission denied"
        BrowseFailureCategory.UnsupportedApiVersion -> "Unsupported server"
        BrowseFailureCategory.TlsOrCertificate -> "Certificate problem"
        BrowseFailureCategory.PublicApiError -> "Browse failed"
        BrowseFailureCategory.InvalidResponse -> "Invalid response"
    }

internal fun playbackFailureTitle(category: PlaybackFailureCategory): String =
    when (category) {
        PlaybackFailureCategory.MissingSource -> "Media Source unavailable"
        PlaybackFailureCategory.MissingAccessToken -> "Authentication required"
        PlaybackFailureCategory.UnreachableServer -> "Server unreachable"
        PlaybackFailureCategory.Unauthorized -> "Authentication failed"
        PlaybackFailureCategory.Forbidden -> "Permission denied"
        PlaybackFailureCategory.UnsupportedApiVersion -> "Unsupported server"
        PlaybackFailureCategory.TlsOrCertificate -> "Certificate problem"
        PlaybackFailureCategory.UnsupportedSource -> "Unsupported source"
        PlaybackFailureCategory.SourceUnavailable -> "Source unavailable"
        PlaybackFailureCategory.SessionConflict -> "Session conflict"
        PlaybackFailureCategory.PublicApiError -> "Playback request failed"
        PlaybackFailureCategory.InvalidResponse -> "Invalid response"
    }

internal fun playbackModeLabel(mode: ClientPlaybackMode): String =
    when (mode) {
        ClientPlaybackMode.DirectPlay -> "Direct"
        ClientPlaybackMode.Remux -> "Remux"
        ClientPlaybackMode.Transcode -> "HLS"
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
