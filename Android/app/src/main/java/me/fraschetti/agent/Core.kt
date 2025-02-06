@file:Suppress("NAME_SHADOWING")

package me.fraschetti.agent

import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import io.ktor.client.HttpClient
import io.ktor.client.engine.cio.CIO
import io.ktor.client.plugins.HttpTimeout
import me.fraschetti.agent.shared.handleResponse
import me.fraschetti.agent.shared.processEvent
import me.fraschetti.agent.shared.view
import me.fraschetti.agent.shared_types.Effect
import me.fraschetti.agent.shared_types.Event
import me.fraschetti.agent.shared_types.HttpResult
import me.fraschetti.agent.shared_types.Request
import me.fraschetti.agent.shared_types.Requests
import me.fraschetti.agent.shared_types.ViewModel
import me.fraschetti.agent.utils.Navigator
import me.fraschetti.agent.utils.requestHttp

open class Core : androidx.lifecycle.ViewModel() {
    var navigator: Navigator = Navigator()

    //    val navigator: NavController.Companion,
    var view: ViewModel? by mutableStateOf(null)
        private set

    private val httpClient = HttpClient(CIO)

    suspend fun update(event: Event) {
        val effects = processEvent(event.bincodeSerialize())

        val requests = Requests.bincodeDeserialize(effects)
        for (request in requests) {
            processEffect(request)
        }
    }

    private suspend fun processEffect(request: Request) {
        when (val effect = request.effect) {
            is Effect.Render -> {
                val previousScreen = this.view?.current_screen
                this.view = ViewModel.bincodeDeserialize(view())
                if (this.view?.current_screen != previousScreen) {
                    Log.d("NAV", "Should navigate!")
                    this.view?.current_screen?.let { this.navigator.navigateTo(it) }
//                    this.navigator.navigateTo(this.view?.screen)
                } else {
                    Log.d("NAV", "Screen haven't change")
                }
            }

            is Effect.Http -> {
//                val response = requestHttp(httpClient, effect.value)
                val response =
                    requestHttp(
                        HttpClient(CIO) {
                            install(HttpTimeout)
                        },
                        effect.value,
                    )

                val effects =
                    handleResponse(
                        request.id.toUInt(),
                        HttpResult.Ok(response).bincodeSerialize(),
                    )

                val requests = Requests.bincodeDeserialize(effects)
                for (request in requests) {
                    processEffect(request)
                }
            }
        }
    }
}
