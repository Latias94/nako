package dev.taru.android.browse

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.PublicApiAuth
import dev.taru.android.connection.PublicApiFailure
import dev.taru.android.connection.PublicApiFailureKind
import dev.taru.android.connection.PublicApiResult
import dev.taru.android.connection.PublicClientApiExecutor
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.sdk.TaruPublicClientRequests
import dev.taru.sdk.GenreItemsResponse as SdkGenreItemsResponse
import dev.taru.sdk.GenreListResponse as SdkGenreListResponse
import dev.taru.sdk.ImagesResponse as SdkImagesResponse
import dev.taru.sdk.ItemDetailResponse as SdkItemDetailResponse
import dev.taru.sdk.ItemsResponse as SdkItemsResponse
import dev.taru.sdk.LibraryListResponse as SdkLibraryListResponse
import dev.taru.sdk.LibraryResponse as SdkLibraryResponse
import dev.taru.sdk.LibrarySourcesResponse as SdkLibrarySourcesResponse
import dev.taru.sdk.PersonItemsResponse as SdkPersonItemsResponse
import dev.taru.sdk.PersonResponse as SdkPersonResponse
import dev.taru.sdk.SearchResponse as SdkSearchResponse
import dev.taru.sdk.TagItemsResponse as SdkTagItemsResponse
import dev.taru.sdk.TagsResponse as SdkTagsResponse
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
        executeSdkJson<SdkLibraryListResponse, LibraryListResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .listLibraries(page.toSdkPageQuery())
                .pathAndQuery,
            transform = SdkLibraryListResponse::toAndroid,
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

        return executeSdkJson<SdkLibraryResponse, LibraryResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .getLibrary(libraryId)
                .pathAndQuery,
            notFoundCategory = BrowseFailureCategory.MissingLibrary,
            transform = SdkLibraryResponse::toAndroid,
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

        return executeSdkJson<SdkLibrarySourcesResponse, LibrarySourcesResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .listLibrarySources(libraryId, page.toSdkPageQuery())
                .pathAndQuery,
            notFoundCategory = BrowseFailureCategory.MissingLibrary,
            transform = SdkLibrarySourcesResponse::toAndroid,
        )
    }

    suspend fun listItems(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(limit = 24),
    ): BrowseResult<ItemsResponse> =
        executeSdkJson<SdkItemsResponse, ItemsResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .listItems(page.toSdkPageQuery())
                .pathAndQuery,
            transform = SdkItemsResponse::toAndroid,
        )

    suspend fun searchItems(
        profile: ServerProfile,
        accessToken: String,
        query: SearchRequest,
    ): BrowseResult<SearchResponse> =
        executeSdkJson<SdkSearchResponse, SearchResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .searchItems(
                    query = query.query,
                    facets = query.facets,
                    page = query.page.toSdkPageQuery(),
                )
                .pathAndQuery,
            transform = SdkSearchResponse::toAndroid,
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

        return executeSdkJson<SdkItemDetailResponse, ItemDetailResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .getItem(itemId)
                .pathAndQuery,
            transform = SdkItemDetailResponse::toAndroid,
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

        return executeSdkJson<SdkImagesResponse, ImagesResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .listItemImages(itemId)
                .pathAndQuery,
            transform = SdkImagesResponse::toAndroid,
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

        return executeSdkJson<SdkPersonResponse, PersonResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .getPerson(personId)
                .pathAndQuery,
            notFoundCategory = BrowseFailureCategory.MissingPerson,
            transform = SdkPersonResponse::toAndroid,
        )
    }

    suspend fun listGenres(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<GenreListResponse> =
        executeSdkJson<SdkGenreListResponse, GenreListResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .listGenres(page.toSdkPageQuery())
                .pathAndQuery,
            transform = SdkGenreListResponse::toAndroid,
        )

    suspend fun listTags(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<TagListResponse> =
        executeSdkJson<SdkTagsResponse, TagListResponse>(
            profile = profile,
            accessToken = accessToken,
            pathAndQuery = TaruPublicClientRequests
                .listTags(page.toSdkPageQuery())
                .pathAndQuery,
            transform = SdkTagsResponse::toAndroid,
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
            val result = executeSdkJson<SdkGenreItemsResponse, GenreItemsResponse>(
                profile = profile,
                accessToken = accessToken,
                pathAndQuery = TaruPublicClientRequests
                    .listGenreItems(genreId, page.toSdkPageQuery())
                    .pathAndQuery,
                transform = SdkGenreItemsResponse::toAndroid,
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
            val result = executeSdkJson<SdkTagItemsResponse, TagItemsResponse>(
                profile = profile,
                accessToken = accessToken,
                pathAndQuery = TaruPublicClientRequests
                    .listTagItems(tagId, page.toSdkPageQuery())
                    .pathAndQuery,
                transform = SdkTagItemsResponse::toAndroid,
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
            val result = executeSdkJson<SdkPersonItemsResponse, PersonItemsResponse>(
                profile = profile,
                accessToken = accessToken,
                pathAndQuery = TaruPublicClientRequests
                    .listPersonItems(personId, page.toSdkPageQuery())
                    .pathAndQuery,
                transform = SdkPersonItemsResponse::toAndroid,
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

    private suspend inline fun <reified WireT, AppT> executeSdkJson(
        profile: ServerProfile,
        accessToken: String,
        pathAndQuery: String,
        notFoundCategory: BrowseFailureCategory = BrowseFailureCategory.MissingItem,
        transform: (WireT) -> AppT,
    ): BrowseResult<AppT> =
        when (
            val result = executor.executeJson<WireT>(
                baseUrl = profile.baseUrl,
                pathAndQuery = pathAndQuery,
                auth = PublicApiAuth.Bearer(accessToken),
            )
        ) {
            is PublicApiResult.Success -> BrowseResult.Success(
                value = transform(result.value),
                request = result.request,
            )
            is PublicApiResult.Failure -> failureFor(notFoundCategory, result.failure)
        }

    @PublishedApi
    internal fun failureFor(
        notFoundCategory: BrowseFailureCategory,
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
                404 -> notFoundCategory
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

}
