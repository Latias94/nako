package dev.nako.android.ui.artwork

import dev.nako.android.artwork.PublicArtworkRequest
import dev.nako.android.artwork.PublicArtworkSource
import dev.nako.android.browse.PublicImageRefDto
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.TokenVault

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
