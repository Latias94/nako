package dev.taru.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import dev.taru.android.ui.TaruAndroidApp
import dev.taru.android.ui.theme.TaruAndroidTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            TaruAndroidTheme {
                TaruAndroidApp()
            }
        }
    }
}
