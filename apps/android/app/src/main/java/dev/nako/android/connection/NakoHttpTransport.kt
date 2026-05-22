package dev.nako.android.connection

data class NakoHttpRequest(
    val method: String,
    val url: String,
    val headers: Map<String, String> = emptyMap(),
    val body: String? = null,
)

data class NakoHttpResponse(
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

interface NakoHttpTransport {
    suspend fun execute(request: NakoHttpRequest): NakoHttpResponse
}
