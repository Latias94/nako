package dev.nako.android.browse

import dev.nako.android.media.toAndroid
import dev.nako.sdk.CanonicalMetadataDto as SdkCanonicalMetadataDto
import dev.nako.sdk.CollectionItemDto as SdkCollectionItemDto
import dev.nako.sdk.ContentRatingDto as SdkContentRatingDto
import dev.nako.sdk.ExternalIdDto as SdkExternalIdDto
import dev.nako.sdk.GenreDto as SdkGenreDto
import dev.nako.sdk.GenreItemsResponse as SdkGenreItemsResponse
import dev.nako.sdk.GenreListResponse as SdkGenreListResponse
import dev.nako.sdk.ImagesResponse as SdkImagesResponse
import dev.nako.sdk.ItemCreditDto as SdkItemCreditDto
import dev.nako.sdk.ItemDetailResponse as SdkItemDetailResponse
import dev.nako.sdk.ItemGenreDto as SdkItemGenreDto
import dev.nako.sdk.ItemStudioDto as SdkItemStudioDto
import dev.nako.sdk.ItemTagDto as SdkItemTagDto
import dev.nako.sdk.ItemsResponse as SdkItemsResponse
import dev.nako.sdk.LibraryDto as SdkLibraryDto
import dev.nako.sdk.LibraryListResponse as SdkLibraryListResponse
import dev.nako.sdk.LibraryOptionsDto as SdkLibraryOptionsDto
import dev.nako.sdk.LibraryResponse as SdkLibraryResponse
import dev.nako.sdk.LibrarySourceResponse as SdkLibrarySourceResponse
import dev.nako.sdk.LibrarySourcesResponse as SdkLibrarySourcesResponse
import dev.nako.sdk.MediaItemDto as SdkMediaItemDto
import dev.nako.sdk.MediaSourceDto as SdkMediaSourceDto
import dev.nako.sdk.PageInfo as SdkPageInfo
import dev.nako.sdk.PersonDto as SdkPersonDto
import dev.nako.sdk.PersonItemsResponse as SdkPersonItemsResponse
import dev.nako.sdk.PersonResponse as SdkPersonResponse
import dev.nako.sdk.PublicImageRefDto as SdkPublicImageRefDto
import dev.nako.sdk.SearchItemHit as SdkSearchItemHit
import dev.nako.sdk.SearchResponse as SdkSearchResponse
import dev.nako.sdk.TagDto as SdkTagDto
import dev.nako.sdk.TagItemsResponse as SdkTagItemsResponse
import dev.nako.sdk.TagsResponse as SdkTagsResponse

