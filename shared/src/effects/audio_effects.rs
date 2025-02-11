#[derive(Debug, Clone, PartialEq)]
pub enum AudioEffect {
    StartRecording,
    StopRecording,
    // PlaybackAudio { track: RtcTrack },
    // other audio-specific effects...
}
