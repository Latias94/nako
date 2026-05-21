package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.PageRequest
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.playback.InMemoryPlaybackPreferencesStore
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import dev.taru.sdk.TARU_API_VERSION_HEADER

class ClientBrowseDataSourceTest {
    @Test
    fun `detail version probe and playback decision fail safely without saved access`() = runBlocking {
        val transport = RecordingTransport()
        val dataSource = ClientBrowseDataSource(
            profile = testProfile(),
            tokenVault = InMemoryTokenVault(),
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            userPlaybackClient = TaruUserPlaybackClient(transport),
        )

        val detailState = dataSource.loadItemDetail("night-harbor")
        val relationshipIndexState = dataSource.loadRelationshipIndex(RelationshipIndexFamily.Genres)
        val probeState = dataSource.loadSourceProbe("source-a")
        val playbackState = dataSource.loadPlaybackSelection("source-a")

        assertEquals(
            BrowseFailureCategory.MissingAccessToken,
            (detailState as ItemDetailUiState.Failure).diagnostics.category,
        )
        assertEquals(
            BrowseFailureCategory.MissingAccessToken,
            (relationshipIndexState as RelationshipIndexUiState.Failure).diagnostics.category,
        )
        assertEquals(
            PlaybackFailureCategory.MissingAccessToken,
            (probeState as SourceProbeUiState.Failure).diagnostics.category,
        )
        assertEquals(
            PlaybackFailureCategory.MissingAccessToken,
            (playbackState as PlaybackSelectionUiState.Failure).diagnostics.category,
        )
        assertEquals(emptyList<TaruHttpRequest>(), transport.requests)
    }

