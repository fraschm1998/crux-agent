#[derive(Debug, Clone, PartialEq)]
pub enum LiveKitEffect {
    JoinRoom { url: String, token: String },
    LeaveRoom,
    // other LiveKit-specific effects...
}
