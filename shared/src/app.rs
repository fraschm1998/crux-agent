use crux_core::Request;
use crux_core::{command::CommandContext, compose::Compose, render, Command};
use crux_http::protocol::HttpRequest;
use livekit::track::{LocalAudioTrack, LocalTrack, RemoteTrack, TrackSource};
use livekit::webrtc::prelude::{AudioSourceOptions, RtcAudioSource};
use livekit::{Room, RoomEvent, RoomOptions};
use std::{default, future::IntoFuture, time::Duration};

use crux_core::render::{Render, RenderOperation};
use crux_http::command::Http;
use crux_http::{http::Url, Config, HttpError};
use serde::{Deserialize, Serialize};

use crate::capabilities::audio::{AudioOperation, RecordingState};
use crate::capabilities::livekit::{LiveKit, LiveKitOperation};
use crate::effects::livekit_effects::LiveKitEffect;
use crate::events::audio::AudioEvent;
// use crate::events::Event;
use crate::{
    capabilities::audio::{Audio, AudioData},
    events::livekit::LiveKitEvent,
};

const API_URL: &str = "http://192.168.20.20:8000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    LiveKit(LiveKitEvent),
    #[serde(skip)]
    Audio(AudioEvent),
    AppEffect(AppEffect),
    #[serde(skip)]
    Nothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEffect {
    Http(HttpRequest),         // Want to make HTTP request
    Audio(AudioOperation),     // Want to play/record audio
    LiveKit(LiveKitOperation), // Want to do something with video call
    Render(RenderOperation),   // Want to update the UI
}

// ANCHOR: model
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Model {
    // TODO: remove this field in future, ViewModel.current_screen can be backstack.last()
    backstack: Vec<View>,
    pub current_screen: View,
    pub title: String,
    // pub state: AppModel,
    // pub settings: Settings,
    // pub system: System,
    pub recording: AudioModel,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AudioModel {
    pub recording_state: RecordingState,
    audio_buffer: Vec<f32>, // or Vec<i16> depending on sample format
    sample_rate: u32,
    channels: u16,
    error: Option<String>,
}

impl Model {
    fn navigate_back(&mut self) {
        self.backstack.pop();
        self.current_screen = match self.backstack.last() {
            Some(screen) => screen.clone(),
            None => View::Home,
        };
    }
    fn navigate_to(&mut self, screen: View) {
        self.backstack.push(screen.clone());
        self.current_screen = screen;
    }
}

#[derive(Debug, Serialize, Deserialize)]
// #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewModel {
    pub title: String,
    pub current_screen: View,
    // pub speech: RecordingView,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum View {
    // Authentication & Onboarding Views
    Splash,
    #[default]
    Home,
    Analytics,
    Profile,
    Report,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    Limited,
}

// ANCHOR: capabilities
#[cfg_attr(feature = "typegen", derive(crux_core::macros::Export))]
#[derive(crux_core::macros::Effect)]
pub struct Capabilities {
    pub audio: Audio<Event>,
    pub livekit: LiveKit<Event>,
    pub render: Render<Event>,
    pub http: crux_http::Http<Event>,

    #[effect(skip)]
    pub compose: Compose<Event>,
}
// ANCHOR_END: capabilities

