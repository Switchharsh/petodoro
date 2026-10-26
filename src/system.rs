//! Desktop integration: notifications, sounds, autostart, installing,
//! single instance. One section per operating system.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn spawn(cmd: &str, args: &[&str]) -> bool {
    quiet(Command::new(cmd).args(args)).spawn().is_ok()
}

fn quiet(c: &mut Command) -> &mut Command {
    c.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flashing up
    }
    c
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
}

fn other_error(e: impl ToString) -> std::io::Error {
    std::io::Error::other(e.to_string())
}

fn write_icon_png(path: &Path) -> std::io::Result<()> {
    crate::render::icon(crate::pets::PetKind::Tomo, 256).save_png(path).map_err(other_error)
}

/// Start a detached copy of petodoro.
pub fn launch(bin: &Path) {
    let _ = quiet(&mut Command::new(bin)).spawn();
}

fn remove_files(paths: &[PathBuf]) -> std::io::Result<()> {
    for p in paths {
        match fs::remove_file(p) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

// ------------------------------------------------------------ Linux / BSD

#[cfg(all(unix, not(target_os = "macos")))]
mod os {
    use super::*;
    use crate::config::config_dir;

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

    /// Quote a path for a .desktop `Exec=` line, per the Desktop Entry spec.
    fn exec_quote(p: &Path) -> String {
        let mut out = String::from("\"");
        for c in p.display().to_string().chars() {
            match c {
                // Escaped for the quoted argument (\c), then the backslash
                // escaped again for the string value (\\c).
                '"' | '`' | '$' => {
                    out.push_str("\\\\");
                    out.push(c);
                }
                '\\' => out.push_str("\\\\\\\\"),
                '%' => out.push_str("%%"),
                '\n' | '\r' => {}
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    pub fn desktop_entry(exec: &Path) -> String {
        format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Petodoro\n\
             Comment=Floating desktop pet with a pomodoro timer and stopwatch\n\
             Exec={}\n\
             Icon=petodoro\n\
             Terminal=false\n\
             Categories=Utility;Clock;\n\
             Keywords=pomodoro;timer;stopwatch;pet;focus;\n\
             X-GNOME-Autostart-enabled=true\n",
            exec_quote(exec)
        )
    }

    pub fn autostart_enabled() -> bool {
        autostart_file().exists()
    }

    pub fn set_autostart(on: bool) -> std::io::Result<()> {
        let f = autostart_file();
        if on {
            fs::create_dir_all(f.parent().unwrap())?;
            fs::write(&f, desktop_entry(&std::env::current_exe()?))
        } else {
            remove_files(&[f])
        }
    }

    fn data_dir() -> PathBuf {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home().join(".local").join("share"))
    }

    fn installed_paths() -> [PathBuf; 3] {
        let data = data_dir();
        [
            home().join(".local").join("bin").join("petodoro"),
            data.join("applications").join("petodoro.desktop"),
            data.join("icons").join("hicolor").join("256x256").join("apps").join("petodoro.png"),
        ]
    }

    /// Copy this binary to ~/.local/bin, add a menu entry and icon, and turn
    /// on autostart. Returns the installed binary's path.
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
        write_icon_png(&icon)?;
        fs::write(&desktop, desktop_entry(&bin))?;
        let auto = autostart_file();
        fs::create_dir_all(auto.parent().unwrap())?;
        fs::write(auto, desktop_entry(&bin))?;
        Ok(bin)
    }

    pub fn uninstall_self() -> std::io::Result<()> {
        stop_others();
        let mut paths = installed_paths().to_vec();
        paths.push(autostart_file());
        remove_files(&paths)
    }

    /// Running petodoro processes other than this one: (pid, executable).
    fn others() -> Vec<(u32, String)> {
        let me = std::process::id();
        let Ok(dir) = fs::read_dir("/proc") else { return vec![] };
        dir.flatten()
            .filter_map(|e| {
                let pid = e.file_name().to_str()?.parse::<u32>().ok()?;
                let comm = fs::read_to_string(e.path().join("comm")).ok()?;
                if pid == me || comm.trim() != "petodoro" {
                    return None;
                }
                let exe = fs::read_link(e.path().join("exe")).map(|p| p.display().to_string()).unwrap_or_default();
                Some((pid, exe))
            })
            .collect()
    }

    /// Stop running copies that are a different binary than this one, or
    /// whose binary was replaced on disk (an upgrade). Returns true if any
    /// were asked to quit.
    pub fn stop_outdated_instances() -> bool {
        let mine = std::env::current_exe().map(|p| p.display().to_string()).unwrap_or_default();
        let mut stopped = false;
        for (pid, exe) in others() {
            if exe != mine || exe.ends_with(" (deleted)") {
                unsafe { libc::kill(pid as i32, libc::SIGTERM) };
                stopped = true;
            }
        }
        stopped
    }

    /// Ask any other running petodoro to quit (but not this process).
    fn stop_others() {
        for (pid, _) in others() {
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
        }
    }
}

// ------------------------------------------------------------------ macOS

#[cfg(target_os = "macos")]
mod os {
    use super::*;

    fn osa_quote(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    pub fn notify(summary: &str, body: &str) {
        let script = format!("display notification \"{}\" with title \"{}\"", osa_quote(body), osa_quote(summary));
        spawn("osascript", &["-e", &script]);
    }

    pub fn chime() -> bool {
        spawn("afplay", &["/System/Library/Sounds/Glass.aiff"])
    }

    fn agent_file() -> PathBuf {
        home().join("Library").join("LaunchAgents").join("com.petodoro.plist")
    }

    fn agent_plist(exec: &Path) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\"><dict>\n\
             <key>Label</key><string>com.petodoro</string>\n\
             <key>ProgramArguments</key><array><string>{}</string></array>\n\
             <key>RunAtLoad</key><true/>\n\
             </dict></plist>\n",
            exec.display()
        )
    }

    pub fn autostart_enabled() -> bool {
        agent_file().exists()
    }

    pub fn set_autostart(on: bool) -> std::io::Result<()> {
        let f = agent_file();
        if on {
            fs::create_dir_all(f.parent().unwrap())?;
            fs::write(&f, agent_plist(&std::env::current_exe()?))
        } else {
            remove_files(&[f])
        }
    }

    fn app_dir() -> PathBuf {
        home().join("Applications").join("Petodoro.app")
    }

    /// Build ~/Applications/Petodoro.app around this binary and start it on login.
    pub fn install_self() -> std::io::Result<PathBuf> {
        use std::os::unix::fs::PermissionsExt;
        let contents = app_dir().join("Contents");
        let macos = contents.join("MacOS");
        fs::create_dir_all(&macos)?;
        fs::create_dir_all(contents.join("Resources"))?;
        let bin = macos.join("petodoro");
        let exe = std::env::current_exe()?;
        if exe.canonicalize().ok() != bin.canonicalize().ok() {
            let tmp = bin.with_extension("new");
            fs::copy(&exe, &tmp)?;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))?;
            fs::rename(&tmp, &bin)?;
        }
        fs::write(
            contents.join("Info.plist"),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
                 <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
                 <plist version=\"1.0\"><dict>\n\
                 <key>CFBundleName</key><string>Petodoro</string>\n\
                 <key>CFBundleIdentifier</key><string>com.petodoro</string>\n\
                 <key>CFBundleExecutable</key><string>petodoro</string>\n\
                 <key>CFBundleVersion</key><string>{v}</string>\n\
                 <key>CFBundleShortVersionString</key><string>{v}</string>\n\
                 <key>CFBundlePackageType</key><string>APPL</string>\n\
                 <key>LSUIElement</key><true/>\n\
                 </dict></plist>\n",
                v = env!("CARGO_PKG_VERSION")
            ),
        )?;
        write_icon_png(&contents.join("Resources").join("petodoro.png"))?;
        let f = agent_file();
        fs::create_dir_all(f.parent().unwrap())?;
        fs::write(f, agent_plist(&bin))?;
        Ok(bin)
    }

    pub fn uninstall_self() -> std::io::Result<()> {
        stop_others();
        remove_files(&[agent_file()])?;
        match fs::remove_dir_all(app_dir()) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    pub fn stop_outdated_instances() -> bool {
        false
    }

    fn stop_others() {
        let me = std::process::id();
        let Ok(out) = Command::new("pgrep").args(["-x", "petodoro"]).output() else { return };
        for pid in String::from_utf8_lossy(&out.stdout).lines().filter_map(|l| l.trim().parse::<i32>().ok()) {
            if pid as u32 != me {
                unsafe { libc::kill(pid, libc::SIGTERM) };
            }
        }
    }
}

