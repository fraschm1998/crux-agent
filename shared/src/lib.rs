#[cfg(target_os = "android")]
mod android_init;

pub mod app;
pub mod capabilities;
pub mod effects;
pub mod events;
pub mod middleware;
pub mod utils;

use lazy_static::lazy_static;
use middleware::LiveKitMiddleware;
use wasm_bindgen::prelude::wasm_bindgen;

pub use crux_core::{bridge::Bridge, Core, Request};
pub use crux_http as http;

pub use app::{App, Event};

uniffi::include_scaffolding!("shared");

lazy_static! {
    static ref CORE: Bridge<LiveKitMiddleware<Core<App>>> = {
        let core = Core::new();
        let middleware = LiveKitMiddleware::new(core);
        Bridge::new(middleware)
    };
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
