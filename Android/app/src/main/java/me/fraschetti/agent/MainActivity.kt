package me.fraschetti.agent

// import androidx.compose.ui.Modifier
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.rememberNavController
import kotlinx.coroutines.launch
import me.fraschetti.agent.shared_types.Event
import me.fraschetti.agent.shared_types.LiveKitEvent
import me.fraschetti.agent.ui.components.CircleWaveState
import me.fraschetti.agent.ui.components.CircleWaveViewWithState
import me.fraschetti.agent.ui.theme.PhoneTheme

// import coil.compose.AsyncImage

// class MainActivity : ComponentActivity() {
//    init {
//        System.loadLibrary("shared")
//    }
//
//    private external fun initializeRustContext(context: android.content.Context): Int
//
//    override fun onCreate(savedInstanceState: Bundle?) {
//        super.onCreate(savedInstanceState)
//        enableEdgeToEdge()
// //        initializeRustContext(applicationContext)
//
//        // Add explicit check that JNI initialization succeeded
//        val initResult = initializeRustContext(applicationContext)
//        if (initResult != 0) {
//            Log.e("MainActivity", "Failed to initialize Rust context")
//            finish()
//            return
//        }
//
//        enableEdgeToEdge()
//
//        setContent {
//            PhoneTheme {
//                val navController = rememberNavController()
//                val coreViewModel: Core = viewModel()
//
//                NavigationComponent(navController, coreViewModel)
//            }
//        }
//    }

class MainActivity : ComponentActivity() {
    init {
        System.loadLibrary("shared")
    }

    // Add WebRTC initialization function
//    @JvmStatic
    private external fun initializeWebRTC(context: android.content.Context)

    // Declare the external function to initialize the Rust context
    private external fun initializeRustContext(context: android.content.Context)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        enableEdgeToEdge()
        initializeRustContext(applicationContext)
        initializeWebRTC(applicationContext)
        setContent {
            PhoneTheme {
                val navController = rememberNavController()
                val coreViewModel: Core = viewModel()

                NavigationComponent(navController, coreViewModel)
            }
        }
    }

    // Load the Rust shared library
//    companion object {
//        init {
//            System.loadLibrary("shared")
//        }
//    }
}

// @Preview
// @Composable
// fun HomeScreen(
// //    core: Core,
// //    onNavigateToPrepareDebate: () -> Unit
// ) {
//    WaveScreen()
// //    InterceptBackAction(core)
//    // //////
// }

// @Preview(
//    showBackground = true,
//    showSystemUi = true,
// )
@Composable
fun WaveScreen(core: Core) {
    val coroutineScope = rememberCoroutineScope()
    val waveState = remember { CircleWaveState() }

    Box(
        modifier =
            Modifier
                .fillMaxSize()
                .background(Color.Black),
    ) {
        Column(
            modifier = Modifier.fillMaxSize(),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Spacer(modifier = Modifier.height(200.dp))

            // Custom wave view
            CircleWaveViewWithState(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .height(400.dp),
                state = waveState,
                circleColor = Color.Gray,
            )

//                // Close button (initially hidden)
//                Image(
//                    painter = painterResource(id = R.drawable.ic_close),
//                    contentDescription = "Close",
//                    modifier = Modifier
//                        .size(70.dp)
//                        .then(if (isCloseVisible) Modifier.visible() else Modifier.gone())
//                )

            Spacer(modifier = Modifier.height(10.dp))

            // Clear and reset button
            Button(
                onClick = {
                    coroutineScope.launch {
                        core.update(
                            Event.LiveKit(
                                LiveKitEvent.JoinRoom(),
                            ),
                        )
                    }
                },
//                onClick = {
//
//                    // Generate 4 random heights for the wave animation
// //                    val randomHeights = List(4) { 0.5f + kotlin.random.Random.nextFloat() * 1f }
// //                    waveState.updateCircles(randomHeights)
//                },
                modifier =
                    Modifier
                        .width(200.dp)
                        .height(40.dp),
//                colors =
//                    ButtonDefaults.buttonColors(
//                        backgroundColor = Color(0xFFF08585),
//                    ),
            ) {
                Text(
                    text = "connect",
                    fontSize = 10.sp,
                )
            }
        }
    }
}
//
// private fun startCircleWaveAnimation() {
//    val waveState = CircleWaveState()
// //    val waveView = findViewById<CircleWaveView>(R.id.circle_wave_view)
//
//    // Generate 4 random heights for the wave animation
//    val randomHeights = List(4) { 0.5f + kotlin.random.Random.nextFloat() * 1f }
//
//    // Update the CircleWaveView with the new heights
// //    waveView.updateCircles(randomHeights)
//    waveState.updateCircles(randomHeights)
// }
