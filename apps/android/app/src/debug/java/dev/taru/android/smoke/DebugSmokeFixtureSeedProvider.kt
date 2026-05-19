package dev.taru.android.smoke

import android.content.ContentProvider
import android.content.ContentValues
import android.database.Cursor
import android.net.Uri
import android.os.Bundle

class DebugSmokeFixtureSeedProvider : ContentProvider() {
    override fun onCreate(): Boolean = true

    override fun call(
        method: String,
        arg: String?,
        extras: Bundle?,
    ): Bundle {
        if (method != METHOD_SEED) {
            return resultBundle(
                status = STATUS_ERROR,
                error = "Unsupported smoke fixture method '$method'.",
            )
        }

        val appContext = requireNotNull(context?.applicationContext) {
            "Smoke fixture provider context is unavailable."
        }
        return runCatching {
            val request = debugSmokeFixtureSeedRequest(
                baseUrl = arg ?: extras?.getString(DebugSmokeFixtureSeedActivity.EXTRA_BASE_URL),
                accessToken = extras?.getString(DebugSmokeFixtureSeedActivity.EXTRA_ACCESS_TOKEN),
                displayName = extras?.getString(DebugSmokeFixtureSeedActivity.EXTRA_DISPLAY_NAME),
                checkedAtMillis = extras?.getLong(
                    DebugSmokeFixtureSeedActivity.EXTRA_CHECKED_AT_MILLIS,
                    System.currentTimeMillis(),
                ) ?: System.currentTimeMillis(),
                resumeMediaItemId = extras?.getString(DebugSmokeFixtureSeedActivity.EXTRA_RESUME_MEDIA_ITEM_ID),
                resumeSourceId = extras?.getString(DebugSmokeFixtureSeedActivity.EXTRA_RESUME_SOURCE_ID),
                resumePositionMs = extras?.longOrNull(DebugSmokeFixtureSeedActivity.EXTRA_RESUME_POSITION_MS),
                resumeDurationMs = extras?.longOrNull(DebugSmokeFixtureSeedActivity.EXTRA_RESUME_DURATION_MS),
                forceRemux = extras?.booleanOrNull(DebugSmokeFixtureSeedActivity.EXTRA_FORCE_REMUX),
            )
            seedDebugSmokeFixture(appContext, request)
            resultBundle(status = STATUS_OK)
        }.getOrElse { error ->
            resultBundle(
                status = STATUS_ERROR,
                error = error.message.orEmpty(),
            )
        }
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? = null

    override fun getType(uri: Uri): String? = null

    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    override fun delete(
        uri: Uri,
        selection: String?,
        selectionArgs: Array<out String>?,
    ): Int = 0

    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        selectionArgs: Array<out String>?,
    ): Int = 0

    private fun resultBundle(
        status: String,
        error: String = "",
    ): Bundle =
        Bundle().apply {
            putString(KEY_STATUS, status)
            putString(KEY_ERROR, error)
        }

    companion object {
        const val AUTHORITY = "dev.taru.android.smoke.fixture"
        const val METHOD_SEED = "seed"
        const val KEY_STATUS = "status"
        const val KEY_ERROR = "error"
        const val STATUS_OK = "ok"
        const val STATUS_ERROR = "error"
    }
}

private fun Bundle.longOrNull(key: String): Long? =
    if (containsKey(key)) getLong(key) else null

private fun Bundle.booleanOrNull(key: String): Boolean? =
    if (containsKey(key)) getBoolean(key) else null
