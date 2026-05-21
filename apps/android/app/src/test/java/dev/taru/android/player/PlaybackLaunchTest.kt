package dev.taru.android.player

import android.content.SharedPreferences
import dev.taru.android.playback.ClientPlaybackMode
import dev.taru.android.playback.PlaybackRequestDescriptor
import dev.taru.android.playback.PlaybackRequestTarget
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PlaybackLaunchTest {
    @Test
    fun `launch request debug output uses safe request preview only`() {
        val launch = playbackLaunchRequest(
            title = "Night Harbor",
            target = PlaybackRequestTarget(
                request = PlaybackRequestDescriptor(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream/hls/playlist.m3u8",
                ),
            ),
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.Transcode,
            sessionId = "session-1",
            resumePositionMs = 12_000,
        )

        assertEquals(null, launch.request.headers["Authorization"])
        assertEquals("Bearer <redacted>", launch.safeRequest.headers["Authorization"])
        assertEquals(
            "Bearer secret-token",
            launch.authenticatedRequest("secret-token").headers["Authorization"],
        )
        assertTrue(launch.toString().contains("Bearer <redacted>"))
        assertTrue(launch.toString().contains("session-1"))
        assertFalse(launch.toString().contains("secret-token"))
    }

    @Test(expected = IllegalArgumentException::class)
    fun `playback request descriptor rejects authorization headers`() {
        PlaybackRequestDescriptor(
            method = "GET",
            url = "http://127.0.0.1:3018/sources/source-1/stream",
            headers = mapOf("Authorization" to "Bearer secret-token"),
        )
    }

    @Test(expected = IllegalArgumentException::class)
    fun `playback request descriptor rejects bearer tokens in non auth headers`() {
        PlaybackRequestDescriptor(
            method = "GET",
            url = "http://127.0.0.1:3018/sources/source-1/stream",
            headers = mapOf("X-Debug" to "Bearer secret-token"),
        )
    }

    @Test
    fun `launch request position key is scoped to active server media item and source`() {
        val launch = playbackLaunchRequest(
            title = "Night Harbor",
            target = PlaybackRequestTarget(
                request = PlaybackRequestDescriptor(
                    method = "GET",
                    url = "http://127.0.0.1:3018/sources/source-1/stream",
                ),
            ),
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
            playbackMode = ClientPlaybackMode.DirectPlay,
        )

        assertEquals("server-1", launch.positionKey.serverProfileId)
        assertEquals("item-1", launch.positionKey.mediaItemId)
        assertEquals("source-1", launch.positionKey.sourceId)
    }

    @Test
    fun `device local playback position does not mix across server profiles`() {
        val store = InMemoryDevicePlaybackPositionStore()
        val homeKey = DevicePlaybackPositionKey(
            serverProfileId = "home-server",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )
        val travelKey = homeKey.copy(serverProfileId = "travel-server")

        store.save(
            DevicePlaybackPosition(
                key = homeKey,
                positionMs = 90_000,
                durationMs = 600_000,
                updatedAtMillis = 1_779_000_000_000,
            ),
        )

        assertEquals(90_000L, store.load(homeKey)?.positionMs)
        assertNull(store.load(travelKey))
    }

    @Test
    fun `device local playback position clears non positive positions`() {
        val store = InMemoryDevicePlaybackPositionStore()
        val key = DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 42_000,
                updatedAtMillis = 1,
            ),
        )
        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 0,
                updatedAtMillis = 2,
            ),
        )

        assertNull(store.load(key))
    }

    @Test
    fun `persistent device local playback position survives store instances`() {
        val preferences = FakeSharedPreferences()
        val key = DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

        SharedPreferencesDevicePlaybackPositionStore(preferences).save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 123_000,
                durationMs = 900_000,
                updatedAtMillis = 1_779_155_000_000,
            ),
        )
        val reloaded = SharedPreferencesDevicePlaybackPositionStore(preferences).load(key)

        assertEquals(123_000L, reloaded?.positionMs)
        assertEquals(900_000L, reloaded?.durationMs)
        assertEquals(1_779_155_000_000L, reloaded?.updatedAtMillis)
    }

    @Test
    fun `persistent device local playback position remains scoped by server item and source`() {
        val store = SharedPreferencesDevicePlaybackPositionStore(FakeSharedPreferences())
        val key = DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 77_000,
                updatedAtMillis = 1,
            ),
        )

        assertEquals(77_000L, store.load(key)?.positionMs)
        assertNull(store.load(key.copy(serverProfileId = "server-2")))
        assertNull(store.load(key.copy(mediaItemId = "item-2")))
        assertNull(store.load(key.copy(sourceId = "source-2")))
    }

    @Test
    fun `persistent device local playback position clears stored values`() {
        val preferences = FakeSharedPreferences()
        val store = SharedPreferencesDevicePlaybackPositionStore(preferences)
        val key = DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 42_000,
                updatedAtMillis = 1,
            ),
        )
        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 0,
                updatedAtMillis = 2,
            ),
        )

        assertNull(SharedPreferencesDevicePlaybackPositionStore(preferences).load(key))
    }

    @Test
    fun `persistent device local playback position drops corrupt local data`() {
        val preferences = FakeSharedPreferences()
        val store = SharedPreferencesDevicePlaybackPositionStore(preferences)
        val key = DevicePlaybackPositionKey(
            serverProfileId = "server-1",
            mediaItemId = "item-1",
            sourceId = "source-1",
        )

        store.save(
            DevicePlaybackPosition(
                key = key,
                positionMs = 42_000,
                updatedAtMillis = 1,
            ),
        )
        preferences.edit().putString(preferences.all.keys.first(), "{not-json").commit()

        assertNull(store.load(key))
        assertTrue(preferences.all.isEmpty())
    }
}

