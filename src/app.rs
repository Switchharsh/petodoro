//! Application state: timers, pet mood, input handling and the menu.

use crate::config::{Config, Mode, SCALES};
use crate::pets::{Mood, Pose};
use crate::platform::{Res, Surface, X};
use crate::render::{self, Bubble, MenuView, Scene};
use crate::system;
use crate::timer::{format_clock, Phase, Pomodoro, Stopwatch};
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;
use x11rb::protocol::Event;

const DRAG_THRESHOLD: i32 = 4;
const SLEEP_AFTER: Duration = Duration::from_secs(180);
const CHEER_FOR: Duration = Duration::from_millis(3500);
const LOVE_FOR: Duration = Duration::from_millis(1800);
const BLINK_FOR: Duration = Duration::from_millis(140);

const FOCUS_CHOICES: [u32; 10] = [10, 15, 20, 25, 30, 40, 45, 50, 60, 90];
const SHORT_CHOICES: [u32; 5] = [3, 5, 7, 10, 15];
const LONG_CHOICES: [u32; 5] = [10, 15, 20, 25, 30];

const ACCENT_FOCUS: u32 = 0xe8475f;
const ACCENT_BREAK: u32 = 0x3fae6a;
const ACCENT_LONG: u32 = 0x3a8fd6;
const ACCENT_WATCH: u32 = 0x8a63d2;

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Toggle,
    Reset,
    Skip,
    Mode,
    Pet,
    Size,
    Focus,
    ShortBreak,
    LongBreak,
    Auto,
    Sound,
    Autostart,
    Quit,
}

impl Action {
    /// Settings that cycle through values keep the menu open.
    fn keeps_menu_open(self) -> bool {
        !matches!(self, Action::Toggle | Action::Reset | Action::Skip | Action::Quit)
    }
}

struct Menu {
    surf: Surface,
    pm: Pixmap,
    view: MenuView,
    actions: Vec<Action>,
    at: (i32, i32),
    opened: Instant,
    pressed_inside: bool,
    dirty: bool,
}

struct Press {
    root: (i32, i32),
    win_origin: (i32, i32),
}

pub struct App {
    pub x: X,
    surf: Surface,
    pm: Pixmap,
    pets: render::PetCache,
    cfg: Config,
    pomo: Pomodoro,
    sw: Stopwatch,
    pos: (i32, i32),
    press: Option<Press>,
    dragging: bool,
    cheer_until: Option<Instant>,
    love_until: Option<Instant>,
    /// Shown in the bubble after a phase ends, until the user reacts.
    attention: Option<&'static str>,
    last_activity: Instant,
    next_blink: Instant,
    blink_until: Instant,
    rng: u32,
    mood: Mood,
    mood_since: Instant,
    last_scene: Option<Scene>,
    dirty: bool,
    menu: Option<Menu>,
    pub quit: bool,
}

