//! Application state: timers, pet mood, input handling and the menu.

use crate::config::{level, local_date, Config, Mode, SCALES};
use crate::lines::{self, Event as Say};
use crate::pets::{Mood, Persona, PetKind, Pose};
use crate::platform::{Display, Res, Surface, WinEvent};
use crate::render::{self, Bubble, MenuView, Scene};
use crate::system;
use crate::timer::{format_clock, Phase, Pomodoro, Stopwatch};
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;

const DRAG_THRESHOLD: i32 = 4;
const CHEER_FOR: Duration = Duration::from_millis(3500);
const LOVE_FOR: Duration = Duration::from_millis(1800);
const DIZZY_FOR: Duration = Duration::from_millis(2600);
const BLINK_FOR: Duration = Duration::from_millis(140);
const COMBO_WINDOW: Duration = Duration::from_millis(700);
const SNACK_COOLDOWN: Duration = Duration::from_secs(45);
/// Horizontal swing (px) that counts as one shake.
const SHAKE_PX: i32 = 25;

const FOCUS_CHOICES: [u32; 11] = [5, 10, 15, 20, 25, 30, 40, 45, 50, 60, 90];
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
    Snack,
    Stats,
    Mode,
    Pet(PetKind),
    Size,
    Theme,
    Focus,
    ShortBreak,
    LongBreak,
    Auto,
    Wander,
    Chatter,
    Sound,
    Autostart,
    Quit,
    Page(Page),
}

/// Which page of the right-click menu is showing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Main,
    Settings,
    Pets,
}

impl Action {
    /// Settings that cycle through values keep the menu open.
    fn keeps_menu_open(self) -> bool {
        !matches!(self, Action::Toggle | Action::Reset | Action::Skip | Action::Snack | Action::Quit)
    }
}

struct Menu {
    page: Page,
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

/// Detects the pet being shaken back and forth while dragged.
#[derive(Default)]
struct Shake {
    dir: i32,
    extreme: i32,
    flips: Vec<Instant>,
    shaken: bool,
}

impl Shake {
    fn track(&mut self, x: i32, now: Instant) {
        if self.dir == 0 {
            if (x - self.extreme).abs() > SHAKE_PX {
                self.dir = (x - self.extreme).signum();
                self.extreme = x;
            }
        } else if (x - self.extreme) * self.dir > 0 {
            self.extreme = x;
        } else if (self.extreme - x) * self.dir > SHAKE_PX {
            self.flips.push(now);
            self.dir = -self.dir;
            self.extreme = x;
        }
        self.flips.retain(|t| now.duration_since(*t) < Duration::from_millis(1500));
        if self.flips.len() >= 4 {
            self.shaken = true;
        }
    }
}

struct Walk {
    dir: i32,
    left: f32,
    x: f32,
    last: Instant,
}

pub struct App {
    pub dpy: Display,
    surf: Surface,
    pm: Pixmap,
    pets: render::PetCache,
    cfg: Config,
    pomo: Pomodoro,
    sw: Stopwatch,
    pos: (i32, i32),
    press: Option<Press>,
    dragging: bool,
    shake: Shake,
    cheer_until: Option<Instant>,
    love_until: Option<Instant>,
    dizzy_until: Option<Instant>,
    eat_until: Option<Instant>,
    last_snack: Option<Instant>,
    last_pat: Option<Instant>,
    last_friend_pat: Option<Instant>,
    combo: u32,
    speech: Option<(String, Instant)>,
    walk: Option<Walk>,
    next_walk: Instant,
    next_chatter: Instant,
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

fn secs(n: u64) -> Duration {
    Duration::from_secs(n)
}

impl App {
    pub fn new(dpy: Display, cfg: Config) -> Res<App> {
        let (w, h) = render::window_size(cfg.scale);
        let default = (dpy.work.right() - w as i32 - 60, dpy.work.bottom() - h as i32 - 40);
        let pos = clamp_pos(&dpy, cfg.pos.unwrap_or(default), w, h);
        let surf = dpy.create_pet_window(pos.0, pos.1, w, h)?;
        let now = Instant::now();
        let mut app = App {
            pm: Pixmap::new(w as u32, h as u32).ok_or("bad window size")?,
            pets: render::PetCache::default(),
            surf,
            dpy,
            pomo: Pomodoro::new(&cfg.pomo),
            sw: Stopwatch::default(),
            cfg,
            pos,
            press: None,
            dragging: false,
            shake: Shake::default(),
            cheer_until: None,
            love_until: Some(now + LOVE_FOR),
            dizzy_until: None,
            eat_until: None,
            last_snack: None,
            last_pat: None,
            last_friend_pat: None,
            combo: 0,
            speech: None,
            walk: None,
            next_walk: now + secs(30),
            next_chatter: now + secs(240),
            attention: None,
            last_activity: now,
            next_blink: now + secs(2),
            blink_until: now,
            rng: (std::process::id() ^ 0x9e37_79b9) | 1,
            mood: Mood::Idle,
            mood_since: now,
            last_scene: None,
            dirty: true,
            menu: None,
            quit: false,
        };
        app.say_event(Say::Hello, now);
        app.cfg.roll_day(&local_date());
        Ok(app)
    }

