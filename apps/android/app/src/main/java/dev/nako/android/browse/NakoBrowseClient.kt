package dev.nako.android.browse

import dev.nako.android.connection.PublicApiFailure
import dev.nako.android.connection.PublicApiFailureKind
import dev.nako.android.connection.PublicApiResult
import dev.nako.android.connection.PublicClientRuntime
import dev.nako.android.connection.PublicErrorEnvelope
import dev.nako.android.connection.SafeRequestPreview
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.connection.NakoHttpTransport
import dev.nako.sdk.GenreItemsResponse as SdkGenreItemsResponse
import dev.nako.sdk.GenreListResponse as SdkGenreListResponse
import dev.nako.sdk.ImagesResponse as SdkImagesResponse
import dev.nako.sdk.ItemDetailResponse as SdkItemDetailResponse
import dev.nako.sdk.ItemsResponse as SdkItemsResponse
import dev.nako.sdk.LibraryListResponse as SdkLibraryListResponse
import dev.nako.sdk.LibraryResponse as SdkLibraryResponse
import dev.nako.sdk.LibrarySourcesResponse as SdkLibrarySourcesResponse
import dev.nako.sdk.PersonItemsResponse as SdkPersonItemsResponse
import dev.nako.sdk.PersonResponse as SdkPersonResponse
import dev.nako.sdk.SearchResponse as SdkSearchResponse
import dev.nako.sdk.TagItemsResponse as SdkTagItemsResponse
import dev.nako.sdk.TagsResponse as SdkTagsResponse
import kotlinx.serialization.json.Json

class NakoBrowseClient private constructor(
    transport: NakoHttpTransport,
    json: Json = Json { ignoreUnknownKeys = true },
    private val browseCore: BrowseCore,
) {
    constructor(
        transport: NakoHttpTransport,
        json: Json = Json { ignoreUnknownKeys = true },
    ) : this(
        transport = transport,
        json = json,
        browseCore = RustBrowseCore,
    )

    private val runtime = PublicClientRuntime(transport, json)

    suspend fun listLibraries(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<LibraryListResponse> =
        executeSdkJson<SdkLibraryListResponse, LibraryListResponse>(
            accessToken = accessToken,
            buildRequest = { token -> browseCore.listLibraries(profile, token, page).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.libraryDetail(profile, token, libraryId).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.librarySources(profile, token, libraryId, page).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.listItems(profile, token, page).request },
            transform = SdkItemsResponse::toAndroid,
        )

    suspend fun searchItems(
        profile: ServerProfile,
        accessToken: String,
        query: SearchRequest,
    ): BrowseResult<SearchResponse> =
        executeSdkJson<SdkSearchResponse, SearchResponse>(
            accessToken = accessToken,
            buildRequest = { token -> browseCore.searchItems(profile, token, query).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.itemDetail(profile, token, itemId).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.itemImages(profile, token, itemId).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.personDetail(profile, token, personId).request },
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
            accessToken = accessToken,
            buildRequest = { token -> browseCore.listGenres(profile, token, page).request },
            transform = SdkGenreListResponse::toAndroid,
        )

    suspend fun listTags(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest = PageRequest(),
    ): BrowseResult<TagListResponse> =
        executeSdkJson<SdkTagsResponse, TagListResponse>(
            accessToken = accessToken,
            buildRequest = { token -> browseCore.listTags(profile, token, page).request },
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
                accessToken = accessToken,
                buildRequest = { token -> browseCore.listGenreItems(profile, token, genreId, page).request },
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
                accessToken = accessToken,
                buildRequest = { token -> browseCore.listTagItems(profile, token, tagId, page).request },
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
                accessToken = accessToken,
                buildRequest = { token -> browseCore.listPersonItems(profile, token, personId, page).request },
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
        accessToken: String,
        crossinline buildRequest: (String) -> NakoHttpRequest,
        notFoundCategory: BrowseFailureCategory = BrowseFailureCategory.MissingItem,
        noinline transform: (WireT) -> AppT,
    ): BrowseResult<AppT> =
        when (
            val result = runtime.executeAuthenticatedJson(
                accessToken = accessToken,
                buildRequest = buildRequest,
                transform = transform,
            )
        ) {
            is PublicApiResult.Success -> BrowseResult.Success(
                value = result.value,
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
                "This server is not compatible with this Nako app version."
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
