package dev.nako.android.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.unit.dp

object NakoSpacing {
    val xsmall = 4.dp
    val small = 8.dp
    val medium = 12.dp
    val large = 16.dp
    val xlarge = 24.dp
    val xxlarge = 32.dp
}

object NakoShape {
    val small = RoundedCornerShape(6.dp)
    val medium = RoundedCornerShape(8.dp)
    val large = RoundedCornerShape(16.dp)
    val expressive = RoundedCornerShape(24.dp)
}

object NakoAspectRatio {
    const val poster = 2f / 3f
    const val backdrop = 16f / 9f
}

object NakoTouchTarget {
    val minimum = 48.dp
}

object NakoMotion {
    const val pressMillis = 120
    const val routeEnterMillis = 220
    const val routeExitMillis = 140
    const val stateMillis = 180
    const val loadingPulseMillis = 900
}

object NakoElevation {
    val flat = 0.dp
    val raised = 1.dp
    val floating = 6.dp
}
