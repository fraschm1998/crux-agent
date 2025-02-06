package me.fraschetti.agent.components.buttons

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MaterialTheme.typography
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

// AuthButtonVariant.kt
enum class AuthButtonVariant {
    PRIMARY,
    SECONDARY,
    OUTLINE,
}

// AuthButton.kt
@Composable
fun AuthButton(
    text: String,
    variant: AuthButtonVariant,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val backgroundColor =
        when (variant) {
            AuthButtonVariant.PRIMARY -> MaterialTheme.colorScheme.primary
            AuthButtonVariant.SECONDARY -> Color.Gray.copy(alpha = 0.1f)
            AuthButtonVariant.OUTLINE -> Color.Transparent
        }

    val textColor =
        when (variant) {
            AuthButtonVariant.PRIMARY -> Color.White
            AuthButtonVariant.SECONDARY, AuthButtonVariant.OUTLINE -> Color.LightGray
        }

    val border =
        if (variant == AuthButtonVariant.OUTLINE) {
            BorderStroke(
                width = 1.dp,
                color = Color.Gray.copy(alpha = 0.2f),
            )
        } else {
            null
        }

    Button(
        onClick = onClick,
        modifier =
            modifier
                .fillMaxWidth()
                .height(50.dp),
        colors =
            ButtonDefaults.buttonColors(
                containerColor = backgroundColor,
                contentColor = textColor,
            ),
        shape = RoundedCornerShape(48.dp),
        border = border,
    ) {
        Text(
            text = text,
            style = typography.bodyLarge,
            textAlign = TextAlign.Center,
        )
    }
}

// Usage example:
// @Preview
// @Composable
// fun AuthButtonPreview() {
//    Column(
//        modifier = Modifier
//            .padding(16.dp)
//            .fillMaxWidth(),
//        verticalArrangement = Arrangement.spacedBy(8.dp)
//    ) {
//        AuthButton(
//            text = "Primary Button",
//            variant = AuthButtonVariant.PRIMARY,
//            onClick = {}
//        )
//
//        AuthButton(
//            text = "Secondary Button",
//            variant = AuthButtonVariant.SECONDARY,
//            onClick = {}
//        )
//
//        AuthButton(
//            text = "Outline Button",
//            variant = AuthButtonVariant.OUTLINE,
//            onClick = {}
//        )
//    }
// }

// ├── components/          // Reusable UI components
// │       │   │   │   │   ├── buttons/
// │       │   │   │   │   │   ├── PrimaryButton.kt
// │       │   │   │   │   │   └── SecondaryButton.kt
// │       │   │   │   │   │
// │       │   │   │   │   ├── cards/
// │       │   │   │   │   │   ├── DebateCard.kt
// │       │   │   │   │   │   └── MetricsCard.kt
// │       │   │   │   │   │
// │       │   │   │   │   ├── inputs/
// │       │   │   │   │   │   ├── AudioInput.kt
// │       │   │   │   │   │   └── TextInput.kt
// │       │   │   │   │   │
// │       │   │   │   │   └── metrics/
// │       │   │   │   │       ├── SpeechMetrics.kt
// │       │   │   │   │       └── DebateMetrics.kt
