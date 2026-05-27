package com.ddeeaaddlyy.zeit.ui.view

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ddeeaaddlyy.zeit.model.UiState
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.SoftDarkBlack
import com.ddeeaaddlyy.zeit.ui.theme.SoftRose

object ObjectHome {
    @Composable
    public fun HomeScreen(
        state: UiState,
        onLoggingChanged: (Boolean) -> Unit,
        onTelegramChanged: (Boolean) -> Unit,
        onDiscordChanged: (Boolean) -> Unit,
        onToggleService: () -> Unit
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            LiquidGlassPanel {
                ToggleRow(
                    title = "Enable logging",
                    subtitle = "Store application events",
                    checked = state.isButtonLoggingEnabled,
                    onCheckedChange = onLoggingChanged
                )
                ToggleRow(
                    title = "Enable the Telegram bot",
                    subtitle = "Receive commands from Telegram",
                    checked = state.isButtonTelegramBotEnabled,
                    onCheckedChange = onTelegramChanged
                )
                ToggleRow(
                    title = "Enable Discord Bot",
                    subtitle = "Sync Discord channel",
                    checked = state.isButtonDiscordBotEnabled,
                    onCheckedChange = onDiscordChanged
                )
            }
            Spacer(modifier = Modifier.weight(1f, fill = false))
            Button(
                onClick = onToggleService,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(64.dp),
                shape = RoundedCornerShape(18.dp),
                colors = ButtonDefaults.buttonColors(
                    containerColor = if (state.isServiceEnabled) AsphaltWhite else SoftRose,
                    contentColor = SoftDarkBlack
                ),
                contentPadding = PaddingValues(horizontal = 24.dp)
            ) {
                Text(
                    text = if (state.isServiceEnabled) "Turn OFF" else "Turn ON",
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 0.sp
                )
            }
        }
    }
}