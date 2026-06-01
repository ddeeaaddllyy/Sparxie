package com.ddeeaaddlyy.zeit.ui.navigation

import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ddeeaaddlyy.zeit.model.ZeitTab
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.SoftDarkBlack
import com.ddeeaaddlyy.zeit.ui.theme.SoftRose

@Composable
fun ZeitBottomBar(
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
                        text = tab.iconLabel,
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

private val ZeitTab.iconLabel: String
    get() = when (this) {
        ZeitTab.Home -> "M"
        ZeitTab.Logs -> "L"
        ZeitTab.Account -> "A"
    }
