package dev.taru.android.ui.theme

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class TaruArtworkAccentsTest {
    @Test
    fun artworkAccentIsStableForTheSameSeed() {
        assertEquals(
            TaruArtworkAccents.fromSeed("Library A"),
            TaruArtworkAccents.fromSeed("Library A"),
        )
    }

    @Test
    fun blankArtworkSeedUsesTheTaruFallbackAccent() {
        assertEquals(
            TaruAccentDim,
            TaruArtworkAccents.fromSeed("   ").container,
        )
    }

    @Test
    fun differentArtworkSeedsCanResolveToDifferentAccents() {
        assertNotEquals(
            TaruArtworkAccents.fromSeed("a").container,
            TaruArtworkAccents.fromSeed("b").container,
        )
    }
}
