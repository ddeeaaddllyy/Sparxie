package com.ddeeaaddlyy.zeit.ui.component

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.dp
import com.ddeeaaddlyy.zeit.ui.theme.AsphaltWhite
import com.ddeeaaddlyy.zeit.ui.theme.GlassStroke

@Composable
fun LiquidGlassPanel(
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
