//! The pets. Every pet is drawn with vector shapes in a 100x100 unit box
//! (feet at y ~ 98) so it scales cleanly to any size.

mod cat;
mod girl;
mod slime;

use crate::gfx::{circle, ellipse, mix, rgb, rgba, shade, with_alpha, Canvas, P};
use tiny_skia::{Color, Transform};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PetKind {
    Sakura,
    Yuki,
    Hikari,
    Mochi,
    Puru,
}

pub const ALL: [PetKind; 5] = [PetKind::Sakura, PetKind::Yuki, PetKind::Hikari, PetKind::Mochi, PetKind::Puru];

impl PetKind {
    pub fn name(self) -> &'static str {
        match self {
            PetKind::Sakura => "sakura",
            PetKind::Yuki => "yuki",
            PetKind::Hikari => "hikari",
            PetKind::Mochi => "mochi",
            PetKind::Puru => "puru",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            PetKind::Sakura => "pink twin-tails, sailor uniform",
            PetKind::Yuki => "silver hair, cat-ear hoodie",
            PetKind::Hikari => "blonde ponytail, maid outfit",
            PetKind::Mochi => "orange tabby cat",
            PetKind::Puru => "slime with a sprout",
        }
    }

    pub fn from_name(s: &str) -> Option<PetKind> {
        ALL.iter().copied().find(|p| p.name().eq_ignore_ascii_case(s.trim()))
    }

    pub fn cycle(self, forward: bool) -> PetKind {
        let i = ALL.iter().position(|p| *p == self).unwrap_or(0);
        let n = ALL.len();
        ALL[if forward { (i + 1) % n } else { (i + n - 1) % n }]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    /// Nothing running.
    Idle,
    /// Focus session or stopwatch running: reading a little book.
    Focus,
    /// On a break: humming.
    Relax,
    /// Ignored for a while.
    Sleep,
    /// A phase just finished: jumping with sparkles.
    Cheer,
    /// Was clicked: hearts.
    Love,
    /// Being dragged around.
    Held,
}

impl Mood {
    /// How long one animation frame lasts, in seconds.
    pub fn frame_secs(self) -> f32 {
        match self {
            Mood::Idle => 0.7,
            Mood::Focus => 0.6,
            Mood::Relax => 0.45,
            Mood::Sleep => 0.6,
            Mood::Cheer => 0.11,
            Mood::Love => 0.18,
            Mood::Held => 0.16,
        }
    }

    /// Number of frames after which the animation repeats exactly.
    pub fn cycle(self) -> u32 {
        match self {
            Mood::Idle | Mood::Focus => 2,
            Mood::Relax => 4,
            Mood::Sleep => 6,
            Mood::Love => 8,
            Mood::Held => 12,
            Mood::Cheer => 24,
        }
    }