private class FakeSharedPreferences : SharedPreferences {
    private val values = linkedMapOf<String, Any?>()

    override fun getAll(): MutableMap<String, *> = values.toMutableMap()

    override fun getString(
        key: String?,
        defValue: String?,
    ): String? = values[key] as? String ?: defValue

    override fun getStringSet(
        key: String?,
        defValues: MutableSet<String>?,
    ): MutableSet<String>? = defValues

    override fun getInt(
        key: String?,
        defValue: Int,
    ): Int = defValue

    override fun getLong(
        key: String?,
        defValue: Long,
    ): Long = defValue

    override fun getFloat(
        key: String?,
        defValue: Float,
    ): Float = defValue

    override fun getBoolean(
        key: String?,
        defValue: Boolean,
    ): Boolean = defValue

    override fun contains(key: String?): Boolean = values.containsKey(key)

    override fun edit(): SharedPreferences.Editor = Editor()

    override fun registerOnSharedPreferenceChangeListener(
        listener: SharedPreferences.OnSharedPreferenceChangeListener?,
    ) = Unit

    override fun unregisterOnSharedPreferenceChangeListener(
        listener: SharedPreferences.OnSharedPreferenceChangeListener?,
    ) = Unit

    private inner class Editor : SharedPreferences.Editor {
        private val updates = linkedMapOf<String, Any?>()
        private val removals = linkedSetOf<String>()
        private var clearAll = false

        override fun putString(
            key: String?,
            value: String?,
        ): SharedPreferences.Editor = apply {
            if (key != null) {
                updates[key] = value
            }
        }

        override fun putStringSet(
            key: String?,
            values: MutableSet<String>?,
        ): SharedPreferences.Editor = this

        override fun putInt(
            key: String?,
            value: Int,
        ): SharedPreferences.Editor = this

        override fun putLong(
            key: String?,
            value: Long,
        ): SharedPreferences.Editor = this

        override fun putFloat(
            key: String?,
            value: Float,
        ): SharedPreferences.Editor = this

        override fun putBoolean(
            key: String?,
            value: Boolean,
        ): SharedPreferences.Editor = this

        override fun remove(key: String?): SharedPreferences.Editor = apply {
            if (key != null) {
                removals += key
            }
        }

        override fun clear(): SharedPreferences.Editor = apply {
            clearAll = true
        }

        override fun commit(): Boolean {
            apply()
            return true
        }

        override fun apply() {
            if (clearAll) {
                values.clear()
            }
            removals.forEach(values::remove)
            updates.forEach { (key, value) ->
                if (value == null) {
                    values.remove(key)
                } else {
                    values[key] = value
                }
            }
        }
    }
}
