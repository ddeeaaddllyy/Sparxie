package com.ddeeaaddlyy.zeit.ui.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.SoftRose

@Composable
fun ZeitHeader() {
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