    @Test
    fun `genre relationship index loads public genre list into facet targets`() = runBlocking {
        val transport = QueuedTransport(
            ok(
                """
                {
                  "genres": [
                    {"id": "genre-mystery", "name": "Mystery", "source": "nfo"},
                    {"id": "genre-empty-name", "name": "", "source": "nfo"}
                  ],
                  "page": {"limit": 50, "offset": 0, "returned": 2}
                }
                """.trimIndent(),
            ),
        )
        val vault = InMemoryTokenVault()
        val profile = testProfile()
        vault.saveToken(profile.tokenReference, "secret-token")
        val dataSource = ClientBrowseDataSource(
            profile = profile,
            tokenVault = vault,
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            userPlaybackClient = TaruUserPlaybackClient(transport),
        )

        val state = dataSource.loadRelationshipIndex(RelationshipIndexFamily.Genres)

        val content = state as RelationshipIndexUiState.Content
        val row = content.rows.single()
        assertEquals(RelationshipIndexFamily.Genres, content.family)
        assertEquals("Mystery", row.title)
        assertEquals("Genre", row.subtitle)
        assertEquals(BrowseFacetUiFamily.Genre, row.target.family)
        assertEquals("genre-mystery", row.target.id)
        assertEquals(
            listOf("http://127.0.0.1:3018/genres?limit=50&offset=0"),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer secret-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
    }

    @Test
    fun `tag relationship index loads public tag list into facet targets`() = runBlocking {
        val transport = QueuedTransport(
            ok(
                """
                {
                  "tags": [
                    {"id": "tag-lighthouse", "name": "Lighthouse", "source": "nfo"},
                    {"id": "tag-empty-name", "name": "", "source": "nfo"}
                  ],
                  "page": {"limit": 50, "offset": 0, "returned": 2}
                }
                """.trimIndent(),
            ),
        )
        val vault = InMemoryTokenVault()
        val profile = testProfile()
        vault.saveToken(profile.tokenReference, "secret-token")
        val dataSource = ClientBrowseDataSource(
            profile = profile,
            tokenVault = vault,
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            userPlaybackClient = TaruUserPlaybackClient(transport),
        )

        val state = dataSource.loadRelationshipIndex(RelationshipIndexFamily.Tags)

        val content = state as RelationshipIndexUiState.Content
        val row = content.rows.single()
        assertEquals(RelationshipIndexFamily.Tags, content.family)
        assertEquals("Lighthouse", row.title)
        assertEquals("Tag", row.subtitle)
        assertEquals(BrowseFacetUiFamily.Tag, row.target.family)
        assertEquals("tag-lighthouse", row.target.id)
        assertEquals(
            listOf("http://127.0.0.1:3018/tags?limit=50&offset=0"),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer secret-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
    }

    @Test
    fun `person detail loads person and related media items through public client routes`() = runBlocking {
        val transport = QueuedTransport(
            ok(
                """
                {
                  "person": {
                    "id": "person 1",
                    "name": "Demo Actor",
                    "sort_name": "Actor, Demo",
                    "overview": "Keeps the lighthouse.",
                    "external_ids": []
                  }
                }
                """.trimIndent(),
            ),
            ok(
                """
                {
                  "person": {
                    "id": "person 1",
                    "name": "Demo Actor",
                    "sort_name": "Actor, Demo",
                    "overview": "Keeps the lighthouse.",
                    "external_ids": []
                  },
                  "items": [
                    ${mediaItemJson(id = "night-harbor", title = "Night Harbor").prependIndent("                    ")}
                  ],
                  "page": {"limit": 24, "offset": 0, "returned": 1}
                }
                """.trimIndent(),
            ),
        )
        val vault = InMemoryTokenVault()
        val profile = testProfile()
        vault.saveToken(profile.tokenReference, "secret-token")
        val dataSource = ClientBrowseDataSource(
            profile = profile,
            tokenVault = vault,
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            userPlaybackClient = TaruUserPlaybackClient(transport),
        )

        val state = dataSource.loadPersonDetail("person 1")

        val content = state as PersonDetailUiState.Content
        assertEquals("Demo Actor", content.response.person.name)
        assertEquals("Night Harbor", content.relatedItems.items.single().metadata.title)
        assertEquals(
            listOf(
                "http://127.0.0.1:3018/people/person%201",
                "http://127.0.0.1:3018/people/person%201/items?limit=24&offset=0",
            ),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer secret-token", "Bearer secret-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
    }

    @Test
    fun `search and facet data source preserve explicit paging requests`() = runBlocking {
        val target = BrowseFacetTarget(
            family = BrowseFacetUiFamily.Genre,
            label = "Mystery",
            id = "genre-mystery",
        )
        val transport = QueuedTransport(
            ok(
                """
                {
                  "hits": [],
                  "page": {"limit": 24, "offset": 48, "returned": 0}
                }
                """.trimIndent(),
            ),
            ok(
                """
                {
                  "genre": {"id":"genre-mystery","name":"Mystery","source":"nfo"},
                  "items": [],
                  "page": {"limit": 24, "offset": 72, "returned": 0}
                }
                """.trimIndent(),
            ),
        )
        val vault = InMemoryTokenVault()
        val profile = testProfile()
        vault.saveToken(profile.tokenReference, "secret-token")
        val dataSource = ClientBrowseDataSource(
            profile = profile,
            tokenVault = vault,
            browseClient = TaruBrowseClient(transport),
            playbackClient = TaruPlaybackClient(transport),
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            userPlaybackClient = TaruUserPlaybackClient(transport),
        )

        val searchState = dataSource.search("night harbor", page = PageRequest(limit = 24, offset = 48))
        val facetState = dataSource.loadFacet(target, page = PageRequest(limit = 24, offset = 72))

        assertEquals(0, (searchState as SearchUiState.Content).response.page.returned)
        assertEquals(0, (facetState as FacetUiState.Content).response.page.returned)
        assertEquals(
            listOf(
                "http://127.0.0.1:3018/search?q=night+harbor&limit=24&offset=48",
                "http://127.0.0.1:3018/genres/genre-mystery/items?limit=24&offset=72",
            ),
            transport.requests.map { it.url },
        )
    }

    private fun mediaItemJson(
        id: String,
        title: String,
    ): String =
        """
        {
          "id": "$id",
          "kind": "movie",
          "parent_id": null,
          "metadata": {
            "title": "$title",
            "original_title": null,
            "sort_title": null,
            "overview": null,
            "release_date": null,
            "runtime_minutes": null,
            "tagline": null,
            "genres": [],
            "tags": [],
            "ratings": [],
            "credits": [],
            "collections": [],
            "studios": [],
            "external_ids": []
          }
        }
        """.trimIndent()
}

private class RecordingTransport : TaruHttpTransport {
    val requests: MutableList<TaruHttpRequest> = mutableListOf()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return TaruHttpResponse(statusCode = 500)
    }
}

private class QueuedTransport(
    vararg responses: TaruHttpResponse,
) : TaruHttpTransport {
    private val responses = ArrayDeque(responses.toList())
    val requests: MutableList<TaruHttpRequest> = mutableListOf()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return responses.removeFirst()
    }
}

private fun ok(body: String): TaruHttpResponse =
    TaruHttpResponse(
        statusCode = 200,
        headers = mapOf(TARU_API_VERSION_HEADER to listOf("v1")),
        body = body,
    )

private fun testProfile(): ServerProfile =
    ServerProfile(
        id = "server-1",
        displayName = "Home",
        baseUrl = "http://127.0.0.1:3018",
        tokenReference = "server-token:server-1",
        lastObservedApiVersion = "v1",
    )
