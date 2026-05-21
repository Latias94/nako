package dev.taru.android.artwork

import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.urlOn
import dev.taru.sdk.TaruPublicClientRequests
import dev.taru.sdk.TaruRequestDescriptor

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
) {
    fun requestFor(image: PublicImageRefDto?): PublicArtworkRequest? {
        image ?: return null
        val descriptor = image.publicImageRequestDescriptorOrNull()
        if (accessToken.isBlank() || descriptor == null) {
            return null
        }

        val request = TaruHttpRequest(
            method = "GET",
            url = descriptor.urlOn(profile),
            headers = mapOf("Authorization" to "Bearer $accessToken"),
        )
        return PublicArtworkRequest(
            request = request,
            safeRequest = SafeRequestPreview(
                method = request.method,
                url = SensitiveText.sanitize(request.url, listOf(accessToken)),
                headers = request.headers.mapValues { (name, value) ->
                    if (name.equals("Authorization", ignoreCase = true)) {
                        "Bearer ${SensitiveText.redacted}"
                    } else {
                        SensitiveText.sanitize(value, listOf(accessToken))
                    }
                },
            ),
            image = image,
        )
    }
}

private fun PublicImageRefDto.publicImageRequestDescriptorOrNull(): TaruRequestDescriptor? {
    val path = url.trim()
    if (
        id.isBlank() ||
        path.contains("..") ||
        path.contains("//") ||
        path.contains("?") ||
        path.contains("#")
    ) {
        return null
    }
    return TaruPublicClientRequests
        .image(id)
        .takeIf { descriptor -> descriptor.pathAndQuery == path }
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
                image.kindWireValue() == kind && image.publicImageRequestDescriptorOrNull() != null
            }
        }
        ?: images.firstOrNull { image -> image.publicImageRequestDescriptorOrNull() != null }
}

fun PublicImageRefDto.kindWireValue(): String =
    kind.lowercase()
