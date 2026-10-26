//! Desktop integration: notifications, sounds, autostart, single instance.

use crate::config::config_dir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn spawn(cmd: &str, args: &[&str]) -> bool {
    Command::new(cmd).args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().is_ok()
}

/// Desktop notification via `notify-send` (libnotify-bin, preinstalled on Mint).
pub fn notify(summary: &str, body: &str) {
    spawn("notify-send", &["--app-name=Petodoro", "--icon=petodoro", "--urgency=normal", summary, body]);
}

const SOUNDS: [&str; 4] = [
    "/usr/share/sounds/freedesktop/stereo/complete.oga",
    "/usr/share/sounds/freedesktop/stereo/bell.oga",
    "/usr/share/sounds/LinuxMint/stereo/dialog-information.ogg",
    "/usr/share/sounds/ubuntu/stereo/message.ogg",
];

/// Play a short chime. Returns false if nothing could be played.
pub fn chime() -> bool {
    if let Some(f) = SOUNDS.iter().find(|f| Path::new(f).exists()) {
        if spawn("paplay", &[f]) || spawn("pw-play", &[f]) {
            return true;
        }
    }
    spawn("canberra-gtk-play", &["--id=complete"])
}

fn autostart_file() -> PathBuf {
    config_dir().join("autostart").join("petodoro.desktop")
}

pub fn desktop_entry(exec: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Petodoro\n\
         Comment=Floating desktop pet with a pomodoro timer and stopwatch\n\
         Exec=\"{}\"\n\
         Icon=petodoro\n\
         Terminal=false\n\
         Categories=Utility;\n\
         X-GNOME-Autostart-enabled=true\n",
        exec.display()
    )
}

pub fn autostart_enabled() -> bool {
    autostart_file().exists()
}

pub fn set_autostart(on: bool) -> std::io::Result<()> {
    let f = autostart_file();
    if on {
        fs::create_dir_all(f.parent().unwrap())?;
        let exe = std::env::current_exe()?;
        fs::write(&f, desktop_entry(&exe))
    } else {
        match fs::remove_file(&f) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

/// Take an exclusive lock so only one pet runs per user session. The lock
/// is released automatically when the process exits.
pub fn single_instance() -> bool {
    use std::os::unix::io::IntoRawFd;
    let dir = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let uid = unsafe { libc::getuid() };
    let path = dir.join(format!("petodoro-{uid}.lock"));
    let Ok(file) = fs::OpenOptions::new().create(true).truncate(false).write(true).open(&path) else {
        return true; // Can't lock: better to run than to refuse.
    };
    let fd = file.into_raw_fd(); // Leaked on purpose: keeps the lock held.
    unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) == 0 }
}

/// Let the kernel reap our fire-and-forget children (notify-send, paplay).
pub fn ignore_children() {
    unsafe {
        libc::signal(libc::SIGCHLD, libc::SIG_IGN);
    }
}
