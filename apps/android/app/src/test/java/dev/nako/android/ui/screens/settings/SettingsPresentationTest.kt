package dev.nako.android.ui.screens.settings

import dev.nako.android.connection.PublicErrorEnvelope
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.ServerProfileSnapshot
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
        assertEquals("Connection issue", presentation.lastErrorLabel)
        assertTrue(presentation.report.contains("display_name=Home"))
        assertFalse(presentation.report.contains("server-token"))
        assertFalse(presentation.report.contains("secret-token"))
        assertFalse(presentation.lastErrorLabel.contains("_"))
    }

    @Test
    fun settingsPresentationUsesUserFacingLabelsForEmptyDiagnostics() {
        val presentation = settingsDiagnosticsPresentation(
            profile = ServerProfile(
                id = "server-1",
                displayName = "Home",
                baseUrl = "https://home.example.test",
                tokenReference = "server-token:server-1",
            ),
            snapshot = ServerProfileSnapshot(
                profiles = emptyList(),
                activeProfileId = null,
            ),
        )

        assertEquals("Not checked yet", presentation.apiLabel)
        assertEquals("No recent issue", presentation.lastErrorLabel)
        assertEquals("No successful check yet", presentation.connectionLabel)
    }
}
