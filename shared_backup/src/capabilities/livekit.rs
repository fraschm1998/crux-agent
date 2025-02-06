// #[derive(Debug)]
// pub enum LiveKitCommand {
//     JoinRoom(String),
//     LeaveRoom,
//     // StartDebate(DebateContext),
//     SendAudio(Vec<u8>),
//     StopAudio,
// }

// use livekit::{Room, RoomOptions, TrackPublication};

use futures::StreamExt;
use livekit::{track::RemoteTrack, webrtc::audio_stream::native::NativeAudioStream, RoomEvent};

use crate::{
    capabilities,
    events::{self, audio::AudioEvent},
    Capabilities, Effect, RUNTIME,
};

#[derive(Default)]
pub struct Model {
    room_state: RoomState,
    current_room: Option<String>,
    error: Option<String>,
}

#[derive(Default, PartialEq, Eq)]
pub enum RoomState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}

// #[derive(Clone)]
// pub enum Event {
//     JoinRoom(String, String), // (room_name, token)
//     LeaveRoom,
//     RoomConnected,
//     RoomDisconnected,
//     RoomError(String),
//     ParticipantJoined(String),
//     ParticipantLeft(String),
// }

// // This would be your main capability for Livekit operations
// pub struct LivekitCore {
//     room: Option<Room>,
//     event_tx: mpsc::Sender<Event>,
// }
//
// impl LivekitCore {
//     pub fn new(event_tx: mpsc::Sender<Event>) -> Self {
//         Self {
//             room: None,
//             event_tx,
//         }
//     }
//
//     pub async fn join_room(&mut self, url: String, token: String) -> Result<(), String> {
//         let room = Room::connect(&url, &token, RoomOptions::default())
//             .await
//             .map_err(|e| e.to_string())?;
//
//         // Set up room event handlers
//         let event_tx = self.event_tx.clone();
//         room.on_connected(move |_| {
//             let tx = event_tx.clone();
//             async move {
//                 let _ = tx.send(Event::RoomConnected).await;
//             }
//         });
//
//         let event_tx = self.event_tx.clone();
//         room.on_disconnected(move |_| {
//             let tx = event_tx.clone();
//             async move {
//                 let _ = tx.send(Event::RoomDisconnected).await;
//             }
//         });
//
//         // Handle participant events
//         let event_tx = self.event_tx.clone();
//         room.on_participant_connected(move |participant| {
//             let tx = event_tx.clone();
//             async move {
//                 let _ = tx
//                     .send(Event::ParticipantJoined(participant.identity().to_string()))
//                     .await;
//             }
//         });
//
//         self.room = Some(room);
//         Ok(())
//     }
//
//     pub async fn leave_room(&mut self) {
//         if let Some(room) = self.room.take() {
//             room.disconnect().await;
//         }
//     }
// }

// #[derive(Debug)]
// pub enum LiveKitEvent {
//     RoomJoined(String),
//     RoomLeft,
//     DebateStarted,
//     AudioReceived(Vec<u8>),
//     Error(String),
// }
//
// // Define the capability
// pub struct LiveKitCapability;
//
// impl Capability for LiveKitCapability {
//     type Event = LiveKitEvent;
//     type Command = LiveKitCommand;
// }

