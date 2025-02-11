pub mod livekit_handler;

use std::sync::Arc;

use livekit::{webrtc::audio_source::native::NativeAudioSource, Room};
// Re-export the middleware for easier access
pub use livekit_handler::LiveKitMiddleware;

// Common types and traits for our middleware
use crate::{app::Effect, capabilities::livekit::LiveKitOperation};
use crux_core::Request;

// If we need to define any shared traits or types for our middleware
pub trait EffectWithLiveKit {
    fn is_livekit(&self) -> bool;
    fn into_livekit(self) -> Option<Request<LiveKitOperation>>;
}

// Implement for our Effect type
impl EffectWithLiveKit for Effect {
    fn is_livekit(&self) -> bool {
        matches!(self, Effect::LiveKit(_))
    }

    fn into_livekit(self) -> Option<Request<LiveKitOperation>> {
        match self {
            Effect::LiveKit(request) => Some(request),
            _ => None,
        }
    }
}

// Common types used across middleware
#[derive(Debug)]
pub enum LiveKitCommand {
    JoinRoom {
        url: String,
        token: String,
    },
    LeaveRoom,
    StartMicRecording {
        audio_source: Arc<NativeAudioSource>,
    },
    StopMicRecording,
}

// State tracking for LiveKit
// #[derive(Default)]
// pub struct LiveKitState {
//     room: Option<Room>,
//     is_recording: bool,
// }
