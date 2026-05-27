package com.ddeeaaddlyy.zeit.viewmodel

import androidx.lifecycle.ViewModel
import com.ddeeaaddlyy.zeit.model.States
import com.ddeeaaddlyy.zeit.model.States.START_TELEGRAM
import com.ddeeaaddlyy.zeit.model.States.ABORT
import com.ddeeaaddlyy.zeit.model.States.LAUNCHING_ALL
import com.ddeeaaddlyy.zeit.model.States.START_DISCORD
import com.ddeeaaddlyy.zeit.model.States.PING_TEST
import com.ddeeaaddlyy.zeit.model.ZeitTab
import com.ddeeaaddlyy.zeit.model.UiState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update

class AppViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(UiState())
    val uiState: StateFlow<UiState> = _uiState.asStateFlow()
    var botState: States = ABORT

    fun selectTab(tab: ZeitTab) {
        _uiState.update { state ->
            state.copy(selectedTab = tab)
        }
    }

    fun setLoggingEnabled(enabled: Boolean) {
        _uiState.updateWithLog(
            log = if (enabled) "[ ! ] Logging is ON [ ! ]" else "[ ! ] Logging is OFF [ ! ]"
        ) { state ->
            state.copy(isButtonLoggingEnabled = enabled)
        }
    }

    fun setTelegramBotEnabled(enabled: Boolean) {
        botState = if (enabled) START_TELEGRAM else ABORT
        _uiState.updateWithLog(
            log = if (enabled) "Telegram bot is ON" else "Telegram is OFF"
        ) { state ->
            state.copy(isButtonTelegramBotEnabled = enabled)
        }
    }

    fun setDiscordBotEnabled(enabled: Boolean) {
        botState = if (enabled) START_DISCORD else ABORT
        _uiState.updateWithLog(
            log = if (enabled) "Discord bot is ON" else "Discord bit is OFF"
        ) { state ->
            state.copy(isButtonDiscordBotEnabled = enabled)
        }
    }

    fun toggleService() {
        _uiState.updateWithLog(
            log = if (_uiState.value.isServiceEnabled) "ZEIT has been suspended $botState" else "ZEIT is running $botState"
        ) { state ->
            state.copy(isServiceEnabled = !state.isServiceEnabled)
        }
    }

    private fun MutableStateFlow<UiState>.updateWithLog(
        log: String,
        transform: (UiState) -> UiState
    ) {
        update { current ->
            val updated = transform(current)
            updated.copy(logs = (listOf(log) + updated.logs).take(n = 20))
        }
    }
}
