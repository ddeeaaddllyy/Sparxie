package com.ddeeaaddlyy.zeit.ui.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.SoftRose

@Composable
fun LiquidBackground() {
    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .padding(top = 52.dp, start = 22.dp)
                .size(170.dp)
                .blur(42.dp)
                .clip(CircleShape)
                .background(SoftRose.copy(alpha = 0.18f))
        )
        Box(
            modifier = Modifier
                .align(Alignment.BottomEnd)
                .padding(bottom = 130.dp, end = 16.dp)
                .size(210.dp)
                .blur(52.dp)
                .clip(CircleShape)
                .background(AsphaltWhite.copy(alpha = 0.10f))
        )
    }
}
