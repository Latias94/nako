package dev.nako.sdk

import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class NakoClientSdkTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun exposesPublicApiVersionConstantsAndPaths() {
        assertEquals("v1", NAKO_API_VERSION)
        assertEquals("x-nako-api-version", NAKO_API_VERSION_HEADER)
        assertEquals("x-nako-playback-session-id", NAKO_PLAYBACK_SESSION_ID_HEADER)
        assertTrue("/health" in NAKO_PUBLIC_PATHS)
        assertTrue("/libraries" in NAKO_PUBLIC_PATHS)
        assertTrue("/users/me/playback-state/continue-watching" in NAKO_PUBLIC_PATHS)
        assertFalse(NAKO_PUBLIC_PATHS.any { it.startsWith("/admin") })
    }

    @Test
    fun buildsGeneratedHealthAndLibraryRequestDescriptors() {
        val health = NakoPublicClientRequests.health()
        val authProbe = NakoPublicClientRequests.listLibraries(PageQuery(limit = 1, offset = 0))
        val browsePage = NakoPublicClientRequests.listLibraries(PageQuery(limit = 20, offset = 40))

        assertEquals("GET", health.method)
        assertEquals("/health", health.pathAndQuery)
        assertFalse(health.authRequired)
        assertEquals("GET", authProbe.method)
        assertEquals("/libraries?limit=1&offset=0", authProbe.pathAndQuery)
        assertTrue(authProbe.authRequired)
        assertEquals("/libraries?limit=20&offset=40", browsePage.pathAndQuery)
        assertEquals("title%20with%20space", NakoPublicClientRequests.encodePathSegment("title with space"))
        assertEquals("/libraries/library%201", NakoPublicClientRequests.getLibrary("library 1").pathAndQuery)
        assertEquals(
            "/libraries/library%201/sources?limit=10&offset=20",
            NakoPublicClientRequests.listLibrarySources("library 1", PageQuery(limit = 10, offset = 20))
                .pathAndQuery,
        )
        assertEquals("/items?limit=24&offset=0", NakoPublicClientRequests.listItems(PageQuery(limit = 24, offset = 0)).pathAndQuery)
        assertEquals("/items/item%201", NakoPublicClientRequests.getItem("item 1").pathAndQuery)
        assertEquals("/items/item%201/images", NakoPublicClientRequests.listItemImages("item 1").pathAndQuery)
        assertEquals("/images/image%201", NakoPublicClientRequests.image("image 1").pathAndQuery)
        assertEquals(
            "/images/image%201?width=320&height=180",
            NakoPublicClientRequests.image("image 1", ImageVariantQuery(width = 320, height = 180))
                .pathAndQuery,
        )
        assertEquals("HEAD", NakoPublicClientRequests.headImage("image 1").method)
        assertEquals("/people/person%201", NakoPublicClientRequests.getPerson("person 1").pathAndQuery)
        assertEquals(
            "/people/person%201/items?limit=24&offset=0",
            NakoPublicClientRequests.listPersonItems("person 1", PageQuery(limit = 24, offset = 0)).pathAndQuery,
        )
        assertEquals("/genres?limit=50&offset=100", NakoPublicClientRequests.listGenres(PageQuery(limit = 50, offset = 100)).pathAndQuery)
        assertEquals(
            "/genres/genre%201/items?limit=24&offset=0",
            NakoPublicClientRequests.listGenreItems("genre 1", PageQuery(limit = 24, offset = 0)).pathAndQuery,
        )
        assertEquals("/tags?limit=50&offset=100", NakoPublicClientRequests.listTags(PageQuery(limit = 50, offset = 100)).pathAndQuery)
        assertEquals(
            "/tags/tag%201/items?limit=24&offset=0",
            NakoPublicClientRequests.listTagItems("tag 1", PageQuery(limit = 24, offset = 0)).pathAndQuery,
        )
        assertEquals(
            "/search?q=route+demo&facet=genre%3Atest%2Ctag%3Afavorite&limit=12&offset=6",
            NakoPublicClientRequests.searchItems(
                query = "route demo",
                facets = listOf("genre:test", "tag:favorite"),
                page = PageQuery(limit = 12, offset = 6),
            ).pathAndQuery,
        )
        assertEquals("/sources/source%201/probe", NakoPublicClientRequests.getSourceProbe("source 1").pathAndQuery)
        assertEquals(
            "/sources/source%201/playback/decision?direct_play=true&container=mp4%2Cwebm&video_codec=h264&audio_codec=aac%2Copus",
            NakoPublicClientRequests.getSourcePlaybackDecision(
                sourceId = "source 1",
                capabilities = PlaybackCapabilitiesQuery(
                    directPlay = true,
                    containers = listOf("mp4", "webm"),
                    videoCodecs = listOf("h264"),
                    audioCodecs = listOf("aac", "opus"),
                ),
            ).pathAndQuery,
        )
        assertEquals("/sources/source%201/stream", NakoPublicClientRequests.streamSource("source 1").pathAndQuery)
        assertEquals("HEAD", NakoPublicClientRequests.headStreamSource("source 1").method)
        assertEquals(
            "/sources/source%201/stream/remux?direct_play=false&container=mp4%2Cmkv&video_codec=h264&audio_codec=aac&output_container=mkv",
            NakoPublicClientRequests.remuxStreamSource(
                sourceId = "source 1",
                query = RemuxPlaybackQuery(
                    directPlay = false,
                    containers = listOf("mp4", "mkv"),
                    videoCodecs = listOf("h264"),
                    audioCodecs = listOf("aac"),
                    outputContainer = RemuxOutputContainer.Mkv,
                ),
            ).pathAndQuery,
        )
        assertEquals("HEAD", NakoPublicClientRequests.headRemuxStreamSource("source 1").method)
        assertEquals(
            "/sources/source%201/stream/hls/playlist.m3u8?container=hls&video_codec=h264",
            NakoPublicClientRequests.hlsPlaylistSource(
                sourceId = "source 1",
                capabilities = PlaybackCapabilitiesQuery(
                    containers = listOf("hls"),
                    videoCodecs = listOf("h264"),
                ),
            ).pathAndQuery,
        )
        assertEquals("/playback/sessions/session%201", NakoPublicClientRequests.getPlaybackSession("session 1").pathAndQuery)
        assertEquals("POST", NakoPublicClientRequests.cancelPlaybackSession("session 1").method)
        assertEquals(
            "/playback/sessions/session%201/hls/segments/seg%20001.ts",
            NakoPublicClientRequests.hlsSegment("session 1", "seg 001.ts").pathAndQuery,
        )
        assertEquals(
            "/users/me/playback-state/items/item%201",
            NakoPublicClientRequests.getUserPlaybackState("item 1").pathAndQuery,
        )
        assertEquals(
            "/users/me/playback-state/continue-watching?limit=12&offset=0",
            NakoPublicClientRequests.listContinueWatching(PageQuery(limit = 12, offset = 0)).pathAndQuery,
        )
        assertEquals(
            "/users/me/playback-state/items/item%201/progress",
            NakoPublicClientRequests.updateUserPlaybackProgress("item 1").pathAndQuery,
        )
        assertEquals(
            "/users/me/playback-state/items/item%201/watched",
            NakoPublicClientRequests.setUserWatchedState("item 1").pathAndQuery,
        )
    }

    @Test
    fun decodesHealthEnvelopeWithGeneratedWireValue() {
        val health = json.decodeFromString<HealthResponse>(
            """{"status":"ok","version":"v1"}""",
        )

        assertEquals("ok", health.status)
        assertEquals(HealthResponseVersion.V1, health.version)
        assertEquals("v1", health.version.wireValue)
        assertTrue(health.version.isKnown)
    }

    @Test
    fun decodesUnknownPublicWireValuesWithoutLosingRawValue() {
        val health = json.decodeFromString<HealthResponse>(
            """{"status":"ok","version":"v2"}""",
        )
        val error = json.decodeFromString<ErrorResponse>(
            """{"code":"rate_limited","message":"wait before retrying"}""",
        )

        assertEquals("v2", health.version.wireValue)
        assertFalse(health.version.isKnown)
        assertEquals("rate_limited", error.code.wireValue)
        assertFalse(error.code.isKnown)
        assertEquals(
            """{"status":"ok","version":"v2"}""",
            json.encodeToString(HealthResponse(status = "ok", version = HealthResponseVersion("v2"))),
        )
    }

    @Test
    fun decodesLibraryListWithGeneratedWireTypes() {
        val libraries = json.decodeFromString<LibraryListResponse>(
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
                    "naming_strategy": "movie",
                    "scan": {"realtime_monitor": true, "max_depth": null},
                    "metadata_profile": {
                      "item_kinds": ["movie"],
                      "local_readers": ["nfo"],
                      "metadata_providers": ["tmdb"],
                      "image_providers": ["tmdb"],
                      "language": "en",
                      "country": "US",
                      "refresh_mode": "missing_only",
                      "local_metadata_policy": "local_first"
                    }
                  }
                }
              ],
              "page": {"limit": 20, "offset": 40, "returned": 1}
            }
            """.trimIndent(),
        )

        val library = libraries.libraries.single()
        assertEquals("Movies", library.name)
        assertEquals("video", library.options.domain.wireValue)
        assertEquals("movies", library.options.preset.wireValue)
        assertEquals(40L, libraries.page.offset)
    }
}
