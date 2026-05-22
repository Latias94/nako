package dev.nako.android.ui.theme

import androidx.compose.ui.graphics.Color

data class NakoArtworkAccent(
    val container: Color,
    val onContainer: Color,
    val outline: Color,
)

object NakoArtworkAccents {
    private val palette = listOf(
        NakoAccentDim,
        Color(0xFF28465A),
        Color(0xFF3A3E5E),
        Color(0xFF3E4F40),
        Color(0xFF5A4338),
        Color(0xFF25504A),
    )

    fun fromSeed(seed: String?): NakoArtworkAccent {
        val normalized = seed?.trim().orEmpty()
        val container = if (normalized.isBlank()) {
            NakoAccentDim
        } else {
            palette[kotlin.math.abs(normalized.hashCode()) % palette.size]
        }

        return NakoArtworkAccent(
            container = container,
            onContainer = NakoTextPrimary,
            outline = container.copy(alpha = 0.72f),
        )
    }
}
