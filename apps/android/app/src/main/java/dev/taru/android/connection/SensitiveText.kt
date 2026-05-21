package dev.taru.android.connection

object SensitiveText {
    const val redacted = "<redacted>"

    private val bearerPattern = Regex("(?i)Bearer\\s+[^\\s\"']+")
    private val windowsPathPattern = Regex("[A-Za-z]:\\\\[^\\s\"']+")
    private val fileUrlPattern = Regex("file://[^\\s\"']+")
    private val unixPathPattern = Regex("(?<![A-Za-z0-9_])/(Users|home|mnt|var|tmp|Volumes)/[^\\s\"']+")
    private val ffmpegCommandPattern = Regex("(?i)\\bffmpeg(\\.exe)?\\b[^\\n\\r]*")
    private val secretReferencePattern =
        Regex("(?i)(secret[_-]?(env|ref|reference)\\s*[:=]\\s*)[^\\s,\"'}]+")

    fun sanitize(
        input: String,
        secrets: Iterable<String> = emptyList(),
    ): String {
        var sanitized = input
        secrets
            .filter { it.isNotBlank() }
            .forEach { secret ->
                sanitized = sanitized.replace(secret, redacted)
            }

        sanitized = bearerPattern.replace(sanitized, "Bearer $redacted")
        sanitized = windowsPathPattern.replace(sanitized, "<local-path>")
        sanitized = fileUrlPattern.replace(sanitized, "<local-path>")
        sanitized = unixPathPattern.replace(sanitized, "<local-path>")
        sanitized = ffmpegCommandPattern.replace(sanitized, "<ffmpeg-command>")
        sanitized = secretReferencePattern.replace(sanitized) {
            "${it.groupValues[1]}$redacted"
        }
        return sanitized
    }

    fun sanitizeEnvelope(
        envelope: PublicErrorEnvelope,
        secrets: Iterable<String> = emptyList(),
    ): PublicErrorEnvelope =
        PublicErrorEnvelope(
            code = sanitize(envelope.code, secrets),
            message = sanitize(envelope.message, secrets),
        )
}
