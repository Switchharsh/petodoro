//! Composes the pet window (speech bubble + pet) and the popup menu.

use crate::font::{draw_text, text_height, text_width};
use crate::gfx::{rgb, rgba, rrect, shade, Canvas, P};
use crate::pets::{self, Mood, PetKind, Pose};
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
    {
        let mut cv = Canvas::new(pm, Transform::from_scale(s, s));
        let path = bubble_path();
        cv.fill(&path, rgba(0xfffaf6, 0.95));
        cv.stroke(&path, shade(accent, 0.85), 1.6);
    }
    let (bx, by, bw, bh) = (BUBBLE.0 * s, BUBBLE.1 * s, BUBBLE.2 * s, BUBBLE.3 * s);
    let pad = (4.0 * s).round();
    let inner_w = bw - 2.0 * pad;

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
    let cx = bx + bw / 2.0;

    let ink = rgb(0x4a3341);
    let label_col = if b.paused { rgb(0x9a8a94) } else { shade(accent, 0.9) };
    draw_text(pm, (cx - text_width(&b.label, pl) as f32 / 2.0).round() as i32, y as i32, &b.label, pl, label_col);
    y += text_height(pl) as f32 + gap;
    let time_col = if b.paused { rgb(0x9a8a94) } else { ink };
    draw_text(pm, (cx - text_width(&b.time, pt) as f32 / 2.0).round() as i32, y as i32, &b.time, pt, time_col);
    y += text_height(pt) as f32 + bar_gap;

    if let Some(permille) = b.progress {
        let mut cv = Canvas::new(pm, Transform::identity());
        let (x0, w) = (bx + pad * 1.5, bw - pad * 3.0);
        cv.fill(&rrect(x0, y, w, bar_h, bar_h / 2.0), rgba(0x000000, 0.08));
        let fw = (w * permille as f32 / 1000.0).max(bar_h);
        cv.fill(&rrect(x0, y, fw, bar_h, bar_h / 2.0), if b.paused { rgb(0xc8b8c2) } else { accent });
    }
}

// ------------------------------------------------------------------ menu

pub struct MenuView {
    pub title: String,
    /// (label, separator above)
    pub items: Vec<(String, bool)>,
    pub hover: Option<usize>,
    pub accent: u32,
}

const M_P: u32 = 2;
const M_PAD: i32 = 12;
const M_ROW: i32 = 22;
const M_TITLE: i32 = 28;
const M_SEP: i32 = 7;

/// Top y of each item row.
fn menu_rows(v: &MenuView) -> Vec<i32> {
    let mut y = M_TITLE;
    v.items
        .iter()
        .map(|(_, sep)| {
            if *sep {
                y += M_SEP;
            }
            let top = y;
            y += M_ROW;
            top
        })
        .collect()
}

pub fn menu_size(v: &MenuView) -> (u16, u16) {
    let widest = v.items.iter().map(|(l, _)| text_width(l, M_P)).chain([text_width(&v.title, M_P)]).max().unwrap_or(0);
    let h = menu_rows(v).last().map_or(M_TITLE, |y| y + M_ROW) + 6;
    (widest as u16 + 2 * M_PAD as u16 + 8, h as u16)
}

pub fn menu_item_at(v: &MenuView, x: i32, y: i32) -> Option<usize> {
    let (w, _) = menu_size(v);
    if x < 3 || x >= w as i32 - 3 {
        return None;
    }
    menu_rows(v).iter().position(|&top| y >= top && y < top + M_ROW)
}

