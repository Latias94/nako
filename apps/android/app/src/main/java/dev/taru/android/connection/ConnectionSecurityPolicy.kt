package dev.taru.android.connection

import java.io.IOException
import java.net.URI

class CleartextHttpNotPermittedException(
    url: String,
) : IOException("Cleartext HTTP is not permitted for ${SensitiveText.sanitize(url)}")

data class ConnectionSecurityPolicy(
    val allowCleartextHttp: Boolean = false,
) {
    fun cleartextFailure(uri: URI): PublicErrorEnvelope? {
        val scheme = uri.scheme?.lowercase()
        return if (scheme == "http" && !allowCleartextHttp) {
            PublicErrorEnvelope(
                code = "cleartext_http_not_allowed",
                message = "Use HTTPS for this server, or use a local-development build that explicitly allows HTTP.",
            )
        } else {
            null
        }
    }

    fun requireRequestAllowed(url: String) {
        val uri = runCatching { URI(url) }.getOrNull()
        if (uri?.scheme?.lowercase() == "http" && !allowCleartextHttp) {
            throw CleartextHttpNotPermittedException(url)
        }
    }

    companion object {
        fun production(): ConnectionSecurityPolicy =
            ConnectionSecurityPolicy(allowCleartextHttp = false)

        fun allowCleartextForLocalDevelopment(): ConnectionSecurityPolicy =
            ConnectionSecurityPolicy(allowCleartextHttp = true)
    }
}
