package dev.taru.android.browse

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

data class PageRequest(
    val limit: Int = 50,
    val offset: Long = 0,
) {
    init {
        require(limit > 0) { "limit must be positive" }
        require(offset >= 0) { "offset must not be negative" }
    }
}

@Serializable
data class PageInfo(
    val limit: Int,
    val offset: Long,
    val returned: Int,
)

@Serializable
data class LibraryListResponse(
    val libraries: List<LibraryDto>,
    val page: PageInfo,
)

@Serializable
data class LibraryDto(
    val id: String,
    val name: String,
    val roots: List<String> = emptyList(),
    val options: LibraryOptionsDto? = null,
)

@Serializable
data class LibraryOptionsDto(
    val domain: String? = null,
    val preset: String? = null,
)

@Serializable
data class ItemsResponse(
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

@Serializable
data class MediaItemDto(
    val id: String,
    val kind: String,
    @SerialName("parent_id")
    val parentId: String? = null,
    val metadata: CanonicalMetadataDto,
)

@Serializable
data class CanonicalMetadataDto(
    val title: String,
    @SerialName("original_title")
    val originalTitle: String? = null,
    @SerialName("sort_title")
    val sortTitle: String? = null,
    val overview: String? = null,
    @SerialName("release_date")
    val releaseDate: String? = null,
    @SerialName("runtime_minutes")
    val runtimeMinutes: Int? = null,
    val tagline: String? = null,
    val genres: List<String> = emptyList(),
    val tags: List<String> = emptyList(),
    val ratings: List<ContentRatingDto> = emptyList(),
    val images: List<ImageRefDto> = emptyList(),
)

@Serializable
data class ContentRatingDto(
    val source: String,
    val value: String,
)

@Serializable
data class ImageRefDto(
    val kind: String,
    val uri: String,
    val provider: String? = null,
    val width: Int? = null,
    val height: Int? = null,
    val language: String? = null,
)

enum class BrowseFailureCategory {
    MissingAccessToken,
    UnreachableServer,
    Unauthorized,
    Forbidden,
    UnsupportedApiVersion,
    TlsOrCertificate,
    PublicApiError,
    InvalidResponse,
}

data class SafeBrowseDiagnostics(
    val category: BrowseFailureCategory,
    val userMessage: String,
    val statusCode: Int? = null,
    val expectedApiVersion: String = dev.taru.android.connection.TaruPublicApiContract.expectedApiVersion,
    val observedApiVersion: String? = null,
    val publicError: PublicErrorEnvelope? = null,
    val request: SafeRequestPreview? = null,
)

sealed interface BrowseResult<out T> {
    data class Success<T>(
        val value: T,
        val request: SafeRequestPreview,
    ) : BrowseResult<T>

    data class Failure(
        val diagnostics: SafeBrowseDiagnostics,
    ) : BrowseResult<Nothing>
}
