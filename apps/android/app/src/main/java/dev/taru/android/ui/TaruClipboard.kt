package dev.taru.android.ui

import android.content.ClipData
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

internal interface TaruClipboard {
    fun copyPlainText(
        label: String,
        text: String,
    )
}

@Composable
internal fun rememberTaruClipboard(): TaruClipboard {
    val clipboard = LocalClipboard.current
    val scope = rememberCoroutineScope()
    return remember(clipboard, scope) {
        ComposeTaruClipboard(
            clipboard = clipboard,
            scope = scope,
        )
    }
}

private class ComposeTaruClipboard(
    private val clipboard: androidx.compose.ui.platform.Clipboard,
    private val scope: CoroutineScope,
) : TaruClipboard {
    override fun copyPlainText(
        label: String,
        text: String,
    ) {
        scope.launch {
            clipboard.setClipEntry(
                ClipEntry(
                    ClipData.newPlainText(label, text),
                ),
            )
        }
    }
}
