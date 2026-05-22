package dev.nako.android.ui.screens.player

import android.content.Context
import androidx.media3.common.Player
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.TokenVault
import dev.nako.android.playback.NakoPlaybackClient
import dev.nako.android.player.DevicePlaybackPositionStore
import dev.nako.android.player.PlaybackExitCoordinator
import dev.nako.android.player.PlaybackExitSnapshot
import dev.nako.android.player.PlaybackLaunchRequest
import dev.nako.android.userplayback.NakoUserPlaybackClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow

internal interface PlaybackSessionRuntime {
    val player: Player
    val state: StateFlow<PlayerSessionState>
    fun attach()
    fun prepare()
    fun retry()
    fun back()
    fun dispose()
}

internal interface PlaybackSessionRuntimeFactory {
    fun create(launch: PlaybackLaunchRequest): PlaybackSessionRuntime
}

internal class AndroidPlaybackSessionRuntimeFactory private constructor(
    private val profile: ServerProfile,
    private val tokenVault: TokenVault,
    private val engineFactory: PlayerRouteEngineFactory,
    private val exitEffectRunner: PlaybackExitEffectRunner,
    private val platformSessionFactory: PlayerPlatformSessionFactory,
) : PlaybackSessionRuntimeFactory {
    constructor(
        context: Context,
        profile: ServerProfile,
        tokenVault: TokenVault,
        playbackClient: NakoPlaybackClient,
        userPlaybackClient: NakoUserPlaybackClient,
        positionStore: DevicePlaybackPositionStore,
        exitEffectScope: CoroutineScope,
    ) : this(
        profile = profile,
        tokenVault = tokenVault,
        engineFactory = Media3PlayerRouteEngineFactory(context.applicationContext),
        exitEffectRunner = CoroutinePlaybackExitEffectRunner(
            profile = profile,
            tokenVault = tokenVault,
            exitCoordinator = PlaybackExitCoordinator(
                playbackClient = playbackClient,
                userPlaybackClient = userPlaybackClient,
                positionStore = positionStore,
            ),
            exitEffectScope = exitEffectScope,
        ),
        platformSessionFactory = AndroidMediaSessionPlayerPlatformSessionFactory(context.applicationContext),
    )

    override fun create(launch: PlaybackLaunchRequest): PlaybackSessionRuntime {
        val accessToken = tokenVault.readToken(profile.tokenReference).orEmpty()
        return PlayerRouteHost(
            launch = launch,
            engine = engineFactory.create(accessToken),
            exitEffectRunner = exitEffectRunner,
            platformSessionFactory = platformSessionFactory,
        )
    }

    companion object {
        fun fromDependencies(
            profile: ServerProfile,
            tokenVault: TokenVault,
            engineFactory: PlayerRouteEngineFactory,
            exitEffectRunner: PlaybackExitEffectRunner,
            platformSessionFactory: PlayerPlatformSessionFactory = NoOpPlayerPlatformSessionFactory,
        ): AndroidPlaybackSessionRuntimeFactory =
            AndroidPlaybackSessionRuntimeFactory(
                profile = profile,
                tokenVault = tokenVault,
                engineFactory = engineFactory,
                exitEffectRunner = exitEffectRunner,
                platformSessionFactory = platformSessionFactory,
            )
    }
}

internal interface PlayerRouteEngineFactory {
    fun create(accessToken: String): PlayerRouteEngine
}

private class Media3PlayerRouteEngineFactory(
    private val context: Context,
) : PlayerRouteEngineFactory {
    override fun create(accessToken: String): PlayerRouteEngine =
        PlaybackControllerRouteEngine(
            media3PlaybackEngineController(
                context = context,
                accessToken = accessToken,
            ),
        )
}

internal interface PlayerRouteEngine {
    val player: Player
    val playbackState: Int
    val isPlaying: Boolean
    fun prepare(launch: PlaybackLaunchRequest)
    fun addListener(listener: PlayerRouteEngineListener)
    fun removeListener(listener: PlayerRouteEngineListener)
    fun snapshot(): PlaybackExitSnapshot
    fun release()
}

internal interface PlayerRouteEngineListener {
    fun onPlaybackStateChanged(playbackState: Int)
    fun onIsPlayingChanged(isPlaying: Boolean)
    fun onPlayerError(errorCodeName: String?)
}

internal interface PlayerPlatformSessionFactory {
    fun create(playerProvider: () -> Player): PlayerPlatformSession
}

internal interface PlayerPlatformSession {
    fun onPlaybackStateChanged(playbackState: Int, isPlaying: Boolean)
    fun release()
}

internal object NoOpPlayerPlatformSessionFactory : PlayerPlatformSessionFactory {
    override fun create(playerProvider: () -> Player): PlayerPlatformSession =
        NoOpPlayerPlatformSession
}

private object NoOpPlayerPlatformSession : PlayerPlatformSession {
    override fun onPlaybackStateChanged(playbackState: Int, isPlaying: Boolean) = Unit
    override fun release() = Unit
}
