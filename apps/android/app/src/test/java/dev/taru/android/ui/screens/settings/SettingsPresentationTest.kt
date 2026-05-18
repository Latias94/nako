package dev.taru.android.ui.screens.settings

import dev.taru.android.connection.PublicErrorEnvelope
import dev.taru.android.connection.ServerProfile
import dev.taru.android.connection.ServerProfileSnapshot
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SettingsPresentationTest {
    @Test
    fun diagnosticsReportUsesServerFactsWithoutTokenReferenceOrTokenValue() {
        val presentation = settingsDiagnosticsPresentation(
            profile = ServerProfile(
                id = "server-1",
                displayName = "Home",
                baseUrl = "http://127.0.0.1:3018",
                tokenReference = "server-token:server-1",
                lastObservedApiVersion = "2026-05-01",
                lastSuccessfulConnectionAtMillis = 42,
                lastPublicError = PublicErrorEnvelope(
                    code = "transport_error",
                    message = "timeout",
                ),
            ),
            snapshot = ServerProfileSnapshot(
                profiles = listOf(
                    ServerProfile(
                        id = "server-1",
                        displayName = "Home",
                        baseUrl = "http://127.0.0.1:3018",
                        tokenReference = "server-token:server-1",
                    ),
                ),
                activeProfileId = "server-1",
            ),
        )

        assertEquals("2026-05-01", presentation.apiLabel)
        assertEquals("transport_error", presentation.lastErrorLabel)
        assertTrue(presentation.report.contains("display_name=Home"))
        assertFalse(presentation.report.contains("server-token"))
        assertFalse(presentation.report.contains("secret-token"))
    }
}
