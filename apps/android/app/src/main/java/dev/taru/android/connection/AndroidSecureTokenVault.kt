package dev.taru.android.connection

import android.content.Context
import android.content.SharedPreferences
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.nio.charset.StandardCharsets
import java.security.KeyStore
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.Base64
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

class AndroidSecureTokenVault(
    context: Context,
) : TokenVault {
    private val appContext = context.applicationContext

    private val preferences = appContext.getSharedPreferences(
        TOKEN_VAULT_PREFERENCES_NAME,
        Context.MODE_PRIVATE,
    )

    private val cipher = AndroidKeystoreTokenCipher(
        keyAlias = TOKEN_VAULT_KEY_ALIAS,
    )

    private val delegate = SharedPreferencesTokenVault(
        preferences = preferences,
        cipher = cipher,
    )

    override fun saveToken(reference: String, token: String) {
        delegate.saveToken(reference, token)
    }

    override fun readToken(reference: String): String? = delegate.readToken(reference)

    override fun deleteToken(reference: String) {
        delegate.deleteToken(reference)
    }

    private companion object {
        const val TOKEN_VAULT_PREFERENCES_NAME = "taru_token_vault_v2"
        const val TOKEN_VAULT_KEY_ALIAS = "taru_token_vault_aes_gcm_v1"
    }
}

internal class SharedPreferencesTokenVault(
    private val preferences: SharedPreferences,
    private val cipher: TokenVaultCipher,
    private val migrationSource: TokenVaultMigrationSource = NoTokenVaultMigrationSource,
) : TokenVault {
    override fun saveToken(reference: String, token: String) {
        val record = cipher.encrypt(token.encodeToByteArray())
        preferences.edit()
            .putString(storageKey(reference), record.toWireString())
            .commit()
        migrationSource.deleteToken(reference)
    }

    override fun readToken(reference: String): String? {
        val encoded = preferences.getString(storageKey(reference), null)
            ?: return readAndMigrateToken(reference)
        val record = runCatching { EncryptedTokenRecord.fromWireString(encoded) }
            .getOrElse {
                deleteToken(reference)
                return null
            }
        return runCatching {
            String(cipher.decrypt(record), StandardCharsets.UTF_8)
        }.getOrElse {
            deleteToken(reference)
            null
        }
    }

    override fun deleteToken(reference: String) {
        preferences.edit()
            .remove(storageKey(reference))
            .commit()
        migrationSource.deleteToken(reference)
    }

    private fun storageKey(reference: String): String =
        "token.${reference.sha256Base64Url()}"

    private fun readAndMigrateToken(reference: String): String? {
        val legacyToken = migrationSource.readToken(reference) ?: return null
        val record = cipher.encrypt(legacyToken.encodeToByteArray())
        preferences.edit()
            .putString(storageKey(reference), record.toWireString())
            .commit()
        migrationSource.deleteToken(reference)
        return legacyToken
    }
}

internal interface TokenVaultCipher {
    fun encrypt(plainText: ByteArray): EncryptedTokenRecord
    fun decrypt(record: EncryptedTokenRecord): ByteArray
}

internal interface TokenVaultMigrationSource {
    fun readToken(reference: String): String?
    fun deleteToken(reference: String)
}

private object NoTokenVaultMigrationSource : TokenVaultMigrationSource {
    override fun readToken(reference: String): String? = null
    override fun deleteToken(reference: String) = Unit
}

internal data class EncryptedTokenRecord(
    val version: Int,
    val algorithm: String,
    val iv: ByteArray,
    val cipherText: ByteArray,
) {
    fun toWireString(): String =
        listOf(
            version.toString(),
            algorithm,
            iv.base64UrlEncode(),
            cipherText.base64UrlEncode(),
        ).joinToString(separator = ".")

    companion object {
        fun fromWireString(value: String): EncryptedTokenRecord {
            val parts = value.split('.')
            require(parts.size == 4) { "Invalid encrypted token record." }
            return EncryptedTokenRecord(
                version = parts[0].toInt(),
                algorithm = parts[1],
                iv = parts[2].base64UrlDecode(),
                cipherText = parts[3].base64UrlDecode(),
            )
        }
    }
}

internal class AndroidKeystoreTokenCipher(
    private val keyAlias: String,
) : TokenVaultCipher {
    override fun encrypt(plainText: ByteArray): EncryptedTokenRecord {
        val iv = ByteArray(GCM_IV_BYTES)
        secureRandom.nextBytes(iv)

        val cipher = Cipher.getInstance(AES_GCM_TRANSFORMATION)
        cipher.init(
            Cipher.ENCRYPT_MODE,
            secretKey(),
            GCMParameterSpec(GCM_TAG_BITS, iv),
        )

        return EncryptedTokenRecord(
            version = TOKEN_RECORD_VERSION,
            algorithm = TOKEN_RECORD_ALGORITHM,
            iv = iv,
            cipherText = cipher.doFinal(plainText),
        )
    }

    override fun decrypt(record: EncryptedTokenRecord): ByteArray {
        require(record.version == TOKEN_RECORD_VERSION) { "Unsupported token record version." }
        require(record.algorithm == TOKEN_RECORD_ALGORITHM) { "Unsupported token cipher algorithm." }

        val cipher = Cipher.getInstance(AES_GCM_TRANSFORMATION)
        cipher.init(
            Cipher.DECRYPT_MODE,
            secretKey(),
            GCMParameterSpec(GCM_TAG_BITS, record.iv),
        )
        return cipher.doFinal(record.cipherText)
    }

    private fun secretKey(): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE_PROVIDER).apply {
            load(null)
        }
        val existing = keyStore.getKey(keyAlias, null)
        if (existing is SecretKey) {
            return existing
        }

        val keyGenerator = KeyGenerator.getInstance(
            KeyProperties.KEY_ALGORITHM_AES,
            ANDROID_KEYSTORE_PROVIDER,
        )
        val spec = KeyGenParameterSpec.Builder(
            keyAlias,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setRandomizedEncryptionRequired(true)
            .setKeySize(AES_KEY_BITS)
            .build()
        keyGenerator.init(spec)
        return keyGenerator.generateKey()
    }

    private companion object {
        const val ANDROID_KEYSTORE_PROVIDER = "AndroidKeyStore"
        const val AES_GCM_TRANSFORMATION = "AES/GCM/NoPadding"
        const val TOKEN_RECORD_VERSION = 1
        const val TOKEN_RECORD_ALGORITHM = "AES256_GCM"
        const val AES_KEY_BITS = 256
        const val GCM_TAG_BITS = 128
        const val GCM_IV_BYTES = 12

        val secureRandom = SecureRandom()
    }
}

private fun String.sha256Base64Url(): String {
    val digest = MessageDigest.getInstance("SHA-256")
        .digest(toByteArray(StandardCharsets.UTF_8))
    return digest.base64UrlEncode()
}

private fun ByteArray.base64UrlEncode(): String =
    Base64.getUrlEncoder()
        .withoutPadding()
        .encodeToString(this)

private fun String.base64UrlDecode(): ByteArray =
    Base64.getUrlDecoder().decode(this)
