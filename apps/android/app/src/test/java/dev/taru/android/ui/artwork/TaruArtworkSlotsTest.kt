package dev.taru.android.ui.artwork

import org.junit.Assert.assertEquals
import org.junit.Test

class TaruArtworkSlotsTest {
    @Test
    fun fallbackPresentationUsesTitleInitialAndReadableKind() {
        val presentation = artworkFallbackPresentation(
            title = " night harbor ",
            kind = "media_item",
        )

        assertEquals("N", presentation.initial)
        assertEquals("Media Item", presentation.kindLabel)
        assertEquals("night harbor", presentation.seed)
    }

    @Test
    fun fallbackPresentationDoesNotCreateFakeArtworkForBlankMedia() {
        val presentation = artworkFallbackPresentation(
            title = " ",
            kind = null,
        )

        assertEquals("T", presentation.initial)
        assertEquals("Media", presentation.kindLabel)
        assertEquals("Media", presentation.seed)
    }

    @Test
    fun fallbackPresentationNormalizesHyphenatedKinds() {
        val presentation = artworkFallbackPresentation(
            title = "Episode 3",
            kind = "source-variant",
        )

        assertEquals("E", presentation.initial)
        assertEquals("Source Variant", presentation.kindLabel)
    }
}
