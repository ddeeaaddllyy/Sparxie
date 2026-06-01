package com.ddeeaaddlyy.zeit.ui.screen.account

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ddeeaaddlyy.zeit.model.UiState
import com.ddeeaaddlyy.zeit.ui.component.LiquidGlassPanel
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.SoftDarkBlack
import com.ddeeaaddlyy.zeit.ui.theme.SoftRose

@Composable
fun AccountScreen(state: UiState) {
    LiquidGlassPanel(horizontalAlignment = Alignment.CenterHorizontally) {
        Box(
            modifier = Modifier
                .size(72.dp)
                .clip(CircleShape)
                .background(
                    Brush.linearGradient(
                        colors = listOf(SoftRose, AsphaltWhite.copy(alpha = 0.74f))
                    )
                ),
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "Z",
                color = SoftDarkBlack,
                fontSize = 32.sp,
                fontWeight = FontWeight.Black,
                letterSpacing = 0.sp
            )
        }
        Spacer(modifier = Modifier.height(14.dp))
        Text(
            text = state.accountName,
            color = AsphaltWhite,
            fontSize = 19.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 0.sp
        )
        Text(
            text = "Local profile",
            color = AsphaltWhite.copy(alpha = 0.58f),
            textAlign = TextAlign.Center
        )
    }
}
