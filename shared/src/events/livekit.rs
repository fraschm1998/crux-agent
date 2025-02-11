use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LiveKitEvent {
    JoinRoom, // (String, String), // (room_name, token)
    // Connecting,
    RoomJoined,
    // RoomJoined(String), // do we need to specify the room?
    RoomLeft,
    // DebateStarted,
    // AudioReceived(Vec<u8>),
    Error(String),
}
