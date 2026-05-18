package dev.taru.android.ui.theme

import androidx.compose.ui.graphics.Color

data class TaruArtworkAccent(
    val container: Color,
    val onContainer: Color,
    val outline: Color,
)

object TaruArtworkAccents {
    private val palette = listOf(
        TaruAccentDim,
        Color(0xFF28465A),
        Color(0xFF3A3E5E),
        Color(0xFF3E4F40),
        Color(0xFF5A4338),
        Color(0xFF25504A),
    )

    fun fromSeed(seed: String?): TaruArtworkAccent {
        val normalized = seed?.trim().orEmpty()
        val container = if (normalized.isBlank()) {
            TaruAccentDim
        } else {
            palette[kotlin.math.abs(normalized.hashCode()) % palette.size]
        }

        return TaruArtworkAccent(
            container = container,
            onContainer = TaruTextPrimary,
            outline = container.copy(alpha = 0.72f),
        )
    }
}
