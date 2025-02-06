// use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
// use crux_core::capability::{CapabilityContext, Operation};
// use serde::{Deserialize, Serialize};
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// pub enum AudioOperation {
//     RequestPermission,
//     InitializeRecording {
//         sample_rate: u32,
//         channels: u16,
//     },
//     StartRecording,
//     StopRecording,
//     PauseRecording,
//     GetAudioData,
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct AudioConfig {
//     pub sample_rate: u32,
//     pub channels: u16,
//     pub buffer_size: usize,
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct AudioData {
//     pub samples: Vec<f32>,
//     pub config: AudioConfig,
// }
//
// impl Operation for AudioOperation {
//     type Output = Result<AudioData, String>;
// }
//
// // #[derive(crux_core::macros::Capability)]
// // pub struct Audio<Event> {
// //     context: CapabilityContext<AudioOperation, Event>,
// // }
//
// #[derive(crux_core::macros::Capability)]
// pub struct Audio<Event> {
//     context: CapabilityContext<AudioOperation, Event>,
//     stream: Option<cpal::Stream>,
//     recording_buffer: Vec<f32>,
//     // sender: Option<mpsc::Sender<Vec<f32>>>,
// }
//
// impl<Ev> Audio<Ev>
// where
//     Ev: 'static,
// {
//     // pub fn new(context: CapabilityContext<AudioOperation, Ev>) -> Self {
//     //     Self { context }
//     // }
//
//     pub fn new(context: CapabilityContext<AudioOperation, Ev>) -> Self {
//         Self {
//             context,
//             stream: None,
//             recording_buffer: Vec::new(),
//             // recording_buffer: Arc::new(Mutex::new(Vec::new())),
//             // sender: None,
//             // stream: None,
//         }
//     }
//
//     pub fn request_permission(&self, on_complete: Ev)
//     where
//         Ev: Send,
//     {
//         self.context.spawn({
//             let context = self.context.clone();
//
//             async move {
//                 let result = context
//                     .request_from_shell(AudioOperation::RequestPermission)
//                     .await;
//
//                 if result.is_ok() {
//                     context.update_app(on_complete);
//                 }
//             }
//         })
//     }
//
//     pub fn initialize_recording(
//         &self,
//         sample_rate: u32,
//         channels: u16,
//         on_complete: impl Fn(Result<AudioConfig, String>) -> Ev + Send + 'static,
//     ) where
//         Ev: Send,
//     {
//         self.context.spawn({
//             let context = self.context.clone();
//
//             async move {
//                 let result = context
//                     .request_from_shell(AudioOperation::InitializeRecording {
//                         sample_rate,
//                         channels,
//                     })
//                     .await;
//
//                 let event = on_complete(result.map(|audio_data| AudioConfig {
//                     sample_rate: audio_data.config.sample_rate,
//                     channels: audio_data.config.channels,
//                     buffer_size: audio_data.config.buffer_size,
//                 }));
//                 context.update_app(event);
//             }
//         })
//     }
//
//     pub fn start_recording(&self, on_data: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     where
//         Ev: Send,
//     {
//         self.context.spawn({
//             let context = self.context.clone();
//             let (sender, receiver) = mpsc::channel();
//
//             async move {
//                 let host = cpal::default_host();
//                 let device = host.default_input_device().ok_or("No input device available").unwrap();
//                 let config = device.default_input_config().map_err(|e| e.to_string()).unwrap();
//
//                 let stream_config: cpal::StreamConfig = config.into();
//                 let stream = device.build_input_stream(
//                     &stream_config,
//                     move |data: &[f32], _: &cpal::InputCallbackInfo| {
//                         let audio_data = data.to_vec();
//                         sender.send(audio_data).unwrap();
//                     },
//                     move |err| {
//                         eprintln!("Audio stream error: {:?}", err);
//                     },
//                     None, // Optional buffer duration
//                 ).map_err(|e| e.to_string()).unwrap();
//
//                 stream.play().map_err(|e| e.to_string()).unwrap();
//
//                 while let Ok(audio_data) = receiver.recv() {
//                     let event = on_data(Ok(AudioData {
//                         samples: audio_data,
//                         config: AudioConfig {
//                             sample_rate: stream_config.sample_rate.0,
//                             channels: stream_config.channels,
//                             buffer_size: 1024,
//                         },
//                     }));
//                     context.update_app(event);
//                 }
//             }
//         })
//     }
//
//     // pub fn stop_recording(&self, on_complete: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     // where
//     //     Ev: Send,
//     // {
//     //     self.context.spawn({
//     //         let context = self.context.clone();
//     //
//     //         async move {
//     //             if let Some(stream) = &self.stream {
//     //                 stream.pause().map_err(|e| e.to_string()).unwrap();
//     //             }
//     //             let event = on_complete(Ok(AudioData {
//     //                 samples: Vec::new(),
//     //                 config: AudioConfig {
//     //                     sample_rate: 0,
//     //                     channels: 0,
//     //                     buffer_size: 0,
//     //                 },
//     //             }));
//     //             context.update_app(event);
//     //         }
//     //     })
//     // }
//     //
//     // pub fn pause_recording(&self, on_complete: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     // where
//     //     Ev: Send,
//     // {
//     //     self.context.spawn({
//     //         let context = self.context.clone();
//     //
//     //         async move {
//     //             if let Some(stream) = &self.stream {
//     //                 stream.pause().map_err(|e| e.to_string()).unwrap();
//     //             }
//     //             let event = on_complete(Ok(AudioData {
//     //                 samples: Vec::new(),
//     //                 config: AudioConfig {
//     //                     sample_rate: 0,
//     //                     channels: 0,
//     //                     buffer_size: 0,
//     //                 },
//     //             }));
//     //             context.update_app(event);
//     //         }
//     //     })
//     // }
//
//     // pub fn start_recording(&self, on_data: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     // where
//     //     Ev: Send,
//     // {
//     //     self.context.spawn({
//     //         let context = self.context.clone();
//     //
//     //         async move {
//     //             let result = context
//     //                 .request_from_shell(AudioOperation::StartRecording)
//     //                 .await;
//     //
//     //             let event = on_data(result);
//     //             context.update_app(event);
//     //         }
//     //     })
//     // }
//     //
//     // pub fn stop_recording(&self, on_complete: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     // where
//     //     Ev: Send,
//     // {
//     //     self.context.spawn({
//     //         let context = self.context.clone();
//     //
//     //         async move {
//     //             let result = context
//     //                 .request_from_shell(AudioOperation::StopRecording)
//     //                 .await;
//     //
//     //             let event = on_complete(result);
//     //             context.update_app(event);
//     //         }
//     //     })
//     // }
//     //
//     // pub fn pause_recording(&self, on_complete: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     // where
//     //     Ev: Send,
//     // {
//     //     self.context.spawn({
//     //         let context = self.context.clone();
//     //
//     //         async move {
//     //             let result = context
//     //                 .request_from_shell(AudioOperation::PauseRecording)
//     //                 .await;
//     //
//     //             let event = on_complete(result);
//     //             context.update_app(event);
//     //         }
//     //     })
//     // }
//
//     pub fn get_audio_data(&self, on_data: impl Fn(Result<AudioData, String>) -> Ev + Send + 'static)
//     where
//         Ev: Send,
//     {
//         self.context.spawn({
//             let context = self.context.clone();
//
//             async move {
//                 let result = context
//                     .request_from_shell(AudioOperation::GetAudioData)
//                     .await;
//
//                 let event = on_data(result);
//                 context.update_app(event);
//             }
//         })
//     }
// }

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crux_core::capability::{CapabilityContext, Operation};
use futures::StreamExt;
use livekit::webrtc::{
    audio_source::native::NativeAudioSource, audio_stream::native::NativeAudioStream,
    prelude::RtcAudioTrack,
};
use livekit::webrtc::audio_frame::AudioFrame as LiveKitAudioFrame;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

// Audio Operation types
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AudioOperation {
    ToggleRecording,
    PlaybackAudio {
        sample_rate: u32,
        channels: u16,
        samples: Vec<f32>, // Replace Box<dyn AudioStreamTrait> with actual audio data
    },
}

// Define trait for audio playback without serialization requirements
pub trait AudioStreamTrait: Send {
    fn next(&mut self) -> Box<dyn std::future::Future<Output = Option<AudioFrame>> + Send + Unpin>;
}

// Concrete implementation that can be serialized
#[derive(Debug)]
pub struct AudioStream {
    stream: NativeAudioStream,
}
// pub struct AudioStream {
//     samples: Vec<f32>,
//     position: usize,
//     sample_rate: u32,
//     channels: u16,
// }

// impl AudioStream {
//     pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
//         Self {
//             samples,
//             position: 0,
//             sample_rate,
//             channels,
//         }
//     }
// }

#[derive(Debug)]
pub struct AudioFrame {
    pub data: Vec<i16>,
    pub samples_per_channel: u32,
    pub num_channels: u16,
}

impl Operation for AudioOperation {
    type Output = ();
    // type Output = Result<AudioResponse, AudioError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AudioError {
    DeviceNotFound,
    RecordingFailed(String),
    UnsupportedFormat,
    InvalidState,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::DeviceNotFound => write!(f, "Audio device not found"),
            AudioError::RecordingFailed(msg) => write!(f, "Recording failed: {}", msg),
            AudioError::InvalidState => write!(f, "Invalid audio state"),
            AudioError::UnsupportedFormat => write!(f, "Unsupported audio format"),
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RecordingState {
    #[default]
    Idle,
    Recording,
    Finished,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AudioResponse {
    StateChanged(RecordingState),
    Data(AudioData),
}

// Audio Events for ViewModel
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AudioEvent {
    RecordingStateChanged(RecordingState),
    RecordingComplete(AudioData),
    RecordingError(String),
}

// Shared state for recording
struct AudioState {
    stream: Option<cpal::Stream>,
    samples: Vec<f32>,
    config: Option<cpal::StreamConfig>,
    state: RecordingState,
}

// Implement Send and Sync for AudioState
unsafe impl Send for AudioState {}
unsafe impl Sync for AudioState {}

#[derive(crux_core::macros::Capability, Clone)]
pub struct Audio<Event> {
    context: CapabilityContext<AudioOperation, Event>,
    recording_state: Arc<Mutex<AudioState>>,
}

#[derive(Debug)]
pub struct UninitializedContext;

impl<Event> Default for Audio<Event> {
    fn default() -> Self {
        Self {
            context: unimplemented!("Audio capability must be properly initialized with new()"),
            recording_state: Arc::new(Mutex::new(AudioState {
                stream: None,
                samples: Vec::new(),
                config: None,
                state: RecordingState::Idle,
            })),
        }
    }
}

impl<Event> Audio<Event>
where
    Event: 'static + Send,
{
    // pub fn playback_audio(
    //     &self,
    //     track: RtcAudioTrack,
    // ) -> Result<(), AudioError> {
    //     // Get default host and output device
    //     let host = cpal::default_host();
    //     let device = host
    //         .default_output_device()
    //         .ok_or(AudioError::DeviceNotFound)?;
    //
    //     log::info!("Using output device: {:?}", device.name());
    //
    //     // Get supported config
    //     let supported_config = device
    //         .default_output_config()
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     log::info!("Default supported config: {:?}", supported_config);
    //
    //     // Create audio stream with device's native config
    //     let config = supported_config.config();
    //     let sample_rate = config.sample_rate.0;
    //     let channels = config.channels;
    //
    //     log::info!(
    //         "Setting up audio playback with sample_rate: {}, channels: {}",
    //         sample_rate,
    //         channels
    //     );
    //
    //     // Create circular buffer for audio samples
    //     let buffer_size = 1024 * 16; // Larger buffer for smoother playback
    //     let samples = Arc::new(std::sync::Mutex::new(Vec::with_capacity(buffer_size)));
    //     let read_position = Arc::new(AtomicUsize::new(0));
    //
    //     // Spawn audio frame collection task
    //     let samples_producer = samples.clone();
    //     let mut audio_stream = NativeAudioStream::new(track, sample_rate as i32, channels as i32);
    //
    //     std::thread::spawn(move || {
    //         let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    //         rt.block_on(async {
    //             while let Some(frame) = audio_stream.next().await {
    //                 let mut buffer = samples_producer.lock().unwrap();
    //
    //                 // Convert i16 samples to f32 and add to buffer
    //                 let new_samples: Vec<f32> = frame.data.iter()
    //                     .map(|&s| (s as f32) / (i16::MAX as f32))
    //                     .collect();
    //
    //                 buffer.extend(new_samples);
    //
    //                 // Keep buffer size manageable by removing oldest samples
    //                 let current_len = buffer.len();
    //                 if current_len > buffer_size {
    //                     buffer.drain(0..(current_len - buffer_size));
    //                 }
    //             }
    //         });
    //     });
    //
    //     // Build output stream
    //     let samples_consumer = samples.clone();
    //     let pos = read_position.clone();
    //
    //     let stream = device.build_output_stream(
    //         &config,
    //         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //             let buffer = samples_consumer.lock().unwrap();
    //             let current_pos = pos.load(Ordering::Relaxed);
    //
    //             for (i, sample) in data.iter_mut().enumerate() {
    //                 let buffer_pos = (current_pos + i) % buffer.len();
    //                 *sample = if buffer_pos < buffer.len() {
    //                     buffer[buffer_pos]
    //                 } else {
    //                     0.0
    //                 };
    //             }
    //
    //             pos.store((current_pos + data.len()) % buffer.len(), Ordering::Relaxed);
    //         },
    //         |err| log::error!("Audio output error: {}", err),
    //         None,
    //     )
    //     .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     // Start playback
    //     stream.play().map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //     log::info!("Audio playback started successfully");
    //
    //     // Keep stream alive
    //     std::mem::forget(stream);
    //
    //     Ok(())
    // }
    // pub fn playback_audio(
    //     &self,
    //     track: RtcAudioTrack,
    // ) -> Result<(), AudioError> {
    //     // Get default host and output device
    //     let host = cpal::default_host();
    //     let device = host
    //         .default_output_device()
    //         .ok_or(AudioError::DeviceNotFound)?;
    //
    //     log::info!("Using output device: {:?}", device.name());
    //
    //     // Get the default output configuration
    //     let supported_config = device
    //         .default_output_config()
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     log::info!("Default supported config: {:?}", supported_config);
    //
    //     // Set up audio config
    //     let config = supported_config.config();
    //     let sample_rate = config.sample_rate.0;
    //     let channels = config.channels;
    //
    //     log::info!(
    //         "Setting up audio playback with sample_rate: {}, channels: {}",
    //         sample_rate,
    //         channels
    //     );
    //
    //     // **Use a thread-safe buffer**
    //     let buffer_size = 1024 * 16; // Ensure buffer size is large enough
    //     let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
    //
    //     // Flag to check if the producer is still sending data
    //     let is_running = Arc::new(AtomicBool::new(true));
    //
    //     // Spawn async task to pull audio samples
    //     let buffer_producer = Arc::clone(&buffer);
    //     let running_producer = Arc::clone(&is_running);
    //     let mut audio_stream = NativeAudioStream::new(track, sample_rate as i32, channels as i32);
    //
    //     thread::spawn(move || {
    //         // Create a Tokio runtime since we're outside of an async context
    //         let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
    //         rt.block_on(async {
    //             while let Some(frame) = audio_stream.next().await {
    //                 let mut buffer = buffer_producer.lock().unwrap();
    //
    //                 // Convert i16 samples to f32 and add to buffer
    //                 let new_samples: Vec<f32> = frame.data.iter()
    //                     .map(|&s| (s as f32) / (i16::MAX as f32)) // Normalize i16 -> f32
    //                     .collect();
    //
    //                 buffer.extend(new_samples);
    //
    //                 // Prevent buffer from growing indefinitely
    //                 if let Some(extra) = buffer.len().checked_sub(buffer_size) {
    //                     buffer.drain(0..extra);
    //                 }
    //             }
    //
    //             // Notify that producer is done
    //             running_producer.store(false, Ordering::Relaxed);
    //         });
    //     });
    //
    //     // **Build output stream**
    //     let buffer_consumer = Arc::clone(&buffer);
    //     let running_consumer = Arc::clone(&is_running);
    //
    //     let stream = device.build_output_stream(
    //         &config,
    //         move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //             let mut buffer = buffer_consumer.lock().unwrap();
    //
    //             for sample in output.iter_mut() {
    //                 if let Some(audio_sample) = buffer.pop_front() {
    //                     *sample = audio_sample;
    //                 } else if running_consumer.load(Ordering::Relaxed) {
    //                     // Buffer underrun: fill with silence until we get data
    //                     *sample = 0.0;
    //                 } else {
    //                     // We are done streaming, just stop sending samples
    //                     *sample = 0.0;
    //                 }
    //             }
    //         },
    //         |err| log::error!("Audio output error: {}", err),
    //         None,
    //     )
    //     .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     // Start playback
    //     stream.play().map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //     log::info!("Audio playback started successfully");
    //
    //     // Keep playback stream alive
    //     std::mem::forget(stream);
    //
    //     Ok(())
    // }
    pub fn playback_audio(&self, track: RtcAudioTrack) -> Result<(), AudioError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::DeviceNotFound)?;

        log::info!("Using output device: {:?}", device.name());

        let supported_config = device
            .default_output_config()
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;

        log::info!("Default supported config: {:?}", supported_config);

        let config = supported_config.config();
        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        log::info!(
            "Setting up audio playback - Sample Rate: {}, Channels: {}",
            sample_rate,
            channels
        );

        // ------------------------------
        // 🌀 Circular buffer for audio storage
        // ------------------------------
        let buffer_size = sample_rate as usize * channels as usize * 5; // 5 seconds buffer capacity
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(buffer_size)));
        let is_running = Arc::new(AtomicBool::new(true));
        let playback_started = Arc::new(AtomicBool::new(false));

        // Clone references for async producer thread
        let buffer_producer = Arc::clone(&buffer);
        let running_producer = Arc::clone(&is_running);
        let playback_started_prod = Arc::clone(&playback_started);
        let mut audio_stream = NativeAudioStream::new(track, sample_rate as i32, channels as i32);

        // ------------------------------
        // 🎤 LiveKit Audio Producer Thread - Adds Audio to Buffer
        // ------------------------------
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
            rt.block_on(async {
                while let Some(frame) = audio_stream.next().await {
                    let mut buffer = buffer_producer.lock().unwrap();

                    // Convert i16 samples to f32
                    let new_samples: Vec<f32> = frame
                        .data
                        .iter()
                        .map(|&s| (s as f32) / (i16::MAX as f32))
                        .collect();

                    buffer.extend(new_samples);

                    // ✅ Avoid trimming data before playback starts
                    if playback_started_prod.load(Ordering::Relaxed) {
                        if buffer.len() > buffer_size {
                            let extra_samples = buffer.len() - buffer_size;
                            buffer.drain(0..extra_samples);
                        }
                    }
                }

                log::info!("LiveKit audio stream ended.");
                running_producer.store(false, Ordering::Relaxed); // Mark producer as finished
            });
        });

        // ------------------------------
        // ⏳ Wait for Buffer to Fill Before Starting Playback
        // ------------------------------
        let buffer_consumer = Arc::clone(&buffer);
        let running_consumer = Arc::clone(&is_running);
        let playback_started_cons = Arc::clone(&playback_started);

        while buffer_consumer.lock().unwrap().len()
            < (sample_rate as usize) * (channels as usize) / 2
        {
            log::info!("Waiting for buffer...");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        log::info!("Audio buffer filled, starting playback!");
        playback_started_cons.store(true, Ordering::Relaxed); // Mark playback started

        let stream = device
            .build_output_stream(
                &config,
                move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = buffer_consumer.lock().unwrap();
                    for sample in output.iter_mut() {
                        if let Some(audio_sample) = buffer.pop_front() {
                            *sample = audio_sample;
                        } else {
                            // Handle end of stream gracefully
                            if running_consumer.load(Ordering::Relaxed) {
                                *sample = 0.0; // Buffer underrun: Fill with silence while waiting
                            } else {
                                *sample = 0.0; // Stop playback when stream truly ends
                            }
                        }
                    }
                },
                |err| log::error!("Audio output error: {}", err),
                None,
            )
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
        log::info!("Audio playback started!");

        // 🛑 **Ensure the stream stays alive until all samples play out**
        while is_running.load(Ordering::Relaxed) || !buffer.lock().unwrap().is_empty() {
            std::thread::sleep(Duration::from_millis(100));
        }

        log::info!("Audio playback finished");

        Ok(())
    }

    pub async fn capture_mic_audio(&self, audio_source: Arc<NativeAudioSource>) {
        log::info!("CAPTURING AUDIO....");
        let host = cpal::default_host();
        let device = host.default_input_device().expect("No input device found");
        log::info!("Using input device: {:?}", device.name());
    
        let config = device.default_input_config().expect("Failed to get default input config");
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();
        log::info!("Microphone config - Sample Rate: {}, Channels: {}", sample_rate, channels);
    
        let err_fn = |err| log::error!("Audio capture error: {}", err);
    
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[i16], _| {
                let audio_data: Vec<i16> = data.to_vec(); // Clone data to ensure ownership
                let frame = LiveKitAudioFrame {  
                    num_channels: channels as u32,
                    samples_per_channel: (audio_data.len() / channels as usize) as u32, 
                    data: audio_data.into(), // Transfer ownership
                    sample_rate, // ✅ Added missing field
                };
        
                let audio_source = audio_source.clone();
                tokio::spawn(async move {
                    if let Err(e) = audio_source.capture_frame(&frame).await {
                        log::error!("Failed to push mic audio frame: {:?}", e);
                    }
                });
            },
            err_fn,
            None,
        ).expect("Failed to build input stream");

        log::info!("Starting microphone capture...");
        stream.play().expect("Failed to start audio stream");
    
        std::thread::sleep(std::time::Duration::from_secs(600));  // Keep it running for a long time
    }

    // pub fn playback_audio(
    //     &self,
    //     track: RtcAudioTrack,
    //     sample_rate: i32,
    //     num_channels: i32,
    // ) -> Result<(), AudioError> {
    //     log::info!(
    //         "Setting up audio playback with sample_rate: {}, channels: {}",
    //         sample_rate,
    //         num_channels
    //     );
    //
    //     let mut audio_stream = NativeAudioStream::new(track, sample_rate, num_channels);
    //
    //     let host = cpal::default_host();
    //     let device = host
    //         .default_output_device()
    //         .ok_or(AudioError::DeviceNotFound)?;
    //
    //     log::info!("Using output device: {:?}", device.name());
    //
    //     let config = cpal::StreamConfig {
    //         channels: num_channels as u16,
    //         sample_rate: cpal::SampleRate(sample_rate as u32),
    //         buffer_size: cpal::BufferSize::Default,
    //     };
    //
    //     let output_stream = device
    //         .build_output_stream(
    //             &config,
    //             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //                 match futures::executor::block_on(audio_stream.next()) {
    //                     Some(frame) => {
    //                         log::debug!("Received audio frame with {} samples", frame.data.len());
    //                         // ... process frame ...
    //                     }
    //                     None => {
    //                         log::debug!("No audio frame available");
    //                         data.fill(0.0);
    //                     }
    //                 }
    //
    //                 // Use a blocking executor to get the next frame
    //                 if let Some(frame) = futures::executor::block_on(audio_stream.next()) {
    //                     let len = frame.samples_per_channel as usize * frame.num_channels as usize;
    //                     let len = len.min(data.len());
    //
    //                     // Convert samples to f32 and apply proper scaling
    //                     for i in 0..len {
    //                         if i < frame.data.len() {
    //                             let sample = frame.data[i] as f32;
    //                             let normalized = (sample / i16::MAX as f32) * 1.5; // Amplification factor of 1.5
    //                             data[i] = normalized.clamp(-1.0, 1.0);
    //                         }
    //                     }
    //
    //                     // Fill remaining buffer with silence if needed
    //                     if len < data.len() {
    //                         data[len..].fill(0.0);
    //                     }
    //                 } else {
    //                     // If no frame is available, output silence
    //                     data.fill(0.0);
    //                 }
    //             },
    //             |err| log::error!("Audio output error: {}", err),
    //             None,
    //         )
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     output_stream
    //         .play()
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //     log::info!("Audio playback started successfully");
    //
    //     // Keep the stream alive
    //     std::mem::forget(output_stream);
    //
    //     Ok(())
    // }
    // pub fn playback_audio(
    //     &self,
    //     track: RtcAudioTrack,
    //     sample_rate: i32,
    //     num_channels: i32,
    // ) -> Result<(), AudioError> {
    //     log::info!("Setting up audio playback...");
    //
    //     let mut audio_stream = NativeAudioStream::new(track, sample_rate, num_channels);
    //
    //     let host = cpal::default_host();
    //     let device = host
    //         .default_output_device()
    //         .ok_or(AudioError::DeviceNotFound)?;
    //
    //     log::info!("Using output device: {:?}", device.name());
    //
    //     let config = cpal::StreamConfig {
    //         channels: num_channels as u16,
    //         sample_rate: cpal::SampleRate(sample_rate as u32),
    //         buffer_size: cpal::BufferSize::Default,
    //     };
    //
    //     let output_stream = device
    //         .build_output_stream(
    //             &config,
    //             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //                 if let Some(frame) = futures::executor::block_on(audio_stream.next()) {
    //                     let len = frame.samples_per_channel as usize * frame.num_channels as usize;
    //                     let len = len.min(data.len());
    //
    //                     // Convert samples to f32 and apply proper scaling
    //                     for i in 0..len {
    //                         if i < frame.data.len() {
    //                             let sample = frame.data[i] as f32;
    //                             let normalized = (sample / i16::MAX as f32) * 1.5;
    //                             data[i] = normalized.clamp(-1.0, 1.0);
    //                         }
    //                     }
    //
    //                     // Fill remaining buffer with silence if needed
    //                     if len < data.len() {
    //                         data[len..].fill(0.0);
    //                     }
    //                 } else {
    //                     data.fill(0.0);
    //                 }
    //             },
    //             |err| log::error!("Audio output error: {}", err),
    //             None,
    //         )
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     output_stream
    //         .play()
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //     log::info!("Audio playback started successfully");
    //
    //     // Keep the stream alive
    //     std::mem::forget(output_stream);
    //
    //     Ok(())
    // }

    // pub fn playback_audio(
    //     &self,
    //     samples: Vec<f32>,
    //     sample_rate: u32,
    //     channels: u16,
    // ) -> Result<(), AudioError> {
    //     let stream = AudioStream::new(samples, sample_rate, channels);
    //     log::info!("Setting up audio playback...");
    //
    //     let host = cpal::default_host();
    //     let device = host
    //         .default_output_device()
    //         .ok_or(AudioError::DeviceNotFound)?;
    //
    //     log::info!("Using output device: {:?}", device.name());
    //
    //     let config = cpal::StreamConfig {
    //         channels,
    //         sample_rate: cpal::SampleRate(sample_rate),
    //         buffer_size: cpal::BufferSize::Default,
    //     };
    //
    //     let mut audio_stream = stream;
    //
    //     let samples = audio_stream.samples.clone();
    //     let mut sample_index = Arc::new(Mutex::new(0));
    //
    //     let output_stream = device
    //         .build_output_stream(
    //             &config,
    //             move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //                 let mut index = sample_index.lock().unwrap();
    //
    //                 for (i, output_sample) in data.iter_mut().enumerate() {
    //                     if *index < samples.len() {
    //                         *output_sample = samples[*index];
    //                         *index += 1;
    //                     } else {
    //                         *output_sample = 0.0;
    //                     }
    //                 }
    //             },
    //             |err| log::error!("Audio output error: {}", err),
    //             None,
    //         )
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //
    //     output_stream
    //         .play()
    //         .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;
    //     log::info!("Audio playback started successfully");
    //
    //     // Keep the stream alive
    //     std::mem::forget(output_stream);
    //
    //     Ok(())
    // }

    pub fn new(context: CapabilityContext<AudioOperation, Event>) -> Self {
        Self {
            context,
            recording_state: Arc::new(Mutex::new(AudioState {
                stream: None,
                samples: Vec::new(),
                config: None,
                state: RecordingState::Idle,
            })),
        }
    }

    pub fn start_recording(&self) -> Result<(), AudioError> {
        log::info!("Starting audio recording setup...");

        let host = cpal::default_host();
        log::info!("Got default host");

        let device = host
            .default_input_device()
            .ok_or(AudioError::DeviceNotFound)?;
        log::info!("Got default input device");

        // Get the default config
        let default_config = device
            .default_input_config()
            .map_err(|e| AudioError::RecordingFailed(e.to_string()))?;

        log::info!("Default config: {:?}", default_config);

        // Create a known working config for Android
        let config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(44100),
            buffer_size: cpal::BufferSize::Fixed(1024),
        };

        log::info!("Using stream config: {:?}", config);

        let recording_state = self.recording_state.clone();

        // Error handling callback
        let err_fn = move |err| {
            log::error!("An error occurred on stream: {}", err);
        };

        // Data handling callback
        let input_data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if let Ok(mut state) = recording_state.try_lock() {
                if state.state == RecordingState::Recording {
                    // Log the first few samples and buffer length for debugging
                    let preview: Vec<f32> = data.iter().take(5).cloned().collect();
                    log::info!(
                        "Recording samples - Buffer size: {}, First few samples: {:?}, Max amplitude: {:.2}",
                        data.len(),
                        preview,
                        data.iter().fold(0f32, |max, &x| max.max(x.abs()))
                    );

                    state.samples.extend_from_slice(data);
                }
            }
        };

        log::info!("Building input stream...");

        // Build the stream with explicit config
        let stream = device
            .build_input_stream(
                &config,
                input_data_fn,
                err_fn,
                Some(std::time::Duration::from_secs(1)),
            )
            .map_err(|e| {
                log::error!("Failed to build input stream: {}", e);
                AudioError::RecordingFailed(e.to_string())
            })?;

        log::info!("Stream built successfully, attempting to play...");

        // Try to play the stream
        stream.play().map_err(|e| {
            log::error!("Failed to play stream: {}", e);
            AudioError::RecordingFailed(e.to_string())
        })?;

        log::info!("Stream playing successfully");

        // Update state
        let mut state = self.recording_state.lock().unwrap();
        state.stream = Some(stream);
        state.config = Some(config);
        state.samples.clear();
        state.state = RecordingState::Recording;

        log::info!("Recording started successfully");
        Ok(())
    }

    pub fn stop_recording(&self) -> Result<AudioData, AudioError> {
        let mut state = self.recording_state.lock().unwrap();

        // Take ownership of the stream and drop it to stop recording
        let _stream = state.stream.take().ok_or(AudioError::InvalidState)?;

        let config = state.config.as_ref().ok_or(AudioError::InvalidState)?;
        let audio_data = AudioData {
            samples: state.samples.clone(),
            sample_rate: config.sample_rate.0,
            channels: config.channels,
        };

        state.state = RecordingState::Finished;
        Ok(audio_data)
    }
}

