package dev.taru.android.ui.screens.sourcepicker

import dev.taru.android.browse.MediaSourceDto
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
        assertTrue(visibleText.contains("Media Library library-a"))
        assertFalse(visibleText.contains("file://"))
        assertFalse(visibleText.contains("/srv/private"))
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
        assertEquals("Server work required", hls.warning)
        assertTrue(hls.consequence.contains("Server prepares"))
    }
}
