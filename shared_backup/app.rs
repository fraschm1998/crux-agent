use crux_core::{command::CommandContext, compose::Compose, render, Command};
use futures::StreamExt;
use livekit::options::TrackPublishOptions;
use livekit::track::Track;
use livekit::track::{LocalAudioTrack, LocalTrack, RemoteTrack, TrackSource};
use livekit::webrtc::audio_source::native::NativeAudioSource;
use livekit::webrtc::audio_stream::native::NativeAudioStream;
use livekit::webrtc::prelude::{AudioSourceOptions, RtcAudioSource};
use livekit::{Room, RoomEvent, RoomOptions};
use std::{default, future::IntoFuture, time::Duration};
use tokio::sync::mpsc;

// use auth::{LoginCredentials, LoginModel, RegisterModel, UnauthenticatedModel};
use crux_core::render::Render;
use crux_http::command::Http;
use crux_http::{http::Url, Config, HttpError};
use serde::{Deserialize, Serialize};

use crate::capabilities::audio::RecordingState;
use crate::capabilities::livekit::LiveKit;
use crate::events::audio::AudioEvent;
// use crate::events::Event;
use crate::{
    capabilities::audio::{Audio, AudioData},
    events::livekit::LiveKitEvent,
};
use crate::{EVENT_SENDER, RUNTIME};

