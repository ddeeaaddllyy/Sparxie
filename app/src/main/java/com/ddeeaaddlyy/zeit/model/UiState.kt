package com.ddeeaaddlyy.zeit.model

data class UiState(
    val selectedTab: ZeitTab = ZeitTab.Home,
    val isButtonLoggingEnabled: Boolean = true,
    val isButtonTelegramBotEnabled: Boolean = false,
    val isButtonDiscordBotEnabled: Boolean = false,
    val isServiceEnabled: Boolean = false,
    val logs: List<String> = listOf("System is start", "Waiting for start polling bot"),
    val accountName: String = "ZEIT User"
)
