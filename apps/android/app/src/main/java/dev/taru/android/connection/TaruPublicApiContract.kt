package dev.taru.android.connection

object TaruPublicApiContract {
    const val expectedApiVersion = "v1"
    const val apiVersionHeader = "x-taru-api-version"
    const val healthPath = "/health"
    const val authProbePath = "/libraries?limit=1&offset=0"
    const val redacted = "<redacted>"
}
