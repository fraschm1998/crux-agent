use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveKitEvent {
    JoinRoom, // (String, String), // (room_name, token)
    // Connecting,
    RoomJoined(String),
    RoomLeft,
    // DebateStarted,
    // AudioReceived(Vec<u8>),
    Error(String),
}
