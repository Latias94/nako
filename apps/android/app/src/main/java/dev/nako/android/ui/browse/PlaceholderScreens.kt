package dev.nako.android.ui.browse

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import dev.nako.android.ui.NakoStrings
import dev.nako.android.ui.components.NakoScreenColumn

@Composable
internal fun PlaceholderTopLevel(
    title: String,
    subtitle: String,
    body: String,
    icon: ImageVector,
) {
    BrowseScaffoldContent {
        NakoScreenColumn {
            PageTitle(
                title = title,
                subtitle = subtitle,
                icon = icon,
            )
            EmptyCard(
                title = "$title shell",
                body = body,
            )
        }
    }
}

@Composable
internal fun PlaceholderRoute(
    title: String,
    subtitle: String,
    body: String,
    onBack: () -> Unit,
) {
    NakoScreenColumn {
        IconButton(onClick = onBack) {
            Icon(
                imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                contentDescription = stringResource(NakoStrings.back),
            )
        }
        PageTitle(
            title = title,
            subtitle = subtitle,
            icon = Icons.Rounded.Search,
        )
        EmptyCard(title = "Server-backed page", body = body)
    }
}
