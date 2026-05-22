package dev.nako.android.ui.theme

import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

private val DarkColorScheme = darkColorScheme(
    primary = NakoAccent,
    onPrimary = NakoBackdrop,
    secondary = NakoTextSecondary,
    onSecondary = NakoBackdrop,
    background = NakoBackdrop,
    onBackground = NakoTextPrimary,
    surface = NakoSurface,
    onSurface = NakoTextPrimary,
    surfaceVariant = NakoSurfaceRaised,
    onSurfaceVariant = NakoTextSecondary,
    outline = NakoSurfaceMuted,
    error = NakoError,
    onError = NakoBackdrop,
)

private val LightColorScheme = lightColorScheme(
    primary = ColorRolesLight.primary,
    onPrimary = ColorRolesLight.onPrimary,
    background = ColorRolesLight.background,
    onBackground = ColorRolesLight.onBackground,
    surface = ColorRolesLight.surface,
    onSurface = ColorRolesLight.onSurface,
    surfaceVariant = ColorRolesLight.surfaceVariant,
    onSurfaceVariant = ColorRolesLight.onSurfaceVariant,
    outline = ColorRolesLight.outline,
    error = ColorRolesLight.error,
    onError = ColorRolesLight.onError,
)

@Composable
fun NakoAndroidTheme(
    darkTheme: Boolean = true,
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit,
) {
    val context = LocalContext.current
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S ->
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = NakoTypography,
        content = content,
    )
}
