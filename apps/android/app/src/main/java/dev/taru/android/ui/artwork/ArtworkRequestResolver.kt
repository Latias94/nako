package dev.taru.android.ui.artwork

import dev.taru.android.artwork.PublicArtworkRequest
import dev.taru.android.artwork.PublicArtworkSource
import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault

internal fun interface ArtworkRequestResolver {
    fun requestFor(image: PublicImageRefDto?): PublicArtworkRequest?
}

internal object EmptyArtworkRequestResolver : ArtworkRequestResolver {
    override fun requestFor(image: PublicImageRefDto?): PublicArtworkRequest? = null
}

internal class TokenVaultArtworkRequestResolver(
    private val profile: ServerProfile,
    private val tokenVault: TokenVault,
) : ArtworkRequestResolver {
    override fun requestFor(image: PublicImageRefDto?): PublicArtworkRequest? =
        PublicArtworkSource(
            profile = profile,
            accessToken = tokenVault.readToken(profile.tokenReference).orEmpty(),
        ).requestFor(image)
}
