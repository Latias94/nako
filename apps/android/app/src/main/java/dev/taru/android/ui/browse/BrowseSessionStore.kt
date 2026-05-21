package dev.taru.android.ui.browse

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update

internal interface BrowseSessionStore {
    val value: BrowseShellState
    fun set(state: BrowseShellState)
    fun update(transform: (BrowseShellState) -> BrowseShellState)
}

internal class FlowBrowseSessionStore(
    private val state: MutableStateFlow<BrowseShellState>,
) : BrowseSessionStore {
    override val value: BrowseShellState
        get() = state.value

    override fun set(state: BrowseShellState) {
        this.state.value = state
    }

    override fun update(transform: (BrowseShellState) -> BrowseShellState) {
        state.update(transform)
    }
}
