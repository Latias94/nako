package dev.taru.android.playback

import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.PlaybackLaunchRequest
import dev.taru.android.player.playbackLaunchRequest
import dev.taru.android.player.resolvePlaybackResumePosition
import dev.taru.android.userplayback.UserPlaybackStateDto

internal data class PlaybackStartRequest(
    val title: String,
    val mediaItemId: String,
    val sourceId: String,
    val decision: PlaybackDecisionResponse,
    val capabilities: PlaybackCapabilities,
    val target: PlaybackRequestTarget,
    val userPlaybackState: UserPlaybackStateDto? = null,
)

internal sealed interface PlaybackStartResult {
    data class Success(
        val launch: PlaybackLaunchRequest,
        val preparedTarget: PlaybackRequestTarget,
    ) : PlaybackStartResult

    data class Failure(
        val diagnostics: SafePlaybackDiagnostics,
    ) : PlaybackStartResult
}

internal class PlaybackStartCoordinator(
    private val playbackClient: TaruPlaybackClient,
    private val positionStore: DevicePlaybackPositionStore,
) {
    suspend fun start(
        profile: ServerProfile,
        tokenVault: TokenVault,
        request: PlaybackStartRequest,
    ): PlaybackStartResult {
        val preparedTarget = when (val prepared = prepareTarget(profile, tokenVault, request)) {
            is PlaybackResult.Success -> prepared.value
            is PlaybackResult.Failure -> return PlaybackStartResult.Failure(prepared.diagnostics)
        }
        val resumePosition = resolvePlaybackResumePosition(
            profileId = profile.id,
            mediaItemId = request.mediaItemId,
            sourceId = request.sourceId,
            userPlaybackState = request.userPlaybackState,
            positionStore = positionStore,
        )

        return PlaybackStartResult.Success(
            launch = playbackLaunchRequest(
                title = request.title.ifBlank { "Taru Playback" },
                target = preparedTarget,
                serverProfileId = profile.id,
                mediaItemId = request.mediaItemId,
                sourceId = request.sourceId,
                playbackMode = request.decision.decision.mode,
                sessionId = preparedTarget.sessionId,
                resumePositionMs = resumePosition?.positionMs,
                resumeSource = resumePosition?.source,
            ),
            preparedTarget = preparedTarget,
        )
    }

    private suspend fun prepareTarget(
        profile: ServerProfile,
        tokenVault: TokenVault,
        request: PlaybackStartRequest,
    ): PlaybackResult<PlaybackRequestTarget> {
        if (!request.target.sessionId.isNullOrBlank()) {
            return PlaybackResult.Success(request.target, request.target.safeRequest)
        }

        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        if (accessToken.isBlank()) {
            return PlaybackResult.Failure(
                SafePlaybackDiagnostics(
                    category = PlaybackFailureCategory.MissingAccessToken,
                    userMessage = "Sign in again before requesting playback.",
                ),
            )
        }

        return playbackClient.prepareRecommendedPlaybackTarget(
            profile = profile,
            accessToken = accessToken,
            decision = request.decision,
            capabilities = request.capabilities,
        )
    }
}
