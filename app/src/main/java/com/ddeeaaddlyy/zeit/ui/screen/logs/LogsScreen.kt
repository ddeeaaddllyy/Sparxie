package com.ddeeaaddlyy.zeit.ui.screen.logs

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ddeeaaddlyy.zeit.ui.component.LiquidGlassPanel
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite

@Composable
fun LogsScreen(logs: List<String>) {
    LiquidGlassPanel {
        Text(
            text = "Logs",
            color = AsphaltWhite,
            fontSize = 20.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 0.sp
        )
        Spacer(modifier = Modifier.height(12.dp))
        logs.forEach { log ->
            Text(
                text = log,
                color = AsphaltWhite.copy(alpha = 0.72f),
                modifier = Modifier.padding(vertical = 7.dp)
            )
        }
    }
}
