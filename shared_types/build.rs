use crux_core::typegen::TypeGen;
use shared::app::{AppEffect, View};
use shared::capabilities::audio::{AudioError, AudioResponse, RecordingState};
use shared::capabilities::livekit::LiveKitError;
use shared::events::livekit::LiveKitEvent;
use shared::{http::HttpError, App};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../shared");

    let mut gen = TypeGen::new();

    gen.register_app::<App>()?;

    gen.register_type::<HttpError>()?;
    
    // Recording Types
    gen.register_type::<AudioError>()?;
    gen.register_type::<AudioResponse>()?;
    gen.register_type::<RecordingState>()?;

    gen.register_type::<LiveKitEvent>()?;
    gen.register_type::<LiveKitError>()?;

    gen.register_type::<AppEffect>()?;
    gen.register_type::<View>()?;

    let output_root = PathBuf::from("./generated");

    gen.swift("SharedTypes", output_root.join("swift"))?;

    gen.java("me.fraschetti.agent.shared_types", output_root.join("java"))?;

    gen.typescript("shared_types", output_root.join("typescript"))?;

    Ok(())
}
