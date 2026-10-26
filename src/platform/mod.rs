//! Windowing backends. Each one provides a `Display` and a `Surface` with the
//! same small API: an always-on-top, borderless, per-pixel transparent pet
//! window, a popup menu window, and mouse events.
//!
//! - Linux / BSD: X11 (x11rb)
//! - Windows: Win32 layered windows
//! - macOS: Cocoa
//! - Android: overlay views driven through JNI

use std::error::Error;

pub type Res<T> = Result<T, Box<dyn Error>>;

/// A rectangle in screen pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Bounds {
    pub fn right(&self) -> i32 {
        self.x + self.w
    }
    pub fn bottom(&self) -> i32 {
        self.y + self.h
    }
}

pub type WinId = u64;

/// Input from the windowing system. Buttons: 1 left, 2 middle, 3 right,
/// 4/5 wheel up/down. `x`/`y` are window-relative, `rx`/`ry` are screen
/// coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // not every backend produces every event
pub enum WinEvent {
    Press {
        win: WinId,
        button: u8,
        x: i32,
        y: i32,
        rx: i32,
        ry: i32,
    },
    Release {
        win: WinId,
        button: u8,
        x: i32,
        y: i32,
        rx: i32,
        ry: i32,
    },
    Motion {
        win: WinId,
        x: i32,
        y: i32,
        rx: i32,
        ry: i32,
    },
    /// The window's contents were lost and must be redrawn.
    Expose(WinId),
    /// The window lost focus (closes the menu where pointer grabs don't exist).
    FocusLost(WinId),
    /// The user or the system asked the app to quit.
    Close,
}

/// Convert tiny-skia's premultiplied RGBA into premultiplied BGRA.
#[allow(dead_code)]
pub fn rgba_to_bgra(src: &[u8], dst: &mut Vec<u8>) {
    dst.clear();
    dst.reserve(src.len());
    for px in src.chunks_exact(4) {
        dst.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
    }
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
mod x11;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
pub use x11::{Display, Surface};

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::{Display, Surface};

#[cfg(target_os = "android")]
pub(crate) mod android;
#[cfg(target_os = "android")]
pub use android::{Display, Surface};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{Display, Surface};
