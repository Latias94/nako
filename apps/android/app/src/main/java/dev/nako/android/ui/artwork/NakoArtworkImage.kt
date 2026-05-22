package dev.nako.android.ui.artwork

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import coil3.compose.AsyncImage
import coil3.network.NetworkHeaders
import coil3.network.httpHeaders
import coil3.request.ImageRequest
import coil3.request.crossfade
import dev.nako.android.artwork.PublicArtworkRequest

@Composable
internal fun NakoArtworkImage(
    request: PublicArtworkRequest?,
    contentDescription: String?,
    modifier: Modifier = Modifier,
    contentScale: ContentScale = ContentScale.Crop,
    fallback: @Composable () -> Unit,
    overlay: @Composable () -> Unit = {},
) {
    var failed by remember(request?.request?.url, request?.request?.headers) { mutableStateOf(false) }
    Box(modifier = modifier) {
        fallback()
        if (request != null && !failed) {
            val context = androidx.compose.ui.platform.LocalContext.current
            AsyncImage(
                model = ImageRequest.Builder(context)
                    .data(request.request.url)
                    .httpHeaders(networkHeaders(request.request.headers))
                    .crossfade(true)
                    .build(),
                contentDescription = contentDescription,
                modifier = Modifier.fillMaxSize(),
                contentScale = contentScale,
                onError = { failed = true },
            )
        }
        overlay()
    }
}

@Composable
internal fun NakoArtworkGradientOverlay(
    modifier: Modifier = Modifier,
    colors: List<Color>,
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(Brush.verticalGradient(colors = colors)),
    )
}

private fun networkHeaders(headers: Map<String, String>): NetworkHeaders =
    headers.entries
        .fold(NetworkHeaders.Builder()) { builder, entry ->
            builder.set(entry.key, entry.value)
        }
        .build()