fn pick_next<T: Copy + PartialEq + PartialOrd>(choices: &[T], cur: T, forward: bool) -> T {
    if forward {
        choices.iter().copied().find(|&c| c > cur).unwrap_or(choices[0])
    } else {
        choices.iter().rev().copied().find(|&c| c < cur).unwrap_or(choices[choices.len() - 1])
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

impl App {
    pub fn new(x: X, cfg: Config) -> Res<App> {
        let (w, h) = render::window_size(cfg.scale);
        let default = (x.screen_w - w as i32 - 60, x.screen_h - h as i32 - 90);
        let pos = clamp_pos(&x, cfg.pos.unwrap_or(default), w, h);
        let surf = x.create_pet_window(pos.0, pos.1, w, h)?;
        let now = Instant::now();
        Ok(App {
            pm: Pixmap::new(w as u32, h as u32).ok_or("bad window size")?,
            pets: render::PetCache::default(),
            surf,
            x,
            pomo: Pomodoro::new(&cfg.pomo),
            sw: Stopwatch::default(),
            cfg,
            pos,
            press: None,
            dragging: false,
            cheer_until: None,
            love_until: Some(now + LOVE_FOR), // Say hi on startup.
            attention: None,
            last_activity: now,
            next_blink: now + Duration::from_secs(2),
            blink_until: now,
            rng: std::process::id() | 1,
            mood: Mood::Idle,
            mood_since: now,
            last_scene: None,
            dirty: true,
            menu: None,
            quit: false,
        })
    }

    fn rand(&mut self) -> u32 {
        // xorshift32: plenty for blink timing.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    // ------------------------------------------------------------ events

    pub fn handle(&mut self, ev: Event) -> Res<()> {
        let menu_win = self.menu.as_ref().map(|m| m.surf.win);
        match ev {
            Event::Expose(e) if Some(e.window) == menu_win => self.menu.as_mut().unwrap().dirty = true,
            Event::Expose(_) => self.dirty = true,
            Event::ButtonPress(e) if Some(e.event) == menu_win => {
                self.menu_press(e.event_x as i32, e.event_y as i32)?
            }
            Event::ButtonRelease(e) if Some(e.event) == menu_win => {
                self.menu_release(e.event_x as i32, e.event_y as i32, e.detail)?
            }
            Event::MotionNotify(e) if Some(e.event) == menu_win => self.menu_motion(e.event_x as i32, e.event_y as i32),
            Event::ButtonPress(e) if e.event == self.surf.win => {
                let now = Instant::now();
                self.last_activity = now;
                match e.detail {
                    1 => {
                        self.press = Some(Press {
                            root: (e.root_x as i32, e.root_y as i32),
                            win_origin: (e.root_x as i32 - e.event_x as i32, e.root_y as i32 - e.event_y as i32),
                        });
                    }
                    2 => self.act(Action::Reset, true)?,
                    3 => self.open_menu(e.root_x as i32, e.root_y as i32)?,
                    4 | 5 => self.scroll(e.detail == 4),
                    _ => {}
                }
            }
            Event::MotionNotify(e) if e.event == self.surf.win => {
                if let Some(p) = &self.press {
                    let (dx, dy) = (e.root_x as i32 - p.root.0, e.root_y as i32 - p.root.1);
                    if !self.dragging && dx.abs().max(dy.abs()) > DRAG_THRESHOLD {
                        self.dragging = true;
                    }
                    if self.dragging {
                        self.pos = (p.win_origin.0 + dx, p.win_origin.1 + dy);
                        self.x.move_window(&self.surf, self.pos.0, self.pos.1)?;
                    }
                }
            }
            Event::ButtonRelease(e) if e.event == self.surf.win && e.detail == 1 => {
                if self.press.take().is_some() {
                    if self.dragging {
                        self.dragging = false;
                        let clamped = clamp_pos(&self.x, self.pos, self.surf.w, self.surf.h);
                        if clamped != self.pos {
                            self.pos = clamped;
                            self.x.move_window(&self.surf, clamped.0, clamped.1)?;
                        }
                        self.cfg.pos = Some(self.pos);
                        self.cfg.save();
                    } else {
                        self.click(e.event_x as i32, e.event_y as i32);
                    }
                }
            }
            Event::ClientMessage(e) => {
                let a = &self.x.atoms;
                if e.type_ == a.WM_PROTOCOLS && e.format == 32 && e.data.as_data32()[0] == a.WM_DELETE_WINDOW {
                    self.quit = true;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn click(&mut self, x: i32, y: i32) {
        let (bx, by, bw, bh) = render::bubble_rect(self.cfg.scale);
        let on_bubble = self.cfg.mode != Mode::PetOnly && x >= bx && x < bx + bw && y >= by && y < by + bh;
        if on_bubble {
            let _ = self.act(Action::Toggle, true);
        } else {
            self.love_until = Some(Instant::now() + LOVE_FOR);
        }
    }

    /// Mouse wheel: adjust the length of the current (not yet started) phase.
    fn scroll(&mut self, up: bool) {
        if self.cfg.mode != Mode::Pomodoro || !self.pomo.is_fresh(&self.cfg.pomo) {
            return;
        }
        let m = self.cfg.pomo.minutes_mut(self.pomo.phase);
        *m = if up { (*m + 1).min(180) } else { m.saturating_sub(1).max(1) };
        self.pomo.reset(&self.cfg.pomo);
        self.cfg.save();
    }

    fn act(&mut self, a: Action, forward: bool) -> Res<()> {
        let now = Instant::now();
        self.last_activity = now;
        let was_fresh = self.pomo.is_fresh(&self.cfg.pomo);
        match a {
            Action::Toggle => {
                self.attention = None;
                match self.cfg.mode {
                    Mode::Pomodoro => self.pomo.toggle(now),
                    Mode::Stopwatch => self.sw.toggle(now),
                    Mode::PetOnly => {}
                }
            }
            Action::Reset => {
                self.attention = None;
                match self.cfg.mode {
                    Mode::Pomodoro => self.pomo.reset(&self.cfg.pomo),
                    Mode::Stopwatch => self.sw.reset(),
                    Mode::PetOnly => {}
                }
            }
            Action::Skip => {
                self.attention = None;
                self.pomo.skip(&self.cfg.pomo);
            }
            Action::Mode => self.cfg.mode = self.cfg.mode.next(),
            Action::Pet => {
                self.cfg.pet = self.cfg.pet.cycle(forward);
                self.love_until = Some(now + LOVE_FOR);
            }
            Action::Size => {
                let i = SCALES.iter().position(|(s, _)| *s >= self.cfg.scale).unwrap_or(0);
                let n = SCALES.len();
                self.set_scale(SCALES[if forward { (i + 1) % n } else { (i + n - 1) % n }].0)?;
            }
            Action::Focus => self.cfg.pomo.focus_min = pick_next(&FOCUS_CHOICES, self.cfg.pomo.focus_min, forward),
            Action::ShortBreak => self.cfg.pomo.short_min = pick_next(&SHORT_CHOICES, self.cfg.pomo.short_min, forward),
            Action::LongBreak => self.cfg.pomo.long_min = pick_next(&LONG_CHOICES, self.cfg.pomo.long_min, forward),
            Action::Auto => self.cfg.pomo.auto_continue = !self.cfg.pomo.auto_continue,
            Action::Sound => self.cfg.sound = !self.cfg.sound,
            Action::Autostart => {
                if let Err(e) = system::set_autostart(!system::autostart_enabled()) {
                    eprintln!("petodoro: autostart: {e}");
                }
            }
            Action::Quit => self.quit = true,
        }
        if matches!(a, Action::Focus | Action::ShortBreak | Action::LongBreak) && was_fresh {
            // Apply the new length right away if the phase hasn't started.
            self.pomo.reset(&self.cfg.pomo);
        }
        if a.keeps_menu_open() {
            self.cfg.save();
        }
        Ok(())
    }

    fn set_scale(&mut self, scale: f32) -> Res<()> {
        let (w, h) = render::window_size(scale);
        // Keep the pet's feet where they were.
        let x = self.pos.0 + (self.surf.w as i32 - w as i32) / 2;
        let y = self.pos.1 + self.surf.h as i32 - h as i32;
        self.pos = clamp_pos(&self.x, (x, y), w, h);
        self.cfg.scale = scale;
        self.cfg.pos = Some(self.pos);
        self.x.move_resize(&mut self.surf, self.pos.0, self.pos.1, w, h)?;
        self.pm = Pixmap::new(w as u32, h as u32).ok_or("bad window size")?;
        self.dirty = true;
        Ok(())
    }

    // -------------------------------------------------------------- menu

    fn menu_items(&self) -> (Vec<(String, bool)>, Vec<Action>) {
        let c = &self.cfg;
        let on_off = |b: bool| if b { "ON" } else { "OFF" };
        let mut items: Vec<(String, bool, Action)> = vec![];
        match c.mode {
            Mode::Pomodoro => {
                let p = &self.pomo;
                let what = if p.phase.is_break() { "BREAK" } else { "FOCUS" };
                let toggle = if p.is_running() {
                    "PAUSE".to_string()
                } else if p.is_fresh(&c.pomo) {
                    format!("START {what}")
                } else {
                    "RESUME".to_string()
                };
                items.push((toggle, false, Action::Toggle));
                items.push(("RESET".into(), false, Action::Reset));
                let skip = if p.phase.is_break() { "SKIP TO FOCUS" } else { "SKIP TO BREAK" };
                items.push((skip.into(), false, Action::Skip));
            }
            Mode::Stopwatch => {
                let toggle = if self.sw.is_running() {
                    "PAUSE"
                } else if self.sw.elapsed(Instant::now()).is_zero() {
                    "START"
                } else {
                    "RESUME"
                };
                items.push((toggle.into(), false, Action::Toggle));
                items.push(("RESET".into(), false, Action::Reset));
            }
            Mode::PetOnly => {}
        }
        let first = items.is_empty();
        let mode = match c.mode {
            Mode::Pomodoro => "POMODORO",
            Mode::Stopwatch => "STOPWATCH",
            Mode::PetOnly => "JUST PET",
        };
        items.push((format!("MODE: {mode}"), !first, Action::Mode));
        items.push((format!("PET: {}", c.pet.name().to_uppercase()), false, Action::Pet));
        let size = SCALES.iter().find(|(s, _)| *s >= c.scale).map_or("M", |(_, n)| n);
        items.push((format!("SIZE: {size}"), false, Action::Size));
        if c.mode == Mode::Pomodoro {
            items.push((format!("FOCUS: {} MIN", c.pomo.focus_min), true, Action::Focus));
            items.push((format!("BREAK: {} MIN", c.pomo.short_min), false, Action::ShortBreak));
            items.push((format!("LONG BREAK: {} MIN", c.pomo.long_min), false, Action::LongBreak));
            items.push((format!("AUTO-START NEXT: {}", on_off(c.pomo.auto_continue)), false, Action::Auto));
        }
        items.push((format!("SOUND: {}", on_off(c.sound)), true, Action::Sound));
        items.push((format!("START ON LOGIN: {}", on_off(system::autostart_enabled())), false, Action::Autostart));
        items.push(("QUIT".into(), true, Action::Quit));
        let actions = items.iter().map(|i| i.2).collect();
        (items.into_iter().map(|(l, s, _)| (l, s)).collect(), actions)
    }

    fn menu_accent(&self) -> u32 {
        match self.cfg.mode {
            Mode::Pomodoro => match self.pomo.phase {
                Phase::Focus => ACCENT_FOCUS,
                Phase::ShortBreak => ACCENT_BREAK,
                Phase::LongBreak => ACCENT_LONG,
            },
            Mode::Stopwatch => ACCENT_WATCH,
            Mode::PetOnly => ACCENT_FOCUS,
        }
    }

    fn menu_origin(&self, rx: i32, ry: i32, w: u16, h: u16) -> (i32, i32) {
        let x = if rx + w as i32 > self.x.screen_w { rx - w as i32 } else { rx + 1 };
        let y = if ry + h as i32 > self.x.screen_h { (ry - h as i32).max(0) } else { ry + 1 };
        (x.max(0), y)
    }

    fn open_menu(&mut self, rx: i32, ry: i32) -> Res<()> {
        self.close_menu()?;
        let (items, actions) = self.menu_items();
        let view = MenuView { title: "PETODORO".into(), items, hover: None, accent: self.menu_accent() };
        let (w, h) = render::menu_size(&view);
        let (x, y) = self.menu_origin(rx, ry, w, h);
        let surf = self.x.create_menu_window(x, y, w, h)?;
        self.menu = Some(Menu {
            surf,
            pm: Pixmap::new(w as u32, h as u32).ok_or("bad menu size")?,
            view,
            actions,
            at: (x, y),
            opened: Instant::now(),
            pressed_inside: false,
            dirty: true,
        });
        Ok(())
    }

    fn close_menu(&mut self) -> Res<()> {
        if let Some(m) = self.menu.take() {
            self.x.destroy(m.surf)?;
        }
        Ok(())
    }

    fn refresh_menu(&mut self) -> Res<()> {
        let (items, actions) = self.menu_items();
        let accent = self.menu_accent();
        let Some(m) = self.menu.as_mut() else { return Ok(()) };
        m.view.items = items;
        m.view.accent = accent;
        m.actions = actions;
        m.dirty = true;
        let (w, h) = render::menu_size(&m.view);
        if (w, h) != (m.surf.w, m.surf.h) {
            self.x.move_resize(&mut m.surf, m.at.0, m.at.1, w, h)?;
            m.pm = Pixmap::new(w as u32, h as u32).ok_or("bad menu size")?;
        }
        Ok(())
    }

    fn menu_inside(&self, x: i32, y: i32) -> bool {
        let m = self.menu.as_ref().unwrap();
        x >= 0 && y >= 0 && x < m.surf.w as i32 && y < m.surf.h as i32
    }

    fn menu_press(&mut self, x: i32, y: i32) -> Res<()> {
        if self.menu_inside(x, y) {
            self.menu.as_mut().unwrap().pressed_inside = true;
            Ok(())
        } else {
            self.close_menu()
        }
    }

    fn menu_release(&mut self, x: i32, y: i32, button: u8) -> Res<()> {
        let m = self.menu.as_ref().unwrap();
        // Ignore the release of the right-click that opened the menu,
        // unless the user dragged onto an item (press-drag-release style).
        let armed = m.pressed_inside || m.opened.elapsed() > Duration::from_millis(300);
        if !armed || !matches!(button, 1 | 3) {
            return Ok(());
        }
        let Some(i) = render::menu_item_at(&m.view, x, y) else { return Ok(()) };
        let action = m.actions[i];
        self.menu.as_mut().unwrap().pressed_inside = false;
        if action.keeps_menu_open() {
            self.act(action, button == 1)?;
            self.refresh_menu()
        } else {
            self.close_menu()?;
            self.act(action, true)
        }
    }

    fn menu_motion(&mut self, x: i32, y: i32) {
        let m = self.menu.as_mut().unwrap();
        let hover = render::menu_item_at(&m.view, x, y);
        if hover != m.view.hover {
            m.view.hover = hover;
            m.dirty = true;
        }
    }

    // ------------------------------------------------------- time & draw

    pub fn update(&mut self, now: Instant) {
        if let Some(done) = self.pomo.tick(&self.cfg.pomo, now) {
            self.phase_done(done, now);
        }
        if now >= self.next_blink {
            self.blink_until = now + BLINK_FOR;
            self.next_blink = now + Duration::from_millis(2500 + (self.rand() % 4000) as u64);
        }
        let mood = self.compute_mood(now);
        if mood != self.mood {
            self.mood = mood;
            self.mood_since = now;
        }
    }

    fn phase_done(&mut self, done: Phase, now: Instant) {
        let pet = capitalize(self.cfg.pet.name());
        let next = self.pomo.phase;
        let mins = self.cfg.pomo.minutes(next);
        let (title, body) = if done == Phase::Focus {
            ("Focus session done!", format!("{pet} says: take a {mins} minute break~"))
        } else {
            ("Break's over!", format!("{pet} is ready for {mins} more minutes of focus!"))
        };
        if self.cfg.notify {
            system::notify(title, &body);
        }
        if self.cfg.sound && !system::chime() {
            self.x.bell();
        }
        self.cheer_until = Some(now + CHEER_FOR);
        self.attention = if self.cfg.pomo.auto_continue {
            None
        } else if next.is_break() {
            Some("BREAK TIME!")
        } else {
            Some("FOCUS TIME!")
        };
        self.last_activity = now;
    }

    fn compute_mood(&self, now: Instant) -> Mood {
        let active = |t: Option<Instant>| t.is_some_and(|t| now < t);
        if self.dragging {
            Mood::Held
        } else if active(self.cheer_until) {
            Mood::Cheer
        } else if active(self.love_until) {
            Mood::Love
        } else if self.pomo.is_running() && self.cfg.mode != Mode::Stopwatch {
            if self.pomo.phase.is_break() {
                Mood::Relax
            } else {
                Mood::Focus
            }
        } else if self.sw.is_running() && self.cfg.mode != Mode::Pomodoro {
            Mood::Focus
        } else if self.attention.is_none() && now.duration_since(self.last_activity) > SLEEP_AFTER {
            Mood::Sleep
        } else {
            Mood::Idle
        }
    }

    fn pose(&self, now: Instant) -> Pose {
        let t = now.duration_since(self.mood_since).as_secs_f32();
        let frame = (t / self.mood.frame_secs()) as u32 % self.mood.cycle();
        Pose { mood: self.mood, frame, blink: self.mood.blinks() && now < self.blink_until }
    }

    fn bubble(&self, now: Instant) -> Option<Bubble> {
        match self.cfg.mode {
            Mode::PetOnly => None,
            Mode::Pomodoro => {
                let (p, s) = (&self.pomo, &self.cfg.pomo);
                let running = p.is_running();
                let label = match (self.attention, p.phase) {
                    (Some(a), _) => a.to_string(),
                    _ if !running && !p.is_fresh(s) => "PAUSED".into(),
                    (None, Phase::Focus) => format!("FOCUS {}/{}", p.session_in_cycle(s), s.long_every),
                    (None, Phase::ShortBreak) => "BREAK".into(),
                    (None, Phase::LongBreak) => "LONG BREAK".into(),
                };
                let accent = match p.phase {
                    Phase::Focus => ACCENT_FOCUS,
                    Phase::ShortBreak => ACCENT_BREAK,
                    Phase::LongBreak => ACCENT_LONG,
                };
                Some(Bubble {
                    label,
                    time: format_clock(p.remaining(now), true),
                    // Quantized so the bar alone doesn't force a redraw every tick.
                    progress: Some((p.progress(s, now) * 200.0) as u16 * 5),
                    accent,
                    paused: !running && self.attention.is_none(),
                })
            }
            Mode::Stopwatch => {
                let elapsed = self.sw.elapsed(now);
                let running = self.sw.is_running();
                Some(Bubble {
                    label: if !running && !elapsed.is_zero() { "PAUSED".into() } else { "STOPWATCH".into() },
                    time: format_clock(elapsed, false),
                    progress: None,
                    accent: ACCENT_WATCH,
                    paused: !running,
                })
            }
        }
    }

    pub fn render(&mut self, now: Instant) -> Res<()> {
        let scene = Scene { scale: self.cfg.scale, pet: self.cfg.pet, pose: self.pose(now), bubble: self.bubble(now) };
        if self.dirty || self.last_scene.as_ref() != Some(&scene) {
            render::draw_scene(&mut self.pm, &scene, &mut self.pets);
            self.x.present(&mut self.surf, &self.pm)?;
            self.last_scene = Some(scene);
            self.dirty = false;
        }
        if let Some(m) = self.menu.as_mut().filter(|m| m.dirty) {
            render::draw_menu(&mut m.pm, &m.view);
            self.x.present(&mut m.surf, &m.pm)?;
            m.dirty = false;
        }
        Ok(())
    }

    /// How long the event loop may sleep before something on screen changes.
    pub fn next_timeout(&self, now: Instant) -> Duration {
        let mut t = Duration::from_secs(10);
        let fs = self.mood.frame_secs();
        let into = now.duration_since(self.mood_since).as_secs_f32() % fs;
        t = t.min(Duration::from_secs_f32(fs - into));
        if self.mood.blinks() {
            let at = if self.blink_until > now { self.blink_until } else { self.next_blink };
            t = t.min(at.saturating_duration_since(now));
        }
        if self.pomo.is_running() {
            let ms = self.pomo.remaining(now).as_millis() as u64;
            t = t.min(Duration::from_millis(ms % 1000 + 1));
        }
        if self.sw.is_running() {
            let ms = self.sw.elapsed(now).as_millis() as u64;
            t = t.min(Duration::from_millis(1000 - ms % 1000));
        }
        for until in [self.cheer_until, self.love_until].into_iter().flatten() {
            if until > now {
                t = t.min(until - now);
            }
        }
        if self.mood == Mood::Idle && self.attention.is_none() {
            let sleep_at = self.last_activity + SLEEP_AFTER;
            if sleep_at > now {
                t = t.min(sleep_at - now + Duration::from_millis(5));
            }
        }
        t.max(Duration::from_millis(5))
    }
}

/// Keep at least part of the window on screen so the pet can't get lost.
fn clamp_pos(x: &X, (px, py): (i32, i32), w: u16, h: u16) -> (i32, i32) {
    let (w, h) = (w as i32, h as i32);
    let keep = 48;
    (px.clamp(keep - w, x.screen_w - keep), py.clamp(-h / 2, x.screen_h - keep))
}
