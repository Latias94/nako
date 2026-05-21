package dev.taru.android.browse

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.PublicApiAuth
import dev.taru.android.connection.PublicApiFailure
import dev.taru.android.connection.PublicApiFailureKind
import dev.taru.android.connection.PublicApiResult
import dev.taru.android.connection.PublicApiUrl
import dev.taru.android.connection.PublicClientApiExecutor
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpTransport
import kotlinx.serialization.json.Json

class TaruBrowseClient(
    private val transport: TaruHttpTransport,
    private val json: Json = Json { ignoreUnknownKeys = true },
) {
    private val executor = PublicClientApiExecutor(transport, json)

    suspend fun listLibraries(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<LibraryListResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/libraries${pageQuery(page)}",
        )

    suspend fun libraryDetail(
        profile: ServerProfile,
        accessToken: String,
        libraryId: String,
    ): BrowseResult<LibraryResponse> {
        if (libraryId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.MissingLibrary,
                userMessage = "Choose a library before opening details.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/libraries/${PublicApiUrl.encodePathSegment(libraryId)}",
        )
    }

    suspend fun librarySources(
        profile: ServerProfile,
        accessToken: String,
        libraryId: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<LibrarySourcesResponse> {
        if (libraryId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.MissingLibrary,
                userMessage = "Choose a library before listing playable versions.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/libraries/${PublicApiUrl.encodePathSegment(libraryId)}/sources${pageQuery(page)}",
        )
    }

    suspend fun listItems(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<ItemsResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/items${pageQuery(page)}",
        )

    suspend fun searchItems(
        profile: ServerProfile,
        accessToken: String,
        query: SearchRequest,
    ): BrowseResult<SearchResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/search${searchQuery(query)}",
        )

    suspend fun itemDetail(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): BrowseResult<ItemDetailResponse> {
        if (itemId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.MissingItem,
                userMessage = "Choose a title before opening details.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/items/${PublicApiUrl.encodePathSegment(itemId)}",
        )
    }

    suspend fun itemImages(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): BrowseResult<ImagesResponse> {
        if (itemId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.MissingItem,
                userMessage = "Choose a title before loading artwork.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/items/${PublicApiUrl.encodePathSegment(itemId)}/images",
        )
    }

    suspend fun personDetail(
        profile: ServerProfile,
        accessToken: String,
        personId: String,
    ): BrowseResult<PersonResponse> {
        if (personId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.MissingPerson,
                userMessage = "Choose a Person before opening detail.",
            )
        }

        return executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/people/${PublicApiUrl.encodePathSegment(personId)}",
        )
    }

    suspend fun listGenres(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<GenreListResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/genres${pageQuery(page)}",
        )

    suspend fun listTags(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<TagListResponse> =
        executeJson(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = "/tags${pageQuery(page)}",
        )

    suspend fun listGenreItems(
        profile: ServerProfile,
        accessToken: String,
        genreId: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<FacetItemsResponse> {
        if (genreId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.InvalidResponse,
                userMessage = "Choose a supported genre before browsing related titles.",
            )
        }

        return when (
            val result = executeJson<GenreItemsResponse>(
                profile = profile,
                accessToken = accessToken,
                pathAndQuery = "/genres/${PublicApiUrl.encodePathSegment(genreId)}/items${pageQuery(page)}",
            )
        ) {
            is BrowseResult.Success -> BrowseResult.Success(
                value = FacetItemsResponse(
                    family = BrowseFacetFamily.Genre,
                    facetId = result.value.genre.id,
                    facetLabel = result.value.genre.name,
                    items = result.value.items,
                    page = result.value.page,
                ),
                request = result.request,
            )
            is BrowseResult.Failure -> result
        }
    }

    suspend fun listTagItems(
        profile: ServerProfile,
        accessToken: String,
        tagId: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<FacetItemsResponse> {
        if (tagId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.InvalidResponse,
                userMessage = "Choose a supported tag before browsing related titles.",
            )
        }

        return when (
            val result = executeJson<TagItemsResponse>(
                profile = profile,
                accessToken = accessToken,
                pathAndQuery = "/tags/${PublicApiUrl.encodePathSegment(tagId)}/items${pageQuery(page)}",
            )
        ) {
            is BrowseResult.Success -> BrowseResult.Success(
                value = FacetItemsResponse(
                    family = BrowseFacetFamily.Tag,
                    facetId = result.value.tag.id,
                    facetLabel = result.value.tag.name,
                    items = result.value.items,
                    page = result.value.page,
                ),
                request = result.request,
            )
            is BrowseResult.Failure -> result
        }
    }

    suspend fun listPersonItems(
        profile: ServerProfile,
        accessToken: String,
        personId: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<FacetItemsResponse> {
        if (personId.isBlank()) {
            return failure(
                category = BrowseFailureCategory.InvalidResponse,
                userMessage = "Choose a supported person before browsing related titles.",
            )
        }

        return when (
            val result = executeJson<PersonItemsResponse>(
                profile = profile,
                accessToken = accessToken,
                pathAndQuery = "/people/${PublicApiUrl.encodePathSegment(personId)}/items${pageQuery(page)}",
            )
        ) {
            is BrowseResult.Success -> BrowseResult.Success(
                value = FacetItemsResponse(
                    family = BrowseFacetFamily.Person,
                    facetId = result.value.person.id,
                    facetLabel = result.value.person.name,
                    items = result.value.items,
                    page = result.value.page,
                ),
                request = result.request,
            )
            is BrowseResult.Failure -> result
        }
    }

    private suspend inline fun <reified T> executeJson(
        profile: ServerProfile,
        accessToken: String,
        pathAndQuery: String,
    ): BrowseResult<T> {
        return when (
            val result = executor.executeJson<T>(
                baseUrl = profile.baseUrl,
                pathAndQuery = pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
            )
        ) {
            is PublicApiResult.Success -> BrowseResult.Success(
                value = result.value,
                request = result.request,
            )
            is PublicApiResult.Failure -> failureFor(pathAndQuery, result.failure)
        }
    }

    @PublishedApi
    internal fun failureFor(
        pathAndQuery: String,
        failure: PublicApiFailure,
    ): BrowseResult.Failure {
        val category = when (failure.kind) {
            PublicApiFailureKind.MissingAccessToken -> BrowseFailureCategory.MissingAccessToken
            PublicApiFailureKind.UnreachableServer -> BrowseFailureCategory.UnreachableServer
            PublicApiFailureKind.TlsOrCertificate -> BrowseFailureCategory.TlsOrCertificate
            PublicApiFailureKind.UnsupportedApiVersion -> BrowseFailureCategory.UnsupportedApiVersion
            PublicApiFailureKind.InvalidResponse -> BrowseFailureCategory.InvalidResponse
            PublicApiFailureKind.HttpError -> when (failure.statusCode) {
                401 -> BrowseFailureCategory.Unauthorized
                403 -> BrowseFailureCategory.Forbidden
                404 -> notFoundCategory(pathAndQuery)
                else -> BrowseFailureCategory.PublicApiError
            }
        }
        return failure(
            category = category,
            userMessage = userMessageFor(category),
            statusCode = failure.statusCode,
            observedApiVersion = failure.observedApiVersion,
            publicError = failure.publicError,
            request = failure.request,
        )
    }

    private fun notFoundCategory(pathAndQuery: String): BrowseFailureCategory =
        if (pathAndQuery.contains("/libraries/")) {
            BrowseFailureCategory.MissingLibrary
        } else if (pathAndQuery.contains("/people/") && !pathAndQuery.contains("/items")) {
            BrowseFailureCategory.MissingPerson
        } else {
            BrowseFailureCategory.MissingItem
        }

    private fun userMessageFor(category: BrowseFailureCategory): String =
        when (category) {
            BrowseFailureCategory.MissingAccessToken ->
                "Sign in again before browsing."
            BrowseFailureCategory.MissingItem ->
                "The requested title is no longer available."
            BrowseFailureCategory.MissingLibrary ->
                "The requested library is no longer available."
            BrowseFailureCategory.MissingPerson ->
                "The requested Person is no longer available."
            BrowseFailureCategory.Unauthorized ->
                "The server access key is invalid or expired."
            BrowseFailureCategory.Forbidden ->
                "This profile cannot browse the requested content."
            BrowseFailureCategory.UnsupportedApiVersion ->
                "This server is not compatible with this Taru app version."
            BrowseFailureCategory.TlsOrCertificate ->
                "The server TLS certificate could not be trusted."
            BrowseFailureCategory.UnreachableServer ->
                "The server could not be reached. Check the address and network."
            BrowseFailureCategory.InvalidResponse ->
                "The server reply could not be understood."
            BrowseFailureCategory.PublicApiError ->
                "The server reported a browsing issue."
        }

    private fun failure(
        category: BrowseFailureCategory,
        userMessage: String,
        statusCode: Int? = null,
        observedApiVersion: String? = null,
        publicError: PublicErrorEnvelope? = null,
        request: SafeRequestPreview? = null,
    ): BrowseResult.Failure =
        BrowseResult.Failure(
            diagnostics = SafeBrowseDiagnostics(
                category = category,
                userMessage = userMessage,
                statusCode = statusCode,
                observedApiVersion = observedApiVersion,
                publicError = publicError,
                request = request,
            ),
        )

    private fun searchQuery(query: SearchRequest): String =
        PublicApiUrl.queryString(
            buildList {
                if (query.query.isNotBlank()) {
                    add("q" to query.query)
                }
                if (query.facets.isNotEmpty()) {
                    add("facet" to query.facets.joinToString(","))
                }
                add("limit" to query.page.limit.toString())
                add("offset" to query.page.offset.toString())
            },
        )

    private fun pageQuery(page: PageRequest): String =
        PublicApiUrl.pageQuery(limit = page.limit, offset = page.offset)
}