    pub fn blinks(self) -> bool {
        matches!(self, Mood::Idle | Mood::Focus | Mood::Held)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pose {
    pub mood: Mood,
    pub frame: u32,
    pub blink: bool,
}

pub const LASH: u32 = 0x3a2430;

const CHEER_JUMP: [f32; 8] = [0.0, 5.0, 9.0, 11.0, 11.0, 9.0, 5.0, 0.0];
const HELD_SWING: [f32; 6] = [-7.0, -3.0, 3.0, 7.0, 3.0, -3.0];

/// Vertical lift of the body (for the shadow), in units.
fn lift(pose: &Pose) -> f32 {
    match pose.mood {
        Mood::Cheer => CHEER_JUMP[pose.frame as usize % CHEER_JUMP.len()],
        Mood::Love => [0.0, 2.0, 3.0, 2.0][pose.frame as usize % 4],
        _ => 0.0,
    }
}

/// Per-mood body motion, applied around the feet (or the scruff when held).
fn body_transform(pose: &Pose) -> Transform {
    let f = pose.frame as usize;
    let around = |sx: f32, sy: f32, px: f32, py: f32| {
        Transform::from_translate(px, py).pre_scale(sx, sy).pre_translate(-px, -py)
    };
    match pose.mood {
        Mood::Idle => around(if f % 2 == 1 { 1.015 } else { 1.0 }, if f % 2 == 1 { 0.985 } else { 1.0 }, 50.0, 98.0),
        Mood::Focus => Transform::from_translate(0.0, if f % 2 == 1 { 0.8 } else { 0.0 }),
        Mood::Relax => Transform::from_rotate_at([-2.5, 0.0, 2.5, 0.0][f % 4], 50.0, 98.0),
        Mood::Sleep => {
            let s = [1.0, 0.99, 0.98, 0.97, 0.98, 0.99][f % 6];
            around(2.0 - s, s, 50.0, 98.0)
        }
        Mood::Cheer => {
            let squash = if f % CHEER_JUMP.len() == 0 { around(1.06, 0.94, 50.0, 98.0) } else { Transform::identity() };
            Transform::from_translate(0.0, -lift(pose)).pre_concat(squash)
        }
        Mood::Love => Transform::from_translate(0.0, -lift(pose)),
        Mood::Held => Transform::from_rotate_at(HELD_SWING[f % HELD_SWING.len()], 50.0, 12.0)
            .pre_concat(around(0.97, 1.04, 50.0, 12.0)),
    }
}

/// Draw `kind` into the 100x100 unit box mapped by `cv.ts`.
pub fn draw(cv: &mut Canvas, kind: PetKind, pose: &Pose) {
    if pose.mood != Mood::Held {
        let l = lift(pose);
        let rx = 24.0 - l * 0.8;
        cv.fill(&ellipse(50.0, 97.5, rx, 3.2 - l * 0.1), rgba(0x000000, 0.16));
    }
    cv.with(body_transform(pose), |cv| match kind {
        PetKind::Sakura => girl::draw(cv, &girl::SAKURA, pose),
        PetKind::Yuki => girl::draw(cv, &girl::YUKI, pose),
        PetKind::Hikari => girl::draw(cv, &girl::HIKARI, pose),
        PetKind::Mochi => cat::draw(cv, pose),
        PetKind::Puru => slime::draw(cv, pose),
    });
    draw_effects(cv, pose);
}

// ---------------------------------------------------------------- faces

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EyeStyle {
    Open,
    Narrow,
    HappyArc,
    Sleepy,
    Blink,
    Squeeze,
    Wide,
}

pub fn eye_style(pose: &Pose) -> EyeStyle {
    if pose.blink && pose.mood.blinks() {
        return EyeStyle::Blink;
    }
    match pose.mood {
        Mood::Idle => EyeStyle::Open,
        Mood::Focus => EyeStyle::Narrow,
        Mood::Relax | Mood::Cheer => EyeStyle::HappyArc,
        Mood::Sleep => EyeStyle::Sleepy,
        Mood::Love => EyeStyle::Squeeze,
        Mood::Held => EyeStyle::Wide,
    }
}

/// Big anime eyes. `w`/`h` are the iris radii.
pub fn eyes(cv: &mut Canvas, lx: f32, rx: f32, y: f32, w: f32, h: f32, iris: Color, pose: &Pose) {
    let lash = rgb(LASH);
    let style = eye_style(pose);
    for (i, &cx) in [lx, rx].iter().enumerate() {
        let side = if i == 0 { -1.0 } else { 1.0 };
        match style {
            EyeStyle::Open | EyeStyle::Narrow => {
                let hh = if style == EyeStyle::Narrow { h * 0.78 } else { h };
                let cy = y + (h - hh) * 0.6;
                let path = ellipse(cx, cy, w, hh);
                cv.fill_vgrad(&path, cy - hh, shade(iris, 0.45), cy + hh, mix(iris, rgb(0xffffff), 0.35));
                cv.stroke(&path, lash, 0.7);
                cv.fill(&ellipse(cx, cy + hh * 0.12, w * 0.46, hh * 0.52), shade(iris, 0.28));
                cv.fill(&circle(cx - w * 0.32, cy - hh * 0.4, w * 0.36), rgb(0xffffff));
                cv.fill(&circle(cx + w * 0.3, cy + hh * 0.42, w * 0.17), rgba(0xffffff, 0.9));
                // Upper lash line with a flick at the outer corner.
                let top = P::new()
                    .m(cx - w * 1.2, cy - hh * 0.45)
                    .q(cx, cy - hh * 1.45, cx + w * 1.2, cy - hh * 0.45)
                    .build();
                cv.stroke(&top, lash, 1.7);
                let ox = cx + side * w * 1.15;
                let flick = P::new().m(ox, cy - hh * 0.5).l(ox + side * 1.6, cy - hh * 0.75).build();
                cv.stroke(&flick, lash, 1.1);
            }
            EyeStyle::HappyArc => {
                let p = P::new().m(cx - w, y + 1.5).q(cx, y - h * 1.1, cx + w, y + 1.5).build();
                cv.stroke(&p, lash, 1.7);
            }
            EyeStyle::Sleepy => {
                let p = P::new().m(cx - w, y + 0.5).q(cx, y + h * 0.7, cx + w, y + 0.5).build();
                cv.stroke(&p, lash, 1.6);
            }
            EyeStyle::Blink => {
                let p = P::new().m(cx - w * 1.1, y + 1.5).q(cx, y + 3.0, cx + w * 1.1, y + 1.0).build();
                cv.stroke(&p, lash, 1.6);
            }
            EyeStyle::Squeeze => {
                // ">" on the left eye, "<" on the right.
                let s = -side;
                let p = P::new()
                    .m(cx - s * w * 0.8, y - h * 0.55)
                    .l(cx + s * w * 0.8, y)
                    .l(cx - s * w * 0.8, y + h * 0.55)
                    .build();
                cv.stroke(&p, lash, 1.7);
            }
            EyeStyle::Wide => {
                let white = ellipse(cx, y, w * 0.95, h * 0.95);
                cv.fill(&white, rgb(0xffffff));
                cv.stroke(&white, lash, 1.2);
                cv.fill(&circle(cx, y + 0.5, w * 0.45), shade(iris, 0.6));
                cv.fill(&circle(cx - w * 0.15, y - 0.3, w * 0.15), rgb(0xffffff));
            }
        }
    }
}

pub fn brows(cv: &mut Canvas, lx: f32, rx: f32, y: f32, color: Color, pose: &Pose) {
    let c = shade(color, 0.7);
    match pose.mood {
        Mood::Focus => {
            // Determined: slanted down toward the middle.
            cv.stroke(&P::new().m(lx - 4.0, y - 1.2).l(lx + 3.5, y + 0.6).build(), c, 1.4);
            cv.stroke(&P::new().m(rx + 4.0, y - 1.2).l(rx - 3.5, y + 0.6).build(), c, 1.4);
        }
        Mood::Held => {
            // Worried: slanted up toward the middle.
            cv.stroke(&P::new().m(lx - 4.0, y + 0.4).l(lx + 3.0, y - 1.4).build(), c, 1.3);
            cv.stroke(&P::new().m(rx + 4.0, y + 0.4).l(rx - 3.0, y - 1.4).build(), c, 1.3);
        }
        _ => {}
    }
}

pub fn blush(cv: &mut Canvas, lx: f32, rx: f32, y: f32, pose: &Pose) {
    let a = match pose.mood {
        Mood::Love | Mood::Cheer => 0.6,
        Mood::Held => 0.25,
        _ => 0.38,
    };
    let c = rgba(0xff7f9e, a);
    cv.fill(&ellipse(lx, y, 4.2, 2.2), c);
    cv.fill(&ellipse(rx, y, 4.2, 2.2), c);
    if matches!(pose.mood, Mood::Love | Mood::Cheer) {
        let line = rgba(0xe0567a, 0.7);
        for (cx, _) in [(lx, 0), (rx, 1)] {
            for k in 0..3 {
                let x = cx - 2.4 + k as f32 * 1.8;
                cv.stroke(&P::new().m(x + 0.6, y - 1.0).l(x - 0.4, y + 1.0).build(), line, 0.6);
            }
        }
    }
}

pub fn mouth(cv: &mut Canvas, x: f32, y: f32, pose: &Pose, cat: bool) {
    let line = rgb(0x8a3a48);
    let inside = rgb(0xb8404f);
    match pose.mood {
        Mood::Idle if cat => {
            let p = P::new().m(x - 3.2, y).q(x - 1.6, y + 2.6, x, y).q(x + 1.6, y + 2.6, x + 3.2, y).build();
            cv.stroke(&p, line, 1.0);
        }
        Mood::Idle => cv.stroke(&P::new().m(x - 2.2, y).q(x, y + 1.9, x + 2.2, y).build(), line, 1.1),
        Mood::Focus => cv.stroke(&P::new().m(x - 1.7, y + 0.7).l(x + 1.7, y + 0.7).build(), line, 1.1),
        Mood::Relax => {
            let p = P::new().m(x - 2.8, y - 0.4).q(x, y + 4.4, x + 2.8, y - 0.4).z().build();
            cv.fl(&p, inside, line);
        }
        Mood::Sleep => cv.fl(&ellipse(x, y + 0.9, 1.1, 1.3), inside, line),
        Mood::Cheer | Mood::Love => {
            let p = P::new().m(x - 3.6, y - 1.0).q(x, y + 6.2, x + 3.6, y - 1.0).z().build();
            cv.fl(&p, inside, line);
            cv.fill(&ellipse(x, y + 2.5, 1.8, 0.9), rgb(0xff8fa3));
        }
        Mood::Held => cv.fl(&ellipse(x, y + 1.0, 1.6, 2.2), inside, line),
    }
}

// --------------------------------------------------------------- effects

pub fn heart(cx: f32, cy: f32, s: f32) -> tiny_skia::Path {
    P::new()
        .m(cx, cy + s * 0.9)
        .c(cx - s * 1.6, cy - s * 0.1, cx - s * 0.9, cy - s * 1.3, cx, cy - s * 0.45)
        .c(cx + s * 0.9, cy - s * 1.3, cx + s * 1.6, cy - s * 0.1, cx, cy + s * 0.9)
        .z()
        .build()
}

fn sparkle(cx: f32, cy: f32, s: f32) -> tiny_skia::Path {
    P::new()
        .m(cx, cy - s)
        .q(cx, cy, cx + s, cy)
        .q(cx, cy, cx, cy + s)
        .q(cx, cy, cx - s, cy)
        .q(cx, cy, cx, cy - s)
        .z()
        .build()
}

fn draw_effects(cv: &mut Canvas, pose: &Pose) {
    let f = pose.frame;
    match pose.mood {
        Mood::Sleep => {
            let col = rgb(0x6f7fd8);
            for i in 0..3u32 {
                let q = ((f + i * 2) % 6) as f32 / 6.0;
                let (x, y, s) = (72.0 + q * 16.0, 30.0 - q * 22.0, 3.0 + q * 3.5);
                let z = P::new().m(x, y).l(x + s, y).l(x, y + s).l(x + s, y + s).build();
                cv.stroke(&z, with_alpha(col, 1.0 - q * 0.8), 0.6 + s * 0.22);
            }
        }
        Mood::Relax => {
            let col = rgb(0xff6f9c);
            for i in 0..2u32 {
                let q = ((f + i * 2) % 4) as f32 / 4.0;
                let (x, y) = (if i == 0 { 76.0 } else { 18.0 } + q * 5.0, 38.0 - q * 18.0);
                let c = with_alpha(col, 1.0 - q * 0.7);
                cv.with(Transform::from_rotate_at(-20.0, x, y), |cv| cv.fill(&ellipse(x, y, 2.4, 1.7), c));
                cv.stroke(
                    &P::new().m(x + 2.0, y - 0.4).l(x + 2.0, y - 8.0).q(x + 5.0, y - 6.5, x + 5.0, y - 3.5).build(),
                    c,
                    1.1,
                );
            }
        }
        Mood::Cheer => {
            let spots = [(14.0, 30.0), (86.0, 24.0), (18.0, 64.0), (84.0, 58.0), (50.0, 4.0)];
            for (i, (x, y)) in spots.iter().enumerate() {
                if (f as usize + i) % 3 != 0 {
                    let s = if (f as usize + i) % 2 == 0 { 4.5 } else { 3.0 };
                    let p = sparkle(*x, *y, s);
                    cv.fl(&p, rgb(0xffd84d), rgb(0xe0a400));
                }
            }
        }
        Mood::Love => {
            for i in 0..3u32 {
                let q = ((f + i * 3) % 8) as f32 / 8.0;
                let x = [72.0, 26.0, 82.0][i as usize] + (q * 6.0).sin() * 2.0;
                let y = [30.0, 26.0, 50.0][i as usize] - q * 18.0;
                let p = heart(x, y, 3.2 + q * 1.5);
                let c = with_alpha(rgb(0xff5c8a), 1.0 - q * 0.7);
                cv.fl(&p, c, with_alpha(rgb(0xc93a63), 1.0 - q * 0.7));
            }
        }
        Mood::Held => {
            let dy = if f % 2 == 0 { 0.0 } else { 1.5 };
            let (x, y) = (80.0, 26.0 + dy);
            let p = P::new().m(x, y - 3.5).q(x + 3.0, y + 1.0, x, y + 2.5).q(x - 3.0, y + 1.0, x, y - 3.5).z().build();
            cv.fl(&p, rgb(0x9fdcff), rgb(0x4a9fd8));
        }
        Mood::Idle | Mood::Focus => {}
    }
}
