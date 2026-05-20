package dev.taru.android.browse

import dev.taru.android.connection.ConnectionCheckResult
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileRepository
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.connection.TaruPublicApiContract
import java.io.IOException
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class TaruBrowseClientTest {
    @Test
    fun `list libraries decodes page and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "libraries": [
                        {
                          "id": "library-1",
                          "name": "Movies",
                          "roots": ["file:///srv/media/movies"],
                          "options": {
                            "domain": "video",
                            "preset": "movies",
                            "scan": {"realtime_monitor": true}
                          }
                        }
                      ],
                      "page": {"limit": 20, "offset": 40, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            page = PageRequest(limit = 20, offset = 40),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/libraries?limit=20&offset=40", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Movies", success.value.libraries.single().name)
        assertEquals("movies", success.value.libraries.single().options?.preset)
        assertEquals(40L, success.value.page.offset)
        assertFalse(success.request.toString().contains("secret-token"))
    }

    @Test
    fun `library detail and sources use active profile and decode safe source inventory`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "library": {
                        "id": "library 1",
                        "name": "Movies",
                        "roots": ["file:///srv/media/movies"],
                        "options": {
                          "domain": "video",
                          "preset": "movies"
                        }
                      }
                    }
                    """.trimIndent(),
                ),
            ),
            ResponseStep(
                ok(
                    """
                    {
                      "library": {
                        "id": "library 1",
                        "name": "Movies",
                        "roots": ["file:///srv/media/movies"],
                        "options": {"domain": "video", "preset": "movies"}
                      },
                      "sources": [
                        {
                          "source": {
                            "id": "source 1",
                            "library_id": "library 1",
                            "item_id": "item 1",
                            "file_name": "Night Harbor.mp4",
                            "size_bytes": 2097152,
                            "fingerprint": "hash-source"
                          },
                          "item": {
                            "id": "item 1",
                            "kind": "movie",
                            "metadata": {
                              "title": "Night Harbor",
                              "genres": [],
                              "tags": [],
                              "ratings": []
                            }
                          },
                          "probe": {
                            "duration_ms": 120000,
                            "container": "mp4",
                            "bit_rate": 4200000,
                            "streams": [
                              {"index":0,"kind":"video","codec":"h264","width":1920,"height":1080},
                              {"index":1,"kind":"audio","codec":"aac","channels":2}
                            ]
                          }
                        }
                      ],
                      "page": {"limit": 10, "offset": 20, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val detail = client.libraryDetail(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            libraryId = "library 1",
        )
        val sources = client.librarySources(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            libraryId = "library 1",
            page = PageRequest(limit = 10, offset = 20),
        )

        assertTrue(detail is BrowseResult.Success)
        assertTrue(sources is BrowseResult.Success)
        detail as BrowseResult.Success
        sources as BrowseResult.Success
        assertEquals(
            listOf(
                "http://home.example.test/api/libraries/library%201",
                "http://home.example.test/api/libraries/library%201/sources?limit=10&offset=20",
            ),
            transport.requests.map { it.url },
        )
        assertEquals("Bearer <redacted>", detail.request.headers["Authorization"])
        assertEquals("Bearer <redacted>", sources.request.headers["Authorization"])
        assertEquals("Movies", detail.value.library.name)
        assertEquals("Night Harbor.mp4", sources.value.sources.single().source.fileName)
        assertEquals("Night Harbor", sources.value.sources.single().item?.metadata?.title)
        assertEquals("mp4", sources.value.sources.single().probe?.container)
        assertEquals(20L, sources.value.page.offset)
        assertFalse(detail.toString().contains("secret-token"))
        assertFalse(sources.toString().contains("secret-token"))
        assertFalse(sources.toString().contains("file:///srv"))
    }

    @Test
    fun `blank library routes fail locally without transport`() = runBlocking {
        val transport = FakeTransport()
        val client = TaruBrowseClient(transport)

        val detail = client.libraryDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            libraryId = " ",
        )
        val sources = client.librarySources(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            libraryId = " ",
        )

        assertTrue(detail is BrowseResult.Failure)
        assertTrue(sources is BrowseResult.Failure)
        assertEquals(BrowseFailureCategory.MissingLibrary, (detail as BrowseResult.Failure).diagnostics.category)
        assertEquals(BrowseFailureCategory.MissingLibrary, (sources as BrowseResult.Failure).diagnostics.category)
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `list items decodes minimal media item tracer`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "items": [
                        {
                          "id": "item-1",
                          "kind": "movie",
                          "parent_id": null,
                          "metadata": {
                            "title": "Arrival",
                            "release_date": "2016-11-11",
                            "runtime_minutes": 116,
                            "genres": ["Science Fiction"],
                            "tags": [],
                            "ratings": [],
                            "images": []
                          }
                        }
                      ],
                      "page": {"limit": 24, "offset": 0, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listItems(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            page = PageRequest(limit = 24, offset = 0),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/items?limit=24&offset=0", transport.requests.single().url)
        assertEquals("Arrival", success.value.items.single().metadata.title)
        assertEquals("movie", success.value.items.single().kind)
        assertEquals(1, success.value.page.returned)
    }

    @Test
    fun `item detail decodes public detail response and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "item": {
                        "id": "item 1",
                        "kind": "movie",
                        "parent_id": null,
                        "metadata": {
                          "title": "Arrival",
                          "original_title": "Arrival",
                          "overview": "A linguist works with alien visitors.",
                          "release_date": "2016-11-11",
                          "runtime_minutes": 116,
                          "genres": ["Science Fiction"],
                          "tags": ["first-contact"],
                          "ratings": [{"source":"mpaa","value":"PG-13"}],
                          "images": [{"kind":"poster","uri":"taru://artwork/poster-1","provider":"local"}]
                        }
                      },
                      "sources": [
                        {
                          "id": "source-1",
                          "library_id": "library-1",
                          "item_id": "item 1",
                          "locator": "local:///srv/media/arrival.mkv",
                          "file_name": "arrival.mkv",
                          "size_bytes": 42
                        }
                      ],
                      "credits": [
                        {"item_id":"item 1","person_id":"person-1","role":"director","character":null}
                      ],
                      "genres": [{"item_id":"item 1","genre_id":"genre-1"}],
                      "tags": [{"item_id":"item 1","tag_id":"tag-1"}],
                      "collections": [],
                      "studios": [],
                      "images": [
                        {
                          "id":"image-1",
                          "owner":{"item":"item 1"},
                          "kind":"poster",
                          "url":"/images/image-1",
                          "width":1000,
                          "height":1500,
                          "language":null,
                          "media_type":"image/jpeg",
                          "etag":"hash-1"
                        }
                      ]
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.itemDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item 1",
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/items/item%201", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Arrival", success.value.item.metadata.title)
        assertEquals("A linguist works with alien visitors.", success.value.item.metadata.overview)
        assertEquals(1, success.value.sources.size)
        assertEquals(1, success.value.credits.size)
        assertEquals(1, success.value.images.size)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `item images decodes public image refs and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "item_id": "item 1",
                      "images": [
                        {
                          "id": "poster-1",
                          "owner": {"item": "item 1"},
                          "kind": "poster",
                          "url": "/images/poster-1",
                          "width": 1000,
                          "height": 1500,
                          "language": null,
                          "media_type": "image/png",
                          "etag": "hash-1"
                        }
                      ]
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.itemImages(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item 1",
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/items/item%201/images", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("item 1", success.value.itemId)
        assertEquals("/images/poster-1", success.value.images.single().url)
        assertEquals("image/png", success.value.images.single().mediaType)
        assertFalse(success.toString().contains("secret-token"))
        assertFalse(success.toString().contains("source_uri"))
    }

    @Test
    fun `search items encodes query facets pagination and decodes hits`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "hits": [
                        {
                          "item": {
                            "id": "item-1",
                            "kind": "movie",
                            "metadata": {
                              "title": "Search Route Demo",
                              "genres": [],
                              "tags": [],
                              "ratings": [],
                              "images": []
                            }
                          },
                          "score": 0.82
                        }
                      ],
                      "page": {"limit": 12, "offset": 6, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.searchItems(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            query = SearchRequest(
                query = "route demo",
                facets = listOf("genre:test", "tag:favorite"),
                page = PageRequest(limit = 12, offset = 6),
            ),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals(
            "http://home.example.test/search?q=route+demo&facet=genre%3Atest%2Ctag%3Afavorite&limit=12&offset=6",
            transport.requests.single().url,
        )
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Search Route Demo", success.value.hits.single().item.metadata.title)
        assertEquals(0.82f, success.value.hits.single().score, 0.001f)
        assertEquals(1, success.value.page.returned)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `list genres decodes page and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "genres": [
                        {"id":"genre-1","name":"Mystery","source":"nfo"},
                        {"id":"genre 2","name":"Science Fiction","source":{"provider":"tmdb","key":"878"}}
                      ],
                      "page": {"limit": 50, "offset": 100, "returned": 2}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listGenres(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            page = PageRequest(limit = 50, offset = 100),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/api/genres?limit=50&offset=100", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Mystery", success.value.genres.first().name)
        assertEquals("genre 2", success.value.genres.last().id)
        assertEquals(100L, success.value.page.offset)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `list tags decodes page and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "tags": [
                        {"id":"tag-1","name":"Lighthouse","source":"nfo"},
                        {"id":"tag 2","name":"Staff Pick","source":{"provider":"local","key":"staff-pick"}}
                      ],
                      "page": {"limit": 50, "offset": 100, "returned": 2}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listTags(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            page = PageRequest(limit = 50, offset = 100),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/api/tags?limit=50&offset=100", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("Lighthouse", success.value.tags.first().name)
        assertEquals("tag 2", success.value.tags.last().id)
        assertEquals(100L, success.value.page.offset)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `genre items decodes facet result with facet label`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "genre": {"id":"genre-1","name":"Mystery","source":"nfo"},
                      "items": [
                        {
                          "id": "item-1",
                          "kind": "movie",
                          "metadata": {
                            "title": "Night Harbor",
                            "genres": ["Mystery"],
                            "tags": [],
                            "ratings": [],
                            "images": []
                          }
                        }
                      ],
                      "page": {"limit": 24, "offset": 0, "returned": 1}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listGenreItems(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            genreId = "genre 1",
            page = PageRequest(limit = 24, offset = 0),
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/genres/genre%201/items?limit=24&offset=0", transport.requests.single().url)
        assertEquals("Mystery", success.value.facetLabel)
        assertEquals("Night Harbor", success.value.items.single().metadata.title)
        assertEquals(1, success.value.page.returned)
    }

    @Test
    fun `tag and person item routes use active profile and safe requests`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "tag": {"id":"tag-1","name":"favorite","source":"user"},
                      "items": [],
                      "page": {"limit": 10, "offset": 0, "returned": 0}
                    }
                    """.trimIndent(),
                ),
            ),
            ResponseStep(
                ok(
                    """
                    {
                      "person": {"id":"person-1","name":"Demo Actor","sort_name":null,"overview":null,"external_ids":[]},
                      "items": [],
                      "page": {"limit": 10, "offset": 10, "returned": 0}
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)
        val currentProfile = profile("http://home.example.test")

        val tagResult = client.listTagItems(
            profile = currentProfile,
            accessToken = "secret-token",
            tagId = "tag-1",
            page = PageRequest(limit = 10, offset = 0),
        )
        val personResult = client.listPersonItems(
            profile = currentProfile,
            accessToken = "secret-token",
            personId = "person-1",
            page = PageRequest(limit = 10, offset = 10),
        )

        assertTrue(tagResult is BrowseResult.Success)
        assertTrue(personResult is BrowseResult.Success)
        assertEquals(
            listOf(
                "http://home.example.test/tags/tag-1/items?limit=10&offset=0",
                "http://home.example.test/people/person-1/items?limit=10&offset=10",
            ),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer secret-token", "Bearer secret-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
        assertEquals("favorite", (tagResult as BrowseResult.Success).value.facetLabel)
        assertEquals("Demo Actor", (personResult as BrowseResult.Success).value.facetLabel)
        assertFalse(tagResult.toString().contains("secret-token"))
        assertFalse(personResult.toString().contains("secret-token"))
    }

    @Test
    fun `person detail decodes public person response and redacts safe request`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok(
                    """
                    {
                      "person": {
                        "id": "person 1",
                        "name": "Demo Actor",
                        "sort_name": "Actor, Demo",
                        "overview": "Keeps the lighthouse.",
                        "external_ids": [
                          {"provider":"tmdb","value":"42"}
                        ]
                      }
                    }
                    """.trimIndent(),
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.personDetail(
            profile = profile("http://home.example.test/api"),
            accessToken = "secret-token",
            personId = "person 1",
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertEquals("http://home.example.test/api/people/person%201", transport.requests.single().url)
        assertEquals("Bearer secret-token", transport.requests.single().headers["Authorization"])
        assertEquals("Bearer <redacted>", success.request.headers["Authorization"])
        assertEquals("person 1", success.value.person.id)
        assertEquals("Demo Actor", success.value.person.name)
        assertEquals("Actor, Demo", success.value.person.sortName)
        assertEquals("Keeps the lighthouse.", success.value.person.overview)
        assertEquals(1, success.value.person.externalIds.size)
        assertFalse(success.toString().contains("secret-token"))
    }

    @Test
    fun `empty libraries response remains a successful empty state input`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok("""{"libraries":[],"page":{"limit":50,"offset":0,"returned":0}}"""),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Success)
        val success = result as BrowseResult.Success
        assertTrue(success.value.libraries.isEmpty())
        assertEquals(0, success.value.page.returned)
    }

    @Test
    fun `unauthorized browse response is actionable and sanitized`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 401,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"unauthorized","message":"bad token secret-token in file:///tmp/source.mkv"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.Unauthorized, diagnostics.category)
        assertEquals(401, diagnostics.statusCode)
        assertEquals("unauthorized", diagnostics.publicError?.code)
        assertEquals("bad token <redacted> in <local-path>", diagnostics.publicError?.message)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("file:///tmp"))
    }

    @Test
    fun `forbidden item detail maps to permission diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 403,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"forbidden","message":"token secret-token cannot access item"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.itemDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item-1",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.Forbidden, diagnostics.category)
        assertEquals(403, diagnostics.statusCode)
        assertEquals("forbidden", diagnostics.publicError?.code)
        assertEquals("token <redacted> cannot access item", diagnostics.publicError?.message)
        assertFalse(diagnostics.toString().contains("secret-token"))
    }

    @Test
    fun `missing item detail maps to unavailable item diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 404,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"not_found","message":"item is missing"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.itemDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item-1",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.MissingItem, diagnostics.category)
        assertEquals(404, diagnostics.statusCode)
        assertEquals("not_found", diagnostics.publicError?.code)
        assertEquals("item is missing", diagnostics.publicError?.message)
    }

    @Test
    fun `missing person detail maps to unavailable person diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 404,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"not_found","message":"person is missing"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.personDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            personId = "person-1",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.MissingPerson, diagnostics.category)
        assertEquals(404, diagnostics.statusCode)
        assertEquals("not_found", diagnostics.publicError?.code)
        assertEquals("person is missing", diagnostics.publicError?.message)
    }

    @Test
    fun `unreachable browse request returns sanitized diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ThrowStep(IOException("failed with secret-token at C:\\media\\demo.mkv")),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listItems(
            profile = profile("https://taru.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.UnreachableServer, diagnostics.category)
        assertEquals("transport_error", diagnostics.publicError?.code)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
    }

    @Test
    fun `unsupported api version on genre index is rejected`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v2")),
                    body = """{"genres":[{"id":"genre-1","name":"Mystery","source":"nfo"}],"page":{"limit":50,"offset":0,"returned":1}}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listGenres(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.UnsupportedApiVersion, diagnostics.category)
        assertEquals("v2", diagnostics.observedApiVersion)
        assertEquals("http://home.example.test/genres?limit=50&offset=0", diagnostics.request?.url)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
    }

    @Test
    fun `unsupported api version on tag index is rejected`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v2")),
                    body = """{"tags":[{"id":"tag-1","name":"Lighthouse","source":"nfo"}],"page":{"limit":50,"offset":0,"returned":1}}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listTags(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.UnsupportedApiVersion, diagnostics.category)
        assertEquals("v2", diagnostics.observedApiVersion)
        assertEquals("http://home.example.test/tags?limit=50&offset=0", diagnostics.request?.url)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
    }

    @Test
    fun `unsupported api version on item detail is rejected`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v2")),
                    body = """{"item":{"id":"item-1","kind":"movie","metadata":{"title":"Arrival"}},"sources":[],"credits":[],"genres":[],"tags":[],"collections":[],"studios":[],"images":[]}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.itemDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item-1",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.UnsupportedApiVersion, diagnostics.category)
        assertEquals("v2", diagnostics.observedApiVersion)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
    }

    @Test
    fun `unsupported api version on person detail is rejected`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 200,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v2")),
                    body = """{"person":{"id":"person-1","name":"Demo Actor","sort_name":null,"overview":null,"external_ids":[]}}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.personDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            personId = "person-1",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.UnsupportedApiVersion, diagnostics.category)
        assertEquals("v2", diagnostics.observedApiVersion)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
    }

    @Test
    fun `invalid item detail response maps to invalid response diagnostics`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                ok("""{"item":{"id":"item-1","kind":"movie","metadata":{}},"sources":[]}"""),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.itemDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = "item-1",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.InvalidResponse, diagnostics.category)
        assertEquals("Bearer <redacted>", diagnostics.request?.headers?.get("Authorization"))
        assertFalse(diagnostics.toString().contains("secret-token"))
    }

    @Test
    fun `blank item detail request fails locally without transport`() = runBlocking {
        val transport = FakeTransport()
        val client = TaruBrowseClient(transport)

        val result = client.itemDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            itemId = " ",
        )

        assertTrue(result is BrowseResult.Failure)
        assertEquals(BrowseFailureCategory.MissingItem, (result as BrowseResult.Failure).diagnostics.category)
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `blank person detail request fails locally without transport`() = runBlocking {
        val transport = FakeTransport()
        val client = TaruBrowseClient(transport)

        val result = client.personDetail(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
            personId = " ",
        )

        assertTrue(result is BrowseResult.Failure)
        assertEquals(BrowseFailureCategory.MissingPerson, (result as BrowseResult.Failure).diagnostics.category)
        assertTrue(transport.requests.isEmpty())
    }

    @Test
    fun `public api browse errors keep diagnostics client safe`() = runBlocking {
        val transport = FakeTransport(
            ResponseStep(
                TaruHttpResponse(
                    statusCode = 500,
                    headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
                    body = """{"code":"storage_error","message":"ffmpeg.exe -i C:\\media\\demo.mkv secret-token"}""",
                ),
            ),
        )
        val client = TaruBrowseClient(transport)

        val result = client.listLibraries(
            profile = profile("http://home.example.test"),
            accessToken = "secret-token",
        )

        assertTrue(result is BrowseResult.Failure)
        val diagnostics = (result as BrowseResult.Failure).diagnostics
        assertEquals(BrowseFailureCategory.PublicApiError, diagnostics.category)
        assertEquals("storage_error", diagnostics.publicError?.code)
        assertFalse(diagnostics.toString().contains("secret-token"))
        assertFalse(diagnostics.toString().contains("C:\\media"))
        assertFalse(diagnostics.toString().contains("ffmpeg.exe"))
    }

    @Test
    fun `item detail uses active profile base url and token reference`() = runBlocking {
        val repository = ServerProfileRepository()
        val vault = InMemoryTokenVault()
        val home = repository.upsertConnectedProfile(
            displayName = "Home",
            tokenReference = null,
            result = successFor("http://home.example.test"),
        )
        vault.saveToken(home.tokenReference, "home-token")
        val lab = repository.upsertConnectedProfile(
            displayName = "Lab",
            tokenReference = null,
            result = successFor("http://lab.example.test"),
        )
        vault.saveToken(lab.tokenReference, "lab-token")
        val transport = FakeTransport(
            ResponseStep(ok(detailBody("lab-item"))),
            ResponseStep(ok(detailBody("home-item"))),
        )
        val client = TaruBrowseClient(transport)

        val activeLab = repository.activeProfile() ?: error("active profile required")
        client.itemDetail(activeLab, vault.readToken(activeLab.tokenReference).orEmpty(), "lab-item")
        repository.switchActive(home.id)
        val activeHome = repository.activeProfile() ?: error("active profile required")
        client.itemDetail(activeHome, vault.readToken(activeHome.tokenReference).orEmpty(), "home-item")

        assertEquals(
            listOf(
                "http://lab.example.test/items/lab-item",
                "http://home.example.test/items/home-item",
            ),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer lab-token", "Bearer home-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
    }

    @Test
    fun `active profile switching changes browse base url and token reference`() = runBlocking {
        val repository = ServerProfileRepository()
        val vault = InMemoryTokenVault()
        val home = repository.upsertConnectedProfile(
            displayName = "Home",
            tokenReference = null,
            result = successFor("http://home.example.test"),
        )
        vault.saveToken(home.tokenReference, "home-token")
        val lab = repository.upsertConnectedProfile(
            displayName = "Lab",
            tokenReference = null,
            result = successFor("http://lab.example.test"),
        )
        vault.saveToken(lab.tokenReference, "lab-token")
        val transport = FakeTransport(
            ResponseStep(ok("""{"libraries":[],"page":{"limit":50,"offset":0,"returned":0}}""")),
            ResponseStep(ok("""{"libraries":[],"page":{"limit":50,"offset":0,"returned":0}}""")),
        )
        val client = TaruBrowseClient(transport)

        val activeLab = repository.activeProfile() ?: error("active profile required")
        client.listLibraries(activeLab, vault.readToken(activeLab.tokenReference).orEmpty())
        repository.switchActive(home.id)
        val activeHome = repository.activeProfile() ?: error("active profile required")
        client.listLibraries(activeHome, vault.readToken(activeHome.tokenReference).orEmpty())

        assertEquals(
            listOf(
                "http://lab.example.test/libraries?limit=50&offset=0",
                "http://home.example.test/libraries?limit=50&offset=0",
            ),
            transport.requests.map { it.url },
        )
        assertEquals(
            listOf("Bearer lab-token", "Bearer home-token"),
            transport.requests.map { it.headers["Authorization"] },
        )
    }

    private fun profile(baseUrl: String): ServerProfile =
        ServerProfile(
            id = "server-1",
            displayName = "Home",
            baseUrl = baseUrl,
            tokenReference = "server-token:server-1",
            lastObservedApiVersion = "v1",
        )

    private fun successFor(baseUrl: String): ConnectionCheckResult.Success =
        ConnectionCheckResult.Success(
            normalizedBaseUrl = baseUrl,
            apiVersion = "v1",
            checkedAtMillis = 42L,
            healthRequest = SafeRequestPreview("GET", "$baseUrl/health"),
            authProbeRequest = SafeRequestPreview(
                method = "GET",
                url = "$baseUrl/libraries?limit=1&offset=0",
                headers = mapOf("Authorization" to "Bearer <redacted>"),
            ),
        )

    private fun ok(body: String): TaruHttpResponse =
        TaruHttpResponse(
            statusCode = 200,
            headers = mapOf(TaruPublicApiContract.apiVersionHeader to listOf("v1")),
            body = body,
        )

    private fun detailBody(itemId: String): String =
        """{"item":{"id":"$itemId","kind":"movie","metadata":{"title":"Arrival"}},"sources":[],"credits":[],"genres":[],"tags":[],"collections":[],"studios":[],"images":[]}"""
}

private sealed interface FakeStep

private data class ResponseStep(val response: TaruHttpResponse) : FakeStep

private data class ThrowStep(val error: IOException) : FakeStep

private class FakeTransport(
    vararg steps: FakeStep,
) : TaruHttpTransport {
    private val steps = ArrayDeque(steps.toList())
    val requests = mutableListOf<TaruHttpRequest>()

    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse {
        requests += request
        return when (val step = steps.removeFirst()) {
            is ResponseStep -> step.response
            is ThrowStep -> throw step.error
        }
    }
}
