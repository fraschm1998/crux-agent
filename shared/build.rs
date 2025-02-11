use std::env;
use std::path::PathBuf;

fn main() {
    // Generate UniFFI bindings
    uniffi::generate_scaffolding("./src/shared.udl").unwrap();

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let cargo_bin_name = env::var("CARGO_BIN_NAME").unwrap_or_default();

    // Only configure WebRTC for Android and when not building uniffi-bindgen
    if target_os == "android" && cargo_bin_name != "uniffi-bindgen" {
        configure_android().unwrap();
    }
}

fn configure_android() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Configure WebRTC JNI symbols
    webrtc_sys_build::configure_jni_symbols()?;

    // Add linking configuration
    println!("cargo:rustc-link-lib=static=webrtc");
    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=dylib=c++_shared");

    // Add search paths
    let webrtc_lib_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
        .join("third_party")
        .join("webrtc")
        .join("lib");

    println!(
        "cargo:rustc-link-search=native={}",
        webrtc_lib_dir.display()
    );

    // Generate JNI bindings configuration
    let jni_config = out_dir.join("jni_config.h");
    std::fs::write(
        &jni_config,
        r#"
        #define JNI_LIB_NAME "shared"
        #define JNI_PACKAGE_NAME "me.fraschetti.agent.shared"
    "#,
    )?;

    Ok(())
}

