use livekit::webrtc::{audio_stream::native::NativeAudioStream, prelude::RtcAudioTrack};
use std::sync::Arc;
// use chrono::{DateTime, Utc};
// use crux_http::http::StatusCode;
// use crux_http::http::StatusCode;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::pin::Pin;
// use uuid::Uuid;

use crate::capabilities::audio::{AudioData, AudioOperation, RecordingState};
// use crate::speech::{AnalysisType, RecordingConfig};
// use crate::RecordingMetrics;

// pin_project_lite::pin_project! {
//     pub struct ResponseAsync {
//         #[pin]
//         res: crux_http::http::Response,
//     }
// }
//
// impl Clone for ResponseAsync {
//     fn clone(&self) -> Self {
//         Self {
//             res: self.res.clone(),
//         }
//     }
// }

/// Speech analysis events split between shell and core events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioEvent {
    // Shell Events (User/Platform Initiated)
    /// Start recording and analysis
    // StartRecordingRequested,
    StartRecordingRequested(Arc<NativeAudioSource>),
    // StartRecordingRequested(RecordingConfig),
    /// Stop recording
    StopRecordingRequested,
    /// Send recording
    SendRecordingRequested(AudioData),
    /// Microphone control
    MicrophoneToggleRequested,
    /// Audio Playback
    #[serde(skip)]
    PlaybackAudio {
        track: RtcAudioTrack,
        // sample_rate: i32,
        // num_channels: i32,
    },
    RecordingComplete(AudioData),
    RecordingStateChanged(RecordingState),
    RecordingError(String),
}

impl From<AudioOperation> for AudioEvent {
    fn from(op: AudioOperation) -> Self {
        match op {
            AudioOperation::PlaybackAudio { sample_rate, channels, samples } => {
                AudioEvent::RecordingComplete(AudioData {
                    samples,
                    sample_rate,
                    channels,
                })
            }
            AudioOperation::ToggleRecording => {
                AudioEvent::RecordingStateChanged(RecordingState::Recording)
            }
        }
    }
}
