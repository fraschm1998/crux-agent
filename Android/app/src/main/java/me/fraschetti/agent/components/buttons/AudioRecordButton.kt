package me.fraschetti.agent.components.buttons

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import me.fraschetti.agent.Core

// import me.fraschetti.speech.shared_types.SpeechEvent

@Composable
fun AudioRecordButton(core: Core) {
    val coroutineScope = rememberCoroutineScope()

    Column {
        Box(
            modifier = Modifier.fillMaxSize(),
            contentAlignment = Alignment.Center,
        ) {
//            Button(
//                onClick = {
//                    // Launch the coroutine to handle the core.update call
//                    coroutineScope.launch(Dispatchers.IO) {
//                        core.update(
//                            Event.Speech(
//                                SpeechEvent.MicrophoneToggleRequested(),
//                            ),
//                        )
//                    }
//                },
//            ) {
//                Text(
//                    text =
//                        core.view
//                            ?.speech
//                            ?.recording_text
//                            .toString(),
//                )
//            }
        }
    }
}

private fun checkPermission(context: Context): Boolean =
    ContextCompat.checkSelfPermission(
        context,
        Manifest.permission.RECORD_AUDIO,
    ) == PackageManager.PERMISSION_GRANTED

private fun requestPermission(context: Context) {
    ActivityCompat.requestPermissions(
        context as androidx.activity.ComponentActivity,
        arrayOf(Manifest.permission.RECORD_AUDIO),
        200,
    )
}
