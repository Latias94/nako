package dev.taru.android.ui.screens.sourcepicker

import dev.taru.android.browse.MediaSourceDto
import dev.taru.android.media.ClientMediaStreamKind
import dev.taru.android.media.MediaProbeDto
import dev.taru.android.media.MediaStreamDto
import dev.taru.android.playback.ClientHardwareAcceleration
import dev.taru.android.playback.ClientOutputContainer
import dev.taru.android.playback.ClientPlaybackDecision
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.ClientTranscodePlan
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SourcePickerDisplayModelTest {
    @Test
    fun sourcePickerModelUsesClientSafeFactsWithoutLocator() {
        val model = sourcePickerDisplayModel(
            source = MediaSourceDto(
                id = "source-1",
                libraryId = "library-a",
                locator = "file:///srv/private/media/night-harbor.mkv",
                fileName = "night-harbor.mkv",
                sizeBytes = 1_073_741_824L,
                fingerprint = "fingerprint",
            ),
            index = 0,
            selected = true,
            activeDecision = null,
        )

        val visibleText = buildList {
            add(model.primaryLabel)
            add(model.secondaryText)
            addAll(model.factLabels)
        }.joinToString(" ")

        assertTrue(visibleText.contains("night-harbor.mkv"))
        assertTrue(visibleText.contains("Library library-a"))
        assertFalse(visibleText.contains("file://"))
        assertFalse(visibleText.contains("/srv/private"))
    }

    @Test
    fun sourcePickerAccessibilityModelDescribesSelectionAndAction() {
        val selected = sourcePickerDisplayModel(
            source = MediaSourceDto(
                id = "source-1",
                libraryId = "library-a",
                fileName = "night-harbor.mkv",
                sizeBytes = 42,
            ),
            index = 0,
            selected = true,
            activeDecision = null,
        )
        val unselected = selected.copy(selected = false)

        assertEquals("Selected version: night-harbor.mkv. Library library-a / Version 1.", selected.accessibilityLabel)
        assertEquals("Choose version: night-harbor.mkv. Library library-a / Version 1.", unselected.accessibilityLabel)
        assertEquals("Selected", selected.stateDescription)
        assertEquals("Not selected", unselected.stateDescription)
    }

    @Test
    fun sourcePickerFallbackCopyUsesVersionLanguage() {
        val model = sourcePickerDisplayModel(
            source = MediaSourceDto(
                id = "source-1",
                libraryId = "",
                fileName = "",
            ),
            index = 1,
            selected = false,
            activeDecision = null,
        )

        assertEquals("Version 2", model.primaryLabel)
        assertFalse(model.primaryLabel.contains("Media Source"))
        assertTrue(model.accessibilityLabel.startsWith("Choose version"))
    }

    @Test
    fun probeFactLabelsSummarizeTechnicalFacts() {
        val labels = probeFactLabels(
            MediaProbeDto(
                durationMs = 7_200_000,
                container = "matroska",
                bitRate = 12_000_000,
                streams = listOf(
                    MediaStreamDto(
                        index = 0,
                        kind = ClientMediaStreamKind.Video,
                        codec = "h265",
                        width = 3840,
                        height = 2160,
                    ),
                    MediaStreamDto(
                        index = 1,
                        kind = ClientMediaStreamKind.Audio,
                        codec = "aac",
                    ),
                    MediaStreamDto(
                        index = 2,
                        kind = ClientMediaStreamKind.Subtitle,
                        codec = "subrip",
                    ),
                ),
            ),
        )

        assertEquals(
            listOf("MATROSKA", "2h 0m", "12 Mbps", "3840x2160 / h265", "1 audio", "1 subtitle"),
            labels,
        )
    }

    @Test
    fun playbackModePresentationExplainsModeConsequences() {
        val direct = playbackModePresentation(
            ClientPlaybackDecision(
                mode = ClientPlaybackMode.DirectPlay,
                reason = "client supports source",
            ),
        )
        val remux = playbackModePresentation(
            ClientPlaybackDecision(
                mode = ClientPlaybackMode.Remux,
                reason = "container change",
            ),
        )
        val hls = playbackModePresentation(
            ClientPlaybackDecision(
                mode = ClientPlaybackMode.Transcode,
                reason = "adaptive output",
                transcodePlan = ClientTranscodePlan(
                    outputContainer = ClientOutputContainer.Hls,
                    hardwareAcceleration = ClientHardwareAcceleration.None,
                ),
            ),
        )

        assertEquals("Direct", direct.label)
        assertEquals(null, direct.warning)
        assertEquals("Remux", remux.label)
        assertEquals("Container change", remux.warning)
        assertEquals("HLS", hls.label)
        assertEquals("Prepared on server", hls.warning)
        assertTrue(hls.consequence.contains("adaptive stream"))
    }

    @Test
    fun resumeCopyAvoidsInternalUserPlaybackStateLanguage() {
        val serverResume = resumePositionPresentation(
            dev.taru.android.player.ResumePlaybackPosition(
                positionMs = 92_000,
                source = dev.taru.android.player.PlaybackResumeSource.UserPlaybackState,
            ),
        )
        val localResume = resumePositionPresentation(
            dev.taru.android.player.ResumePlaybackPosition(
                positionMs = 92_000,
                source = dev.taru.android.player.PlaybackResumeSource.DeviceLocal,
            ),
        )

        assertEquals("Resume from your last server position", serverResume.title)
        assertFalse(serverResume.body.contains("User Playback State"))
        assertEquals("Resume where this device stopped", localResume.title)
        assertFalse(localResume.body.contains("source"))
    }
}
