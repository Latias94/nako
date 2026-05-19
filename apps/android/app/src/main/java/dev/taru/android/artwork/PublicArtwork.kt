package dev.taru.android.artwork

import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.android.connection.SensitiveText
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TaruHttpRequest
import dev.taru.android.connection.TaruPublicApiContract
import kotlinx.serialization.json.JsonPrimitive

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
        val path = image.url.trim()
        if (accessToken.isBlank() || !path.isPublicImagePath()) {
            return null
        }

        val request = TaruHttpRequest(
            method = "GET",
            url = "${profile.baseUrl.trimEnd('/')}$path",
            headers = mapOf("Authorization" to "Bearer $accessToken"),
        )
        return PublicArtworkRequest(
            request = request,
            safeRequest = SafeRequestPreview(
                method = request.method,
                url = SensitiveText.sanitize(request.url, listOf(accessToken)),
                headers = request.headers.mapValues { (name, value) ->
                    if (name.equals("Authorization", ignoreCase = true)) {
                        "Bearer ${TaruPublicApiContract.redacted}"
                    } else {
                        SensitiveText.sanitize(value, listOf(accessToken))
                    }
                },
            ),
            image = image,
        )
    }
}

private fun String.isPublicImagePath(): Boolean =
    startsWith("/images/") &&
        !contains("..") &&
        !contains("//") &&
        !contains("?") &&
        !contains("#")

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
                image.kindWireValue() == kind && image.url.startsWith("/images/")
            }
        }
        ?: images.firstOrNull { image -> image.url.startsWith("/images/") }
}

fun PublicImageRefDto.kindWireValue(): String =
    when (val value = kind) {
        is JsonPrimitive -> value.content.lowercase()
        else -> value.toString().trim('"').lowercase()
    }
