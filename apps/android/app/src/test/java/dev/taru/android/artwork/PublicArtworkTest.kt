package dev.taru.android.artwork

import dev.taru.android.browse.ItemDetailResponse
import dev.taru.android.connection.ServerProfile
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PublicArtworkTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun `item detail decodes public image refs without raw locators`() {
        val detail = json.decodeFromString<ItemDetailResponse>(
            """
            {
              "item": {
                "id": "item-1",
                "kind": "movie",
                "metadata": {
                  "title": "Night Harbor",
                  "genres": [],
                  "tags": [],
                  "ratings": []
                }
              },
              "sources": [],
              "credits": [],
              "genres": [],
              "tags": [],
              "collections": [],
              "studios": [],
              "images": [
                {
                  "id": "poster-1",
                  "owner": {"item": "item-1"},
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
        )

        val image = detail.images.single()
        assertEquals("poster-1", image.id)
        assertEquals("/images/poster-1", image.url)
        assertEquals("poster", image.kindWireValue())
        assertEquals("image/png", image.mediaType)
        assertFalse(image.toString().contains("source_uri"))
        assertFalse(image.toString().contains("managed-artwork://"))
    }

    @Test
    fun `selected artwork request is scoped to active server and redacts safe output`() {
        val detail = json.decodeFromString<ItemDetailResponse>(detailWithImages())
        val source = PublicArtworkSource(
            profile = profile("server-1", "http://home.example.test/base"),
            accessToken = "secret-token",
        )

        val request = source.requestFor(
            image = preferredPublicArtwork(detail.images, PublicArtworkSlot.Poster),
        )

        assertNotNull(request)
        requireNotNull(request)
        assertEquals("GET", request.request.method)
        assertEquals("http://home.example.test/base/images/poster-1", request.request.url)
        assertEquals("Bearer secret-token", request.request.headers["Authorization"])
        assertEquals("Bearer <redacted>", request.safeRequest.headers["Authorization"])
        assertTrue(request.toString().contains("Bearer <redacted>"))
        assertFalse(request.toString().contains("secret-token"))
    }

    @Test
    fun `active server controls selected artwork base url and token`() {
        val image = preferredPublicArtwork(
            images = json.decodeFromString<ItemDetailResponse>(detailWithImages()).images,
            slot = PublicArtworkSlot.Backdrop,
        )

        val home = PublicArtworkSource(
            profile = profile("home", "http://home.example.test"),
            accessToken = "home-token",
        ).requestFor(image)
        val travel = PublicArtworkSource(
            profile = profile("travel", "https://travel.example.test/taru"),
            accessToken = "travel-token",
        ).requestFor(image)

        assertEquals("http://home.example.test/images/backdrop-1", home?.request?.url)
        assertEquals("https://travel.example.test/taru/images/backdrop-1", travel?.request?.url)
        assertEquals("Bearer home-token", home?.request?.headers?.get("Authorization"))
        assertEquals("Bearer travel-token", travel?.request?.headers?.get("Authorization"))
        assertFalse(home.toString().contains("home-token"))
        assertFalse(travel.toString().contains("travel-token"))
    }

    @Test
    fun `selected artwork request rejects unsafe or unusable image urls`() {
        val source = PublicArtworkSource(
            profile = profile("server-1", "http://home.example.test"),
            accessToken = "secret-token",
        )
        val detail = json.decodeFromString<ItemDetailResponse>(
            detailWithImages(
                posterUrl = "https://evil.example.test/images/poster-1",
                backdropUrl = "/admin/v1/artwork/private",
            ),
        )

        assertNull(source.requestFor(preferredPublicArtwork(detail.images, PublicArtworkSlot.Poster)))
        assertNull(source.requestFor(preferredPublicArtwork(detail.images, PublicArtworkSlot.Backdrop)))
        assertNull(
            source.requestFor(
                json.decodeFromString<ItemDetailResponse>(
                    detailWithImages(posterUrl = "/images/poster-1?token=leak"),
                ).images.first { it.id == "poster-1" },
            ),
        )
        assertNull(
            PublicArtworkSource(
                profile = profile("server-1", "http://home.example.test"),
                accessToken = " ",
            ).requestFor(detail.images.first()),
        )
    }

    private fun profile(
        id: String,
        baseUrl: String,
    ): ServerProfile =
        ServerProfile(
            id = id,
            displayName = id,
            baseUrl = baseUrl,
            tokenReference = "server-token:$id",
            lastObservedApiVersion = "v1",
        )

    private fun detailWithImages(
        posterUrl: String = "/images/poster-1",
        backdropUrl: String = "/images/backdrop-1",
    ): String =
        """
        {
          "item": {
            "id": "item-1",
            "kind": "movie",
            "metadata": {
              "title": "Night Harbor",
              "genres": [],
              "tags": [],
              "ratings": []
            }
          },
          "sources": [],
          "credits": [],
          "genres": [],
          "tags": [],
          "collections": [],
          "studios": [],
          "images": [
            {
              "id": "backdrop-1",
              "owner": {"item": "item-1"},
              "kind": "backdrop",
              "url": "$backdropUrl",
              "width": 1920,
              "height": 1080,
              "language": null,
              "media_type": "image/webp",
              "etag": "hash-backdrop"
            },
            {
              "id": "poster-1",
              "owner": {"item": "item-1"},
              "kind": "poster",
              "url": "$posterUrl",
              "width": 1000,
              "height": 1500,
              "language": null,
              "media_type": "image/png",
              "etag": "hash-poster"
            }
          ]
        }
        """.trimIndent()
}
