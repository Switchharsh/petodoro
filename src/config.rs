//! Settings persisted to ~/.config/petodoro/config as simple `key = value` lines.

use crate::pets::PetKind;
use crate::timer::PomoSettings;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Pomodoro,
    Stopwatch,
    /// No speech bubble, just the pet.
    PetOnly,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Pomodoro => "pomodoro",
            Mode::Stopwatch => "stopwatch",
            Mode::PetOnly => "pet",
        }
    }

    pub fn from_name(s: &str) -> Option<Mode> {
        [Mode::Pomodoro, Mode::Stopwatch, Mode::PetOnly].into_iter().find(|m| m.name() == s)
    }

    pub fn next(self) -> Mode {
        match self {
            Mode::Pomodoro => Mode::Stopwatch,
            Mode::Stopwatch => Mode::PetOnly,
            Mode::PetOnly => Mode::Pomodoro,
        }
    }
}

pub const SCALES: [(f32, &str); 4] = [(1.0, "S"), (1.5, "M"), (2.0, "L"), (2.5, "XL")];

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub pet: PetKind,
    pub mode: Mode,
    pub scale: f32,
    pub pos: Option<(i32, i32)>,
    pub pomo: PomoSettings,
    pub sound: bool,
    pub notify: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            pet: PetKind::Sakura,
            mode: Mode::Pomodoro,
            scale: 1.5,
            pos: None,
            pomo: PomoSettings::default(),
            sound: true,
            notify: true,
        }
    }
}

pub fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn path() -> PathBuf {
    config_dir().join("petodoro").join("config")
}

impl Config {
    pub fn load() -> Config {
        let mut c = Config::default();
        if let Ok(text) = fs::read_to_string(path()) {
            c.apply(&text);
        }
        c
    }

    fn apply(&mut self, text: &str) {
        let (mut x, mut y) = (None, None);
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let (k, v) = (k.trim(), v.trim());
            let num = || v.parse::<u32>().ok().filter(|n| (1..=600).contains(n));
            let flag = || matches!(v, "true" | "on" | "yes" | "1");
            match k {
                "pet" => self.pet = PetKind::from_name(v).unwrap_or(self.pet),
                "mode" => self.mode = Mode::from_name(v).unwrap_or(self.mode),
                "scale" => {
                    if let Ok(s) = v.parse::<f32>() {
                        self.scale = s.clamp(0.5, 4.0);
                    }
                }
                "x" => x = v.parse().ok(),
                "y" => y = v.parse().ok(),
                "focus_minutes" => self.pomo.focus_min = num().unwrap_or(self.pomo.focus_min),
                "short_break_minutes" => self.pomo.short_min = num().unwrap_or(self.pomo.short_min),
                "long_break_minutes" => self.pomo.long_min = num().unwrap_or(self.pomo.long_min),
                "long_break_every" => self.pomo.long_every = num().unwrap_or(self.pomo.long_every),
                "auto_continue" => self.pomo.auto_continue = flag(),
                "sound" => self.sound = flag(),
                "notify" => self.notify = flag(),
                _ => {}
            }
        }
        if let (Some(x), Some(y)) = (x, y) {
            self.pos = Some((x, y));
        }
    }

    fn serialize(&self) -> String {
        let mut s = format!(
            "# Petodoro settings (also editable from the right-click menu)\n\
             pet = {}\nmode = {}\nscale = {}\n\
             focus_minutes = {}\nshort_break_minutes = {}\nlong_break_minutes = {}\nlong_break_every = {}\n\
             auto_continue = {}\nsound = {}\nnotify = {}\n",
            self.pet.name(),
            self.mode.name(),
            self.scale,
            self.pomo.focus_min,
            self.pomo.short_min,
            self.pomo.long_min,
            self.pomo.long_every,
            self.pomo.auto_continue,
            self.sound,
            self.notify,
        );
        if let Some((x, y)) = self.pos {
            s += &format!("x = {x}\ny = {y}\n");
        }
        s
    }

    pub fn save(&self) {
        let p = path();
        if let Some(dir) = p.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Err(e) = fs::write(&p, self.serialize()) {
            eprintln!("petodoro: could not save {}: {e}", p.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut c = Config::default();
        c.pet = PetKind::Puru;
        c.mode = Mode::Stopwatch;
        c.scale = 2.0;
        c.pos = Some((-5, 300));
        c.pomo.focus_min = 50;
        c.pomo.auto_continue = true;
        c.sound = false;
        let mut back = Config::default();
        back.apply(&c.serialize());
        assert_eq!(back, c);
    }

    #[test]
    fn ignores_garbage() {
        let mut c = Config::default();
        c.apply("pet = nobody\nfocus_minutes = 0\nnonsense\nscale = big\n");
        assert_eq!(c, Config::default());
    }
}
