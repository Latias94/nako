package dev.nako.android.ui.browse

import dev.nako.android.browse.BrowseFacetFamily
import dev.nako.android.browse.CanonicalMetadataDto
import dev.nako.android.browse.FacetItemsResponse
import dev.nako.android.browse.ItemDetailResponse
import dev.nako.android.browse.LibraryDto
import dev.nako.android.browse.LibrarySourcesResponse
import dev.nako.android.browse.MediaItemDto
import dev.nako.android.browse.MediaSourceDto
import dev.nako.android.browse.PageInfo
import dev.nako.android.browse.PersonDto
import dev.nako.android.browse.PersonResponse
import dev.nako.android.playback.ClientPlaybackDecision
import dev.nako.android.playback.ClientPlaybackMode
import dev.nako.android.playback.PlaybackDecisionResponse
import dev.nako.android.playback.PlaybackMediaSourceDto
import dev.nako.android.playback.PlaybackRequestDescriptor
import dev.nako.android.playback.PlaybackRequestTarget
import dev.nako.android.player.playbackLaunchRequest

internal fun testPageInfo(returned: Int = 1): PageInfo =
    PageInfo(
        limit = 24,
        offset = 0,
        returned = returned,
    )

internal fun testMediaItem(itemId: String): MediaItemDto =
    MediaItemDto(
        id = itemId,
        kind = "movie",
        metadata = CanonicalMetadataDto(title = "Night Harbor"),
    )

internal fun testDetailResponse(
    itemId: String,
    sourceIds: List<String>,
): ItemDetailResponse =
    ItemDetailResponse(
        item = testMediaItem(itemId),
        sources = sourceIds.map { sourceId ->
            MediaSourceDto(
                id = sourceId,
                libraryId = "library-movies",
                itemId = itemId,
            )
        },
    )

internal fun testLibrarySourcesState(libraryId: String): LibrarySourcesResponse =
    LibrarySourcesResponse(
        library = LibraryDto(
            id = libraryId,
            name = "Movies",
        ),
        page = testPageInfo(returned = 0),
    )

internal fun testPersonResponse(personId: String): PersonResponse =
    PersonResponse(
        person = PersonDto(
            id = personId,
            name = "Demo Actor",
        ),
    )

internal fun testFacetItems(facetId: String): FacetItemsResponse =
    FacetItemsResponse(
        family = BrowseFacetFamily.Person,
        facetId = facetId,
        facetLabel = "Demo Actor",
        items = listOf(testMediaItem("night-harbor")),
        page = testPageInfo(),
    )

internal fun testRelationshipContent(family: RelationshipIndexFamily): RelationshipIndexUiState.Content =
    RelationshipIndexUiState.Content(
        family = family,
        rows = emptyList(),
        page = testPageInfo(returned = 0),
    )

internal fun testPlaybackDecisionFixture(sourceId: String): PlaybackDecisionResponse =
    PlaybackDecisionResponse(
        source = PlaybackMediaSourceDto(
            id = sourceId,
            libraryId = "library-movies",
            itemId = "night-harbor",
        ),
        decision = ClientPlaybackDecision(
            mode = ClientPlaybackMode.DirectPlay,
            reason = "direct",
        ),
    )

internal fun testPlaybackTargetFixture(sourceId: String): PlaybackRequestTarget =
    PlaybackRequestTarget(
        request = PlaybackRequestDescriptor(
            method = "GET",
            url = "http://127.0.0.1:3018/sources/$sourceId/stream",
        ),
    )

internal fun testPlaybackLaunchFixture() =
    playbackLaunchRequest(
        title = "Night Harbor",
        target = testPlaybackTargetFixture("source-1"),
        serverProfileId = "server-1",
        mediaItemId = "night-harbor",
        sourceId = "source-1",
        playbackMode = ClientPlaybackMode.DirectPlay,
    )
