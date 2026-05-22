package dev.nako.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import dev.nako.android.ui.NakoAndroidApp
import dev.nako.android.ui.theme.NakoAndroidTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            NakoAndroidTheme {
                NakoAndroidApp()
            }
        }
    }
}
