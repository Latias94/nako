package dev.nako.android.ui.screens.player

import android.app.Activity
import android.app.PictureInPictureParams
import android.content.Context
import android.content.ContextWrapper
import android.os.Build
import android.util.Rational
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import dev.nako.android.player.PlaybackLaunchRequest

internal data class PlaybackPictureInPictureRequest(
    val aspectRatioWidth: Int,
    val aspectRatioHeight: Int,
)

internal fun playbackPictureInPictureRequest(launch: PlaybackLaunchRequest): PlaybackPictureInPictureRequest =
    PlaybackPictureInPictureRequest(
        aspectRatioWidth = 16,
        aspectRatioHeight = 9,
    )

internal interface PlaybackPictureInPictureGateway {
    val isAvailable: Boolean
    fun enter(launch: PlaybackLaunchRequest): Boolean
}

@Composable
internal fun rememberPlaybackPictureInPictureGateway(): PlaybackPictureInPictureGateway {
    val context = LocalContext.current
    return remember(context) {
        ActivityPlaybackPictureInPictureGateway(context.findActivity())
    }
}

private class ActivityPlaybackPictureInPictureGateway(
    private val activity: Activity?,
) : PlaybackPictureInPictureGateway {
    override val isAvailable: Boolean
        get() = Build.VERSION.SDK_INT >= Build.VERSION_CODES.O && activity != null

    override fun enter(launch: PlaybackLaunchRequest): Boolean {
        val targetActivity = activity ?: return false
        if (!isAvailable) {
            return false
        }
        return targetActivity.enterPictureInPictureMode(
            playbackPictureInPictureRequest(launch).toPlatformParams(),
        )
    }
}

private fun PlaybackPictureInPictureRequest.toPlatformParams(): PictureInPictureParams =
    PictureInPictureParams.Builder()
        .setAspectRatio(Rational(aspectRatioWidth, aspectRatioHeight))
        .build()

private tailrec fun Context.findActivity(): Activity? =
    when (this) {
        is Activity -> this
        is ContextWrapper -> baseContext.findActivity()
        else -> null
    }
