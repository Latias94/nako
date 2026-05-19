package dev.taru.android.browse

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

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

data class SearchRequest(
    val query: String = "",
    val facets: List<String> = emptyList(),
    val page: PageRequest = PageRequest(limit = 24),
)

@Serializable
data class SearchResponse(
    val hits: List<SearchItemHit>,
    val page: PageInfo,
)

@Serializable
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

@Serializable
data class GenreItemsResponse(
    val genre: GenreDto,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

@Serializable
data class TagItemsResponse(
    val tag: TagDto,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

@Serializable
data class PersonItemsResponse(
    val person: PersonDto,
    val items: List<MediaItemDto>,
    val page: PageInfo,
)

@Serializable
data class GenreDto(
    val id: String,
    val name: String,
    val source: JsonElement? = null,
)

@Serializable
data class TagDto(
    val id: String,
    val name: String,
    val source: JsonElement? = null,
)

@Serializable
data class PersonDto(
    val id: String,
    val name: String,
    @SerialName("sort_name")
    val sortName: String? = null,
    val overview: String? = null,
    @SerialName("external_ids")
    val externalIds: List<JsonElement> = emptyList(),
)

@Serializable
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

@Serializable
data class ImagesResponse(
    @SerialName("item_id")
    val itemId: String,
    val images: List<PublicImageRefDto> = emptyList(),
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
)

@Serializable
data class ContentRatingDto(
    val source: String,
    val value: String,
)

@Serializable
data class MediaSourceDto(
    val id: String,
    @SerialName("library_id")
    val libraryId: String = "",
    @SerialName("item_id")
    val itemId: String = "",
    val locator: String = "",
    @SerialName("file_name")
    val fileName: String = "",
    @SerialName("size_bytes")
    val sizeBytes: Long? = null,
    val fingerprint: String? = null,
)

@Serializable
data class ItemCreditDto(
    @SerialName("item_id")
    val itemId: String = "",
    @SerialName("person_id")
    val personId: String = "",
    val role: JsonElement? = null,
    val character: String? = null,
    @SerialName("sort_order")
    val sortOrder: Int? = null,
)

@Serializable
data class ItemGenreDto(
    @SerialName("item_id")
    val itemId: String = "",
    @SerialName("genre_id")
    val genreId: String = "",
)

@Serializable
data class ItemTagDto(
    @SerialName("item_id")
    val itemId: String = "",
    @SerialName("tag_id")
    val tagId: String = "",
)

@Serializable
data class CollectionItemDto(
    @SerialName("collection_id")
    val collectionId: String = "",
    @SerialName("item_id")
    val itemId: String = "",
    @SerialName("sort_order")
    val sortOrder: Int? = null,
)

@Serializable
data class ItemStudioDto(
    @SerialName("item_id")
    val itemId: String = "",
    @SerialName("studio_id")
    val studioId: String = "",
)

@Serializable
data class PublicImageRefDto(
    val id: String,
    val owner: JsonElement? = null,
    val kind: JsonElement,
    val url: String,
    val width: Int? = null,
    val height: Int? = null,
    val language: String? = null,
    @SerialName("media_type")
    val mediaType: String? = null,
    val etag: String? = null,
)

enum class BrowseFailureCategory {
    MissingItem,
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
