//! Composes the pet window (speech bubble + pet) and the popup menu.

use crate::font::{draw_text, text_height, text_width};
use crate::gfx::{rgb, rgba, rrect, shade, with_alpha, Canvas, P};
use crate::pets::{self, Mood, PetKind, Pose, Theme};
use tiny_skia::Color;
use tiny_skia::{Pixmap, Transform};

/// Window size in "units" before scaling.
const UNIT_W: f32 = 120.0;
const UNIT_H: f32 = 160.0;
/// Speech bubble rectangle (x, y, w, h) in units; its tail reaches y = 52.
const BUBBLE: (f32, f32, f32, f32) = (8.0, 2.0, 104.0, 44.0);
/// Top-left of the 100x100 pet box in units.
const PET_AT: (f32, f32) = (10.0, 58.0);

pub fn window_size(scale: f32) -> (u16, u16) {
    ((UNIT_W * scale).ceil() as u16, (UNIT_H * scale).ceil() as u16)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bubble {
    pub label: String,
    pub time: String,
    /// Progress bar fill in permille, if this timer has one.
    pub progress: Option<u16>,
    pub accent: u32,
    pub paused: bool,
    /// Something the pet is saying; replaces the label and time for a moment.
    pub speech: Option<String>,
    pub theme: Theme,
}

struct Palette {
    bg: Color,
    ink: Color,
    dim: Color,
    faint: Color,
}

fn palette(t: Theme) -> Palette {
    match t {
        Theme::Light => {
            Palette { bg: rgba(0xfffaf6, 0.95), ink: rgb(0x4a3341), dim: rgb(0x9a8a94), faint: rgba(0x000000, 0.08) }
        }
        Theme::Dark => {
            Palette { bg: rgba(0x221f2b, 0.95), ink: rgb(0xf1e8f4), dim: rgb(0x8f8599), faint: rgba(0xffffff, 0.12) }
        }
        Theme::Amoled => {
            Palette { bg: rgba(0x000000, 1.0), ink: rgb(0xffffff), dim: rgb(0x8a8a8a), faint: rgba(0xffffff, 0.14) }
        }
    }
}

fn outline(t: Theme, accent: u32) -> Color {
    if t == Theme::Light {
        shade(rgb(accent), 0.85)
    } else {
        rgb(accent)
    }
}

/// Greedy word wrap to at most `cols` characters per line.
pub fn wrap(text: &str, cols: usize) -> Vec<String> {
    let cols = cols.max(1);
    let mut lines: Vec<String> = vec![];
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let mut word = word.to_string();
        while word.chars().count() > cols {
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
            }
            let head: String = word.chars().take(cols).collect();
            word = word.chars().skip(cols).collect();
            lines.push(head);
        }
        if cur.is_empty() {
            cur = word;
        } else if cur.chars().count() + 1 + word.chars().count() <= cols {
            cur.push(' ');
            cur.push_str(&word);
        } else {
            lines.push(std::mem::replace(&mut cur, word));
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// Everything that determines what the pet window looks like. The app only
/// redraws when this changes, which keeps CPU use near zero.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub scale: f32,
    pub pet: PetKind,
    pub pose: Pose,
    pub bubble: Option<Bubble>,
}

/// Bubble rectangle in window pixels, for hit-testing.
pub fn bubble_rect(scale: f32) -> (i32, i32, i32, i32) {
    let (x, y, w, h) = BUBBLE;
    ((x * scale) as i32, (y * scale) as i32, (w * scale) as i32, ((h + 6.0) * scale) as i32)
}

/// Recently drawn pet frames. Animations loop over a few frames, so most
/// redraws (e.g. the clock ticking) just reuse a cached pet layer.
#[derive(Default)]
pub struct PetCache {
    entries: Vec<(PetKind, Pose, u32, Pixmap)>,
}

const PET_CACHE_SIZE: usize = 8;

impl PetCache {
    fn get(&mut self, pet: PetKind, pose: Pose, scale: f32, w: u32, h: u32) -> &Pixmap {
        let key = (pet, pose, scale.to_bits());
        if let Some(i) =
            self.entries.iter().position(|e| (e.0, e.1, e.2) == key && e.3.width() == w && e.3.height() == h)
        {
            let hit = self.entries.remove(i);
            self.entries.push(hit);
        } else {
            let mut pm = match self.entries.len() >= PET_CACHE_SIZE {
                true => self.entries.remove(0).3,
                false => Pixmap::new(w, h).unwrap(),
            };
            if pm.width() != w || pm.height() != h {
                pm = Pixmap::new(w, h).unwrap();
            }
            pm.fill(tiny_skia::Color::TRANSPARENT);
            let ts = Transform::from_row(scale, 0.0, 0.0, scale, PET_AT.0 * scale, PET_AT.1 * scale);
            pets::draw(&mut Canvas::new(&mut pm, ts), pet, &pose);
            self.entries.push((pet, pose, scale.to_bits(), pm));
        }
        &self.entries.last().unwrap().3
    }
}

pub fn draw_scene(pm: &mut Pixmap, sc: &Scene, cache: &mut PetCache) {
    pm.fill(tiny_skia::Color::TRANSPARENT);
    if let Some(b) = &sc.bubble {
        draw_bubble(pm, sc.scale, b);
    }
    let layer = cache.get(sc.pet, sc.pose, sc.scale, pm.width(), pm.height());
    pm.draw_pixmap(0, 0, layer.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
}

fn bubble_path() -> tiny_skia::Path {
    let (x, y, w, h) = BUBBLE;
    let r = 11.0;
    let (b, cx) = (y + h, x + w / 2.0);
    P::new()
        .m(x + r, y)
        .l(x + w - r, y)
        .q(x + w, y, x + w, y + r)
        .l(x + w, b - r)
        .q(x + w, b, x + w - r, b)
        .l(cx + 6.0, b)
        .q(cx + 2.0, b + 1.0, cx, b + 7.0)
        .q(cx - 2.0, b + 1.0, cx - 6.0, b)
        .l(x + r, b)
        .q(x, b, x, b - r)
        .l(x, y + r)
        .q(x, y, x + r, y)
        .z()
        .build()
}

fn draw_bubble(pm: &mut Pixmap, s: f32, b: &Bubble) {
    let accent = rgb(b.accent);
    let pal = palette(b.theme);
    {
        let mut cv = Canvas::new(pm, Transform::from_scale(s, s));
        let path = bubble_path();
        cv.fill(&path, pal.bg);
        cv.stroke(&path, outline(b.theme, b.accent), 1.6);
    }
    let (bx, by, bw, bh) = (BUBBLE.0 * s, BUBBLE.1 * s, BUBBLE.2 * s, BUBBLE.3 * s);
    let pad = (4.0 * s).round();
    let inner_w = bw - 2.0 * pad;
    let cx = bx + bw / 2.0;
    let label_col = if b.paused {
        pal.dim
    } else if b.theme == Theme::Light {
        shade(accent, 0.9)
    } else {
        accent
    };

    if let Some(text) = &b.speech {
        // Biggest pixel size at which the wrapped text fits.
        let mut p = ((s * 1.6).round() as u32).max(1);
        let lines = loop {
            let cols = ((inner_w + p as f32) / (6 * p) as f32).floor() as usize;
            let lines = wrap(text, cols);
            let h = lines.len() as f32 * text_height(p) as f32 + (lines.len() as f32 - 1.0) * 2.0 * p as f32;
            // Shrink rather than break a word in the middle.
            let longest = text.split_whitespace().map(|w| w.chars().count()).max().unwrap_or(0);
            let fits = h <= bh - 2.0 * pad && longest <= cols;
            if fits || p == 1 {
                break lines;
            }
            p -= 1;
        };
        let lh = text_height(p) as f32 + 2.0 * p as f32;
        let total = lines.len() as f32 * lh - 2.0 * p as f32;
        let mut y = by + ((bh - total) / 2.0).round();
        for line in &lines {
            let x = (cx - text_width(line, p) as f32 / 2.0).round() as i32;
            draw_text(pm, x, y as i32, line, p, pal.ink);
            y += lh;
        }
        return;
    }

    // Label: pixel size scales with the window but must fit the width.
    let mut pl = ((s * 1.3).round() as u32).max(1);
    while pl > 1 && text_width(&b.label, pl) as f32 > inner_w {
        pl -= 1;
    }
    let gap = (2.5 * s).round();
    let bar_h = if b.progress.is_some() { (3.0 * s).round().max(2.0) } else { 0.0 };
    let bar_gap = if b.progress.is_some() { (3.0 * s).round() } else { 0.0 };
    let avail = bh - 2.0 * pad - text_height(pl) as f32 - gap - bar_gap - bar_h;
    let mut pt = ((avail / 7.0).floor() as u32).max(1);
    while pt > 1 && text_width(&b.time, pt) as f32 > inner_w {
        pt -= 1;
    }
    let total = text_height(pl) as f32 + gap + text_height(pt) as f32 + bar_gap + bar_h;
    let mut y = by + ((bh - total) / 2.0).round();

    draw_text(pm, (cx - text_width(&b.label, pl) as f32 / 2.0).round() as i32, y as i32, &b.label, pl, label_col);
    y += text_height(pl) as f32 + gap;
    let time_col = if b.paused { pal.dim } else { pal.ink };
    draw_text(pm, (cx - text_width(&b.time, pt) as f32 / 2.0).round() as i32, y as i32, &b.time, pt, time_col);
    y += text_height(pt) as f32 + bar_gap;

    if let Some(permille) = b.progress {
        let mut cv = Canvas::new(pm, Transform::identity());
        let (x0, w) = (bx + pad * 1.5, bw - pad * 3.0);
        cv.fill(&rrect(x0, y, w, bar_h, bar_h / 2.0), pal.faint);
        let fw = (w * permille as f32 / 1000.0).max(bar_h);
        cv.fill(&rrect(x0, y, fw, bar_h, bar_h / 2.0), if b.paused { pal.dim } else { accent });
    }
}

// ------------------------------------------------------------------ menu

pub struct MenuView {
    pub title: String,
    /// List rows: (label, separator above).
    pub items: Vec<(String, bool)>,
    /// Pet picker cells, laid out in a grid below the rows.
    pub grid: Vec<(PetKind, Pixmap)>,
    /// Highlighted pet in the grid.
    pub current: Option<PetKind>,
    /// Hovered entry: rows first, then grid cells.
    pub hover: Option<usize>,
    pub accent: u32,
    pub theme: Theme,
    /// Pixel size of the menu font (bigger = readable from further away).
    pub p: u32,
}

pub const GRID_COLS: usize = 5;

/// Menu font size for a pet scale: S 2, M 3, L and XL 4.
pub fn menu_font(scale: f32) -> u32 {
    ((scale * 2.0).round() as u32).clamp(2, 4)
}

struct MenuLayout {
    rows: Vec<i32>,
    row_h: i32,
    cells: Vec<(i32, i32)>,
    cell: (i32, i32),
    w: i32,
    h: i32,
}

impl MenuView {
    /// Size of a pet portrait in the picker.
    pub fn icon_size(p: u32) -> u32 {
        22 * p
    }

    fn layout(&self) -> MenuLayout {
        let p = self.p as i32;
        let (pad, row_h, title_h, sep) = (6 * p, 11 * p, 14 * p, 3 * p);
        let mut y = title_h;
        let rows: Vec<i32> = self
            .items
            .iter()
            .map(|(_, s)| {
                if *s {
                    y += sep;
                }
                let top = y;
                y += row_h;
                top
            })
            .collect();
        let cell = (Self::icon_size(self.p) as i32 + 6 * p, Self::icon_size(self.p) as i32 + 11 * p);
        let grid_top = y + sep;
        let cells: Vec<(i32, i32)> = (0..self.grid.len())
            .map(|i| (pad + (i % GRID_COLS) as i32 * cell.0, grid_top + (i / GRID_COLS) as i32 * cell.1))
            .collect();
        if !cells.is_empty() {
            y = grid_top + cells.len().div_ceil(GRID_COLS) as i32 * cell.1;
        }
        let widest = self
            .items
            .iter()
            .map(|(l, _)| text_width(l, self.p))
            .chain([text_width(&self.title, self.p)])
            .max()
            .unwrap_or(0) as i32;
        let grid_w = if cells.is_empty() { 0 } else { GRID_COLS.min(self.grid.len()) as i32 * cell.0 };
        let w = (widest + 2 * pad + 4 * p).max(grid_w + 2 * pad);
        MenuLayout { rows, row_h, cells, cell, w, h: y + pad }
    }
}

pub fn menu_size(v: &MenuView) -> (u16, u16) {
    let l = v.layout();
    (l.w as u16, l.h as u16)
}

pub fn menu_item_at(v: &MenuView, x: i32, y: i32) -> Option<usize> {
    let l = v.layout();
    if x < 3 || x >= l.w - 3 {
        return None;
    }
    if let Some(i) = l.rows.iter().position(|&top| y >= top && y < top + l.row_h) {
        return Some(i);
    }
    l.cells
        .iter()
        .position(|&(cx, cy)| x >= cx && x < cx + l.cell.0 && y >= cy && y < cy + l.cell.1)
        .map(|i| i + v.items.len())
}

pub fn draw_menu(pm: &mut Pixmap, v: &MenuView) {
    pm.fill(tiny_skia::Color::TRANSPARENT);
    let l = v.layout();
    let (w, h) = (pm.width() as f32, pm.height() as f32);
    let accent = rgb(v.accent);
    let pal = palette(v.theme);
    let p = v.p as i32;
    let pad = 6 * p;
    {
        let mut cv = Canvas::new(pm, Transform::identity());
        let bg = rrect(1.0, 1.0, w - 2.0, h - 2.0, 4.0 * p as f32);
        cv.fill(&bg, with_alpha(pal.bg, 0.97_f32.max(pal.bg.alpha())));
        cv.stroke(&bg, outline(v.theme, v.accent), 1.5);
    }
    let th = text_height(v.p) as i32;
    draw_text(pm, pad + 2 * p, (14 * p - th) / 2 + p / 2, &v.title, v.p, accent);
    let hover_fill = rgba(v.accent, if v.theme == Theme::Light { 0.18 } else { 0.32 });
    for (i, ((label, sep), &top)) in v.items.iter().zip(&l.rows).enumerate() {
        let mut cv = Canvas::new(pm, Transform::identity());
        if *sep {
            let y = top as f32 - 1.5 * p as f32;
            cv.stroke(&P::new().m(5.0 * p as f32, y).l(w - 5.0 * p as f32, y).build(), pal.faint, 1.0);
        }
        if v.hover == Some(i) {
            cv.fill(
                &rrect(2.5 * p as f32, top as f32 + 1.0, w - 5.0 * p as f32, l.row_h as f32 - 2.0, 3.0 * p as f32),
                hover_fill,
            );
        }
        draw_text(pm, pad + 2 * p, top + (l.row_h - th) / 2, label, v.p, pal.ink);
    }
    let name_p = (v.p - 1).max(1);
    for (i, ((pet, icon), &(cx, cy))) in v.grid.iter().zip(&l.cells).enumerate() {
        let selected = v.current == Some(*pet);
        let hovered = v.hover == Some(i + v.items.len());
        if selected || hovered {
            let mut cv = Canvas::new(pm, Transform::identity());
            let r =
                rrect(cx as f32 + 2.0, cy as f32 + 2.0, l.cell.0 as f32 - 4.0, l.cell.1 as f32 - 4.0, 3.0 * p as f32);
            cv.fill(&r, if hovered { hover_fill } else { rgba(v.accent, 0.10) });
            if selected {
                cv.stroke(&r, accent, 1.5);
            }
        }
        let ix = cx + (l.cell.0 - icon.width() as i32) / 2;
        pm.draw_pixmap(ix, cy + p, icon.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
        let name = pet.name().to_uppercase();
        let nx = cx + (l.cell.0 - text_width(&name, name_p) as i32) / 2;
        draw_text(pm, nx, cy + icon.height() as i32 + 2 * p, &name, name_p, if selected { accent } else { pal.ink });
    }
}

/// Portraits for the pet picker.
pub fn pet_icons(size: u32) -> Vec<(PetKind, Pixmap)> {
    pets::ALL
        .iter()
        .map(|&pet| {
            let mut pm = Pixmap::new(size, size).unwrap();
            let s = size as f32 / 100.0;
            let mut cv = Canvas::new(&mut pm, Transform::from_scale(s, s));
            pets::draw(&mut cv, pet, &Pose::new(Mood::Idle, 0, pet.persona()));
            (pet, pm)
        })
        .collect()
}

// --------------------------------------------------------------- preview

fn theme_bg(t: Theme) -> Color {
    match t {
        Theme::Light => rgb(0xeee8f0),
        Theme::Dark => rgb(0x2a2633),
        Theme::Amoled => rgb(0x000000),
    }
}

fn paste(sheet: &mut Pixmap, cell: &Pixmap, x: u32, y: u32, bg: Color) {
    let mut cv = Canvas::new(sheet, Transform::identity());
    cv.fill(&rrect(x as f32, y as f32, cell.width() as f32, cell.height() as f32, 0.0), bg);
    sheet.draw_pixmap(
        x as i32,
        y as i32,
        cell.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

/// A contact sheet of every pet in every mood (used by `--preview`).
pub fn preview_sheet(scale: f32) -> Pixmap {
    let (cw, ch) = window_size(scale);
    let (cw, ch) = (cw as u32, ch as u32);
    let mut sheet = Pixmap::new(cw * pets::MOODS.len() as u32, ch * pets::ALL.len() as u32).unwrap();
    let mut cell = Pixmap::new(cw, ch).unwrap();
    let mut cache = PetCache::default();
    for (r, &pet) in pets::ALL.iter().enumerate() {
        for (c, &mood) in pets::MOODS.iter().enumerate() {
            let (label, accent) = match mood {
                Mood::Idle => (pet.name().to_uppercase(), 0xe8475f),
                Mood::Focus => ("FOCUS 2/4".into(), 0xe8475f),
                Mood::Relax => ("BREAK".into(), 0x3fae6a),
                Mood::Sleep => ("PAUSED".into(), 0xe8475f),
                Mood::Cheer => ("BREAK TIME!".into(), 0x3fae6a),
                _ => ("STOPWATCH".into(), 0x8a63d2),
            };
            let speech = match mood {
                Mood::Love => Some(crate::lines::sample(pet.persona(), crate::lines::Event::Pat)),
                Mood::Dizzy => Some(crate::lines::sample(pet.persona(), crate::lines::Event::Dizzy)),
                Mood::Eat => Some(crate::lines::sample(pet.persona(), crate::lines::Event::Snack)),
                _ => None,
            };
            let frame = match mood {
                Mood::Cheer => 3,
                Mood::Eat => 2,
                _ => 1,
            };
            let sc = Scene {
                scale,
                pet,
                pose: Pose { flip: mood == Mood::Walk, ..Pose::new(mood, frame, pet.persona()) },
                bubble: Some(Bubble {
                    label,
                    time: if mood == Mood::Walk { "1:02:03".into() } else { "24:59".into() },
                    progress: if mood == Mood::Walk { None } else { Some(300) },
                    accent,
                    paused: mood == Mood::Sleep,
                    speech: speech.map(|s| s.to_string()),
                    theme: pet.theme(),
                }),
            };
            draw_scene(&mut cell, &sc, &mut cache);
            paste(&mut sheet, &cell, c as u32 * cw, r as u32 * ch, theme_bg(pet.theme()));
        }
    }
    sheet
}

/// Every pet saying hello, `cols` per row (for the README).
pub fn gallery(scale: f32, cols: u32) -> Pixmap {
    let (cw, ch) = window_size(scale);
    let (cw, ch) = (cw as u32, ch as u32);
    let rows = (pets::ALL.len() as u32).div_ceil(cols);
    let mut sheet = Pixmap::new(cw * cols, ch * rows).unwrap();
    sheet.fill(theme_bg(Theme::Light));
    let mut cell = Pixmap::new(cw, ch).unwrap();
    let mut cache = PetCache::default();
    for (i, &pet) in pets::ALL.iter().enumerate() {
        let persona = pet.persona();
        let mood = if persona == pets::Persona::Sleepy { Mood::Sleep } else { Mood::Idle };
        let sc = Scene {
            scale,
            pet,
            pose: Pose::new(mood, 2, persona),
            bubble: Some(Bubble {
                label: String::new(),
                time: String::new(),
                progress: None,
                accent: 0xe8475f,
                paused: false,
                speech: Some(format!("{} ({})", pet.name(), persona.name()).to_uppercase()),
                theme: pet.theme(),
            }),
        };
        draw_scene(&mut cell, &sc, &mut cache);
        paste(&mut sheet, &cell, (i as u32 % cols) * cw, (i as u32 / cols) * ch, theme_bg(pet.theme()));
    }
    sheet
}

/// A square app icon of the given pet.
pub fn icon(pet: PetKind, size: u32) -> Pixmap {
    let mut pm = Pixmap::new(size, size).unwrap();
    let s = size as f32 / 100.0;
    let mut cv = Canvas::new(&mut pm, Transform::from_scale(s, s));
    pets::draw(&mut cv, pet, &Pose::new(Mood::Idle, 1, pets::Persona::Sweet));
    pm
}

#[cfg(test)]
mod tests {
    use super::wrap;

    #[test]
    fn wraps_words() {
        assert_eq!(wrap("B-BAKA! DON'T TOUCH ME!", 10), ["B-BAKA!", "DON'T", "TOUCH ME!"]);
        assert_eq!(wrap("WHEEEEEEEEEE", 5), ["WHEEE", "EEEEE", "EE"]);
        assert!(wrap("", 5).is_empty());
    }
}

#[cfg(test)]
mod bench {
    use super::*;
    #[test]
    #[ignore]
    fn frame_cost() {
        let mut pm = Pixmap::new(180, 240).unwrap();
        for pet in pets::ALL {
            let t = std::time::Instant::now();
            for i in 0..200 {
                let sc = Scene { scale: 1.5, pet, pose: Pose::new(Mood::Focus, i, pet.persona()), bubble: None };
                draw_scene(&mut pm, &sc, &mut PetCache::default());
            }
            eprintln!("{:?} pet: {:?}/frame", pet, t.elapsed() / 200);
        }
        let b = Bubble {
            label: "FOCUS 1/4".into(),
            time: "24:59".into(),
            progress: Some(300),
            accent: 0xe8475f,
            paused: false,
            speech: None,
            theme: Theme::Light,
        };
        let t = std::time::Instant::now();
        for _ in 0..200 {
            pm.fill(tiny_skia::Color::TRANSPARENT);
            draw_bubble(&mut pm, 1.5, &b);
        }
        eprintln!("bubble: {:?}/frame", t.elapsed() / 200);
    }
}
