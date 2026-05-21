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
    when (wireValue) {
        SdkClientPlaybackDecisionMode.DirectPlay.wireValue -> ClientPlaybackMode.DirectPlay
        SdkClientPlaybackDecisionMode.Remux.wireValue -> ClientPlaybackMode.Remux
        SdkClientPlaybackDecisionMode.Transcode.wireValue -> ClientPlaybackMode.Transcode
        else -> ClientPlaybackMode.Unknown
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
    when (wireValue) {
        SdkClientOutputContainer.Hls.wireValue -> ClientOutputContainer.Hls
        SdkClientOutputContainer.Mp4.wireValue -> ClientOutputContainer.Mp4
        SdkClientOutputContainer.Mkv.wireValue -> ClientOutputContainer.Mkv
        else -> ClientOutputContainer.Unknown
    }

private fun SdkClientHardwareAcceleration.toAndroid(): ClientHardwareAcceleration =
    when (wireValue) {
        SdkClientHardwareAcceleration.None.wireValue -> ClientHardwareAcceleration.None
        SdkClientHardwareAcceleration.Vaapi.wireValue -> ClientHardwareAcceleration.Vaapi
        SdkClientHardwareAcceleration.Nvenc.wireValue -> ClientHardwareAcceleration.Nvenc
        SdkClientHardwareAcceleration.QuickSync.wireValue -> ClientHardwareAcceleration.QuickSync
        else -> ClientHardwareAcceleration.Unknown
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
    when (wireValue) {
        SdkTranscodeSessionKind.Remux.wireValue -> ClientTranscodeSessionKind.Remux
        SdkTranscodeSessionKind.HlsTranscode.wireValue -> ClientTranscodeSessionKind.HlsTranscode
        else -> ClientTranscodeSessionKind.Unknown
    }

private fun SdkTranscodeSessionState.toAndroid(): ClientTranscodeSessionState =
    when (wireValue) {
        SdkTranscodeSessionState.Planned.wireValue -> ClientTranscodeSessionState.Planned
        SdkTranscodeSessionState.Starting.wireValue -> ClientTranscodeSessionState.Starting
        SdkTranscodeSessionState.Running.wireValue -> ClientTranscodeSessionState.Running
        SdkTranscodeSessionState.CancelRequested.wireValue -> ClientTranscodeSessionState.CancelRequested
        SdkTranscodeSessionState.Cancelled.wireValue -> ClientTranscodeSessionState.Cancelled
        SdkTranscodeSessionState.Failed.wireValue -> ClientTranscodeSessionState.Failed
        SdkTranscodeSessionState.Finished.wireValue -> ClientTranscodeSessionState.Finished
        else -> ClientTranscodeSessionState.Unknown
    }

private fun SdkTranscodeFailureCategory.toAndroid(): ClientTranscodeFailureCategory =
    when (wireValue) {
        SdkTranscodeFailureCategory.InvalidRequest.wireValue -> ClientTranscodeFailureCategory.InvalidRequest
        SdkTranscodeFailureCategory.Runner.wireValue -> ClientTranscodeFailureCategory.Runner
        SdkTranscodeFailureCategory.Timeout.wireValue -> ClientTranscodeFailureCategory.Timeout
        SdkTranscodeFailureCategory.Storage.wireValue -> ClientTranscodeFailureCategory.Storage
        SdkTranscodeFailureCategory.Stale.wireValue -> ClientTranscodeFailureCategory.Stale
        SdkTranscodeFailureCategory.Cancelled.wireValue -> ClientTranscodeFailureCategory.Cancelled
        SdkTranscodeFailureCategory.Unknown.wireValue -> ClientTranscodeFailureCategory.Unknown
        else -> ClientTranscodeFailureCategory.Unknown
    }
