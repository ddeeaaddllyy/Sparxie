package com.ddeeaaddlyy.zeit.ui.app

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.ddeeaaddlyy.zeit.model.UiState
import com.ddeeaaddlyy.zeit.model.ZeitTab
import com.ddeeaaddlyy.zeit.ui.component.LiquidBackground
import com.ddeeaaddlyy.zeit.ui.component.ZeitHeader
import com.ddeeaaddlyy.zeit.ui.navigation.ZeitBottomBar
import com.ddeeaaddlyy.zeit.ui.screen.account.AccountScreen
import com.ddeeaaddlyy.zeit.ui.screen.home.HomeScreen
import com.ddeeaaddlyy.zeit.ui.screen.logs.LogsScreen
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
    state: UiState,
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

@Preview(showBackground = true, backgroundColor = 0xFF0A0908)
@Composable
private fun ZeitScreenPreview() {
    ZeitTheme {
        ZeitScreen(
            state = UiState(),
            onTabSelected = {},
            onLoggingChanged = {},
            onTelegramChanged = {},
            onDiscordChanged = {},
            onToggleService = {}
        )
    }
}