// use serde::{Deserialize, Serialize};
//
// use crux_core::capability::Operation;
//
// // Audio Operation types
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
// pub enum AudioOperation {
//     ToggleRecording,
// }
//
// impl Operation for AudioOperation {
//     type Output = Result<AudioResponse, AudioError>;
// }
//
// #[derive(crux_core::macros::Capability, Clone)]
// pub struct Audio<Event> {
//     context: CapabilityContext<AudioOperation, Event>,
//     recording_state: Arc<Mutex<AudioState>>,
// }
//
// impl<Event> Audio<Event>
// where
//     Event: 'static + Send,
// {
//     pub fn new(context: CapabilityContext<AudioOperation, Event>) -> Self {
//         Self {
//             context,
//             recording_state: Arc::new(Mutex::new(AudioState {
//                 stream: None,
//                 samples: Vec::new(),
//                 config: None,
//                 state: RecordingState::Idle,
//             })),
//         }
//     }
//
//     pub fn start_recording(&self) -> Result<(), AudioError> {
//         log::info!("Starting audio recording setup...");
//
//         let host = cpal::default_host();
//         log::info!("Got default host");
//
//         let device = host
//             .default_input_device()
//             .ok_or(AudioError::DeviceNotFound)?;
//         log::info!("Got default input device");
//
//         // Get the default config
//         let default_config = device
//             .default_input_config()
//             .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
//
//         log::info!("Default config: {:?}", default_config);
//
//         // Create a known working config for Android
//         let config = cpal::StreamConfig {
//             channels: 1,
//             sample_rate: cpal::SampleRate(44100),
//             buffer_size: cpal::BufferSize::Fixed(1024),
//         };
//
//         log::info!("Using stream config: {:?}", config);
//
//         let recording_state = self.recording_state.clone();
//
//         // Error handling callback
//         let err_fn = move |err| {
//             log::error!("An error occurred on stream: {}", err);
//         };
//
//         // Data handling callback
//         let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
//             if let Ok(mut state) = recording_state.try_lock() {
//                 if state.state == RecordingState::Recording {
//                     // Log the first few samples and buffer length for debugging
//                     let preview: Vec<f32> = data.iter().take(5).cloned().collect();
//                     log::info!(
//                         "Recording samples - Buffer size: {}, First few samples: {:?}, Max amplitude: {:.2}",
//                         data.len(),
//                         preview,
//                         data.iter().fold(0f32, |max, &x| max.max(x.abs()))
//                     );
//
//                     state.samples.extend_from_slice(data);
//                 }
//             }
//         };
//
//         log::info!("Building input stream...");
//
//         // Build the stream with explicit config
//         let stream = device
//             .build_input_stream(
//                 &config,
//                 input_data_fn,
//                 err_fn,
//                 Some(std::time::Duration::from_secs(1)),
//             )
//             .map_err(|e| {
//                 log::error!("Failed to build input stream: {}", e);
//                 AudioError::RecordingFailed(e.to_string())
//             })?;
//
//         log::info!("Stream built successfully, attempting to play...");
//
//         // Try to play the stream
//         stream.play().map_err(|e| {
//             log::error!("Failed to play stream: {}", e);
//             AudioError::RecordingFailed(e.to_string())
//         })?;
//
//         log::info!("Stream playing successfully");
//
//         // Update state
//         let mut state = self.recording_state.lock().unwrap();
//         state.stream = Some(stream);
//         state.config = Some(config);
//         state.samples.clear();
//         state.state = RecordingState::Recording;
//
//         log::info!("Recording started successfully");
//         Ok(())
//     }
//
//     pub fn stop_recording(&self) -> Result<AudioData, AudioError> {
//         let mut state = self.recording_state.lock().unwrap();
//
//         // Take ownership of the stream and drop it to stop recording
//         let _stream = state.stream.take().ok_or(AudioError::InvalidState)?;
//
//         let config = state.config.as_ref().ok_or(AudioError::InvalidState)?;
//         let audio_data = AudioData {
//             samples: state.samples.clone(),
//             sample_rate: config.sample_rate.0,
//             channels: config.channels,
//         };
//
//         state.state = RecordingState::Finished;
//         Ok(audio_data)
//     }
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
// pub struct SseRequest {
//     pub url: String,
// }

// #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
// pub enum SseResponse {
//     Chunk(Vec<u8>),
//     Done,
// }

// impl Operation for SseRequest {
//     type Output = SseResponse;
// }

// #[derive(crux_core::macros::Capability)]
// pub struct ServerSentEvents<Ev> {
//     context: CapabilityContext<SseRequest, Ev>,
// }

// impl<Ev> ServerSentEvents<Ev>
// where
//     Ev: 'static,
// {
//     pub fn new(context: CapabilityContext<SseRequest, Ev>) -> Self {
//         Self { context }
//     }
//
//     pub fn get_json<F, T>(&self, url: impl AsRef<str>, make_event: F)
//     where
//         F: Fn(T) -> Ev + Clone + Send + 'static,
//         T: DeserializeOwned,
//     {
//         self.context.spawn({
//             let context = self.context.clone();
//             let url = url.as_ref().to_string();
//
//             async move {
//                 let mut stream = context.stream_from_shell(SseRequest { url });
//
//                 while let Some(response) = stream.next().await {
//                     let make_event = make_event.clone();
//
//                     match response {
//                         SseResponse::Chunk(data) => {
//                             let mut reader = decode(Cursor::new(data));
//
//                             while let Some(sse_event) = reader.next().await {
//                                 if let Ok(Event::Message(msg)) = sse_event {
//                                     let t: T = serde_json::from_slice(msg.data()).unwrap();
//                                     context.update_app(make_event(t));
//                                 }
//                             }
//                         }
//                         SseResponse::Done => break,
//                     }
//                 }
//             }
//         });
//     }
// }

