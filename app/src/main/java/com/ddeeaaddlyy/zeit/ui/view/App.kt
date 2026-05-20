package com.ddeeaaddlyy.zeit.ui.view

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ddeeaaddlyy.zeit.model.ZeitTab
import com.ddeeaaddlyy.zeit.model.ZeitUiState
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.GlassStroke
import com.ddeeaaddlyy.zeit.ui.theme.SoftRose
import com.ddeeaaddlyy.zeit.ui.theme.SoftDarkBlack
import com.ddeeaaddlyy.zeit.ui.theme.ZeitTheme
import com.ddeeaaddlyy.zeit.viewmodel.AppViewModel

@Composable
fun ZeitApp(viewModel: AppViewModel) {
    val state by viewModel.uiState.collectAsState()

    ZeitScreen(
        state = state,
        onTabSelected = viewModel::selectTab,
        onLoggingChanged = viewModel::setLoggingEnabled,
        onTelegramChanged = viewModel::setTelegramBotEnabled,
        onDiscordChanged = viewModel::setDiscordBotEnabled,
        onToggleService = viewModel::toggleService
    )
}

@Composable
private fun ZeitScreen(
    state: ZeitUiState,
    onTabSelected: (ZeitTab) -> Unit,
    onLoggingChanged: (Boolean) -> Unit,
    onTelegramChanged: (Boolean) -> Unit,
    onDiscordChanged: (Boolean) -> Unit,
    onToggleService: () -> Unit
) {
    Scaffold(
        containerColor = SoftDarkBlack,
        bottomBar = {
            ZeitBottomBar(
                selectedTab = state.selectedTab,
                onTabSelected = onTabSelected
            )
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(SoftDarkBlack)
                .padding(innerPadding)
        ) {
            LiquidBackground()
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .statusBarsPadding()
                    .padding(horizontal = 20.dp, vertical = 18.dp),
                verticalArrangement = Arrangement.spacedBy(18.dp)
            ) {
                ZeitHeader()
                when (state.selectedTab) {
                    ZeitTab.Home -> HomeScreen(
                        state = state,
                        onLoggingChanged = onLoggingChanged,
                        onTelegramChanged = onTelegramChanged,
                        onDiscordChanged = onDiscordChanged,
                        onToggleService = onToggleService
                    )

                    ZeitTab.Logs -> LogsScreen(logs = state.logs)
                    ZeitTab.Account -> AccountScreen(state = state)
                }
            }
        }
    }
}

@Composable
private fun LiquidBackground() {
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

@Composable
private fun ZeitHeader() {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(
            text = "ZEIT",
            color = SoftRose,
            fontSize = 34.sp,
            fontWeight = FontWeight.Black,
            letterSpacing = 0.sp
        )
        Text(
            text = "Bot control panel",
            color = AsphaltWhite.copy(alpha = 0.72f),
            style = MaterialTheme.typography.bodyMedium
        )
    }
}

@Composable
private fun HomeScreen(
    state: ZeitUiState,
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

@Composable
private fun ToggleRow(
    title: String,
    subtitle: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Text(
                text = title,
                color = AsphaltWhite,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = 0.sp
            )
            Text(
                text = subtitle,
                color = AsphaltWhite.copy(alpha = 0.56f),
                style = MaterialTheme.typography.bodySmall
            )
        }
        Spacer(modifier = Modifier.width(16.dp))
        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            colors = SwitchDefaults.colors(
                checkedThumbColor = SoftDarkBlack,
                checkedTrackColor = SoftRose,
                uncheckedThumbColor = AsphaltWhite,
                uncheckedTrackColor = AsphaltWhite.copy(alpha = 0.16f),
                uncheckedBorderColor = GlassStroke
            )
        )
    }
}

@Composable
private fun LogsScreen(logs: List<String>) {
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

@Composable
private fun AccountScreen(state: ZeitUiState) {
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

@Composable
private fun LiquidGlassPanel(
    modifier: Modifier = Modifier,
    horizontalAlignment: Alignment.Horizontal = Alignment.Start,
    content: @Composable ColumnScope.() -> Unit
) {
    Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(24.dp),
        colors = CardDefaults.cardColors(
            containerColor = AsphaltWhite.copy(alpha = 0.08f)
        ),
        border = BorderStroke(1.dp, GlassStroke)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            AsphaltWhite.copy(alpha = 0.12f),
                            AsphaltWhite.copy(alpha = 0.04f)
                        )
                    )
                )
                .padding(20.dp),
            horizontalAlignment = horizontalAlignment,
            content = content
        )
    }
}

@Composable
private fun ZeitBottomBar(
    selectedTab: ZeitTab,
    onTabSelected: (ZeitTab) -> Unit
) {
    NavigationBar(
        modifier = Modifier.navigationBarsPadding(),
        containerColor = SoftDarkBlack.copy(alpha = 0.96f),
        tonalElevation = 0.dp
    ) {
        ZeitTab.entries.forEach { tab ->
            NavigationBarItem(
                selected = selectedTab == tab,
                onClick = { onTabSelected(tab) },
                icon = {
                    Text(
                        text = when (tab) {
                            ZeitTab.Home -> "M"
                            ZeitTab.Logs -> "L"
                            ZeitTab.Account -> "A"
                        },
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Black,
                        color = if (selectedTab == tab) SoftDarkBlack else AsphaltWhite.copy(alpha = 0.72f),
                        letterSpacing = 0.sp
                    )
                },
                label = {
                    Text(
                        text = tab.title,
                        maxLines = 1,
                        letterSpacing = 0.sp
                    )
                },
                colors = NavigationBarItemDefaults.colors(
                    selectedIconColor = SoftDarkBlack,
                    selectedTextColor = SoftRose,
                    indicatorColor = SoftRose,
                    unselectedIconColor = AsphaltWhite.copy(alpha = 0.66f),
                    unselectedTextColor = AsphaltWhite.copy(alpha = 0.66f)
                )
            )
        }
    }
}

@Preview(showBackground = true, backgroundColor = 0xFF0A0908)
@Composable
private fun ZeitScreenPreview() {
    ZeitTheme {
        ZeitScreen(
            state = ZeitUiState(),
            onTabSelected = {},
            onLoggingChanged = {},
            onTelegramChanged = {},
            onDiscordChanged = {},
            onToggleService = {}
        )
    }
}
