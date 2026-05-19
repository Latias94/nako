package dev.taru.android.smoke

import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Test

class DebugSmokeFixtureSeedActivityTest {
    @Test
    fun `seed request normalizes profile inputs without exposing token as profile data`() {
        val request = debugSmokeFixtureSeedRequest(
            baseUrl = " http://127.0.0.1:3018/ ",
            accessToken = " demo-fixture-token ",
            displayName = " Smoke Server ",
            checkedAtMillis = 42L,
        )

        val snapshot = debugSmokeFixtureProfileSnapshot(request)
        val profile = snapshot.profiles.single()

        assertEquals("server-1", profile.id)
        assertEquals("Smoke Server", profile.displayName)
        assertEquals("http://127.0.0.1:3018", profile.baseUrl)
        assertEquals("server-token:server-1", profile.tokenReference)
        assertEquals("v1", profile.lastObservedApiVersion)
        assertEquals(42L, profile.lastSuccessfulConnectionAtMillis)
        assertEquals("server-1", snapshot.activeProfileId)
        assertEquals("demo-fixture-token", request.accessToken)
    }

    @Test
    fun `seed request normalizes optional local resume input`() {
        val request = debugSmokeFixtureSeedRequest(
            baseUrl = "http://127.0.0.1:3018",
            accessToken = "demo-fixture-token",
            displayName = "Smoke Server",
            checkedAtMillis = 42L,
            resumeMediaItemId = " item-1 ",
            resumeSourceId = " source-1 ",
            resumePositionMs = 1_000L,
            resumeDurationMs = 2_000L,
        )

        val resume = requireNotNull(request.resumePosition)
        assertEquals("item-1", resume.mediaItemId)
        assertEquals("source-1", resume.sourceId)
        assertEquals(1_000L, resume.positionMs)
        assertEquals(2_000L, resume.durationMs)
    }

    @Test
    fun `seed request rejects partial or nonpositive resume input`() {
        assertThrows(IllegalArgumentException::class.java) {
            debugSmokeFixtureSeedRequest(
                baseUrl = "http://127.0.0.1:3018",
                accessToken = "demo-fixture-token",
                displayName = "Smoke Server",
                checkedAtMillis = 42L,
                resumeMediaItemId = "item-1",
                resumeSourceId = "source-1",
                resumePositionMs = 0L,
            )
        }

        assertThrows(IllegalArgumentException::class.java) {
            debugSmokeFixtureSeedRequest(
                baseUrl = "http://127.0.0.1:3018",
                accessToken = "demo-fixture-token",
                displayName = "Smoke Server",
                checkedAtMillis = 42L,
                resumeMediaItemId = "item-1",
                resumePositionMs = 1_000L,
            )
        }
    }

    @Test
    fun `seed request rejects invalid base url and blank token`() {
        assertThrows(IllegalArgumentException::class.java) {
            debugSmokeFixtureSeedRequest(
                baseUrl = "file:///tmp/taru",
                accessToken = "demo-fixture-token",
                displayName = "Smoke Server",
                checkedAtMillis = 42L,
            )
        }

        assertThrows(IllegalArgumentException::class.java) {
            debugSmokeFixtureSeedRequest(
                baseUrl = "http://127.0.0.1:3018",
                accessToken = " ",
                displayName = "Smoke Server",
                checkedAtMillis = 42L,
            )
        }
    }
}
