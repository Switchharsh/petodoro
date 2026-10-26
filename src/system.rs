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
         Categories=Utility;Clock;\n\
         Keywords=pomodoro;timer;stopwatch;pet;focus;\n\
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

fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home().join(".local").join("share"))
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn installed_paths() -> [PathBuf; 3] {
    let data = data_dir();
    [
        home().join(".local").join("bin").join("petodoro"),
        data.join("applications").join("petodoro.desktop"),
        data.join("icons").join("hicolor").join("256x256").join("apps").join("petodoro.png"),
    ]
}

/// Copy this binary to ~/.local/bin, add a menu entry and icon, and turn on
/// autostart. Returns the installed binary's path.
pub fn install_self() -> std::io::Result<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let [bin, desktop, icon] = installed_paths();
    for p in [&bin, &desktop, &icon] {
        fs::create_dir_all(p.parent().unwrap())?;
    }
    let exe = std::env::current_exe()?;
    if exe.canonicalize().ok() != bin.canonicalize().ok() {
        // Copy then rename, so replacing a running copy works.
        let tmp = bin.with_extension("new");
        fs::copy(&exe, &tmp)?;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))?;
        fs::rename(&tmp, &bin)?;
    }
    crate::render::icon(crate::pets::PetKind::Sakura, 256)
        .save_png(&icon)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    fs::write(&desktop, desktop_entry(&bin))?;
    let auto = autostart_file();
    fs::create_dir_all(auto.parent().unwrap())?;
    fs::write(auto, desktop_entry(&bin))?;
    Ok(bin)
}

pub fn uninstall_self() -> std::io::Result<()> {
    stop_others();
    for p in installed_paths().iter().chain([&autostart_file()]) {
        match fs::remove_file(p) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

/// Ask any other running petodoro to quit (but not this process).
fn stop_others() {
    let me = std::process::id();
    let Ok(dir) = fs::read_dir("/proc") else { return };
    for entry in dir.flatten() {
        let Some(pid) = entry.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let is_pet = fs::read_to_string(entry.path().join("comm")).is_ok_and(|c| c.trim() == "petodoro");
        if pid != me && is_pet {
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        }
    }
}

/// Start a detached copy of petodoro.
pub fn launch(bin: &Path) {
    let _ = Command::new(bin).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
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