#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Model = Model;
    type Event = Event;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;
    type Effect = Effect;

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
        caps: &Self::Capabilities,
    ) -> Command<Effect, Event> {
        match event {
            Event::Nothing => Command::done(),
            Event::AppEffect(_) => Command::done(),
            Event::LiveKit(livekit_event) => match livekit_event {
                LiveKitEvent::JoinRoom => {
                    Command::notify_shell(LiveKitOperation::JoinRoom(
                        "http://192.168.20.20:7880".to_string(),
                        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU".to_string(),
                    ))
                }
                LiveKitEvent::RoomJoined => {
                    log::info!("WE JOINED THE ROOM"); // perhaps specify which room?
                                                      // RoomJoined(Room)
                    // model.current_screen = View::Connected;
                    Command::done()
                }
                LiveKitEvent::Error(error) => {
                    log::error!("LiveKit error: {}", error);
                    // model.current_screen = View::Error;
                    Command::done()
                }
                _ => {
                    log::info!("Unhandled event type in App update");
                    Command::done()
                }
            },
            Event::Audio(audio_event) => {
                match audio_event {
                    AudioEvent::MicrophoneToggleRequested => {
                        model.recording = AudioModel {
                            recording_state: RecordingState::Idle,
                            audio_buffer: Vec::new(),
                            sample_rate: 44100,
                            channels: 1,
                            error: None,
                        };

                        let recording_model = &model.recording;

                        log::info!("Recording State: {:#?}", recording_model.recording_state);
                        match recording_model.recording_state {
                            RecordingState::Recording => {
                                let _ = Command::<Effect, Event>::event(Event::Audio(
                                    AudioEvent::StopRecordingRequested,
                                ));
                            }
                            RecordingState::Idle => {
                                // let _ = Command::<Effect, Event>::event(Event::Audio(
                                //     AudioEvent::StartRecordingRequested(Arc::new(NativeAudioSource::new(
                                //         AudioSourceOptions {
                                //             echo_cancellation: true,
                                //             noise_suppression: true,
                                //             auto_gain_control: true,
                                //             ..Default::default()
                                //         },
                                //         44100,
                                //         1,
                                //         100,
                                //     ))),
                                // ));
                            }
                            //     Event::Audio(AudioEvent::StartRecordingRequested(speech::RecordingConfig {
                            //         sample_rate: 44100,
                            //         channels: 1,
                            //         bit_depth: 16,
                            //         encoding: speech::AudioEncoding::WAV,
                            //         noise_reduction: true,
                            //         echo_cancellation: true,
                            //         auto_gain: true,
                            //         device_id: None,
                            //     }));
                            // },
                            // RecordingState::Ready => todo!(),
                            // RecordingState::Paused => todo!(),
                            // RecordingState::Processing => todo!(),
                            _ => todo!(),
                        }
                        render::render()
                        // if recording.recording_state == RecordingState::Recording {
                        //     Event::Audio(AudioEvent::StopRecordingRequested);
                        // } else if recor
                    }
                    AudioEvent::StartRecordingRequested => {
                        // Should first match authenticated then anything else
                        log::info!(
                            "@222AUTHMODEL RECORDING STATE MODEL {:#?}",
                            &model.recording
                        );
                        model.recording.recording_state = RecordingState::Recording;
                        // caps.audio.start_recording().unwrap();
                        caps.audio.start_recording().unwrap();
                        Command::done()
                    }
                    AudioEvent::StopRecordingRequested => {
                        model.recording.recording_state = RecordingState::Idle;
                        let data = caps.audio.stop_recording().unwrap();
                        Command::event(Event::Audio(AudioEvent::SendRecordingRequested(data)))
                    }
                    AudioEvent::PlaybackAudio { track } => {
                        log::info!("Received PlaybackAudio event in app update");
                        log::info!("Starting audio playback...");
                        match caps.audio.playback_audio(track) {
                            Ok(_) => {
                                log::info!("Audio playback started successfully");
                                render::render()
                            }
                            Err(e) => {
                                log::error!("Failed to start audio playback: {:?}", e);
                                Command::done()
                            }
                        }
                    }
                    _ => todo!(),
                    // AudioEvent::PlaybackAudio(samples) => {
                    //     caps.audio.playback_audio(samples, 44100, 1);
                    //     Command::done()
                    // }
                    // AudioEvent::SendRecordingRequested(audio_data) => {
                    //     // caps.http
                    //     //     .post("http://192.168.20.20:8000/receive_audio")
                    //     //     .body_json(&audio_data)
                    //     //     .expect("could not serialize body")
                    //     //     .send(|response| Event::Audio(AudioEvent::SentRecordingRequested(response)));
                    //     caps.compose.spawn(|ctx| {
                    //         let http = caps.http.clone();
                    //
                    //         async move {
                    //             let request = http
                    //                 .post(format!("{API_URL}/receive_audio"))
                    //                 .body_json(&audio_data)
                    //                 .expect("could not serialize body");
                    //
                    //             log::warn!("RESPONSE123: {:#?}", request);
                    //
                    //             let mut response = request.into_future().await.unwrap();
                    //             log::warn!("Response status: {}", response.status());
                    //
                    //             if response.status() == 200 {
                    //                 match response.body_json::<serde_json::Value>().await {
                    //                     Ok(json_value) => {
                    //                         match serde_json::from_value::<RecordingMetrics>(json_value) {
                    //                             Ok(metrics) => {
                    //                                 ctx.update_app(Event::Audio(AudioEvent::SentRecordingRequested(Ok(metrics))))
                    //                             }
                    //                             Err(e) => {
                    //                                 log::error!("Failed to decode metrics from JSON value: {}", e);
                    //                                 ctx.update_app(Event::Audio(AudioEvent::SentRecordingRequested(Err(HttpError::Json(e.to_string())))))
                    //                             }
                    //                         }
                    //                     }
                    //                     Err(e) => {
                    //                         log::error!("Failed to decode response as JSON: {}", e);
                    //                         ctx.update_app(Event::Audio(AudioEvent::SentRecordingRequested(Err(e))))
                    //                     }
                    //                 }
                    //             } else {
                    //                 log::error!("Request failed with status: {}", response.status());
                    //                 ctx.update_app(Event::Audio(AudioEvent::SentRecordingRequested(Err(
                    //                     HttpError::Json(response.status().to_string())
                    //                 ))))
                    //             }
                    //
                    //             // ctx.update_app(Event::Audio(AudioEvent::SentRecordingRequested(Ok(
                    //             //     response,
                    //             // ))))
                    //         }
                    //     });
                    // }
                }
            } // For your LiveKit use case, the first approach (block_on) is probably better because:
              //
              // You need to maintain context to send events back
              // You want to handle errors directly
              // The room connection needs to be established before proceeding
              // You need to maintain the WebSocket connection
              // --------------------------------------------------------------------------------------
              // The second approach would be better for tasks that:
              //
              // Can run independently
              // Don't need to report back results
              // Should not block other operations
              // Are truly background tasks
              // --------------------------------------------------------------------------------------
              //
              // In your LiveKit case, you need direct communication because:
              //
              // You need to update the UI based on room events
              // You need to handle connection errors
              // You need to maintain state about the connection
              // You need to respond to user actions
              // --------------------------------------------------------------------------------------

              // Event::LiveKit(LiveKitEvent::JoinRoom) => {
              //     RUNTIME.spawn(async move {
              //         let url = "http://192.168.20.20:7880";
              //         let token = "test";
              //
              //         log::info!("Connecting to {} with token {}", url, token);
              //
              //         let mut options = RoomOptions::default();
              //         options.adaptive_stream = false;
              //         options.dynacast = false;
              //
              //         match Room::connect(&url, &token, options).await {
              //             Ok((room, mut events)) => {
              //                 log::info!("Connected to room {}", String::from(room.sid().await));
              //                 while let Some(event) = events.recv().await {
              //                     if let Err(e) = handle_room_event(event).await {
              //                         log::error!("Error handling room event: {:?}", e);
              //                         // Note: You'll need to handle sending events differently here
              //                         // since we're in a different context
              //                         break;
              //                     }
              //                 }
              //             }
              //             Err(err) => {
              //                 log::error!("Failed to connect: {:?}", err);
              //                 // Handle error
              //             }
              //         }
              //     });
              //     Command::done()
              // }
        }
    }

    fn view(&self, model: &Model) -> ViewModel {
        ViewModel {
            title: model.title.clone(),
            current_screen: model.current_screen.clone(),
        }

        // match model.current_screen {
        //     _ => ViewModel {
        //         title: model.title.clone(),
        //         current_screen: model.current_screen.clone(),
        //     },
        // }
    }
}

pub fn setup_logging(level_filter: log::LevelFilter) {
    #[cfg(target_os = "ios")]
    {
        oslog::OsLogger::new("Core")
            .level_filter(level_filter)
            .category_level_filter(
                "geo::algorithm::relate::geomgraph::edge_and_bundle_star",
                log::LevelFilter::Warn,
            )
            .category_level_filter(
                "geo::algorithm::relate::geomgraph::node",
                log::LevelFilter::Warn,
            )
            .category_level_filter(
                "geo::algorithm::relate::relate_operation",
                log::LevelFilter::Warn,
            )
            .init()
            .expect("failed to init logger");
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        // Initialize default logger for other platforms
        env_logger::Builder::new().filter_level(level_filter).init();
    }
}
