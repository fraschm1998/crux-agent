package me.fraschetti.agent.utils

import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import me.fraschetti.agent.shared_types.View

class Navigator {
    private val _sharedFlow =
        MutableSharedFlow<View>(extraBufferCapacity = 1)
    val sharedFlow = _sharedFlow.asSharedFlow()

    fun navigateTo(navTarget: View) {
        _sharedFlow.tryEmit(navTarget)
    }
}
