package dev.nako.android.ui

import android.content.ClipData
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch

internal interface NakoClipboard {
    fun copyPlainText(
        label: String,
        text: String,
    )
}

@Composable
internal fun rememberNakoClipboard(): NakoClipboard {
    val clipboard = LocalClipboard.current
    val scope = rememberCoroutineScope()
    return remember(clipboard, scope) {
        ComposeNakoClipboard(
            clipboard = clipboard,
            scope = scope,
        )
    }
}

private class ComposeNakoClipboard(
    private val clipboard: androidx.compose.ui.platform.Clipboard,
    private val scope: CoroutineScope,
) : NakoClipboard {
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