internal fun SdkLibraryListResponse.toAndroid(): LibraryListResponse =
    LibraryListResponse(
        libraries = libraries.map(SdkLibraryDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkLibraryResponse.toAndroid(): LibraryResponse =
    LibraryResponse(
        library = library.toAndroid(),
    )

internal fun SdkLibrarySourcesResponse.toAndroid(): LibrarySourcesResponse =
    LibrarySourcesResponse(
        library = library.toAndroid(),
        sources = sources.map(SdkLibrarySourceResponse::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkLibrarySourceResponse.toAndroid(): LibrarySourceResponse =
    LibrarySourceResponse(
        source = source.toAndroid(),
        item = item?.toAndroid(),
        probe = probe?.toAndroid(),
    )

internal fun SdkLibraryDto.toAndroid(): LibraryDto =
    LibraryDto(
        id = id,
        name = name,
        roots = roots,
        options = options.toAndroid(),
    )

internal fun SdkLibraryOptionsDto.toAndroid(): LibraryOptionsDto =
    LibraryOptionsDto(
        domain = domain.wireValue,
        preset = preset.wireValue,
    )

internal fun SdkItemsResponse.toAndroid(): ItemsResponse =
    ItemsResponse(
        items = items.map(SdkMediaItemDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkSearchResponse.toAndroid(): SearchResponse =
    SearchResponse(
        hits = hits.map(SdkSearchItemHit::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkSearchItemHit.toAndroid(): SearchItemHit =
    SearchItemHit(
        item = item.toAndroid(),
        score = score,
    )

internal fun SdkGenreListResponse.toAndroid(): GenreListResponse =
    GenreListResponse(
        genres = genres.map(SdkGenreDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkTagsResponse.toAndroid(): TagListResponse =
    TagListResponse(
        tags = tags.map(SdkTagDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkGenreItemsResponse.toAndroid(): GenreItemsResponse =
    GenreItemsResponse(
        genre = genre.toAndroid(),
        items = items.map(SdkMediaItemDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkTagItemsResponse.toAndroid(): TagItemsResponse =
    TagItemsResponse(
        tag = tag.toAndroid(),
        items = items.map(SdkMediaItemDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkPersonItemsResponse.toAndroid(): PersonItemsResponse =
    PersonItemsResponse(
        person = person.toAndroid(),
        items = items.map(SdkMediaItemDto::toAndroid),
        page = page.toAndroid(),
    )

internal fun SdkPersonResponse.toAndroid(): PersonResponse =
    PersonResponse(
        person = person.toAndroid(),
    )

internal fun SdkItemDetailResponse.toAndroid(): ItemDetailResponse =
    ItemDetailResponse(
        item = item.toAndroid(),
        sources = sources.map(SdkMediaSourceDto::toAndroid),
        credits = credits.map(SdkItemCreditDto::toAndroid),
        genres = genres.map(SdkItemGenreDto::toAndroid),
        tags = tags.map(SdkItemTagDto::toAndroid),
        collections = collections.map(SdkCollectionItemDto::toAndroid),
        studios = studios.map(SdkItemStudioDto::toAndroid),
        images = images.map(SdkPublicImageRefDto::toAndroid),
    )

internal fun SdkImagesResponse.toAndroid(): ImagesResponse =
    ImagesResponse(
        itemId = itemId,
        images = images.map(SdkPublicImageRefDto::toAndroid),
    )

internal fun SdkMediaItemDto.toAndroid(): MediaItemDto =
    MediaItemDto(
        id = id,
        kind = kind.wireValue,
        parentId = parentId,
        metadata = metadata.toAndroid(),
    )

internal fun SdkCanonicalMetadataDto.toAndroid(): CanonicalMetadataDto =
    CanonicalMetadataDto(
        title = title,
        originalTitle = originalTitle,
        sortTitle = sortTitle,
        overview = overview,
        releaseDate = releaseDate,
        runtimeMinutes = runtimeMinutes,
        tagline = tagline,
        genres = genres,
        tags = tags,
        ratings = ratings.map(SdkContentRatingDto::toAndroid),
    )

internal fun SdkContentRatingDto.toAndroid(): ContentRatingDto =
    ContentRatingDto(
        source = source,
        value = value,
    )

internal fun SdkMediaSourceDto.toAndroid(): MediaSourceDto =
    MediaSourceDto(
        id = id,
        libraryId = libraryId,
        itemId = itemId,
        fileName = fileName,
        sizeBytes = sizeBytes,
        fingerprint = fingerprint,
    )

internal fun SdkGenreDto.toAndroid(): GenreDto =
    GenreDto(
        id = id,
        name = name,
        source = source,
    )

internal fun SdkTagDto.toAndroid(): TagDto =
    TagDto(
        id = id,
        name = name,
        source = source,
    )

internal fun SdkPersonDto.toAndroid(): PersonDto =
    PersonDto(
        id = id,
        name = name,
        sortName = sortName,
        overview = overview,
        externalIds = externalIds.map(SdkExternalIdDto::toAndroid),
    )

internal fun SdkExternalIdDto.toAndroid(): ExternalIdDto =
    ExternalIdDto(
        provider = provider,
        value = value,
    )

internal fun SdkItemCreditDto.toAndroid(): ItemCreditDto =
    ItemCreditDto(
        itemId = itemId,
        personId = personId,
        role = role,
        character = character,
        sortOrder = sortOrder,
    )

internal fun SdkItemGenreDto.toAndroid(): ItemGenreDto =
    ItemGenreDto(
        itemId = itemId,
        genreId = genreId,
    )

internal fun SdkItemTagDto.toAndroid(): ItemTagDto =
    ItemTagDto(
        itemId = itemId,
        tagId = tagId,
    )

internal fun SdkCollectionItemDto.toAndroid(): CollectionItemDto =
    CollectionItemDto(
        collectionId = collectionId,
        itemId = itemId,
        sortOrder = sortOrder,
    )

internal fun SdkItemStudioDto.toAndroid(): ItemStudioDto =
    ItemStudioDto(
        itemId = itemId,
        studioId = studioId,
    )

internal fun SdkPublicImageRefDto.toAndroid(): PublicImageRefDto =
    PublicImageRefDto(
        id = id,
        owner = owner,
        kind = kind,
        url = url,
        width = width,
        height = height,
        language = language,
        mediaType = mediaType,
        etag = etag,
    )

internal fun SdkPageInfo.toAndroid(): PageInfo =
    PageInfo(
        limit = limit,
        offset = offset,
        returned = returned,
    )
