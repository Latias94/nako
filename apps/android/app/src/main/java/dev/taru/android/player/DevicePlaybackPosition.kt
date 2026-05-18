package dev.taru.android.player

data class DevicePlaybackPositionKey(
    val serverProfileId: String,
    val mediaItemId: String,
    val sourceId: String,
) {
    init {
        require(serverProfileId.isNotBlank()) { "serverProfileId must not be blank" }
        require(mediaItemId.isNotBlank()) { "mediaItemId must not be blank" }
        require(sourceId.isNotBlank()) { "sourceId must not be blank" }
    }
}

data class DevicePlaybackPosition(
    val key: DevicePlaybackPositionKey,
    val positionMs: Long,
    val durationMs: Long? = null,
    val updatedAtMillis: Long,
)

interface DevicePlaybackPositionStore {
    fun load(key: DevicePlaybackPositionKey): DevicePlaybackPosition?
    fun save(position: DevicePlaybackPosition)
    fun clear(key: DevicePlaybackPositionKey)
}

class InMemoryDevicePlaybackPositionStore : DevicePlaybackPositionStore {
    private val positions = linkedMapOf<DevicePlaybackPositionKey, DevicePlaybackPosition>()

    override fun load(key: DevicePlaybackPositionKey): DevicePlaybackPosition? = positions[key]

    override fun save(position: DevicePlaybackPosition) {
        if (position.positionMs <= 0L) {
            positions.remove(position.key)
            return
        }
        positions[position.key] = position
    }

    override fun clear(key: DevicePlaybackPositionKey) {
        positions.remove(key)
    }
}
