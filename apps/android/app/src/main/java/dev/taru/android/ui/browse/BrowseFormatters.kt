package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.LibraryDto
import dev.taru.android.browse.MediaItemDto

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
