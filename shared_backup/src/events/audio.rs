use livekit::webrtc::{audio_stream::native::NativeAudioStream, prelude::RtcAudioTrack};
use std::sync::Arc;
// use chrono::{DateTime, Utc};
// use crux_http::http::StatusCode;
// use crux_http::http::StatusCode;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;
// use std::pin::Pin;
// use uuid::Uuid;

use crate::capabilities::audio::AudioData;
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
    StartRecordingRequested,
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
    // Core Events (Internal/Response Handling)
    // /// Audio processing result
    // #[serde(skip)]
    // AudioProcessed(Result<ProcessedAudio, AudioError>),
    // /// Speech metrics calculation
    // #[serde(skip)]
    // SpeechMetricsCalculated(Result<SpeechMetrics, ProcessingError>),
    // /// Filler word detection
    // #[serde(skip)]
    // FillerWordsDetected(Vec<FillerWord>),
    // /// Clarity score update
    // #[serde(skip)]
    // ClarityScoreUpdated(f32),
    // /// Speaking pace calculation
    // #[serde(skip)]
    // PaceCalculated(f32),
    // /// Pronunciation analysis
    // #[serde(skip)]
    // PronunciationAnalysisCompleted(Result<PronunciationResult, AnalysisError>),
    // /// Emotion analysis
    // #[serde(skip)]
    // EmotionalToneAnalyzed(Result<EmotionalAnalysis, AnalysisError>),
    // /// Accent analysis
    // #[serde(skip)]
    // AccentProfileGenerated(Result<AccentProfile, AnalysisError>),
    // /// Audio quality issue detected
    // #[serde(skip)]
    // AudioQualityIssueDetected(AudioQualityIssue),
    // /// Calibration completed
    // #[serde(skip)]
    // CalibrationCompleted(Result<CalibrationResult, CalibrationError>),
}

// enum AudioEvent {
//     Initialize,
//     StartRecording,
//     StopRecording,
//     PauseRecording,
//     AudioDataReceived(Vec<f32>),
//     SetError(String),
//     ClearError,
//     ProcessRecording,
// }
