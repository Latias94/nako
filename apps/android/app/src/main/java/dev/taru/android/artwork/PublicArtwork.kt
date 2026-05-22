package dev.taru.android.artwork

import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.toAndroidPreview
import dev.taru.android.connection.toAndroidRequest
import uniffi.taru_client_uniffi.CoreArtworkImageRequestInput
import uniffi.taru_client_uniffi.CoreHttpRequest

enum class PublicArtworkSlot {
    Poster,
    Backdrop,
    Thumbnail,
}

data class PublicArtworkRequest(
    val request: TaruHttpRequest,
    val safeRequest: SafeRequestPreview,
    val image: PublicImageRefDto,
) {
    override fun toString(): String =
        "PublicArtworkRequest(safeRequest=$safeRequest, image=$image)"
}

class PublicArtworkSource(
    private val profile: ServerProfile,
    private val accessToken: String,
    private val artworkCore: ArtworkCore = RustArtworkCore,
) {
    fun requestFor(image: PublicImageRefDto?): PublicArtworkRequest? {
        image ?: return null
        val descriptor = image.publicImageRequestDescriptorOrNull(
            profile = profile,
            accessToken = accessToken,
            artworkCore = artworkCore,
        ) ?: return null

        return PublicArtworkRequest(
            request = descriptor.request,
            safeRequest = descriptor.safePreview,
            image = image,
        )
    }
}

interface ArtworkCore {
    fun image(
        profile: ServerProfile,
        accessToken: String,
        imageId: String,
        width: UInt? = null,
        height: UInt? = null,
    ): ArtworkRequestDescriptor
}

object RustArtworkCore : ArtworkCore {
    override fun image(
        profile: ServerProfile,
        accessToken: String,
        imageId: String,
        width: UInt?,
        height: UInt?,
    ): ArtworkRequestDescriptor =
        uniffi.taru_client_uniffi.buildArtworkImageRequest(
            CoreArtworkImageRequestInput(
                baseUrl = profile.baseUrl,
                accessToken = accessToken,
                imageId = imageId,
                width = width,
                height = height,
            ),
        ).toArtworkDescriptor()
}

data class ArtworkRequestDescriptor(
    val request: TaruHttpRequest,
    val safePreview: SafeRequestPreview,
)

private fun PublicImageRefDto.publicImageRequestDescriptorOrNull(
    profile: ServerProfile,
    accessToken: String,
    artworkCore: ArtworkCore,
): ArtworkRequestDescriptor? {
    if (id.isBlank() || accessToken.isBlank()) {
        return null
    }
    val expected = artworkCore.image(
        profile = profile,
        accessToken = accessToken,
        imageId = id,
    )
    val path = url.trim()
    if (
        path.isBlank() ||
        path.contains("..") ||
        path.contains("//") ||
        path.contains("?") ||
        path.contains("#")
    ) {
        return null
    }
    return expected.takeIf { descriptor ->
        descriptor.request.url.pathAndQueryOn(profile) == path
    }
}

fun preferredPublicArtwork(
    images: List<PublicImageRefDto>,
    slot: PublicArtworkSlot,
): PublicImageRefDto? {
    val preferredKinds = when (slot) {
        PublicArtworkSlot.Poster -> listOf("poster", "thumbnail", "backdrop")
        PublicArtworkSlot.Backdrop -> listOf("backdrop", "poster", "thumbnail")
        PublicArtworkSlot.Thumbnail -> listOf("thumbnail", "poster", "backdrop")
    }
    return preferredKinds
        .firstNotNullOfOrNull { kind ->
            images.firstOrNull { image ->
                image.kindWireValue() == kind && image.hasPublicImageRouteShape()
            }
        }
        ?: images.firstOrNull { image -> image.hasPublicImageRouteShape() }
}

fun PublicImageRefDto.kindWireValue(): String =
    kind.lowercase()

private fun PublicImageRefDto.hasPublicImageRouteShape(): Boolean {
    val path = url.trim()
    return id.isNotBlank() &&
        path.startsWith("/images/") &&
        !path.contains("..") &&
        !path.contains("//") &&
        !path.contains("?") &&
        !path.contains("#")
}

private fun String.pathAndQueryOn(profile: ServerProfile): String? {
    val baseUrl = profile.baseUrl.trimEnd('/')
    return takeIf { fullUrl -> fullUrl.startsWith(baseUrl) }
        ?.removePrefix(baseUrl)
        ?.takeIf { pathAndQuery -> pathAndQuery.startsWith("/") }
}

private fun CoreHttpRequest.toArtworkDescriptor(): ArtworkRequestDescriptor =
    ArtworkRequestDescriptor(
        request = toAndroidRequest(),
        safePreview = safePreview.toAndroidPreview(),
    )
