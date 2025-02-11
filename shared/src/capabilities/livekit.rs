use livekit::{track::RemoteTrack, webrtc::audio_stream::native::NativeAudioStream, RoomEvent};

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

use crate::{events::audio::AudioEvent, Event as AppEvent};
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
    Event: 'static + Send,
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

                if let RemoteTrack::Audio(audio_track) = track {
                    log::info!("Processing audio track through middleware");
                    let rtc_track = audio_track.rtc_track();
                    log::info!("Created RTC track for playback");
                    return Ok(AppEvent::Audio(AudioEvent::PlaybackAudio {
                        track: rtc_track,
                    }));
                }
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
