//! Android backend. The pet runs on its own thread; windows are overlay
//! views ("display over other apps") owned by the Kotlin `PetService`,
//! which this module drives through JNI. Touch input comes back through
//! `push_touch` into an event queue.

use super::{Bounds, Res, WinEvent, WinId};
use jni::objects::{GlobalRef, JValue};
use jni::{JNIEnv, JavaVM};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;
use tiny_skia::Pixmap;

struct Bridge {
    vm: JavaVM,
    service: GlobalRef,
}

static BRIDGE: Mutex<Option<Bridge>> = Mutex::new(None);
static QUEUE: Mutex<VecDeque<WinEvent>> = Mutex::new(VecDeque::new());
static WAKE: Condvar = Condvar::new();
/// Screen width, height (pixels) and density (1.0 = 160 dpi).
static SCREEN: Mutex<(i32, i32, f32)> = Mutex::new((1080, 1920, 2.0));
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Called by the service when it starts (or restarts).
pub(crate) fn set_bridge(vm: JavaVM, service: GlobalRef, w: i32, h: i32, density: f32) {
    *BRIDGE.lock().unwrap() = Some(Bridge { vm, service });
    *SCREEN.lock().unwrap() = (w, h, density);
    // Drop a stale Close from a previous session so the new pet doesn't quit.
    QUEUE.lock().unwrap().clear();
}

pub(crate) fn clear_bridge() {
    *BRIDGE.lock().unwrap() = None;
}

pub(crate) fn set_screen(w: i32, h: i32) {
    let mut s = SCREEN.lock().unwrap();
    s.0 = w;
    s.1 = h;
}

pub(crate) fn push(ev: WinEvent) {
    QUEUE.lock().unwrap().push_back(ev);
    WAKE.notify_all();
}

/// Touch from a view. Actions: 0 down, 1 up, 2 move, 3 long press
/// (= right click), 4 touch outside the window.
pub(crate) fn push_touch(id: i32, action: i32, x: i32, y: i32, rx: i32, ry: i32) {
    let win = id as WinId;
    let ev = match action {
        0 => WinEvent::Press { win, button: 1, x, y, rx, ry },
        1 => WinEvent::Release { win, button: 1, x, y, rx, ry },
        2 => WinEvent::Motion { win, x, y, rx, ry },
        3 => WinEvent::Press { win, button: 3, x, y, rx, ry },
        4 => WinEvent::FocusLost(win),
        _ => return,
    };
    push(ev);
}

/// Run `f` with a JNI env attached to this thread and the service object.
pub(crate) fn with_service(f: impl FnOnce(&mut JNIEnv, &GlobalRef) -> jni::errors::Result<()>) {
    let guard = BRIDGE.lock().unwrap();
    let Some(b) = guard.as_ref() else { return };
    let Ok(mut env) = b.vm.attach_current_thread_permanently() else { return };
    if f(&mut env, &b.service).is_err() || env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
    }
}

fn call(method: &str, sig: &str, args: &[JValue]) {
    with_service(|env, s| env.call_method(s, method, sig, args).map(|_| ()));
}

pub struct Display {
    pub screen: Bounds,
    pub work: Bounds,
    /// Phones pack 2-4x more pixels per inch than desktops.
    pub ui_scale: f32,
}

pub struct Surface {
    id: WinId,
    pub w: u16,
    pub h: u16,
}

impl Surface {
    pub fn id(&self) -> WinId {
        self.id
    }
}

impl Display {
    pub fn connect() -> Res<Display> {
        if BRIDGE.lock().unwrap().is_none() {
            return Err("the Android service is not running".into());
        }
        let (w, h, density) = *SCREEN.lock().unwrap();
        let screen = Bounds { x: 0, y: 0, w, h };
        // Start above the launcher's dock and search bar, not on top of them.
        let work = Bounds { x: 0, y: 0, w, h: h - (200.0 * density) as i32 };
        Ok(Display { screen, work, ui_scale: (density / 1.6).max(1.0) })
    }

    fn create(&self, x: i32, y: i32, w: u16, h: u16, menu: bool) -> Res<Surface> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        call(
            "createWindow",
            "(IIIIIZ)V",
            &[
                JValue::Int(id as i32),
                JValue::Int(x),
                JValue::Int(y),
                JValue::Int(w as i32),
                JValue::Int(h as i32),
                JValue::Bool(menu as u8),
            ],
        );
        Ok(Surface { id, w, h })
    }

    pub fn create_pet_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        self.create(x, y, w, h, false)
    }

    pub fn create_menu_window(&self, x: i32, y: i32, w: u16, h: u16) -> Res<Surface> {
        self.create(x, y, w, h, true)
    }

    pub fn destroy(&self, s: Surface) -> Res<()> {
        call("destroyWindow", "(I)V", &[JValue::Int(s.id as i32)]);
        Ok(())
    }

    pub fn move_window(&self, s: &Surface, x: i32, y: i32) -> Res<()> {
        call("moveWindow", "(III)V", &[JValue::Int(s.id as i32), JValue::Int(x), JValue::Int(y)]);
        Ok(())
    }

    pub fn move_resize(&self, s: &mut Surface, x: i32, y: i32, w: u16, h: u16) -> Res<()> {
        s.w = w;
        s.h = h;
        call(
            "resizeWindow",
            "(IIIII)V",
            &[JValue::Int(s.id as i32), JValue::Int(x), JValue::Int(y), JValue::Int(w as i32), JValue::Int(h as i32)],
        );
        Ok(())
    }

    pub fn bell(&self) {
        call("chime", "()V", &[]);
    }

    /// Android's ARGB_8888 bitmaps are premultiplied RGBA in memory, the
    /// same layout tiny-skia uses, so the pixels go across unchanged.
    pub fn present(&self, s: &mut Surface, pm: &Pixmap) -> Res<()> {
        let (id, w, h) = (s.id as i32, pm.width() as i32, pm.height() as i32);
        with_service(|env, svc| {
            let pixels = env.byte_array_from_slice(pm.data())?;
            let args = [JValue::Int(id), JValue::Int(w), JValue::Int(h), JValue::Object(&pixels)];
            env.call_method(svc, "present", "(III[B)V", &args)?;
            env.delete_local_ref(pixels)
        });
        Ok(())
    }

    pub fn poll(&self) -> Res<Option<WinEvent>> {
        Ok(QUEUE.lock().unwrap().pop_front())
    }

    pub fn flush(&self) -> Res<()> {
        Ok(())
    }

    pub fn wait(&self, timeout: Duration) {
        let q = QUEUE.lock().unwrap();
        if q.is_empty() {
            let _ = WAKE.wait_timeout(q, timeout);
        }
    }
}
