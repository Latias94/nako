package dev.nako.android.ui.theme

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class NakoArtworkAccentsTest {
    @Test
    fun artworkAccentIsStableForTheSameSeed() {
        assertEquals(
            NakoArtworkAccents.fromSeed("Library A"),
            NakoArtworkAccents.fromSeed("Library A"),
        )
    }

    @Test
    fun blankArtworkSeedUsesTheNakoFallbackAccent() {
        assertEquals(
            NakoAccentDim,
            NakoArtworkAccents.fromSeed("   ").container,
        )
    }

    @Test
    fun differentArtworkSeedsCanResolveToDifferentAccents() {
        assertNotEquals(
            NakoArtworkAccents.fromSeed("a").container,
            NakoArtworkAccents.fromSeed("b").container,
        )
    }
}
