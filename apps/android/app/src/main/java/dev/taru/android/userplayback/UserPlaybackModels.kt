package dev.taru.android.userplayback

import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import dev.taru.sdk.TARU_API_VERSION

data class UserPlaybackStateResponse(
    val state: UserPlaybackStateDto,
)

data class ContinueWatchingResponse(
    val items: List<ContinueWatchingItemDto> = emptyList(),
    val page: PageInfo,
)

data class ContinueWatchingItemDto(
    val item: MediaItemDto,
    val state: UserPlaybackStateDto,
    val images: List<PublicImageRefDto> = emptyList(),
)

data class UserPlaybackStateDto(
    val itemId: String,
    val sourceId: String? = null,
    val resumePositionMs: Long? = null,
    val durationMs: Long? = null,
    val progressPercent: Float? = null,
    val watched: Boolean,
    val watchedAt: String? = null,
    val lastPlayedAt: String? = null,
    val updatedAt: String? = null,
    val version: Long,
)

data class UpdatePlaybackProgressRequest(
    val sourceId: String? = null,
    val positionMs: Long,
    val durationMs: Long? = null,
    val reportedAt: String? = null,
)

data class SetWatchedStateRequest(
    val watched: Boolean,
    val sourceId: String? = null,
    val positionMs: Long? = null,
    val durationMs: Long? = null,
    val markedAt: String? = null,
)

enum class UserPlaybackFailureCategory {
    MissingItem,
    MissingAccessToken,
    UnreachableServer,
    Unauthorized,
    Forbidden,
    UnsupportedApiVersion,
    TlsOrCertificate,
    Conflict,
    PublicApiError,
    InvalidResponse,
}

data class SafeUserPlaybackDiagnostics(
    val category: UserPlaybackFailureCategory,
    val userMessage: String,
    val statusCode: Int? = null,
    val expectedApiVersion: String = TARU_API_VERSION,
    val observedApiVersion: String? = null,
    val publicError: PublicErrorEnvelope? = null,
    val request: SafeRequestPreview? = null,
)

sealed interface UserPlaybackResult<out T> {
    data class Success<T>(
        val value: T,
        val request: SafeRequestPreview,
    ) : UserPlaybackResult<T>

    data class Failure(
        val diagnostics: SafeUserPlaybackDiagnostics,
    ) : UserPlaybackResult<Nothing>
}
