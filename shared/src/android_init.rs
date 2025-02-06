use jni::errors::Error as JniError;
use jni::objects::JObject;
use jni::sys::{jint, JNI_VERSION_1_6};
use jni::{JNIEnv, JavaVM};
use once_cell::sync::OnceCell;
use std::ffi::c_void;
use std::ptr;
use std::sync::Once;

static INIT: Once = Once::new();
//
// #[allow(non_snake_case)]
// #[no_mangle]
// pub extern "C" fn JNI_OnLoad(vm: JavaVM, _: *mut c_void) -> jint {
//     // Only initialize logging, defer WebRTC initialization
//     android_logger::init_once(
//         android_logger::Config::default()
//             .with_max_level(log::LevelFilter::Info)
//             .with_tag("AudioRecorder"),
//     );
//     log::info!("JNI_OnLoad called");
//     log::info!("JNI_OnLoad, initializing LiveKit");
//     livekit::webrtc::android::initialize_android(&vm);
//     JNI_VERSION_1_6
// }

#[no_mangle]
pub extern "system" fn Java_me_fraschetti_agent_MainActivity_initializeRustContext(
    env: JNIEnv,
    _: JObject,
    context: JObject,
) {
    // Initialize the Android logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("Agent"),
    );

    // Get the Java VM pointer
    let vm = env.get_java_vm().unwrap();
    let vm_ptr = vm.get_java_vm_pointer() as *mut c_void;

    // Initialize the Android context
    unsafe {
        ndk_context::initialize_android_context(vm_ptr, context.as_raw() as *mut c_void);
    }

    // Log success
    log::info!("Android context initialized successfully");
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_me_fraschetti_agent_MainActivity_initializeWebRTC(
    env: jni::JNIEnv,
    _: jni::objects::JClass,
    context: jni::objects::JObject,
) {
    let vm = env.get_java_vm().unwrap();
    livekit::webrtc::android::initialize_android(&vm);
}

// #[no_mangle]
// pub extern "system" fn Java_me_fraschetti_agent_MainActivity_initializeRustContext(
//     env: JNIEnv,
//     _: JObject,
//     context: JObject,
// ) -> jint {
//     let result = std::panic::catch_unwind(|| {
//         // Get the Java VM pointer
//         let vm = env.get_java_vm().unwrap();
//         let vm_ptr = vm.get_java_vm_pointer() as *mut c_void;
//
//         // Initialize the Android context
//         unsafe {
//             ndk_context::initialize_android_context(vm_ptr, context.as_raw() as *mut c_void);
//         }
//         log::info!("Android context initialized successfully");
//
//         // Initialize WebRTC only once
//         INIT.call_once(|| {
//             log::info!("Initializing LiveKit/WebRTC");
//             livekit::webrtc::android::initialize_android(&vm);
//             log::info!("LiveKit/WebRTC initialization completed");
//         });
//
//         0 // Success
//     });
//
//     match result {
//         Ok(status) => status,
//         Err(e) => {
//             log::error!("Initialization failed: {:?}", e);
//             -1 // Error
//         }
//     }
// }
