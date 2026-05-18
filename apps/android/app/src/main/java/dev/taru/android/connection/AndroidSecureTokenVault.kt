@file:Suppress("DEPRECATION")

package dev.taru.android.connection

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

class AndroidSecureTokenVault(
    context: Context,
) : TokenVault {
    private val appContext = context.applicationContext
    private val masterKey = MasterKey.Builder(appContext)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()
    private val preferences = EncryptedSharedPreferences.create(
        appContext,
        "taru_access_tokens",
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    override fun saveToken(reference: String, token: String) {
        preferences.edit().putString(reference, token).apply()
    }

    override fun readToken(reference: String): String? = preferences.getString(reference, null)

    override fun deleteToken(reference: String) {
        preferences.edit().remove(reference).apply()
    }
}
