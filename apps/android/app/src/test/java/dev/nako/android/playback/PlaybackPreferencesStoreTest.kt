package dev.nako.android.playback

import android.content.SharedPreferences
import org.junit.Assert.assertEquals
import org.junit.Test

class PlaybackPreferencesStoreTest {
    @Test
    fun `default playback capabilities preserve product defaults`() {
        val store = InMemoryPlaybackPreferencesStore()

        assertEquals(PlaybackCapabilities(), store.loadCapabilities("server-1"))
    }

    @Test
    fun `stored playback capabilities can make mkv sources remux for smoke fixtures`() {
        val store = InMemoryPlaybackPreferencesStore()
        val capabilities = PlaybackCapabilities(
            directPlay = true,
            containers = listOf(" mp4 ", "MP4"),
            videoCodecs = listOf("h264"),
            audioCodecs = listOf("aac"),
        )

        store.saveCapabilities("server-1", capabilities)

        assertEquals(
            PlaybackCapabilities(
                directPlay = true,
                containers = listOf("mp4"),
                videoCodecs = listOf("h264"),
                audioCodecs = listOf("aac"),
            ),
            store.loadCapabilities("server-1"),
        )
    }

    @Test
    fun `persistent playback capabilities survive store instances`() {
        val preferences = FakeSharedPreferences()
        SharedPreferencesPlaybackPreferencesStore(preferences).saveCapabilities(
            "server-1",
            PlaybackCapabilities(
                directPlay = true,
                containers = listOf("mp4"),
                videoCodecs = listOf("h264"),
                audioCodecs = listOf("aac"),
            ),
        )

        assertEquals(
            PlaybackCapabilities(
                directPlay = true,
                containers = listOf("mp4"),
                videoCodecs = listOf("h264"),
                audioCodecs = listOf("aac"),
            ),
            SharedPreferencesPlaybackPreferencesStore(preferences).loadCapabilities("server-1"),
        )
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
