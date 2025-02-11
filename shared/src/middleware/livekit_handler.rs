// middleware/livekit_middleware.rs
use crux_core::{App, Middleware, Request};
use livekit::{
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, RemoteTrack, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        prelude::{AudioSourceOptions, RtcAudioSource},
    },
    Room, RoomEvent, RoomOptions,
};
use std::{collections::VecDeque, sync::Arc};
use tokio::{
    runtime::Runtime,
    sync::{mpsc, Mutex},
};

use crate::{
    capabilities::livekit::{self as LiveKitCap, LiveKitError},
    events::audio::AudioEvent,
};

use crate::{
    app::Effect, capabilities::livekit::LiveKitOperation, events::livekit::LiveKitEvent, Event,
};

use super::LiveKitCommand;

pub struct LiveKitMiddleware<Core: Middleware> {
    core: Core,
    runtime: Arc<tokio::runtime::Runtime>,
    tx: mpsc::Sender<LiveKitCommand>,
    pending_events: Arc<Mutex<VecDeque<Event>>>,
}

impl<Core: Middleware> LiveKitMiddleware<Core> {
    pub fn new(core: Core) -> Self {
        let (tx, mut rx) = mpsc::channel(100);
        let runtime = Arc::new(Runtime::new().unwrap());
        let pending_events = Arc::new(Mutex::new(VecDeque::new()));

        // Setup LiveKit handler thread
        let pending_clone = pending_events.clone();
        let runtime_clone = runtime.clone();
        let runtime_spawn = runtime.clone(); // Create a separate clone for spawning
        std::thread::spawn(move || {
            runtime_clone.block_on(async move {
                while let Some(cmd) = rx.recv().await {
                    match cmd {
                        LiveKitCommand::JoinRoom { url, token } => {
                            let mut options = RoomOptions::default();
                            options.adaptive_stream = false;
                            options.dynacast = false;

                            match Room::connect(&url, &token, options).await {
                                Ok((room, mut events)) => {
                                    let room_sid = room.sid().await;
                                    log::info!("Connected to room {:#?}", room_sid);

                                    // Create audio source
                                    let audio_source = Arc::new(NativeAudioSource::new(
                                        AudioSourceOptions {
                                            echo_cancellation: true,
                                            noise_suppression: true,
                                            auto_gain_control: true,
                                            ..Default::default()
                                        },
                                        44100,
                                        1,
                                        100,
                                    ));

                                    // Queue start recording event
                                    // pending_clone.lock().await.push_back(Event::Audio(
                                    //     AudioEvent::StartRecordingRequested, // audio_source.clone()
                                    // ));

                                    log::info!("Sending mic...");

                                    // Setup local audio track
                                    let rtc_source =
                                        RtcAudioSource::Native(audio_source.as_ref().clone());
                                    let local_audio_track = LocalAudioTrack::create_audio_track(
                                        "microphone",
                                        rtc_source,
                                    );
                                    let local_track = LocalTrack::Audio(local_audio_track);

                                    // Publish track and handle room events
                                    match room
                                        .local_participant()
                                        .publish_track(
                                            local_track,
                                            TrackPublishOptions {
                                                source: TrackSource::Microphone,
                                                ..Default::default()
                                            },
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            // Spawn room event listener
                                            let events_pending = pending_clone.clone();
                                            runtime_spawn.spawn(async move {
                                                while let Some(room_event) = events.recv().await {
                                                    match Self::handle_room_event(room_event) {
                                                        Ok(app_event) => {
                                                            log::info!("Created app event: {:?}", app_event);
                                                            let mut guard = events_pending.lock().await;
                                                            guard.push_back(app_event);
                                                            log::warn!("GUARD {:#?}", guard);
                                                            log::info!("Pushed event to queue, size: {}", guard.len());
                                                        }
                                                        Err(e) => {
                                                            events_pending.lock().await.push_back(
                                                                Event::LiveKit(
                                                                    LiveKitEvent::Error(
                                                                        e.to_string(),
                                                                    ),
                                                                ),
                                                            );
                                                            break;
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        Err(e) => {
                                            let mut pending = pending_clone.lock().await;
                                            pending.push_back(Event::LiveKit(LiveKitEvent::Error(
                                                format!("Failed to publish track: {}", e),
                                            )));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let mut pending = pending_clone.lock().await;
                                    pending.push_back(Event::LiveKit(LiveKitEvent::Error(
                                        e.to_string(),
                                    )));
                                }
                            }
                        }
                        _ => todo!(),
                    }
                }
            });
        });

        Self {
            core,
            runtime,
            tx,
            pending_events,
        }
    }

    pub fn handle_room_event(event: RoomEvent) -> Result<Event, LiveKitError> {
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
            RoomEvent::TrackSubscribed { track, .. } => {
                if let RemoteTrack::Audio(audio_track) = track {
                    log::info!("Processing audio track through middleware");
                    let rtc_track = audio_track.rtc_track();
                    log::info!("Created RTC track for playback");
                    
                    let event = Event::Audio(AudioEvent::PlaybackAudio { track: rtc_track });
                    log::info!("Created PlaybackAudio event: {:?}", event);
                    
                    return Ok(event);
                } else {
                    return Ok(Event::Nothing);
                }
            }            // RoomEvent::TrackSubscribed {
            //     track,
            //     publication,
            //     participant,
            // } => {
            //     log::error!(
            //         "Subscribed to track: {} ({}) from {}",
            //         track.sid(),
            //         publication.sid(),
            //         participant.identity()
            //     );
            //
            //     if let RemoteTrack::Audio(audio_track) = track {
            //         log::info!("Processing audio track through middleware");
            //         let rtc_track = audio_track.rtc_track();
            //         log::info!("Created RTC track for playback");
            //         return Ok(Event::Audio(AudioEvent::PlaybackAudio { track: rtc_track }));
            //     }
            // }
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
        Ok(Event::Nothing)
    }
}

impl<Core: Middleware> Middleware for LiveKitMiddleware<Core>
where
    Core::App: crux_core::App<Event = Event, Effect = Effect>,
{
    type App = Core::App;

    // fn process_event(
    //     &self,
    //     event: <Self::App as crux_core::App>::Event,
    // ) -> impl Iterator<Item = <<Self as Middleware>::App as crux_core::App>::Effect> {
    //     let mut all_effects = Vec::new();
    //
    //     // First process any pending events
    //     let pending_events = self.runtime.block_on(async {
    //         let mut guard = self.pending_events.lock().await;
    //         log::info!("Processing pending events, count: {}", guard.len());
    //         guard.drain(..).collect::<Vec<_>>()
    //     });
    //
    //     // Process each pending event
    //     for pending_event in pending_events {
    //         log::info!("Processing pending event: {:?}", pending_event);
    //         match &pending_event {
    //             Event::Audio(AudioEvent::PlaybackAudio { .. }) => {
    //                 log::info!("Found PlaybackAudio event, processing...");
    //             }
    //             _ => {}
    //         }
    //         all_effects.extend(self.core.process_event(pending_event));
    //     }
    //
    //     // Then process the current event
    //     match event {
    //         Event::LiveKit(LiveKitEvent::JoinRoom) => {
    //             log::info!("Handling JoinRoom effect");
    //             let _ = self.tx.try_send(LiveKitCommand::JoinRoom(
    //                 "http://192.168.20.20:7880".to_string(),
    //                 "your-token".to_string(),
    //             ));
    //         }
    //         _ => {
    //             all_effects.extend(self.core.process_event(event));
    //         }
    //     }
    //
    //     // Process effects again to catch any new ones
    //     all_effects.extend(self.process_effects());
    //
    //     all_effects.into_iter()
    // }

    fn process_event(
        &self,
        event: <Self::App as crux_core::App>::Event,
    ) -> impl Iterator<Item = <<Self as Middleware>::App as crux_core::App>::Effect> {
        log::info!("PROCESSING EVENTS");

        // Get pending events
        let pending_events: Vec<Event> = self.runtime.block_on(async {
            let mut guard = self.pending_events.lock().await;
            // Add debug log here
            log::info!("Draining pending events: {:?}", guard);
            guard.drain(..).collect()
        });

        log::info!("Processing {} pending events", pending_events.len());

        // Collect pending events effects into a Vec
        let pending_effects: Vec<_> = pending_events
            .into_iter()
            .flat_map(|pending_event| {
                log::info!(
                    "Processing pending event in middleware: {:?}",
                    pending_event
                );
                // Collect into Vec so we can both log and use the effects
                let effects: Vec<_> = self.core.process_event(pending_event).collect();
                log::info!("Effects from pending event: {:?}", effects);
                effects
            })
            .collect();

        // Process current event
        let current_effects = self.core.process_event(event);

        // Combine and handle all effects with logging
        pending_effects
            .into_iter()
            .chain(current_effects)
            .flat_map(|effect| {
                log::info!("Processing effect in middleware: {:?}", effect);
                match effect {
                    Effect::LiveKit(Request {
                        operation: LiveKitOperation::JoinRoom(url, token),
                        ..
                    }) => {
                        log::info!("Handling JoinRoom effect");
                        let _ = self.tx.try_send(LiveKitCommand::JoinRoom { url, token });
                        Vec::new()
                    }
                    other_effect => vec![other_effect],
                }
            })
    }

    //     let pending_events: Vec<Event> = self.runtime.block_on(async {
    //         // Get lock and drain events
    //         let mut guard = self.pending_events.lock().await;
    //         guard.drain(..).collect()
    //     });
    //
    //     log::info!("Processing {} pending events", pending_events.len());
    //
    //     // Create iterator from pending events
    //     let pending_effects = pending_events.into_iter().flat_map(|pending_event| {
    //         log::info!("Processing pending event: {:?}", pending_event);
    //         self.core.process_event(pending_event)
    //     });
    //
    //     // Process current event
    //     let current_effects = self.core.process_event(event);
    //
    //     // Combine and handle all effects
    //     pending_effects
    //         .chain(current_effects)
    //         .flat_map(|effect| match effect {
    //             Effect::LiveKit(Request {
    //                 operation: LiveKitOperation::JoinRoom(url, token),
    //                 ..
    //             }) => {
    //                 log::info!("Handling JoinRoom effect");
    //                 let _ = self.tx.try_send(LiveKitCommand::JoinRoom { url, token });
    //                 Vec::new()
    //             }
    //             other_effect => vec![other_effect],
    //         })
    // }

    // fn process_effects(&self) -> impl Iterator<Item = <Self::App as App>::Effect> {
    //     self.core.process_effects()
    // }
    fn process_effects(&self) -> impl Iterator<Item = <<Self as Middleware>::App as crux_core::App>::Effect> {
        log::info!("WE ARE HERE");
        let pending_events: Vec<Event> = self.runtime.block_on(async {
            let mut guard = self.pending_events.lock().await;
            guard.drain(..).collect()
        });
        log::info!("WE ARE NOW HERE");

        let mut all_effects = Vec::new();
        for pending_event in pending_events {
            log::info!("Processing pending event during effects: {:?}", pending_event);
            all_effects.extend(self.core.process_event(pending_event));
        }
        log::info!("WE ARE NOW HERE3");

        all_effects.into_iter()
    }

    fn view(&self) -> <Self::App as App>::ViewModel {
        self.core.view()
    }
}
