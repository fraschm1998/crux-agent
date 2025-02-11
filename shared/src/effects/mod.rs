use audio_effects::AudioEffect;
use livekit_effects::LiveKitEffect;

pub mod audio_effects;
pub mod livekit_effects;

pub enum RustEffect {
    None,
    Audio(AudioEffect),
    LiveKit(LiveKitEffect),
}
