//! Settings persisted to ~/.config/petodoro/config as simple `key = value` lines.

use crate::pets::{PetKind, Theme};
use crate::timer::PomoSettings;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Pomodoro,
    Stopwatch,
    /// No timer bubble, just the pet (it still talks).
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

/// Bubble / menu theme: follow the pet, or force one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeChoice {
    Auto,
    Light,
    Dark,
    Amoled,
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 4] = [ThemeChoice::Auto, ThemeChoice::Light, ThemeChoice::Dark, ThemeChoice::Amoled];

    pub fn name(self) -> &'static str {
        match self {
            ThemeChoice::Auto => "auto",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
            ThemeChoice::Amoled => "amoled",
        }
    }

    pub fn from_name(s: &str) -> Option<ThemeChoice> {
        Self::ALL.into_iter().find(|t| t.name() == s)
    }

    pub fn cycle(self, forward: bool) -> ThemeChoice {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        let n = Self::ALL.len();
        Self::ALL[if forward { (i + 1) % n } else { (i + n - 1) % n }]
    }

    pub fn resolve(self, pet: PetKind) -> Theme {
        match self {
            ThemeChoice::Auto => pet.theme(),
            ThemeChoice::Light => Theme::Light,
            ThemeChoice::Dark => Theme::Dark,
            ThemeChoice::Amoled => Theme::Amoled,
        }
    }
}

pub const SCALES: [(f32, &str); 4] = [(1.0, "S"), (1.5, "M"), (2.0, "L"), (2.5, "XL")];

/// Focus stats for one day.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Today {
    pub date: String,
    pub pomodoros: u32,
    pub focus_minutes: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub pet: PetKind,
    pub mode: Mode,
    pub scale: f32,
    pub pos: Option<(i32, i32)>,
    pub pomo: PomoSettings,
    pub sound: bool,
    pub notify: bool,
    pub theme: ThemeChoice,
    /// Wander around the bottom of the screen when idle.
    pub wander: bool,
    /// Random chatter while idle.
    pub chatter: bool,
    /// Friendship points per pet name.
    pub friendship: BTreeMap<String, u32>,
    pub today: Today,
    pub total_pomodoros: u32,
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
            theme: ThemeChoice::Auto,
            wander: true,
            chatter: true,
            friendship: BTreeMap::new(),
            today: Today::default(),
            total_pomodoros: 0,
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

/// Friendship level from points: 1, 2 at 10, 3 at 40, 4 at 90, 5 at 160...
pub fn level(points: u32) -> u32 {
    ((points as f32 / 10.0).sqrt()) as u32 + 1
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
            let count = || v.parse::<u32>().ok();
            let flag = || matches!(v, "true" | "on" | "yes" | "1");
            if let Some(pet) = k.strip_prefix("friendship.") {
                if let (Some(_), Some(n)) = (PetKind::from_name(pet), count()) {
                    self.friendship.insert(pet.to_string(), n);
                }
                continue;
            }
            match k {
                "pet" => self.pet = PetKind::from_name(v).unwrap_or(self.pet),
                "mode" => self.mode = Mode::from_name(v).unwrap_or(self.mode),
                "theme" => self.theme = ThemeChoice::from_name(v).unwrap_or(self.theme),
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
                "wander" => self.wander = flag(),
                "chatter" => self.chatter = flag(),
                "today" => self.today.date = v.to_string(),
                "today_pomodoros" => self.today.pomodoros = count().unwrap_or(0),
                "today_focus_minutes" => self.today.focus_minutes = count().unwrap_or(0),
                "total_pomodoros" => self.total_pomodoros = count().unwrap_or(0),
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
             pet = {}\nmode = {}\ntheme = {}\nscale = {}\n\
             focus_minutes = {}\nshort_break_minutes = {}\nlong_break_minutes = {}\nlong_break_every = {}\n\
             auto_continue = {}\nsound = {}\nnotify = {}\nwander = {}\nchatter = {}\n",
            self.pet.name(),
            self.mode.name(),
            self.theme.name(),
            self.scale,
            self.pomo.focus_min,
            self.pomo.short_min,
            self.pomo.long_min,
            self.pomo.long_every,
            self.pomo.auto_continue,
            self.sound,
            self.notify,
            self.wander,
            self.chatter,
        );
        if let Some((x, y)) = self.pos {
            s += &format!("x = {x}\ny = {y}\n");
        }
        s += "\n# Stats and friendship (managed by petodoro)\n";
        if !self.today.date.is_empty() {
            s += &format!(
                "today = {}\ntoday_pomodoros = {}\ntoday_focus_minutes = {}\n",
                self.today.date, self.today.pomodoros, self.today.focus_minutes
            );
        }
        s += &format!("total_pomodoros = {}\n", self.total_pomodoros);
        for (pet, n) in &self.friendship {
            s += &format!("friendship.{pet} = {n}\n");
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

    pub fn friendship(&self, pet: PetKind) -> u32 {
        self.friendship.get(pet.name()).copied().unwrap_or(0)
    }

    pub fn add_friendship(&mut self, pet: PetKind, n: u32) {
        *self.friendship.entry(pet.name().to_string()).or_insert(0) += n;
    }

    /// Reset the daily stats if the date changed.
    pub fn roll_day(&mut self, date: &str) {
        if self.today.date != date {
            self.today = Today { date: date.to_string(), ..Today::default() };
        }
    }
}

/// Today's local date as YYYY-MM-DD.
pub fn local_date() -> String {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        format!("{:04}-{:02}-{:02}", tm.tm_year + 1900, tm.tm_mon + 1, tm.tm_mday)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut c = Config::default();
        c.pet = PetKind::Kuro;
        c.mode = Mode::Stopwatch;
        c.theme = ThemeChoice::Amoled;
        c.scale = 2.0;
        c.pos = Some((-5, 300));
        c.pomo.focus_min = 50;
        c.pomo.auto_continue = true;
        c.sound = false;
        c.wander = false;
        c.add_friendship(PetKind::Rin, 42);
        c.today = Today { date: "2026-10-26".into(), pomodoros: 3, focus_minutes: 75 };
        c.total_pomodoros = 9;
        let mut back = Config::default();
        back.apply(&c.serialize());
        assert_eq!(back, c);
    }

    #[test]
    fn ignores_garbage() {
        let mut c = Config::default();
        c.apply("pet = nobody\nfocus_minutes = 0\nnonsense\nscale = big\nfriendship.nobody = 3\ntheme = neon\n");
        assert_eq!(c, Config::default());
    }

    #[test]
    fn levels() {
        assert_eq!(level(0), 1);
        assert_eq!(level(10), 2);
        assert_eq!(level(39), 2);
        assert_eq!(level(40), 3);
        assert_eq!(level(90), 4);
    }

    #[test]
    fn day_rollover() {
        let mut c = Config::default();
        c.roll_day("2026-10-26");
        c.today.pomodoros = 2;
        c.roll_day("2026-10-26");
        assert_eq!(c.today.pomodoros, 2);
        c.roll_day("2026-10-27");
        assert_eq!(c.today.pomodoros, 0);
    }
}
