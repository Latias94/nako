package dev.nako.android.connection

import java.net.HttpURLConnection
import java.net.URL
import java.nio.charset.StandardCharsets
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class JdkNakoHttpTransport(
    private val connectTimeoutMillis: Int = 10_000,
    private val readTimeoutMillis: Int = 10_000,
    private val securityPolicy: ConnectionSecurityPolicy = ConnectionSecurityPolicy.production(),
) : NakoHttpTransport {
    override suspend fun execute(request: NakoHttpRequest): NakoHttpResponse =
        withContext(Dispatchers.IO) {
            securityPolicy.requireRequestAllowed(request.url)
            val connection = URL(request.url).openConnection() as HttpURLConnection
            try {
                connection.requestMethod = request.method
                connection.connectTimeout = connectTimeoutMillis
                connection.readTimeout = readTimeoutMillis
                request.headers.forEach { (name, value) ->
                    connection.setRequestProperty(name, value)
                }
                request.body?.let { body ->
                    val bodyBytes = body.toByteArray(StandardCharsets.UTF_8)
                    connection.doOutput = true
                    connection.setFixedLengthStreamingMode(bodyBytes.size)
                    connection.outputStream.use { output ->
                        output.write(bodyBytes)
                    }
                }

                val statusCode = connection.responseCode
                val body = (if (statusCode >= 400) connection.errorStream else connection.inputStream)
                    ?.bufferedReader()
                    ?.use { it.readText() }
                    .orEmpty()
                val headers = connection.headerFields
                    .filterKeys { it != null }
                    .mapKeys { (key, _) -> key.orEmpty() }

                NakoHttpResponse(
                    statusCode = statusCode,
                    headers = headers,
                    body = body,
                )
            } finally {
                connection.disconnect()
            }
        }
}
