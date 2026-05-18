package dev.taru.android.ui.shell

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.NavigationRailItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.dp
import dev.taru.android.ui.theme.TaruElevation
import dev.taru.android.ui.theme.TaruMotion
import dev.taru.android.ui.theme.TaruTextSecondary

internal data class TaruShellDestination<T>(
    val value: T,
    val label: String,
    val icon: ImageVector,
)

@Composable
internal fun <T> TaruAdaptiveAppShell(
    destinations: List<TaruShellDestination<T>>,
    selectedDestination: T,
    navigationVisible: Boolean,
    onDestinationSelected: (T) -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable (PaddingValues) -> Unit,
) {
    BoxWithConstraints(modifier = modifier.fillMaxSize()) {
        val useRail = navigationVisible && maxWidth >= 720.dp
        val backgroundBrush = Brush.verticalGradient(
            colors = listOf(
                MaterialTheme.colorScheme.background,
                MaterialTheme.colorScheme.background,
                MaterialTheme.colorScheme.surface.copy(alpha = 0.52f),
            ),
        )
        if (useRail) {
            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .background(backgroundBrush),
            ) {
                TaruNavigationRail(
                    destinations = destinations,
                    selectedDestination = selectedDestination,
                    onDestinationSelected = onDestinationSelected,
                )
                Box(modifier = Modifier.fillMaxSize()) {
                    content(PaddingValues())
                }
            }
        } else {
            Scaffold(
                modifier = Modifier
                    .fillMaxSize()
                    .background(backgroundBrush),
                containerColor = MaterialTheme.colorScheme.background,
                bottomBar = {
                    if (navigationVisible) {
                        TaruBottomNavigation(
                            destinations = destinations,
                            selectedDestination = selectedDestination,
                            onDestinationSelected = onDestinationSelected,
                        )
                    }
                },
            ) { innerPadding ->
                content(innerPadding)
            }
        }
    }
}

@Composable
internal fun <T> TaruRouteTransition(
    targetState: T,
    modifier: Modifier = Modifier,
    content: @Composable (T) -> Unit,
) {
    AnimatedContent(
        targetState = targetState,
        modifier = modifier,
        transitionSpec = {
            fadeIn(tween(TaruMotion.routeEnterMillis)) togetherWith
                fadeOut(tween(TaruMotion.routeExitMillis))
        },
        label = "taru-route",
    ) { currentRoute ->
        content(currentRoute)
    }
}

@Composable
private fun <T> TaruBottomNavigation(
    destinations: List<TaruShellDestination<T>>,
    selectedDestination: T,
    onDestinationSelected: (T) -> Unit,
) {
    NavigationBar(
        containerColor = MaterialTheme.colorScheme.surface,
        tonalElevation = TaruElevation.flat,
    ) {
        destinations.forEach { destination ->
            val selected = destination.value == selectedDestination
            NavigationBarItem(
                selected = selected,
                onClick = { onDestinationSelected(destination.value) },
                icon = {
                    Icon(
                        imageVector = destination.icon,
                        contentDescription = destination.label,
                    )
                },
                label = { Text(destination.label) },
                colors = NavigationBarItemDefaults.colors(
                    selectedIconColor = MaterialTheme.colorScheme.onPrimary,
                    selectedTextColor = MaterialTheme.colorScheme.primary,
                    indicatorColor = MaterialTheme.colorScheme.primary,
                    unselectedIconColor = TaruTextSecondary,
                    unselectedTextColor = TaruTextSecondary,
                ),
            )
        }
    }
}

@Composable
private fun <T> TaruNavigationRail(
    destinations: List<TaruShellDestination<T>>,
    selectedDestination: T,
    onDestinationSelected: (T) -> Unit,
) {
    NavigationRail(
        containerColor = MaterialTheme.colorScheme.surface,
    ) {
        destinations.forEach { destination ->
            val selected = destination.value == selectedDestination
            NavigationRailItem(
                selected = selected,
                onClick = { onDestinationSelected(destination.value) },
                icon = {
                    Icon(
                        imageVector = destination.icon,
                        contentDescription = destination.label,
                    )
                },
                label = { Text(destination.label) },
                colors = NavigationRailItemDefaults.colors(
                    selectedIconColor = MaterialTheme.colorScheme.onPrimary,
                    selectedTextColor = MaterialTheme.colorScheme.primary,
                    indicatorColor = MaterialTheme.colorScheme.primary,
                    unselectedIconColor = TaruTextSecondary,
                    unselectedTextColor = TaruTextSecondary,
                ),
            )
        }
    }
}
