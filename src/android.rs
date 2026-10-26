//! JNI entry points for the Android app (`io.github.switchharsh.petodoro.PetService`).

use crate::platform::{android as bridge, WinEvent};
use jni::objects::{JObject, JString};
use jni::sys::{jfloat, jint};
use jni::JNIEnv;
use std::sync::atomic::{AtomicBool, Ordering};

static RUNNING: AtomicBool = AtomicBool::new(false);

/// The service started: remember it, then run the pet on its own thread.
#[no_mangle]
pub extern "system" fn Java_io_github_switchharsh_petodoro_PetService_nativeStart(
    mut env: JNIEnv,
    this: JObject,
    files_dir: JString,
    width: jint,
    height: jint,
    density: jfloat,
) {
    let dir: String = env.get_string(&files_dir).map(Into::into).unwrap_or_default();
    // Settings live under the app's private files directory.
    std::env::set_var("HOME", &dir);
    let (Ok(vm), Ok(service)) = (env.get_java_vm(), env.new_global_ref(&this)) else { return };
    bridge::set_bridge(vm, service, width, height, density);
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        let cfg = crate::config::Config::load();
        let _ = crate::run(cfg);
        RUNNING.store(false, Ordering::SeqCst);
        // The user chose QUIT (or something failed): stop the service.
        bridge::with_service(|env, s| env.call_method(s, "quit", "()V", &[]).map(|_| ()));
    });
}

#[no_mangle]
pub extern "system" fn Java_io_github_switchharsh_petodoro_PetService_nativeTouch(
    _env: JNIEnv,
    _this: JObject,
    id: jint,
    action: jint,
    x: jint,
    y: jint,
    raw_x: jint,
    raw_y: jint,
) {
    bridge::push_touch(id, action, x, y, raw_x, raw_y);
}

#[no_mangle]
pub extern "system" fn Java_io_github_switchharsh_petodoro_PetService_nativeScreen(
    _env: JNIEnv,
    _this: JObject,
    width: jint,
    height: jint,
) {
    bridge::set_screen(width, height);
}

/// The service is being destroyed: quit the pet and forget the service.
#[no_mangle]
pub extern "system" fn Java_io_github_switchharsh_petodoro_PetService_nativeStop(_env: JNIEnv, _this: JObject) {
    bridge::push(WinEvent::Close);
    bridge::clear_bridge();
}
