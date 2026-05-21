package dev.taru.android.connection

import kotlinx.serialization.Serializable

@Serializable
data class HealthEnvelope(
    val status: String,
    val version: String,
)

@Serializable
data class PublicErrorEnvelope(
    val code: String,
    val message: String,
)

@Serializable
data class ServerProfile(
    val id: String,
    val displayName: String,
    val baseUrl: String,
    val tokenReference: String,
    val lastObservedApiVersion: String? = null,
    val lastSuccessfulConnectionAtMillis: Long? = null,
    val lastPublicError: PublicErrorEnvelope? = null,
)

@Serializable
data class ServerProfileSnapshot(
    val profiles: List<ServerProfile> = emptyList(),
    val activeProfileId: String? = null,
)

data class SafeRequestPreview(
    val method: String,
    val url: String,
    val headers: Map<String, String> = emptyMap(),
)

data class SafeConnectionDiagnostics(
    val category: ConnectionFailureCategory,
    val userMessage: String,
    val statusCode: Int? = null,
    val expectedApiVersion: String = TaruPublicApiContract.expectedApiVersion,
    val observedApiVersion: String? = null,
    val publicError: PublicErrorEnvelope? = null,
    val request: SafeRequestPreview? = null,
)

enum class ConnectionFailureCategory {
    InvalidUrl,
    MissingAccessToken,
    UnreachableServer,
    Unauthorized,
    UnsupportedApiVersion,
    TlsOrCertificate,
    InsecureCleartextHttp,
    PublicApiError,
    InvalidResponse,
}

sealed interface ConnectionCheckResult {
    data class Success(
        val normalizedBaseUrl: String,
        val apiVersion: String,
        val checkedAtMillis: Long,
        val healthRequest: SafeRequestPreview,
        val authProbeRequest: SafeRequestPreview,
    ) : ConnectionCheckResult

    data class Failure(
        val normalizedBaseUrl: String?,
        val diagnostics: SafeConnectionDiagnostics,
    ) : ConnectionCheckResult
}
