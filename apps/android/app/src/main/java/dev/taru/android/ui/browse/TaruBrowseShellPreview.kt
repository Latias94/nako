package dev.taru.android.ui.browse

import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import dev.taru.android.browse.TaruBrowseClient
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruHttpResponse
import dev.taru.android.connection.TaruHttpTransport
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.ui.theme.TaruAndroidTheme

@Preview
@Composable
private fun TaruBrowseShellPreview() {
    val tokenVault = InMemoryTokenVault().apply {
        saveToken("server-token:server-1", "preview-token")
    }
    TaruAndroidTheme(darkTheme = true) {
        TaruBrowseShell(
            profile = ServerProfile(
                id = "server-1",
                displayName = "Home Server",
                baseUrl = "http://localhost:3000",
                tokenReference = "server-token:server-1",
                lastObservedApiVersion = "v1",
            ),
            snapshot = ServerProfileSnapshot(
                profiles = listOf(
                    ServerProfile(
                        id = "server-1",
                        displayName = "Home Server",
                        baseUrl = "http://localhost:3000",
                        tokenReference = "server-token:server-1",
                        lastObservedApiVersion = "v1",
                    ),
                ),
                activeProfileId = "server-1",
            ),
            tokenVault = tokenVault,
            browseClient = TaruBrowseClient(
                transport = object : TaruHttpTransport {
                    override suspend fun execute(request: TaruHttpRequest): TaruHttpResponse =
                        if (request.url.contains("/libraries")) {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"libraries":[{"id":"library-1","name":"Movies","options":{"domain":"video","preset":"movies"}}],"page":{"limit":50,"offset":0,"returned":1}}""",
                            )
                        } else if (request.url.contains("/items/item-1")) {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"item":{"id":"item-1","kind":"movie","metadata":{"title":"Night Harbor","overview":"A remote harbor town begins to glow after midnight.","release_date":"2024-01-01","runtime_minutes":106,"genres":["Mystery"],"tags":["Lighthouse"],"ratings":[],"images":[]}},"sources":[{"id":"source-1","library_id":"library-1","item_id":"item-1","locator":"file:///preview/night-harbor.mkv","file_name":"night-harbor.mkv","size_bytes":42}],"credits":[],"genres":[],"tags":[],"collections":[],"studios":[],"images":[]}""",
                            )
                        } else {
                            TaruHttpResponse(
                                statusCode = 200,
                                body = """{"items":[{"id":"item-1","kind":"movie","metadata":{"title":"Night Harbor","release_date":"2024-01-01","runtime_minutes":106,"genres":["Mystery"],"tags":["Lighthouse"],"ratings":[],"images":[]}}],"page":{"limit":24,"offset":0,"returned":1}}""",
                            )
                        }
                },
            ),
            playbackClient = TaruPlaybackClient(
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
                                "locator": "file:///preview/night-harbor.mkv",
                                "file_name": "night-harbor.mkv",
                                "size_bytes": 42
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
            ),
            onChangeServer = {},
        )
    }
}
