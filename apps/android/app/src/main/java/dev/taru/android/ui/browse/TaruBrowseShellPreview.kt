package dev.taru.android.ui.browse

import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.tooling.preview.Preview
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.playback.InMemoryPlaybackPreferencesStore
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.InMemoryDevicePlaybackPositionStore
import dev.taru.android.ui.screens.player.rememberAndroidPlaybackSessionRuntimeFactory
import dev.taru.android.ui.screens.player.rememberPlaybackPlayerRouteRenderer
import dev.taru.android.ui.theme.TaruAndroidTheme
import dev.taru.android.userplayback.TaruUserPlaybackClient

@Preview
@Composable
private fun TaruBrowseShellPreview() {
    val tokenVault = InMemoryTokenVault().apply {
        saveToken("server-token:server-1", "preview-token")
    }
    val profile = ServerProfile(
        id = "server-1",
        displayName = "Home Server",
        baseUrl = "http://localhost:3000",
        tokenReference = "server-token:server-1",
        lastObservedApiVersion = "v1",
    )
    val playbackClient = TaruPlaybackClient(
        transport = object : TaruHttpTransport {
            override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse =
                TaruHttpResponse(
                    statusCode = 200,
                    body = """
                    {
                      "source": {
                        "id": "source-1",
                        "library_id": "library-1",
                        "item_id": "item-1",
                        "file_name": "night-harbor.mkv",
                        "size_bytes": 42,
                        "fingerprint": null
                      },
                      "probe": null,
                      "decision": {
                        "mode": "direct_play",
                        "reason": "preview route",
                        "direct_play": {
                          "source_id": "source-1",
                          "content_type": "video/x-matroska",
                          "supports_range_requests": true
                        },
                        "transcode_plan": null
                      }
                    }
                    """.trimIndent(),
                )
        },
    )
    val userPlaybackClient = TaruUserPlaybackClient(
        transport = object : TaruHttpTransport {
            override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse =
                if (request.url.matchesPreviewRoute(previewRouteContinueWatching(limit = 12))) {
                    TaruHttpResponse(
                        statusCode = 200,
                        body = """
                        {
                          "items": [
                            {
                              "item": ${previewMediaItemJson(
                                  id = "item-1",
                                  title = "Night Harbor",
                                  overview = null,
                                  releaseDate = "2024-01-01",
                                  runtimeMinutes = 106,
                                  genresJson = "[\"Mystery\"]",
                                  tagsJson = "[\"Lighthouse\"]",
                              ).prependIndent("                              ")},
                              "state": {
                                "item_id": "item-1",
                                "source_id": "source-1",
                                "resume_position_ms": 92000,
                                "duration_ms": 6360000,
                                "progress_percent": 1.4,
                                "watched": false,
                                "watched_at": null,
                                "last_played_at": "2026-05-19T00:00:00Z",
                                "updated_at": "2026-05-19T00:00:00Z",
                                "version": 1
                              },
                              "images": []
                            }
                          ],
                          "page": {"limit":12,"offset":0,"returned":1}
                        }
                        """.trimIndent(),
                    )
                } else {
                    TaruHttpResponse(
                        statusCode = 200,
                        body = """
                        {
                          "state": {
                            "item_id": "item-1",
                            "source_id": "source-1",
                            "resume_position_ms": 92000,
                            "duration_ms": 6360000,
                            "progress_percent": 1.4,
                            "watched": false,
                            "watched_at": null,
                            "last_played_at": "2026-05-19T00:00:00Z",
                            "updated_at": "2026-05-19T00:00:00Z",
                            "version": 1
                          }
                        }
                        """.trimIndent(),
                    )
                }
        },
    )
    val positionStore = InMemoryDevicePlaybackPositionStore()
    val playerExitEffectScope = rememberCoroutineScope()
    val playbackSessionRuntimeFactory = rememberAndroidPlaybackSessionRuntimeFactory(
        profile = profile,
        tokenVault = tokenVault,
        playbackClient = playbackClient,
        userPlaybackClient = userPlaybackClient,
        positionStore = positionStore,
        exitEffectScope = playerExitEffectScope,
    )
    val playerRouteRenderer = rememberPlaybackPlayerRouteRenderer(playbackSessionRuntimeFactory)
    TaruAndroidTheme(darkTheme = true) {
        TaruBrowseShell(
            profile = profile,
            snapshot = ServerProfileSnapshot(
                profiles = listOf(profile),
                activeProfileId = "server-1",
            ),
            tokenVault = tokenVault,
            browseClient = TaruBrowseClient(
                transport = object : TaruHttpTransport {
                    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse =
                        if (request.url.matchesPreviewRoute(previewRouteListLibraries())) {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"libraries":[{"id":"library-1","name":"Movies","roots":[],"options":${previewLibraryOptionsJson()}}],"page":{"limit":50,"offset":0,"returned":1}}""",
                            )
                        } else if (request.url.matchesPreviewRoute(previewRouteItem("item-1"))) {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"item":${previewMediaItemJson(id = "item-1", title = "Night Harbor", overview = "A remote harbor town begins to glow after midnight.", releaseDate = "2024-01-01", runtimeMinutes = 106, genresJson = "[\"Mystery\"]", tagsJson = "[\"Lighthouse\"]")},"sources":[{"id":"source-1","library_id":"library-1","item_id":"item-1","file_name":"night-harbor.mkv","size_bytes":42,"fingerprint":null}],"credits":[],"genres":[],"tags":[],"collections":[],"studios":[],"images":[]}""",
                            )
                        } else {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"items":[${previewMediaItemJson(id = "item-1", title = "Night Harbor", releaseDate = "2024-01-01", runtimeMinutes = 106, genresJson = "[\"Mystery\"]", tagsJson = "[\"Lighthouse\"]")}],"page":{"limit":24,"offset":0,"returned":1}}""",
                            )
                        }
                },
            ),
            playbackClient = playbackClient,
            userPlaybackClient = userPlaybackClient,
            playbackPreferencesStore = InMemoryPlaybackPreferencesStore(),
            positionStore = positionStore,
            playerRouteRenderer = playerRouteRenderer,
            onChangeServer = {},
        )
    }
}

private fun previewLibraryOptionsJson(): String =
    """{"domain":"video","preset":"movies","naming_strategy":"movie","scan":{"realtime_monitor":true,"max_depth":null},"metadata_profile":{"item_kinds":["movie"],"local_readers":["nfo"],"metadata_providers":["tmdb"],"image_providers":["tmdb"],"language":"en","country":"US","refresh_mode":"missing_only","local_metadata_policy":"local_first"}}"""

private fun previewMediaItemJson(
    id: String,
    title: String,
    overview: String? = null,
    releaseDate: String? = null,
    runtimeMinutes: Int? = null,
    genresJson: String = "[]",
    tagsJson: String = "[]",
): String =
    """{"id":"$id","kind":"movie","parent_id":null,"metadata":{"title":"$title","original_title":null,"sort_title":null,"overview":${previewJsonStringOrNull(overview)},"release_date":${previewJsonStringOrNull(releaseDate)},"runtime_minutes":${runtimeMinutes ?: "null"},"tagline":null,"genres":$genresJson,"tags":$tagsJson,"ratings":[],"credits":[],"collections":[],"studios":[],"external_ids":[]}}"""

private fun previewJsonStringOrNull(value: String?): String =
    value?.let { "\"${it.replace("\\", "\\\\").replace("\"", "\\\"")}\"" } ?: "null"

private fun previewRouteListLibraries(): String =
    "/libraries?limit=50&offset=0"

private fun previewRouteContinueWatching(limit: Int): String =
    "/users/me/playback-state/continue-watching?limit=$limit&offset=0"

private fun previewRouteItem(itemId: String): String =
    "/items/${itemId.previewPathSegment()}"

private fun String.matchesPreviewRoute(routeSuffix: String): Boolean =
    trimEnd('/').endsWith(routeSuffix)

private fun String.previewPathSegment(): String =
    replace(" ", "%20")
