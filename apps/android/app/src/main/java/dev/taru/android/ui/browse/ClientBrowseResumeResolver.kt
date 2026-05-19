package dev.taru.android.ui.browse

import dev.taru.android.player.DevicePlaybackPositionStore
import dev.taru.android.player.ResumePlaybackPosition
import dev.taru.android.player.resolvePlaybackResumePosition

internal class ClientBrowseResumeResolver(
    private val serverProfileId: String,
    private val positionStore: DevicePlaybackPositionStore,
) : BrowseResumeResolver {
    override fun resolve(
        detailState: ItemDetailUiState,
        selectedSourceId: String?,
    ): ResumePlaybackPosition? {
        val content = detailState as? ItemDetailUiState.Content ?: return null
        val detail = content.response
        val source = detail.sources.firstOrNull { it.id == selectedSourceId } ?: detail.sources.firstOrNull()
        val sourceId = source?.id?.takeIf { it.isNotBlank() } ?: return null
        return resolvePlaybackResumePosition(
            profileId = serverProfileId,
            mediaItemId = detail.item.id,
            sourceId = sourceId,
            userPlaybackState = content.userPlaybackState,
            positionStore = positionStore,
        )
    }
}
