package dev.taru.android.ui.artwork

import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.InMemoryTokenVault
import dev.taru.android.connection.ServerProfile
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Test

class ArtworkRequestResolverTest {
    @Test
    fun tokenVaultResolverBuildsRedactedArtworkRequestsFromCurrentProfileToken() {
        val tokenVault = InMemoryTokenVault().apply {
            saveToken("server-token:home", "secret-token")
        }
        val resolver = TokenVaultArtworkRequestResolver(
            profile = profile(),
            tokenVault = tokenVault,
        )

        val request = resolver.requestFor(publicImage())

        requireNotNull(request)
        assertEquals("http://home.example.test/images/poster-1", request.request.url)
        assertEquals("Bearer secret-token", request.request.headers["Authorization"])
        assertEquals("Bearer <redacted>", request.safeRequest.headers["Authorization"])
        assertFalse(request.toString().contains("secret-token"))
    }

    @Test
    fun tokenVaultResolverReadsTheCurrentTokenAtRequestTime() {
        val tokenVault = InMemoryTokenVault().apply {
            saveToken("server-token:home", "old-token")
        }
        val resolver = TokenVaultArtworkRequestResolver(
            profile = profile(),
            tokenVault = tokenVault,
        )

        tokenVault.saveToken("server-token:home", "new-token")

        val request = resolver.requestFor(publicImage())

        assertEquals("Bearer new-token", request?.request?.headers?.get("Authorization"))
    }

    @Test
    fun tokenVaultResolverReturnsNullWithoutAUsableToken() {
        val resolver = TokenVaultArtworkRequestResolver(
            profile = profile(),
            tokenVault = InMemoryTokenVault(),
        )

        assertNull(resolver.requestFor(publicImage()))
    }

    @Test
    fun emptyResolverAlwaysReturnsNull() {
        assertNull(EmptyArtworkRequestResolver.requestFor(publicImage()))
    }

    private fun profile(): ServerProfile =
        ServerProfile(
            id = "home",
            displayName = "Home",
            baseUrl = "http://home.example.test",
            tokenReference = "server-token:home",
            lastObservedApiVersion = "v1",
        )

    private fun publicImage(): PublicImageRefDto =
        PublicImageRefDto(
            id = "poster-1",
            owner = mapOf("item" to "item-1"),
            kind = "poster",
            url = "/images/poster-1",
            width = 1000,
            height = 1500,
            language = null,
            mediaType = "image/png",
            etag = "hash-1",
        )
}