use crate::Event as AppEvent;
use crux_core::{
    capability::{CapabilityContext, Operation},
    compose, Command,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LiveKitOperation {
    JoinRoom(String, String), // room name, token
    LeaveRoom,
    HandleRoomEvent(RoomEventWrapper),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LiveKitResponse {
    RoomJoined,
    RoomLeft,
    EventHandled,
    Error(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LiveKitError {
    ConnectionFailed(String),
    InvalidState,
}

impl std::fmt::Display for LiveKitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiveKitError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            LiveKitError::InvalidState => write!(f, "Invalid state"),
        }
    }
}

impl Operation for LiveKitOperation {
    // type Output = ();
    // type Output = Result<LiveKitResponse, LiveKitError>;
    type Output = Result<AppEvent, LiveKitError>;
}

// Wrapper for RoomEvent since we can't derive Serialize/Deserialize for it directly
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RoomEventWrapper {
    event_type: String,
    participant_id: Option<String>,
    track_sid: Option<String>,
    // Add other necessary fields
}

#[derive(Default)]
struct LiveKitState {
    room_state: RoomState,
    current_room: Option<String>,
}

#[derive(crux_core::macros::Capability, Clone)]
pub struct LiveKit<Event> {
    context: CapabilityContext<LiveKitOperation, Event>,
    state: Arc<Mutex<LiveKitState>>,
}

impl<Event> LiveKit<Event>
where
    Event: 'static + Send,
    AppEvent: 'static + Send,
    // Event: 'static,
{
    pub fn new(context: CapabilityContext<LiveKitOperation, Event>) -> Self {
        Self {
            context,
            state: Arc::new(Mutex::new(LiveKitState::default())),
        }
    }

    pub async fn join_room(&self, room_name: String, token: String) -> Result<(), LiveKitError> {
        let mut state = self.state.lock().unwrap();

        if state.room_state != RoomState::Disconnected {
            return Err(LiveKitError::InvalidState);
        }

        state.room_state = RoomState::Connecting;
        state.current_room = Some(room_name);

        // Here you would implement the actual room connection logic
        // For now we'll just simulate it

        state.room_state = RoomState::Connected;
        Ok(())
    }

    pub async fn leave_room(&self) -> Result<(), LiveKitError> {
        let mut state = self.state.lock().unwrap();

        if state.room_state != RoomState::Connected {
            return Err(LiveKitError::InvalidState);
        }

        state.room_state = RoomState::Disconnected;
        state.current_room = None;

        Ok(())
    }

    pub async fn handle_room_event(&self, event: RoomEvent) -> Result<AppEvent, LiveKitError> {
        //     match event {
        //         RoomEvent::TrackSubscribed { track, publication, participant } => {
        //             if let RemoteTrack::Audio(audio_track) = track {
        //                 let lk_stream = NativeAudioStream::new(
        //                     audio_track.rtc_track(),
        //                     44100,
        //                     1,
        //                 );
        //
        //                 if let Err(e) = caps.audio.playback_audio(lk_stream, 44100, 1) {
        //                     log::error!("Failed to start audio playback: {:?}", e);
        //                     return Err(LiveKitError::ConnectionFailed(e.to_string()));
        //                 }
        //             }
        //         }
        //         // Handle other events...
        //         _ => {
        //             log::debug!("Unhandled room event: {:?}", event);
        //         }
        //     }
        //     Ok(())
        // }
        // pub async fn handle_room_event(event: RoomEvent, caps: Capabilities) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            RoomEvent::Connected {
                participants_with_tracks,
            } => {
                log::info!(
                    "Room connected with {} initial participants",
                    participants_with_tracks.len()
                );
                for (participant, tracks) in participants_with_tracks {
                    log::info!(
                        "Initial participant: {} with {} tracks",
                        participant.identity(),
                        tracks.len()
                    );
                }
            }
            RoomEvent::Disconnected { reason } => {
                log::info!("Room disconnected: {:?}", reason);
            }
            RoomEvent::Reconnecting => {
                log::info!("Reconnecting to room...");
            }
            RoomEvent::Reconnected => {
                log::info!("Successfully reconnected to room");
            }
            RoomEvent::ParticipantConnected(participant) => {
                log::info!("Participant connected: {}", participant.identity());
            }
            RoomEvent::ParticipantDisconnected(participant) => {
                log::info!("Participant disconnected: {}", participant.identity());
            }
            RoomEvent::LocalTrackPublished {
                publication,
                track,
                participant,
            } => {
                log::info!(
                    "Local track published: {} by {}",
                    publication.sid(),
                    participant.identity()
                );
            }
            RoomEvent::TrackPublished {
                publication,
                participant,
            } => {
                log::info!(
                    "Remote track published: {} by {}",
                    publication.sid(),
                    participant.identity()
                );
            }
            RoomEvent::TrackSubscribed {
                track,
                publication,
                participant,
            } => {
                log::error!(
                    "Subscribed to track: {} ({}) from {}",
                    track.sid(),
                    publication.sid(),
                    participant.identity()
                );

                // if let RemoteTrack::Audio(audio_track) = track {
                //     // Execute directly in the current context instead of spawning
                //     return Ok(AppEvent::Audio(AudioEvent::PlaybackAudio {
                //         track: audio_track.rtc_track(),
                //         sample_rate: 44100,
                //         num_channels: 1,
                //     }));
                // }
                if let RemoteTrack::Audio(audio_track) = track {
                    return Ok(AppEvent::Audio(AudioEvent::PlaybackAudio {
                        track: audio_track.rtc_track(),
                    }));
                }

                // if let RemoteTrack::Audio(audio_track) = track {
                //     self.context.spawn({
                //         // let context = self.context.clone();

                //         async move {
                //             let _ = Command::<Effect, crate::Event>::event(AppEvent::Audio(AudioEvent::PlaybackAudio {
                //                     track: audio_track.rtc_track(),
                //                     sample_rate: 44100,
                //                     num_channels: 1,
                //                 }));
                //             }

                //         // async move {
                //         //     context.update_app(AppEvent::Audio(AudioEvent::PlaybackAudio {
                //         //         track: audio_track.rtc_track(),
                //         //         sample_rate: 44100,
                //         //         num_channels: 1,
                //         //     }));
                //         // }
                //     })
                // }

                // let _ = Command::<Effect, events::Event>::event(AppEvent::Audio(AudioEvent::PlaybackAudio {
                //         track: audio_track.rtc_track(),
                //         sample_rate: 44100,
                //         num_channels: 1,
                //     }));

                // self.context.spawn(async move {
                //     log::error!("AUDIO TRACK PROCESSING IN TOKIO RUNTIME");
                //
                //     // Process audio track in Tokio runtime
                //     let _ = Command::<Effect, events::Event>::event(AppEvent::Audio(AudioEvent::PlaybackAudio {
                //         track: audio_track.rtc_track(),
                //         sample_rate: 44100,
                //         num_channels: 1,
                //     }));
                // });

                // if let RemoteTrack::Audio(audio_track) = track {
                //     log::info!("Received audio track - setting up playback");
                //
                //     let mut lk_stream = NativeAudioStream::new(audio_track.rtc_track(), 44100, 1);
                //
                //     // Convert the stream to Vec<f32>
                //     if let Some(frame) = futures::executor::block_on(lk_stream.next()) {
                //         // Convert the audio data to f32 samples
                //         let samples: Vec<f32> = frame
                //             .data
                //             .iter()
                //             .map(|&sample| {
                //                 // Convert i16 to f32 and normalize
                //                 (sample as f32) / (i16::MAX as f32)
                //             })
                //             .collect();
                //
                //         // Dispatch audio playback event through the Command system
                //         let _ = Command::<Effect, events::Event>::event(AppEvent::Audio(
                //             AudioEvent::PlaybackAudio(samples),
                //         ));
                //         // self.context.update_app(Event::Audio(AudioEvent::PlaybackAudio(samples)));
                //         // if let Err(e) = Command::event(AppEvent::Audio(AudioEvent::PlaybackAudio(samples))) {
                //         //     log::error!("Failed to dispatch audio playback event: {:?}", e);
                //         // }
                //     } else {
                //         log::error!("Failed to get audio frame from stream");
                //     }
                // }
                //
                //
                //
                //
                //
                //
                //
                // log::info!(
                //     "Subscribed to track: {} ({}) from {}",
                //     track.sid(),
                //     publication.sid(),
                //     participant.identity()
                // );
                //
                // // Handle audio track specifically
                // if let RemoteTrack::Audio(audio_track) = track {
                //     log::info!("Received audio track - setting up playback");
                //
                //     // Create an audio stream to receive the data
                //     let lk_stream = NativeAudioStream::new(
                //         audio_track.rtc_track(),
                //         44100, // sample rate - should match the publishing side
                //         1,     // num channels (mono) - should match the publishing side
                //     );
                //
                //     // Set up CPAL audio output
                //     use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
                //
                //     let host = cpal::default_host();
                //     log::info!("Using audio host: {:?}", host.id());
                //
                //     let device = host.default_output_device()
                //         .expect("no output device available");
                //     log::info!("Using output device: {:?}", device.name());
                //
                //     let config = cpal::StreamConfig {
                //         channels: 1,
                //         sample_rate: cpal::SampleRate(44100),
                //         buffer_size: cpal::BufferSize::Default,
                //     };
                //
                //     let mut audio_stream = lk_stream;
                //
                //     // Create the output stream
                //     let output_stream = device.build_output_stream(
                //         &config,
                //         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                //             // Get audio samples from LiveKit stream
                //             if let Some(frame) = futures::executor::block_on(audio_stream.next()) {
                //                 let len = frame.samples_per_channel as usize * frame.num_channels as usize;
                //                 let len = len.min(data.len());
                //
                //                 // Convert samples to f32 and apply proper scaling
                //                 for i in 0..len {
                //                     if i < frame.data.len() {
                //                         // Convert i16 to f32 with proper scaling and amplification
                //                         // Using a scaling factor of 1.5 to boost volume while preventing clipping
                //                         let sample = frame.data[i] as f32;
                //                         let normalized = (sample / i16::MAX as f32) * 1.5;
                //                         // Clamp to prevent distortion
                //                         data[i] = normalized.clamp(-1.0, 1.0);
                //                     }
                //                 }
                //
                //                 // Fill remaining buffer with silence if needed
                //                 if len < data.len() {
                //                     data[len..].fill(0.0);
                //                 }
                //             } else {
                //                 data.fill(0.0);
                //             }
                //         },
                //         |err| log::error!("Audio output error: {}", err),
                //         None
                //     ).expect("Failed to build output stream");
                //
                //     // Start playback immediately
                //     output_stream.play().expect("Failed to start audio playback");
                //     log::info!("Audio playback started successfully");
                //
                //     // Keep the stream alive by storing it in a static or other long-lived storage
                //     // This is important - the stream needs to stay alive!
                //     std::mem::forget(output_stream);
                // }
            }
            RoomEvent::TrackUnsubscribed {
                track,
                publication,
                participant,
            } => {
                log::info!(
                    "Unsubscribed from track: {} ({}) from {}",
                    track.sid(),
                    publication.sid(),
                    participant.identity()
                );
            }
            RoomEvent::TrackSubscriptionFailed {
                participant,
                error,
                track_sid,
            } => {
                log::error!(
                    "Failed to subscribe to track {} from {}: {:?}",
                    track_sid,
                    participant.identity(),
                    error
                );
            }
            RoomEvent::TrackMuted {
                participant,
                publication,
            } => {
                log::info!(
                    "Track muted: {} by {}",
                    publication.sid(),
                    participant.identity()
                );
            }
            RoomEvent::TrackUnmuted {
                participant,
                publication,
            } => {
                log::info!(
                    "Track unmuted: {} by {}",
                    publication.sid(),
                    participant.identity()
                );
            }
            RoomEvent::ConnectionStateChanged(state) => {
                log::info!("Connection state changed: {:?}", state);
            }
            RoomEvent::ConnectionQualityChanged {
                quality,
                participant,
            } => {
                log::info!(
                    "Connection quality for {}: {:?}",
                    participant.identity(),
                    quality
                );
            }
            // RoomEvent::ActiveSpeakersChanged { speakers } => {
            //     let speaker_count = speakers.len();
            //     if speaker_count > 0 {
            //         let identities = futures::future::join_all(
            //             speakers.iter().map(|p| p.identity())
            //         ).await;
            //         log::info!("Active speakers: {}", identities.join(", "));
            //     } else {
            //         log::info!("No active speakers");
            //     }
            // }
            RoomEvent::DataReceived {
                payload,
                topic,
                kind,
                participant,
            } => {
                if let Some(p) = participant {
                    log::info!(
                        "Received data from {}, topic: {:?}, kind: {:?}, size: {} bytes",
                        p.identity(),
                        topic,
                        kind,
                        payload.len()
                    );
                } else {
                    log::info!(
                        "Received data, topic: {:?}, kind: {:?}, size: {} bytes",
                        topic,
                        kind,
                        payload.len()
                    );
                }
            }
            RoomEvent::ChatMessage {
                message,
                participant,
            } => {
                if let Some(p) = participant {
                    log::info!("Chat message from {}: {:?}", p.identity(), message);
                } else {
                    log::info!("Chat message: {:?}", message);
                }
            }
            _ => {
                log::debug!("Unhandled room event: {:?}", event);
            }
        }
        Ok(AppEvent::Nothing)
    }
}
