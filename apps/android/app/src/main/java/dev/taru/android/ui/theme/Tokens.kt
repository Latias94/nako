package dev.taru.android.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.unit.dp

object TaruSpacing {
    val xsmall = 4.dp
    val small = 8.dp
    val medium = 12.dp
    val large = 16.dp
    val xlarge = 24.dp
    val xxlarge = 32.dp
}

object TaruShape {
    val small = RoundedCornerShape(6.dp)
    val medium = RoundedCornerShape(8.dp)
}

object TaruAspectRatio {
    const val poster = 2f / 3f
    const val backdrop = 16f / 9f
}

object TaruTouchTarget {
    val minimum = 48.dp
}
