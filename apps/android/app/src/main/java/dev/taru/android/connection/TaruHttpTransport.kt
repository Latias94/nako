package dev.taru.android.connection

data class TaruHttpRequest(
    val method: String,
    val url: String,
    val headers: Map<String, String> = emptyMap(),
    val body: String? = null,
)

data class TaruHttpResponse(
    val statusCode: Int,
    val headers: Map<String, List<String>> = emptyMap(),
    val body: String = "",
) {
    fun isSuccessful(): Boolean = statusCode in 200..299

    fun header(name: String): String? {
        val expected = name.lowercase()
        return headers.entries
            .firstOrNull { (key, _) -> key.lowercase() == expected }
            ?.value
            ?.firstOrNull()
    }
}

interface TaruHttpTransport {
    suspend fun execute(request: TaruHttpRequest): TaruHttpResponse
}
