package dev.taru.android.connection

import dev.taru.sdk.TaruRequestDescriptor

internal fun TaruRequestDescriptor.urlOn(profile: ServerProfile): String =
    urlOn(profile.baseUrl)

internal fun TaruRequestDescriptor.urlOn(baseUrl: String): String =
    "${baseUrl.trimEnd('/')}$pathAndQuery"
