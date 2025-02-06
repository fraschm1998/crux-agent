package me.fraschetti.agent.utils

import android.util.Log
import androidx.activity.compose.BackHandler
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.navigation.compose.rememberNavController
import me.fraschetti.agent.Core

// import me.fraschetti.speech.shared_types.UiEvent

@Composable
fun InterceptBackAction(core: Core) {
    val coroutineScope = rememberCoroutineScope()
    val navController = rememberNavController()
    BackHandler(
        enabled = true,
    ) {
        Log.d("NAV", "Intercepted back press!!!!!!")
//        coroutineScope.launch {
//            core.update(Event.Ui(UiEvent.NavigateBack()))
//        }
    }
}
