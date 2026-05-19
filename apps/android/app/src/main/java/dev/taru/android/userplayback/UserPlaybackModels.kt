package dev.taru.android.userplayback

import dev.taru.android.browse.MediaItemDto
import dev.taru.android.browse.PageInfo
import dev.taru.android.browse.PublicImageRefDto
import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.SafeRequestPreview
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class UserPlaybackStateResponse(
    val state: UserPlaybackStateDto,
)

@Serializable
data class ContinueWatchingResponse(
    val items: List<ContinueWatchingItemDto> = emptyList(),
    val page: PageInfo,
)

@Serializable
data class ContinueWatchingItemDto(
    val item: MediaItemDto,
    val state: UserPlaybackStateDto,
    val images: List<PublicImageRefDto> = emptyList(),
)

@Serializable
data class UserPlaybackStateDto(
    @SerialName("item_id")
    val itemId: String,
    @SerialName("source_id")
    val sourceId: String? = null,
    @SerialName("resume_position_ms")
    val resumePositionMs: Long? = null,
    @SerialName("duration_ms")
    val durationMs: Long? = null,
    @SerialName("progress_percent")
    val progressPercent: Float? = null,
    val watched: Boolean,
    @SerialName("watched_at")
    val watchedAt: String? = null,
    @SerialName("last_played_at")
    val lastPlayedAt: String? = null,
    @SerialName("updated_at")
    val updatedAt: String? = null,
    val version: Long,
)

@Serializable
data class UpdatePlaybackProgressRequest(
    @SerialName("source_id")
    val sourceId: String? = null,
    @SerialName("position_ms")
    val positionMs: Long,
    @SerialName("duration_ms")
    val durationMs: Long? = null,
    @SerialName("reported_at")
    val reportedAt: String? = null,
)

@Serializable
data class SetWatchedStateRequest(
    val watched: Boolean,
    @SerialName("source_id")
    val sourceId: String? = null,
    @SerialName("position_ms")
    val positionMs: Long? = null,
    @SerialName("duration_ms")
    val durationMs: Long? = null,
    @SerialName("marked_at")
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
    val expectedApiVersion: String = dev.taru.android.connection.TaruPublicApiContract.expectedApiVersion,
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