pub fn draw_menu(pm: &mut Pixmap, v: &MenuView) {
    pm.fill(tiny_skia::Color::TRANSPARENT);
    let (w, h) = (pm.width() as f32, pm.height() as f32);
    let accent = rgb(v.accent);
    {
        let mut cv = Canvas::new(pm, Transform::identity());
        let bg = rrect(1.0, 1.0, w - 2.0, h - 2.0, 9.0);
        cv.fill(&bg, rgba(0xfffaf6, 0.97));
        cv.stroke(&bg, shade(accent, 0.85), 1.5);
    }
    let th = text_height(M_P) as i32;
    draw_text(pm, M_PAD + 4, (M_TITLE - th) / 2 + 1, &v.title, M_P, accent);
    let rows = menu_rows(v);
    for (i, ((label, sep), &top)) in v.items.iter().zip(&rows).enumerate() {
        let mut cv = Canvas::new(pm, Transform::identity());
        if *sep {
            let y = top as f32 - M_SEP as f32 / 2.0;
            cv.stroke(&P::new().m(10.0, y).l(w - 10.0, y).build(), rgba(0x4a3341, 0.15), 1.0);
        }
        if v.hover == Some(i) {
            cv.fill(&rrect(5.0, top as f32 + 1.0, w - 10.0, M_ROW as f32 - 2.0, 6.0), rgba(v.accent, 0.18));
        }
        draw_text(pm, M_PAD + 4, top + (M_ROW - th) / 2, label, M_P, rgb(0x4a3341));
    }
}

// --------------------------------------------------------------- preview

/// A contact sheet of every pet in every mood (used by `--preview`).
pub fn preview_sheet(scale: f32) -> Pixmap {
    let moods = [Mood::Idle, Mood::Focus, Mood::Relax, Mood::Sleep, Mood::Cheer, Mood::Love, Mood::Held];
    let (cw, ch) = window_size(scale);
    let (cw, ch) = (cw as u32, ch as u32);
    let mut sheet = Pixmap::new(cw * moods.len() as u32, ch * pets::ALL.len() as u32).unwrap();
    sheet.fill(tiny_skia::Color::from_rgba8(0xee, 0xe8, 0xf0, 0xff));
    let mut cell = Pixmap::new(cw, ch).unwrap();
    let mut cache = PetCache::default();
    for (r, &pet) in pets::ALL.iter().enumerate() {
        for (c, &mood) in moods.iter().enumerate() {
            let label = match mood {
                Mood::Idle => pet.name().to_uppercase(),
                Mood::Focus => "FOCUS 2/4".into(),
                Mood::Relax => "BREAK".into(),
                Mood::Sleep => "PAUSED".into(),
                Mood::Cheer => "BREAK TIME!".into(),
                Mood::Love => "STOPWATCH".into(),
                Mood::Held => "WHEE!".into(),
            };
            let accent = match mood {
                Mood::Relax | Mood::Cheer => 0x3fae6a,
                Mood::Love => 0x8a63d2,
                _ => 0xe8475f,
            };
            let sc = Scene {
                scale,
                pet,
                pose: Pose { mood, frame: if mood == Mood::Cheer { 3 } else { 1 }, blink: false },
                bubble: Some(Bubble {
                    label,
                    time: if mood == Mood::Love { "1:02:03".into() } else { "24:59".into() },
                    progress: if mood == Mood::Love { None } else { Some(300) },
                    accent,
                    paused: mood == Mood::Sleep,
                }),
            };
            draw_scene(&mut cell, &sc, &mut cache);
            sheet.draw_pixmap(
                (c as u32 * cw) as i32,
                (r as u32 * ch) as i32,
                cell.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                Transform::identity(),
                None,
            );
        }
    }
    sheet
}

/// A square app icon of the given pet.
pub fn icon(pet: PetKind, size: u32) -> Pixmap {
    let mut pm = Pixmap::new(size, size).unwrap();
    let s = size as f32 / 100.0;
    let mut cv = Canvas::new(&mut pm, Transform::from_scale(s, s));
    pets::draw(&mut cv, pet, &Pose { mood: Mood::Love, frame: 0, blink: false });
    pm
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
                let sc =
                    Scene { scale: 1.5, pet, pose: Pose { mood: Mood::Focus, frame: i, blink: false }, bubble: None };
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
        };
        let t = std::time::Instant::now();
        for _ in 0..200 {
            pm.fill(tiny_skia::Color::TRANSPARENT);
            draw_bubble(&mut pm, 1.5, &b);
        }
        eprintln!("bubble: {:?}/frame", t.elapsed() / 200);
    }
}