// ---------------------------------------------------------------- Windows

#[cfg(windows)]
mod os {
    use super::*;

    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    fn ps_quote(s: &str) -> String {
        s.replace('\'', "''").replace(['<', '>', '&'], " ")
    }

    fn powershell(script: &str) -> bool {
        spawn("powershell", &["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", script])
    }

    /// A Windows toast, sent through PowerShell (no extra dependencies).
    pub fn notify(summary: &str, body: &str) {
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
             $x = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
             $t = $x.GetElementsByTagName('text'); \
             $t.Item(0).AppendChild($x.CreateTextNode('{}')) > $null; \
             $t.Item(1).AppendChild($x.CreateTextNode('{}')) > $null; \
             $id = '{{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}}\\WindowsPowerShell\\v1.0\\powershell.exe'; \
             [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($id).Show([Windows.UI.Notifications.ToastNotification]::new($x))",
            ps_quote(summary),
            ps_quote(&body.replace('\n', " "))
        );
        powershell(&script);
    }

    pub fn chime() -> bool {
        false // The caller falls back to the system "asterisk" sound.
    }

    pub fn autostart_enabled() -> bool {
        quiet(Command::new("reg").args(["query", RUN_KEY, "/v", "Petodoro"])).status().is_ok_and(|s| s.success())
    }

