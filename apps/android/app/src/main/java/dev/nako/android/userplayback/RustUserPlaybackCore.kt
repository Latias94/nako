package dev.nako.android.userplayback

import dev.nako.android.browse.PageRequest
import dev.nako.android.connection.ServerProfile
import dev.nako.android.connection.NakoHttpRequest
import dev.nako.android.connection.toAndroidRequest
import uniffi.nako_client_uniffi.CoreHttpRequest
import uniffi.nako_client_uniffi.CorePageQuery
import uniffi.nako_client_uniffi.CoreUserPlaybackItemRequestInput
import uniffi.nako_client_uniffi.CoreUserPlaybackItemWriteRequestInput
import uniffi.nako_client_uniffi.CoreUserPlaybackPagedRequestInput

internal interface UserPlaybackCore {
    fun getState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): UserPlaybackRequestDescriptor

    fun continueWatching(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): UserPlaybackRequestDescriptor

    fun updateProgress(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        bodyUtf8: String,
    ): UserPlaybackRequestDescriptor

    fun setWatchedState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        bodyUtf8: String,
    ): UserPlaybackRequestDescriptor
}

internal object RustUserPlaybackCore : UserPlaybackCore {
    override fun getState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
    ): UserPlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildGetUserPlaybackStateRequest(
            itemInput(profile, accessToken, itemId),
        ).toUserPlaybackDescriptor()

    override fun continueWatching(
        profile: ServerProfile,
        accessToken: String,
        page: PageRequest,
    ): UserPlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildListContinueWatchingRequest(
            CoreUserPlaybackPagedRequestInput(
                baseUrl = profile.baseUrl,
                accessToken = accessToken,
                page = page.toCore(),
            ),
        ).toUserPlaybackDescriptor()

    override fun updateProgress(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        bodyUtf8: String,
    ): UserPlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildUpdateUserPlaybackProgressRequest(
            itemWriteInput(profile, accessToken, itemId, bodyUtf8),
        ).toUserPlaybackDescriptor()

    override fun setWatchedState(
        profile: ServerProfile,
        accessToken: String,
        itemId: String,
        bodyUtf8: String,
    ): UserPlaybackRequestDescriptor =
        uniffi.nako_client_uniffi.buildSetUserWatchedStateRequest(
            itemWriteInput(profile, accessToken, itemId, bodyUtf8),
        ).toUserPlaybackDescriptor()
}

internal data class UserPlaybackRequestDescriptor(
    val request: NakoHttpRequest,
)

private fun itemInput(
    profile: ServerProfile,
    accessToken: String,
    itemId: String,
): CoreUserPlaybackItemRequestInput =
    CoreUserPlaybackItemRequestInput(
        baseUrl = profile.baseUrl,
        accessToken = accessToken,
        itemId = itemId,
    )

private fun itemWriteInput(
    profile: ServerProfile,
    accessToken: String,
    itemId: String,
    bodyUtf8: String,
): CoreUserPlaybackItemWriteRequestInput =
    CoreUserPlaybackItemWriteRequestInput(
        baseUrl = profile.baseUrl,
        accessToken = accessToken,
        itemId = itemId,
        bodyUtf8 = bodyUtf8,
    )

private fun PageRequest.toCore(): CorePageQuery =
    CorePageQuery(limit = limit.toUInt(), offset = offset.toULong())

private fun CoreHttpRequest.toUserPlaybackDescriptor(): UserPlaybackRequestDescriptor =
    UserPlaybackRequestDescriptor(request = toAndroidRequest())
