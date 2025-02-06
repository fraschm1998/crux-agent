package me.fraschetti.agent.ui.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color

@Composable
fun CircleWaveView(
    modifier: Modifier = Modifier,
    circleColor: Color = Color.Gray,
) {
    var circleScales by remember { mutableStateOf(listOf(1f, 1f, 1f, 1f)) }

    val fixedWidth = 150f
    val initialHeight = 150f
    val maxHeight = 400f
    val spacing = 170f

    Canvas(modifier = modifier.fillMaxSize()) {
        val widthCenter = size.width / 2
        val heightCenter = size.height / 2

        for (i in 0 until 4) {
            val ellipseHeight = initialHeight * circleScales[i]

            drawRoundRect(
                color = circleColor,
                topLeft =
                    Offset(
                        x = widthCenter + (i - 1.5f) * spacing - fixedWidth / 2,
                        y = heightCenter - ellipseHeight / 2,
                    ),
                size =
                    Size(
                        width = fixedWidth,
                        height = ellipseHeight,
                    ),
                cornerRadius =
                    CornerRadius(
                        x = fixedWidth / 2,
                        y = fixedWidth / 2,
                    ),
            )
        }
    }
}

// Extension function to update circle scales
fun CircleWaveState.updateCircles(scaleList: List<Float>) {
    if (scaleList.size == 4) {
        circleScales = scaleList.map { 1f + (it * 2f) }
    }
}

// Extension function to reset circles
fun CircleWaveState.resetCircles() {
    circleScales = listOf(1f, 1f, 1f, 1f)
}

// State holder class for the CircleWaveView
class CircleWaveState {
    var circleScales by mutableStateOf(listOf(1f, 1f, 1f, 1f))
}

// Composable with state hoisting for more control
@Composable
fun CircleWaveViewWithState(
    modifier: Modifier = Modifier,
    circleColor: Color = Color.Gray,
    state: CircleWaveState = remember { CircleWaveState() },
) {
    Canvas(modifier = modifier.fillMaxSize()) {
        val widthCenter = size.width / 2
        val heightCenter = size.height / 2
        val fixedWidth = 150f
        val initialHeight = 150f
        val spacing = 170f

        for (i in 0 until 4) {
            val ellipseHeight = initialHeight * state.circleScales[i]

            drawRoundRect(
                color = circleColor,
                topLeft =
                    Offset(
                        x = widthCenter + (i - 1.5f) * spacing - fixedWidth / 2,
                        y = heightCenter - ellipseHeight / 2,
                    ),
                size =
                    Size(
                        width = fixedWidth,
                        height = ellipseHeight,
                    ),
                cornerRadius =
                    CornerRadius(
                        x = fixedWidth / 2,
                        y = fixedWidth / 2,
                    ),
            )
        }
    }
}