    fn set_run_key(exe: &Path) -> std::io::Result<()> {
        let value = format!("\"{}\"", exe.display());
        let ok =
            quiet(Command::new("reg").args(["add", RUN_KEY, "/v", "Petodoro", "/t", "REG_SZ", "/d", &value, "/f"]))
                .status()?
                .success();
        if ok {
            Ok(())
        } else {
            Err(other_error("reg add failed"))
        }
    }

    pub fn set_autostart(on: bool) -> std::io::Result<()> {
        if on {
            set_run_key(&std::env::current_exe()?)
        } else {
            let _ = quiet(Command::new("reg").args(["delete", RUN_KEY, "/v", "Petodoro", "/f"])).status();
            Ok(())
        }
    }

    fn install_dir() -> PathBuf {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(home).join("Programs").join("Petodoro")
    }

    fn shortcut() -> PathBuf {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(home)
            .join(r"Microsoft\Windows\Start Menu\Programs\Petodoro.lnk")
    }

    /// An .ico file that simply wraps a PNG (supported since Windows Vista).
    fn write_ico(path: &Path) -> std::io::Result<()> {
        let png = crate::render::icon(crate::pets::PetKind::Tomo, 256).encode_png().map_err(other_error)?;
        let mut ico = vec![0, 0, 1, 0, 1, 0]; // header: icon, 1 image
        ico.extend_from_slice(&[0, 0, 0, 0, 1, 0, 32, 0]); // 256x256, 32 bpp
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&22u32.to_le_bytes()); // data offset
        ico.extend_from_slice(&png);
        fs::write(path, ico)
    }

    /// Copy to %LOCALAPPDATA%\Programs\Petodoro, add a Start-menu shortcut,
    /// and start on login.
    pub fn install_self() -> std::io::Result<PathBuf> {
        let dir = install_dir();
        fs::create_dir_all(&dir)?;
        let bin = dir.join("petodoro.exe");
        let exe = std::env::current_exe()?;
        if exe.canonicalize().ok() != bin.canonicalize().ok() {
            if bin.exists() {
                // A running exe can't be overwritten, but it can be renamed away.
                let _ = fs::rename(&bin, dir.join("petodoro.old.exe"));
            }
            fs::copy(&exe, &bin)?;
        }
        let ico = dir.join("petodoro.ico");
        write_ico(&ico)?;
        let script = format!(
            "$s = (New-Object -ComObject WScript.Shell).CreateShortcut('{}'); \
             $s.TargetPath = '{}'; $s.IconLocation = '{}'; $s.Description = 'Desktop pet with a pomodoro timer'; $s.Save()",
            ps_quote(&shortcut().display().to_string()),
            ps_quote(&bin.display().to_string()),
            ps_quote(&ico.display().to_string())
        );
        quiet(Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script])).status()?;
        set_run_key(&bin)?;
        Ok(bin)
    }

    pub fn stop_outdated_instances() -> bool {
        false
    }

    pub fn uninstall_self() -> std::io::Result<()> {
        let me = std::process::id().to_string();
        let filter = format!("PID ne {me}");
        let _ = quiet(Command::new("taskkill").args(["/F", "/IM", "petodoro.exe", "/FI", &filter])).status();
        set_autostart(false)?;
        let dir = install_dir();
        remove_files(&[shortcut(), dir.join("petodoro.ico"), dir.join("petodoro.old.exe")])?;
        // We may be running from that folder, so delete it a moment after we
        // exit. The path goes in as a PowerShell literal ('' escaped), never
        // through cmd.exe's quoting rules.
        let script = format!(
            "Start-Sleep -Seconds 2; Remove-Item -LiteralPath '{}' -Recurse -Force -ErrorAction SilentlyContinue",
            dir.display().to_string().replace('\'', "''")
        );
        powershell(&script);
        Ok(())
    }
}

