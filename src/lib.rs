//! Petodoro: a tiny floating desktop pet with a pomodoro timer and stopwatch.
//! The library holds everything; `main.rs` is the desktop command line and
//! `android.rs` the entry points for the Android app.
#![allow(unknown_lints, clippy::manual_is_multiple_of, clippy::too_many_arguments)]

pub mod app;
pub mod config;
pub mod font;
pub mod gfx;
pub mod lines;
pub mod pets;
pub mod platform;
pub mod render;
pub mod system;
pub mod timer;

#[cfg(target_os = "android")]
mod android;

use std::time::Instant;

/// Run the pet until the user quits.
pub fn run(cfg: config::Config) -> platform::Res<()> {
    let dpy = platform::Display::connect()?;
    let mut app = app::App::new(dpy, cfg)?;
    loop {
        while let Some(ev) = app.dpy.poll()? {
            app.handle(ev)?;
        }
        if app.quit {
            return Ok(());
        }
        let now = Instant::now();
        app.update(now)?;
        app.render(now)?;
        app.dpy.flush()?;
        // Events may have been buffered while we were talking to the server.
        if let Some(ev) = app.dpy.poll()? {
            app.handle(ev)?;
            continue;
        }
        // Sleep until there's input or the next frame is due.
        app.dpy.wait(app.next_timeout(Instant::now()));
    }
}
