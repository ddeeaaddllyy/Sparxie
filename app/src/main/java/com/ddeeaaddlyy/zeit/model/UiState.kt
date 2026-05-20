package com.ddeeaaddlyy.zeit.model

enum class ZeitTab(val title: String) {
    Home("Main Menu"),
    Logs("Logging"),
    Account("Account")
}

data class ZeitUiState(
    val selectedTab: ZeitTab = ZeitTab.Home,
    val isButtonLoggingEnabled: Boolean = true,
    val isButtonTelegramBotEnabled: Boolean = false,
    val isButtonDiscordBotEnabled: Boolean = false,
    val isServiceEnabled: Boolean = false,
    val logs: List<String> = listOf("System is start", "Waiting for start polling bot"),
    val accountName: String = "ZEIT User"
)
