#[cfg(target_os = "android")]
mod android_init;

pub mod app;
pub mod capabilities;
pub mod events;
pub mod utils;

use std::sync::Mutex;

use lazy_static::lazy_static;
use tokio::sync::mpsc;
use wasm_bindgen::prelude::wasm_bindgen;

pub use crux_core::{bridge::Bridge, Core, Request};
pub use crux_http as http;

pub use app::*;
// pub use capabilities::audio;
// pub use capabilities::livekit;
// pub use capabilities::sse;

// TODO hide this plumbing

uniffi::include_scaffolding!("shared");

lazy_static! {
    static ref CORE: Bridge<App> = Bridge::new(Core::new());
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    static ref EVENT_SENDER: Mutex<Option<mpsc::UnboundedSender<Event>>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn process_event(data: &[u8]) -> Vec<u8> {
    CORE.process_event(data)
}

#[wasm_bindgen]
pub fn handle_response(id: u32, data: &[u8]) -> Vec<u8> {
    CORE.handle_response(id, data)
}

#[wasm_bindgen]
pub fn view() -> Vec<u8> {
    CORE.view()
}