const API_URL: &str = "http://192.168.20.20:8000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    LiveKit(LiveKitEvent),
    #[serde(skip)]
    Audio(AudioEvent), // Audio events are handled internally and don't need serialization
    #[serde(skip)]
    Nothing,
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
            // Event::LiveKit(LiveKitEvent::JoinRoom) => {
            //     caps.compose.spawn(|ctx| async move {
            //         // Signal that connection is in progress
            //         ctx.update_app(Event::LiveKit(LiveKitEvent::Connecting));
            //
            //         let connect_result = RUNTIME.spawn(async move {
            //             let url = "http://192.168.20.20:7880";
            //             let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";
            //
            //             log::info!("Connecting to {} with token {}", url, token);
            //
            //             let mut options = RoomOptions::default();
            //             options.adaptive_stream = false;
            //             options.dynacast = false;
            //
            //             Room::connect(&url, &token, options).await
            //         });
            //
            //         match connect_result.await.expect("runtime spawn failed") {
            //             Ok((room, mut events)) => {
            //                 let room_id = String::from(room.sid().await);
            //                 log::info!("Connected to room {}", room_id);
            //
            //                 // Signal successful connection
            //                 ctx.update_app(Event::LiveKit(LiveKitEvent::Connected(room_id)));
            //
            //                 while let Some(event) = events.recv().await {
            //                     if let Err(e) = handle_room_event(event).await {
            //                         log::error!("Error handling room event: {:?}", e);
            //                         ctx.update_app(Event::LiveKit(LiveKitEvent::Error(e.to_string())));
            //                         break;
            //                     }
            //                 }
            //             }
            //             Err(err) => {
            //                 log::error!("Failed to connect: {:?}", err);
            //                 ctx.update_app(Event::LiveKit(LiveKitEvent::Error(err.to_string())));
            //             }
            //         }
            //
            //         // Signal that we're disconnected (whether due to error or normal exit)
            //         ctx.update_app(Event::LiveKit(LiveKitEvent::Disconnected));
            //     });
            //     Command::done()
            // }
            Event::LiveKit(livekit_event) => match livekit_event {
                //LiveKitEvent::JoinRoom => {
                //    let livekit = caps.livekit.clone();

                //    caps.compose.spawn(|ctx| async move {
                //        log::info!("Starting connection process...");

                //        let ctx_clone = ctx.clone(); // Now ctx is in scope from the closure parameter

                //        let url = "http://192.168.20.20:7880";
                //        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";
                //        log::info!("Connecting to {} with token {}", url, token);

                //        let mut options = RoomOptions::default();
                //        options.adaptive_stream = false;
                //        options.dynacast = false;

                //        match Room::connect(&url, &token, options).await {
                //            Ok((room, mut events)) => {
                //                let room_sid = room.sid().await;
                //                log::info!("Connected to room {:#?}", room_sid);

                //                // Audio setup
                //                let options = AudioSourceOptions {
                //                    echo_cancellation: true,
                //                    noise_suppression: true,
                //                    auto_gain_control: true,
                //                    ..Default::default()
                //                };

                //                let audio_source = NativeAudioSource::new(
                //                    options,
                //                    44100,
                //                    1,
                //                    100,
                //                );

                //                let rtc_source = RtcAudioSource::Native(audio_source);
                //                let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
                //                let local_track = LocalTrack::Audio(local_audio_track);
                //
                //                match room.local_participant()
                //                    .publish_track(local_track, TrackPublishOptions {
                //                        source: TrackSource::Microphone,
                //                        ..Default::default()
                //                    })
                //                    .await
                //                {
                //                    Ok(track_pub) => {
                //                        log::info!("Published local audio track: {:#?}", track_pub);

                //                        // Create a channel for event communication
                //                        let (tx, mut rx) = tokio::sync::mpsc::channel(32);

                //                        // Spawn a separate task for event processing
                //                        let events_task = tokio::spawn(async move {
                //                            while let Some(event) = events.recv().await {
                //                                log::warn!("NEW EVENT: {:#?}", event);
                //                                if let Err(e) = tx.send(event).await {
                //                                    log::error!("Failed to send event: {:?}", e);
                //                                    break;
                //                                }
                //                            }
                //                        });

                //                        // Process events in the main runtime context
                //                        while let Some(event) = rx.recv().await {
                //                            if let Err(e) = livekit.handle_room_event(event).await {
                //                                log::error!("Error handling room event: {:?}", e);
                //                                ctx_clone.update_app(Event::LiveKit(LiveKitEvent::Error(e.to_string())));
                //                                break;
                //                            }
                //                        }

                //                        // Clean up the events task
                //                        events_task.abort();
                //                    }
                //                    Err(e) => {
                //                        log::error!("Failed to publish track: {:?}", e);
                //                        ctx_clone.update_app(Event::LiveKit(LiveKitEvent::Error(e.to_string())));
                //                    }
                //                }
                //            }
                //            Err(err) => {
                //                log::error!("Failed to connect: {:?}", err);
                //                ctx_clone.update_app(Event::LiveKit(LiveKitEvent::Error(err.to_string())));
                //            }
                //        }
                //    });
                //    Command::done()
                //}
                LiveKitEvent::JoinRoom => {
                    let livekit = caps.livekit.clone();
    
                    // Create channel for events
                    let (tx, mut rx) = mpsc::unbounded_channel();
                    
                    // Store sender in global state
                    *EVENT_SENDER.lock().unwrap() = Some(tx.clone());

                    // Create a single Command that handles both event processing and LiveKit connection
                    Command::new(move |ctx| async move {
                        // Spawn LiveKit connection handling in Tokio runtime
                        let livekit_handle = RUNTIME.spawn(async move {
                            let url = "http://192.168.20.20:7880";
                            let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";

                            log::info!("Connecting to {} with token {}", url, token);

                            let mut options = RoomOptions::default();
                            options.adaptive_stream = false;
                            options.dynacast = false;


                            match Room::connect(&url, &token, options).await {
                                Ok((room, mut events)) => {
                                    let room_sid = room.sid().await;
                                    log::info!("Connected to room {:#?}", room_sid);

                                    // Audio setup
                                    let options = AudioSourceOptions {
                                        echo_cancellation: true,
                                        noise_suppression: true,
                                        auto_gain_control: true,
                                        ..Default::default()
                                    };

                                    let audio_source = NativeAudioSource::new(
                                        options,
                                        44100,
                                        1,
                                        100,
                                    );

                                    let rtc_source = RtcAudioSource::Native(audio_source);
                                    let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
                                    let local_track = LocalTrack::Audio(local_audio_track);

                                    match room.local_participant()
                                        .publish_track(
                                            local_track,
                                            TrackPublishOptions {
                                                source: TrackSource::Microphone,
                                                ..Default::default()
                                            },
                                        ).await 
                                    {
                                        Ok(track_pub) => {
                                            log::info!("Published local audio track: {:#?}", track_pub);

                                            // Event loop
                                            while let Some(event) = events.recv().await {
                                                log::warn!("NEW EVENT: {:#?}", event);
                                                match livekit.handle_room_event(event).await {
                                                    Ok(app_event) => {
                                                        log::info!("EVENT STARTED: {:#?}", app_event);
                                                        // Send event through channel instead of direct update
                                                        if let Err(e) = tx.send(app_event) {
                                                            log::error!("Failed to send event through channel: {:?}", e);
                                                            break;
                                                        }
                                                        log::info!("EVENT SENT TO CHANNEL");
                                                    }
                                                    Err(e) => {
                                                        log::error!("Error handling room event: {:?}", e);
                                                        if let Err(e) = tx.send(Event::LiveKit(LiveKitEvent::Error(e.to_string()))) {
                                                            log::error!("Failed to send error event through channel: {:?}", e);
                                                        }
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("Failed to publish track: {:?}", e);
                                            if let Err(e) = tx.send(Event::LiveKit(LiveKitEvent::Error(e.to_string()))) {
                                                log::error!("Failed to send error event through channel: {:?}", e);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to connect to room: {:?}", e);
                                    if let Err(e) = tx.send(Event::LiveKit(LiveKitEvent::Error(e.to_string()))) {
                                        log::error!("Failed to send error event through channel: {:?}", e);
                                    }
                                }
                            }
                        });
                        
                                                // Process events from the channel while LiveKit connection is running
                                                tokio::select! {
                                                    _ = livekit_handle => {
                                                        log::info!("LiveKit connection ended");
                                                    }
                                                    _ = async {
                                                        while let Some(event) = rx.recv().await {
                                                            match event {
                                                                Event::Audio(audio_event @ AudioEvent::PlaybackAudio { .. }) => {
                                                                    // Handle audio event directly
                                                                    log::info!("Received audio event: {:?}", audio_event);
                                                                    ctx.send_event(Event::Audio(audio_event));
                                                                }
                                                                other_event => {
                                                                    // Handle other events
                                                                    ctx.send_event(other_event);
                                                                }
                                                            }
                                                        }
                                                    } => {
                                                        log::info!("Event processing ended");
                            }
                        }
                    })
                }

                    // let _ = Command::<Effect, Event>::done();
                    // render::render();

                // }
                //LiveKitEvent::JoinRoom => {
                //    // Clone the LiveKit capability before moving into async block
                //    let livekit = caps.livekit.clone();
                //
                //    // Create a channel for communication
                //    let (tx, mut rx) = mpsc::unbounded_channel();
                //
                //    // Store sender in global state
                //    *EVENT_SENDER.lock().unwrap() = Some(tx.clone());
                //
                //    // Create a single Command that spawns both tasks
                //    let _ = Command::new(move |ctx: crux_core::command::CommandContext<Effect, Event>| async move {
                //        // Spawn the event processing task
                //        let event_task = {
                //            let ctx = ctx.clone();
                //            tokio::spawn(async move {
                //                while let Some(event) = rx.recv().await {
                //                    ctx.send_event(event);
                //                }
                //            })
                //        };
                //
                //        // Spawn the LiveKit connection task
                //        let livekit_task = {
                //            let tx = tx.clone();
                //            RUNTIME.spawn(async move {
                //                let url = "http://192.168.20.20:7880";
                //                let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";
                //
                //                let mut options = RoomOptions::default();
                //                options.adaptive_stream = false;
                //                options.dynacast = false;
                //
                //                log::info!("Connecting to {} with token {}", url, token);
                //
                //                match Room::connect(&url, &token, options).await {
                //                    Ok((room, mut events)) => {
                //                        let room_sid = room.sid().await;
                //                        log::info!("Connected to room {:#?}", room_sid);
                //
                //                        // Audio setup
                //                        let options = AudioSourceOptions {
                //                            echo_cancellation: true,
                //                            noise_suppression: true,
                //                            auto_gain_control: true,
                //                            ..Default::default()
                //                        };
                //
                //                        let audio_source = NativeAudioSource::new(
                //                            options,
                //                            44100,
                //                            1,
                //                            100,
                //                        );
                //
                //                        let rtc_source = RtcAudioSource::Native(audio_source);
                //                        let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
                //                        let local_track = LocalTrack::Audio(local_audio_track);
                //
                //                        match room.local_participant()
                //                            .publish_track(
                //                                local_track,
                //                                TrackPublishOptions {
                //                                    source: TrackSource::Microphone,
                //                                    ..Default::default()
                //                                },
                //                            ).await
                //                        {
                //                            Ok(track_pub) => {
                //                                log::info!("Published local audio track: {:#?}", track_pub);
                //
                //                                // Event loop
                //                                while let Some(event) = events.recv().await {
                //                                    log::warn!("NEW EVENT: {:#?}", event);
                //                                    match livekit.handle_room_event(event).await {
                //                                        Ok(app_event) => {
                //                                            let _ = tx.send(app_event);
                //                                        }
                //                                        Err(e) => {
                //                                            let _ = tx.send(Event::LiveKit(
                //                                                LiveKitEvent::Error(e.to_string())
                //                                            ));
                //                                            break;
                //                                        }
                //                                    }
                //                                }
                //                            }
                //                            Err(e) => {
                //                                let _ = tx.send(Event::LiveKit(
                //                                    LiveKitEvent::Error(e.to_string())
                //                                ));
                //                            }
                //                        }
                //                    }
                //                    Err(e) => {
                //                        let _ = tx.send(Event::LiveKit(
                //                            LiveKitEvent::Error(e.to_string())
                //                        ));
                //                    }
                //                }
                //            })
                //        };
                //
                //        // Wait for either task to complete
                //        tokio::select! {
                //            _ = event_task => {},
                //            _ = livekit_task => {},
                //        }
                //    });

                //LiveKitEvent::JoinRoom => {
                //    // Clone the LiveKit capability before moving into async block
                //    let livekit = caps.livekit.clone();

                //    // Create a channel for communication
                //    let (tx, mut rx) = mpsc::unbounded_channel();

                //    // Store sender in global state
                //    *EVENT_SENDER.lock().unwrap() = Some(tx.clone());

                //    // Create a Command that will process events from the channel
                //    Command::new(move |ctx: crux_core::command::CommandContext<Effect, Event>| async move {
                //        while let Some(event) = rx.recv().await {
                //            // Use send_event instead of update_app
                //            ctx.send_event(event);
                //        }
                //    })
                //    .and(Command::new(move |_| async move {
                //        let url = "http://192.168.20.20:7880";
                //        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";


                //        let mut options = RoomOptions::default();
                //        options.adaptive_stream = false;
                //        options.dynacast = false;

                //        // Instead of returning the JoinHandle, we'll await it
                //        let handle = RUNTIME.spawn(async move {
                //            log::info!("Connecting to {} with token {}", url, token);

                //            match Room::connect(&url, &token, options).await {
                //                Ok((room, mut events)) => {
                //                    let room_sid = room.sid().await;
                //                    log::info!("Connected to room {:#?}", room_sid);

                //                    // Audio setup
                //                    let options = AudioSourceOptions {
                //                        echo_cancellation: true,
                //                        noise_suppression: true,
                //                        auto_gain_control: true,
                //                        ..Default::default()
                //                    };

                //                    let audio_source = NativeAudioSource::new(
                //                        options,
                //                        44100,
                //                        1,
                //                        100,
                //                    );

                //                    let rtc_source = RtcAudioSource::Native(audio_source);
                //                    let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
                //                    let local_track = LocalTrack::Audio(local_audio_track);

                //                    match room.local_participant()
                //                        .publish_track(
                //                            local_track,
                //                            TrackPublishOptions {
                //                                source: TrackSource::Microphone,
                //                                ..Default::default()
                //                            },
                //                        ).await
                //                    {
                //                        Ok(track_pub) => {
                //                            log::info!("Published local audio track: {:#?}", track_pub);

                //                            // Event loop
                //                            while let Some(event) = events.recv().await {
                //                                log::warn!("NEW EVENT: {:#?}", event);
                //                                match livekit.handle_room_event(event).await {
                //                                    Ok(app_event) => {
                //                                        if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                //                                            let _ = tx.send(app_event);
                //                                        }
                //                                    }
                //                                    Err(e) => {
                //                                        if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                //                                            let _ = tx.send(Event::LiveKit(
                //                                                LiveKitEvent::Error(e.to_string())
                //                                            ));
                //                                        }
                //                                        break;
                //                                    }
                //                                }
                //                            }
                //                        }
                //                        Err(e) => {
                //                            if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                //                                let _ = tx.send(Event::LiveKit(
                //                                    LiveKitEvent::Error(e.to_string())
                //                                ));
                //                            }
                //                        }
                //                    }
                //                }
                //                Err(e) => {
                //                    if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                //                        let _ = tx.send(Event::LiveKit(
                //                            LiveKitEvent::Error(e.to_string())
                //                        ));
                //                    }
                //                }
                //            }
                //        });

                //        // Await the handle to satisfy the Future<Output = ()> requirement
                //        let _ = handle.await;
                //    }));

                    // LiveKitEvent::JoinRoom => {
                    //     // Clone the LiveKit capability before moving into async block
                    //     let livekit = caps.livekit.clone();
                    //
                    //     // Create a channel for communication
                    //     let (tx, mut rx) = mpsc::unbounded_channel();
                    //
                    //     // Store sender in global state
                    //     *EVENT_SENDER.lock().unwrap() = Some(tx.clone());
                    //
                    //     // Create a Command that will process events from the channel
                    //     Command::new(move |ctx| async move {
                    //         while let Some(event) = rx.recv().await {
                    //             ctx.update_app(event);
                    //         }
                    //     })
                    //     .and(Command::new(move |_| async move {
                    //         let url = "http://192.168.20.20:7880";
                    //         let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";
                    //
                    //         RUNTIME.spawn(async move {
                    //             log::info!("Connecting to {} with token {}", url, token);
                    //
                    //             match Room::connect(&url, &token, options).await {
                    //                 Ok((room, mut events)) => {
                    //                     let room_sid = room.sid().await;
                    //                     log::info!("Connected to room {:#?}", room_sid);
                    //
                    //                     // Audio setup
                    //                     let options = AudioSourceOptions {
                    //                         echo_cancellation: true,
                    //                         noise_suppression: true,
                    //                         auto_gain_control: true,
                    //                         ..Default::default()
                    //                     };
                    //
                    //                     let audio_source = NativeAudioSource::new(
                    //                         options,
                    //                         44100,
                    //                         1,
                    //                         100,
                    //                     );
                    //
                    //                     let rtc_source = RtcAudioSource::Native(audio_source);
                    //                     let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
                    //                     let local_track = LocalTrack::Audio(local_audio_track);
                    //
                    //                     match room.local_participant()
                    //                         .publish_track(
                    //                             local_track,
                    //                             TrackPublishOptions {
                    //                                 source: TrackSource::Microphone,
                    //                                 ..Default::default()
                    //                             },
                    //                         ).await
                    //                     {
                    //                         Ok(track_pub) => {
                    //                             log::info!("Published local audio track: {:#?}", track_pub);
                    //
                    //                             // Event loop
                    //                             while let Some(event) = events.recv().await {
                    //                                 log::warn!("NEW EVENT: {:#?}", event);
                    //                                 match livekit.handle_room_event(event).await {
                    //                                     Ok(app_event) => {
                    //                                         if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                    //                                             let _ = tx.send(app_event);
                    //                                         }                                                    }
                    //                                     Err(e) => {
                    //                                         if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                    //                                             let _ = tx.send(Event::LiveKit(
                    //                                                 LiveKitEvent::Error(e.to_string())
                    //                                             ));
                    //                                         }
                    //                                         break;                                                    }
                    //                                 }
                    //                             }
                    //                         }
                    //                         Err(e) => {
                    //                             if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                    //                                 let _ = tx.send(Event::LiveKit(
                    //                                     LiveKitEvent::Error(e.to_string())
                    //                                 ));
                    //                             }
                    //                         }                                    }
                    //                 }
                    //                 Err(e) => {
                    //                     if let Some(tx) = &*EVENT_SENDER.lock().unwrap() {
                    //                         let _ = tx.send(Event::LiveKit(
                    //                             LiveKitEvent::Error(e.to_string())
                    //                         ));
                    //                     }
                    //                 }
                    //             }
                    //         })
                    //     }));

                    //caps.compose.spawn(|ctx| async move {
                    //    log::info!("Starting connection process...");

                    //    RUNTIME.spawn(async move {
                    //        let url = "http://192.168.20.20:7880";
                    //        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";

                    //        log::info!("Connecting to {} with token {}", url, token);

                    //        let mut options = RoomOptions::default();
                    //        options.adaptive_stream = false;
                    //        options.dynacast = false;

                    //        match Room::connect(&url, &token, options).await {
                    //            Ok((room, mut events)) => {
                    //                let room_sid = room.sid().await;
                    //                log::info!("Connected to room {:#?}", room_sid);

                    //                // Audio setup
                    //                let options = AudioSourceOptions {
                    //                    echo_cancellation: true,
                    //                    noise_suppression: true,
                    //                    auto_gain_control: true,
                    //                    ..Default::default()
                    //                };

                    //                let audio_source = NativeAudioSource::new(
                    //                    options,
                    //                    44100,
                    //                    1,
                    //                    100,
                    //                );

                    //                let rtc_source = RtcAudioSource::Native(audio_source);
                    //                let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
                    //                let local_track = LocalTrack::Audio(local_audio_track);

                    //                match room.local_participant()
                    //                    .publish_track(
                    //                        local_track,
                    //                        TrackPublishOptions {
                    //                            source: TrackSource::Microphone,
                    //                            ..Default::default()
                    //                        },
                    //                    ).await
                    //                {
                    //                    Ok(track_pub) => {
                    //                        log::info!("Published local audio track: {:#?}", track_pub);

                    //                        // Event loop
                    //                        while let Some(event) = events.recv().await {
                    //                            log::warn!("NEW EVENT: {:#?}", event);
                    //                            match livekit.handle_room_event(event).await {
                    //                                Ok(app_event) => {
                    //                                    log::info!("STARTED EVENT");
                    //                                    ctx.update_app(app_event);
                    //                                    log::info!("EVENT FINISHED");
                    //                                }
                    //                                Err(e) => {
                    //                                    log::error!("Error handling room event: {:?}", e);
                    //                                    ctx.update_app(Event::LiveKit(LiveKitEvent::Error(e.to_string())));
                    //                                    break;
                    //                                }
                    //                            }
                    //                        }
                    //                    }
                    //                    Err(e) => {
                    //                        log::error!("Failed to publish track: {:?}", e);
                    //                        ctx.update_app(Event::LiveKit(LiveKitEvent::Error(e.to_string())));
                    //                    }
                    //                }
                    //            }
                    //            Err(err) => {
                    //                log::error!("Failed to connect: {:?}", err);
                    //                ctx.update_app(Event::LiveKit(LiveKitEvent::Error(err.to_string())));
                    //            }
                    //        }
                    //    });
                    //});
                _ => todo!(),
            },
            // Event::LiveKit(livekit_event) => match livekit_event {
            //     LiveKitEvent::JoinRoom => {
            //         caps.compose.spawn(|ctx| async move {
            //             // Signal that we're attempting to connect
            //             // ctx.update_app(Event::LiveKit(LiveKitEvent::Connecting));
            //
            //             log::info!("Starting connection process...");
            //
            //             let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            //
            //             RUNTIME.spawn(async move {
            //                 let url = "http://192.168.20.20:7880";
            //                 let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3NDI3NDg1MjcsImlzcyI6ImRldmtleSIsIm5hbWUiOiJsb2NhbGsiLCJuYmYiOjE3Mzg0Mjg1MjcsInN1YiI6ImxvY2FsayIsInZpZGVvIjp7InJvb20iOiJ0ZXN0Iiwicm9vbUpvaW4iOnRydWV9fQ.2i_6v5LD2cjyC25fomVCCrlR_PFKFw8b6zbyqXVr-MU";
            //
            //                 log::info!("Connecting to {} with token {}", url, token);
            //
            //                 let mut options = RoomOptions::default();
            //                 options.adaptive_stream = false;
            //                 options.dynacast = false;
            //
            //                 let result = Room::connect(&url, &token, options).await;
            //                 log::info!("Connection attempt completed with result: {:#?}", result);
            //                 tx.send(result).await.expect("Failed to send result");
            //             });
            //
            //             log::info!("Awaiting connect_result...");
            //
            //             if let Some(result) = rx.recv().await {
            //                 match result {
            //                     Ok((room, mut events)) => {
            //                         log::info!("Successfully got room and events");
            //                         let room_sid = room.sid().await;
            //                         log::info!("Got room SID");
            //                         log::info!("Connected to room {:#?}", room_sid);
            //
            //                         // Create and publish audio track once before entering event loop
            //                         let options = AudioSourceOptions {
            //                             echo_cancellation: true,
            //                             noise_suppression: true,
            //                             auto_gain_control: true,
            //                             ..Default::default()
            //                         };
            //
            //                         let audio_source = NativeAudioSource::new(
            //                             options,
            //                             44100,  // sample rate
            //                             1,      // num channels (mono)
            //                             100,    // queue size in ms
            //                         );
            //
            //                         let rtc_source = livekit::webrtc::audio_source::RtcAudioSource::Native(audio_source);
            //                         let local_audio_track = LocalAudioTrack::create_audio_track("microphone", rtc_source);
            //                         let local_track = LocalTrack::Audio(local_audio_track);
            //
            //                         let track_pub = room.local_participant()
            //                             .publish_track(
            //                                 local_track,
            //                                 TrackPublishOptions {
            //                                     source: TrackSource::Microphone,
            //                                     ..Default::default()
            //                                 },
            //                             )
            //                             .await.unwrap();
            //
            //                         log::info!("Published local audio track: {:#?}", track_pub);
            //
            //                         // Now enter the event loop
            //                         while let Some(event) = events.recv().await {
            //                             if let Err(e) = handle_room_event(event).await {
            //                                 log::error!("Error handling room event: {:?}", e);
            //                                 ctx.update_app(Event::LiveKit(LiveKitEvent::Error(e.to_string())));
            //                                 break;
            //                             }
            //                         }
            //                     }
            //                     _ => {
            //                         log::error!("Failed to connect");
            //                         ctx.update_app(Event::LiveKit(LiveKitEvent::Error("errrrr".to_string())));
            //                     }
            //                 }
            //             }
            //         });
            //         Command::done()
            //     }
            //     _ => todo!(),
            // },
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
                                let _ = Command::<Effect, Event>::event(Event::Audio(
                                    AudioEvent::StartRecordingRequested,
                                ));
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
                    AudioEvent::PlaybackAudio {
                        track,
                        sample_rate,
                        num_channels,
                    } => {
                        log::info!("Starting audio playback...");
                        match caps.audio.playback_audio(track, sample_rate, num_channels) {
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
