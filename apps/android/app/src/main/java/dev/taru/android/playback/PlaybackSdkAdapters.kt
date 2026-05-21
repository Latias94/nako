package dev.taru.android.playback

import dev.taru.android.media.toAndroid
import dev.taru.sdk.ClientDirectPlayPlan as SdkClientDirectPlayPlan
import dev.taru.sdk.ClientPlaybackDecision as SdkClientPlaybackDecision
import dev.taru.sdk.ClientPlaybackDecisionMode as SdkClientPlaybackDecisionMode
import dev.taru.sdk.ClientTranscodePlan as SdkClientTranscodePlan
import dev.taru.sdk.ClientTranscodePlanHardwareAcceleration as SdkClientHardwareAcceleration
import dev.taru.sdk.ClientTranscodePlanOutputContainer as SdkClientOutputContainer
import dev.taru.sdk.MediaSourceDto as SdkMediaSourceDto
import dev.taru.sdk.PlaybackDecisionResponse as SdkPlaybackDecisionResponse
import dev.taru.sdk.TranscodeSessionDto as SdkTranscodeSessionDto
import dev.taru.sdk.TranscodeSessionDtoFailureCategory as SdkTranscodeFailureCategory
import dev.taru.sdk.TranscodeSessionDtoKind as SdkTranscodeSessionKind
import dev.taru.sdk.TranscodeSessionDtoState as SdkTranscodeSessionState
import dev.taru.sdk.TranscodeSessionResponse as SdkTranscodeSessionResponse

internal fun SdkPlaybackDecisionResponse.toAndroid(): PlaybackDecisionResponse =
    PlaybackDecisionResponse(
        source = source.toAndroidPlaybackSource(),
        probe = probe?.toAndroid(),
        decision = decision.toAndroid(),
    )

private fun SdkMediaSourceDto.toAndroidPlaybackSource(): PlaybackMediaSourceDto =
    PlaybackMediaSourceDto(
        id = id,
        libraryId = libraryId,
        itemId = itemId,
        fileName = fileName,
        sizeBytes = sizeBytes,
        fingerprint = fingerprint,
    )

private fun SdkClientPlaybackDecision.toAndroid(): ClientPlaybackDecision =
    ClientPlaybackDecision(
        mode = mode.toAndroid(),
        reason = reason,
        directPlay = directPlay?.toAndroid(),
        transcodePlan = transcodePlan?.toAndroid(),
    )

private fun SdkClientPlaybackDecisionMode.toAndroid(): ClientPlaybackMode =
    when (this) {
        SdkClientPlaybackDecisionMode.DirectPlay -> ClientPlaybackMode.DirectPlay
        SdkClientPlaybackDecisionMode.Remux -> ClientPlaybackMode.Remux
        SdkClientPlaybackDecisionMode.Transcode -> ClientPlaybackMode.Transcode
    }

private fun SdkClientDirectPlayPlan.toAndroid(): ClientDirectPlayPlan =
    ClientDirectPlayPlan(
        sourceId = sourceId,
        contentType = contentType,
        supportsRangeRequests = supportsRangeRequests,
    )

private fun SdkClientTranscodePlan.toAndroid(): ClientTranscodePlan =
    ClientTranscodePlan(
        outputContainer = outputContainer.toAndroid(),
        videoCodec = videoCodec,
        audioCodec = audioCodec,
        hardwareAcceleration = hardwareAcceleration.toAndroid(),
    )

private fun SdkClientOutputContainer.toAndroid(): ClientOutputContainer =
    when (this) {
        SdkClientOutputContainer.Hls -> ClientOutputContainer.Hls
        SdkClientOutputContainer.Mp4 -> ClientOutputContainer.Mp4
        SdkClientOutputContainer.Mkv -> ClientOutputContainer.Mkv
    }

private fun SdkClientHardwareAcceleration.toAndroid(): ClientHardwareAcceleration =
    when (this) {
        SdkClientHardwareAcceleration.None -> ClientHardwareAcceleration.None
        SdkClientHardwareAcceleration.Vaapi -> ClientHardwareAcceleration.Vaapi
        SdkClientHardwareAcceleration.Nvenc -> ClientHardwareAcceleration.Nvenc
        SdkClientHardwareAcceleration.QuickSync -> ClientHardwareAcceleration.QuickSync
    }

internal fun SdkTranscodeSessionResponse.toAndroid(): TranscodeSessionResponse =
    TranscodeSessionResponse(session = session.toAndroid())

private fun SdkTranscodeSessionDto.toAndroid(): TranscodeSessionDto =
    TranscodeSessionDto(
        id = id,
        sourceId = sourceId,
        kind = kind.toAndroid(),
        requestKey = requestKey,
        state = state.toAndroid(),
        failureCategory = failureCategory?.toAndroid(),
        failureMessage = failureMessage,
        createdAt = createdAt,
        updatedAt = updatedAt,
        startedAt = startedAt,
        completedAt = completedAt,
    )

private fun SdkTranscodeSessionKind.toAndroid(): ClientTranscodeSessionKind =
    when (this) {
        SdkTranscodeSessionKind.Remux -> ClientTranscodeSessionKind.Remux
        SdkTranscodeSessionKind.HlsTranscode -> ClientTranscodeSessionKind.HlsTranscode
    }

private fun SdkTranscodeSessionState.toAndroid(): ClientTranscodeSessionState =
    when (this) {
        SdkTranscodeSessionState.Planned -> ClientTranscodeSessionState.Planned
        SdkTranscodeSessionState.Starting -> ClientTranscodeSessionState.Starting
        SdkTranscodeSessionState.Running -> ClientTranscodeSessionState.Running
        SdkTranscodeSessionState.CancelRequested -> ClientTranscodeSessionState.CancelRequested
        SdkTranscodeSessionState.Cancelled -> ClientTranscodeSessionState.Cancelled
        SdkTranscodeSessionState.Failed -> ClientTranscodeSessionState.Failed
        SdkTranscodeSessionState.Finished -> ClientTranscodeSessionState.Finished
    }

private fun SdkTranscodeFailureCategory.toAndroid(): ClientTranscodeFailureCategory =
    when (this) {
        SdkTranscodeFailureCategory.InvalidRequest -> ClientTranscodeFailureCategory.InvalidRequest
        SdkTranscodeFailureCategory.Runner -> ClientTranscodeFailureCategory.Runner
        SdkTranscodeFailureCategory.Timeout -> ClientTranscodeFailureCategory.Timeout
        SdkTranscodeFailureCategory.Storage -> ClientTranscodeFailureCategory.Storage
        SdkTranscodeFailureCategory.Stale -> ClientTranscodeFailureCategory.Stale
        SdkTranscodeFailureCategory.Cancelled -> ClientTranscodeFailureCategory.Cancelled
        SdkTranscodeFailureCategory.Unknown -> ClientTranscodeFailureCategory.Unknown
    }
