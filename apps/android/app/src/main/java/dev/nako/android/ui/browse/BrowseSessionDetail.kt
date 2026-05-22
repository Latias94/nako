package dev.nako.android.ui.browse

import dev.nako.android.player.ResumePlaybackPosition
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal class BrowseItemDetailSession(
    private val store: BrowseSessionStore,
    private val scope: CoroutineScope,
    private val dataSource: BrowseDataSource,
    private val routeStatePolicy: BrowseRouteStatePolicy,
    private val resolveResumePosition: (ItemDetailUiState, String?) -> ResumePlaybackPosition?,
) {
    fun selectSource(sourceId: String): Job {
        routeStatePolicy.beginSourceAndPlaybackSelection()
        store.update {
            it.copy(
                selectedSourceId = sourceId,
                resumePosition = resolveResumePosition(it.detailState, sourceId),
                sourceProbeState = SourceProbeUiState.Loading,
                playbackRequestSourceId = null,
                playbackState = PlaybackSelectionUiState.Idle,
            )
        }
        return loadSourceProbe(sourceId)
    }

    fun requestPlayback(sourceId: String): List<Job> {
        routeStatePolicy.beginSourceAndPlaybackSelection()
        store.update {
            it.copy(
                selectedSourceId = sourceId,
                resumePosition = resolveResumePosition(it.detailState, sourceId),
                sourceProbeState = SourceProbeUiState.Loading,
                playbackRequestSourceId = sourceId,
                playbackState = PlaybackSelectionUiState.Loading,
            )
        }
        return listOf(
            loadSourceProbe(sourceId),
            loadPlaybackSelection(sourceId),
        )
    }

    fun loadItemDetail(itemId: String): Job {
        val requestId = routeStatePolicy.beginItemDetail()
        store.update {
            it.copy(
                detailState = ItemDetailUiState.Loading,
                selectedSourceId = null,
                resumePosition = null,
                sourceProbeState = SourceProbeUiState.Idle,
                playbackRequestSourceId = null,
                playbackState = PlaybackSelectionUiState.Idle,
            )
        }
        return scope.launch {
            val nextState = dataSource.loadItemDetail(itemId)
            var acceptedSourceId: String? = null
            store.update { current ->
                val routeStillCurrent = current.currentRoute == NakoRoute.ItemDetail(itemId)
                if (routeStatePolicy.acceptsItemDetail(requestId) && routeStillCurrent) {
                    val selectedSourceId = nextState.firstSourceIdOrNull()
                    acceptedSourceId = selectedSourceId
                    current.copy(
                        detailState = nextState,
                        selectedSourceId = selectedSourceId,
                        resumePosition = resolveResumePosition(nextState, selectedSourceId),
                        sourceProbeState = if (selectedSourceId == null) {
                            SourceProbeUiState.Idle
                        } else {
                            SourceProbeUiState.Loading
                        },
                        playbackRequestSourceId = null,
                        playbackState = PlaybackSelectionUiState.Idle,
                    )
                } else {
                    current
                }
            }
            acceptedSourceId?.let { sourceId ->
                if (store.value.currentRoute == NakoRoute.ItemDetail(itemId)) {
                    loadSourceProbe(sourceId)
                }
            }
        }
    }

    fun loadSelectedSourceProbe(): Job? {
        val sourceId = store.value.selectedSourceId?.takeIf { it.isNotBlank() }
        return if (sourceId == null || store.value.currentRoute !is NakoRoute.ItemDetail) {
            routeStatePolicy.invalidateSourceProbe()
            store.update { it.copy(sourceProbeState = SourceProbeUiState.Idle) }
            null
        } else {
            loadSourceProbe(sourceId)
        }
    }

    fun loadRequestedPlaybackSelection(): Job? {
        val sourceId = store.value.playbackRequestSourceId?.takeIf { it.isNotBlank() }
        return if (sourceId == null || store.value.currentRoute !is NakoRoute.ItemDetail) {
            routeStatePolicy.invalidatePlaybackSelection()
            store.update { it.copy(playbackState = PlaybackSelectionUiState.Idle) }
            null
        } else {
            loadPlaybackSelection(sourceId)
        }
    }

    private fun loadSourceProbe(sourceId: String): Job {
        val requestId = routeStatePolicy.beginSourceProbe()
        store.update { it.copy(sourceProbeState = SourceProbeUiState.Loading) }
        return scope.launch {
            val nextState = dataSource.loadSourceProbe(sourceId)
            store.update { current ->
                val routeStillCurrent = current.currentRoute is NakoRoute.ItemDetail
                if (
                    routeStatePolicy.acceptsSourceProbe(requestId) &&
                    routeStillCurrent &&
                    current.selectedSourceId == sourceId
                ) {
                    current.copy(sourceProbeState = nextState)
                } else {
                    current
                }
            }
        }
    }

    private fun loadPlaybackSelection(sourceId: String): Job {
        val requestId = routeStatePolicy.beginPlaybackSelection()
        store.update { it.copy(playbackState = PlaybackSelectionUiState.Loading) }
        return scope.launch {
            val nextState = dataSource.loadPlaybackSelection(sourceId)
            store.update { current ->
                val routeStillCurrent = current.currentRoute is NakoRoute.ItemDetail
                if (
                    routeStatePolicy.acceptsPlaybackSelection(requestId) &&
                    routeStillCurrent &&
                    current.playbackRequestSourceId == sourceId
                ) {
                    current.copy(playbackState = nextState)
                } else {
                    current
                }
            }
        }
    }
}

private fun ItemDetailUiState.firstSourceIdOrNull(): String? =
    (this as? ItemDetailUiState.Content)
        ?.response
        ?.sources
        ?.firstOrNull()
        ?.id
