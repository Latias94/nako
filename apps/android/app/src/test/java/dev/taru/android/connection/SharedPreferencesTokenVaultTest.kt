package dev.taru.android.connection

import android.content.SharedPreferences
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SharedPreferencesTokenVaultTest {
    @Test
    fun `saved token can be read without storing raw token or reference`() {
        val preferences = FakeSharedPreferences()
        val vault = SharedPreferencesTokenVault(
            preferences = preferences,
            cipher = ReversibleTokenVaultCipher(),
        )

        vault.saveToken("server-token:server-1", "super-secret-token")

        assertEquals("super-secret-token", vault.readToken("server-token:server-1"))
        assertFalse(preferences.snapshot().keys.any { it.contains("server-token:server-1") })
        assertFalse(preferences.snapshot().values.any { it.contains("super-secret-token") })
    }

    @Test
    fun `delete removes stored token`() {
        val preferences = FakeSharedPreferences()
        val vault = SharedPreferencesTokenVault(
            preferences = preferences,
            cipher = ReversibleTokenVaultCipher(),
        )

        vault.saveToken("server-token:server-1", "super-secret-token")
        vault.deleteToken("server-token:server-1")

        assertNull(vault.readToken("server-token:server-1"))
        assertTrue(preferences.snapshot().isEmpty())
    }

    @Test
    fun `invalid token record is purged and treated as absent`() {
        val preferences = FakeSharedPreferences()
        val vault = SharedPreferencesTokenVault(
            preferences = preferences,
            cipher = ReversibleTokenVaultCipher(),
        )

        vault.saveToken("server-token:server-1", "super-secret-token")
        preferences.edit()
            .putString(preferences.snapshot().keys.single(), "not-a-token-record")
            .commit()

        assertNull(vault.readToken("server-token:server-1"))
        assertTrue(preferences.snapshot().isEmpty())
    }

    @Test
    fun `legacy token is migrated into encrypted storage and removed from source`() {
        val preferences = FakeSharedPreferences()
        val migrationSource = RecordingMigrationSource(
            "server-token:server-1" to "legacy-secret-token",
        )
        val vault = SharedPreferencesTokenVault(
            preferences = preferences,
            cipher = ReversibleTokenVaultCipher(),
            migrationSource = migrationSource,
        )

        assertEquals("legacy-secret-token", vault.readToken("server-token:server-1"))
        assertEquals("legacy-secret-token", vault.readToken("server-token:server-1"))

        assertEquals(listOf("server-token:server-1"), migrationSource.readReferences)
        assertEquals(listOf("server-token:server-1"), migrationSource.deletedReferences)
        assertFalse(preferences.snapshot().keys.any { it.contains("server-token:server-1") })
        assertFalse(preferences.snapshot().values.any { it.contains("legacy-secret-token") })
        assertTrue(preferences.snapshot().isNotEmpty())
    }

    @Test
    fun `save and delete clear migrated legacy token source`() {
        val migrationSource = RecordingMigrationSource(
            "server-token:server-1" to "legacy-secret-token",
        )
        val vault = SharedPreferencesTokenVault(
            preferences = FakeSharedPreferences(),
            cipher = ReversibleTokenVaultCipher(),
            migrationSource = migrationSource,
        )

        vault.saveToken("server-token:server-1", "new-secret-token")
        vault.deleteToken("server-token:server-1")

        assertEquals(
            listOf("server-token:server-1", "server-token:server-1"),
            migrationSource.deletedReferences,
        )
        assertNull(migrationSource.readToken("server-token:server-1"))
    }

    private class ReversibleTokenVaultCipher : TokenVaultCipher {
        override fun encrypt(plainText: ByteArray): EncryptedTokenRecord =
            EncryptedTokenRecord(
                version = 1,
                algorithm = "TEST",
                iv = byteArrayOf(1, 2, 3),
                cipherText = plainText.reversedArray(),
            )

        override fun decrypt(record: EncryptedTokenRecord): ByteArray {
            require(record.version == 1)
            require(record.algorithm == "TEST")
            return record.cipherText.reversedArray()
        }
    }

    private class RecordingMigrationSource(
        vararg tokens: Pair<String, String>,
    ) : TokenVaultMigrationSource {
        private val tokens = linkedMapOf(*tokens)
        val readReferences: MutableList<String> = mutableListOf()
        val deletedReferences: MutableList<String> = mutableListOf()

        override fun readToken(reference: String): String? {
            readReferences += reference
            return tokens[reference]
        }

        override fun deleteToken(reference: String) {
            deletedReferences += reference
            tokens.remove(reference)
        }
    }

    private class FakeSharedPreferences : SharedPreferences {
        private val values = linkedMapOf<String, String>()

        fun snapshot(): Map<String, String> = values.toMap()

        override fun getString(key: String?, defValue: String?): String? =
            values[key] ?: defValue

        override fun edit(): SharedPreferences.Editor = Editor()

        override fun getAll(): MutableMap<String, *> = values.toMutableMap()
        override fun getStringSet(key: String?, defValues: MutableSet<String>?): MutableSet<String>? = defValues
        override fun getInt(key: String?, defValue: Int): Int = defValue
        override fun getLong(key: String?, defValue: Long): Long = defValue
        override fun getFloat(key: String?, defValue: Float): Float = defValue
        override fun getBoolean(key: String?, defValue: Boolean): Boolean = defValue
        override fun contains(key: String?): Boolean = values.containsKey(key)
        override fun registerOnSharedPreferenceChangeListener(
            listener: SharedPreferences.OnSharedPreferenceChangeListener?,
        ) = Unit
        override fun unregisterOnSharedPreferenceChangeListener(
            listener: SharedPreferences.OnSharedPreferenceChangeListener?,
        ) = Unit

        private inner class Editor : SharedPreferences.Editor {
            private val pendingPuts = linkedMapOf<String, String>()
            private val pendingRemovals = linkedSetOf<String>()
            private var clearRequested = false

            override fun putString(key: String?, value: String?): SharedPreferences.Editor {
                requireNotNull(key)
                if (value == null) {
                    pendingRemovals += key
                } else {
                    pendingPuts[key] = value
                }
                return this
            }

            override fun remove(key: String?): SharedPreferences.Editor {
                requireNotNull(key)
                pendingRemovals += key
                return this
            }

            override fun clear(): SharedPreferences.Editor {
                clearRequested = true
                return this
            }

            override fun commit(): Boolean {
                if (clearRequested) {
                    values.clear()
                }
                pendingRemovals.forEach(values::remove)
                values.putAll(pendingPuts)
                return true
            }

            override fun apply() {
                commit()
            }

            override fun putStringSet(key: String?, values: MutableSet<String>?): SharedPreferences.Editor = this
            override fun putInt(key: String?, value: Int): SharedPreferences.Editor = this
            override fun putLong(key: String?, value: Long): SharedPreferences.Editor = this
            override fun putFloat(key: String?, value: Float): SharedPreferences.Editor = this
            override fun putBoolean(key: String?, value: Boolean): SharedPreferences.Editor = this
        }
    }
}
