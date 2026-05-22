package dev.taru.android.ui.screens.player

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.TokenVault
import dev.taru.android.playback.TaruPlaybackClient
import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.userplayback.TaruUserPlaybackClient
import kotlinx.coroutines.CoroutineScope

@Composable
internal fun rememberAndroidPlaybackSessionRuntimeFactory(
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: TaruPlaybackClient,
    userPlaybackClient: TaruUserPlaybackClient,
    positionStore: DevicePlaybackPositionStore,
    exitEffectScope: CoroutineScope,
): PlaybackSessionRuntimeFactory =
    LocalContext.current.let { context ->
        remember(context, profile, tokenVault, playbackClient, userPlaybackClient, positionStore, exitEffectScope) {
            AndroidPlaybackSessionRuntimeFactory(
                context = context,
                profile = profile,
                tokenVault = tokenVault,
                playbackClient = playbackClient,
                userPlaybackClient = userPlaybackClient,
                positionStore = positionStore,
                exitEffectScope = exitEffectScope,
            )
        }
    }
