use crate::app::{App, Event, Effect};
use crate::events::audio::AudioEvent;
use crate::capabilities::audio::{Audio, AudioData, AudioOperation, RecordingState};
use crate::{AppEffect, Capabilities, Model, ViewModel, RUNTIME};
use crux_core::{App as CruxApp, Capability, Command, Core};
use livekit::webrtc::audio_source::native::NativeAudioSource;
use tokio::task::JoinHandle;
use std::sync::Arc;

pub struct AudioMiddleware {
    core: Core<App>,
    audio: Audio<Event>,
    mic_task: Option<JoinHandle<()>>,
}

impl Default for AudioMiddleware {
    fn default() -> Self {
        // Create a placeholder instance that should be replaced with new()
        Self {
            core: Core::default(),
            audio: Audio::default(),
            mic_task: None,
        }
    }
}

impl AudioMiddleware {
    pub fn new(core: Core<App>, audio: Audio<Event>) -> Self {
        Self {
            core,
            audio,
            mic_task: None,
        }
    }

    pub fn start_mic_capture(&mut self, audio_source: Arc<NativeAudioSource>) {
        let audio = self.audio.clone();
        
        // Start mic capture in a separate task
        self.mic_task = Some(RUNTIME.spawn(async move {
            // Just await the result since capture_mic_audio returns ()
            audio.capture_mic_audio(audio_source).await;
        }));
    }

    pub fn handle_effect(&mut self, effect: Effect) -> Vec<Effect> {
        match effect {
            Effect::Audio(audio_effect) => {
                match audio_effect.operation {
                    AudioOperation::ToggleRecording => {
                        // Start/stop mic capture
                        if self.mic_task.is_none() {
                            // Assume audio_source is accessible here; may need to pass it in
                            self.start_mic_capture(audio_source);
                        } else {
                            self.stop_mic_capture();
                        }
                    }
                    AudioOperation::PlaybackAudio { sample_rate, channels, samples } => {
                        // Handle audio playback
                        self.audio.play_audio(sample_rate, channels, samples).await;
                    }
                }
                vec![] // Return processed effects
            }
            other_effect => {
                // Convert Effect to AppEffect
                let app_effect = match other_effect {
                    Effect::Audio(request) => {
                        AppEffect::Audio(request.operation)
                    },
                    Effect::Http(request) => {
                        AppEffect::Http(request.operation)
                    },
                    Effect::LiveKit(request) => {
                        AppEffect::LiveKit(request.operation)
                    },
                    Effect::Render(request) => {
                        AppEffect::Render(request.operation)
                    },
                };
                self.core.process_event(Event::AppEffect(app_effect))
            }
        }
    }
}

impl CruxApp for AudioMiddleware {
    type Event = Event;
    type Effect = Effect;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(
        &self,
        event: Self::Event,
        _model: &mut Self::Model,
        _caps: &Self::Capabilities,
    ) -> crux_core::Command<Self::Effect, Self::Event> {
        match &event {
            Event::Audio(audio_event) => {
                match &event {
                    Event::Audio(AudioEvent::StartRecordingRequested) => {
                        // Obtain audio_source from somewhere (may need to inject into middleware)
                        self.start_mic_capture(audio_source.clone());
                    }
                    _ => todo!()
                    // AudioEvent::RecordingStateChanged(state) => {
                    //     // Handle recording state changes
                    //     log::info!("Recording state changed: {:?}", state);
                    // }
                    // AudioEvent::RecordingComplete(data) => {
                    //     // Handle recording completion
                    //     log::info!("Recording complete: {:?}", data);
                    // }
                    // AudioEvent::RecordingError(err) => {
                    //     log::error!("Recording error: {}", err);
                    // }
                }
            }
            _ => {}
        }

        let effects = self.core.process_event(event);
        
        Command::new(move |ctx| async move {
            for effect in effects {
                match effect {
                    Effect::Audio(request) => {
                        // Access the operation field directly
                        let event = match request.operation {
                            AudioOperation::PlaybackAudio { sample_rate, channels, samples } => {
                                Event::Audio(AudioEvent::RecordingComplete(AudioData {
                                    samples: samples.clone(),
                                    sample_rate,
                                    channels,
                                }))
                            }
                            AudioOperation::ToggleRecording => {
                                Event::Audio(AudioEvent::RecordingStateChanged(RecordingState::Recording))
                            }
                        };
                        ctx.send_event(event);
                    }
                    // Handle other effect types if needed
                    _ => {}
                }
            }
        })
    }

    fn view(&self, _model: &Self::Model) -> Self::ViewModel {
        self.core.view()
    }
}
