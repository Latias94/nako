package dev.taru.android.ui.browse

import dev.taru.android.playback.PlaybackRequestTarget
import dev.taru.android.playback.PlaybackStartRequest
import dev.taru.android.playback.PlaybackStartResult
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class BrowsePlaybackSession(
    private val store: BrowseSessionStore,
    private val scope: CoroutineScope,
    private val playbackStarter: BrowsePlaybackStarter,
    private val navigation: BrowseSessionNavigation,
) {
    fun startPlayback(target: PlaybackRequestTarget): Job? {
        val current = store.value
        val detailContent = current.detailState as? ItemDetailUiState.Content ?: return null
        val playbackContent = current.playbackState as? PlaybackSelectionUiState.Content ?: return null
        val detail = detailContent.response
        val item = detail.item
        val sourceId = current.selectedSourceId
            ?: detail.sources.firstOrNull()?.id
            ?: playbackContent.response.source.id
        if (sourceId.isBlank()) {
            return null
        }

        store.update { it.copy(playbackState = PlaybackSelectionUiState.Loading) }
        return scope.launch {
            when (
                val start = playbackStarter.start(
                    PlaybackStartRequest(
                        title = item.metadata.title,
                        mediaItemId = item.id,
                        sourceId = sourceId,
                        decision = playbackContent.response,
                        capabilities = playbackContent.capabilities,
                        target = target,
                        userPlaybackState = detailContent.userPlaybackState,
                    ),
                )
            ) {
                is PlaybackStartResult.Success -> {
                    store.update { beforeOpen ->
                        beforeOpen.copy(
                            playbackState = playbackContent.copy(target = start.preparedTarget),
                        )
                    }
                    store.set(
                        navigation.reduceAndPrepare(
                            current = store.value,
                            action = BrowseAction.OpenPlayer(start.launch),
                        ),
                    )
                }
                is PlaybackStartResult.Failure -> {
                    store.update {
                        it.copy(
                            playbackState = PlaybackSelectionUiState.Failure(start.diagnostics),
                        )
                    }
                }
            }
        }
    }
}
