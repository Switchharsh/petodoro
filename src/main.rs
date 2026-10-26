//! Petodoro: a tiny floating desktop pet with a pomodoro timer and stopwatch.
#![allow(unknown_lints, clippy::manual_is_multiple_of, clippy::too_many_arguments)]

mod app;
mod config;
mod font;
mod gfx;
mod lines;
mod pets;
mod platform;
mod render;
mod system;
mod timer;

use config::{Config, Mode};
use std::time::Instant;
use x11rb::connection::Connection;

const HELP: &str = "\
petodoro - a floating desktop pet with a pomodoro timer and stopwatch

USAGE:
    petodoro [OPTIONS]

SETUP:
    --install             copy petodoro to ~/.local/bin, add it to the app menu,
                          start it on login, and launch it
    --uninstall           undo --install (settings are kept)

OPTIONS:
    --pet NAME            see --list-pets
    --mode MODE           pomodoro | stopwatch | pet
    --theme THEME         auto | light | dark | amoled (bubble and menu)
    --scale N             size multiplier (1 = small, 1.5 = default, 2.5 = huge)
    --focus MIN           focus session length in minutes
    --break MIN           short break length in minutes
    --reset-position      put the pet back in the bottom-right corner
    --autostart on|off    start (or stop starting) petodoro when you log in
    --list-pets           show the available pets and their personalities
    --preview FILE.png    render every pet in every mood to a PNG and exit
    --gallery FILE.png    render all pets saying hello to a PNG and exit
    --icon FILE.png       render a 256px app icon and exit
    -h, --help            show this help
    -V, --version         show the version

Options that change settings are remembered in ~/.config/petodoro/config.

MOUSE:
    left-drag             pick the pet up and drop it anywhere (shake it and it gets dizzy)
    left-click bubble     start / pause the timer
    left-click pet        pet them (click fast for a combo!)
    middle-click          reset the timer
    scroll on pet         change the length of a phase that hasn't started
    right-click           menu: snacks, stats, mode, pet, size, theme, timer lengths,
                          walking, chatter, sound, autostart, quit
";

fn die(msg: impl std::fmt::Display) -> ! {
    eprintln!("petodoro: {msg}");
    std::process::exit(2)
}

fn main() {
    let mut cfg = Config::load();
    let mut changed = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().unwrap_or_else(|| die(format!("{arg} needs a value")));
        let minutes = |v: String| {
            v.parse::<u32>().ok().filter(|m| (1..=600).contains(m)).unwrap_or_else(|| die("minutes must be 1-600"))
        };
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return;
            }
            "-V" | "--version" => {
                println!("petodoro {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--list-pets" => {
                for p in pets::ALL {
                    println!("{:<8} {:<9} {}", p.name(), p.persona().name(), p.blurb());
                }
                return;
            }
            "--install" => {
                let bin = system::install_self().unwrap_or_else(|e| die(format!("install failed: {e}")));
                println!("Installed {}", bin.display());
                println!("Petodoro is in your app menu and will start when you log in");
                println!("(turn that off from its right-click menu). Launching it now~");
                system::launch(&bin);
                return;
            }
            "--uninstall" => {
                system::uninstall_self().unwrap_or_else(|e| die(format!("uninstall failed: {e}")));
                println!("Petodoro removed (settings kept in ~/.config/petodoro). Bye bye~");
                return;
            }
            "--gallery" => {
                let out = value();
                render::gallery(1.5, 7).save_png(&out).unwrap_or_else(|e| die(e));
                println!("wrote {out}");
                return;
            }
            "--theme" => {
                let v = value();
                cfg.theme = config::ThemeChoice::from_name(&v).unwrap_or_else(|| die(format!("unknown theme {v:?}")));
                changed = true;
            }
            "--preview" => {
                let out = value();
                render::preview_sheet(1.5).save_png(&out).unwrap_or_else(|e| die(e));
                println!("wrote {out}");
                return;
            }
            "--icon" => {
                let out = value();
                render::icon(cfg.pet, 256).save_png(&out).unwrap_or_else(|e| die(e));
                return;
            }
            "--autostart" => {
                let on = match value().as_str() {
                    "on" => true,
                    "off" => false,
                    _ => die("--autostart takes on or off"),
                };
                system::set_autostart(on).unwrap_or_else(|e| die(e));
                println!("autostart {}", if on { "enabled" } else { "disabled" });
                return;
            }
            "--pet" => {
                let v = value();
                cfg.pet =
                    pets::PetKind::from_name(&v).unwrap_or_else(|| die(format!("unknown pet {v:?} (try --list-pets)")));
                changed = true;
            }
            "--mode" => {
                let v = value();
                cfg.mode = Mode::from_name(&v).unwrap_or_else(|| die(format!("unknown mode {v:?}")));
                changed = true;
            }
            "--scale" => {
                cfg.scale = value()
                    .parse::<f32>()
                    .ok()
                    .filter(|s| (0.5..=4.0).contains(s))
                    .unwrap_or_else(|| die("scale must be 0.5-4"));
                changed = true;
            }
            "--focus" => {
                cfg.pomo.focus_min = minutes(value());
                changed = true;
            }
            "--break" => {
                cfg.pomo.short_min = minutes(value());
                changed = true;
            }
            "--reset-position" => {
                cfg.pos = None;
                changed = true;
            }
            _ => die(format!("unknown option {arg:?} (see --help)")),
        }
    }

    if !system::single_instance() {
        eprintln!("petodoro is already running");
        return;
    }
    if changed {
        cfg.save();
    }
    system::ignore_children();

    if let Err(e) = run(cfg) {
        if std::env::var_os("DISPLAY").is_none() {
            die("no X display found. Petodoro needs an X11 session (the default on Linux Mint)");
        }
        die(e);
    }
}

fn run(cfg: Config) -> platform::Res<()> {
    let x = platform::X::connect()?;
    let fd = x.fd();
    let mut app = app::App::new(x, cfg)?;
    loop {
        while let Some(ev) = app.x.conn.poll_for_event()? {
            app.handle(ev)?;
        }
        if app.quit {
            return Ok(());
        }
        let now = Instant::now();
        app.update(now)?;
        app.render(now)?;
        app.x.conn.flush()?;
        // Events may have been buffered while we were talking to the server.
        if let Some(ev) = app.x.conn.poll_for_event()? {
            app.handle(ev)?;
            continue;
        }
        let timeout = app.next_timeout(Instant::now());
        let mut pfd = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
        // Sleep until X has something for us or the next frame is due.
        unsafe { libc::poll(&mut pfd, 1, timeout.as_millis().clamp(1, 60_000) as i32) };
    }
}
