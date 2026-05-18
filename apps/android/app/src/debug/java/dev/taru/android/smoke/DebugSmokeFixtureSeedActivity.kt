package dev.taru.android.smoke

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import dev.taru.android.connection.AndroidSecureTokenVault
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import dev.taru.android.connection.SharedPreferencesServerProfileStore
import dev.taru.android.connection.TaruPublicApiContract
import java.net.URI

class DebugSmokeFixtureSeedActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        runCatching {
            val request = debugSmokeFixtureSeedRequest(
                baseUrl = intent.getStringExtra(EXTRA_BASE_URL),
                accessToken = intent.getStringExtra(EXTRA_ACCESS_TOKEN),
                displayName = intent.getStringExtra(EXTRA_DISPLAY_NAME),
                checkedAtMillis = intent.getLongExtra(
                    EXTRA_CHECKED_AT_MILLIS,
                    System.currentTimeMillis(),
                ),
            )
            seedDebugSmokeFixture(request)
            setResult(RESULT_OK)
        }.onFailure { error ->
            setResult(
                RESULT_CANCELED,
                Intent().putExtra(EXTRA_ERROR, error.message.orEmpty()),
            )
        }

        finish()
    }

    private fun seedDebugSmokeFixture(request: DebugSmokeFixtureSeedRequest) {
        val snapshot = debugSmokeFixtureProfileSnapshot(request)
        SharedPreferencesServerProfileStore(this).save(snapshot)
        AndroidSecureTokenVault(this).saveToken(request.tokenReference, request.accessToken)
    }

    companion object {
        const val EXTRA_BASE_URL = "base_url"
        const val EXTRA_ACCESS_TOKEN = "access_token"
        const val EXTRA_DISPLAY_NAME = "display_name"
        const val EXTRA_CHECKED_AT_MILLIS = "checked_at_millis"
        const val EXTRA_ERROR = "error"
    }
}

internal data class DebugSmokeFixtureSeedRequest(
    val baseUrl: String,
    val accessToken: String,
    val displayName: String,
    val checkedAtMillis: Long,
) {
    val profileId: String = "server-1"
    val tokenReference: String = "server-token:$profileId"
}

internal fun debugSmokeFixtureSeedRequest(
    baseUrl: String?,
    accessToken: String?,
    displayName: String?,
    checkedAtMillis: Long,
): DebugSmokeFixtureSeedRequest {
    val normalizedBaseUrl = normalizeSmokeFixtureBaseUrl(baseUrl)
    val token = accessToken?.trim().orEmpty()
    require(token.isNotBlank()) { "Smoke fixture access token is required." }

    return DebugSmokeFixtureSeedRequest(
        baseUrl = normalizedBaseUrl,
        accessToken = token,
        displayName = displayName?.trim().takeUnless { it.isNullOrBlank() } ?: "Smoke Server",
        checkedAtMillis = checkedAtMillis,
    )
}

internal fun debugSmokeFixtureProfileSnapshot(
    request: DebugSmokeFixtureSeedRequest,
): ServerProfileSnapshot =
    ServerProfileSnapshot(
        profiles = listOf(
            ServerProfile(
                id = request.profileId,
                displayName = request.displayName,
                baseUrl = request.baseUrl,
                tokenReference = request.tokenReference,
                lastObservedApiVersion = TaruPublicApiContract.expectedApiVersion,
                lastSuccessfulConnectionAtMillis = request.checkedAtMillis,
                lastPublicError = null,
            ),
        ),
        activeProfileId = request.profileId,
    )

private fun normalizeSmokeFixtureBaseUrl(input: String?): String {
    val trimmed = input?.trim()?.trimEnd('/').orEmpty()
    require(trimmed.isNotBlank()) { "Smoke fixture base URL is required." }

    val uri = runCatching { URI(trimmed) }.getOrNull()
        ?: throw IllegalArgumentException("Smoke fixture base URL must be valid.")
    val scheme = uri.scheme?.lowercase()
    require(scheme == "http" || scheme == "https") {
        "Smoke fixture base URL must use HTTP or HTTPS."
    }
    require(!uri.host.isNullOrBlank()) { "Smoke fixture base URL must include a host." }

    return uri.toString().trimEnd('/')
}