    fn rand(&mut self) -> u32 {
        // xorshift32: plenty for blinks and chatter.
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    /// Random duration in [lo, hi) seconds.
    fn rand_secs(&mut self, lo: u64, hi: u64) -> Duration {
        Duration::from_millis(lo * 1000 + (self.rand() as u64 % ((hi - lo).max(1) * 1000)))
    }

    fn persona(&self) -> Persona {
        self.cfg.pet.persona()
    }

    fn say(&mut self, text: String, now: Instant) {
        let len = text.chars().count() as u64;
        self.speech = Some((text, now + Duration::from_millis((2200 + len * 70).min(6000))));
    }

    fn say_event(&mut self, e: Say, now: Instant) {
        let r = self.rand();
        self.say(lines::pick(self.persona(), e, r).to_string(), now);
    }

    // ------------------------------------------------------------ events

    pub fn handle(&mut self, ev: WinEvent) -> Res<()> {
        let menu_win = self.menu.as_ref().map(|m| m.surf.id());
        let pet_win = self.surf.id();
        match ev {
            WinEvent::Expose(w) if Some(w) == menu_win => self.menu.as_mut().unwrap().dirty = true,
            WinEvent::Expose(_) => self.dirty = true,
            WinEvent::FocusLost(w) if Some(w) == menu_win => self.close_menu()?,
            WinEvent::Press { win, x, y, .. } if Some(win) == menu_win => self.menu_press(x, y)?,
            WinEvent::Release { win, button, x, y, .. } if Some(win) == menu_win => self.menu_release(x, y, button)?,
            WinEvent::Motion { win, x, y, .. } if Some(win) == menu_win => self.menu_motion(x, y),
            WinEvent::Press { win, button, x, y, rx, ry } if win == pet_win => {
                let now = Instant::now();
                match button {
                    1 => {
                        self.end_walk();
                        self.press = Some(Press { root: (rx, ry), win_origin: (rx - x, ry - y) });
                        self.shake = Shake { extreme: rx, ..Shake::default() };
                    }
                    2 => self.act(Action::Reset, true)?,
                    3 => self.open_menu(rx, ry)?,
                    4 | 5 => self.scroll(button == 4),
                    _ => {}
                }
                self.last_activity = now;
            }
            WinEvent::Motion { win, rx, ry, .. } if win == pet_win => {
                if let Some(p) = &self.press {
                    let (dx, dy) = (rx - p.root.0, ry - p.root.1);
                    if !self.dragging && dx.abs().max(dy.abs()) > DRAG_THRESHOLD {
                        self.dragging = true;
                    }
                    if self.dragging {
                        self.pos = (p.win_origin.0 + dx, p.win_origin.1 + dy);
                        self.dpy.move_window(&self.surf, self.pos.0, self.pos.1)?;
                        self.shake.track(rx, Instant::now());
                    }
                }
            }
            WinEvent::Release { win, button: 1, x, y, .. } if win == pet_win => {
                if self.press.take().is_some() {
                    if self.dragging {
                        self.drop_pet()?;
                    } else {
                        self.click(x, y);
                    }
                }
            }
            WinEvent::Close => self.quit = true,
            _ => {}
        }
        Ok(())
    }

    fn drop_pet(&mut self) -> Res<()> {
        let now = Instant::now();
        self.dragging = false;
        let clamped = clamp_pos(&self.dpy, self.pos, self.surf.w, self.surf.h);
        if clamped != self.pos {
            self.pos = clamped;
            self.dpy.move_window(&self.surf, clamped.0, clamped.1)?;
        }
        self.cfg.pos = Some(self.pos);
        self.cfg.save();
        if self.shake.shaken {
            self.dizzy_until = Some(now + DIZZY_FOR);
            self.say_event(Say::Dizzy, now);
        } else if self.rand() % 3 == 0 {
            self.say_event(Say::Drop, now);
        }
        Ok(())
    }

    fn click(&mut self, x: i32, y: i32) {
        let (bx, by, bw, bh) = render::bubble_rect(self.cfg.scale);
        let on_bubble = self.cfg.mode != Mode::PetOnly && x >= bx && x < bx + bw && y >= by && y < by + bh;
        if on_bubble {
            let _ = self.act(Action::Toggle, true);
        } else {
            self.pat(Instant::now());
        }
    }

    /// Clicking the pet: hearts, friendship, and a combo counter for fidgeting.
    fn pat(&mut self, now: Instant) {
        let was_asleep = self.mood == Mood::Sleep;
        self.combo = if self.last_pat.is_some_and(|t| now.duration_since(t) < COMBO_WINDOW) {
            self.combo.saturating_add(1)
        } else {
            1
        };
        self.last_pat = Some(now);
        self.love_until = Some(now + LOVE_FOR);
        if self.last_friend_pat.map_or(true, |t| now.duration_since(t) > secs(3)) {
            self.last_friend_pat = Some(now);
            self.cfg.add_friendship(self.cfg.pet, 1);
            self.cfg.save();
        }
        if was_asleep {
            self.say_event(Say::Wake, now);
        } else if self.combo >= 3 {
            let bang = if self.persona() == Persona::Hyper { "!!!" } else { "!" };
            self.say(format!("COMBO X{}{bang}", self.combo), now);
        } else if level(self.cfg.friendship(self.cfg.pet)) >= 4 && self.rand() % 2 == 0 {
            self.say_event(Say::PatFriend, now);
        } else {
            self.say_event(Say::Pat, now);
        }
    }

    fn snack(&mut self, now: Instant) {
        if self.last_snack.is_some_and(|t| now.duration_since(t) < SNACK_COOLDOWN) {
            self.say_event(Say::Full, now);
            return;
        }
        self.last_snack = Some(now);
        let fs = Mood::Eat.frame_secs(self.persona());
        self.eat_until = Some(now + Duration::from_secs_f32(fs * Mood::Eat.cycle() as f32));
        self.cfg.add_friendship(self.cfg.pet, 3);
        self.cfg.save();
        self.say_event(Say::Snack, now);
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
                    Mode::Pomodoro => {
                        self.pomo.toggle(now);
                        if self.pomo.is_running() && self.pomo.phase == Phase::Focus {
                            self.say_event(Say::FocusStart, now);
                        }
                    }
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
            Action::Snack => self.snack(now),
            Action::Stats => {}
            Action::Mode => self.cfg.mode = self.cfg.mode.next(),
            Action::Pet(pet) => {
                if pet != self.cfg.pet {
                    self.cfg.pet = pet;
                    self.walk = None;
                    self.love_until = Some(now + LOVE_FOR);
                    self.say_event(Say::Hello, now);
                }
            }
            Action::Page(page) => {
                if let Some(m) = self.menu.as_mut() {
                    m.page = page;
                    m.view.hover = None;
                }
            }
            Action::Size => {
                let i = SCALES.iter().position(|(s, _)| *s >= self.cfg.scale).unwrap_or(0);
                let n = SCALES.len();
                self.set_scale(SCALES[if forward { (i + 1) % n } else { (i + n - 1) % n }].0)?;
            }
            Action::Theme => self.cfg.theme = self.cfg.theme.cycle(forward),
            Action::Focus => self.cfg.pomo.focus_min = pick_next(&FOCUS_CHOICES, self.cfg.pomo.focus_min, forward),
            Action::ShortBreak => self.cfg.pomo.short_min = pick_next(&SHORT_CHOICES, self.cfg.pomo.short_min, forward),
            Action::LongBreak => self.cfg.pomo.long_min = pick_next(&LONG_CHOICES, self.cfg.pomo.long_min, forward),
            Action::Auto => self.cfg.pomo.auto_continue = !self.cfg.pomo.auto_continue,
            Action::Wander => {
                self.cfg.wander = !self.cfg.wander;
                self.end_walk();
            }
            Action::Chatter => self.cfg.chatter = !self.cfg.chatter,
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
        self.pos = clamp_pos(&self.dpy, (x, y), w, h);
        self.cfg.scale = scale;
        self.cfg.pos = Some(self.pos);
        self.dpy.move_resize(&mut self.surf, self.pos.0, self.pos.1, w, h)?;
        self.pm = Pixmap::new(w as u32, h as u32).ok_or("bad window size")?;
        self.dirty = true;
        Ok(())
    }

    // -------------------------------------------------------------- menu

    fn menu_items(&self, page: Page) -> (Vec<(String, bool)>, Vec<Action>) {
        let c = &self.cfg;
        let on_off = |b: bool| if b { "ON" } else { "OFF" };
        let mut items: Vec<(String, bool, Action)> = vec![];
        match page {
            Page::Main => {
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
                items.push(("CHOOSE PET >".into(), !first, Action::Page(Page::Pets)));
                items.push(("GIVE A SNACK ♥".into(), false, Action::Snack));
                let mode = match c.mode {
                    Mode::Pomodoro => "POMODORO",
                    Mode::Stopwatch => "STOPWATCH",
                    Mode::PetOnly => "JUST PET",
                };
                items.push((format!("MODE: {mode}"), false, Action::Mode));
                let size = SCALES.iter().find(|(s, _)| *s >= c.scale).map_or("M", |(_, n)| n);
                items.push((format!("SIZE: {size}"), false, Action::Size));
                items.push(("SETTINGS >".into(), true, Action::Page(Page::Settings)));
                items.push(("QUIT".into(), false, Action::Quit));
            }
            Page::Settings => {
                items.push(("< BACK".into(), false, Action::Page(Page::Main)));
                let today = &c.today;
                let stats = format!("TODAY: {} POMS, {} MIN", today.pomodoros, today.focus_minutes);
                items.push((stats, true, Action::Stats));
                items.push((format!("THEME: {}", c.theme.name().to_uppercase()), true, Action::Theme));
                items.push((format!("FOCUS: {} MIN", c.pomo.focus_min), false, Action::Focus));
                items.push((format!("BREAK: {} MIN", c.pomo.short_min), false, Action::ShortBreak));
                items.push((format!("LONG BREAK: {} MIN", c.pomo.long_min), false, Action::LongBreak));
                items.push((format!("AUTO-START NEXT: {}", on_off(c.pomo.auto_continue)), false, Action::Auto));
                items.push((format!("WALK AROUND: {}", on_off(c.wander)), true, Action::Wander));
                items.push((format!("CHATTER: {}", on_off(c.chatter)), false, Action::Chatter));
                items.push((format!("SOUND: {}", on_off(c.sound)), false, Action::Sound));
                let login = on_off(system::autostart_enabled());
                items.push((format!("START ON LOGIN: {login}"), false, Action::Autostart));
            }
            Page::Pets => {
                items.push(("< BACK".into(), false, Action::Page(Page::Main)));
            }
        }
        let mut actions: Vec<Action> = items.iter().map(|i| i.2).collect();
        if page == Page::Pets {
            actions.extend(crate::pets::ALL.iter().map(|&p| Action::Pet(p)));
        }
        (items.into_iter().map(|(l, s, _)| (l, s)).collect(), actions)
    }

    fn menu_title(&self) -> String {
        let lv = level(self.cfg.friendship(self.cfg.pet));
        let persona = self.cfg.pet.persona().name();
        format!("{} ({persona})  LV {lv} ♥", self.cfg.pet.name()).to_uppercase()
    }

    fn accent(&self) -> u32 {
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
        let sc = self.dpy.screen;
        let x = if rx + w as i32 > sc.right() { rx - w as i32 } else { rx + 1 };
        let y = if ry + h as i32 > sc.bottom() { (ry - h as i32).max(sc.y) } else { ry + 1 };
        (x.max(sc.x), y)
    }

    fn menu_view(&self, page: Page) -> (MenuView, Vec<Action>) {
        let (items, actions) = self.menu_items(page);
        let p = render::menu_font(self.cfg.scale);
        let grid = if page == Page::Pets { render::pet_icons(MenuView::icon_size(p)) } else { vec![] };
        let title = match page {
            Page::Main => self.menu_title(),
            Page::Settings => "SETTINGS".into(),
            Page::Pets => "CHOOSE A PET".into(),
        };
        let view = MenuView {
            title,
            items,
            grid,
            current: Some(self.cfg.pet),
            hover: None,
            accent: self.accent(),
            theme: self.cfg.theme.resolve(self.cfg.pet),
            p,
        };
        (view, actions)
    }

    fn open_menu(&mut self, rx: i32, ry: i32) -> Res<()> {
        self.close_menu()?;
        self.end_walk();
        let (view, actions) = self.menu_view(Page::Main);
        let (w, h) = render::menu_size(&view);
        let (x, y) = self.menu_origin(rx, ry, w, h);
        let surf = self.dpy.create_menu_window(x, y, w, h)?;
        self.menu = Some(Menu {
            page: Page::Main,
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
            self.dpy.destroy(m.surf)?;
        }
        Ok(())
    }

    fn refresh_menu(&mut self) -> Res<()> {
        let Some(page) = self.menu.as_ref().map(|m| m.page) else { return Ok(()) };
        let (mut view, actions) = self.menu_view(page);
        let sc = self.dpy.screen;
        let m = self.menu.as_mut().unwrap();
        view.hover = m.view.hover;
        if page == Page::Pets && m.view.grid.len() == view.grid.len() {
            // Keep the already rendered portraits.
            view.grid = std::mem::take(&mut m.view.grid);
        }
        m.view = view;
        m.actions = actions;
        m.dirty = true;
        let (w, h) = render::menu_size(&m.view);
        if (w, h) != (m.surf.w, m.surf.h) {
            // Keep it on screen if it grew.
            let x = m.at.0.min(sc.right() - w as i32).max(sc.x);
            let y = m.at.1.min(sc.bottom() - h as i32).max(sc.y);
            m.at = (x, y);
            self.dpy.move_resize(&mut m.surf, x, y, w, h)?;
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

    // ------------------------------------------------------------ walking

    fn walk_speed(&self) -> f32 {
        let base = match self.persona() {
            Persona::Hyper => 120.0,
            Persona::Angry => 70.0,
            Persona::Sleepy => 22.0,
            _ => 45.0,
        };
        base * self.cfg.scale / 1.5
    }

    fn walk_interval(&mut self) -> Duration {
        match self.persona() {
            Persona::Hyper => self.rand_secs(10, 30),
            Persona::Sleepy => self.rand_secs(150, 300),
            _ => self.rand_secs(45, 110),
        }
    }

    fn start_walk(&mut self, now: Instant) {
        let (min_x, max_x) = (self.dpy.screen.x, self.dpy.screen.right() - self.surf.w as i32);
        let dist = (40 + self.rand() % 160) as f32 * self.cfg.scale / 1.5;
        let mut dir = if self.rand() % 2 == 0 { 1 } else { -1 };
        // Head back toward the middle when near an edge.
        if self.pos.0 < min_x + 80 {
            dir = 1;
        } else if self.pos.0 > max_x - 80 {
            dir = -1;
        }
        self.walk = Some(Walk { dir, left: dist, x: self.pos.0 as f32, last: now });
    }

    fn end_walk(&mut self) {
        if self.walk.take().is_some() {
            self.cfg.pos = Some(self.pos);
            self.cfg.save();
        }
    }

    fn step_walk(&mut self, now: Instant) -> Res<()> {
        let speed = self.walk_speed();
        let min_x = self.dpy.screen.x as f32;
        let max_x = (self.dpy.screen.right() - self.surf.w as i32) as f32;
        let Some(w) = self.walk.as_mut() else { return Ok(()) };
        let dt = now.duration_since(w.last).as_secs_f32().min(0.2);
        w.last = now;
        let d = speed * dt;
        w.x += w.dir as f32 * d;
        w.left -= d;
        if w.x <= min_x || w.x >= max_x {
            w.x = w.x.clamp(min_x, max_x.max(min_x));
            w.left = 0.0;
        }
        let nx = w.x.round() as i32;
        let done = w.left <= 0.0;
        if nx != self.pos.0 {
            self.pos.0 = nx;
            self.dpy.move_window(&self.surf, self.pos.0, self.pos.1)?;
        }
        if done {
            self.end_walk();
        }
        Ok(())
    }

    // ------------------------------------------------------- time & draw

    pub fn update(&mut self, now: Instant) -> Res<()> {
        if let Some(done) = self.pomo.tick(&self.cfg.pomo, now) {
            self.phase_done(done, now);
        }
        if now >= self.next_blink {
            self.blink_until = now + BLINK_FOR;
            self.next_blink = now + Duration::from_millis(2500 + (self.rand() % 4000) as u64);
        }
        if self.speech.as_ref().is_some_and(|(_, until)| now >= *until) {
            self.speech = None;
        }

        let calm = self.base_mood(now) == Mood::Idle && self.menu.is_none() && self.press.is_none();
        if self.walk.is_some() {
            if calm {
                self.step_walk(now)?;
            } else {
                self.end_walk();
            }
        } else if calm && self.cfg.wander && now >= self.next_walk {
            self.next_walk = now + self.walk_interval();
            self.start_walk(now);
        }
        if calm && self.cfg.chatter && now >= self.next_chatter {
            let wait = match self.persona() {
                Persona::Hyper => self.rand_secs(90, 200),
                _ => self.rand_secs(240, 480),
            };
            self.next_chatter = now + wait;
            if self.speech.is_none() {
                self.say_event(Say::Idle, now);
            }
        }

        let mood = self.compute_mood(now);
        if mood != self.mood {
            self.mood = mood;
            self.mood_since = now;
        }
        Ok(())
    }

    fn phase_done(&mut self, done: Phase, now: Instant) {
        let pet = capitalize(self.cfg.pet.name());
        let next = self.pomo.phase;
        let mins = self.cfg.pomo.minutes(next);
        let (title, body) = if done == Phase::Focus {
            self.cfg.roll_day(&local_date());
            let c = &mut self.cfg;
            c.today.pomodoros = c.today.pomodoros.saturating_add(1);
            c.today.focus_minutes = c.today.focus_minutes.saturating_add(c.pomo.focus_min);
            c.total_pomodoros = c.total_pomodoros.saturating_add(1);
            self.cfg.add_friendship(self.cfg.pet, 5);
            self.cfg.save();
            let tip = lines::BREAK_TIPS[self.rand() as usize % lines::BREAK_TIPS.len()];
            ("Focus session done!", format!("{pet} says: take a {mins} minute break~\n{tip}"))
        } else {
            ("Break's over!", format!("{pet} is ready for {mins} more minutes of focus!"))
        };
        if self.cfg.notify {
            system::notify(title, &body);
        }
        if self.cfg.sound && !system::chime() {
            self.dpy.bell();
        }
        self.cheer_until = Some(now + CHEER_FOR);
        self.say_event(if done == Phase::Focus { Say::FocusDone } else { Say::BreakDone }, now);
        self.attention = if self.cfg.pomo.auto_continue {
            None
        } else if next.is_break() {
            Some("BREAK TIME!")
        } else {
            Some("FOCUS TIME!")
        };
        self.last_activity = now;
    }

    /// Mood from timers and idleness alone (no short reactions).
    fn base_mood(&self, now: Instant) -> Mood {
        if self.pomo.is_running() && self.cfg.mode != Mode::Stopwatch {
            if self.pomo.phase.is_break() {
                Mood::Relax
            } else {
                Mood::Focus
            }
        } else if self.sw.is_running() && self.cfg.mode != Mode::Pomodoro {
            Mood::Focus
        } else if self.attention.is_none() && now.duration_since(self.last_activity) > self.persona().sleep_after() {
            Mood::Sleep
        } else {
            Mood::Idle
        }
    }

    fn compute_mood(&self, now: Instant) -> Mood {
        let active = |t: Option<Instant>| t.is_some_and(|t| now < t);
        if self.dragging {
            Mood::Held
        } else if active(self.dizzy_until) {
            Mood::Dizzy
        } else if active(self.cheer_until) {
            Mood::Cheer
        } else if active(self.eat_until) {
            Mood::Eat
        } else if active(self.love_until) {
            Mood::Love
        } else if self.walk.is_some() {
            Mood::Walk
        } else {
            self.base_mood(now)
        }
    }

    fn pose(&self, now: Instant) -> Pose {
        let t = now.duration_since(self.mood_since).as_secs_f32();
        let p = self.persona();
        let frame = (t / self.mood.frame_secs(p)) as u32 % self.mood.cycle();
        Pose {
            mood: self.mood,
            frame,
            blink: self.mood.blinks() && now < self.blink_until,
            persona: p,
            flip: self.walk.as_ref().is_some_and(|w| w.dir < 0),
        }
    }

    fn bubble(&self, now: Instant) -> Option<Bubble> {
        let theme = self.cfg.theme.resolve(self.cfg.pet);
        let speech = self.speech.as_ref().map(|(s, _)| s.clone());
        let mut b = match self.cfg.mode {
            Mode::PetOnly => Bubble {
                label: String::new(),
                time: String::new(),
                progress: None,
                accent: ACCENT_FOCUS,
                paused: false,
                speech: None,
                theme,
            },
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
                Bubble {
                    label,
                    time: format_clock(p.remaining(now), true),
                    // Quantized so the bar alone doesn't force a redraw every tick.
                    progress: Some((p.progress(s, now) * 200.0) as u16 * 5),
                    accent: self.accent(),
                    paused: !running && self.attention.is_none(),
                    speech: None,
                    theme,
                }
            }
            Mode::Stopwatch => {
                let elapsed = self.sw.elapsed(now);
                let running = self.sw.is_running();
                Bubble {
                    label: if !running && !elapsed.is_zero() { "PAUSED".into() } else { "STOPWATCH".into() },
                    time: format_clock(elapsed, false),
                    progress: None,
                    accent: ACCENT_WATCH,
                    paused: !running,
                    speech: None,
                    theme,
                }
            }
        };
        match speech {
            Some(s) => {
                b.speech = Some(s);
                // Only the speech matters while it's shown; keep the scene
                // stable so the clock doesn't force redraws underneath.
                b.time.clear();
                b.progress = None;
                Some(b)
            }
            None if self.cfg.mode == Mode::PetOnly => None,
            None => Some(b),
        }
    }

    pub fn render(&mut self, now: Instant) -> Res<()> {
        let scene = Scene { scale: self.cfg.scale, pet: self.cfg.pet, pose: self.pose(now), bubble: self.bubble(now) };
        if self.dirty || self.last_scene.as_ref() != Some(&scene) {
            render::draw_scene(&mut self.pm, &scene, &mut self.pets);
            self.dpy.present(&mut self.surf, &self.pm)?;
            self.last_scene = Some(scene);
            self.dirty = false;
        }
        if let Some(m) = self.menu.as_mut().filter(|m| m.dirty) {
            render::draw_menu(&mut m.pm, &m.view);
            self.dpy.present(&mut m.surf, &m.pm)?;
            m.dirty = false;
        }
        Ok(())
    }

    /// How long the event loop may sleep before something on screen changes.
    pub fn next_timeout(&self, now: Instant) -> Duration {
        if self.walk.is_some() {
            return Duration::from_millis(33);
        }
        let mut t = Duration::from_secs(10);
        let fs = self.mood.frame_secs(self.persona());
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
        let deadlines = [
            self.cheer_until,
            self.love_until,
            self.dizzy_until,
            self.eat_until,
            self.speech.as_ref().map(|s| s.1),
            Some(self.next_walk),
            Some(self.next_chatter),
        ];
        for until in deadlines.into_iter().flatten() {
            if until > now {
                t = t.min(until - now);
            }
        }
        if self.mood == Mood::Idle && self.attention.is_none() {
            let sleep_at = self.last_activity + self.persona().sleep_after();
            if sleep_at > now {
                t = t.min(sleep_at - now + Duration::from_millis(5));
            }
        }
        t.max(Duration::from_millis(5))
    }
}

/// Keep at least part of the window on screen so the pet can't get lost.
fn clamp_pos(dpy: &Display, (px, py): (i32, i32), w: u16, h: u16) -> (i32, i32) {
    let (w, h, sc) = (w as i32, h as i32, dpy.screen);
    let keep = 48;
    (px.clamp(sc.x + keep - w, sc.right() - keep), py.clamp(sc.y - h / 2, sc.bottom() - keep))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_detection() {
        let t0 = Instant::now();
        let mut s = Shake { extreme: 500, ..Shake::default() };
        let xs = [500, 540, 580, 530, 480, 540, 600, 520, 470, 560];
        for (i, x) in xs.iter().enumerate() {
            s.track(*x, t0 + Duration::from_millis(i as u64 * 80));
        }
        assert!(s.shaken);

        let mut slow = Shake { extreme: 500, ..Shake::default() };
        for (i, x) in xs.iter().enumerate() {
            slow.track(*x, t0 + Duration::from_secs(i as u64));
        }
        assert!(!slow.shaken, "a slow drag back and forth is not a shake");
    }

    #[test]
    fn choices_wrap() {
        assert_eq!(pick_next(&FOCUS_CHOICES, 25, true), 30);
        assert_eq!(pick_next(&FOCUS_CHOICES, 90, true), 5);
        assert_eq!(pick_next(&FOCUS_CHOICES, 5, false), 90);
        assert_eq!(pick_next(&FOCUS_CHOICES, 27, false), 25);
    }
}
