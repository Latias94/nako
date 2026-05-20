package dev.taru.android.ui.browse

import dev.taru.android.browse.BrowseFailureCategory
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import dev.taru.android.playback.InMemoryPlaybackPreferencesStore
import dev.taru.android.playback.PlaybackFailureCategory
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class ClientBrowseDataSourceTest {
    @Test
    fun `detail source probe and playback decision fail safely without access token`() = runBlocking {
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
                    {
                      "id": "night-harbor",
                      "kind": "movie",
                      "metadata": {
                        "title": "Night Harbor",
                        "genres": [],
                        "tags": [],
                        "ratings": []
                      }
                    }
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
        headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
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
