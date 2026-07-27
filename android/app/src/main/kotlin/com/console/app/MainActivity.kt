package com.console.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import com.console.app.ui.ConsoleApp
import com.console.app.ui.theme.ConsoleTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val container = (application as ConsoleApplication).container
        setContent {
            ConsoleTheme {
                ConsoleApp(container = container)
            }
        }
    }
}
