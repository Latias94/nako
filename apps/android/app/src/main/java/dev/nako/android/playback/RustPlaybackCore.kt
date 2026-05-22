package dev.nako.android.playback

import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.toAndroidRequest
import uniffi.nako_client_uniffi.CoreHttpRequest
import uniffi.nako_client_uniffi.CoreOutputContainer
import uniffi.nako_client_uniffi.CorePlaybackCapabilities
import uniffi.nako_client_uniffi.CorePlaybackDecisionSummary
import uniffi.nako_client_uniffi.CorePlaybackMode

object RustPlaybackCore : PlaybackCore {
    override fun playbackDecisionRequest(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
        capabilities: PlaybackCapabilities,
    ): PlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildPlaybackDecisionRequest(
            baseUrl = profile.baseUrl,
            accessToken = accessToken,
            sourceId = sourceId,
            capabilities = capabilities.toCore(),
        ).toPlaybackDescriptor()

    override fun recommendedPlaybackTarget(
        profile: ServerProfile,
        decision: PlaybackDecisionResponse,
        capabilities: PlaybackCapabilities,
    ): PlaybackRequestTarget? =
        uniffi.nako_client_uniffi.buildRecommendedPlaybackTarget(
            baseUrl = profile.baseUrl,
            decision = decision.toCoreSummary(),
            capabilities = capabilities.toCore(),
        )?.let { coreTarget ->
            PlaybackRequestTarget(
                request = coreTarget.request.toPlaybackDescriptor(),
                sessionProbeRequest = coreTarget.sessionProbeRequest?.toPlaybackDescriptor(),
            )
        }

    override fun directPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
    ): PlaybackRequestTarget =
        uniffi.nako_client_uniffi.buildDirectPlaybackTarget(
            baseUrl = profile.baseUrl,
            sourceId = sourceId,
        ).toPlaybackTarget()

    override fun headDirectPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
    ): PlaybackRequestTarget =
        uniffi.nako_client_uniffi.buildHeadDirectPlaybackTarget(
            baseUrl = profile.baseUrl,
            sourceId = sourceId,
        ).toPlaybackTarget()

    override fun remuxPlaybackTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities,
        outputContainer: ClientOutputContainer?,
    ): PlaybackRequestTarget =
        uniffi.nako_client_uniffi.buildRemuxPlaybackTarget(
            baseUrl = profile.baseUrl,
            sourceId = sourceId,
            capabilities = capabilities.toCore(),
            outputContainer = outputContainer?.toCore(),
        ).toPlaybackTarget()

    override fun hlsPlaylistTarget(
        profile: ServerProfile,
        sourceId: String,
        capabilities: PlaybackCapabilities,
    ): PlaybackRequestTarget =
        uniffi.nako_client_uniffi.buildHlsPlaylistTarget(
            baseUrl = profile.baseUrl,
            sourceId = sourceId,
            capabilities = capabilities.toCore(),
        ).toPlaybackTarget()

    override fun hlsSegmentTarget(
        profile: ServerProfile,
        sessionId: String,
        segmentName: String,
    ): PlaybackRequestTarget =
        PlaybackRequestTarget(
            request = uniffi.nako_client_uniffi.buildHlsSegmentRequest(
                baseUrl = profile.baseUrl,
                sessionId = sessionId,
                segmentName = segmentName,
            ).toPlaybackDescriptor(),
        )

    override fun sourceProbeRequest(
        profile: ServerProfile,
        accessToken: String,
        sourceId: String,
    ): PlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildSourceProbeRequest(
            uniffi.nako_client_uniffi.CorePlaybackSourceRequestInput(
                baseUrl = profile.baseUrl,
                accessToken = accessToken,
                sourceId = sourceId,
            ),
        ).toPlaybackDescriptor()

    override fun playbackSessionRequest(
        profile: ServerProfile,
        accessToken: String,
        sessionId: String,
    ): PlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildGetPlaybackSessionRequest(
            uniffi.nako_client_uniffi.CorePlaybackSessionRequestInput(
                baseUrl = profile.baseUrl,
                accessToken = accessToken,
                sessionId = sessionId,
            ),
        ).toPlaybackDescriptor()

    override fun cancelPlaybackSessionRequest(
        profile: ServerProfile,
        accessToken: String,
        sessionId: String,
    ): PlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildCancelPlaybackSessionRequest(
            uniffi.nako_client_uniffi.CorePlaybackSessionRequestInput(
                baseUrl = profile.baseUrl,
                accessToken = accessToken,
                sessionId = sessionId,
            ),
        ).toPlaybackDescriptor()
}

private fun uniffi.nako_client_uniffi.CorePlaybackTarget.toPlaybackTarget(): PlaybackRequestTarget =
    PlaybackRequestTarget(
        request = request.toPlaybackDescriptor(),
        sessionProbeRequest = sessionProbeRequest?.toPlaybackDescriptor(),
    )

private fun PlaybackCapabilities.toCore(): CorePlaybackCapabilities =
    CorePlaybackCapabilities(
        directPlay = directPlay,
        containers = containers,
        videoCodecs = videoCodecs,
        audioCodecs = audioCodecs,
    )

private fun PlaybackDecisionResponse.toCoreSummary(): CorePlaybackDecisionSummary =
    CorePlaybackDecisionSummary(
        sourceId = source.id,
        mode = decision.mode.toCore(),
        transcodeOutputContainer = decision.transcodePlan?.outputContainer?.toCore(),
    )

private fun ClientPlaybackMode.toCore(): CorePlaybackMode =
    when (this) {
        ClientPlaybackMode.DirectPlay -> CorePlaybackMode.DIRECT_PLAY
        ClientPlaybackMode.Remux -> CorePlaybackMode.REMUX
        ClientPlaybackMode.Transcode -> CorePlaybackMode.TRANSCODE
        ClientPlaybackMode.Unknown -> CorePlaybackMode.UNKNOWN
    }

private fun ClientOutputContainer.toCore(): CoreOutputContainer =
    when (this) {
        ClientOutputContainer.Hls -> CoreOutputContainer.HLS
        ClientOutputContainer.Mp4 -> CoreOutputContainer.MP4
        ClientOutputContainer.Mkv -> CoreOutputContainer.MKV
        ClientOutputContainer.Unknown -> CoreOutputContainer.UNKNOWN
    }

private fun CoreHttpRequest.toPlaybackDescriptor(): PlaybackRequestDescriptor {
    val request = toAndroidRequest()
    return PlaybackRequestDescriptor(
        method = request.method,
        url = request.url,
        headers = request.headers.filterKeys { !it.equals("Authorization", ignoreCase = true) },
    )
}
