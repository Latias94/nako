package dev.taru.android.ui.theme

import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

private val DarkColorScheme = darkColorScheme(
    primary = TaruAccent,
    onPrimary = TaruBackdrop,
    secondary = TaruTextSecondary,
    onSecondary = TaruBackdrop,
    background = TaruBackdrop,
    onBackground = TaruTextPrimary,
    surface = TaruSurface,
    onSurface = TaruTextPrimary,
    surfaceVariant = TaruSurfaceRaised,
    onSurfaceVariant = TaruTextSecondary,
    outline = TaruSurfaceMuted,
    error = TaruError,
    onError = TaruBackdrop,
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
fun TaruAndroidTheme(
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
        typography = TaruTypography,
        content = content,
    )
}