pub use os::*;

// ------------------------------------------------------ process helpers

/// Take an exclusive lock so only one pet runs per user session.
#[cfg(unix)]
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

#[cfg(windows)]
pub fn single_instance() -> bool {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    let name: Vec<u16> = "Local\\PetodoroSingleInstance".encode_utf16().chain([0]).collect();
    unsafe {
        let h = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        !h.is_null() && GetLastError() != ERROR_ALREADY_EXISTS // the handle is kept open on purpose
    }
}

/// Let the kernel reap our fire-and-forget children (notify-send, paplay).
pub fn ignore_children() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGCHLD, libc::SIG_IGN);
    }
}

/// On Windows the app has no console of its own; borrow the terminal's when
/// started from one so `--help` and friends print something.
pub fn attach_console() {
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Today's local date as YYYY-MM-DD.
#[cfg(unix)]
pub fn local_date() -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        format!("{:04}-{:02}-{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday)
    }
}

#[cfg(windows)]
pub fn local_date() -> String {
    unsafe {
        let mut st: windows_sys::Win32::Foundation::SYSTEMTIME = std::mem::zeroed();
        windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut st);
        format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay)
    }
}

#[cfg(all(test, unix, not(target_os = "macos")))]
mod tests {
    use super::*;

    #[test]
    fn desktop_exec_is_quoted() {
        let e = desktop_entry(Path::new("/home/a b/$x`y\"z\\/100%/petodoro"));
        let line = e.lines().find(|l| l.starts_with("Exec=")).unwrap();
        assert_eq!(line, r#"Exec="/home/a b/\\$x\\`y\\"z\\\\/100%%/petodoro""#);
    }
}
