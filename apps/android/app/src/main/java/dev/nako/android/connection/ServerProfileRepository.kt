package dev.nako.android.connection

class ServerProfileRepository(
    initialSnapshot: ServerProfileSnapshot = ServerProfileSnapshot(),
) {
    private val profiles = linkedMapOf<String, ServerProfile>()
    private var activeProfileId: String? = initialSnapshot.activeProfileId

    init {
        initialSnapshot.profiles.forEach { profile ->
            profiles[profile.id] = profile
        }
        if (activeProfileId !in profiles.keys) {
            activeProfileId = profiles.keys.firstOrNull()
        }
    }

    fun snapshot(): ServerProfileSnapshot =
        ServerProfileSnapshot(
            profiles = profiles.values.toList(),
            activeProfileId = activeProfileId,
        )

    fun listProfiles(): List<ServerProfile> = profiles.values.toList()

    fun activeProfile(): ServerProfile? = activeProfileId?.let(profiles::get)

    fun activeBaseUrl(): String? = activeProfile()?.baseUrl

    fun switchActive(profileId: String) {
        require(profileId in profiles) { "unknown server profile: $profileId" }
        activeProfileId = profileId
    }

    fun upsertConnectedProfile(
        displayName: String,
        tokenReference: String?,
        result: ConnectionCheckResult.Success,
    ): ServerProfile {
        val existing = profiles.values.firstOrNull { it.baseUrl == result.normalizedBaseUrl }
        val profileId = existing?.id ?: nextProfileId()
        val resolvedTokenReference = tokenReference ?: existing?.tokenReference ?: tokenReferenceFor(profileId)
        val profile = ServerProfile(
            id = profileId,
            displayName = displayName.ifBlank { result.normalizedBaseUrl },
            baseUrl = result.normalizedBaseUrl,
            tokenReference = resolvedTokenReference,
            lastObservedApiVersion = result.apiVersion,
            lastSuccessfulConnectionAtMillis = result.checkedAtMillis,
            lastPublicError = null,
        )
        profiles[profile.id] = profile
        activeProfileId = profile.id
        return profile
    }

    fun recordFailure(
        profileId: String,
        failure: ConnectionCheckResult.Failure,
    ) {
        val current = profiles[profileId] ?: return
        profiles[profileId] = current.copy(
            lastPublicError = failure.diagnostics.publicError,
            lastObservedApiVersion = failure.diagnostics.observedApiVersion ?: current.lastObservedApiVersion,
        )
    }

    fun tokenReferenceFor(profileId: String): String = "server-token:$profileId"

    private fun nextProfileId(): String {
        var next = profiles.size + 1
        while ("server-$next" in profiles) {
            next += 1
        }
        return "server-$next"
    }
}
