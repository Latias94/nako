package dev.taru.android.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable

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
    error = TaruWarning,
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
)

@Composable
fun TaruAndroidTheme(
    darkTheme: Boolean = true,
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) DarkColorScheme else LightColorScheme,
        typography = TaruTypography,
        content = content,
    )
}
