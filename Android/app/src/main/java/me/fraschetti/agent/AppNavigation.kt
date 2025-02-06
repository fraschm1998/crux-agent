package me.fraschetti.agent

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.onEach
import me.fraschetti.agent.shared_types.View

@Composable
fun NavigationComponent(
    navController: NavHostController,
    viewModel: Core,
//    paddingValues: PaddingValues,
) {
    val coroutineScope = rememberCoroutineScope()
    LaunchedEffect("navigation") {
        viewModel.navigator.sharedFlow
            .onEach {
                navController.navigate(it.toString())
            }.launchIn(this)
    }

    NavHost(
        navController = navController,
        startDestination = View.Home().toString(),
    ) {
        composable(View.Home().toString()) {
            WaveScreen(viewModel)
//            HomeScreen()
//            HomeScreen(viewModel)
        }
//        composable(View.Dashboard().toString()) {
// //            ScreenOne(viewModel, Modifier.padding(paddingValues))
//            HomeScreen(viewModel)
//        }
//        composable(View.Login().toString()) {
//            LoginPage(viewModel)
//        }
//        composable(View.Report().toString()) {
//            TranscriptionAnalysis(viewModel)
//        }
//        composable(View.Recording().toString()) {
//            androidx.compose.foundation.layout.Box(
//                modifier =
//                    Modifier
//                        .fillMaxWidth()
//                        .padding(top = 48.dp),
//                contentAlignment = Alignment.Center,
//            ) {
//                Text(
//                    text =
//                        viewModel.view
//                            ?.speech
//                            ?.topic
//                            .toString(),
//                    modifier =
//                        Modifier.clickable {
//                            coroutineScope.launch {
//                                viewModel.update(
//                                    Event.Debate(
//                                        DebateEvent.ToggleTopic(),
//                                    ),
//                                )
//                            }
//                        },
//                    style = MaterialTheme.typography.titleLarge,
//                    textAlign = TextAlign.Center,
//                    fontWeight = FontWeight.Bold,
//                )
//            }
//            AudioRecordButton(viewModel)
//
//            androidx.compose.foundation.layout.Box(
//                modifier =
//                    Modifier
//                        .fillMaxSize()
//                        .padding(bottom = 80.dp), // Increased bottom padding to account for bottom navigation
//                contentAlignment = Alignment.BottomCenter,
//            ) {
//                androidx.compose.foundation.layout.Row(
//                    modifier = Modifier.fillMaxWidth(),
//                    horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceEvenly,
//                ) {
//                    androidx.compose.material3.Button(
//                        onClick = {
//                            coroutineScope.launch {
//                                viewModel.update(
//                                    Event.Auth(
//                                        AuthEvent.LoginRequested(
//                                            LoginCredentials("", ""),
//                                        ),
//                                    ),
//                                )
// //                                viewModel.update(Event.Navigation(View.Login()))
//                            }
//                        },
//                    ) {
//                        Text("Login")
//                    }
//
//                    androidx.compose.material3.Button(
//                        onClick = {
//                            coroutineScope.launch {
//                                viewModel.update(
//                                    Event.Auth(
//                                        AuthEvent.LoginRequested(
//                                            LoginCredentials("login", "login"),
//                                        ),
//                                    ),
//                                )
//                            }
//                        },
//                    ) {
//                        Text("Login Form")
//                    }
//
//                    androidx.compose.material3.Button(
//                        onClick = {
//                            coroutineScope.launch {
//                                viewModel.update(
//                                    Event.Auth(
//                                        AuthEvent.LoginRequested(
//                                            LoginCredentials("hello", "world"),
//                                        ),
//                                    ),
//                                )
//                            }
//                        },
//                    ) {
//                        Text("Dashboard")
//                    }
//
//                    androidx.compose.material3.Button(
//                        onClick = {
//                            coroutineScope.launch {
//                                viewModel.update(
//                                    Event.Auth(
//                                        AuthEvent.LoginRequested(
//                                            LoginCredentials("test", "test"),
//                                        ),
//                                    ),
//                                )
//                            }
//                        },
//                    ) {
//                        Text("Report")
//                    }
//                }
//            }
//        }
//        composable(Screen.Registration().toString()) {
//            PrepareDebateScreen(viewModel)
//        }
    }
}