// // shared/src/audio.rs
// use serde::{Deserialize, Serialize};
// use crux_core::macros::Capability;
//
// #[derive(Serialize, Deserialize, Debug)]
// pub enum AudioEffect {
//     RequestPermission,
//     StartRecording,
//     StopRecording,
// }
//
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct AudioSamples {
//     pub samples: Vec<f32>,
// }
//
// // #[derive(Debug)]
// // pub enum Event {
// //     PermissionGranted,
// //     PermissionDenied,
// //     RecordingStarted,
// //     RecordingStopped(AudioSamples),
// //     RecordingError(String),
// // }
//
// #[derive(Capability)]
// pub struct Audio<Event> {
//     context: CapabilityContext<AudioOperation, Event>,
//     pub callback: Box<dyn Fn(Event) + Send>,
// }
//
// impl<Ev> Audio<Ev> {
//     pub fn new(callback: impl Fn(Ev) + Send + 'static) -> Self {
//         Self {
//             callback: Box::new(callback),
//         }
//     }
// }
//
// // For iOS
// #[cfg(target_os = "ios")]
// pub mod platform {
//     use super::*;
//
//     extern "C" {
//         fn ios_request_microphone_permission() -> bool;
//         fn ios_start_recording() -> bool;
//         fn ios_stop_recording() -> *const f32;
//         fn ios_get_samples_count() -> i32;
//     }
//
//     #[no_mangle]
//     pub extern "C" fn handle_ios_samples(samples: *const f32, count: i32) {
//         // This will be called from Swift when samples are ready
//         unsafe {
//             let samples = std::slice::from_raw_parts(samples, count as usize).to_vec();
//             // Send samples back through callback
//         }
//     }
// }
//
// // For Android
// #[cfg(target_os = "android")]
// pub mod platform {
//     use super::*;
//     use jni::objects::JObject;
//     use jni::JNIEnv;
//
//     #[no_mangle]
//     pub extern "system" fn Java_com_example_AudioRecorder_handleSamples(
//         env: JNIEnv,
//         _: JObject,
//         samples: JObject,
//     ) {
//         // This will be called from Kotlin/Java when samples are ready
//         // Convert JNI array to Vec<f32>
//     }
// }
//
// // For Web
// #[cfg(target_arch = "wasm32")]
// pub mod platform {
//     use super::*;
//     use wasm_bindgen::prelude::*;
//     use web_sys::{MediaRecorder, MediaStream, MediaStreamConstraints};
//
//     #[wasm_bindgen]
//     extern "C" {
//         #[wasm_bindgen(js_namespace = console)]
//         fn log(s: &str);
//     }
//
//     #[wasm_bindgen]
//     pub fn handle_web_samples(samples: &[f32]) {
//         // This will be called from JS when samples are ready
//     }
// }
