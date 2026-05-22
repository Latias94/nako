package dev.nako.android.ui.screens.player

import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.player.DevicePlaybackPositionStore
import dev.nako.android.userplayback.NakoUserPlaybackClient
import kotlinx.coroutines.CoroutineScope

@Composable
internal fun rememberAndroidPlaybackSessionRuntimeFactory(
    profile: ServerProfile,
    tokenVault: TokenVault,
    playbackClient: NakoPlaybackClient,
    userPlaybackClient: NakoUserPlaybackClient,
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
