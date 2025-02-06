// use crux_core::bridge::Bridge;
//
// use crate::{Event, RUNTIME};
//
// // Audio Middleware
// pub struct AudioMiddleware {
//     core: Bridge,
//     audio_system: AudioSystem,
// }
//
// impl AudioMiddleware {
//     pub fn new(core: Bridge, audio_system: AudioSystem) -> Self {
//         Self { core, audio_system }
//     }
//
//     pub fn handle_event(&self, event: Event) {
//         match event {
//             Event::Audio(audio_event) => {
//                 // Handle audio-specific events
//                 match audio_event {
//                     AudioEvent::StartRecordingRequested(audio_source) => {
//                         // Start the audio recording in a separate tokio task
//                         let core = self.core.clone();
//                         let audio_system = self.audio_system.clone();
//                         
//                         RUNTIME.spawn(async move {
//                             match audio_system.start_recording(audio_source).await {
//                                 Ok(_) => core.send_event(Event::Audio(AudioEvent::RecordingStarted)),
//                                 Err(e) => core.send_event(Event::Audio(AudioEvent::Error(e.to_string()))),
//                             }
//                         });
//                     },
//                     // Handle other audio events...
//                 }
//             },
//             Event::LiveKit(livekit_event) => {
//                 // Handle LiveKit events
//                 let core = self.core.clone();
//                 RUNTIME.spawn(async move {
//                     // Your LiveKit handling code here
//                     // Use core.send_event() to send events back to Crux
//                 });
//             },
//             // Handle other events...
//         }
//     }
// }
//
// // use crate::app::{App, Event, Effect};
// // use crate::capabilities::audio::{Audio, AudioData, AudioOperation, RecordingState};
// // use crate::{Capabilities, Model, ViewModel, RUNTIME};
// // use crux_core::{App as CruxApp, Capability, Command, Core};
// // use livekit::webrtc::{
// //     audio_source::{native::NativeAudioSource, AudioSourceOptions},
// //     prelude::*,
// // };
// // use std::sync::Arc;
// // use tokio::task::JoinHandle;
// //
// // pub struct AudioMiddleware {
// //     core: Core<App>,
// //     audio: Audio<Event>,
// //     mic_task: Option<JoinHandle<()>>,
// // }
// //
// // impl Default for AudioMiddleware {
// //     fn default() -> Self {
// //         Self {
// //             core: Core::default(),
// //             audio: Audio::default(),
// //             mic_task: None,
// //         }
// //     }
// // }
// //
// // impl AudioMiddleware {
// //     /// Creates a new middleware with a given `core` and `audio` capability.
// //     pub fn new(core: Core<App>, audio: Audio<Event>) -> Self {
// //         Self {
// //             core,
// //             audio,
// //             mic_task: None,
// //         }
// //     }
// //
// //     /// Processes effects from Crux Core
// //     pub fn handle_effect(&mut self, effect: Effect) -> Vec<Effect> {
// //         match effect {
// //             Effect::Audio(audio_effect) => match audio_effect.operation {
// //                 AudioOperation::ToggleRecording => {
// //                     if self.mic_task.is_none() {
// //                         let audio_source = Arc::new(NativeAudioSource::new(
// //                             livekit::webrtc::prelude::AudioSourceOptions::default(),
// //                             44100,
// //                             1,
// //                             100,
// //                         ));
// //                         self.start_mic_capture(audio_source);
// //                     } else {
// //                         self.stop_mic_capture();
// //                     }
// //                     vec![] // Effect is handled
// //                 }
// //                 AudioOperation::PlaybackAudio {
// //                     sample_rate,
// //                     channels,
// //                     samples,
// //                 } => {
// //                     let audio = self.audio.clone();
// //                     RUNTIME.spawn(async move {
// //                         // Create an audio source
// //                         let audio_source = NativeAudioSource::new(
// //                             AudioSourceOptions::default(),
// //                             sample_rate,
// //                             channels as u32,
// //                             100, // buffer_size
// //                         );
// //                         
// //                         // Push the samples to the audio source
// //                         audio_source.push_samples(&samples);
// //                         
// //                         // Create the audio track from the source
// //                         let track = audio_source.create_track();
// //                         
// //                         if let Err(e) = audio.playback_audio(track) {
// //                             log::error!("Audio playback failed: {:?}", e);
// //                         }
// //                     });
// //                     vec![]
// //                 }
// //             },
// //             other_effect => {
// //                 // Let core handle non-audio effects
// //                 self.core.process_event(Event::AppEffect(other_effect))
// //             }
// //         }
// //     }
// //
// //     /// Starts microphone capture asynchronously.
// //     pub fn start_mic_capture(&mut self, audio_source: Arc<NativeAudioSource>) {
// //         let audio = self.audio.clone();
// //         log::info!("Starting microphone capture in background task...");
// //
// //         self.mic_task = Some(RUNTIME.spawn(async move {
// //             audio.capture_mic_audio(audio_source).await;
// //             log::info!("Microphone capture task completed.");
// //         }));
// //     }
// //
// //     /// Stops microphone capturing.
// //     pub fn stop_mic_capture(&mut self) {
// //         if let Some(task) = self.mic_task.take() {
// //             task.abort();
// //             log::info!("Microphone capture stopped.");
// //         }
// //     }
// //
// //     /// Processes an incoming audio effect (e.g., from WASM or native layer).
// //     pub fn process_audio_effect(&self, data: &[u8]) -> Vec<u8> {
// //         log::info!("Processing incoming audio effect...");
// //         
// //         // Deserialize received data into an audio event
// //         if let Ok(effect) = bincode::deserialize::<Effect>(data) {
// //             let result_effects = self.handle_effect(effect);
// //             
// //             // Serialize the output effects
// //             bincode::serialize(&result_effects).unwrap_or_else(|_| vec![])
// //         } else {
// //             log::error!("Failed to deserialize audio effect");
// //             vec![]
// //         }
// //     }
// // }
// //
// // impl CruxApp for AudioMiddleware {
// //     type Model = Model;
// //     type Event = Event;
// //     type ViewModel = ViewModel;
// //     type Capabilities = Capabilities;
// //     type Effect = Effect;
// //
// //     /// Processes an event and returns commands.
// //     fn update(
// //         &self,
// //         event: Self::Event,
// //         _model: &mut Self::Model,
// //         _caps: &Self::Capabilities,
// //     ) -> Command<Self::Effect, Self::Event> {
// //         let effects = self.core.process_event(event);
// //
// //         Command::new(move |ctx| async move {
// //             for effect in effects {
// //                 if let Effect::Audio(request) = effect {
// //                     let event = match request.operation {
// //                         AudioOperation::PlaybackAudio {
// //                             sample_rate,
// //                             channels,
// //                             samples,
// //                         } => Event::Audio(AudioEvent::RecordingComplete(AudioData {
// //                             samples,
// //                             sample_rate,
// //                             channels,
// //                         })),
// //                         AudioOperation::ToggleRecording => {
// //                             Event::Audio(AudioEvent::RecordingStateChanged(RecordingState::Recording))
// //                         }
// //                     };
// //                     ctx.send_event(event);
// //                 }
// //             }
// //         })
// //     }
// //
// //     /// Returns the current view model state.
// //     fn view(&self, _model: &Self::Model) -> Self::ViewModel {
// //         self.core.view()
// //     }
// // }
// //
// //
// // // use crate::app::{App, Event, Effect};
// // // use crate::events::audio::AudioEvent;
// // // use crate::capabilities::audio::{Audio, AudioData, AudioOperation, RecordingState};
// // // use crate::{AppEffect, Capabilities, Model, ViewModel, RUNTIME};
// // // use crux_core::{App as CruxApp, Capability, Command, Core};
// // // use livekit::webrtc::audio_source::native::NativeAudioSource;
// // // use tokio::task::JoinHandle;
// // // use std::sync::Arc;
// //
// // // pub struct AudioMiddleware {
// // //     core: Core<App>,
// // //     audio: Audio<Event>,
// // //     mic_task: Option<JoinHandle<()>>,
// // // }
// //
// // // impl Default for AudioMiddleware {
// // //     fn default() -> Self {
// // //         // Create a placeholder instance that should be replaced with new()
// // //         Self {
// // //             core: Core::default(),
// // //             audio: Audio::default(),
// // //             mic_task: None,
// // //         }
// // //     }
// // // }
// //
// // // impl AudioMiddleware {
// // //     pub fn new(core: Core<App>, audio: Audio<Event>) -> Self {
// // //         Self {
// // //             core,
// // //             audio,
// // //             mic_task: None,
// // //         }
// // //     }
// //
// // //     pub fn start_mic_capture(&mut self, audio_source: Arc<NativeAudioSource>) {
// // //         let audio = self.audio.clone();
// //         
// // //         // Start mic capture in a separate task
// // //         self.mic_task = Some(RUNTIME.spawn(async move {
// // //             // Just await the result since capture_mic_audio returns ()
// // //             audio.capture_mic_audio(audio_source).await;
// // //         }));
// // //     }
// //
// // //     pub fn handle_effect(&mut self, effect: Effect) -> Vec<Effect> {
// // //         match effect {
// // //             Effect::Audio(audio_effect) => {
// // //                 match audio_effect.operation {
// // //                     AudioOperation::ToggleRecording => {
// // //                         // Start/stop mic capture
// // //                         if self.mic_task.is_none() {
// // //                             // Assume audio_source is accessible here; may need to pass it in
// // //                             self.start_mic_capture(audio_source);
// // //                         } else {
// // //                             self.stop_mic_capture();
// // //                         }
// // //                     }
// // //                     AudioOperation::PlaybackAudio { sample_rate, channels, samples } => {
// // //                         // Handle audio playback
// // //                         // Ensure this is called within an async context
// // //                         tokio::spawn(async move {
// // //                             self.audio.playback_audio(sample_rate, channels, samples).unwrap();
// // //                         });
// // //                     }
// // //                 }
// // //                 vec![] // Return processed effects
// // //             }
// // //             other_effect => {
// // //                 // Convert Effect to AppEffect
// // //                 let app_effect = match other_effect {
// // //                     Effect::Audio(request) => {
// // //                         AppEffect::Audio(request.operation)
// // //                     },
// // //                     Effect::Http(request) => {
// // //                         AppEffect::Http(request.operation)
// // //                     },
// // //                     Effect::LiveKit(request) => {
// // //                         AppEffect::LiveKit(request.operation)
// // //                     },
// // //                     Effect::Render(request) => {
// // //                         AppEffect::Render(request.operation)
// // //                     },
// // //                 };
// // //                 self.core.process_event(Event::AppEffect(app_effect))
// // //             }
// // //         }
// // //     }
// // // }
// //
// // // impl CruxApp for AudioMiddleware {
// // //     type Event = Event;
// // //     type Effect = Effect;
// // //     type Model = Model;
// // //     type ViewModel = ViewModel;
// // //     type Capabilities = Capabilities;
// //
// // //     fn update(
// // //         &self,
// // //         event: Self::Event,
// // //         _model: &mut Self::Model,
// // //         _caps: &Self::Capabilities,
// // //     ) -> crux_core::Command<Self::Effect, Self::Event> {
// // //         match &event {
// // //             Event::Audio(audio_event) => {
// // //                 match &event {
// // //                     Event::Audio(AudioEvent::StartRecordingRequested) => {
// // //                         // Obtain audio_source from somewhere (may need to inject into middleware)
// // //                         self.start_mic_capture(audio_source.clone());
// // //                     }
// // //                     _ => todo!()
// // //                     // AudioEvent::RecordingStateChanged(state) => {
// // //                     //     // Handle recording state changes
// // //                     //     log::info!("Recording state changed: {:?}", state);
// // //                     // }
// // //                     // AudioEvent::RecordingComplete(data) => {
// // //                     //     // Handle recording completion
// // //                     //     log::info!("Recording complete: {:?}", data);
// // //                     // }
// // //                     // AudioEvent::RecordingError(err) => {
// // //                     //     log::error!("Recording error: {}", err);
// // //                     // }
// // //                 }
// // //             }
// // //             _ => {}
// // //         }
// //
// // //         let effects = self.core.process_event(event);
// //         
// // //         Command::new(move |ctx| async move {
// // //             for effect in effects {
// // //                 match effect {
// // //                     Effect::Audio(request) => {
// // //                         // Access the operation field directly
// // //                         let event = match request.operation {
// // //                             AudioOperation::PlaybackAudio { sample_rate, channels, samples } => {
// // //                                 Event::Audio(AudioEvent::RecordingComplete(AudioData {
// // //                                     samples: samples.clone(),
// // //                                     sample_rate,
// // //                                     channels,
// // //                                 }))
// // //                             }
// // //                             AudioOperation::ToggleRecording => {
// // //                                 Event::Audio(AudioEvent::RecordingStateChanged(RecordingState::Recording))
// // //                             }
// // //                         };
// // //                         ctx.send_event(event);
// // //                     }
// // //                     // Handle other effect types if needed
// // //                     _ => {}
// // //                 }
// // //             }
// // //         })
// // //     }
// //
// // //     fn view(&self, _model: &Self::Model) -> Self::ViewModel {
// // //         self.core.view()
// // //     }
// // // }
// // //     pub fn stop_mic_capture(&mut self) {
// // //         if let Some(task) = self.mic_task.take() {
// // //             task.abort();
// // //             log::info!("Microphone capture stopped.");
// // //         }
// // //     }
// // //     pub fn process_audio_effect(&self, data: &[u8]) -> Vec<u8> {
// // //         // Implement the logic to process audio effects
// // //         // This is a placeholder implementation
// // //         log::info!("Processing audio effect...");
// // //         data.to_vec()
// // //     }
