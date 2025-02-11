#[cfg(target_os = "android")]
mod android_init;

pub mod app;
pub mod capabilities;
pub mod effects;
pub mod events;
pub mod middleware;
pub mod utils;

// mod middleware {
//     mod livekit_handler;
//     pub use livekit_handler::LiveKitHandler;
// }

use app::{Effect, View};
use lazy_static::lazy_static;
use middleware::LiveKitMiddleware;
// use middleware::LiveKitHandler;
use wasm_bindgen::prelude::wasm_bindgen;

pub use crux_core::{bridge::Bridge, Core, Request};
pub use crux_http as http;

pub use app::{App, Event};

// Define CoreLike trait
// pub trait CoreLike {
//     fn handle_event(&self, event: Event) -> Effect;
//     fn handle_response(&self, id: u32, data: &[u8]) -> Effect;
//     fn view(&self) -> View;
// }

uniffi::include_scaffolding!("shared");

// lazy_static! {
//     static ref HANDLER: LiveKitHandler = LiveKitHandler {
//         core: Core::new(),
//         runtime: tokio::runtime::Builder::new_multi_thread()
//             .enable_all()
//             .build()
//             .unwrap(),
//     };
//     static ref CORE: Bridge<LiveKitHandler> = Bridge::new(HANDLER.clone());
// }

// lazy_static! {
//     static ref HANDLER: LiveKitHandler = LiveKitHandler::new(Core::new());
//     static ref CORE: Bridge<LiveKitHandler> = Bridge::new(HANDLER.clone());
// }

// lazy_static! {
//     static ref CORE: Bridge<Core<App>> = Bridge::new(Core::new());
//     static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap();
// }

// lazy_static! {
//     static ref HANDLER: LiveKitHandler = {
//         let core = Core::new();
//         let context = CapabilityContext::new();
//         LiveKitHandler::new(core, context)
//     };
//     static ref CORE: Bridge<LiveKitHandler> = Bridge::new(HANDLER.clone());
// }

// lazy_static! {
//     // Core wrapped by LiveKitMiddleware, then given to Bridge
//     static ref CORE: Bridge<LiveKitMiddleware<Core<App>>> = {
//         // Create the base Core
//         let core = Core::new();
//
//         // Wrap it in our LiveKit middleware
//         let middleware = LiveKitMiddleware::new(core);
//
//         // Create the Bridge with the middleware
//         Bridge::new(middleware)
//     };
// }

lazy_static! {
    static ref CORE: Bridge<LiveKitMiddleware<Core<App>>> = {
        let core = Core::new();
        let middleware = LiveKitMiddleware::new(core);
        Bridge::new(middleware)
    };
}

// lazy_static! {
//     static ref HANDLER: LiveKitHandler = {
//         let core = Core::new();
//         let context = CapabilityContext::new();
//         LiveKitHandler::new(core, context)
//     };
//     static ref CORE: Bridge<LiveKitHandler> = Bridge::new(HANDLER.clone());
// }

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
