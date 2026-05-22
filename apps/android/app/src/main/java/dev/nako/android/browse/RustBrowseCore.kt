package dev.nako.android.browse

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.connection.toAndroidRequest
import uniffi.nako_client_uniffi.CoreBrowseEntityPagedRequestInput
import uniffi.nako_client_uniffi.CoreBrowseEntityRequestInput
import uniffi.nako_client_uniffi.CoreBrowsePagedRequestInput
import uniffi.nako_client_uniffi.CoreHttpRequest
import uniffi.nako_client_uniffi.CorePageQuery
import uniffi.nako_client_uniffi.CoreSearchItemsRequestInput

internal interface BrowseCore {
    fun listLibraries(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun libraryDetail(
        profile: ServerProfile,
        accessToken: String,
        libraryId: String,
    ): BrowseRequestDescriptor

    fun librarySources(
        profile: ServerProfile,
        accessToken: String,
        libraryId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun listItems(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun searchItems(
        profile: ServerProfile,
        accessToken: String,
        query: SearchRequest,
    ): BrowseRequestDescriptor

    fun itemDetail(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): BrowseRequestDescriptor

    fun itemImages(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): BrowseRequestDescriptor

    fun personDetail(
        profile: ServerProfile,
        accessToken: String,
        personId: String,
    ): BrowseRequestDescriptor

    fun listPersonItems(
        profile: ServerProfile,
        accessToken: String,
        personId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun listGenres(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun listGenreItems(
        profile: ServerProfile,
        accessToken: String,
        genreId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun listTags(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor

    fun listTagItems(
        profile: ServerProfile,
        accessToken: String,
        tagId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor
}

internal object RustBrowseCore : BrowseCore {
    override fun listLibraries(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListLibrariesRequest(
            pagedInput(profile, accessToken, page),
        ).toBrowseDescriptor()

    override fun libraryDetail(
        profile: ServerProfile,
        accessToken: String,
        libraryId: String,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildGetLibraryRequest(
            entityInput(profile, accessToken, libraryId),
        ).toBrowseDescriptor()

    override fun librarySources(
        profile: ServerProfile,
        accessToken: String,
        libraryId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListLibrarySourcesRequest(
            entityPagedInput(profile, accessToken, libraryId, page),
        ).toBrowseDescriptor()

    override fun listItems(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListItemsRequest(
            pagedInput(profile, accessToken, page),
        ).toBrowseDescriptor()

    override fun searchItems(
        profile: ServerProfile,
        accessToken: String,
        query: SearchRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildSearchItemsRequest(
            CoreSearchItemsRequestInput(
                baseUrl = profile.baseUrl,
                accessToken = accessToken,
                query = query.query,
                facets = query.facets,
                page = query.page.toCore(),
            ),
        ).toBrowseDescriptor()

    override fun itemDetail(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildGetItemRequest(
            entityInput(profile, accessToken, itemId),
        ).toBrowseDescriptor()

    override fun itemImages(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListItemImagesRequest(
            entityInput(profile, accessToken, itemId),
        ).toBrowseDescriptor()

    override fun personDetail(
        profile: ServerProfile,
        accessToken: String,
        personId: String,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildGetPersonRequest(
            entityInput(profile, accessToken, personId),
        ).toBrowseDescriptor()

    override fun listPersonItems(
        profile: ServerProfile,
        accessToken: String,
        personId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListPersonItemsRequest(
            entityPagedInput(profile, accessToken, personId, page),
        ).toBrowseDescriptor()

    override fun listGenres(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListGenresRequest(
            pagedInput(profile, accessToken, page),
        ).toBrowseDescriptor()

    override fun listGenreItems(
        profile: ServerProfile,
        accessToken: String,
        genreId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListGenreItemsRequest(
            entityPagedInput(profile, accessToken, genreId, page),
        ).toBrowseDescriptor()

    override fun listTags(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListTagsRequest(
            pagedInput(profile, accessToken, page),
        ).toBrowseDescriptor()

    override fun listTagItems(
        profile: ServerProfile,
        accessToken: String,
        tagId: String,
        page: PageRequest,
    ): BrowseRequestDescriptor =
        uniffi.nako_client_uniffi.buildListTagItemsRequest(
            entityPagedInput(profile, accessToken, tagId, page),
        ).toBrowseDescriptor()
}

internal data class BrowseRequestDescriptor(
    val request: NakoHttpRequest,
)

private fun pagedInput(
    profile: ServerProfile,
    accessToken: String,
    page: PageRequest,
): CoreBrowsePagedRequestInput =
    CoreBrowsePagedRequestInput(
        baseUrl = profile.baseUrl,
        accessToken = accessToken,
        page = page.toCore(),
    )

private fun entityInput(
    profile: ServerProfile,
    accessToken: String,
    id: String,
): CoreBrowseEntityRequestInput =
    CoreBrowseEntityRequestInput(
        baseUrl = profile.baseUrl,
        accessToken = accessToken,
        id = id,
    )

private fun entityPagedInput(
    profile: ServerProfile,
    accessToken: String,
    id: String,
    page: PageRequest,
): CoreBrowseEntityPagedRequestInput =
    CoreBrowseEntityPagedRequestInput(
        baseUrl = profile.baseUrl,
        accessToken = accessToken,
        id = id,
        page = page.toCore(),
    )

private fun PageRequest.toCore(): CorePageQuery =
    CorePageQuery(limit = limit.toUInt(), offset = offset.toULong())

private fun CoreHttpRequest.toBrowseDescriptor(): BrowseRequestDescriptor =
    BrowseRequestDescriptor(request = toAndroidRequest())
