package com.ddeeaaddlyy.zeit

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.viewmodel.compose.viewModel
import com.ddeeaaddlyy.zeit.ui.theme.ZeitTheme
import com.ddeeaaddlyy.zeit.ui.app.ZeitApp
import com.ddeeaaddlyy.zeit.viewmodel.AppViewModel

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            ZeitTheme() {
                val viewModel: AppViewModel = viewModel()
                ZeitApp(viewModel)
            }
        }
    }
}


