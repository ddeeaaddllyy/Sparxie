package com.ddeeaaddlyy.zeit.ui.theme

import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext

private val DarkColorScheme = darkColorScheme(
    primary = SoftRose,
    secondary = AsphaltWhite,
    tertiary = Pink80,
    background = SoftDarkBlack,
    surface = SoftDarkBlack,
    onPrimary = SoftDarkBlack,
    onSecondary = SoftDarkBlack,
    onBackground = AsphaltWhite,
    onSurface = AsphaltWhite
)

private val LightColorScheme = lightColorScheme(
    primary = SoftRose,
    secondary = SoftDarkBlack,
    tertiary = Pink40,
    background = SoftDarkBlack,
    surface = SoftDarkBlack,
    onPrimary = SoftDarkBlack,
    onSecondary = AsphaltWhite,
    onBackground = AsphaltWhite,
    onSurface = AsphaltWhite
)

@Composable
fun ZeitTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val context = LocalContext.current
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }

        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
