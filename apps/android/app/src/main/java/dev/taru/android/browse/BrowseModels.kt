package dev.taru.android.browse

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.media.MediaProbeDto
import dev.taru.sdk.TARU_API_VERSION

data class PageRequest(
    val limit: Int = 50,
    val offset: Long = 0,
) {
    init {
        require(limit > 0) { "limit must be positive" }
        require(offset >= 0) { "offset must not be negative" }
    }
}

data class PageInfo(
    val limit: Int,
    val offset: Long,
    val returned: Int,
)

data class LibraryListResponse(
    val libraries: List<LibraryDto>,
    val page: PageInfo,
)

data class LibraryResponse(
    val library: LibraryDto,
)

data class LibrarySourcesResponse(
    val library: LibraryDto,
    val sources: List<LibrarySourceResponse> = emptyList(),
    val page: PageInfo,
)

data class LibrarySourceResponse(
    val source: MediaSourceDto,
    val item: MediaItemDto? = null,
    val probe: MediaProbeDto? = null,
) {
    override fun toString(): String =
        "LibrarySourceResponse(source=$source, item=$item, probe=$probe)"
}

data class LibraryDto(
    val id: String,
    val name: String,
    val roots: List<String> = emptyList(),
    val options: LibraryOptionsDto? = null,
) {
    override fun toString(): String =
        "LibraryDto(id=$id, name=$name, roots=<redacted:${roots.size}>, options=$options)"
}

data class LibraryOptionsDto(
    val domain: String? = null,
    val preset: String? = null,
)

data class ItemsResponse(
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

data class SearchRequest(
    val query: String = "",
    val facets: List<String> = emptyList(),
    val page: PageRequest = PageRequest(limit = 24),
)

data class SearchResponse(
    val hits: List<SearchItemHit>,
    val page: PageInfo,
)

data class SearchItemHit(
    val item: MediaItemDto,
    val score: Float,
)

data class FacetItemsResponse(
    val family: BrowseFacetFamily,
    val facetId: String,
    val facetLabel: String,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

enum class BrowseFacetFamily {
    Genre,
    Tag,
    Person,
}

data class GenreListResponse(
    val genres: List<GenreDto>,
    val page: PageInfo,
)

data class TagListResponse(
    val tags: List<TagDto>,
    val page: PageInfo,
)

data class GenreItemsResponse(
    val genre: GenreDto,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

data class TagItemsResponse(
    val tag: TagDto,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

data class PersonItemsResponse(
    val person: PersonDto,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

data class PersonResponse(
    val person: PersonDto,
)

data class GenreDto(
    val id: String,
    val name: String,
    val source: String = "",
)

data class TagDto(
    val id: String,
    val name: String,
    val source: String = "",
)

data class PersonDto(
    val id: String,
    val name: String,
    val sortName: String? = null,
    val overview: String? = null,
    val externalIds: List<ExternalIdDto> = emptyList(),
)

data class ExternalIdDto(
    val provider: String,
    val value: String,
)

data class ItemDetailResponse(
    val item: MediaItemDto,
    val sources: List<MediaSourceDto> = emptyList(),
    val credits: List<ItemCreditDto> = emptyList(),
    val genres: List<ItemGenreDto> = emptyList(),
    val tags: List<ItemTagDto> = emptyList(),
    val collections: List<CollectionItemDto> = emptyList(),
    val studios: List<ItemStudioDto> = emptyList(),
    val images: List<PublicImageRefDto> = emptyList(),
)

data class ImagesResponse(
    val itemId: String,
    val images: List<PublicImageRefDto> = emptyList(),
)

data class MediaItemDto(
    val id: String,
    val kind: String,
    val parentId: String? = null,
    val metadata: CanonicalMetadataDto,
)

data class CanonicalMetadataDto(
    val title: String,
    val originalTitle: String? = null,
    val sortTitle: String? = null,
    val overview: String? = null,
    val releaseDate: String? = null,
    val runtimeMinutes: Int? = null,
    val tagline: String? = null,
    val genres: List<String> = emptyList(),
    val tags: List<String> = emptyList(),
    val ratings: List<ContentRatingDto> = emptyList(),
)

data class ContentRatingDto(
    val source: String,
    val value: String,
)

data class MediaSourceDto(
    val id: String,
    val libraryId: String = "",
    val itemId: String = "",
    val fileName: String = "",
    val sizeBytes: Long? = null,
    val fingerprint: String? = null,
) {
    override fun toString(): String =
        "MediaSourceDto(id=$id, libraryId=$libraryId, itemId=$itemId, fileName=$fileName, sizeBytes=$sizeBytes, fingerprint=$fingerprint)"
}

data class ItemCreditDto(
    val itemId: String = "",
    val personId: String = "",
    val role: String? = null,
    val character: String? = null,
    val sortOrder: Int? = null,
)

data class ItemGenreDto(
    val itemId: String = "",
    val genreId: String = "",
)

data class ItemTagDto(
    val itemId: String = "",
    val tagId: String = "",
)

data class CollectionItemDto(
    val collectionId: String = "",
    val itemId: String = "",
    val sortOrder: Int? = null,
)

data class ItemStudioDto(
    val itemId: String = "",
    val studioId: String = "",
)

data class PublicImageRefDto(
    val id: String,
    val owner: Map<String, String> = emptyMap(),
    val kind: String,
    val url: String,
    val width: Int? = null,
    val height: Int? = null,
    val language: String? = null,
    val mediaType: String? = null,
    val etag: String? = null,
)

enum class BrowseFailureCategory {
    MissingItem,
    MissingLibrary,
    MissingPerson,
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
    val expectedApiVersion: String = TARU_API_VERSION,
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
