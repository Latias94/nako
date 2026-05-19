package dev.taru.android.ui.screens.player

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

internal fun launchPlayerExitEffect(
    exitEffectScope: CoroutineScope,
    block: suspend () -> Unit,
): Job =
    exitEffectScope.launch {
        block()
    }
